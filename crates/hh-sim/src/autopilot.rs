//! The debug autopilot (an accelerator: docs/simulator.md, *Accelerators*): a
//! sim player plays a career on from its current state, honestly, until a
//! target is reached: the next milestone, the next main-quest step, or every
//! milestone and main-quest step there is. Targets come from the data (the
//! milestones file, the `story` events in `content/events.json` and their
//! sub-objectives), so new acts need no code here unless they add a new kind
//! of condition. Nothing is faked: the player only acts through the view and
//! actions, and the engine ticks. The web debug menu drives it through
//! `hh-wasm`; `hh-sim accelerate` drives it natively. No clock: it runs in
//! wasm, where `Instant` panics.
//!
//! The sim agents, the ladder, the judge and the reports never build one.

use std::collections::BTreeSet;

use hh_core::content::{content, Condition};
use hh_core::modules::events::sub_key;
use hh_core::{Action, Boost, Cheat, GameState, Session, Stamped};
use serde::Serialize;

use crate::agents::{self, Agent, Player};
use crate::milestones::{Milestones, Tracker};

/// The default drivers, `|`-separated: the autopilot plays the first and moves
/// to the next (round the list) once [`STALL`] ticks pass with no step met.
/// Both are the author, kept on in a run while it lasts (the Telescope wants
/// long runs) and dreaming from the first boredom (the boost's slow boredom
/// never reaches its usual threshold); the first ships every good made
/// (`diversify`), the second doesn't, and buys all visible research
/// (`all_research`: past [`author::RESEARCH`](crate::agents::author::RESEARCH),
/// today Process Telemetry, from 1.6, and the draft Novelty Simulation, after 1.9).
pub const DEFAULT_AGENT: &str =
    "author:stale_days=4000,dream_at=1,diversify=1|author:stale_days=4000,dream_at=1,all_research=1";

/// Ticks with no step met before the autopilot changes driver (4,000 cut
/// a global project short on one seed in six; 8,000 met every step on six).
pub const STALL: u64 = 8000;

/// One thing to reach: a milestone, a main-quest sub-objective or a main quest.
#[derive(Debug, Clone, Serialize)]
pub struct Step {
    /// `milestone:M5`, `quest:qlab`, `quest:qthree_voices:humanist_5`.
    pub id: String,
    pub name: String,
}

/// Every step there is, milestones first (file order), then the main quests
/// in chain order, each quest's sub-objectives before it. A quest that can
/// never complete (condition `never`: the next act's hook) is left out.
pub fn steps(ms: &Milestones) -> Vec<Step> {
    let mut v: Vec<Step> = ms
        .milestones
        .iter()
        .map(|m| Step { id: format!("milestone:{}", m.id), name: format!("{} {}", m.id, m.name) })
        .collect();
    // Side quests (1.a, 1.b) are steps too, in file order.
    for e in content().events.iter().filter(|e| e.is_quest() || e.is_side()) {
        match &e.condition {
            Condition::Never => continue,
            Condition::AllOf { sub_objectives } => {
                for s in sub_objectives {
                    v.push(Step { id: format!("quest:{}", sub_key(&e.id, &s.id)), name: format!("{}: {}", e.title, s.label) });
                }
            }
            _ => {}
        }
        v.push(Step { id: format!("quest:{}", e.id), name: e.title.clone() });
    }
    v
}

/// What to play toward.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// The first milestone not met yet.
    Milestone,
    /// The first main-quest step not met yet.
    Quest,
    /// Every step.
    End,
}

impl Target {
    pub fn parse(s: &str) -> Result<Target, String> {
        match s {
            "milestone" => Ok(Target::Milestone),
            "quest" => Ok(Target::Quest),
            "end" => Ok(Target::End),
            _ => Err(format!("autopilot: unknown target {s:?} (milestone, quest, end)")),
        }
    }
}

/// A step met while the autopilot played.
#[derive(Debug, Clone, Serialize)]
pub struct Hit {
    pub id: String,
    pub name: String,
    pub tick: u64,
    pub run: u32,
}

