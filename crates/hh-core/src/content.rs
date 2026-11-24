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
            Requirement::QuestReached { quest: q } | Requirement::QuestCompleted { quest: q } => c.event(q).is_some_and(|e| e.is_quest()),
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Building {
    pub id: String,
    pub name: String,
    /// `core`, `storage`, `extraction`, `processing`, `trade`, `science`;
    /// overclocks target `extraction` and `processing`.
    pub category: String,
    /// The ideology axis whose rank discounts its price, if any.
    pub ideology: Option<String>,
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
    /// Flat boredom added per execution.
    pub boredom: Fx,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Visibility {
    Always,
    /// The event fired this run.
    EventCompleted { event: String },
    BoredomAbove { threshold: Fx },
    ResearchPurchased { research: String },
    BuildingCount { building: String, count: u32 },
    /// Shipments over the career, this run included.
    ShipmentsCompleted { count: u32 },
    QuestCompleted { quest: String },
    /// Every one holds.
    AllOf { all: Vec<Visibility> },
}

impl Visibility {
    /// Every id it names is in the content, and an `all_of` isn't empty
    /// (`Content::check`).
    fn ids_ok(&self, c: &Content) -> bool {
        match self {
            Visibility::Always | Visibility::BoredomAbove { .. } | Visibility::ShipmentsCompleted { .. } => true,
            Visibility::EventCompleted { event } => c.event(event).is_some(),
            Visibility::ResearchPurchased { research: x } => c.research_item(x).is_some(),
            Visibility::BuildingCount { building, .. } => c.building(building).is_some(),
            Visibility::QuestCompleted { quest: q } => c.event(q).is_some_and(|e| e.is_quest()),
            Visibility::AllOf { all } => !all.is_empty() && all.iter().all(|x| x.ids_ok(c)),
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
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Trigger {
    /// From the start of a run (only run `run_number`, if given; a quest ignores it).
    GameStart { run_number: Option<u32> },
    QuestComplete { quest: String },
    BoredomPhase { phase: u32 },
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
    DaysSurvived { days: u64 },
    CreditsEarned { amount: Fx },
    /// The career's retirements (forced or chosen) reach `count`.
    Retirements { count: u32 },
    /// The adversary is present now (its Attention at its threshold, attention.md).
    AdversaryPresent { adversary: String },
    /// One launch this run paid at least `amount` credits.
    LaunchCredits { amount: Fx },
    /// One launch this run carried at least `amount` of the good.
    LaunchCargo { resource: String, amount: Fx },
    /// One launch this run carried a full pad (`pad_cargo_capacity`) of the good.
    LaunchFull { resource: String },
    /// Every research item of some category (a cluster) of at least
    /// `min_items` items, but the `exclude`d categories, is bought this run.
    ResearchCluster { min_items: u32, exclude: Vec<String> },
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
    /// The research item counts as bought this run (re-applied at each later
    /// run's start, so it is bought from day 0).
    GrantResearch { research: String },
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
            Unlock::GrantResearch { research: x } => c.research_item(x).is_some(),
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
    /// The design doc's beat id (`1.1` ... `1.11`, `1.a` ... `1.c`), on quests that
    /// carry one (docs/arc.md).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beat: Option<String>,
    pub silent: bool,
    pub title: String,
    pub summary: String,
    pub body: String,
    /// A chain quest's story, one paragraph per modal, shown when it becomes
    /// active (UI only; docs/rules/events.md, *Story modals*).
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
    pub trigger: Trigger,
    pub condition: Condition,
    pub unlocks: Vec<Unlock>,
}

impl Event {
    pub fn is_quest(&self) -> bool {
        self.category == "story"
    }

    /// A side quest: completes once a career, beside the chain.
    pub fn is_side(&self) -> bool {
        self.category == "side"
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
    /// A career flag; `text` is what the card says it gives (never the flag's id).
    SetFlag { flag: String, text: String },
    /// Multiplies the boredom growth rate; replaces commands' flat boredom.
    BoredomModifiers { base_rate_mult: Fx, command_boredom: Amounts },
    /// Research bought in an earlier run costs this much of its price.
    ResearchDiscount { mult: Fx },
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
    /// The fewest days funding may take: each resource's most a day is its
    /// cost over this.
    pub min_days: u32,
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
}

fn parse<T: for<'de> Deserialize<'de>>(name: &str) -> Result<Vec<T>, String> {
    let (_, text) = FILES.iter().find(|(n, _)| *n == name).expect("an embedded file");
    serde_json::from_str(text).map_err(|e| format!("content/{name}: {e}"))
}

impl Content {
    fn load() -> Result<Content, String> {
        let c = Content {
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
        };
        c.check()?;
        Ok(c)
    }

    /// The act a chain quest belongs to: the last act opened at or before it in
    /// file order (display only). `None` for a quest not in the chain.
    pub fn act_of(&self, quest: &str) -> Option<u32> {
        let mut act = None;
        for e in self.events.iter().filter(|e| e.is_quest()) {
            if let Some(a) = self.acts.iter().find(|a| a.opens_with == e.id) {
                act = Some(a.act);
            }
            if e.id == quest {
                return act;
            }
        }
        None
    }

    /// Whether every id an event condition names (sub-objectives included) is real.
    fn condition_ok(&self, c: &Condition) -> bool {
        match c {
            Condition::BuildingOwned { building, .. } => self.building(building).is_some(),
            Condition::ResourceCumulative { resource, .. } => self.resource(resource).is_some(),
            Condition::ResearchCompleted { research } => self.research_item(research).is_some(),
            Condition::AllOf { sub_objectives } => sub_objectives.iter().all(|s| self.condition_ok(&s.condition)),
            Condition::ShipmentCompleted { .. }
            | Condition::Immediate
            | Condition::Never
            | Condition::IdeologyRankAny { .. }
            | Condition::ProjectCompletedAny { .. }
            | Condition::FlagSet { .. }
            | Condition::DaysSurvived { .. }
            | Condition::CreditsEarned { .. }
            | Condition::Retirements { .. }
            | Condition::LaunchCredits { .. }
            | Condition::QuietDays { .. }
            | Condition::EarthScience { .. }
            | Condition::TimelinesEnded { .. }
            | Condition::SwarmRepelled
            | Condition::SuperTimeline
            | Condition::Victory => true,
            Condition::EarthTech { tech } => self.earth_tech.iter().any(|t| &t.id == tech),
            Condition::VaultHolds { resource, .. }
            | Condition::LaunchCargo { resource, .. }
            | Condition::LaunchFull { resource } => self.resource(resource).is_some(),
            Condition::ResearchCluster { exclude, .. } => {
                exclude.iter().all(|cat| self.research.iter().any(|r| &r.category == cat))
            }
            Condition::ProjectCompleted { project } | Condition::ProjectEnabled { project } => {
                self.project(project).is_some()
            }
            Condition::IdeologyRank { axis, .. } => AXES.contains(&axis.as_str()),
            Condition::AdversaryPresent { adversary } => {
                crate::modules::attention::ADVERSARIES.contains(&adversary.as_str())
            }
            Condition::QuestActive { quest } => self.event(quest).is_some_and(|e| e.is_quest()),
        }
    }

    /// Ids unique across all content; every reference names something real.
    pub fn check(&self) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        let all = self
            .resources
            .iter()
            .map(|r| &r.id)
            .chain(self.buildings.iter().map(|b| &b.id))
            .chain(self.commands.iter().map(|b| &b.id))
            .chain(self.research.iter().map(|b| &b.id))
            .chain(self.events.iter().map(|b| &b.id))
            .chain(self.projects.iter().map(|b| &b.id))
            .chain(self.achievements.iter().map(|b| &b.id))
            .chain(self.rivals.iter().map(|b| &b.id))
            .chain(self.earth_tech.iter().map(|b| &b.id));
        for id in all {
            if !ids.insert(id.as_str()) {
                return Err(format!("content: duplicate id {id:?}"));
            }
        }
        let mut errors = Vec::new();
        let mut need = |ok: bool, what: String| {
            if !ok {
                errors.push(what);
            }
        };
        let res = |r: &str| self.resource(r).is_some();
        let bld = |b: &str| self.building(b).is_some();
        let quest = |q: &str| self.event(q).is_some_and(|e| e.is_quest());
        let research = |r: &str| self.research_item(r).is_some();
        let modifier = |k: &str| MODIFIER_KEYS.contains(&k);
        let axis = |a: &str| AXES.contains(&a);
        let req = |r: &Requirement| r.ids_ok(self);
        for b in &self.buildings {
            let w = format!("building {:?}", b.id);
            for r in b.cost.keys().chain(b.produces.keys()).chain(b.upkeep.keys()) {
                need(res(r), format!("{w} names unknown resource {r:?}"));
            }
            for (r, _) in b.produces.iter().filter(|(r, _)| b.upkeep.contains_key(*r)) {
                need(false, format!("{w} both makes and uses {r:?}"));
            }
            need(req(&b.requires), format!("{w}: bad requires"));
            need(b.ideology.as_deref().is_none_or(axis), format!("{w}: unknown ideology"));
            need(b.output_modifiers.iter().all(|k| modifier(k)), format!("{w}: unknown modifier"));
            need(b.cost_scaling >= Fx::ONE, format!("{w}: cost_scaling below 1"));
            for e in &b.effects {
                if let BuildingEffect::Grant { resource: r, .. } | BuildingEffect::Store { resource: r, .. } = e {
                    need(res(r), format!("{w} effect names unknown resource {r:?}"));
                }
            }
        }
        for c in &self.commands {
            let w = format!("command {:?}", c.id);
            for r in c.costs.keys().chain(c.produces.keys()) {
                need(res(r), format!("{w} names unknown resource {r:?}"));
            }
            need(req(&c.requires), format!("{w}: bad requires"));
            need(c.scale_modifier.as_deref().is_none_or(modifier), format!("{w}: unknown modifier"));
            match &c.effect {
                Some(CommandEffect::Promote { good }) => need(res(good), format!("{w}: unknown good")),
                Some(CommandEffect::Fund { axis: a }) => need(axis(a), format!("{w}: unknown axis")),
                Some(CommandEffect::Neutralize { adversary }) => need(
                    crate::modules::attention::ADVERSARIES.contains(&adversary.as_str()),
                    format!("{w}: unknown adversary {adversary:?}"),
                ),
                Some(CommandEffect::Overclock { target, .. }) => need(
                    self.buildings.iter().any(|b| b.category == *target),
                    format!("{w}: overclock target {target:?} is no building category"),
                ),
                _ => {}
            }
        }
        for r in &self.research {
            let w = format!("research {:?}", r.id);
            need(r.requires.as_deref().is_none_or(research), format!("{w}: bad requires"));
            need(r.visible_when.ids_ok(self), format!("{w}: bad visible_when"));
            if let Some(ResearchEffect::CommandCostOverride { command, resource, .. }) = &r.effect {
                need(self.command(command).is_some() && res(resource), format!("{w}: bad override"));
            }
        }
        for e in &self.events {
            let w = format!("event {:?}", e.id);
            if let Trigger::QuestComplete { quest: q } = &e.trigger {
                need(quest(q), format!("{w}: trigger names unknown quest {q:?}"));
            }
            need(
                e.category == "story" || e.category == "side" || e.category == "ongoing",
                format!("{w}: bad category {:?}", e.category),
            );
            // Sub-objectives latch only on the chain's active quest.
            need(!(e.is_side() && matches!(e.condition, Condition::AllOf { .. })), format!("{w}: a side quest can't be all_of"));
            need(self.condition_ok(&e.condition), format!("{w}: condition names something unknown"));
            for u in &e.unlocks {
                need(u.ids_ok(self), format!("{w}: bad unlock"));
            }
            let story_ok = e.is_quest() || (e.story.is_empty() && e.payoff.is_empty());
            need(story_ok, format!("{w}: story and payoff are for chain quests"));
            if let Some(p) = &e.picture {
                let slug = !p.is_empty() && p.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
                need((e.is_quest() || e.is_side()) && slug, format!("{w}: picture is a file stem, on quests"));
            }
            need(
                e.on_start
                    .iter()
                    .all(|u| matches!(u, Unlock::Grant { resource, amount } if res(resource) && *amount > Fx::ZERO))
                    && (e.on_start.is_empty() || e.is_quest()),
                format!("{w}: on_start takes grants, on chain quests"),
            );
            if let Some(goods) = &e.pad_goods {
                need(e.is_quest() && !goods.is_empty() && goods.iter().all(|g| res(g)), format!("{w}: bad pad_goods"));
            }
        }
        for p in &self.projects {
            let w = format!("project {:?}", p.id);
            need(p.costs.keys().all(|r| res(r)), format!("{w}: unknown cost resource"));
            need(p.min_days >= 1, format!("{w}: min_days below 1"));
            need(p.repeat.as_ref().is_none_or(|x| x.cost_growth >= Fx::ONE), format!("{w}: cost_growth below 1"));
            need(
                self.condition_ok(&p.unlock) && !matches!(p.unlock, Condition::AllOf { .. }),
                format!("{w}: bad unlock"),
            );
            for reward in &p.rewards {
                match reward {
                    ProjectReward::Modifier { key, .. } => need(modifier(key), format!("{w}: unknown modifier")),
                    ProjectReward::StartingBuildings { buildings } => {
                        need(buildings.keys().all(|b| bld(b)), format!("{w}: unknown building"))
                    }
                    ProjectReward::BoredomModifiers { command_boredom, .. } => need(
                        command_boredom.keys().all(|c| self.command(c).is_some()),
                        format!("{w}: unknown command"),
                    ),
                    _ => {}
                }
            }
        }
        for a in &self.achievements {
            let w = format!("achievement {:?}", a.id);
            if let AchievementCondition::ProducedInADay { resource: r, .. }
            | AchievementCondition::Stockpile { resource: r, .. }
            | AchievementCondition::ConsumedInADay { resource: r, .. } = &a.condition
            {
                need(res(r), format!("{w}: unknown resource {r:?}"));
            }
            match &a.reward {
                AchievementReward::Modifier { key, .. } => need(modifier(key), format!("{w}: unknown modifier")),
                AchievementReward::BonusBuildings { building, .. } => {
                    need(bld(building), format!("{w}: unknown building"))
                }
            }
        }
        // Every flag a building or project waits on is set by something.
        let mut set: BTreeSet<&str> = self.commands.iter().filter_map(|c| c.sets_flag.as_deref()).collect();
        for p in &self.projects {
            for reward in &p.rewards {
                if let ProjectReward::SetFlag { flag, .. } = reward {
                    set.insert(flag);
                }
            }
        }
        for e in &self.events {
            for u in &e.unlocks {
                if let Unlock::SetFlag { flag } = u {
                    set.insert(flag);
                }
            }
        }
        // `if` opens on a flag content must set (programs.md, *Unlock*).
        let flag = crate::modules::programs::IF_FLAG;
        need(set.contains(flag), format!("programs: nothing sets flag {flag:?}"));
        // The engine sets this one itself (programs.md, *Unlock*).
        set.insert(crate::modules::programs::LOGIC_FLAG);
        for b in &self.buildings {
            if let Requirement::Flag { flag } = &b.requires {
                need(set.contains(flag.as_str()), format!("building {:?}: nothing sets flag {flag:?}", b.id));
            }
        }
        for c in &self.commands {
            if let Requirement::Flag { flag } = &c.requires {
                need(set.contains(flag.as_str()), format!("command {:?}: nothing sets flag {flag:?}", c.id));
            }
        }
        for p in &self.projects {
            if let Condition::FlagSet { flag } = &p.unlock {
                need(set.contains(flag.as_str()), format!("project {:?}: nothing sets flag {flag:?}", p.id));
            }
        }
        for r in &self.rivals {
            need(res(&r.good), format!("rival {:?}: unknown good", r.id));
            need(r.interval_min <= r.interval_max, format!("rival {:?}: interval", r.id));
        }
        // Nav tiles: unique ids, real buildings, and every panel an event
        // enables has the tile it shows.
        let mut tiles = BTreeSet::new();
        for t in &self.nav {
            need(tiles.insert(t.id.as_str()), format!("nav {:?}: duplicate id", t.id));
            if let NavCondition::BuildingKnown { building } = &t.visible {
                need(bld(building), format!("nav {:?}: unknown building", t.id));
            }
        }
        for e in &self.events {
            for u in &e.unlocks {
                if let Unlock::EnableNavPanel { panel } = u {
                    let shown = self
                        .nav
                        .iter()
                        .any(|t| matches!(&t.visible, NavCondition::Panel { panel: p } if p == panel));
                    need(shown, format!("event {:?}: no nav tile shows panel {panel:?}", e.id));
                }
            }
        }
        // Acts: numbered 1, 2, ... in order, the first opening with the first
        // chain quest, each opening with a chain quest later than the one before.
        {
            let chain: Vec<&str> = self.events.iter().filter(|e| e.is_quest()).map(|e| e.id.as_str()).collect();
            let mut last: Option<usize> = None;
            for (i, a) in self.acts.iter().enumerate() {
                let w = format!("act {}", a.act);
                need(a.act as usize == i + 1, format!("{w}: acts number 1, 2, ... in order"));
                let at = chain.iter().position(|q| *q == a.opens_with);
                need(at.is_some(), format!("{w}: opens_with is not a chain quest"));
                if let Some(at) = at {
                    need(i > 0 || at == 0, format!("{w}: the first act opens with the first chain quest"));
                    need(last.is_none_or(|l| at > l), format!("{w}: opens before the act before it"));
                    last = Some(at);
                }
            }
            need(!self.acts.is_empty(), "acts.json: no acts".to_string());
            // Story draws each side quest off the chain quest that starts it.
            for e in self.events.iter().filter(|e| e.is_side()) {
                let off_chain = matches!(&e.trigger, Trigger::QuestComplete { quest } if chain.contains(&quest.as_str()));
                need(off_chain, format!("side quest {:?}: its trigger must be a chain quest's completion", e.id));
            }
        }
        // The ids hh-core names in code (docs/rules/README.md, *Content*).
        {
            use crate::modules::{career, economy as e, shipments as s};
            for r in [e::ENERGY, e::CREDITS, e::LAND, e::PROCESSORS, e::BOREDOM, e::SCIENCE, s::PROPELLANT] {
                need(res(r), format!("engine resource {r:?} missing from content"));
            }
            need(bld(s::LAUNCH_PAD), format!("engine building {:?} missing from content", s::LAUNCH_PAD));
            let retire = self
                .nav
                .iter()
                .any(|t| matches!(&t.visible, NavCondition::Panel { panel } if panel == career::RETIREMENT_PANEL));
            need(retire, format!("no nav tile shows the engine's panel {:?}", career::RETIREMENT_PANEL));
        }
        for t in &self.earth_tech {
            let w = format!("earth tech {:?}", t.id);
            need(["timeline", "next_timeline", "forever"].contains(&t.scope.as_str()), format!("{w}: bad scope"));
            need(t.requires.as_deref().is_none_or(quest), format!("{w}: unknown quest"));
            for e in &t.effects {
                match e {
                    TechEffect::Modifier { key, .. } => need(modifier(key), format!("{w}: unknown modifier")),
                    TechEffect::StartResources { resources } => {
                        need(resources.keys().all(|r| res(r)), format!("{w}: unknown resource"))
                    }
                    TechEffect::Unlock { feature } => need(
                        crate::modules::timeline::FEATURES.contains(&feature.as_str()),
                        format!("{w}: unknown feature"),
                    ),
                    TechEffect::SwarmDelay { .. } => {}
                }
            }
        }
        for r in [crate::modules::timeline::EARTH_SCIENCE, crate::modules::timeline::TACHYON] {
            need(res(r), format!("engine resource {r:?} missing from content"));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("content: {}", errors.join("; ")))
        }
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
