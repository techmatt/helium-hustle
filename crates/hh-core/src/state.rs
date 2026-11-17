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
}

/// Buildings of one type: owned, of which `active` run, `bonus` were granted
/// at run start by projects or achievements (they don't inflate the cost
/// curve) and `free` took no land (starting and bonus copies: selling one
/// refunds none).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Holding {
    pub owned: u32,
    pub active: u32,
    pub bonus: u32,
    pub free: u32,
}

/// One line of a program: run `command` `count` times before moving on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub command: String,
    pub count: u32,
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
    /// Every output was at its cap when the tick began (it still produced).
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
    /// Everything produced this run, by resource (before caps).
    pub produced: BTreeMap<String, Fx>,
}

/// The current day's flows, cleared at the start of each tick.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Today {
    pub produced: BTreeMap<String, Fx>,
    pub consumed: BTreeMap<String, Fx>,
    /// Production lost to a cap at the end-of-tick clamp.
    pub overflow: BTreeMap<String, Fx>,
    pub stalls: BTreeMap<String, Stall>,
    /// What each program line did the last time a processor ran it today:
    /// program -> line -> outcome.
    pub lines: BTreeMap<u32, BTreeMap<u32, LineOutcome>>,
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
    /// career log's kinds (`quest`, `achievement`, `project`, `research`).
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
    /// Strategic (per-run) project progress and completions.
    pub project_progress: BTreeMap<String, BTreeMap<String, Fx>>,
    pub completed_projects: BTreeSet<String>,
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
    /// The run's activity feed, oldest first (display only: no rule reads it).
    pub activity: Vec<Activity>,
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
    /// `quest`, `achievement`, `project`, `research`, `retired`, `forced_retirement`.
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
    /// Long-Term (persistent) project progress and completions.
    pub project_progress: BTreeMap<String, BTreeMap<String, Fx>>,
    pub completed_projects: BTreeSet<String>,
    pub achievements: BTreeSet<String>,
    pub log: Vec<LogEntry>,
    /// The last finished run, as it stood when it retired (display: the UI's
    /// retirement summary). No rule reads it.
    pub last_run: Option<LastRun>,
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

    /// Adds production: the resource, today's flow and the run's total.
    pub(crate) fn produce(&mut self, resource: &str, v: Fx) {
        if v <= Fx::ZERO {
            return;
        }
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
    pub(crate) fn consume(&mut self, resource: &str, v: Fx) {
        if v <= Fx::ZERO {
            return;
        }
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
