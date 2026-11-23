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
use crate::content::{content, Amounts, BuildingEffect, CommandEffect, Condition, NavCondition, Requirement, AXES, MODIFIER_KEYS};
use crate::fx::Fx;
use crate::modifiers::modifier;
use crate::modules::*;
use crate::rules::Rules;
use crate::state::{Activity, GameState, LastRun, Loadout, Overclock, RunStats, Stall};

use crate::modules::programs as program_rules;
use breakdown::{breakdown, short_view};

mod breakdown;
// The programs panel; the game's own programs module is `program_rules` here.
mod programs;
// The Projects panel; the game's own projects module is `modules::projects`.
mod projects;
mod saga;

pub use self::projects::{condition_text, ProjectView};
pub use saga::{SagaView, TechView};

pub use breakdown::{BreakdownView, FlowRow, ShortView};
pub use programs::{Choice, ClauseView, ConditionView, LinesRun, ProgramView, ReadingView, VerbView};

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
    /// The ideology axis whose rank discounts it, if any.
    pub ideology: Option<Text>,
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
    pub description: Text,
    pub unlocked: bool,
    pub visible: bool,
    pub costs: BTreeMap<String, Fx>,
    pub produces: BTreeMap<String, Fx>,
    pub boredom: Fx,
    /// A per-target command folded into one verb (docs/rulings.md, the flat
    /// template): its verb (`Buy`, `Promote`, `Fund`, `Overclock`) and its
    /// target's label (the name past the verb). `None` for the rest.
    pub verb: Option<&'static str>,
    pub target: Option<Text>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PadView {
    pub good: String,
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
}

#[derive(Debug, Clone, Serialize)]
pub struct AxisView {
    pub axis: &'static str,
    pub score: Fx,
    pub rank: i64,
    /// Scores (in size) where this rank and the next start, for a progress bar.
    pub rank_score: Fx,
    pub next_rank_score: Fx,
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
    pub summary: Text,
    pub body: Text,
    pub progress: Option<ProgressView>,
    pub sub_objectives: Vec<SubObjectiveView>,
}

/// An event's text (a completed quest or a fired one-off event).
#[derive(Debug, Clone, Serialize)]
pub struct StoryView {
    pub id: Text,
    pub title: Text,
    pub summary: Text,
    /// Owned only when a `{rate}` was filled in.
    pub body: Cow<'static, str>,
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
    pub rate: Fx,
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

    /// How many of a building are owned (zero for an unknown id).
    pub fn owned(&self, id: &str) -> u32 {
        self.buildings.iter().find(|b| b.id == id).map_or(0, |b| b.owned)
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
    pub verbs: Vec<VerbView>,
    /// Processors the active buildings grant (what can be assigned).
    pub processors: u32,
    pub processors_free: u32,
    /// Assigned processors running today: fewer than assigned while Data
    /// Centers go unpaid (the shortfall off the last programs first).
    pub processors_running: u32,
    pub pads: Vec<PadView>,
    pub loading_priority: Vec<String>,
    pub markets: Vec<MarketView>,
    pub research: Vec<ResearchView>,
    pub ideology: Vec<AxisView>,
    pub projects: Vec<ProjectView>,
    pub quest: Option<QuestView>,
    pub completed_quests: Vec<Text>,
    /// The completed quests' texts, in chain order.
    pub story: Vec<StoryView>,
    /// The texts of the one-off events fired this run, in content order, but of the
    /// boredom-phase events only the latest phase's: each phase supersedes the last
    /// (the earlier ones stay in `activity`).
    pub event_texts: Vec<StoryView>,
    /// Every achievement, earned or not, in content order.
    pub achievement_list: Vec<AchievementView>,
    pub rivals: Vec<RivalView>,
    /// Each adversary's Attention, threshold and presence (attention.md).
    pub adversaries: Vec<AdversaryView>,
    /// Live overclocks.
    pub overclocks: Vec<Overclock>,
    /// Every modifier key's value now (1 with no source).
    pub modifiers: BTreeMap<Text, Fx>,
    /// Every live project reward as text, `Project: what it gives`.
    pub live_rewards: Vec<String>,
    pub career: CareerView,
    pub next_run: NextRunView,
    /// The last finished run's summary, if any.
    pub last_run: Option<LastRun>,
    /// This run's running totals (the state's own).
    pub run_stats: RunStats,
    pub boredom: BoredomView,
    /// The nav grid's tiles (`content/nav.json`), in order, each shown or not.
    pub nav: Vec<NavView>,
    pub can_retire: bool,
    /// The next day ends this run (boredom reaches its cap): the per-day rates,
    /// read from that day, are 0 and say nothing.
    pub retires_next: bool,
    /// The run's activity feed, oldest first.
    pub activity: Vec<Activity>,
    /// Each resource's flows by source for the last day (and the rolling
    /// average), and who went short of it; resources with none are left out.
    pub breakdown: Vec<BreakdownView>,
    /// Days in the breakdown's rolling average.
    pub flow_average_days: i64,
    /// Display names by content id (resources, buildings, commands, research,
    /// projects, achievements, rivals, and events by title). Content only, so
    /// built once.
    pub names: &'static BTreeMap<Text, Text>,
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


fn progress(p: Option<(Fx, Fx)>) -> Option<ProgressView> {
    p.map(|(now, target)| ProgressView { now, target })
}

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
            }
        })
        .collect()
}

