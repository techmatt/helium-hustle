//! Plays one career: an agent for N retirements, in game time, measured tick
//! by tick (docs/simulator.md, *Measures*).
//!
//! The loop: at each look the agent reads the view and acts, then says how
//! many ticks to skip to its next look ([`crate::agents::Agent::next_look`]);
//! a forced retirement wakes it early. Results are ticks and decisions (looks
//! that acted); real time is a separate projection ([`crate::human`]). The
//! measuring is a [`Meter`], which `hh-sim playlog` also runs over a human's
//! career.

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use hh_core::modules::{boredom, economy, research};
use hh_core::{content, Action, Config, Scenario, Session};
use serde::Serialize;

use crate::agents::{self, Player};
use crate::human::{Clock, Human};
use crate::milestones::{capped, Hit, Milestones, Tracker, When};
use crate::objectives::{self, Objective};

/// One career to play.
pub struct Game<'a> {
    pub config: &'a Config,
    pub seed: u64,
    pub scenario: Option<&'a Scenario>,
    pub agent: &'a str,
    pub retirements: u32,
    pub max_ticks: u64,
    pub milestones: &'a Milestones,
    /// Optimizer-scenario objectives scored on run 1 (may be empty).
    pub objectives: &'a [Objective],
    /// Keep the agent's note at each look (`hh-sim trace`).
    pub trace: bool,
}

/// One finished run.
#[derive(Debug, Clone, Serialize)]
pub struct RunResult {
    pub run: u32,
    pub days: u64,
    pub forced: bool,
    /// Career ticks at its start and its end.
    pub start_tick: u64,
    pub end_tick: u64,
    /// Looks in it, those that acted (decisions), and actions applied.
    pub looks: u32,
    pub decisions: u32,
    pub actions: u32,
    /// Boredom from daily growth and from commands net of Dream, summed over
    /// the run's ticks.
    pub boredom_growth: f64,
    pub boredom_commands: f64,
    /// The run's totals as last seen before it ended.
    pub credits_earned: f64,
    pub shipments: u32,
    pub buildings_built: u32,
    pub research: u32,
    /// Its best credits earned a day over the milestones' longest window.
    pub best_credit_rate: f64,
}

/// The run's totals as last seen (a retirement resets them).
#[derive(Default, Clone, Copy)]
struct Totals {
    credits: f64,
    shipments: u32,
    buildings: u32,
    research: u32,
}

impl Totals {
    fn of(s: &Session) -> Totals {
        let st = &s.state.run.stats;
        Totals {
            credits: st.credits_earned.to_f64(),
            shipments: st.shipments,
            buildings: st.buildings_built,
            research: st.research_completed,
        }
    }
}

/// A building bought: career tick, run, building.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Build {
    pub tick: u64,
    pub run: u32,
    pub building: String,
}

/// A career's measures (`games.jsonl`, one line a game).
#[derive(Debug, Clone, Serialize)]
pub struct GameResult {
    pub seed: u64,
    pub agent: String,
    /// Career ticks played, from `start_tick`.
    pub ticks: u64,
    pub start_tick: u64,
    pub retirements: u32,
    pub looks: u64,
    pub decisions: u64,
    pub actions: u64,
    pub milestones: BTreeMap<String, Hit>,
    /// Run 1's day each objective of the spec's scenario file first held.
    pub objectives: BTreeMap<String, u64>,
    pub runs: Vec<RunResult>,
    /// The run under way at the end, if any (its totals so far; `forced` false).
    pub current: Option<RunResult>,
    /// Ticks with nothing to buy (no building, no research).
    pub idle_ticks: u64,
    /// Ticks each capped resource sat at its cap, of `ticks`.
    pub at_cap: BTreeMap<String, u64>,
    /// Production lost to caps, and production, by resource.
    pub overflow: BTreeMap<String, f64>,
    pub produced: BTreeMap<String, f64>,
    pub never_built: Vec<String>,
    pub never_used: Vec<String>,
    /// Every building bought, in order.
    pub builds: Vec<Build>,
    /// The career's long goals at the end: each axis's best rank, projects
    /// completed (Long-Term), quests and sub-objectives done, achievements.
    pub best_ranks: BTreeMap<String, i64>,
    pub projects: Vec<String>,
    pub quests: Vec<String>,
    pub sub_objectives: Vec<String>,
    pub achievements: Vec<String>,
    pub wall_s: f64,
    pub fingerprint: String,
    pub replay_ok: bool,
    /// A human's real seconds played (from the play log's clock); none for a
    /// sim player, whose real time is a projection ([`GameResult::clock`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub real_s: Option<f64>,
    /// The agent's notes by tick (traced games only).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<(u64, String)>,
    /// Each look: its tick and the actions it applied.
    #[serde(skip)]
    pub timeline: Vec<(u64, u32)>,
    /// A human's real clock (from the play log).
    #[serde(skip)]
    pub own_clock: Option<Clock>,
}

