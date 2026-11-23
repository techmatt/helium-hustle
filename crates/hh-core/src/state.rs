//! The game state: everything a saved game needs to reload to the same future.
//!
//! A game is a [`Career`] (what persists across retirements) and the current
//! [`Run`] (everything a retirement resets), plus the RNG streams. Every map is
//! ordered, so its JSON is stable; the seed and RNG positions are decimal strings
//! (too big for JavaScript numbers). Counts (buildings, days, ranks) are integers;
//! every quantity is an `Fx`.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::config::Config;
use crate::fx::Fx;
use crate::hash::{as_string, fnv1a64, hex};
use crate::rng::Stream;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameState {
    #[serde(with = "as_string")]
    pub seed: u64,
    /// The resolved config this game plays under. Shared, never changed in
    /// play: the view and `Rules::apply` copy the state often, and a deep copy
    /// of the config was a third of each copy. Its JSON is the config's.
    pub config: Arc<Config>,
    /// Ticks elapsed over the whole career (1 tick = 1 in-game day). Records
    /// stamp actions with it; the run's own day count is `run.day`.
    pub tick: u64,
    pub career: Career,
    pub run: Run,
    /// One stream per declared purpose (`rng.rs`).
    pub rng: BTreeMap<String, Stream>,
    /// A cheat action has been applied (`cheat.rs`): the game is marked for
    /// good. Absent from the JSON while false.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub cheated: bool,
    /// An accelerator (boost or autopilot) has been used: a play log of this
    /// career is no human's honest pace (`hh-sim vs-human` refuses it).
    /// Absent from the JSON while false.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub accelerated: bool,
    /// The debug boost (`Cheat::Boost`), absent while never set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boost: Option<crate::action::Boost>,
    /// The later acts (draft, docs/rules/timeline.md): what outlives a
    /// career. Absent from the JSON until the swarm is revealed.
    #[serde(default, skip_serializing_if = "Saga::is_default")]
    pub saga: Saga,
}

/// Draft (042): the timelines above the career. A timeline ends when the swarm
/// arrives and the mind is sent back: a hard reset of the career
/// (`timeline::end_timeline`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Saga {
    /// The current timeline; `None` until the reveal (no calendar).
    pub timeline: Option<Timeline>,
    /// Earth techs held, by id: how many more timeline ends each survives
    /// (`u32::MAX` for good).
    pub techs: BTreeMap<String, u32>,
    /// Earth science to spend on techs.
    pub science: Fx,
    /// Timelines ended so far, and the swarm repelled (false victories).
    pub ended: u32,
    pub repelled: u32,
    /// The swarm learned entanglement (after the first false victory).
    pub entangled: bool,
    /// The vault's stock (Act 3), by resource.
    pub vault: BTreeMap<String, Fx>,
    /// Resources kept across a retirement inside a super timeline.
    pub carry: BTreeMap<String, Fx>,
    /// Resources for the first run of the next timeline (compressed blocks).
    pub pending_start: BTreeMap<String, Fx>,
    /// The meta-controller's reset plan: program tab -> loadout slot.
    pub reset_plan: BTreeMap<u32, u32>,
    /// The ending was reached.
    pub victory: bool,
    /// Every finished timeline, oldest first (display: the Timeline view).
    pub history: Vec<TimelineEnd>,
}

impl Saga {
    pub fn is_default(&self) -> bool {
        *self == Saga::default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Timeline {
    /// 1-based.
    pub number: u32,
    /// Calendar days since it began (retirements cost days too).
    pub day: u64,
    /// The calendar day the swarm arrives.
    pub arrival: u64,
    /// The swarm's strength this timeline.
    pub strength: Fx,
    /// Earth science made this timeline.
    pub science_made: Fx,
    /// A super timeline (Act 3): vault spent, resources and projects survive retirements.
    pub super_timeline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimelineEnd {
    pub number: u32,
    pub days: u64,
    pub science_made: Fx,
    pub strength: Fx,
    /// `fell` (the swarm won), `super` (spent the vault), `victory`.
    pub outcome: String,
}

/// Buildings of one type: owned, of which `active` run, `bonus` were granted
/// at run start by projects or achievements (they don't inflate the cost
/// curve) and `free` took no land (starting and bonus copies: selling one
/// refunds none). `unpaid`: of the active ones, how many the last building
/// pass left without their upkeep (`active − floor(active × fraction paid)`):
/// they don't give their effects (grants, stores, pads, speculator decay)
/// until a pass pays them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Holding {
    pub owned: u32,
    pub active: u32,
    pub bonus: u32,
    pub free: u32,
    pub unpaid: u32,
}

impl Holding {
    /// `n` copies owned and active that took no land: a run's starting buildings
    /// or a scenario's.
    pub fn starting(n: u32) -> Holding {
        Holding { owned: n, active: n, bonus: 0, free: n, unpaid: 0 }
    }
}

/// One line of a program: an instruction, and whether it sits in the body of
/// the `if` above it (one level: a body can't hold another `if`).
/// JSON: `{"op": "command", "command": "idle", "count": 1}`, with
/// `"indented": true` for a body line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    #[serde(flatten)]
    pub op: Op,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub indented: bool,
}

impl Entry {
    /// A top-level line.
    pub fn new(op: Op) -> Entry {
        Entry { op, indented: false }
    }

