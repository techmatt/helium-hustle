//! The view's programs part: each program's lines as they run now, the
//! conditions' readings, the readings on offer, and `LinesRun` (lines run
//! over a span of days).

use serde::Serialize;

use super::breakdown::{short_view, ShortView};
use super::{names, BuildingView, ResourceView};
use crate::fx::Fx;
use crate::modules::*;
use crate::rules::Rules;
use crate::state::{Clause, Condition as When, Entry, GameState, LineOutcome, Op, Reading};

#[derive(Debug, Clone, Serialize)]
pub struct ProgramView {
    pub entries: Vec<Entry>,
    /// Assigned processors.
    pub processors: u32,
    /// Of those, the ones running: fewer while Data Centers go unpaid.
    pub running: u32,
    /// The line the next processor runs, and runs done of it.
    pub entry: u32,
    pub step: u32,
    /// What each line did the last time it ran today (`None`: not run today).
    pub outcomes: Vec<Option<LineOutcome>>,
    /// Per line, the input it was short of the last time it ran today
    /// (failed, or a conversion run part way), if it was.
    pub shorts: Vec<Option<ShortView>>,
    /// Per line, an `if` or `wait until` line's condition as it reads now.
    pub conditions: Vec<Option<ConditionView>>,
}

/// A condition now: whether it holds, and each clause's reading (`None`:
/// nothing to read) and whether that clause holds.
#[derive(Debug, Clone, Serialize)]
pub struct ConditionView {
    pub holds: bool,
    pub clauses: Vec<ClauseView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClauseView {
    pub value: Option<Fx>,
    pub holds: bool,
}

/// A reading a condition may use now, with its display label.
#[derive(Debug, Clone, Serialize)]
pub struct ReadingView {
    pub reading: Reading,
    pub label: String,
    pub value: Option<Fx>,
}

fn condition_view(r: &Rules, g: &GameState, c: &When) -> ConditionView {
    let clause = |cl: &Clause| ClauseView {
        value: programs::reading(r, g, &cl.reading),
        holds: programs::clause_holds(r, g, cl),
    };
    ConditionView {
        holds: programs::holds(r, g, c),
        clauses: std::iter::once(&c.first).chain(c.rest.as_ref().map(|j| &j.clause)).map(clause).collect(),
    }
}

/// Every program as it runs now (`running`: `programs::running(g)`).
pub(super) fn programs(r: &Rules, g: &GameState, running: &[u32]) -> Vec<ProgramView> {
    g.run
        .programs
        .iter()
        .zip(LinesRun::today(g).lines)
        .enumerate()
        .map(|(i, (p, outcomes))| ProgramView {
            entries: p.entries.clone(),
            processors: p.processors,
            running: running[i],
            entry: p.entry,
            step: p.step,
            outcomes,
            shorts: (0..p.entries.len())
                .map(|l| {
                    let k = format!("program:{i}:{l}");
                    g.run.today.shorts.get(&k).map(|s| short_view(g, &k, s))
                })
                .collect(),
            conditions: p
                .entries
                .iter()
                .map(|e| match &e.op {
                    Op::If { when } | Op::WaitUntil { when } => Some(condition_view(r, g, when)),
                    _ => None,
                })
                .collect(),
        })
        .collect()
}

/// The readings on offer: what the player can see now. Amount, percent of cap
/// (capped ones) and last-day rate per visible resource (land and processors:
/// amount only); a count per visible building; the day; pads ready once there
/// is a pad; demand per good once exact demand is revealed.
pub(super) fn readings(r: &Rules, g: &GameState, resources: &[ResourceView], buildings: &[BuildingView]) -> Vec<ReadingView> {
    let mut out = Vec::new();
    let mut add = |reading: Reading, label: String| {
        let value = programs::reading(r, g, &reading);
        out.push(ReadingView { reading, label, value });
    };
    for res in resources.iter().filter(|x| x.visible) {
        let id = res.id.to_string();
        let stock = id != economy::LAND && id != economy::PROCESSORS;
        let label = if id == economy::LAND { "Land free" } else { res.name };
        add(Reading::Amount { resource: id.clone() }, label.to_string());
        if res.cap.is_some() {
            add(Reading::Percent { resource: id.clone() }, format!("{} %", res.name));
        }
        if stock && id != economy::BOREDOM {
            add(Reading::Rate { resource: id }, format!("{} rate", res.name));
        }
    }
    for b in buildings.iter().filter(|b| b.visible) {
        add(Reading::Count { building: b.id.to_string() }, format!("{} count", b.name));
    }
    add(Reading::Day, "Day".to_string());
    if !g.run.pads.is_empty() {
        add(Reading::PadsReady, "Pads ready".to_string());
    }
    if research::reveals_demand(g) {
        let names = names();
        for good in g.run.markets.keys() {
            let name = names.get(good.as_str()).copied().unwrap_or(good);
            add(Reading::Demand { good: good.clone() }, format!("{name} demand"));
        }
    }
    out
}

/// The program lines run over a span of days (a client drawing once per
/// several days, or a trace): per program, per line in queue order, its
/// outcome the last time it ran in the span (`None`: not run). Noted after
/// each tick from that day's outcomes; the span holds no actions, so line
/// indices name the same lines throughout. Display only, like the view.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct LinesRun {
    pub days: u64,
    pub lines: Vec<Vec<Option<LineOutcome>>>,
}

impl LinesRun {
    /// Adds the day `g` just played.
    pub fn note(&mut self, g: &GameState) {
        self.days += 1;
        let programs = &g.run.programs;
        // A retirement in the span leaves new (empty) programs: fit to the queues now.
        self.lines.resize(programs.len(), Vec::new());
        for (i, p) in programs.iter().enumerate() {
            let row = &mut self.lines[i];
            row.resize(p.entries.len(), None);
            if let Some(today) = g.run.today.lines.get(&(i as u32)) {
                for (&line, &o) in today {
                    if let Some(slot) = row.get_mut(line as usize) {
                        *slot = Some(o);
                    }
                }
            }
        }
    }

    /// The day `g` just played alone.
    pub fn today(g: &GameState) -> LinesRun {
        let mut l = LinesRun::default();
        l.note(g);
        l
    }
}