fn story(e: &'static crate::content::Event) -> StoryView {
    StoryView { id: &e.id, title: &e.title, summary: &e.summary, body: Cow::Borrowed(&e.body) }
}

/// An event's text with its `{rate}` filled in: for a boredom-phase event, that phase's
/// boredom per day under this run's multiplier (display only, to two places).
fn event_story(r: &Rules, g: &GameState, e: &'static crate::content::Event) -> StoryView {
    let mut s = story(e);
    if let crate::content::Trigger::BoredomPhase { phase } = e.trigger {
        let rates = &r.p.boredom.phase_rates;
        let i = (phase.max(1) as usize - 1).min(rates.len() - 1);
        let shown = self::projects::short(rates[i] * boredom::multiplier(r, g));
        s.body = Cow::Owned(s.body.replace("{rate}", &shown));
    }
    s
}

pub fn build(r: &Rules, g: &GameState) -> View {
    let c = content();
    let next = r.peek(g);
    let same_run = next.career.run_number == g.career.run_number;
    // The highest boredom phase whose event fired this run: Story lists only its text.
    let latest_phase = c
        .events
        .iter()
        .filter(|e| !e.silent && g.run.events.contains(&e.id))
        .filter_map(|e| match e.trigger {
            crate::content::Trigger::BoredomPhase { phase } => Some(phase),
            _ => None,
        })
        .max();
    let resources: Vec<ResourceView> = c
        .resources
        .iter()
        .map(|res| {
            ResourceView {
                id: &res.id,
                name: &res.name,
                amount: g.amount(&res.id),
                cap: economy::cap(r, g, &res.id),
                per_day: if same_run { next.run.day_net.get(&res.id).copied().unwrap_or(Fx::ZERO) } else { Fx::ZERO },
                change_per_day: if same_run { next.amount(&res.id) - g.amount(&res.id) } else { Fx::ZERO },
                overflow: if same_run { next.run.today.overflow.get(&res.id).copied().unwrap_or(Fx::ZERO) } else { Fx::ZERO },
                unmet: if same_run { next.run.today.unmet.get(&res.id).copied().unwrap_or(Fx::ZERO) } else { Fx::ZERO },
                visible: economy::shown(g, res),
                grantable: crate::cheat::grantable(&res.id),
            }
        })
        .collect();
    let buildings: Vec<BuildingView> = c
        .buildings
        .iter()
        .map(|b| {
            let h = g.run.buildings.get(&b.id).copied().unwrap_or_default();
            let cost = economy::price(r, g, b);
            let mut short: Vec<String> =
                cost.iter().filter(|(res, &v)| g.amount(res) < v).map(|(res, _)| res.clone()).collect();
            if g.amount(economy::LAND) < Fx::from_int(b.land as i64) {
                short.push(economy::LAND.to_string());
            }
            let effect = |grant: bool| {
                b.effects
                    .iter()
                    .filter_map(|e| match e {
                        BuildingEffect::Store { resource, amount } if !grant => Some((resource.as_str(), *amount)),
                        BuildingEffect::Grant { resource, amount } if grant => Some((resource.as_str(), *amount)),
                        _ => None,
                    })
                    .collect()
            };
            BuildingView {
                id: &b.id,
                name: &b.name,
                description: &b.description,
                category: &b.category,
                ideology: b.ideology.as_deref(),
                owned: h.owned,
                active: h.active,
                working: g.working(&b.id),
                bonus: h.bonus,
                max_count: b.max_count,
                cost,
                land: b.land,
                short,
                produces: &b.produces,
                upkeep: &b.upkeep,
                requires: &b.requires,
                stores: effect(false),
                grants: effect(true),
                affordable: economy::buy_blocker(r, g, b).is_none(),
                at_limit: economy::at_limit(g, b),
                unlocked: economy::unlocked(g, b),
                visible: economy::visible(g, b),
                stall: g.run.today.stalls.get(&b.id).copied(),
                starved: {
                    let k = format!("building:{}", b.id);
                    g.run.today.shorts.get(&k).map(|s| short_view(g, &k, s))
                },
                buy: Action::Buy { building: b.id.clone() },
            }
        })
        .collect();
    let commands = c
        .commands
        .iter()
        .map(|cmd| {
            let (costs, produces) = program_rules::flows(r, g, cmd);
            let verb = match &cmd.effect {
                Some(CommandEffect::Promote { .. }) => Some("Promote"),
                Some(CommandEffect::Fund { .. }) => Some("Fund"),
                Some(CommandEffect::Overclock { .. }) => Some("Overclock"),
                None if cmd.costs.contains_key(economy::CREDITS) && cmd.produces.len() == 1 => Some("Buy"),
                _ => None,
            };
            let target = verb.and_then(|vb| cmd.name.strip_prefix(vb)).map(str::trim);
            CommandView {
                verb,
                target,
                id: &cmd.id,
                name: &cmd.name,
                description: &cmd.description,
                unlocked: program_rules::unlocked(g, cmd),
                visible: program_rules::visible(g, cmd),
                costs,
                produces,
                boredom: program_rules::boredom_cost(g, cmd),
            }
        })
        .collect();
    let aware = research::reveals_demand(g);
    let markets = shipments::goods(r)
        .into_iter()
        .filter_map(|good| {
            let m = g.run.markets.get(&good)?;
            Some(MarketView {
                name: c.resource(&good).map_or(good.clone(), |x| x.name.clone()),
                demand: aware.then_some(m.demand),
                tier: demand::TIERS[demand::tier(r, m.demand)],
                speculators: m.speculators_seen.then_some(m.speculators),
                good,
            })
        })
        .collect();
    let quest = events::active_quest(g).map(|q| QuestView {
        id: &q.id,
        kind: events::kind(&q.condition),
        title: &q.title,
        summary: &q.summary,
        body: &q.body,
        progress: progress(events::progress(r, g, &q.condition)),
        sub_objectives: match &q.condition {
            Condition::AllOf { sub_objectives } => sub_objectives
                .iter()
                .map(|s| SubObjectiveView {
                    id: &s.id,
                    label: &s.label,
                    done: g.career.completed_sub_objectives.contains(&events::sub_key(&q.id, &s.id)),
                    progress: progress(events::progress(r, g, &s.condition)),
                })
                .collect(),
            _ => Vec::new(),
        },
    });
    // The next run if this one retired now.
    let mut after = g.clone();
    career::retire(r, &mut after, false);
    let next_run = NextRunView {
        starting_credits: career::starting_credits(r, &after),
        boredom_resilience: career::boredom_resilience(r, &after),
        buy_power_mult: career::buy_power_mult(r, &after),
        ideology: axes(r, &after),
    };
    let procs = program_rules::capacity(g);
    let running = program_rules::running(g);
    let readings = programs::readings(r, g, &resources);
    let verbs = programs::verbs(r, g, &buildings);
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
        pads: (0..g.run.pads.len())
            .map(|i| {
                let p = &g.run.pads[i];
                PadView {
                    good: p.good.clone(),
                    paused: p.paused,
                    active: shipments::pad_active(g, i),
                    unpaid: i < g.active(shipments::LAUNCH_PAD) as usize && !shipments::pad_active(g, i),
                    cargo: p.cargo,
                    capacity: r.p.shipments.pad_cargo_capacity,
                    cooldown: p.cooldown,
                    fuel: r.p.shipments.fuel_per_pad,
                    load: shipments::load_amount(r, g),
                    payout: shipments::payout(r, g, i),
                    can_launch: shipments::can_launch(r, g, i),
                    fueled: g.amount(shipments::PROPELLANT) >= r.p.shipments.fuel_per_pad,
                    ready: shipments::ready(r, g, i),
                    switch_returns: shipments::switch_returns(r, g, i),
                }
            })
            .collect(),
        loading_priority: g.run.loading_priority.clone(),
        markets,
        research: c
            .research
            .iter()
            .map(|item| {
                let cost = research::price(r, g, item);
                ResearchView {
                    id: &item.id,
                    name: &item.name,
                    description: &item.description,
                    category: &item.category,
                    cost,
                    requires: item.requires.as_deref(),
                    done: g.run.research.contains(&item.id),
                    visible: research::visible(g, item),
                    affordable: research::blocker(r, g, item).is_none(),
                }
            })
            .collect(),
        ideology: axes(r, g),
        projects: self::projects::projects(r, g),
        live_rewards: self::projects::live_rewards(g),
        quest,
        completed_quests: c
            .events
            .iter()
            .filter(|e| g.career.completed_quests.contains(&e.id))
            .map(|e| e.id.as_str())
            .collect(),
        story: c.events.iter().filter(|e| g.career.completed_quests.contains(&e.id)).map(story).collect(),
        event_texts: c
            .events
            .iter()
            .filter(|e| !e.is_quest() && !e.silent && g.run.events.contains(&e.id))
            .filter(|e| match e.trigger {
                crate::content::Trigger::BoredomPhase { phase } => Some(phase) == latest_phase,
                _ => true,
            })
            .map(|e| event_story(r, g, e))
            .collect(),
        achievement_list: c
            .achievements
            .iter()
            .map(|a| AchievementView {
                id: &a.id,
                name: &a.name,
                category: &a.category,
                description: &a.description,
                reward: &a.reward_description,
                earned: g.career.achievements.contains(&a.id),
            })
            .collect(),
        rivals: c
            .rivals
            .iter()
            .map(|rv| RivalView {
                id: &rv.id,
                name: &rv.name,
                good: &rv.good,
                dumps_seen: g.run.activity.iter().filter(|a| a.kind == "rival_dump" && a.id == rv.id).count() as u32,
            })
            .collect(),
        adversaries: crate::modules::attention::ADVERSARIES
            .iter()
            .map(|&id| AdversaryView {
                id,
                attention: crate::modules::attention::attention(g, id),
                threshold: crate::modules::attention::threshold_of(r, id),
                present: crate::modules::attention::present(r, g, id),
            })
            .collect(),
        // Only those the next day still applies: one whose last day has passed lingers in
        // state until the next tick drops it.
        overclocks: g.run.overclocks.iter().filter(|o| o.until_day > g.run.day).cloned().collect(),
        modifiers: MODIFIER_KEYS.iter().map(|&k| (k, modifier(r, g, k))).collect(),
        career: CareerView {
            retirements: g.career.retirements,
            best_run_days: g.career.best_run_days,
            best_run_credits: g.career.best_run_credits,
            best_run_shipments: g.career.best_run_shipments,
            peak_power: g.career.peak_power,
            starting_credits: career::starting_credits(r, g),
            boredom_resilience: career::boredom_resilience(r, g),
            buy_power_mult: career::buy_power_mult(r, g),
            lifetime_shipments: g.career.lifetime.shipments,
            lifetime_credits_earned: g.career.lifetime.credits_earned,
            lifetime_days: g.career.lifetime.days,
            lifetime_buildings_built: g.career.lifetime.buildings_built,
            lifetime_research_completed: g.career.lifetime.research_completed,
            max_ideology_ranks: g.career.max_ideology_ranks.clone(),
        },
        next_run,
        last_run: g.career.last_run.clone(),
        run_stats: g.run.stats.clone(),
        boredom: BoredomView {
            amount: g.amount(economy::BOREDOM),
            cap: boredom::cap(r, g),
            rate: boredom::rate(r, g),
            phase: boredom::phase(r, g.run.day),
            phases: r.p.boredom.phase_days.len() as u32,
        },
        nav: c
            .nav
            .iter()
            .map(|t| NavView {
                id: &t.id,
                name: &t.name,
                visible: match &t.visible {
                    NavCondition::Always => true,
                    NavCondition::Panel { panel } => g.run.enabled_panels.contains(panel),
                    NavCondition::BuildingKnown { building } => {
                        g.owned(building) > 0 || g.career.owned_buildings.contains(building)
                    }
                },
            })
            .collect(),
        can_retire: career::retire_blocker(g).is_none(),
        retires_next: !same_run,
        activity: g.run.activity.clone(),
        breakdown: breakdown(g),
        flow_average_days: r.p.economy.flow_average_days,
        names: names(),
    }
}