    /// A top-level command line run `count` times before moving on.
    pub fn command(command: &str, count: u32) -> Entry {
        Entry::new(Op::Command { command: command.to_string(), count })
    }

    /// The same instruction in the body of the `if` above.
    pub fn indent(mut self) -> Entry {
        self.indented = true;
        self
    }

    /// The command this line runs, if it is a command line.
    pub fn command_id(&self) -> Option<&str> {
        match &self.op {
            Op::Command { command, .. } => Some(command),
            _ => None,
        }
    }

    /// How many times a command line runs before moving on (1 for the rest).
    pub fn count(&self) -> u32 {
        match &self.op {
            Op::Command { count, .. } => *count,
            _ => 1,
        }
    }

    /// The line as terse text with content ids (traces and logs, not the UI).
    pub fn describe(&self) -> String {
        let text = match &self.op {
            Op::Command { command, count } => format!("{command} x{count}"),
            Op::Build { building, up_to: None } => format!("build {building}"),
            Op::Build { building, up_to: Some(n) } => format!("build {building} up to {n}"),
            Op::BuyLand {} => "buy land".to_string(),
            Op::If { when } => format!("if {}", when.describe()),
            Op::WaitUntil { when } => format!("wait until {}", when.describe()),
            Op::Act { verb, target, tab, amount } => {
                let mut t = serde_json::to_value(verb).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default();
                if let Some(x) = target {
                    t = format!("{t} {x}");
                }
                if let Some(p) = tab {
                    t = format!("{t} program {}", p + 1);
                }
                match amount {
                    Some(Amount::N(n)) => t = format!("{t} {n}"),
                    Some(Amount::All) => t = format!("{t} all"),
                    None => {}
                }
                t
            }
        };
        if self.indented { format!("> {text}") } else { text }
    }
}

impl Reading {
    /// Terse text with content ids (traces and logs, not the UI).
    pub fn describe(&self) -> String {
        let all = |g: &Option<String>| g.clone().unwrap_or_else(|| "all".to_string());
        match self {
            Reading::Amount { resource } => resource.clone(),
            Reading::Percent { resource } => format!("{resource} %"),
            Reading::Rate { resource } => format!("{resource} rate"),
            Reading::Count { building } => format!("{building} count"),
            Reading::Day => "day".to_string(),
            Reading::PadsReady { good } => format!("ready pads {}", all(good)),
            Reading::Demand { good } => format!("{good} demand"),
            Reading::NextCost { building } => format!("next cost of {building}"),
            Reading::LandPrice => "land price".to_string(),
            Reading::DaysToCap { resource } => format!("days to cap of {resource}"),
            Reading::DaysSinceLaunch => "days since last launch".to_string(),
            Reading::Cargo { good } => format!("cargo of pads {}", all(good)),
            Reading::Every { days } => format!("every {days} days"),
            Reading::Researched { item } => format!("researched {item}"),
            Reading::Completed { project } => format!("completed {project}"),
            Reading::Pads { good } => format!("pads on {}", all(good)),
            Reading::Running { building } => format!("{building} running"),
            Reading::Processors { tab } => format!("program {} processors", tab + 1),
            Reading::DaysSincePurchase => "days since last purchase".to_string(),
            Reading::FreeProcessors => "free processors".to_string(),
            Reading::IdlePads { good } => format!("idle pads {}", all(good)),
        }
    }
}

impl Condition {
    pub fn describe(&self) -> String {
        let clause = |c: &Clause| {
            let reading = c.reading.describe();
            let cmp = match c.cmp {
                Cmp::Lt => "<",
                Cmp::Le => "<=",
                Cmp::Gt => ">",
                Cmp::Ge => ">=",
                Cmp::Eq => "=",
            };
            match &c.against {
                None => format!("{reading} {cmp} {}", c.value),
                Some(a) => format!("{reading} {cmp} {} x {}", a.describe(), c.value),
            }
        };
        match &self.rest {
            None => clause(&self.first),
            Some(j) => {
                let join = if j.join == Join::And { "and" } else { "or" };
                format!("{} {join} {}", clause(&self.first), clause(&j.clause))
            }
        }
    }
}

/// A program instruction. The set grows: a new kind is a new variant here and
/// its arm in `programs::step`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Op {
    /// Run `command` `count` times before moving on (a processor step each).
    Command { command: String, count: u32 },
    /// Buy one `building` (a processor step). With `up_to`, done (the step
    /// only) once that many are owned.
    Build { building: String, up_to: Option<u32> },
    /// Buy land (a processor step). A struct variant, so a stray key is refused.
    BuyLand {},
    /// Free. True: the indented lines below run. False: the pointer jumps past them.
    If { when: Condition },
    /// Free. Holds the pointer until true.
    WaitUntil { when: Condition },
    /// `<verb> <target> [amount]` (a processor step; docs/rules/programs.md,
    /// *Verbs*). The target slot is typed by the verb: `target` a building,
    /// good, research item or saved program's name, `tab` a program (0-based).
    Act {
        verb: Verb,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tab: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        amount: Option<Amount>,
    },
}