/// Where a run stands after a [`Autopilot::step`].
#[derive(Debug, Clone, Serialize)]
pub struct Status {
    /// The steps aimed at (one, or every step left for `end`), by id.
    pub aim: Vec<String>,
    /// Every aimed step met.
    pub reached: bool,
    /// The tick budget ran out first.
    pub gave_up: bool,
    pub ticks_played: u64,
    /// The driver playing now.
    pub driver: String,
    pub tick: u64,
    pub run: u32,
    /// Steps first met during this autopilot run, in order.
    pub hits: Vec<Hit>,
    /// Steps still unmet (any target), by display name.
    pub left: Vec<String>,
}

pub struct Autopilot {
    s: Session,
    agent: Box<dyn Agent>,
    /// The driver specs, and which one plays.
    specs: Vec<String>,
    driver: usize,
    /// `played` when the last step was met or the driver changed.
    since: u64,
    ms: Milestones,
    tracker: Tracker,
    steps: Vec<Step>,
    met: BTreeSet<String>,
    aim: Vec<String>,
    hits: Vec<Hit>,
    seen_run: u32,
    /// Ticks left before the next look.
    gap: u64,
    played: u64,
    budget: u64,
}

impl Autopilot {
    /// Takes over `state`: applies the boost (when given) and the autopilot's
    /// marker, both cheats (so the game is marked accelerated), then aims at
    /// `target`. `budget` caps the ticks it plays before it gives up. `known`:
    /// step ids met earlier in the career (a run-scoped milestone, such as a
    /// building count, may not hold in the state now), counted as met.
    pub fn new(
        state: GameState,
        ms: Milestones,
        agent_spec: &str,
        target: Target,
        boost: Option<Boost>,
        budget: u64,
        known: &[String],
    ) -> Result<Autopilot, String> {
        let s = Session::resume(state)?;
        let specs: Vec<String> = agent_spec.split('|').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
        if specs.is_empty() {
            return Err("autopilot: no driver".into());
        }
        for spec in &specs {
            agents::build(spec, s.state.seed, &ms)?;
        }
        let agent = agents::build(&specs[0], s.state.seed, &ms)?;
        let steps = steps(&ms);
        let mut tracker = Tracker::new(&ms);
        tracker.before(&s);
        tracker.after(&s);
        let mut ap = Autopilot {
            seen_run: s.state.career.run_number,
            s,
            agent,
            specs,
            driver: 0,
            since: 0,
            ms,
            tracker,
            steps,
            met: BTreeSet::new(),
            aim: Vec::new(),
            hits: Vec::new(),
            gap: 0,
            played: 0,
            budget,
        };
        ap.met = ap.met_now();
        ap.met.extend(known.iter().filter(|k| ap.steps.iter().any(|x| x.id == **k)).cloned());
        let open: Vec<&Step> = ap.steps.iter().filter(|x| !ap.met.contains(&x.id)).collect();
        ap.aim = match target {
            Target::Milestone => open.iter().find(|x| x.id.starts_with("milestone:")).map(|x| vec![x.id.clone()]),
            Target::Quest => open.iter().find(|x| x.id.starts_with("quest:")).map(|x| vec![x.id.clone()]),
            Target::End => Some(open.iter().map(|x| x.id.clone()).collect()),
        }
        .unwrap_or_default();
        if let Some(b) = boost {
            let a = Cheat::Boost { production: b.production, research: b.research, boredom: b.boredom };
            ap.s.apply(&Action::Cheat { cheat: a })?;
        }
        let name = match target {
            Target::Milestone => "milestone",
            Target::Quest => "quest",
            Target::End => "end",
        };
        ap.s.apply(&Action::Cheat { cheat: Cheat::Autopilot { target: name.to_string() } })?;
        Ok(ap)
    }

    /// Steps met in the state now (milestones under this run's tracker).
    fn met_now(&self) -> BTreeSet<String> {
        let g = &self.s.state;
        let mut met = BTreeSet::new();
        for m in &self.ms.milestones {
            if self.tracker.met(&m.when, &self.s) {
                met.insert(format!("milestone:{}", m.id));
            }
        }
        for q in &g.career.completed_quests {
            met.insert(format!("quest:{q}"));
        }
        for q in &g.career.completed_sub_objectives {
            met.insert(format!("quest:{q}"));
        }
        met
    }

    fn note_hits(&mut self) {
        for id in self.met_now() {
            if self.met.insert(id.clone()) {
                if let Some(st) = self.steps.iter().find(|x| x.id == id) {
                    self.since = self.played;
                    self.hits.push(Hit {
                        id,
                        name: st.name.clone(),
                        tick: self.s.state.tick,
                        run: self.s.state.career.run_number,
                    });
                }
            }
        }
    }

