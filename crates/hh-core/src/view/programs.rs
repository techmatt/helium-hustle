//! The view's programs part: each program's lines as they run now, the
//! conditions' readings, the readings on offer, and `LinesRun` (lines run
//! over a span of days).

use serde::Serialize;

use super::breakdown::{short_view, ShortView};
use super::{names, BuildingView, ResourceView};
use crate::fx::Fx;
use crate::modules::*;
use crate::rules::Rules;
use crate::state::{Clause, Condition as When, Entry, GameState, LineOutcome, Op, Reading, Repeat, Verb};

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
    /// Per line, an `if` line's condition as it reads now.
    pub conditions: Vec<Option<ConditionView>>,
    /// Per line, why it won't run yet this run (`programs::locked`): kept,
    /// and skipped free when run (an `if` with its body).
    pub locked: Vec<Option<String>>,
    /// Per line, an `unparsed` line's reason it doesn't read.
    pub errors: Vec<Option<String>>,
    /// Per repeated command line, its current or last pass (the gutter's dots).
    pub repeats: Vec<Option<Repeat>>,
    /// The tab's light, judged on the last day played; `None` with no
    /// processors assigned.
    pub status: Option<ProgramStatus>,
}

/// A program tab's status light: how its steps went the last day played.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Light {
    /// Every step ran in full.
    Green,
    /// Some steps were short, skipped (outputs full, nothing to act on) or
    /// failed, a line is locked, or a processor didn't run (Data Centers unpaid).
    Yellow,
    /// Nothing useful ran: no lines, no step taken, every step failed, or a
    /// line that doesn't read.
    Red,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgramStatus {
    pub light: Light,
    /// Steps taken, and of those run in full.
    pub steps: u32,
    pub full: u32,
    /// The hover: processors, steps, and the first line in trouble.
    pub tip: String,
}

/// A line for the light's hover: a command or building by its display name,
/// else its text with the first letter capitalized.
fn line_name(e: &Entry) -> String {
    let names = super::names();
    let name = |id: &str| names.get(id).copied().unwrap_or(id).to_string();
    let t = match &e.op {
        Op::Command { command, .. } => return name(command),
        Op::Build { building, .. } => return format!("Build {}", name(building)),
        op => crate::program_text::op_text(op),
    };
    let mut c = t.chars();
    c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
}

/// Program `i`'s light from the day just played (`state::StepTally`, each
/// line's last outcome and shortfall); `None` without processors.
fn status(r: &Rules, g: &GameState, i: usize, running: u32) -> Option<ProgramStatus> {
    let p = &g.run.programs[i];
    if p.processors == 0 {
        return None;
    }
    let tally = g.run.today.steps.get(&(i as u32)).copied().unwrap_or_default();
    let names = super::names();
    let mut head = match p.processors {
        1 => "1 processor".to_string(),
        n => format!("{n} processors"),
    };
    if running < p.processors {
        head += &format!(" ({running} running)");
    }
    let today = g.run.today.lines.get(&(i as u32));
    // The first line in trouble, in line order: a line that doesn't read,
    // then one short of an input, failed or skipped.
    let mut problem = None;
    for ((l, e), block) in p.entries.iter().enumerate().zip(programs::blocks(&p.entries)) {
        if let Op::Unparsed { text } = &e.op {
            let why = crate::program_text::unparsed_reason(r, g, text, e.indented, block);
            problem = Some(format!("line {} doesn't read: {why}", l + 1));
            break;
        }
    }
    let errored = problem.is_some();
    if problem.is_none() {
        for (l, e) in p.entries.iter().enumerate() {
            let short = g.run.today.shorts.get(&format!("program:{i}:{l}"));
            let why = match (today.and_then(|t| t.get(&(l as u32))), short) {
                (_, Some(s)) => format!("short on {}", names.get(s.resource.as_str()).copied().unwrap_or(&s.resource).to_lowercase()),
                (Some(LineOutcome::Failed), None) => "couldn't run".to_string(),
                (Some(LineOutcome::Skipped), None) => "outputs full".to_string(),
                (Some(LineOutcome::Done), None) if programs::pads_verb(&e.op) => "no pad ready".to_string(),
                _ => continue,
            };
            problem = Some(format!("{}: {why}", line_name(e)));
            break;
        }
    }
    // A line this run hasn't unlocked is skipped (programs.md, *Locked lines*).
    let locked = p.entries.iter().find_map(|e| programs::locked(r, g, e).map(|why| format!("{}: locked ({why})", line_name(e))));
    let skipped = locked.is_some() || running < p.processors;
    if problem.is_none() {
        problem = locked;
    }
    let runnable = p.entries.iter().any(|e| !e.op.is_text());
    let light = if errored || !runnable || tally.taken == 0 || tally.failed == tally.taken {
        Light::Red
    } else if tally.full < tally.taken || skipped {
        Light::Yellow
    } else {
        Light::Green
    };
    let mut parts = vec![head];
    if p.entries.is_empty() {
        parts.push("the program is empty".to_string());
    } else if !runnable {
        parts.push("no lines to run".to_string());
    } else if tally.taken == 0 {
        parts.push("no steps ran".to_string());
    } else {
        parts.push(format!("{} of {} steps ran in full", tally.full, tally.taken));
    }
    if let Some(why) = problem {
        parts.push(why);
    }
    Some(ProgramStatus { light, steps: tally.taken, full: tally.full, tip: parts.join(" · ") })
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
                    Op::If { when } => Some(condition_view(r, g, when)),
                    _ => None,
                })
                .collect(),
            locked: p.entries.iter().map(|e| programs::locked(r, g, e)).collect(),
            errors: p
                .entries
                .iter()
                .zip(programs::blocks(&p.entries))
                .map(|(e, block)| match &e.op {
                    Op::Unparsed { text } => Some(crate::program_text::unparsed_reason(r, g, text, e.indented, block)),
                    _ => None,
                })
                .collect(),
            repeats: (0..p.entries.len() as u32).map(|l| p.repeats.get(&l).copied()).collect(),
            status: status(r, g, i, running[i]),
        })
        .collect()
}