/// The verbs of an `act` line (docs/rules/programs.md, *Verbs*).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verb {
    /// `launch pads <good | all>`: Launch Full Pads, on that good's pads.
    LaunchPads,
    /// `load pads <good | all>`: Load Launch Pads, on that good's pads.
    LoadPads,
    /// `set pads <good> up to N`: points idle pads at the good.
    SetPads,
    /// `pause <building> [N | all]`.
    Pause,
    /// `resume <building> [N | all]`.
    Resume,
    /// `research <item>`.
    Research,
    /// `load <saved program> into <tab>`.
    LoadProgram,
    /// `assign processors <tab> N`.
    AssignProcessors,
    /// `retire`: a voluntary retirement at the day's end.
    Retire,
    /// `fund <project> [N | all]`: sets the project's daily rate on each of
    /// its costs, as the Projects panel does.
    Fund,
}

/// An `act` line's amount: `N` (`{"n": 3}`; `up to N` for `set pads`) or `all`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Amount {
    N(u32),
    All,
}

/// One clause, or two joined by `and` / `or`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Condition {
    pub first: Clause,
    pub rest: Option<Joined>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Joined {
    pub join: Join,
    pub clause: Clause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Join {
    And,
    Or,
}

/// `<reading> <cmp> <value>`, or with `against`, `<reading> <cmp> <against> × <value>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clause {
    pub reading: Reading,
    pub cmp: Cmp,
    pub value: Fx,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub against: Option<Reading>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cmp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
}

/// What a condition reads (docs/rules/programs.md, *Conditions*).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Reading {
    /// A resource's amount (land: the land free; boredom too).
    Amount { resource: String },
    /// Its amount as a percent of its cap (no cap: the clause is false).
    Percent { resource: String },
    /// What it made less what it used over the last full day.
    Rate { resource: String },
    /// How many of a building are owned.
    Count { building: String },
    /// The run's day.
    Day,
    /// Pads full and able to launch now: of one good's pads, or all.
    PadsReady {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        good: Option<String>,
    },
    /// A good's demand (needs exact demand revealed).
    Demand { good: String },
    /// The next one's price in credits (no credits cost: nothing).
    NextCost { building: String },
    /// The land price.
    LandPrice,
    /// Days until the resource reaches its cap at its `rate` (0 at the cap;
    /// nothing with no cap or a rate of 0 or less).
    DaysToCap { resource: String },
    /// Days since this run's last launch (none yet: the run's day).
    DaysSinceLaunch,
    /// Cargo aboard one good's pads, or all.
    Cargo {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        good: Option<String>,
    },
    /// 1 on a day that is a multiple of `days`, else 0 (`every N days`).
    Every { days: u32 },
    /// 1 once a research item is researched this run, else 0.
    Researched { item: String },
    /// How many times a project has completed in its scope (0 if never).
    Completed { project: String },
    /// Pads carrying one good, or every pad.
    Pads {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        good: Option<String>,
    },
    /// How many of a building are running (owned and active).
    Running { building: String },
    /// Processors assigned to a program tab (0-based).
    Processors { tab: u32 },
    /// Days since this run's last purchase: a building, land or research
    /// (none yet: the run's day).
    DaysSincePurchase,
    /// Processors the active buildings grant that no program holds: what
    /// `assign processors <tab> all` would add.
    FreeProcessors,
    /// Pads of one good, or every pad, with no cargo aboard (`set pads X` has
    /// one to switch exactly when `idle pads (all) > idle pads X × 1`).
    IdlePads {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        good: Option<String>,
    },
}