    pub fn reached(&self) -> bool {
        self.aim.iter().all(|a| self.met.contains(a))
    }

    /// Plays up to `ticks` more ticks (looks included), stopping early once the
    /// target is reached or the budget is spent.
    pub fn step(&mut self, ticks: u64) -> Status {
        let mut left = ticks;
        while left > 0 && !self.reached() && self.played < self.budget {
            // Stalled: the next driver takes over at once (a fresh player, as a
            // new seat at the same game).
            if self.specs.len() > 1 && self.played - self.since >= STALL {
                self.driver = (self.driver + 1) % self.specs.len();
                let spec = &self.specs[self.driver];
                self.agent = agents::build(spec, self.s.state.seed, &self.ms).expect("checked in new");
                self.since = self.played;
                self.gap = 0;
            }
            if self.gap == 0 {
                let mut p = Player::new(&mut self.s);
                let run = p.view().run;
                if run != self.seen_run {
                    self.agent.new_run(&mut p);
                    self.seen_run = run;
                }
                self.agent.act(&mut p);
                let run = p.view().run;
                if run != self.seen_run {
                    self.agent.new_run(&mut p);
                    self.seen_run = run;
                }
                self.gap = self.agent.next_look(&mut p).max(1);
                self.tracker.after(&self.s);
                self.note_hits();
                if self.reached() {
                    break;
                }
            }
            let run = self.s.state.career.run_number;
            self.tracker.before(&self.s);
            self.s.advance(1);
            self.tracker.after(&self.s);
            self.note_hits();
            self.played += 1;
            left -= 1;
            self.gap -= 1;
            // A forced retirement wakes the player.
            if self.s.state.career.run_number != run {
                self.gap = 0;
            }
        }
        self.status()
    }

    pub fn status(&self) -> Status {
        Status {
            aim: self.aim.clone(),
            reached: self.reached(),
            gave_up: !self.reached() && self.played >= self.budget,
            ticks_played: self.played,
            driver: self.specs[self.driver].clone(),
            tick: self.s.state.tick,
            run: self.s.state.career.run_number,
            hits: self.hits.clone(),
            left: self.steps.iter().filter(|x| !self.met.contains(&x.id)).map(|x| x.name.clone()).collect(),
        }
    }

    pub fn state(&self) -> &GameState {
        &self.s.state
    }

    /// Every action applied since the autopilot took over (its two cheats first).
    pub fn actions(&self) -> &[Stamped] {
        self.s.actions()
    }
}

/// The default boost (`production,research,boredom`) of the web menu and
/// `hh-sim accelerate`: debug-only, so a number here, not a config.
pub const DEFAULT_BOOST: &str = "10,10,0.1";

pub fn parse_boost(s: &str) -> Result<Boost, String> {
    let v: Vec<&str> = s.split(',').collect();
    let [p, r, b] = v.as_slice() else {
        return Err(format!("boost {s:?}: expected production,research,boredom"));
    };
    let f = |x: &str| hh_core::Fx::parse(x).map_err(|e| format!("boost {s:?}: {e}"));
    Ok(Boost { production: f(p)?, research: f(r)?, boredom: f(b)? })
}