impl GameResult {
    /// Real seconds by career tick: a human's own clock, else `model`'s
    /// projection of the looks; none for a sim player without a model.
    pub fn clock(&self, model: Option<&Human>) -> Option<Clock> {
        if let Some(c) = &self.own_clock {
            return Some(c.clone());
        }
        model.map(|m| m.clock(self.start_tick, self.start_tick + self.ticks, &self.timeline))
    }

    /// Finished runs, and the run under way.
    pub fn all_runs(&self) -> impl Iterator<Item = &RunResult> {
        self.runs.iter().chain(self.current.iter())
    }
}

/// Whether anything is buyable now: a building or research.
fn something_to_buy(s: &Session) -> bool {
    let c = content();
    let (r, g) = (&s.rules, &s.state);
    c.buildings.iter().any(|b| economy::buy_blocker(r, g, b).is_none())
        || c.research
            .iter()
            .any(|x| !g.run.research.contains(&x.id) && research::visible(g, x) && research::blocker(r, g, x).is_none())
}

/// A career's measures, taken as it plays: call [`Meter::look`] after each
/// round of actions and [`Meter::tick`] to play each tick, then
/// [`Meter::finish`].
pub struct Meter<'a> {
    milestones: &'a Milestones,
    objectives: &'a [Objective],
    tracker: Tracker,
    hits: BTreeMap<String, Hit>,
    idle: u64,
    at_cap: BTreeMap<String, u64>,
    overflow: BTreeMap<String, f64>,
    produced: BTreeMap<String, f64>,
    runs: Vec<RunResult>,
    growth: f64,
    cmd: f64,
    last_run: u32,
    start_tick: u64,
    run_start: u64,
    looks: u32,
    decisions: u32,
    actions: u32,
    timeline: Vec<(u64, u32)>,
    totals: Totals,
    objectives_hit: BTreeMap<String, u64>,
}

