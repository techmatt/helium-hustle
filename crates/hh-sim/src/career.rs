//! Plays one career: an agent under a profile (attention × speed) for N
//! retirements, measured tick by tick (docs/simulator.md, *Measures*).
//!
//! The loop: at each look the agent reads the view and acts; the speed policy
//! sets the speed from what happened; the game then runs `attention × speed`
//! ticks before the next look. Real time is ticks over speed.

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use hh_core::modules::{boredom, economy, research};
use hh_core::{content, Config, Scenario, Session};
use serde::Serialize;

use crate::agents::{self, Player};
use crate::milestones::{capped, Hit, Milestones, Tracker, When};
use crate::objectives::{self, Objective};
use crate::profile::{Attention, Speed};

/// One career to play.
pub struct Game<'a> {
    pub config: &'a Config,
    pub seed: u64,
    pub scenario: Option<&'a Scenario>,
    pub agent: &'a str,
    pub attention: &'a Attention,
    pub speed: &'a Speed,
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
    pub real_s: f64,
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

/// A career's measures (`games.jsonl`, one line a game).
#[derive(Debug, Clone, Serialize)]
pub struct GameResult {
    pub seed: u64,
    pub agent: String,
    pub ticks: u64,
    pub real_s: f64,
    pub retirements: u32,
    pub looks: u64,
    pub actions: u64,
    pub milestones: BTreeMap<String, Hit>,
    /// Run 1's day each objective of the spec's scenario file first held.
    pub objectives: BTreeMap<String, u64>,
    pub runs: Vec<RunResult>,
    /// Real seconds with nothing to buy (no building, no research).
    pub idle_real_s: f64,
    /// Ticks each capped resource sat at its cap, of `ticks`.
    pub at_cap: BTreeMap<String, u64>,
    /// Production lost to caps, and production, by resource.
    pub overflow: BTreeMap<String, f64>,
    pub produced: BTreeMap<String, f64>,
    pub never_built: Vec<String>,
    pub never_used: Vec<String>,
    pub wall_s: f64,
    pub fingerprint: String,
    pub replay_ok: bool,
    /// The agent's notes by tick (traced games only).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<(u64, String)>,
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

pub fn play(game: &Game) -> Result<GameResult, String> {
    play_session(game).map(|r| r.0)
}

/// [`play`], also returning the session (for `hh-sim trace`).
pub fn play_session(game: &Game) -> Result<(GameResult, Session), String> {
    let start = Instant::now();
    let mut s = Session::new(game.config, game.seed, game.scenario)?;
    let mut agent = agents::build(game.agent, game.seed)?;
    let mut tracker = Tracker::new(game.milestones);
    tracker.after(&s);
    let mut hits: BTreeMap<String, Hit> = BTreeMap::new();
    let mut real = 0.0f64;
    let mut idle = 0.0f64;
    let mut looks = 0u64;
    let mut at_cap: BTreeMap<String, u64> = BTreeMap::new();
    let mut overflow: BTreeMap<String, f64> = BTreeMap::new();
    let mut produced: BTreeMap<String, f64> = BTreeMap::new();
    let mut runs: Vec<RunResult> = Vec::new();
    let mut run_real = 0.0f64;
    let (mut growth, mut cmd) = (0.0f64, 0.0f64);
    let mut last_run = s.state.career.run_number;
    let mut seen_run = 0u32;
    let start_tick = s.state.tick;
    // `ended`: the run a retirement closed in this step, if any. A milestone
    // first seen then counts in that run (on its last day): a retirement, or
    // a career-wide condition (a quest, lifetime shipments, a command used)
    // the run's last tick met. A run-scoped one the new run already reset (a
    // building count, a launch on that tick) is not seen (docs/principles.md).
    let check = |hits: &mut BTreeMap<String, Hit>, tracker: &Tracker, s: &Session, real: f64, ended: Option<&RunResult>| {
        for m in &game.milestones.milestones {
            if !hits.contains_key(&m.id) && tracker.met(&m.when, s) {
                let g = &s.state;
                let (run, day) = match (&m.when, ended) {
                    (_, Some(r)) => (r.run, r.days),
                    (When::Retirements { .. }, None) => {
                        let e = g.career.log.iter().rev().find(|e| e.kind == "retired" || e.kind == "forced_retirement");
                        e.map_or((g.career.run_number, g.run.day), |e| (e.run, e.day))
                    }
                    _ => (g.career.run_number, g.run.day),
                };
                hits.insert(m.id.clone(), Hit { tick: g.tick, run, day, real_s: real });
            }
        }
    };
    // A retirement (forced in a tick, or chosen at a look) closes a run.
    let close = |runs: &mut Vec<RunResult>,
                 s: &Session,
                 tracker: &mut Tracker,
                 t: &Totals,
                 run_real: &mut f64,
                 growth: &mut f64,
                 cmd: &mut f64| {
        let e = s.state.career.log.iter().rev().find(|e| e.kind == "retired" || e.kind == "forced_retirement");
        let best = tracker.take_best_rate().to_f64();
        if let Some(e) = e {
            runs.push(RunResult {
                run: e.run,
                days: e.day,
                forced: e.kind == "forced_retirement",
                real_s: *run_real,
                boredom_growth: *growth,
                boredom_commands: *cmd,
                credits_earned: t.credits,
                shipments: t.shipments,
                buildings_built: t.buildings,
                research: t.research,
                best_credit_rate: best,
            });
        }
        *run_real = 0.0;
        *growth = 0.0;
        *cmd = 0.0;
    };
    let max_tick = start_tick.saturating_add(game.max_ticks);
    let mut totals = Totals::of(&s);
    let mut objectives_hit: BTreeMap<String, u64> = BTreeMap::new();
    let mut notes: Vec<(u64, String)> = Vec::new();
    'career: while s.state.career.retirements < game.retirements && s.state.tick < max_tick {
        looks += 1;
        let acted = {
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
            p.applied > 0
        };
        let mut ended = None;
        if s.state.career.run_number != last_run {
            close(&mut runs, &s, &mut tracker, &totals, &mut run_real, &mut growth, &mut cmd);
            last_run = s.state.career.run_number;
            ended = runs.last().cloned();
        }
        totals = Totals::of(&s);
        tracker.after(&s);
        check(&mut hits, &tracker, &s, real, ended.as_ref());
        if s.state.career.retirements >= game.retirements {
            break;
        }
        let speed = game.speed.at(acted, s.state.career.retirements);
        let ticks = game.attention.seconds * speed;
        let dt = 1.0 / speed as f64;
        for _ in 0..ticks {
            if !something_to_buy(&s) {
                idle += dt;
            }
            let b0 = s.state.amount(economy::BOREDOM);
            let rate = boredom::rate(&s.rules, &s.state);
            tracker.before(&s);
            s.advance(1);
            real += dt;
            run_real += dt;
            let mut ended = None;
            if s.state.career.run_number == last_run {
                let b1 = s.state.amount(economy::BOREDOM);
                growth += rate.to_f64();
                cmd += (b1 - b0 - rate).to_f64();
            } else {
                growth += rate.to_f64();
                close(&mut runs, &s, &mut tracker, &totals, &mut run_real, &mut growth, &mut cmd);
                last_run = s.state.career.run_number;
                ended = runs.last().cloned();
            }
            totals = Totals::of(&s);
            if s.state.career.run_number == 1 {
                for o in game.objectives {
                    if !objectives_hit.contains_key(&o.id) && objectives::holds(o, &s)? {
                        objectives_hit.insert(o.id.clone(), s.state.run.day);
                    }
                }
            }
            let today = &s.state.run.today;
            for (res, v) in &today.overflow {
                *overflow.entry(res.clone()).or_default() += v.to_f64();
            }
            for (res, v) in &today.produced {
                *produced.entry(res.clone()).or_default() += v.to_f64();
            }
            for (res, c) in capped(&s.rules, &s) {
                if s.state.amount(&res) >= c {
                    *at_cap.entry(res).or_default() += 1;
                }
            }
            tracker.after(&s);
            check(&mut hits, &tracker, &s, real, ended.as_ref());
            if s.state.career.retirements >= game.retirements || s.state.tick >= max_tick {
                break 'career;
            }
        }
    }
    let rec = s.record();
    let replay_ok = rec.replay().map(|(g, w)| g == s.state && w.is_empty()).unwrap_or(false);
    let c = content();
    let built: &BTreeSet<String> = &s.state.career.owned_buildings;
    let used: &BTreeSet<String> = &s.state.career.used_commands;
    Ok((GameResult {
        seed: game.seed,
        agent: game.agent.to_string(),
        ticks: s.state.tick - start_tick,
        real_s: real,
        retirements: s.state.career.retirements,
        looks,
        actions: rec.actions.len() as u64,
        milestones: hits,
        objectives: objectives_hit,
        runs,
        idle_real_s: idle,
        at_cap,
        overflow,
        produced,
        never_built: c.buildings.iter().filter(|b| !built.contains(&b.id)).map(|b| b.id.clone()).collect(),
        never_used: c.commands.iter().filter(|x| !used.contains(&x.id)).map(|x| x.id.clone()).collect(),
        wall_s: start.elapsed().as_secs_f64(),
        fingerprint: s.state.fingerprint(),
        replay_ok,
        notes,
    }, s))
}