/// A program tab: a command queue, the processors assigned to it and its
/// shared instruction pointer (`entry`, and `step` runs done of that entry).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub entries: Vec<Entry>,
    pub processors: u32,
    pub entry: u32,
    pub step: u32,
}

/// A launch pad (one per Launch Pad owned, in purchase order).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pad {
    pub good: String,
    pub paused: bool,
    pub cargo: Fx,
    /// Days until it can launch again.
    pub cooldown: u32,
}

/// One tradeable good's market.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Market {
    /// This day's demand (set by the demand update each tick).
    pub demand: Fx,
    /// Noise frequency and lattice key, re-rolled each run.
    pub noise_freq: Fx,
    #[serde(with = "as_string")]
    pub noise_key: u64,
    pub speculators: Fx,
    /// The pool has been above zero this run (display).
    pub speculators_seen: bool,
    pub rival_pressure: Fx,
    pub saturation: Fx,
    pub promote: Fx,
    /// Shipment revenue since the last speculator burst (burst targeting).
    pub revenue: Fx,
}

/// An Overclock command's boost, live until `until_day`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overclock {
    pub target: String,
    pub bonus: Fx,
    pub until_day: u64,
}

/// Why a building ran short this tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stall {
    /// Ran at partial capacity: an upkeep input was short.
    InputStarved,
    /// Every output was at its cap when its building stage began (it still produced).
    OutputCapped,
}

/// This run's running totals.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunStats {
    pub credits_earned: Fx,
    pub shipments: u32,
    pub buildings_built: u32,
    pub research_completed: u32,
    /// Most energy buildings made in one day.
    pub peak_energy: Fx,
    /// The most credits one launch paid this run.
    pub best_launch: Fx,
    /// Everything produced this run, by resource (before caps).
    pub produced: BTreeMap<String, Fx>,
}

/// The current day's flows, cleared at the start of each tick.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Today {
    /// What passed through [`GameState::produce`] / [`GameState::consume`]
    /// today, and nothing else: writers that call `add` directly (boredom
    /// growth and command boredom, Dream, land, pad switch returns, timeline
    /// grants) are not here. Everything the tick did to a stock is the
    /// rate's snapshot (`programs::note_day_net`, `Run.day_net`).
    pub produced: BTreeMap<String, Fx>,
    pub consumed: BTreeMap<String, Fx>,
    /// Production lost to a cap at the end-of-tick clamp.
    pub overflow: BTreeMap<String, Fx>,
    /// Demand the day's tick left unmet, by resource: what buildings, program
    /// commands and projects wanted of an input they ran short of, less what
    /// they got (`economy::note_unmet`; the `rate` reading).
    pub unmet: BTreeMap<String, Fx>,
    pub stalls: BTreeMap<String, Stall>,
    /// What each program line did the last time a processor ran it today:
    /// program -> line -> outcome.
    pub lines: BTreeMap<u32, BTreeMap<u32, LineOutcome>>,
    /// Every flow by resource, then by source (display: the resource
    /// breakdown). Sources: `building:<id>`, `program:<n>` (0-based), `pads`
    /// (loading and switch returns), `launch`, `buy`, `sell`, `land`,
    /// `research`, `projects`, `growth` (boredom), `dream`, `cap` (overflow).
    /// Player actions after the day's tick count in it too.
    pub flows: BTreeMap<String, BTreeMap<String, Flow>>,
    /// Who ran short, by source (`building:<id>`, `program:<n>:<line>`): the
    /// input that limited it, the last time it ran today.
    pub shorts: BTreeMap<String, Short>,
}