/// The readings on offer, one per thing, only what the player has unlocked:
/// the fill percent of each visible capped resource, then boredom's; the
/// amount of each visible uncapped one (credits, science); free processors
/// and free land; idle pads once there is a pad; energy per day and days
/// since a purchase once their research is done this run. A capped
/// resource's entry is its percent; a clause may read its amount instead
/// (the editor's unit toggle).
pub(super) fn readings(r: &Rules, g: &GameState, resources: &[ResourceView]) -> Vec<ReadingView> {
    let mut out = Vec::new();
    let mut add = |reading: Reading, label: String| {
        let value = programs::reading(r, g, &reading);
        out.push(ReadingView { reading, label, value });
    };
    let shown = || resources.iter().filter(|x| x.visible);
    // Land, boredom and processors read on their own lines below.
    let own = |id: &str| id == economy::LAND || id == economy::BOREDOM || id == economy::PROCESSORS;
    for res in shown().filter(|x| x.cap.is_some() && !own(x.id)) {
        add(Reading::Percent { resource: res.id.to_string() }, res.name.to_string());
    }
    for res in shown().filter(|x| x.id == economy::BOREDOM) {
        add(Reading::Percent { resource: res.id.to_string() }, res.name.to_string());
    }
    for res in shown().filter(|x| x.cap.is_none() && !own(x.id)) {
        add(Reading::Amount { resource: res.id.to_string() }, res.name.to_string());
    }
    add(Reading::FreeProcessors, "Free processors".to_string());
    add(Reading::FreeLand, "Free land".to_string());
    if !g.run.pads.is_empty() {
        add(Reading::IdlePads, "Idle pads".to_string());
    }
    if research::opens_program_readings(g) {
        add(Reading::EnergyPerDay, "Energy per day".to_string());
        add(Reading::DaysSincePurchase, "Days since a purchase".to_string());
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
    /// Per program, per line, the days of the span its `if` ended true: bit
    /// 0 the span's last day, bit k the day k before (the last 32 days).
    pub if_true: Vec<Vec<u32>>,
}

impl LinesRun {
    /// Adds the day `g` just played.
    pub fn note(&mut self, g: &GameState) {
        self.days += 1;
        let programs = &g.run.programs;
        // A retirement in the span leaves new (empty) programs: fit to the queues now.
        self.lines.resize(programs.len(), Vec::new());
        self.if_true.resize(programs.len(), Vec::new());
        for (i, p) in programs.iter().enumerate() {
            let row = &mut self.lines[i];
            row.resize(p.entries.len(), None);
            let bits = &mut self.if_true[i];
            bits.resize(p.entries.len(), 0);
            for b in bits.iter_mut() {
                *b <<= 1;
            }
            if let Some(today) = g.run.today.lines.get(&(i as u32)) {
                for (&line, &o) in today {
                    if let Some(slot) = row.get_mut(line as usize) {
                        *slot = Some(o);
                    }
                    if o == LineOutcome::True {
                        if let Some(b) = bits.get_mut(line as usize) {
                            *b |= 1;
                        }
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
