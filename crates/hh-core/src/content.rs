//! Game data, embedded at compile time from `content/` (`hh-core` reads no
//! files). Ids are unique across all content and are what states, actions and
//! records name, so reordering a file never breaks a save. Order matters where
//! the rules say so: buildings resolve in file order, quests chain in file order.
//!
//! The rows were converted from the Godot build's `reference/data/*.json`
//! (docs/rules/README.md, *Content*). Numbers that are global tunables live in
//! `configs/`, not here.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use crate::fx::Fx;
use crate::hash::{fnv1a64, hex};

mod check;

/// The embedded files, in hash order: `(name, text)`.
const FILES: &[(&str, &str)] = &[
    ("resources.json", include_str!("../../../content/resources.json")),
    ("buildings.json", include_str!("../../../content/buildings.json")),
    ("commands.json", include_str!("../../../content/commands.json")),
    ("research.json", include_str!("../../../content/research.json")),
    ("events.json", include_str!("../../../content/events.json")),
    ("projects.json", include_str!("../../../content/projects.json")),
    ("achievements.json", include_str!("../../../content/achievements.json")),
    ("rivals.json", include_str!("../../../content/rivals.json")),
    ("nav.json", include_str!("../../../content/nav.json")),
    ("earth_tech.json", include_str!("../../../content/earth_tech.json")),
    ("acts.json", include_str!("../../../content/acts.json")),
    ("messages.json", include_str!("../../../content/messages.json")),
];

/// The modifier keys rewards may name (docs/rules/README.md, *Modifiers*).
pub const MODIFIER_KEYS: &[&str] = &[
    "extractor_output_mult",
    "excavator_output_mult",
    "solar_output_mult",
    "building_upkeep_mult",
    "promote_effectiveness_mult",
    "speculator_burst_interval_mult",
    "land_cost_mult",
    "storage_cap_mult",
    "shipment_credit_mult",
    "cargo_capacity_mult",
    "demand_ceiling",
    "buy_power_mult",
    "boredom_growth_mult",
    "ideology_cross_mult",
    "science_output_mult",
    "nationalist_growth_mult",
    "ideology_half_life_mult",
];

/// The ideology axes buildings, commands and projects may name.
pub const AXES: &[&str] = &["nationalist", "humanist", "rationalist"];

pub type Amounts = BTreeMap<String, Fx>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub id: String,
    pub name: String,
    /// Base storage cap; `None` is uncapped (boredom's cap is the boredom module's).
    pub cap: Option<Fx>,
    /// Shown from the start; otherwise once a building making it is or was owned, a
    /// command making it was ever used, or some is held (`view.rs`).
    pub always_visible: bool,
}

/// What must hold before a building can be bought or a command run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Requirement {
    None,
    /// At least `count` of `building` owned.
    Building { building: String, count: u32 },
    /// The quest is active or completed (`"type": "quest"` in content).
    #[serde(rename = "quest")]
    QuestReached { quest: String },
    /// The quest is completed (in any run).
    QuestCompleted { quest: String },
    /// Researched this run.
    Research { research: String },
    /// A career flag is set.
    Flag { flag: String },
    /// The project is completed (in its scope: a building it gates follows it).
    Project { project: String },
}