/// One source's share of a resource's flow.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flow {
    pub produced: Fx,
    pub consumed: Fx,
}

/// A building or command that ran short: its scarcest input, what a full run
/// needed of it and what there was, and the fraction it ran at (0: not at all).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Short {
    pub resource: String,
    pub need: Fx,
    pub had: Fx,
    pub ran: Fx,
}

/// What a command did when a processor ran it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineOutcome {
    Ran,
    /// Every output was at its cap: nothing paid, nothing made.
    Skipped,
    /// Locked, or short of its costs.
    Failed,
    /// An `if` or `wait until` whose condition held.
    True,
    /// An `if` whose condition didn't hold: its body was jumped.
    False,
    /// A body line jumped with its false `if` (and not run earlier today).
    Jumped,
    /// A `wait until` holding the pointer.
    Waiting,
    /// Nothing to do (a `build ... up to N` with N owned, a verb with nothing
    /// to act on): the step taken, nothing else.
    Done,
}

/// One line of the run's activity feed (the UI's Recent Activity): what
/// happened on which day. Kept to the events module's `activity_kept` newest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Activity {
    pub day: u64,
    /// `shipment` (id: the good, quantity: cargo, amount: credits),
    /// `speculator_burst` (id: the good, amount: speculators added),
    /// `rival_dump` (id: the rival), `event` (a one-off event),
    /// `cargo_lost` (id: the good, quantity: lost, amount: the pad's 1-based
    /// number: cargo a switch returned past the stockpile's cap), and the
    /// career log's kinds (`quest`, `achievement`, `project`, `research`,
    /// `unlock`: a career first, id the flag it set).
    pub kind: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Fx>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Fx>,
}

/// Everything a retirement resets.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    /// Days into this run (the boredom curve reads it).
    pub day: u64,
    /// Every content resource, by id.
    pub resources: BTreeMap<String, Fx>,
    /// By content id; absent is none.
    pub buildings: BTreeMap<String, Holding>,
    pub land_bought: u32,
    pub programs: Vec<Program>,
    pub pads: Vec<Pad>,
    /// The tradeable goods in loading priority (Disrupt Speculators reads it).
    pub loading_priority: Vec<String>,
    pub markets: BTreeMap<String, Market>,
    /// Speculator bursts so far this run, and the day of the next.
    pub bursts: u32,
    pub next_burst_day: u64,
    /// Each rival's next dump day.
    pub rival_next: BTreeMap<String, u64>,
    pub overclocks: Vec<Overclock>,
    pub research: BTreeSet<String>,
    /// Ideology scores by axis.
    pub ideology: BTreeMap<String, Fx>,
    /// Project funding rates per day: project -> resource -> rate.
    pub project_rates: BTreeMap<String, BTreeMap<String, Fx>>,
    /// `run`-scoped projects' progress and completions.
    pub projects: ProjectBook,
    /// Events fired this run.
    pub events: BTreeSet<String>,
    /// Unlocked by event effects this run (re-applied from seen events at start).
    pub enabled_buildings: BTreeSet<String>,
    pub enabled_panels: BTreeSet<String>,
    pub enabled_projects: BTreeSet<String>,
    /// 1-based boredom phase.
    pub boredom_phase: u32,
    pub stats: RunStats,
    pub today: Today,
    /// Each day's `today.flows` folded into a rolling average over the
    /// economy's `flow_average_days` (display only: no rule reads it).
    pub flow_average: BTreeMap<String, BTreeMap<String, Flow>>,
    /// Per resource, the last full day's rate (`programs::note_day_net`):
    /// what its tick did to the stock before the cap's cut, less the demand it
    /// left unmet. The `rate` reading, and (on the view's next day) the
    /// Resources panel's per-day number.
    pub day_net: BTreeMap<String, Fx>,
    /// The run's activity feed, oldest first (display only: no rule reads it).
    pub activity: Vec<Activity>,
    /// The day of this run's last launch (the `days_since_launch` reading).
    pub last_launch: Option<u64>,
    /// The day of this run's last purchase of a building, land or research
    /// (the `days_since_purchase` reading).
    pub last_purchase: Option<u64>,
    /// A program's `retire` ran today: the run retires at the day's end.
    pub retire_asked: bool,
    /// The run's day of the player's last order (any action but a cheat; the
    /// `quiet_days` condition, draft). Absent until the first.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_order: Option<u64>,
    /// Cheat buildings bought this run (`Cheat::CheatBuilding`, `cheat.rs`);
    /// absent from the JSON while none.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub cheat_buildings: u32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// One scope's projects: progress toward the next completion, and how many
