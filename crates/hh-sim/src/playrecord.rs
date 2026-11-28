//! The play record (Matt, 2026-11-27): what a career did, written into its play log as
//! it plays (web/README.md, *Play logs*), so a log stays readable after the rules
//! change. Nothing in it is recomputed later: `hh-sim vs-human --skilled` reads it
//! instead of replaying ([`PlayRecord`]).
//!
//! Four kinds of line, each with the career `tick` and the `run` and `day` it is about:
//! - `record_start`: the record's format, and whether it began mid-career (`resumed`, a
//!   save from before records, so its first milestones read late);
//! - `snapshot`, about every [`SNAPSHOT_DAYS`] run days and on each run's first look:
//!   resources and buildings owned (whole numbers, zeros left out), ideology ranks,
//!   boredom, and credits earned this run;
//! - `landmark`: `milestone:<id>`, `project:<id>` (a project's first completion),
//!   `rank5:<axis>` (the first this-run rank 5 on that axis), each once a career;
//! - `run_end`: a run's days, credits earned and credits a day.
//!
//! The milestones and landmarks are the sim's own measure ([`Marks`]), so the human's
//! record and a sim player's career read alike.

use std::collections::BTreeMap;

use hh_core::modules::{economy, ideology};
use hh_core::{content, Fx, GameState, Rules};
use serde::{Deserialize, Serialize};

use crate::milestones::{last_retirement, Hit, Marks, Milestones};

pub const FORMAT: &str = "hh-playrecord/1";

/// Run days between snapshots: a log format choice (about 10 days, Matt), not a rule.
pub const SNAPSHOT_DAYS: u64 = 10;

/// A record line, tagged by `kind` (the play log's own field).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Line {
    RecordStart { format: String, tick: u64, run: u32, day: u64, resumed: bool },
    Snapshot {
        tick: u64,
        run: u32,
        day: u64,
        resources: BTreeMap<String, i64>,
        buildings: BTreeMap<String, u32>,
        ranks: BTreeMap<String, i64>,
        boredom: i64,
        /// Credits earned this run so far.
        earned: i64,
    },
    Landmark { tick: u64, run: u32, day: u64, landmark: String },
    RunEnd { tick: u64, run: u32, day: u64, earned: i64, per_day: f64 },
}

fn whole(x: Fx) -> i64 {
    x.floor_int()
}

/// Keeps the record as a career plays: [`Recorder::before`] before each tick,
/// [`Recorder::observe`] after each tick and each round of actions. Serializable: the web
/// client saves it with the game, so a reload keeps the record whole.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recorder {
    marks: Marks,
    run: u32,
    /// The run's credits earned at the last observation (a retirement's `run_end`).
    earned: Fx,
    /// The run and day of the last snapshot.
    snapshot: Option<(u32, u64)>,
}

impl Recorder {
    /// A record from the state now; `resumed`: the career began before it.
    pub fn new(ms: &Milestones, g: &GameState, resumed: bool) -> (Recorder, Vec<Line>) {
        let rec = Recorder { marks: Marks::new(ms, g), run: g.career.run_number, earned: g.run.stats.credits_earned, snapshot: None };
        let start = Line::RecordStart { format: FORMAT.into(), tick: g.tick, run: g.career.run_number, day: g.run.day, resumed };
        (rec, vec![start])
    }

    pub fn before(&mut self, g: &GameState) {
        self.marks.tracker.before_state(g);
    }

    /// The lines the state now adds to the record: after a tick (`ticked`), or after a
    /// round of actions. Long goals are read after ticks only, as the sim's
    /// [`crate::career::Meter`] reads them (milestones after both).
    pub fn observe(&mut self, ms: &Milestones, r: &Rules, g: &GameState, ticked: bool) -> Vec<Line> {
        let mut out = Vec::new();
        let mut ended = None;
        if g.career.run_number != self.run {
            if let Some(e) = last_retirement(g) {
                let per_day = if e.day > 0 { self.earned.to_f64() / e.day as f64 } else { 0.0 };
                out.push(Line::RunEnd { tick: g.tick, run: e.run, day: e.day, earned: whole(self.earned), per_day });
                ended = Some((e.run, e.day));
            }
            self.run = g.career.run_number;
        }
        self.marks.tracker.after_state(g);
        let at = |h: &Hit, key: String| Line::Landmark { tick: g.tick, run: h.run, day: h.day, landmark: key };
        for id in self.marks.check(ms, g, ended) {
            out.push(at(&self.marks.hits[&id], format!("milestone:{id}")));
        }
        if ticked {
            for key in self.marks.landmarks(r, g, ended) {
                out.push(at(&self.marks.landmarks[&key], key));
            }
        }
        self.earned = g.run.stats.credits_earned;
        let due = match self.snapshot {
            Some((run, day)) => run != g.career.run_number || g.run.day >= day + SNAPSHOT_DAYS,
            None => true,
        };
        if due {
            self.snapshot = Some((g.career.run_number, g.run.day));
            out.push(snapshot(r, g));
        }
        out
    }
}

fn snapshot(r: &Rules, g: &GameState) -> Line {
    let c = content();
    let resources = c
        .resources
        .iter()
        .filter(|x| x.id != economy::BOREDOM)
        .map(|x| (x.id.clone(), whole(g.amount(&x.id))))
        .filter(|x| x.1 != 0)
        .collect();
    let buildings = c.buildings.iter().map(|b| (b.id.clone(), g.owned(&b.id))).filter(|x| x.1 > 0).collect();
    let ranks = content::AXES.iter().map(|a| (a.to_string(), ideology::rank(r, g, a))).collect();
    Line::Snapshot {
        tick: g.tick,
        run: g.career.run_number,
        day: g.run.day,
        resources,
        buildings,
        ranks,
        boredom: whole(g.amount(economy::BOREDOM)),
        earned: whole(g.run.stats.credits_earned),
    }
}