impl Requirement {
    /// Every id it names is in the content (`Content::check`).
    fn ids_ok(&self, c: &Content) -> bool {
        match self {
            Requirement::None | Requirement::Flag { .. } => true,
            Requirement::Project { project } => c.project(project).is_some(),
            Requirement::Building { building, .. } => c.building(building).is_some(),
            Requirement::QuestReached { quest: q } | Requirement::QuestCompleted { quest: q } => c.event(q).is_some_and(|e| e.is_chain()),
            Requirement::Research { research: x } => c.research_item(x).is_some(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum BuildingEffect {
    /// Each active one adds `amount` of a derived resource (processors).
    Grant { resource: String, amount: Fx },
    /// Each active one raises `resource`'s cap by `amount`.
    Store { resource: String, amount: Fx },
    /// Each active one speeds speculator decay (demand config).
    SpeculatorDecay,
    /// Each working one takes `amount` boredom off each day, times its
    /// tagged axis's modifier (the Recreation Dome, boredom.md).
    BoredomRelief { amount: Fx },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Building {
    pub id: String,
    pub name: String,
    /// `core`, `storage`, `extraction`, `processing`, `trade`, `science`;
    /// overclocks target `extraction` and `processing`.
    pub category: String,
    /// The ideology axis it is tagged to, if any: its price is ÷ that axis's
    /// modifier, and a boredom effect it has is × it (ideology.md).
    pub ideology: Option<String>,
    /// Each purchase adds this to its tagged axis's score (a boredom tool's
    /// nudge; absent: none).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nudge: Option<Fx>,
    pub requires: Requirement,
    pub land: u32,
    pub cost_scaling: Fx,
    pub max_count: Option<u32>,
    /// Price of the first one, by resource id.
    pub cost: Amounts,
    /// Made per day by each active one.
    pub produces: Amounts,
    /// Used per day by each active one.
    pub upkeep: Amounts,
    pub effects: Vec<BuildingEffect>,
    /// Modifier keys multiplying its production.
    pub output_modifiers: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum CommandEffect {
    LoadPads,
    LaunchFullPads,
    Dream,
    /// Multiplies `target` category output by `1 + bonus` for `duration` days.
    Overclock { target: String, bonus: Fx, duration: u32 },
    Promote { good: String },
    DisruptSpeculators,
    Fund { axis: String },
    /// The neutralize stub: the adversary's Attention goes to zero.
    Neutralize { adversary: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub requires: Requirement,
    pub costs: Amounts,
    pub produces: Amounts,
    /// Flat boredom added per execution (negative: relief, × its tagged
    /// axis's modifier when it has one).
    pub boredom: Fx,
    /// The ideology axis whose boredom tool it is, if any (Dream, Theorize):
    /// its relief is × that axis's modifier, and each run nudges the axis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ideology: Option<String>,
    /// Score each execution adds to its `ideology` axis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nudge: Option<Fx>,
    pub effect: Option<CommandEffect>,
    /// A modifier key scaling both its costs and its production.
    pub scale_modifier: Option<String>,
    /// A career flag each execution sets.
    pub sets_flag: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResearchEffect {
    BoredomRateMultiplier { value: Fx },
    CommandCostOverride { command: String, resource: String, value: Fx },
    OverclockCap { value: Fx },
    LoadPerExecution { value: Fx },
    /// Markets show exact demand (not a tier).
    RevealDemand,
    /// Programs' `energy per day` and `days since a purchase` readings open
    /// (docs/rules/programs.md, *Conditions*).
    ProgramReadings,
    /// Buying it sets a career flag for good (Programming); once the flag holds
    /// the item never shows again.
    CareerFlag { flag: String },
    /// Multiplies a modifier key while it is done this run (`crate::modifiers`).
    Modifier { key: String, value: Fx },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Visibility {
    Always,
    /// Never: a row kept in the engine but not placed in the game yet.
    Never,
    /// The event fired this run.
    EventCompleted { event: String },
    BoredomAbove { threshold: Fx },
    ResearchPurchased { research: String },
    BuildingCount { building: String, count: u32 },
    /// Shipments over the career, this run included.
    ShipmentsCompleted { count: u32 },
    QuestCompleted { quest: String },
    /// The chain quest is active now or completed (events.md, `quest_active`).
    QuestActive { quest: String },
    /// The career flag is set.
    Flag { flag: String },
    /// Every one holds.
    AllOf { all: Vec<Visibility> },
    /// Any one holds.
    AnyOf { any: Vec<Visibility> },
}

impl Visibility {
    /// Every id it names is in the content, and no `all_of` or `any_of` is empty
    /// (`Content::check`).
    fn ids_ok(&self, c: &Content) -> bool {
        match self {
            Visibility::Always
            | Visibility::Never
            | Visibility::BoredomAbove { .. }
            | Visibility::ShipmentsCompleted { .. }
            | Visibility::Flag { .. } => true,
            Visibility::EventCompleted { event } => c.event(event).is_some(),
            Visibility::ResearchPurchased { research: x } => c.research_item(x).is_some(),
            Visibility::BuildingCount { building, .. } => c.building(building).is_some(),
            Visibility::QuestCompleted { quest: q } | Visibility::QuestActive { quest: q } => {
                c.event(q).is_some_and(|e| e.is_chain())
            }
            Visibility::AllOf { all: xs } | Visibility::AnyOf { any: xs } => {
                !xs.is_empty() && xs.iter().all(|x| x.ids_ok(c))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Research {
    pub id: String,
    pub name: String,
    pub category: String,
    pub cost: Fx,
    /// Another research item that must come first.
    pub requires: Option<String>,
    pub effect: Option<ResearchEffect>,
    pub visible_when: Visibility,
    /// This run's trigger for its group: it must hold even for an item bought in an
    /// earlier run, so each run's research unfolds in the same order.
    pub reveal_when: Visibility,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Trigger {
    /// From the start of a run (only run `run_number`, if given; a quest ignores it).
    GameStart { run_number: Option<u32> },
    QuestComplete { quest: String },
    BoredomPhase { phase: u32 },
    /// Once the career flag is set and the chain quest `beside` is active or
    /// done (a side quest hangs beside it in the Story view).
    Flag { flag: String, beside: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubObjective {
    pub id: String,
    pub label: String,
    pub condition: Condition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Condition {
    BuildingOwned { building: String, count: u32 },
    /// Produced this run (before caps).
    ResourceCumulative { resource: String, amount: Fx },
    /// Shipments this run.
    ShipmentCompleted { count: u32 },
    ResearchCompleted { research: String },
    Immediate,
    Never,
    /// Every sub-objective, each latched in the career once met.
    AllOf { sub_objectives: Vec<SubObjective> },
    IdeologyRankAny { rank: i64 },
    /// The career's best rank on `axis` reached `rank` (bests are recorded at
    /// the day's end, so it holds from the next day).
    IdeologyRank { axis: String, rank: i64 },
    /// Any project of the scope is completed.
    ProjectCompletedAny { scope: Scope },
    /// An event's `enable_project` enabled the project this run.
    ProjectEnabled { project: String },
    /// The quest is active now or completed in any run.
    QuestActive { quest: String },
    /// The career flag is set.
    FlagSet { flag: String },
    /// The one-off event has not fired this run (one earlier in file order
    /// firing the same day counts: it fires first).
    EventNotFired { event: String },
    DaysSurvived { days: u64 },
    CreditsEarned { amount: Fx },
    /// The career's retirements (forced or chosen) reach `count`.
    Retirements { count: u32 },
    /// The adversary is present now (its Attention at its threshold, attention.md).
    AdversaryPresent { adversary: String },
    /// Launches paid at least `amount` credits in all this run.
    TradeCredits { amount: Fx },
    /// One launch this run paid at least `amount` credits.
    LaunchCredits { amount: Fx },
    /// One launch this run carried at least `amount` of the good.
    LaunchCargo { resource: String, amount: Fx },
    /// One launch this run carried a full pad (`pad_cargo_capacity`) of the good.
    LaunchFull { resource: String },
    /// At least `count` research items are bought this run.
    ResearchCount { count: u32 },
    /// The career's best rank on every axis reached `rank`.
    IdeologyRankEach { rank: i64 },
    // Draft (042), the later acts (docs/rules/timeline.md):
    /// Days this run since the player's last order (programs don't count).
    QuietDays { days: u64 },
    /// Earth science made this timeline.
    EarthScience { amount: Fx },
    /// The Earth tech is held now.
    EarthTech { tech: String },
    TimelinesEnded { count: u32 },
    SwarmRepelled,
    /// The vault holds at least `amount` of the resource.
    VaultHolds { resource: String, amount: Fx },
    SuperTimeline,
    /// The project (any tier) is completed (career or run).
    ProjectCompleted { project: String },
    Victory,
}

/// A draft Earth tech (`content/earth_tech.json`, docs/rules/timeline.md).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EarthTech {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Earth science.
    pub cost: Fx,
    /// A quest that must be complete first, if any.
    pub requires: Option<String>,
    /// `timeline`, `next_timeline` or `forever`: how many timeline ends it survives.
    pub scope: String,
    pub effects: Vec<TechEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TechEffect {
    Modifier { key: String, value: Fx },
    /// Days added to the swarm's arrival in each timeline that starts while held.
    SwarmDelay { days: u64 },
    /// Resources added at the start of run 1 of the next timeline.
    StartResources { resources: Amounts },
    /// A feature the engine names (`timeline::FEATURES`).
    Unlock { feature: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Unlock {
    EnableBuilding { building: String },
    EnableNavPanel { panel: String },
    EnableProject { project: String },
    SetFlag { flag: String },
    /// Adds `amount` of `resource` to this run's stock (re-applied at each
    /// later run's start, like every unlock: docs/rules/events.md).
    Grant { resource: String, amount: Fx },
    /// A permanent modifier while the quest counts as completed (a side
    /// quest's reward: completions outlive every reset; modifiers.rs).
    Modifier { key: String, value: Fx },
}

impl Unlock {
    /// Every id it names is in the content, and a grant is positive
    /// (`Content::check`).
    fn ids_ok(&self, c: &Content) -> bool {
        match self {
            Unlock::EnableNavPanel { .. } | Unlock::SetFlag { .. } => true,
            Unlock::EnableBuilding { building } => c.building(building).is_some(),
            Unlock::EnableProject { project } => c.project(project).is_some(),
            Unlock::Grant { resource, amount } => c.resource(resource).is_some() && *amount > Fx::ZERO,
            Unlock::Modifier { key, value } => MODIFIER_KEYS.contains(&key.as_str()) && *value > Fx::ZERO,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub id: String,
    /// `story` events are the quest chain, in file order; `side` quests
    /// complete once a career beside it; `ongoing` fire once a run.
    pub category: String,
    /// A side quest's beat id (`1.a` ... `1.d`); a chain quest's is derived from the
    /// chain at load (`1.1` ..., `Content::number_chain`), never in the data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beat: Option<String>,
    pub silent: bool,
    pub title: String,
    /// A one-off event's one line (a quest has `goal` instead).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub summary: String,
    /// A quest's mechanics text (docs/rulings/story-ui.md, *Quest mechanics text*): what to do,
    /// and what completing it does (no how-to steps: Matt, 2026-11-26). Display only, never
    /// read by a rule; the narrative (`body`, `story`, `payoff`) carries no instructions or
    /// numbers.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub goal: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reward: String,
    /// A one-off event's mechanics text: what it does (may hold `{rate}`, as `body` did).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub effect: String,
    pub body: String,
    /// A chain quest's story, one paragraph per modal, shown when it becomes
    /// active; a `once` one-off's, shown when it fires (UI only;
    /// docs/rules/events.md, *Story modals*).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub story: Vec<String>,
    /// A chain quest's payoff, one paragraph per modal, shown when it completes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub payoff: Vec<String>,
    /// A quest's picture (display only, never read by a rule): the stem of a file in
    /// the web client's `src/assets/quests/` (docs/rules/events.md, *Story modals*).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    /// Applied once a career, the first time the quest becomes active
    /// (only `grant` here).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on_start: Vec<Unlock>,
    /// While the quest is active, the only goods a pad may be set to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pad_goods: Option<Vec<String>>,
    /// A one-off that fires at most once a career, not once a run.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub once: bool,
    pub trigger: Trigger,
    pub condition: Condition,
    pub unlocks: Vec<Unlock>,
}

impl Event {
    /// A main-story quest: one of the chain (category `story`), not a side quest.
    pub fn is_chain(&self) -> bool {
        self.category == "story"
    }

    /// A side quest: completes once a career, beside the chain.
    pub fn is_side(&self) -> bool {
        self.category == "side"
    }

    /// A one-off event: neither a chain quest nor a side quest.
    pub fn is_one_off(&self) -> bool {
        !self.is_chain() && !self.is_side()
    }
}

/// How long a project's progress and completions last (docs/rules/projects.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Reset at every retirement.
    Run,
    /// Kept across retirements, reset at a timeline's end (kept into a super timeline).
    Timeline,
    /// Kept for good, timeline ends included.
    Forever,
}

impl Scope {
    pub fn name(self) -> &'static str {
        match self {
            Scope::Run => "run",
            Scope::Timeline => "timeline",
            Scope::Forever => "forever",
        }
    }
}

/// A repeatable project: after each completion its progress starts over and
/// every cost is multiplied by `cost_growth` once more.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repeat {
    pub cost_growth: Fx,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectReward {
    /// Free (bonus) buildings at the start of every run.
    StartingBuildings { buildings: BTreeMap<String, u32> },
    Modifier { key: String, value: Fx },
    /// A flag held while the project counts as completed (never a career
    /// flag: a timeline's end takes it with the completion); `text` is what
    /// the card says it gives (never the flag's id).
    Flag { flag: String, text: String },
    /// Research items counted as bought at completion and at the start of
    /// every run while the project counts as completed.
    StartingResearch { research: Vec<String> },
    /// Multiplies the boredom growth rate; replaces commands' flat boredom.
    BoredomModifiers { base_rate_mult: Fx, command_boredom: Amounts },
    /// Research bought in an earlier run costs this much of its price: only
    /// research of `category` (a research row's), or every item when null.
    ResearchDiscount { mult: Fx, category: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub scope: Scope,
    /// Fundable while this holds (an events condition; `all_of` is refused).
    pub unlock: Condition,
    /// The first completion's costs (a repeatable one's grow per completion).
    pub costs: Amounts,
    /// A deliberate fewest days funding may take (the Deep Space Telescope's):
    /// each resource's most a day is also at most its cost over this. Absent:
    /// only the projects module's funding cap limits it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_days: Option<u32>,
    /// Absent: completes once. Present: repeatable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<Repeat>,
    /// Live while the project counts as completed (at least once).
    pub rewards: Vec<ProjectReward>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AchievementCondition {
    ProducedInADay { resource: String, at_least: Fx },
    Stockpile { resource: String, at_least: Fx },
    ConsumedInADay { resource: String, more_than: Fx },
    ShipmentRevenue { at_least: Fx },
    ShipmentsThisRun { at_least: u32 },
    ShipmentDemand { more_than: Fx },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AchievementReward {
    Modifier { key: String, value: Fx },
    BonusBuildings { building: String, count: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub condition: AchievementCondition,
    pub reward: AchievementReward,
    pub reward_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rival {
    pub id: String,
    pub name: String,
    pub good: String,
    pub interval_min: u32,
    pub interval_max: u32,
    pub demand_hit: Fx,
}

/// When a nav tile shows (display only: no rule reads it).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum NavCondition {
    Always,
    /// An event's `enable_nav_panel` enabled the panel this run (seen events
    /// re-enable theirs at each run start).
    Panel { panel: String },
    /// The building is owned now or was in any run.
    BuildingKnown { building: String },
    /// The career flag holds (`career::flag_set`).
    Flag { flag: String },
}

/// A tile of the UI's nav grid, in display order. Nav ids are panel names, a
/// namespace of their own (not checked against the other content ids).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavTile {
    pub id: String,
    pub name: String,
    pub visible: NavCondition,
}

/// An act of the story, in order (display only: no rule reads it). It opens
/// with the chain quest `opens_with`; each chain quest belongs to the last act
/// opened at or before it in file order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Act {
    pub act: u32,
    pub name: String,
    pub opens_with: String,
}

/// A line of UI prose the view carries (display only), so it lives in content
/// and `scripts/prose.py` exports it with the rest. Every id in `MESSAGES`, no other.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    pub id: String,
    pub text: String,
}

/// The messages the view reads: the run summary's line on why the run ended.
pub const MESSAGES: &[&str] = &["forced_retirement", "retired"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub resources: Vec<Resource>,
    pub buildings: Vec<Building>,
    pub commands: Vec<Command>,
    pub research: Vec<Research>,
    pub events: Vec<Event>,
    pub projects: Vec<Project>,
    pub achievements: Vec<Achievement>,
    pub rivals: Vec<Rival>,
    pub nav: Vec<NavTile>,
    pub earth_tech: Vec<EarthTech>,
    pub acts: Vec<Act>,
    pub messages: Vec<Message>,
}

fn parse<T: for<'de> Deserialize<'de>>(name: &str) -> Result<Vec<T>, String> {
    let (_, text) = FILES.iter().find(|(n, _)| *n == name).expect("an embedded file");
    serde_json::from_str(text).map_err(|e| format!("content/{name}: {e}"))
}

impl Content {
    fn load() -> Result<Content, String> {
        let mut c = Content {
            resources: parse("resources.json")?,
            buildings: parse("buildings.json")?,
            commands: parse("commands.json")?,
            research: parse("research.json")?,
            events: parse("events.json")?,
            projects: parse("projects.json")?,
            achievements: parse("achievements.json")?,
            rivals: parse("rivals.json")?,
            nav: parse("nav.json")?,
            earth_tech: parse("earth_tech.json")?,
            acts: parse("acts.json")?,
            messages: parse("messages.json")?,
        };
        c.number_chain()?;
        c.check()?;
        Ok(c)
    }

    /// Numbers the chain quests from their order: the `k`th quest of act `a` is `a.k`, its
    /// `beat` and the head of its displayed title (`1.2 — First Shipment`), so a reorder or a
    /// deletion touches only the chain. The data carries neither (a side quest keeps its own
    /// `beat`, `1.a`, and title).
    fn number_chain(&mut self) -> Result<(), String> {
        let mut numbers = Vec::new();
        let (mut act, mut k) = (0, 0);
        for e in self.events.iter().filter(|e| e.is_chain()) {
            if e.beat.is_some() || e.title.starts_with(|ch: char| ch.is_ascii_digit()) {
                return Err(format!("event {:?}: a chain quest's number comes from the chain, not the data", e.id));
            }
            if let Some(a) = self.acts.iter().find(|a| a.opens_with == e.id) {
                (act, k) = (a.act, 0);
            }
            k += 1;
            numbers.push(format!("{act}.{k}"));
        }
        for (e, n) in self.events.iter_mut().filter(|e| e.is_chain()).zip(numbers) {
            e.title = format!("{n} — {}", e.title);
            e.beat = Some(n);
        }
        Ok(())
    }

    /// The act a chain quest belongs to: the last act opened at or before it in
    /// file order (display only). `None` for a quest not in the chain.
    pub fn act_of(&self, quest: &str) -> Option<u32> {
        let mut act = None;
        for e in self.events.iter().filter(|e| e.is_chain()) {
            if let Some(a) = self.acts.iter().find(|a| a.opens_with == e.id) {
                act = Some(a.act);
            }
            if e.id == quest {
                return act;
            }
        }
        None
    }

    pub fn resource(&self, id: &str) -> Option<&Resource> {
        self.resources.iter().find(|r| r.id == id)
    }

    pub fn building(&self, id: &str) -> Option<&Building> {
        self.buildings.iter().find(|b| b.id == id)
    }

    pub fn command(&self, id: &str) -> Option<&Command> {
        self.commands.iter().find(|c| c.id == id)
    }

    /// A message's text (every id in `MESSAGES` is there: `check`).
    pub fn message(&self, id: &str) -> &str {
        self.messages.iter().find(|m| m.id == id).map_or("", |m| m.text.as_str())
    }

    pub fn research_item(&self, id: &str) -> Option<&Research> {
        self.research.iter().find(|r| r.id == id)
    }

    pub fn event(&self, id: &str) -> Option<&Event> {
        self.events.iter().find(|e| e.id == id)
    }

    pub fn project(&self, id: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.id == id)
    }

    /// Buildings an event's `enable_building` gates: buyable only once enabled this run.
    pub fn event_gated(&self, building: &str) -> bool {
        self.events.iter().any(|e| {
            e.unlocks
                .iter()
                .any(|u| matches!(u, Unlock::EnableBuilding { building: b } if b == building))
        })
    }
}

/// The embedded content (checked once; a broken file panics at first use and
/// fails every test).
pub fn content() -> &'static Content {
    static C: OnceLock<Content> = OnceLock::new();
    C.get_or_init(|| Content::load().unwrap_or_else(|e| panic!("{e}")))
}

/// FNV-1a over the embedded files' names and raw bytes.
pub fn content_hash() -> String {
    let mut bytes = Vec::new();
    for (name, text) in FILES {
        bytes.extend_from_slice(name.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(text.as_bytes());
        bytes.push(0);
    }
    hex(fnv1a64(&bytes))
}