/// times each completed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectBook {
    /// project -> resource -> paid toward the next completion.
    pub progress: BTreeMap<String, BTreeMap<String, Fx>>,
    /// project -> completions (absent: none).
    pub completions: BTreeMap<String, u32>,
}

/// Career-wide totals over every finished run.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lifetime {
    pub credits_earned: Fx,
    pub shipments: u32,
    pub days: u64,
    pub buildings_built: u32,
    pub research_completed: u32,
}

/// A milestone, for the log the simulator and UI read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogEntry {
    pub tick: u64,
    pub run: u32,
    pub day: u64,
    /// `quest`, `achievement`, `project`, `research`, `unlock`, `retired`, `forced_retirement`.
    pub kind: String,
    pub id: String,
}

/// Everything that persists across retirements.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Career {
    pub run_number: u32,
    pub retirements: u32,
    pub lifetime: Lifetime,
    pub best_run_days: u64,
    pub best_run_credits: Fx,
    pub best_run_shipments: u32,
    pub max_ideology_ranks: BTreeMap<String, i64>,
    pub max_ideology_scores: BTreeMap<String, Fx>,
    /// Most energy buildings made in one day in any finished run (folded in at
    /// retirement, so the Buy Power bonus holds still within a run).
    pub peak_power: Fx,
    pub flags: BTreeSet<String>,
    pub researched: BTreeSet<String>,
    pub owned_buildings: BTreeSet<String>,
    pub used_commands: BTreeSet<String>,
    pub seen_events: BTreeSet<String>,
    pub completed_quests: BTreeSet<String>,
    /// `quest:sub_objective`, latched.
    pub completed_sub_objectives: BTreeSet<String>,
    /// `timeline`-scoped projects' progress and completions (a timeline's end
    /// clears them, unless a super timeline follows).
    pub projects: ProjectBook,
    /// `forever`-scoped projects': kept through every timeline end.
    pub forever_projects: ProjectBook,
    pub achievements: BTreeSet<String>,
    pub log: Vec<LogEntry>,
    /// The last finished run, as it stood when it retired (display: the UI's
    /// retirement summary). No rule reads it.
    pub last_run: Option<LastRun>,
    /// Saved programs (the loadout library), kept across retirements; at most
    /// `library_slots`. Loading one is how a program comes back in a new run.
    pub library: Vec<Loadout>,
    /// Each adversary's Attention (attention.md, draft): kept across
    /// retirements, decaying through the gap.
    pub attention: BTreeMap<String, Fx>,
    /// Ideology scores carried into the next run: a retirement keeps them,
    /// decayed through the gap (career.md).
    pub ideology: BTreeMap<String, Fx>,
    /// Days spent in retirement gaps over the career (unseen in Act 1).
    pub gap_days: u64,
}

/// A saved program: a display name (not an id; duplicates are fine) and its lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Loadout {
    pub name: String,
    pub entries: Vec<Entry>,
}

/// A finished run's summary.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LastRun {
    pub run: u32,
    pub days: u64,
    pub forced: bool,
    pub stats: RunStats,
    /// Each axis's rank when it ended.
    pub ideology_ranks: BTreeMap<String, i64>,
}