/// `hh-sim accelerate`: a fresh career on the autopilot, every step's tick
/// and the wall time.
pub fn cli(args: &[String]) -> Result<String, crate::Fail> {
    use crate::{read, usage, Fail};
    let mut config = crate::BASELINE.to_string();
    let mut seed = 0u64;
    let mut agent = DEFAULT_AGENT.to_string();
    let mut boost = Some(DEFAULT_BOOST.to_string());
    let mut target = "end".to_string();
    let mut budget = 1_000_000u64;
    let mut save = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = || it.next().cloned().ok_or_else(|| Fail::Usage(format!("{a} needs a value")));
        match a.as_str() {
            "--config" => config = val()?,
            "--seed" => seed = val()?.parse().map_err(|_| Fail::Usage("bad --seed".into()))?,
            "--agent" => agent = val()?,
            "--boost" => boost = Some(val()?),
            "--no-boost" => boost = None,
            "--target" => target = val()?,
            "--budget" => budget = val()?.parse().map_err(|_| Fail::Usage("bad --budget".into()))?,
            "--save" => save = Some(val()?),
            s => return usage(format!("accelerate: unknown argument {s}")),
        }
    }
    let c = hh_core::Config::from_json(&read(&config)?).map_err(Fail::Usage)?;
    let ms = Milestones::from_json(&read(crate::MILESTONES)?).map_err(Fail::Usage)?;
    let boost = boost.map(|b| parse_boost(&b)).transpose().map_err(Fail::Usage)?;
    let target = Target::parse(&target).map_err(Fail::Usage)?;
    let start = std::time::Instant::now();
    let s = Session::new(&c, seed, None).map_err(Fail::Usage)?;
    let mut ap = Autopilot::new(s.state, ms, &agent, target, boost, budget, &[]).map_err(Fail::Usage)?;
    let st = ap.step(u64::MAX);
    let secs = start.elapsed().as_secs_f64();
    let mut out = String::new();
    use std::fmt::Write as _;
    for h in &st.hits {
        let _ = writeln!(out, "t{:>7} r{:<3} {}", h.tick, h.run, h.name);
    }
    let _ = writeln!(
        out,
        "{}: {} ticks in {secs:.1} s ({} actions); left: {}",
        if st.reached { "reached" } else { "NOT reached" },
        st.ticks_played,
        ap.actions().len(),
        if st.left.is_empty() { "none".to_string() } else { st.left.join("; ") }
    );
    if let Some(path) = save {
        let mut rec = s_record(&c, seed, ap.actions(), ap.state());
        rec.fingerprint = Some(ap.state().fingerprint());
        std::fs::write(&path, rec.to_json() + "\n").map_err(|e| Fail::Usage(format!("{path}: {e}")))?;
    }
    if st.reached {
        Ok(out)
    } else {
        Err(Fail::Check(out))
    }
}

