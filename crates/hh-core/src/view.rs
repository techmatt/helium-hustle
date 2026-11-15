//! What a client (the UI or an agent) shows and can do: derived display data
//! (prices, rates, caps, what is affordable, unlocked or visible, the action
//! for each), so no client computes a rule. Never read by the rules, not part
//! of the state, the fingerprint or a replay.

use serde::Serialize;
use std::collections::BTreeMap;

use crate::action::Action;
use crate::content::{content, BuildingEffect, Condition, NavCondition, Requirement, AXES, MODIFIER_KEYS};
use crate::fx::Fx;
use crate::modifiers::modifier;
use crate::modules::*;
use crate::rules::Rules;
use crate::state::{Activity, Entry, GameState, LastRun, LineOutcome, LogEntry, Overclock, Stall};

#[derive(Debug, Clone, Serialize)]
pub struct ResourceView {
    pub id: String,
    pub name: String,
    pub amount: Fx,
    /// Most that can be held, if capped.
    pub cap: Option<Fx>,
    /// Change over the next day if nothing else is done.
    pub per_day: Fx,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildingView {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    /// The ideology axis whose rank discounts it, if any.
    pub ideology: Option<String>,
    pub owned: u32,
    pub active: u32,
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
    pub produces: BTreeMap<String, Fx>,
    pub upkeep: BTreeMap<String, Fx>,
    pub stores: BTreeMap<String, Fx>,
    /// Resources one active copy grants (processors).
    pub grants: BTreeMap<String, Fx>,
    /// What unlocks it (content).
    pub requires: Requirement,
    /// Buyable now (unlocked, land, price, limit).
    pub affordable: bool,
    pub unlocked: bool,
    pub visible: bool,
    /// Its stall today, if any.
    pub stall: Option<Stall>,
    /// The action that buys one.
    pub buy: Action,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandView {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlocked: bool,
    pub visible: bool,
    pub costs: BTreeMap<String, Fx>,
    pub produces: BTreeMap<String, Fx>,
    pub boredom: Fx,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgramView {
    pub entries: Vec<Entry>,
    pub processors: u32,
    /// The line the next processor runs, and runs done of it.
    pub entry: u32,
    pub step: u32,
    /// What each line did the last time it ran today (`None`: not run today).
    pub outcomes: Vec<Option<LineOutcome>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PadView {
    pub good: String,
    pub paused: bool,
    pub active: bool,
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
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub cost: Fx,
    /// Research it needs first, if any.
    pub requires: Option<String>,
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

#[derive(Debug, Clone, Serialize)]
pub struct ProjectView {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tier: String,
    pub unlocked: bool,
    pub completed: bool,
    pub costs: BTreeMap<String, Fx>,
    pub progress: BTreeMap<String, Fx>,
    pub rates: BTreeMap<String, Fx>,
    /// The most a rate may be (per resource, per day).
    pub max_rate: Fx,
}

/// A countable condition's progress.
#[derive(Debug, Clone, Serialize)]
pub struct ProgressView {
    pub now: Fx,
    pub target: Fx,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubObjectiveView {
    pub id: String,
    pub label: String,
    pub done: bool,
    pub progress: Option<ProgressView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuestView {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub progress: Option<ProgressView>,
    pub sub_objectives: Vec<SubObjectiveView>,
}

/// An event's text (a completed quest or a fired one-off event).
#[derive(Debug, Clone, Serialize)]
pub struct StoryView {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AchievementView {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub reward: String,
    pub earned: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RivalView {
    pub id: String,
    pub name: String,
    pub good: String,
    /// Dumps this run so far (from the activity feed's kept lines).
    pub dumps_seen: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct NavView {
    pub id: String,
    pub name: String,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoredomView {
    pub amount: Fx,
    pub cap: Fx,
    /// Growth per day now (before command boredom and Dream).
    pub rate: Fx,
    pub phase: u32,
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
    /// Each axis's starting score and rank (the ideology head start).
    pub ideology: Vec<AxisView>,
}

/// This run's running totals.
#[derive(Debug, Clone, Serialize)]
pub struct RunStatsView {
    pub credits_earned: Fx,
    pub shipments: u32,
    pub buildings_built: u32,
    pub research_completed: u32,
    pub peak_energy: Fx,
    /// Everything produced this run, by resource (before caps).
    pub produced: BTreeMap<String, Fx>,
}

#[derive(Debug, Clone, Serialize)]
pub struct View {
    pub tick: u64,
    pub day: u64,
    pub run: u32,
    /// A cheat was used in this game.
    pub cheated: bool,
    pub resources: Vec<ResourceView>,
    pub buildings: Vec<BuildingView>,
    pub land_price: Fx,
    /// Land one Buy Land adds, and whether Buy Land applies now.
    pub land_per_purchase: Fx,
    pub can_buy_land: bool,
    pub commands: Vec<CommandView>,
    pub programs: Vec<ProgramView>,
    pub processors: u32,
    pub processors_free: u32,
    pub pads: Vec<PadView>,
    pub loading_priority: Vec<String>,
    pub markets: Vec<MarketView>,
    pub research: Vec<ResearchView>,
    pub ideology: Vec<AxisView>,
    pub projects: Vec<ProjectView>,
    pub quest: Option<QuestView>,
    pub completed_quests: Vec<String>,
    /// The completed quests' texts, in chain order.
    pub story: Vec<StoryView>,
    /// One-off events fired this run, in content order.
    pub events: Vec<String>,
    /// Their texts, in content order.
    pub event_texts: Vec<StoryView>,
    pub achievements: Vec<String>,
    /// Every achievement, earned or not, in content order.
    pub achievement_list: Vec<AchievementView>,
    pub rivals: Vec<RivalView>,
    /// Live overclocks.
    pub overclocks: Vec<Overclock>,
    /// Every modifier key's value now (1 with no source).
    pub modifiers: BTreeMap<String, Fx>,
    pub career: CareerView,
    pub next_run: NextRunView,
    /// The last finished run's summary, if any.
    pub last_run: Option<LastRun>,
    pub run_stats: RunStatsView,
    pub boredom: BoredomView,
    /// Unlocked nav panels (retirement, projects, ideologies).
    pub panels: Vec<String>,
    /// The nav grid's tiles (`content/nav.json`), in order, each shown or not.
    pub nav: Vec<NavView>,
    pub can_retire: bool,
    pub log: Vec<LogEntry>,
    /// The run's activity feed, oldest first.
    pub activity: Vec<Activity>,
    /// Display names by content id (resources, buildings, commands, research,
    /// projects, achievements, rivals, and events by title).
    pub names: BTreeMap<String, String>,
}

const TIERS: &[&str] = &["LOW", "MEDIUM", "HIGH", "VERY HIGH"];

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

fn story(e: &crate::content::Event) -> StoryView {
    StoryView { id: e.id.clone(), title: e.title.clone(), summary: e.summary.clone(), body: e.body.clone() }
}

pub fn build(r: &Rules, g: &GameState) -> View {
    let c = content();
    let next = r.peek(g);
    let same_run = next.career.run_number == g.career.run_number;
    let resources = c
        .resources
        .iter()
        .map(|res| {
            let made_by_owned = c.buildings.iter().any(|b| {
                b.produces.contains_key(&res.id)
                    && (g.owned(&b.id) > 0 || g.career.owned_buildings.contains(&b.id))
            });
            let made_by_used = c
                .commands
                .iter()
                .any(|cmd| cmd.produces.contains_key(&res.id) && g.career.used_commands.contains(&cmd.id));
            ResourceView {
                id: res.id.clone(),
                name: res.name.clone(),
                amount: g.amount(&res.id),
                cap: economy::cap(r, g, &res.id),
                per_day: if same_run { next.amount(&res.id) - g.amount(&res.id) } else { Fx::ZERO },
                visible: res.always_visible || made_by_owned || made_by_used || g.amount(&res.id) > Fx::ZERO,
            }
        })
        .collect();
    let buildings = c
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
                        BuildingEffect::Store { resource, amount } if !grant => Some((resource.clone(), *amount)),
                        BuildingEffect::Grant { resource, amount } if grant => Some((resource.clone(), *amount)),
                        _ => None,
                    })
                    .collect()
            };
            BuildingView {
                id: b.id.clone(),
                name: b.name.clone(),
                description: b.description.clone(),
                category: b.category.clone(),
                ideology: b.ideology.clone(),
                owned: h.owned,
                active: h.active,
                bonus: h.bonus,
                max_count: b.max_count,
                cost,
                land: b.land,
                short,
                produces: b.produces.clone(),
                upkeep: b.upkeep.clone(),
                requires: b.requires.clone(),
                stores: effect(false),
                grants: effect(true),
                affordable: economy::buy_blocker(r, g, b).is_none(),
                unlocked: economy::unlocked(g, b),
                visible: economy::visible(g, b),
                stall: g.run.today.stalls.get(&b.id).copied(),
                buy: Action::Buy { building: b.id.clone() },
            }
        })
        .collect();
    let commands = c
        .commands
        .iter()
        .map(|cmd| {
            let (costs, produces) = programs::flows(r, g, cmd);
            CommandView {
                id: cmd.id.clone(),
                name: cmd.name.clone(),
                description: cmd.description.clone(),
                unlocked: programs::unlocked(g, cmd),
                visible: programs::visible(g, cmd),
                costs,
                produces,
                boredom: programs::boredom_cost(g, cmd),
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
                tier: TIERS[demand::tier(r, m.demand).min(TIERS.len() - 1)],
                speculators: m.speculators_seen.then_some(m.speculators),
                good,
            })
        })
        .collect();
    let quest = events::active_quest(g).map(|q| QuestView {
        id: q.id.clone(),
        title: q.title.clone(),
        summary: q.summary.clone(),
        body: q.body.clone(),
        progress: progress(events::progress(r, g, &q.condition)),
        sub_objectives: match &q.condition {
            Condition::AllOf { sub_objectives } => sub_objectives
                .iter()
                .map(|s| SubObjectiveView {
                    id: s.id.clone(),
                    label: s.label.clone(),
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
    let mut names = BTreeMap::new();
    let mut name = |id: &str, n: &str| {
        names.insert(id.to_string(), n.to_string());
    };
    c.resources.iter().for_each(|x| name(&x.id, &x.name));
    c.buildings.iter().for_each(|x| name(&x.id, &x.name));
    c.commands.iter().for_each(|x| name(&x.id, &x.name));
    c.research.iter().for_each(|x| name(&x.id, &x.name));
    c.projects.iter().for_each(|x| name(&x.id, &x.name));
    c.achievements.iter().for_each(|x| name(&x.id, &x.name));
    c.rivals.iter().for_each(|x| name(&x.id, &x.name));
    c.events.iter().filter(|e| !e.title.is_empty()).for_each(|x| name(&x.id, &x.title));
    let procs = programs::processors(g);
    View {
        tick: g.tick,
        day: g.run.day,
        run: g.career.run_number,
        cheated: g.cheated,
        resources,
        buildings,
        land_price: economy::land_price(r, g),
        land_per_purchase: r.p.economy.land_per_purchase,
        can_buy_land: r.is_legal(g, &Action::BuyLand),
        commands,
        programs: g
            .run
            .programs
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let today = g.run.today.lines.get(&(i as u32));
                ProgramView {
                    entries: p.entries.clone(),
                    processors: p.processors,
                    entry: p.entry,
                    step: p.step,
                    outcomes: (0..p.entries.len() as u32)
                        .map(|e| today.and_then(|t| t.get(&e)).copied())
                        .collect(),
                }
            })
            .collect(),
        processors: procs,
        processors_free: procs.saturating_sub(programs::assigned(g)),
        pads: (0..g.run.pads.len())
            .map(|i| {
                let p = &g.run.pads[i];
                PadView {
                    good: p.good.clone(),
                    paused: p.paused,
                    active: shipments::pad_active(g, i),
                    cargo: p.cargo,
                    capacity: r.p.shipments.pad_cargo_capacity,
                    cooldown: p.cooldown,
                    fuel: r.p.shipments.fuel_per_pad,
                    load: shipments::load_amount(r, g),
                    payout: shipments::payout(r, g, i),
                    can_launch: shipments::can_launch(r, g, i),
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
                    id: item.id.clone(),
                    name: item.name.clone(),
                    description: item.description.clone(),
                    category: item.category.clone(),
                    cost,
                    requires: item.requires.clone(),
                    done: g.run.research.contains(&item.id),
                    visible: research::visible(g, item),
                    affordable: research::blocker(r, g, item).is_none(),
                }
            })
            .collect(),
        ideology: axes(r, g),
        projects: c
            .projects
            .iter()
            .map(|p| ProjectView {
                id: p.id.clone(),
                name: p.name.clone(),
                description: p.description.clone(),
                tier: p.tier.clone(),
                unlocked: projects::unlocked(g, p),
                completed: projects::completed(g, &p.id),
                costs: p.costs.clone(),
                progress: projects::progress(g, p).cloned().unwrap_or_default(),
                rates: g.run.project_rates.get(&p.id).cloned().unwrap_or_default(),
                max_rate: r.p.projects.max_drain_rate,
            })
            .collect(),
        quest,
        completed_quests: c
            .events
            .iter()
            .filter(|e| g.career.completed_quests.contains(&e.id))
            .map(|e| e.id.clone())
            .collect(),
        story: c.events.iter().filter(|e| g.career.completed_quests.contains(&e.id)).map(story).collect(),
        events: c
            .events
            .iter()
            .filter(|e| !e.is_quest() && !e.silent && g.run.events.contains(&e.id))
            .map(|e| e.id.clone())
            .collect(),
        event_texts: c
            .events
            .iter()
            .filter(|e| !e.is_quest() && !e.silent && g.run.events.contains(&e.id))
            .map(story)
            .collect(),
        achievements: g.career.achievements.iter().cloned().collect(),
        achievement_list: c
            .achievements
            .iter()
            .map(|a| AchievementView {
                id: a.id.clone(),
                name: a.name.clone(),
                category: a.category.clone(),
                description: a.description.clone(),
                reward: a.reward_description.clone(),
                earned: g.career.achievements.contains(&a.id),
            })
            .collect(),
        rivals: c
            .rivals
            .iter()
            .map(|rv| RivalView {
                id: rv.id.clone(),
                name: rv.name.clone(),
                good: rv.good.clone(),
                dumps_seen: g.run.activity.iter().filter(|a| a.kind == "rival_dump" && a.id == rv.id).count() as u32,
            })
            .collect(),
        overclocks: g.run.overclocks.clone(),
        modifiers: MODIFIER_KEYS.iter().map(|&k| (k.to_string(), modifier(r, g, k))).collect(),
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
        run_stats: RunStatsView {
            credits_earned: g.run.stats.credits_earned,
            shipments: g.run.stats.shipments,
            buildings_built: g.run.stats.buildings_built,
            research_completed: g.run.stats.research_completed,
            peak_energy: g.run.stats.peak_energy,
            produced: g.run.stats.produced.clone(),
        },
        boredom: BoredomView {
            amount: g.amount(economy::BOREDOM),
            cap: boredom::cap(r, g),
            rate: boredom::rate(r, g),
            phase: boredom::phase(r, g.run.day),
        },
        panels: g.run.enabled_panels.iter().cloned().collect(),
        nav: c
            .nav
            .iter()
            .map(|t| NavView {
                id: t.id.clone(),
                name: t.name.clone(),
                visible: match &t.visible {
                    NavCondition::Always => true,
                    NavCondition::Panel { panel } => g.run.enabled_panels.contains(panel),
                    NavCondition::BuildingKnown { building } => {
                        g.owned(building) > 0 || g.career.owned_buildings.contains(building)
                    }
                },
            })
            .collect(),
        can_retire: g.run.enabled_panels.contains(career::RETIREMENT_PANEL),
        log: g.career.log.clone(),
        activity: g.run.activity.clone(),
        names,
    }
}
