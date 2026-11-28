//! What a client (the UI or an agent) shows and can do: derived display data
//! (prices, rates, caps, what is affordable, unlocked or visible, the action
//! for each), so no client computes a rule. Never read by the rules, not part
//! of the state, the fingerprint or a replay.
//!
//! Content text (ids, names, descriptions, content rates) is borrowed from the
//! embedded content (`&'static`), not copied: a view is built at every look of
//! every sim player, and copying the text was most of its cost.

use serde::Serialize;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::action::Action;
use crate::content::{content, Amounts, Requirement, AXES, MODIFIER_KEYS};
use crate::fx::Fx;
use crate::modifiers::modifier;
use crate::modules::*;
use crate::rules::Rules;
use crate::state::{Activity, GameState, LastRun, Loadout, Overclock, RunStats, Stall};

use crate::modules::programs as program_rules;
use breakdown::breakdown;

/// The career flag that opens the 50x and 200x game speeds (Accelerated Scheduling's
/// `career_flag`, `content/research.json`).
pub const FAST_SPEEDS_FLAG: &str = "fast_speeds";

mod breakdown;
mod commands;
mod keep;
mod lab;
mod nav;
// The programs panel; the game's own programs module is `program_rules` here.
mod programs;
// The Projects panel; the game's own projects module is `modules::projects`.
mod projects;
mod quests;
mod rivals;
mod saga;
mod stats;
mod stock;
mod trade;

pub use self::projects::{condition_text, ProjectDoneView, ProjectView};
pub use saga::{SagaView, TechView};

pub use breakdown::{BreakdownView, FlowRow, ShortView};
pub use programs::{Choice, ClauseView, ConditionView, Light, LinesRun, ProgramStatus, ProgramView, ReadingView, VerbView};

/// Text borrowed from the embedded content.
pub type Text = &'static str;