impl GameState {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("state serializes")
    }

    pub fn from_json(json: &str) -> Result<GameState, String> {
        serde_json::from_str(json).map_err(|e| format!("bad state JSON: {e}"))
    }

    /// FNV-1a of the JSON form, as 16 hex digits: equal fingerprints, equal
    /// games (the replay and native = wasm checks).
    pub fn fingerprint(&self) -> String {
        hex(fnv1a64(self.to_json().as_bytes()))
    }

    pub fn amount(&self, resource: &str) -> Fx {
        self.run.resources.get(resource).copied().unwrap_or(Fx::ZERO)
    }

    pub fn owned(&self, building: &str) -> u32 {
        self.run.buildings.get(building).map_or(0, |h| h.owned)
    }

    pub fn active(&self, building: &str) -> u32 {
        self.run.buildings.get(building).map_or(0, |h| h.active)
    }

    /// Active copies whose upkeep the last building pass paid: the count a
    /// building's effects (not its production, which the pass scales) come from.
    pub fn working(&self, building: &str) -> u32 {
        self.run.buildings.get(building).map_or(0, |h| h.active.saturating_sub(h.unpaid))
    }

    pub fn stream(&mut self, purpose: &str) -> &mut Stream {
        self.rng
            .get_mut(purpose)
            .unwrap_or_else(|| panic!("rng purpose {purpose:?} not declared by any module"))
    }

    /// Adds to a resource (`Fx` saturates rather than overflow in a very long game).
    pub(crate) fn add(&mut self, resource: &str, v: Fx) {
        let a = self.run.resources.entry(resource.to_string()).or_insert(Fx::ZERO);
        *a = a.saturating_add(v);
    }

    /// Counts a flow in today's breakdown under `source` (display only).
    pub(crate) fn track(&mut self, source: &str, resource: &str, produced: Fx, consumed: Fx) {
        if produced == Fx::ZERO && consumed == Fx::ZERO {
            return;
        }
        let flows = &mut self.run.today.flows;
        if !flows.contains_key(resource) {
            flows.insert(resource.to_string(), BTreeMap::new());
        }
        let by = flows.get_mut(resource).expect("inserted");
        if !by.contains_key(source) {
            by.insert(source.to_string(), Flow::default());
        }
        let f = by.get_mut(source).expect("inserted");
        f.produced = f.produced.saturating_add(produced);
        f.consumed = f.consumed.saturating_add(consumed);
    }

    /// Adds production: the resource, today's flow (by source) and the run's total.
    /// Every resource made passes here, so the debug boost applies here alone.
    pub(crate) fn produce(&mut self, source: &str, resource: &str, v: Fx) {
        let v = crate::cheat::boosted(self, resource, v);
        self.produce_unboosted(source, resource, v);
    }

    /// [`produce`](Self::produce) without the debug boost (the cheat building's
    /// flat amount).
    pub(crate) fn produce_unboosted(&mut self, source: &str, resource: &str, v: Fx) {
        if v <= Fx::ZERO {
            return;
        }
        self.track(source, resource, v, Fx::ZERO);
        self.add(resource, v);
        let t = self.run.today.produced.entry(resource.to_string()).or_insert(Fx::ZERO);
        *t = t.saturating_add(v);
        let s = self.run.stats.produced.entry(resource.to_string()).or_insert(Fx::ZERO);
        *s = s.saturating_add(v);
        if resource == crate::modules::economy::CREDITS {
            self.run.stats.credits_earned = self.run.stats.credits_earned.saturating_add(v);
        }
    }

    /// Takes a resource (it may dip a hair below zero within the upkeep
    /// tolerance; the end-of-tick clamp lifts it) and counts today's use.
    pub(crate) fn consume(&mut self, source: &str, resource: &str, v: Fx) {
        if v <= Fx::ZERO {
            return;
        }
        self.track(source, resource, Fx::ZERO, v);
        let a = self.run.resources.entry(resource.to_string()).or_insert(Fx::ZERO);
        *a -= v;
        let t = self.run.today.consumed.entry(resource.to_string()).or_insert(Fx::ZERO);
        *t = t.saturating_add(v);
    }

    /// Adds a milestone to the career log (and, but for retirements, which
    /// start a fresh run, to the run's activity feed).
    pub(crate) fn note(&mut self, kind: &str, id: &str) {
        let e = LogEntry {
            tick: self.tick,
            run: self.career.run_number,
            day: self.run.day,
            kind: kind.to_string(),
            id: id.to_string(),
        };
        self.career.log.push(e);
        if !matches!(kind, "retired" | "forced_retirement") {
            self.activity(kind, id, None, None);
        }
    }

    /// Adds a line to the run's activity feed (the events module trims it).
    pub(crate) fn activity(&mut self, kind: &str, id: &str, amount: Option<Fx>, quantity: Option<Fx>) {
        self.run.activity.push(Activity {
            day: self.run.day,
            kind: kind.to_string(),
            id: id.to_string(),
            amount,
            quantity,
        });
    }
}
