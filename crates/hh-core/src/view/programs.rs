//! The view's programs part: each program's lines as they run now, the
//! conditions' readings, the readings on offer, and `LinesRun` (lines run
//! over a span of days).

use serde::Serialize;

use super::breakdown::{short_view, ShortView};
use super::{names, BuildingView, ResourceView};
use crate::fx::Fx;
use crate::modules::*;
use crate::rules::Rules;
use crate::state::{Clause, Condition as When, Entry, GameState, LineOutcome, Op, Reading, Verb};

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
/// and days to cap (capped ones) and last-day rate per visible resource (land
/// and processors: amount only); a count, running count and next cost per
/// visible building; researched per visible research item; each program's
/// processors and the free ones; the land price, the day, every N days (at 1:
/// the editor sets N); once there is a pad, pads, idle pads, pads ready and
/// cargo (all, and per good) and
/// days since the last launch; days since the last purchase (unless switched
/// off); demand per good once exact demand is revealed.
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
    for res in resources.iter().filter(|x| x.visible && x.cap.is_some()) {
        let id = res.id.to_string();
        if id != economy::LAND && id != economy::PROCESSORS && id != economy::BOREDOM {
            add(Reading::DaysToCap { resource: id }, format!("Days to cap of {}", res.name));
        }
    }
    for b in buildings.iter().filter(|b| b.visible) {
        add(Reading::Count { building: b.id.to_string() }, format!("{} count", b.name));
    }
    for b in buildings.iter().filter(|b| b.visible) {
        add(Reading::Running { building: b.id.to_string() }, format!("{} running", b.name));
    }
    for b in buildings.iter().filter(|b| b.visible) {
        add(Reading::NextCost { building: b.id.to_string() }, format!("Next cost of {}", b.name));
    }
    for it in crate::content::content().research.iter().filter(|it| research::visible(g, it)) {
        add(Reading::Researched { item: it.id.clone() }, format!("Researched {}", it.name));
    }
    // Projects open now or completed: a guard for a `fund` line.
    for p in crate::content::content().projects.iter() {
        if projects::unlocked(r, g, p) || projects::completions(g, p) > 0 {
            add(Reading::Completed { project: p.id.clone() }, format!("Completed {}", p.name));
        }
    }
    for tab in 0..g.run.programs.len() as u32 {
        add(Reading::Processors { tab }, format!("Program {} processors", tab + 1));
    }
    add(Reading::FreeProcessors, "Free processors".to_string());
    add(Reading::LandPrice, "Land price".to_string());
    add(Reading::Day, "Day".to_string());
    // The editor sets N; offered at 1 (every day).
    add(Reading::Every { days: 1 }, "Every N days".to_string());
    let names = names();
    if !g.run.pads.is_empty() {
        add(Reading::Pads { good: None }, "Pads (all)".to_string());
        add(Reading::IdlePads { good: None }, "Idle pads (all)".to_string());
        add(Reading::PadsReady { good: None }, "Ready pads (all)".to_string());
        add(Reading::Cargo { good: None }, "Cargo of pads (all)".to_string());
        for good in g.run.markets.keys() {
            let name = names.get(good.as_str()).copied().unwrap_or(good);
            add(Reading::Pads { good: Some(good.clone()) }, format!("Pads on {name}"));
            add(Reading::IdlePads { good: Some(good.clone()) }, format!("Idle pads on {name}"));
            add(Reading::PadsReady { good: Some(good.clone()) }, format!("Ready pads {name}"));
            add(Reading::Cargo { good: Some(good.clone()) }, format!("Cargo of pads {name}"));
        }
        add(Reading::DaysSinceLaunch, "Days since last launch".to_string());
    }
    if r.p.programs.days_since_purchase_reading {
        add(Reading::DaysSincePurchase, "Days since last purchase".to_string());
    }
    if research::reveals_demand(g) {
        for good in g.run.markets.keys() {
            let name = names.get(good.as_str()).copied().unwrap_or(good);
            add(Reading::Demand { good: good.clone() }, format!("{name} demand"));
        }
    }
    out
}

/// One choice for a verb's slot: the value a line holds and its display label.
#[derive(Debug, Clone, Serialize)]
pub struct Choice {
    pub value: String,
    pub label: String,
}

/// A verb on offer to the editor: its label, the choices for its target slot
/// (`None`: it takes none; `target_all`: the slot may also be left at `all`,
/// a line's `target` null), whether it takes a program tab, and its amount (`none`, `optional`:
/// `N` or `all`, empty for 1; `n_or_all`, `up_to`).
#[derive(Debug, Clone, Serialize)]
pub struct VerbView {
    pub verb: Verb,
    pub label: &'static str,
    pub targets: Option<Vec<Choice>>,
    /// The target slot may be left at `all`.
    pub target_all: bool,
    pub tab: bool,
    pub amount: &'static str,
}

/// Every verb of an `act` line, with its slots' choices now (visible
/// buildings, the traded goods, visible research, the library's names).
pub(super) fn verbs(r: &Rules, g: &GameState, buildings: &[BuildingView]) -> Vec<VerbView> {
    let names = names();
    let goods: Vec<Choice> = g
        .run
        .markets
        .keys()
        .map(|id| Choice { value: id.clone(), label: names.get(id.as_str()).copied().unwrap_or(id).to_string() })
        .collect();
    let blds: Vec<Choice> =
        buildings.iter().filter(|b| b.visible).map(|b| Choice { value: b.id.to_string(), label: b.name.to_string() }).collect();
    let research: Vec<Choice> = crate::content::content()
        .research
        .iter()
        .filter(|it| research::visible(g, it))
        .map(|it| Choice { value: it.id.clone(), label: it.name.clone() })
        .collect();
    let mut saved: Vec<Choice> = Vec::new();
    for l in &g.career.library {
        if !saved.iter().any(|c| c.value == l.name) {
            saved.push(Choice { value: l.name.clone(), label: l.name.clone() });
        }
    }
    // Projects open to funding now (unlocked, not completed).
    let open: Vec<Choice> = crate::content::content()
        .projects
        .iter()
        .filter(|p| projects::unlocked(r, g, p))
        .map(|p| Choice { value: p.id.clone(), label: p.name.clone() })
        .collect();
    let v = |verb, label, targets: Option<Vec<Choice>>, target_all, tab, amount| VerbView { verb, label, targets, target_all, tab, amount };
    let mut out = vec![
        v(Verb::LaunchPads, "Launch pads", Some(goods.clone()), true, false, "none"),
        v(Verb::LoadPads, "Load pads", Some(goods.clone()), true, false, "none"),
        v(Verb::SetPads, "Set pads", Some(goods), false, false, "up_to"),
        v(Verb::Pause, "Pause", Some(blds.clone()), false, false, "optional"),
        v(Verb::Resume, "Resume", Some(blds), false, false, "optional"),
        v(Verb::Research, "Research", Some(research), false, false, "none"),
        v(Verb::LoadProgram, "Load", Some(saved), false, true, "none"),
        v(Verb::AssignProcessors, "Assign processors", None, false, true, "n_or_all"),
        v(Verb::Retire, "Retire", None, false, false, "none"),
    ];
    if r.p.programs.fund_verb {
        // The editor writes `N` (0 stops it) or `all`; a line with no amount
        // is `all` too.
        out.push(v(Verb::Fund, "Fund", Some(open), false, false, "n_or_all"));
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