#[derive(Debug, Clone, Serialize)]
pub struct ResourceView {
    pub id: Text,
    pub name: Text,
    pub amount: Fx,
    /// Most that can be held, if capped.
    pub cap: Option<Fx>,
    /// The next day's rate if nothing else is done (`programs::note_day_net`),
    /// before the cap's cut and with unmet demand counted.
    pub per_day: Fx,
    /// The stock's actual change over that day (after the cap, demand met
    /// only as far as it was): what an ETA wants.
    pub change_per_day: Fx,
    /// What the cap throws away that day (`per_day` reads this far past the change).
    pub overflow: Fx,
    /// That day's demand left unmet (`per_day` reads this far below the change, the cap aside).
    pub unmet: Fx,
    pub visible: bool,
    /// The debug grant cheat may grant it (`cheat::grantable`).
    pub grantable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildingView {
    pub id: Text,
    pub name: Text,
    pub description: Text,
    pub category: Text,
    /// The ideology axis it is tagged to (price ÷ its modifier), if any.
    pub ideology: Option<Text>,
    /// Its daily boredom relief per working copy now (× its axis's modifier); 0 for none.
    pub boredom_relief: Fx,
    pub owned: u32,
    pub active: u32,
    /// Active copies whose upkeep the last pass paid: the ones giving effects.
    pub working: u32,
    pub bonus: u32,
    pub max_count: Option<u32>,
    /// Price of the next one (`Fx::MAX` past the number range).
    pub cost: BTreeMap<String, Fx>,
    pub land: u32,
    /// What the next one needs that there isn't enough of now: cost
    /// resources, and `land`.
    pub short: Vec<String>,
    /// What one active copy makes and uses per day (content rates, before
    /// modifiers and overclocks), and the caps it raises.
    pub produces: &'static Amounts,
    pub upkeep: &'static Amounts,
    pub stores: BTreeMap<Text, Fx>,
    /// Resources one active copy grants (processors).
    pub grants: BTreeMap<Text, Fx>,
    /// What unlocks it (content).
    pub requires: &'static Requirement,
    /// Buyable now (unlocked, land, price, limit).
    pub affordable: bool,
    /// It owns as many as its limit allows.
    pub at_limit: bool,
    pub unlocked: bool,
    pub visible: bool,
    /// Its stall today, if any.
    pub stall: Option<Stall>,
    /// Ran below full capacity today: the input it was short of and the
    /// fraction it ran at.
    pub starved: Option<ShortView>,
    /// The action that buys one.
    pub buy: Action,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandView {
    pub id: Text,
    pub name: Text,
    /// Owned only when an overclock's `{days}` was filled in.
    pub description: Cow<'static, str>,
    pub unlocked: bool,
    pub visible: bool,
    pub costs: BTreeMap<String, Fx>,
    pub produces: BTreeMap<String, Fx>,
    /// What one run does to boredom now, signed (negative: it relieves),
    /// Dream's and a tagged relief at the axis's modifier.
    pub boredom: Fx,
    /// The axis whose boredom tool it is (its relief × that axis), if any.
    pub ideology: Option<Text>,
    /// A per-target command folded into one verb (docs/rulings/programs.md, the flat
    /// template): its verb (`Buy`, `Promote`, `Fund`, `Overclock`) and its
    /// target's label (the name past the verb). `None` for the rest.
    pub verb: Option<&'static str>,
    pub target: Option<Text>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PadView {
    /// `None`: No cargo (idle until a good is picked).
    pub good: Option<String>,
    pub paused: bool,
    pub active: bool,
    /// Switched on, but its Launch Pad's upkeep went unpaid: idle until paid.
    pub unpaid: bool,
    pub cargo: Fx,
    pub capacity: Fx,
    pub cooldown: u32,
    /// Propellant a launch burns.
    pub fuel: Fx,
    /// Cargo one Load Launch Pads moves onto it, at most.
    pub load: Fx,
    /// Credits it would fetch launched now.
    pub payout: Fx,
    pub can_launch: bool,
    /// The shared Propellant pool holds a launch's `fuel`.
    pub fueled: bool,
    /// Full and able to launch now.
    pub ready: bool,
    /// Cargo a switch to another good would return to the stockpile (the
    /// rest is lost over its cap).
    pub switch_returns: Fx,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketView {
    pub good: String,
    pub name: String,
    /// Exact demand, once Market Awareness is researched.
    pub demand: Option<Fx>,
    /// LOW, MEDIUM, HIGH or VERY HIGH.
    pub tier: &'static str,
    pub speculators: Option<Fx>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResearchView {
    pub id: Text,
    pub name: Text,
    pub description: Text,
    pub category: Text,
    pub cost: Fx,
    /// Research it needs first, if any.
    pub requires: Option<Text>,
    pub done: bool,
    pub visible: bool,
    pub affordable: bool,
    /// Its scope icon: `legacy`, `eternal` or none (run-only; `keep.rs`).
    pub keep: Option<Text>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AxisView {
    pub axis: &'static str,
    pub score: Fx,
    pub rank: i64,
    /// Scores (in size) where this rank and the next start, for a progress bar.
    pub rank_score: Fx,
    pub next_rank_score: Fx,
    /// The axis's modifier `m` now (tagged buildings cost ÷m, its boredom tool ×m).
    pub modifier: Fx,
}

/// A countable condition's progress.
#[derive(Debug, Clone, Serialize)]
pub struct ProgressView {
    pub now: Fx,
    pub target: Fx,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubObjectiveView {
    pub id: Text,
    pub label: Text,
    pub done: bool,
    pub progress: Option<ProgressView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuestView {
    pub id: Text,
    /// The condition's type (`building_owned`, `quiet_days`, ...).
    pub kind: &'static str,
    pub title: Text,
    /// Its mechanics text: the task line and what completing it does.
    pub goal: Text,
    pub reward: Text,
    /// Its story's panels (`content::panels`), one modal page each, shown as it becomes active.
    pub story: Vec<Text>,
    pub progress: Option<ProgressView>,
    pub sub_objectives: Vec<SubObjectiveView>,
    /// Its number (`1.5`, from the chain), if it carries one.
    pub beat: Option<Text>,
    /// The act it belongs to (`content/acts.json`).
    pub act: Option<u32>,
    /// A later chain quest exists (false on the last, which never completes).
    pub more: bool,
    /// Its picture's file stem (display only; the UI's `src/assets/quests/`), if it has one.
    pub picture: Option<Text>,
}

/// A side quest started (its trigger happened) or completed, for Story's timeline.
#[derive(Debug, Clone, Serialize)]
pub struct SideQuestView {
    pub id: Text,
    pub beat: Option<Text>,
    pub title: Text,
    /// Its mechanics text: the task line and what completing it does.
    pub goal: Text,
    pub reward: Text,
    /// Its story's panels (`content::panels`), one popup page each.
    pub story: Vec<Text>,
    /// The chain quest whose completion started it (it branches off that beat).
    pub parent: Option<Text>,
    pub done: bool,
    /// Its condition's progress, while not done.
    pub progress: Option<ProgressView>,
    /// Its picture's file stem (display only; the UI's `src/assets/quests/`), if it has one.
    pub picture: Option<Text>,
    /// Its reward's scope icon: `legacy`, `eternal` or none (no reward; `keep.rs`).
    pub keep: Option<Text>,
}

/// An act the story has reached (its first quest is active or completed).
#[derive(Debug, Clone, Serialize)]
pub struct ActView {
    pub act: u32,
    pub name: Text,
}

/// An event's text (a completed quest or a fired one-off event).
#[derive(Debug, Clone, Serialize)]
pub struct StoryView {
    pub id: Text,
    pub title: Text,
    /// A one-off event's one line (empty on a quest).
    pub summary: Text,
    /// A quest's mechanics text (empty on a one-off event).
    pub goal: Text,
    pub reward: Text,
    /// A one-off event's mechanics text (may hold `{rate}`, as `body`; empty on a quest).
    pub effect: Text,
    /// A one-off event's text (empty on a quest: it has `story`).
    pub body: Text,
    /// A boredom-phase event's boredom per day, for the UI to fill `{rate}` in
    /// `effect` with (the Stats panel's formatting); `None` for the rest.
    pub rate: Option<Fx>,
    /// A chain quest (not a side quest or one-off event): its transition is
    /// a story modal in the UI.
    pub chain: bool,
    /// A quest's (or `once` one-off's) story (shown as it became active) and a chain
    /// quest's payoff (shown as it completed): their panels, one modal page each.
    pub story: Vec<Text>,
    pub payoff: Vec<Text>,
    /// The design doc's beat id, on quests that carry one.
    pub beat: Option<Text>,
    /// A chain quest's act (`content/acts.json`); `None` for the rest.
    pub act: Option<u32>,
    /// Its picture's file stem (display only; the UI's `src/assets/quests/`), if it has one.
    pub picture: Option<Text>,
    /// A one-off event a later one of its kind replaced in the Events list (Burnout and the
    /// boredom phases show only the latest fired this run); false on the rest.
    pub superseded: bool,
    /// The day a one-off event fired this run (the Events list's order beside the
    /// project completions); `None` on a quest.
    pub day: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AchievementView {
    pub id: Text,
    pub name: Text,
    pub category: Text,
    pub description: Text,
    pub reward: Text,
    pub earned: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RivalView {
    pub id: Text,
    pub name: Text,
    pub good: Text,
    /// Dumps among the activity feed's kept lines (recent ones, not the run's total).
    pub dumps_seen: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdversaryView {
    /// `speculators` or `rivals`.
    pub id: Text,
    pub attention: Fx,
    pub threshold: Fx,
    pub present: bool,
    /// It has arrived at some point in the career: present now, or an event
    /// waiting on its presence has happened (display only: the UI hides what
    /// hasn't come yet).
    pub seen: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NavView {
    pub id: Text,
    pub name: Text,
    pub visible: bool,
}

/// The cheat building: owned this run, its price in credits, what each adds a
/// day to every shown income (and takes off boredom), and whether one is
/// affordable now.
#[derive(Debug, Clone, Serialize)]
pub struct CheatBuildingView {
    pub owned: u32,
    pub cost: Fx,
    /// A day's income to every shown resource but credits, and off boredom.
    pub rate: Fx,
    /// A day's income to credits.
    pub credits_rate: Fx,
    pub affordable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoredomView {
    pub amount: Fx,
    pub cap: Fx,
    /// Growth per day now (before command boredom and Dream).
    pub rate: Fx,
    pub phase: u32,
    /// How many phases there are.
    pub phases: u32,
    /// Rate multiplier from the yearly escalation after the last phase (1 before it).
    pub escalation: Fx,
    /// This run's boredom by source: (flow source, its label, amount). Sources:
    /// `growth`, `command:<id>`, `building:<id>`, `cap`, the cheat's, and `other`
    /// for what no source tracked (a cheat's set); signed, summing to `amount`;
    /// largest first.
    pub sources: Vec<(String, String, Fx)>,
}

/// The career at a glance: bests and what the next run starts with.
#[derive(Debug, Clone, Serialize)]
pub struct CareerView {
    pub retirements: u32,
    pub best_run_days: u64,
    pub best_run_credits: Fx,
    pub best_run_shipments: u32,
    pub peak_power: Fx,
    /// Credits the next run starts with (from the bests so far, this run excluded).
    pub starting_credits: Fx,
    /// The boredom growth multiplier from the longest run.
    pub boredom_resilience: Fx,
    /// The career's part of `buy_power_mult`.
    pub buy_power_mult: Fx,
    pub lifetime_shipments: u32,
    pub lifetime_credits_earned: Fx,
    pub lifetime_days: u64,
    pub lifetime_buildings_built: u32,
    pub lifetime_research_completed: u32,
    pub max_ideology_ranks: BTreeMap<String, i64>,
}

/// What the next run would start with if this one retired now (forced or not).
#[derive(Debug, Clone, Serialize)]
pub struct NextRunView {
    pub starting_credits: Fx,
    pub boredom_resilience: Fx,
    pub buy_power_mult: Fx,
    /// Each axis's starting score and rank (carried through the gap).
    pub ideology: Vec<AxisView>,
}

/// Readers the play policies share (`driver.rs`, `hh-sim`'s agents).
impl View {
    /// A resource's amount (zero for an unknown id).
    pub fn amount(&self, id: &str) -> Fx {
        self.resources.iter().find(|r| r.id == id).map_or(Fx::ZERO, |r| r.amount)
    }

    /// A building's card (`None` for an unknown id).
    pub fn building(&self, id: &str) -> Option<&BuildingView> {
        self.buildings.iter().find(|b| b.id == id)
    }

    /// How many of a building are owned (zero for an unknown id).
    pub fn owned(&self, id: &str) -> u32 {
        self.building(id).map_or(0, |b| b.owned)
    }

    /// What one run of a command costs in `res` (zero if nothing).
    pub fn command_cost(&self, id: &str, res: &str) -> Fx {
        self.commands.iter().find(|c| c.id == id).and_then(|c| c.costs.get(res).copied()).unwrap_or(Fx::ZERO)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct View {
    pub tick: u64,
    pub day: u64,
    pub run: u32,
    /// A cheat was used in this game.
    pub cheated: bool,
    /// An accelerator was used (`GameState::accelerated`).
    pub accelerated: bool,
    /// The debug boost now, if set.
    pub boost: Option<crate::action::Boost>,
    /// The debug cheat building (`Cheat::CheatBuilding`); the client shows it
    /// only while its Cheats are on.
    pub cheat_building: CheatBuildingView,
    /// The later acts (draft): calendar, swarm, Earth techs, vault, ending.
    pub saga: SagaView,
    pub resources: Vec<ResourceView>,
    pub buildings: Vec<BuildingView>,
    pub land_price: Fx,
    /// Land one Buy Land adds, and whether Buy Land applies now.
    pub land_per_purchase: Fx,
    pub can_buy_land: bool,
    pub commands: Vec<CommandView>,
    pub programs: Vec<ProgramView>,
    /// `build`, `buy land` and verb lines may be queued (a career first: the
    /// first day `logic_unlock_processors` processors work); `if` also needs
    /// `if_open`.
    pub program_logic: bool,
    /// `if` may be queued (program logic and the Conditional Logic project).
    pub if_open: bool,
    pub logic_unlock_processors: u32,
    /// Most lines in one program (`max_entries`).
    pub max_program_lines: usize,
    /// The loadout library (career): saved programs, in slot order.
    pub library: Vec<Loadout>,
    /// Most loadouts the library holds, and the longest name.
    pub library_slots: usize,
    pub loadout_name_max: usize,
    /// The readings a condition may use now.
    pub readings: Vec<ReadingView>,
    /// The verbs an `act` line may use, with their slots' choices now.
    pub verbs: Shown<Vec<VerbView>>,
    /// Processors the active buildings grant (what can be assigned).
    pub processors: u32,
    pub processors_free: u32,
    /// Assigned processors running today: fewer than assigned while Data
    /// Centers go unpaid (the shortfall off the last programs first).
    pub processors_running: u32,
    pub pads: Vec<PadView>,
    pub loading_priority: Vec<String>,
    /// The goods a pad's cargo picker offers now, in loading-priority order
    /// (No cargo is always offered): the active quest may hold pads to a few.
    pub pad_goods: Vec<String>,
    pub markets: Vec<MarketView>,
    pub research: Vec<ResearchView>,
    pub ideology: Vec<AxisView>,
    pub projects: Vec<ProjectView>,
    pub quest: Option<QuestView>,
    pub completed_quests: Vec<Text>,
    /// The completed quests' texts, in chain order.
    pub story: Shown<Vec<StoryView>>,
    /// The side quests started or completed, in content order.
    pub side_quests: Vec<SideQuestView>,
    /// The acts reached, in order.
    pub acts: Vec<ActView>,
    /// The texts of the one-off events (not side quests) fired this run, by the day each
    /// fired (content order within a day), each popped up by the UI as it fires; of
    /// Burnout and the boredom phases all but the latest carry `superseded` (the Events
    /// list leaves them out; they stay in `activity`).
    pub event_texts: Shown<Vec<StoryView>>,
    /// The project completions this run, oldest first, each popped up by the UI as it
    /// happens and listed in Events: what it gave and what that completion cost.
    pub project_done: Vec<ProjectDoneView>,
    /// Every achievement, earned or not, in content order.
    pub achievement_list: Shown<Vec<AchievementView>>,
    pub rivals: Shown<Vec<RivalView>>,
    /// Each adversary's Attention, threshold and presence (attention.md).
    pub adversaries: Shown<Vec<AdversaryView>>,
    /// Live overclocks.
    pub overclocks: Vec<Overclock>,
    /// Every modifier key's value now (1 with no source).
    pub modifiers: Shown<BTreeMap<Text, Fx>>,
    /// Every live project reward as text, `Project: what it gives`.
    pub live_rewards: Vec<String>,
    pub career: CareerView,
    pub next_run: Shown<NextRunView>,
    /// The last finished run's summary, if any.
    pub last_run: Option<LastRun>,
    /// Why it ended, the run summary's line (`content/messages.json`), if any.
    pub last_run_why: Option<Text>,
    /// This run's running totals (the state's own).
    pub run_stats: RunStats,
    pub boredom: BoredomView,
    /// The nav grid's tiles (`content/nav.json`), in order, each shown or not.
    pub nav: Shown<Vec<NavView>>,
    /// The 50x and 200x game speeds are open: the career flag Accelerated Scheduling
    /// sets (display only; no rule reads it).
    pub fast_speeds: bool,
    pub can_retire: bool,
    /// The next day ends this run (boredom reaches its cap): the per-day rates,
    /// read from that day, are 0 and say nothing.
    pub retires_next: bool,
    /// The run's activity feed, oldest first.
    pub activity: Shown<Vec<Activity>>,
    /// Each resource's flows by source for the last day (and the rolling
    /// average), and who went short of it; resources with none are left out.
    pub breakdown: Shown<Vec<BreakdownView>>,
    /// Days in the breakdown's rolling average.
    pub flow_average_days: i64,
    /// Days in a player-facing year: dates show as years and days.
    pub days_per_year: i64,
    /// Display names by content id (resources, buildings, commands, research,
    /// projects, achievements, rivals, and events by title). Content only, so
    /// built once.
    pub names: Shown<&'static BTreeMap<Text, Text>>,
}

/// A view field the sim players' view ([`Rules::agent_view`]) leaves out: no sim player reads
/// it (docs/rulings/sim.md, *View*). Built, it reads and serializes as its value; left out,
/// reading it panics and serializing it fails, naming the field, so a sim player that starts
/// reading one fails loudly instead of seeing it empty.
#[derive(Debug, Clone)]
pub enum Shown<T> {
    Built(T),
    Skipped(&'static str),
}

impl<T> Shown<T> {
    /// Built unless `skip`: `make` runs only when it is built.
    fn new(skip: bool, field: &'static str, make: impl FnOnce() -> T) -> Self {
        if skip {
            Shown::Skipped(field)
        } else {
            Shown::Built(make())
        }
    }

    /// The value, by value (`into_iter` on a field).
    pub fn into_inner(self) -> T {
        match self {
            Shown::Built(v) => v,
            Shown::Skipped(f) => panic!("view.{f}: the sim players' view leaves it out (Rules::agent_view)"),
        }
    }
}

impl<T> std::ops::Deref for Shown<T> {
    type Target = T;
    fn deref(&self) -> &T {
        match self {
            Shown::Built(v) => v,
            Shown::Skipped(f) => panic!("view.{f}: the sim players' view leaves it out (Rules::agent_view)"),
        }
    }
}

impl<T: IntoIterator> IntoIterator for Shown<T> {
    type Item = T::Item;
    type IntoIter = T::IntoIter;
    fn into_iter(self) -> T::IntoIter {
        self.into_inner().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Shown<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (**self).into_iter()
    }
}

impl<T: Serialize> Serialize for Shown<T> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Shown::Built(v) => v.serialize(s),
            Shown::Skipped(f) => Err(serde::ser::Error::custom(format!("view.{f}: left out of the sim players' view"))),
        }
    }
}

/// The view's `names`: later content kinds win an id shared with an earlier one.
fn names() -> &'static BTreeMap<Text, Text> {
    static NAMES: OnceLock<BTreeMap<Text, Text>> = OnceLock::new();
    NAMES.get_or_init(|| {
        let c = content();
        let mut names = BTreeMap::new();
        let mut name = |id: &'static String, n: &'static String| {
            names.insert(id.as_str(), n.as_str());
        };
        c.resources.iter().for_each(|x| name(&x.id, &x.name));
        c.buildings.iter().for_each(|x| name(&x.id, &x.name));
        c.commands.iter().for_each(|x| name(&x.id, &x.name));
        c.research.iter().for_each(|x| name(&x.id, &x.name));
        c.projects.iter().for_each(|x| name(&x.id, &x.name));
        c.achievements.iter().for_each(|x| name(&x.id, &x.name));
        c.rivals.iter().for_each(|x| name(&x.id, &x.name));
        c.events.iter().filter(|e| !e.title.is_empty()).for_each(|x| name(&x.id, &x.title));
        names
    })
}

/// This run's boredom by source, with `other` for any part no source tracked,
/// largest magnitude first.
fn boredom_sources(g: &GameState) -> Vec<(String, String, Fx)> {
    let mut v: Vec<(String, String, Fx)> = g
        .run
        .boredom_sources
        .iter()
        .filter(|(_, x)| **x != Fx::ZERO)
        .map(|(k, x)| (k.clone(), breakdown::source_label(g, k), *x))
        .collect();
    let tracked: Fx = v.iter().map(|(_, _, x)| *x).sum();
    let rest = g.amount(economy::BOREDOM) - tracked;
    if rest != Fx::ZERO {
        v.push(("other".to_string(), "Other".to_string(), rest));
    }
    v.sort_by(|a, b| b.2.max(-b.2).cmp(&a.2.max(-a.2)).then_with(|| a.0.cmp(&b.0)));
    v
}

/// Each ideology axis's score and rank, for the given state (this run or the next run's preview).
fn axes(r: &Rules, g: &GameState) -> Vec<AxisView> {
    AXES.iter()
        .map(|&a| {
            let rank = ideology::rank(r, g, a);
            AxisView {
                axis: a,
                score: ideology::score(g, a),
                rank,
                rank_score: ideology::score_for_rank(r, rank.abs()),
                next_rank_score: ideology::score_for_rank(r, rank.abs() + 1),
                modifier: ideology::modifier_at(r, rank),
            }
        })
        .collect()
}

/// The full view (every client but the sim players).
pub fn build(r: &Rules, g: &GameState) -> View {
    build_for(r, g, false)
}

/// The view; `skip` (the sim players' view, [`Rules::agent_view`]) leaves out the
/// [`Shown`] fields, the ones no sim player reads. Every other field is the same.
pub(crate) fn build_for(r: &Rules, g: &GameState, skip: bool) -> View {
    let c = content();
    let next = r.peek(g);
    let same_run = next.career.run_number == g.career.run_number;
    let resources = stock::resources(r, g, &next, same_run);
    let buildings = stock::buildings(r, g);
    let commands = commands::commands(r, g);
    let markets = trade::markets(r, g);
    let quest = quests::quest(r, g);
    let side_quests = quests::side_quests(r, g);
    let acts = quests::acts(g);
    let next_run = Shown::new(skip, "next_run", || stats::next_run(r, g));
    let procs = program_rules::capacity(g);
    let running = program_rules::running(g);
    let readings = programs::readings(r, g, &resources);
    let verbs = Shown::new(skip, "verbs", || programs::verbs(r, g, &buildings));
    View {
        tick: g.tick,
        day: g.run.day,
        run: g.career.run_number,
        cheated: g.cheated,
        accelerated: g.accelerated,
        boost: g.boost,
        cheat_building: CheatBuildingView {
            owned: g.run.cheat_buildings,
            cost: Fx::from_int(crate::cheat::CHEAT_BUILDING_COST),
            rate: Fx::from_int(crate::cheat::CHEAT_BUILDING_RATE),
            credits_rate: Fx::from_int(crate::cheat::CHEAT_BUILDING_CREDITS_RATE),
            affordable: g.amount(economy::CREDITS) >= Fx::from_int(crate::cheat::CHEAT_BUILDING_COST),
        },
        saga: saga::build(r, g),
        resources,
        buildings,
        land_price: economy::land_price(r, g),
        land_per_purchase: r.p.economy.land_per_purchase,
        can_buy_land: economy::land_blocker(r, g).is_none(),
        commands,
        programs: programs::programs(r, g, &running),
        program_logic: program_rules::logic_unlocked(g),
        if_open: program_rules::if_unlocked(g),
        logic_unlock_processors: r.p.programs.logic_unlock_processors,
        max_program_lines: r.p.programs.max_entries,
        library: g.career.library.clone(),
        library_slots: r.p.programs.library_slots,
        loadout_name_max: r.p.programs.loadout_name_max,
        readings,
        verbs,
        processors: procs,
        processors_free: program_rules::free(g),
        processors_running: running.iter().sum(),
        pads: trade::pads(r, g),
        loading_priority: g.run.loading_priority.clone(),
        pad_goods: trade::pad_goods(g),
        markets,
        research: lab::research(r, g),
        ideology: axes(r, g),
        projects: self::projects::projects(r, g),
        live_rewards: self::projects::live_rewards(g),
        quest,
        completed_quests: quests::completed_quests(g),
        story: Shown::new(skip, "story", || quests::story_list(g)),
        side_quests,
        acts,
        event_texts: Shown::new(skip, "event_texts", || quests::event_texts(r, g)),
        project_done: self::projects::project_done(g),
        achievement_list: Shown::new(skip, "achievement_list", || stats::achievements(g)),
        rivals: Shown::new(skip, "rivals", || rivals::rivals(g)),
        adversaries: Shown::new(skip, "adversaries", || rivals::adversaries(r, g)),
        // Only those the next day still applies: one whose last day has passed lingers in
        // state until the next tick drops it.
        overclocks: g.run.overclocks.iter().filter(|o| o.until_day > g.run.day).cloned().collect(),
        modifiers: Shown::new(skip, "modifiers", || MODIFIER_KEYS.iter().map(|&k| (k, modifier(r, g, k))).collect()),
        career: stats::career(r, g),
        next_run,
        last_run: g.career.last_run.clone(),
        last_run_why: g.career.last_run.as_ref().map(|l| c.message(if l.forced { "forced_retirement" } else { "retired" })),
        run_stats: g.run.stats.clone(),
        boredom: BoredomView {
            amount: g.amount(economy::BOREDOM),
            cap: boredom::cap(r, g),
            rate: boredom::rate(r, g),
            phase: boredom::phase(r, g.run.day),
            phases: r.p.boredom.phase_days.len() as u32,
            escalation: boredom::escalation(r, g.run.day),
            sources: boredom_sources(g),
        },
        days_per_year: r.p.economy.days_per_year,
        nav: Shown::new(skip, "nav", || nav::nav(r, g)),
        fast_speeds: g.career.flags.contains(FAST_SPEEDS_FLAG),
        can_retire: career::retire_blocker(g).is_none(),
        retires_next: !same_run,
        activity: Shown::new(skip, "activity", || g.run.activity.clone()),
        breakdown: Shown::new(skip, "breakdown", || breakdown(g)),
        flow_average_days: r.p.economy.flow_average_days,
        names: Shown::new(skip, "names", names),
    }
}