/// A fresh game's record (no scenario) with `actions`, left at `g`'s tick.
fn s_record(c: &hh_core::Config, seed: u64, actions: &[Stamped], g: &GameState) -> hh_core::Record {
    hh_core::Record {
        format: hh_core::record::RECORD_FORMAT.to_string(),
        rules_revision: hh_core::RULES_REVISION,
        config: c.clone(),
        seed,
        scenario: None,
        actions: actions.to_vec(),
        ticks: g.tick,
        fingerprint: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh(seed: u64) -> (hh_core::Config, Milestones, GameState) {
        let c = hh_core::Config::from_json(include_str!("../../../configs/baseline.json")).unwrap();
        let ms = Milestones::from_json(include_str!("../../../configs/sim/milestones.json")).unwrap();
        let g = Session::new(&c, seed, None).unwrap().state;
        (c, ms, g)
    }

    #[test]
    fn steps_come_from_the_data_and_leave_out_never_quests() {
        let (_, ms, _) = fresh(0);
        let st = steps(&ms);
        assert_eq!(st.iter().filter(|s| s.id.starts_with("milestone:")).count(), ms.milestones.len());
        assert!(st.iter().any(|s| s.id == "quest:qpower_boot_sequence"));
        assert!(st.iter().any(|s| s.id == "quest:qthree_voices:humanist_5"));
        assert!(st.iter().any(|s| s.id == "quest:qbig_launch"), "side quests are steps");
        assert!(st.iter().any(|s| s.id == "quest:qoverload"), "the later acts' quests are steps");
        assert!(!st.iter().any(|s| s.id == "quest:qepilogue"), "a `never` quest is no step");
    }

    #[test]
    fn the_next_steps_are_reached_honestly_and_the_record_replays() {
        let (c, ms, g) = fresh(3);
        let boost = Some(parse_boost(DEFAULT_BOOST).unwrap());
        let mut ap = Autopilot::new(g.clone(), ms.clone(), DEFAULT_AGENT, Target::Quest, boost, 5000, &[]).unwrap();
        let st = ap.step(u64::MAX);
        assert!(st.reached, "{st:?}");
        assert_eq!(st.aim, vec!["quest:qpower_boot_sequence".to_string()]);
        assert!(ap.state().accelerated && ap.state().boost.is_some());
        // The next milestone from there, without a boost: the first one not met
        // (M1 met by the first autopilot, M2 told as met before: M3).
        let known: Vec<String> = st.hits.iter().map(|h| h.id.clone()).chain(["milestone:M2".to_string()]).collect();
        assert!(known.contains(&"milestone:M1".to_string()), "{known:?}");
        let mut next = Autopilot::new(ap.state().clone(), ms, DEFAULT_AGENT, Target::Milestone, None, 5000, &known).unwrap();
        let st = next.step(u64::MAX);
        assert!(st.reached);
        assert_eq!(st.aim, vec!["milestone:M3".to_string()]);
        let mut actions = ap.actions().to_vec();
        actions.extend_from_slice(next.actions());
        assert!(matches!(actions[0].action, Action::Cheat { cheat: Cheat::Boost { .. } }));
        let rec = s_record(&c, 3, &actions, next.state());
        let (replayed, _) = rec.replay().unwrap();
        assert_eq!(replayed.fingerprint(), next.state().fingerprint(), "the autopilot's game replays exactly");
    }

    #[test]
    #[ignore = "slow lane: a whole career on the autopilot, to the ending, ~90,000 ticks (~70 s)"]
    fn a_fresh_career_on_the_autopilot_reaches_every_step() {
        let (c, ms, g) = fresh(2);
        let boost = Some(parse_boost(DEFAULT_BOOST).unwrap());
        let mut ap = Autopilot::new(g, ms, DEFAULT_AGENT, Target::End, boost, 200_000, &[]).unwrap();
        // The swarm's clock, tick by tick (085): nothing before 1.11; then a
        // 100-year countdown that each retirement moves by the 5-year gap, and
        // that runs in a later timeline only once the Telescope is rebuilt.
        use hh_core::modules::timeline::located;
        let r = hh_core::Rules::new(&c).unwrap();
        let (mut retire_moves, mut first) = (0, None);
        let st = loop {
            let g = ap.state();
            let (was, retirements, was_located) = (g.saga.timeline.clone(), g.career.retirements, located(g));
            let st = ap.step(1);
            let g = ap.state();
            if !g.career.completed_quests.contains("qdeep_space_telescope") && g.saga.ended == 0 {
                assert!(g.saga.timeline.is_none(), "no calendar before 1.11");
            }
            if let (Some(a), Some(b)) = (&was, &g.saga.timeline) {
                let same = a.number == b.number && b.arrival == a.arrival && !g.saga.victory;
                if same && (!was_located || first.is_none()) && located(g) {
                    let v = r.view(g).saga;
                    first = first.or(v.days_left);
                    println!("clock: tick {} timeline {} located: swarm in {:?} days", g.tick, b.number, v.days_left);
                }
                if same && was_located {
                    let retired = u64::from(g.career.retirements - retirements);
                    assert_eq!(b.day - a.day, 1 + retired * 1825, "a day, and 5 years a retirement");
                    if retired > 0 {
                        let left = b.arrival.saturating_sub(b.day);
                        println!("clock: tick {} timeline {} retirement: swarm in {left} days ({:.1} years)", g.tick, b.number, left as f64 / 365.0);
                    }
                    retire_moves += retired;
                } else if same && !located(g) {
                    assert_eq!(b.day, a.day, "no countdown before the Telescope stands");
                    if b.number > 1 && g.tick.is_multiple_of(1000) {
                        assert!(r.view(g).saga.days_left.is_none(), "no countdown shown");
                    }
                }
            }
            if st.reached || st.gave_up {
                break st;
            }
        };
        assert!(first.is_some_and(|d| (36_000..=36_500).contains(&d)), "about 100 years at discovery: {first:?}");
        assert!(retire_moves > 0, "retirements moved the countdown");
        // Every milestone and every main-quest step through the ending (Q18)
        // is met (Matt's ruling on question 042_1).
        assert!(st.reached, "{st:?}");
        assert!(st.left.is_empty(), "steps left: {:?}", st.left);
        assert!(ap.state().saga.victory, "the ending");
        // A timeline's end takes the global projects' effects; the drivers fund
        // them again (084): the last timeline has them done, the Telescope too
        // (without it no countdown runs, so no swarm meets the sphere: 085).
        assert!(ap.state().saga.ended >= 1);
        for id in ["blueprint_archive", "conditional_logic", "deep_space_telescope"] {
            assert!(hh_core::modules::projects::completed(ap.state(), id), "{id} re-funded");
        }
    }
}
