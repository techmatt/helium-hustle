//! A program's lines (`Entry`, `Op` and what an `if` reads) and the program tab that
//! runs them (docs/rules/programs.md). Re-exported from `state`.

use super::*;

/// One line of a program: an instruction, and whether it sits in the body of
/// the `if` or `else` above it (one level: a body can't hold another `if`).
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
            Op::Else {} => "else".to_string(),
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
            Op::Note { text } => format!("# {text}"),
            Op::Unparsed { text } => format!("unparsed: {text}"),
        };
        if self.indented { format!("> {text}") } else { text }
    }
}

impl Reading {
    /// Terse text with content ids (traces and logs, not the UI).
    pub fn describe(&self) -> String {
        match self {
            Reading::Amount { resource } => resource.clone(),
            Reading::Percent { resource } => format!("{resource} %"),
            Reading::FreeProcessors => "free processors".to_string(),
            Reading::FreeLand => "free land".to_string(),
            Reading::IdlePads => "idle pads".to_string(),
            Reading::EnergyPerDay => "energy per day".to_string(),
            Reading::DaysSincePurchase => "days since a purchase".to_string(),
        }
    }
}

impl Condition {
    pub fn describe(&self) -> String {
        let clause = |c: &Clause| {
            format!("{} {} {}", c.reading.describe(), c.cmp.symbol(), c.value)
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
    /// A processor step. True: the indented lines below run (and an `else`'s
    /// body after them is jumped). False: the pointer jumps past them, into
    /// the `else`'s body when one follows.
    If { when: Condition },
    /// Free, never runs, no outcome: follows an `if` or its body; the indented
    /// lines below run when that `if` was false. Reached from the `if`'s body,
    /// the pointer jumps past its own body. A struct variant, so a stray key is refused.
    Else {},
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
    /// Free, never runs: a `#` comment kept with the program (the text
    /// editor's, `program_text`), without its `#`.
    Note { text: String },
    /// Free, never runs: a text-editor line that doesn't parse, kept as its
    /// text until fixed.
    Unparsed { text: String },
}

impl Op {
    /// A note or unparsed line: free, never runs, no outcome.
    pub fn is_text(&self) -> bool {
        matches!(self, Op::Note { .. } | Op::Unparsed { .. })
    }
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

/// `<reading> <cmp> <value>`: the right side is always a number, in the
/// reading's unit (a percent for `percent`, else an amount).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clause {
    pub reading: Reading,
    pub cmp: Cmp,
    pub value: Fx,
}

/// The four comparators, written `<`, `>`, `<=` and `>=`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cmp {
    Lt,
    Gt,
    Le,
    Ge,
}

impl Cmp {
    /// The symbol a condition is written with.
    pub fn symbol(self) -> &'static str {
        match self {
            Cmp::Lt => "<",
            Cmp::Gt => ">",
            Cmp::Le => "<=",
            Cmp::Ge => ">=",
        }
    }
}

/// What a condition reads (docs/rules/programs.md, *Conditions*): one
/// reading per thing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Reading {
    /// A resource's amount (credits, science; a capped one or boredom too,
    /// at most its cap): not land or processors, which have readings of
    /// their own.
    Amount { resource: String },
    /// A capped resource's fill, or boredom's: its amount as a percent of
    /// its cap.
    Percent { resource: String },
    /// Processors the active buildings grant that no program holds: what
    /// `assign processors <tab> all` would add.
    FreeProcessors,
    /// Land not built on.
    FreeLand,
    /// Pads with no cargo aboard (a cooling pad and a No-cargo pad count).
    IdlePads,
    /// Energy's last full day's rate (`Run.day_net`): production less demand,
    /// a brownout's unmet demand counted. Researched (`program_readings`).
    EnergyPerDay,
    /// Days since this run's last purchase of a building, land or research
    /// (`Run.last_purchase`; none yet: the run's day). Researched.
    DaysSincePurchase,
}

impl Reading {
    /// Opened by research (`program_readings`), not offered before it.
    pub fn researched(&self) -> bool {
        matches!(self, Reading::EnergyPerDay | Reading::DaysSincePurchase)
    }
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
    /// Display only: per repeated command line (count over 1), its current or
    /// last pass, so the gutter shows each run. Cleared when the lines change.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub repeats: BTreeMap<u32, Repeat>,
}

/// A repeated line's pass: runs done of it, how many failed, and which of the
/// first eight (bit k: run k) failed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repeat {
    pub done: u32,
    pub failed: u32,
    pub first_fails: u8,
}