impl<'a> Meter<'a> {
    pub fn new(s: &Session, milestones: &'a Milestones, objectives: &'a [Objective]) -> Meter<'a> {
        let mut tracker = Tracker::new(milestones);
        tracker.after(s);
        Meter {
            milestones,
            objectives,
            tracker,
            hits: BTreeMap::new(),
            idle: 0,
            at_cap: BTreeMap::new(),
            overflow: BTreeMap::new(),
            produced: BTreeMap::new(),
            runs: Vec::new(),
            growth: 0.0,
            cmd: 0.0,
            last_run: s.state.career.run_number,
            start_tick: s.state.tick,
            run_start: s.state.tick,
            looks: 0,
            decisions: 0,
            actions: 0,
            timeline: Vec::new(),
            totals: Totals::of(s),
            objectives_hit: BTreeMap::new(),
        }
    }

    /// Records each milestone met now. `ended`: the run a retirement closed
    /// in this step, if any. A milestone first seen then counts in that run
    /// (on its last day): a retirement, or a career-wide condition (a quest,
    /// lifetime shipments, a command used) the run's last tick met. A
    /// run-scoped one the new run already reset (a building count, a launch on
    /// that tick) is not seen (docs/principles.md).
    fn check(&mut self, s: &Session, ended: Option<&RunResult>) {
        for m in &self.milestones.milestones {
            if !self.hits.contains_key(&m.id) && self.tracker.met(&m.when, s) {
                let g = &s.state;
                let (run, day) = match (&m.when, ended) {
                    (_, Some(r)) => (r.run, r.days),
                    (When::Retirements { .. }, None) => {
                        let e = g.career.log.iter().rev().find(|e| e.kind == "retired" || e.kind == "forced_retirement");
                        e.map_or((g.career.run_number, g.run.day), |e| (e.run, e.day))
                    }
                    _ => (g.career.run_number, g.run.day),
                };
                self.hits.insert(m.id.clone(), Hit { tick: g.tick, run, day });
            }
        }
    }

    /// The run under way, as it stands (`days` from the state, not a log entry).
    fn run_so_far(&mut self, s: &Session, run: u32, days: u64, forced: bool, best: f64) -> RunResult {
        let t = &self.totals;
        RunResult {
            run,
            days,
            forced,
            start_tick: self.run_start,
            end_tick: s.state.tick,
            looks: self.looks,
            decisions: self.decisions,
            actions: self.actions,
            boredom_growth: self.growth,
            boredom_commands: self.cmd,
            credits_earned: t.credits,
            shipments: t.shipments,
            buildings_built: t.buildings,
            research: t.research,
            best_credit_rate: best,
        }
    }

    /// A retirement (forced in a tick, or chosen at a look) closes a run.
    fn close(&mut self, s: &Session) -> Option<RunResult> {
        let e = s.state.career.log.iter().rev().find(|e| e.kind == "retired" || e.kind == "forced_retirement").cloned();
        let best = self.tracker.take_best_rate().to_f64();
        if let Some(e) = e {
            let r = self.run_so_far(s, e.run, e.day, e.kind == "forced_retirement", best);
            self.runs.push(r);
        }
        self.growth = 0.0;
        self.cmd = 0.0;
        self.looks = 0;
        self.decisions = 0;
        self.actions = 0;
        self.run_start = s.state.tick;
        self.last_run = s.state.career.run_number;
        self.runs.last().cloned()
    }

    /// After a round of `actions` actions between ticks (0: a look that
    /// changed nothing). A look that retired counts in the new run: what it
    /// did after the retirement (the new run's opening) is most of it.
    pub fn look(&mut self, s: &Session, actions: u32) {
        let mut ended = None;
        if s.state.career.run_number != self.last_run {
            ended = self.close(s);
        }
        self.looks += 1;
        self.actions += actions;
        if actions > 0 {
            self.decisions += 1;
        }
        self.timeline.push((s.state.tick, actions));
        self.totals = Totals::of(s);
        self.tracker.after(s);
        self.check(s, ended.as_ref());
    }

    /// Plays one tick and measures it.
    pub fn tick(&mut self, s: &mut Session) -> Result<(), String> {
        if !something_to_buy(s) {
            self.idle += 1;
        }
        let b0 = s.state.amount(economy::BOREDOM);
        let rate = boredom::rate(&s.rules, &s.state);
        self.tracker.before(s);
        s.advance(1);
        let mut ended = None;
        if s.state.career.run_number == self.last_run {
            let b1 = s.state.amount(economy::BOREDOM);
            self.growth += rate.to_f64();
            self.cmd += (b1 - b0 - rate).to_f64();
        } else {
            self.growth += rate.to_f64();
            ended = self.close(s);
        }
        self.totals = Totals::of(s);
        if s.state.career.run_number == 1 {
            for o in self.objectives {
                if !self.objectives_hit.contains_key(&o.id) && objectives::holds(o, s)? {
                    self.objectives_hit.insert(o.id.clone(), s.state.run.day);
                }
            }
        }
        let today = &s.state.run.today;
        for (res, v) in &today.overflow {
            *self.overflow.entry(res.clone()).or_default() += v.to_f64();
        }
        for (res, v) in &today.produced {
            *self.produced.entry(res.clone()).or_default() += v.to_f64();
        }
        for (res, c) in capped(&s.rules, s) {
            if s.state.amount(&res) >= c {
                *self.at_cap.entry(res).or_default() += 1;
            }
        }
        self.tracker.after(s);
        self.check(s, ended.as_ref());
        Ok(())
    }

    /// The career's measures. `replay_ok`: its record replays to the same state.
    pub fn finish(mut self, s: &Session, seed: u64, agent: &str, notes: Vec<(u64, String)>, wall_s: f64) -> GameResult {
        let rec = s.record();
        let replay_ok = rec.replay().map(|(g, w)| g == s.state && w.is_empty()).unwrap_or(false);
        let c = content();
        let built: &BTreeSet<String> = &s.state.career.owned_buildings;
        let used: &BTreeSet<String> = &s.state.career.used_commands;
        // Each buy's run: a forced retirement ends its run before that tick's
        // actions; a chosen one at its Retire action (a buy before it, at
        // the same tick, is the old run's).
        let mut builds = Vec::new();
        let mut done = 0usize;
        for a in &rec.actions {
            while done < self.runs.len() && self.runs[done].forced && self.runs[done].end_tick <= a.tick {
                done += 1;
            }
            match &a.action {
                Action::Buy { building } => {
                    builds.push(Build { tick: a.tick, run: done as u32 + 1, building: building.clone() });
                }
                Action::Retire if done < self.runs.len() && !self.runs[done].forced => done += 1,
                _ => {}
            }
        }
        let current = (s.state.tick > self.run_start || self.looks > 0).then(|| {
            let best = self.tracker.take_best_rate().to_f64();
            self.run_so_far(s, s.state.career.run_number, s.state.run.day, false, best)
        });
        let looks: u64 = self.timeline.len() as u64;
        let decisions = self.timeline.iter().filter(|l| l.1 > 0).count() as u64;
        GameResult {
            seed,
            agent: agent.to_string(),
            ticks: s.state.tick - self.start_tick,
            start_tick: self.start_tick,
            retirements: s.state.career.retirements,
            looks,
            decisions,
            actions: rec.actions.len() as u64,
            milestones: self.hits,
            objectives: self.objectives_hit,
            runs: self.runs,
            current,
            idle_ticks: self.idle,
            at_cap: self.at_cap,
            overflow: self.overflow,
            produced: self.produced,
            never_built: c.buildings.iter().filter(|b| !built.contains(&b.id)).map(|b| b.id.clone()).collect(),
            never_used: c.commands.iter().filter(|x| !used.contains(&x.id)).map(|x| x.id.clone()).collect(),
            builds,
            best_ranks: s.state.career.max_ideology_ranks.clone(),
            projects: s.state.career.completed_projects.iter().cloned().collect(),
            quests: s.state.career.completed_quests.iter().cloned().collect(),
            sub_objectives: s.state.career.completed_sub_objectives.iter().cloned().collect(),
            achievements: s.state.career.achievements.iter().cloned().collect(),
            wall_s,
            fingerprint: s.state.fingerprint(),
            replay_ok,
            real_s: None,
            notes,
            timeline: self.timeline,
            own_clock: None,
        }
    }
}