/// A run's credits a day: finished, or the run under way (`open`) at its last snapshot.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunCredits {
    pub run: u32,
    pub days: u64,
    pub per_day: f64,
    pub open: bool,
}

/// The record read back from a log: its first line, the milestones and long goals by
/// when each was first met, and credits a day by run.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PlayRecord {
    pub start: Option<(u64, u32, u64, bool)>,
    pub milestones: BTreeMap<String, Hit>,
    pub landmarks: BTreeMap<String, Hit>,
    pub runs: Vec<RunCredits>,
    pub snapshots: u64,
    /// The last snapshot's tick, run, day and credits earned.
    pub last: Option<(u64, u32, u64, i64)>,
}

impl PlayRecord {
    /// Reads one line (in tick order); a line of another kind is not the record's.
    pub fn read(&mut self, line: &serde_json::Value) {
        let Ok(l) = serde_json::from_value::<Line>(line.clone()) else { return };
        match l {
            Line::RecordStart { tick, run, day, resumed, .. } => {
                self.start.get_or_insert((tick, run, day, resumed));
            }
            Line::Snapshot { tick, run, day, earned, .. } => {
                self.snapshots += 1;
                self.last = Some((tick, run, day, earned));
            }
            Line::Landmark { tick, run, day, landmark } => {
                let hit = Hit { tick, run, day };
                match landmark.strip_prefix("milestone:") {
                    Some(id) => self.milestones.entry(id.to_string()).or_insert(hit),
                    None => self.landmarks.entry(landmark).or_insert(hit),
                };
            }
            Line::RunEnd { run, day, per_day, .. } => {
                if !self.runs.iter().any(|r| r.run == run) {
                    self.runs.push(RunCredits { run, days: day, per_day, open: false });
                }
            }
        }
    }

    /// Credits a day by run, the run under way last (from its last snapshot).
    pub fn credits_by_run(&self) -> Vec<RunCredits> {
        let mut v = self.runs.clone();
        if let Some((_, run, day, earned)) = self.last {
            if !v.iter().any(|r| r.run == run) && day > 0 {
                v.push(RunCredits { run, days: day, per_day: earned as f64 / day as f64, open: true });
            }
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hh_core::Session;

    /// The record a career writes as it plays reads back as the sim's own measure: the
    /// same milestones and landmarks a [`crate::career::Meter`] finds on that career.
    #[test]
    fn a_record_reads_back_as_the_meter_measures() {
        let config = hh_core::Config::from_json(include_str!("../../../configs/baseline.json")).unwrap();
        let ms = Milestones::from_json(include_str!("../../../configs/sim/milestones.json")).unwrap();
        // A scripted career's actions, played again under a meter and a recorder.
        let mut played = Session::new(&config, 7, None).unwrap();
        hh_core::driver::play(&mut played, 900, hh_core::driver::Policy::Script);
        let actions = played.record().actions;
        let mut s = Session::new(&config, 7, None).unwrap();
        let mut meter = crate::career::Meter::new(&s, &ms, &[]);
        let (mut rec, mut lines) = Recorder::new(&ms, &s.state, false);
        let mut record = PlayRecord::default();
        let mut snaps = 0;
        let mut i = 0;
        for _ in 0..900u64 {
            let mut acted = 0;
            while i < actions.len() && actions[i].tick == s.state.tick {
                s.apply(&actions[i].action).unwrap();
                i += 1;
                acted += 1;
            }
            if acted > 0 {
                meter.look(&s, acted);
                lines.extend(rec.observe(&ms, &s.rules, &s.state, false));
            }
            rec.before(&s.state);
            meter.tick(&mut s).unwrap();
            lines.extend(rec.observe(&ms, &s.rules, &s.state, true));
        }
        for l in &lines {
            if matches!(l, Line::Snapshot { .. }) {
                snaps += 1;
            }
            record.read(&serde_json::to_value(l).unwrap());
        }
        let g = meter.finish(&s, 7, "test", Vec::new(), 0.0);
        assert!(!g.milestones.is_empty(), "the test career meets a milestone");
        assert_eq!(record.milestones, g.milestones);
        assert_eq!(record.landmarks.keys().collect::<Vec<_>>(), g.landmarks.keys().collect::<Vec<_>>());
        for (k, h) in &record.landmarks {
            assert_eq!((h.run, h.day), (g.landmarks[k].run, g.landmarks[k].day), "{k}");
        }
        assert_eq!(record.snapshots, snaps);
        assert!((900 / SNAPSHOT_DAYS - 1..=900 / SNAPSHOT_DAYS + 2).contains(&snaps), "{snaps} snapshots");
        // A recorder survives a save: the same lines after a round trip.
        let back: Recorder = serde_json::from_str(&serde_json::to_string(&rec).unwrap()).unwrap();
        let (mut a, mut b) = (rec, back);
        a.before(&s.state);
        b.before(&s.state);
        s.advance(1);
        assert_eq!(a.observe(&ms, &s.rules, &s.state, true), b.observe(&ms, &s.rules, &s.state, true));
    }
}
