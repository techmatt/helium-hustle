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

mod program;
pub use program::*;

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
    /// An accelerator (a boost, the autopilot, the cheat building or Complete all
    /// research: `cheat.rs`) has been used: a play log of this
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
    /// Countdown days this timeline: from the swarm's discovery (1.6) in
    /// timeline 1, from the first day in each later one; each retirement's
    /// gap counts too.
    pub day: u64,
    /// The countdown day the swarm arrives.
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

/// A launch pad (one per Launch Pad owned, in purchase order).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pad {
    /// The good it carries; `None` is No cargo (a new pad's choice until
    /// the player picks one): Load skips it. A save from before No cargo
    /// holds a bare good, read as `Some`.
    pub good: Option<String>,
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
    /// Credits every launch paid this run, in all (trade).
    pub trade_credits: Fx,
    /// The most cargo one launch carried this run, by good.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub best_cargo: BTreeMap<String, Fx>,
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
    /// they got (`economy::note_unmet`; the day's rate).
    pub unmet: BTreeMap<String, Fx>,
    pub stalls: BTreeMap<String, Stall>,
    /// What each program line did the last time a processor ran it today:
    /// program -> line -> outcome.
    pub lines: BTreeMap<u32, BTreeMap<u32, LineOutcome>>,
    /// Display only (a program tab's light): per program, the steps its
    /// processors took today and how they went.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub steps: BTreeMap<u32, StepTally>,
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

/// A program's steps one day: taken, run in full (ran, done, or an `if`
/// read, not short of anything; a verb with nothing to act on is not), and
/// failed outright.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepTally {
    pub taken: u32,
    pub full: u32,
    pub failed: u32,
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
    /// An output would pass its cap (held + output > cap): nothing paid, nothing made.
    Skipped,
    /// Locked, or short of its costs.
    Failed,
    /// An `if` whose condition held.
    True,
    /// An `if` whose condition didn't hold: its body was jumped.
    False,
    /// A body line jumped with its false `if` (and not run earlier today).
    Jumped,
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
    /// career log's kinds but the retirements ([`LogEntry::kind`]; `unlock`: a
    /// career first, id the flag it set).
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
    /// The last drain's take of each funded resource the stock couldn't fully
    /// cover (project -> resource -> take): the actual rate the Projects view
    /// and `days_to_done` show. A resource met in full has no entry.
    pub project_short: BTreeMap<String, BTreeMap<String, Fx>>,
    /// `run`-scoped projects' progress and completions.
    pub projects: ProjectBook,
    /// Events fired this run, each with the run day it fired on (display only:
    /// Story's order; rules read only which fired).
    pub events: BTreeMap<String, u64>,
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
    /// Days folded into `flow_average` this run, at most `flow_average_days`
    /// (`economy::fold_flow_average`; display only).
    pub flow_days: u64,
    /// Per resource, the last full day's rate (`programs::note_day_net`):
    /// what its tick did to the stock before the cap's cut, less the demand it
    /// left unmet. On the view's next day, the Resources panel's per-day
    /// number.
    pub day_net: BTreeMap<String, Fx>,
    /// The run's activity feed, oldest first (display only: no rule reads it).
    pub activity: Vec<Activity>,
    /// A program's `retire` ran today: the run retires at the day's end.
    pub retire_asked: bool,
    /// The run's day of its last purchase of a building, land or research, by
    /// hand or by a program line (the `days_since_purchase` reading). Absent
    /// until the first.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_purchase: Option<u64>,
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
    /// Launch pay (`RunStats::trade_credits`) over every finished run.
    #[serde(default)]
    pub trade_credits: Fx,
}

/// A milestone, for the log the simulator and UI read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogEntry {
    pub tick: u64,
    pub run: u32,
    pub day: u64,
    /// `quest`, `achievement`, `project`, `research`, `unlock`, `retired`,
    /// `forced_retirement`, and the timeline's: `timeline` (id `timeline_N`),
    /// `victory` (`swarm_overloaded`), `swarm_repelled` (`false_victory`),
    /// `timeline_end` (the outcome) and `earth_tech` (the tech's id).
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
    /// Chain quests that have become active (their `on_start` applied once).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub started_quests: BTreeSet<String>,
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
    /// The state knows career-flag research (Programming): true in every new
    /// state, absent (read false) in a save from before it, which loading
    /// brings up to date (`research::grant_past_flags`).
    #[serde(default = "FlagResearch::before")]
    pub flag_research: FlagResearch,
}

/// Whether a state was made with career-flag research in the game: `Default`
/// (a new state) is true; a save without the field reads false.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FlagResearch(pub bool);

impl Default for FlagResearch {
    fn default() -> FlagResearch {
        FlagResearch(true)
    }
}

impl FlagResearch {
    fn before() -> FlagResearch {
        FlagResearch(false)
    }
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
        let mut g: GameState = serde_json::from_str(json).map_err(|e| format!("bad state JSON: {e}"))?;
        crate::modules::research::grant_past_flags(&mut g);
        Ok(g)
    }

    /// FNV-1a of the JSON form less the config, as 16 hex digits: equal
    /// fingerprints, equal play (the replay and native = wasm checks). The
    /// config is pinned apart (`Config::hash`, the golden's config hash), so
    /// a config key that changes no play moves no fingerprint.
    pub fn fingerprint(&self) -> String {
        let mut v = serde_json::to_value(self).expect("state serializes");
        v.as_object_mut().expect("state is an object").remove("config");
        hex(fnv1a64(v.to_string().as_bytes()))
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