pub fn play(game: &Game) -> Result<GameResult, String> {
    play_session(game).map(|r| r.0)
}

/// [`play`], also returning the session (for `hh-sim trace`).
pub fn play_session(game: &Game) -> Result<(GameResult, Session), String> {
    let start = Instant::now();
    let mut s = Session::new(game.config, game.seed, game.scenario)?;
    let mut agent = agents::build(game.agent, game.seed)?;
    let mut meter = Meter::new(&s, game.milestones, game.objectives);
    let mut seen_run = 0u32;
    let max_tick = s.state.tick.saturating_add(game.max_ticks);
    let mut notes: Vec<(u64, String)> = Vec::new();
    'career: while s.state.career.retirements < game.retirements && s.state.tick < max_tick {
        let (applied, gap) = {
            let mut p = Player::new(&mut s);
            let run = p.view().run;
            if run != seen_run {
                if seen_run != 0 {
                    agent.new_run(&mut p);
                }
                seen_run = run;
            }
            agent.act(&mut p);
            if game.trace {
                let n = agent.note();
                if !n.is_empty() {
                    notes.push((p.view().tick, n));
                }
            }
            // A retirement the agent chose: its new run's hook comes at once.
            let run = p.view().run;
            if run != seen_run {
                agent.new_run(&mut p);
                seen_run = run;
            }
            let gap = agent.next_look(&mut p).max(1);
            (p.applied, gap)
        };
        meter.look(&s, applied);
        if s.state.career.retirements >= game.retirements {
            break;
        }
        let run = s.state.career.run_number;
        for _ in 0..gap {
            meter.tick(&mut s)?;
            if s.state.career.retirements >= game.retirements || s.state.tick >= max_tick {
                break 'career;
            }
            // A forced retirement wakes the player.
            if s.state.career.run_number != run {
                break;
            }
        }
    }
    let result = meter.finish(&s, game.seed, game.agent, notes, start.elapsed().as_secs_f64());
    Ok((result, s))
}
