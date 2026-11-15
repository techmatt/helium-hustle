//! What a client (the UI or an agent) shows and can do: derived display data
//! (prices, rates, caps, what is affordable, unlocked or visible, the action
//! for each), so no client computes a rule. Never read by the rules, not part
//! of the state, the fingerprint or a replay.

use serde::Serialize;
use std::collections::BTreeMap;

use crate::action::Action;
use crate::content::{content, Condition, AXES};
use crate::fx::Fx;
use crate::modules::*;
use crate::rules::Rules;
use crate::state::{Entry, GameState, LogEntry, Stall};

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
    pub category: String,
    pub owned: u32,
    pub active: u32,
    pub bonus: u32,
    /// Price of the next one (`Fx::MAX` past the number range).
    pub cost: BTreeMap<String, Fx>,
    pub land: u32,
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
}

#[derive(Debug, Clone, Serialize)]
pub struct PadView {
    pub good: String,
    pub paused: bool,
    pub active: bool,
    pub cargo: Fx,
    pub capacity: Fx,
    pub cooldown: u32,
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
    pub category: String,
    pub cost: Fx,
    pub done: bool,
    pub visible: bool,
    pub affordable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AxisView {
    pub axis: &'static str,
    pub score: Fx,
    pub rank: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectView {
    pub id: String,
    pub name: String,
    pub tier: String,
    pub unlocked: bool,
    pub completed: bool,
    pub costs: BTreeMap<String, Fx>,
    pub progress: BTreeMap<String, Fx>,
    pub rates: BTreeMap<String, Fx>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubObjectiveView {
    pub id: String,
    pub label: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuestView {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub sub_objectives: Vec<SubObjectiveView>,
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
}

#[derive(Debug, Clone, Serialize)]
pub struct View {
    pub tick: u64,
    pub day: u64,
    pub run: u32,
    pub resources: Vec<ResourceView>,
    pub buildings: Vec<BuildingView>,
    pub land_price: Fx,
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
    /// One-off events fired this run, in content order.
    pub events: Vec<String>,
    pub achievements: Vec<String>,
    pub career: CareerView,
    pub boredom: BoredomView,
    /// Unlocked nav panels (retirement, projects, ideologies).
    pub panels: Vec<String>,
    pub can_retire: bool,
    pub log: Vec<LogEntry>,
}

const TIERS: &[&str] = &["LOW", "MEDIUM", "HIGH", "VERY HIGH"];

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
            ResourceView {
                id: res.id.clone(),
                name: res.name.clone(),
                amount: g.amount(&res.id),
                cap: economy::cap(r, g, &res.id),
                per_day: if same_run { next.amount(&res.id) - g.amount(&res.id) } else { Fx::ZERO },
                visible: res.always_visible || made_by_owned,
            }
        })
        .collect();
    let buildings = c
        .buildings
        .iter()
        .map(|b| {
            let h = g.run.buildings.get(&b.id).copied().unwrap_or_default();
            BuildingView {
                id: b.id.clone(),
                name: b.name.clone(),
                category: b.category.clone(),
                owned: h.owned,
                active: h.active,
                bonus: h.bonus,
                cost: economy::price(r, g, b),
                land: b.land,
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
        sub_objectives: match &q.condition {
            Condition::AllOf { sub_objectives } => sub_objectives
                .iter()
                .map(|s| SubObjectiveView {
                    id: s.id.clone(),
                    label: s.label.clone(),
                    done: g.career.completed_sub_objectives.contains(&events::sub_key(&q.id, &s.id)),
                })
                .collect(),
            _ => Vec::new(),
        },
    });
    let procs = programs::processors(g);
    View {
        tick: g.tick,
        day: g.run.day,
        run: g.career.run_number,
        resources,
        buildings,
        land_price: economy::land_price(r, g),
        commands,
        programs: g
            .run
            .programs
            .iter()
            .map(|p| ProgramView {
                entries: p.entries.clone(),
                processors: p.processors,
                entry: p.entry,
                step: p.step,
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
                    category: item.category.clone(),
                    cost,
                    done: g.run.research.contains(&item.id),
                    visible: research::visible(g, item),
                    affordable: research::blocker(r, g, item).is_none(),
                }
            })
            .collect(),
        ideology: AXES
            .iter()
            .map(|&a| AxisView { axis: a, score: ideology::score(g, a), rank: ideology::rank(r, g, a) })
            .collect(),
        projects: c
            .projects
            .iter()
            .map(|p| ProjectView {
                id: p.id.clone(),
                name: p.name.clone(),
                tier: p.tier.clone(),
                unlocked: projects::unlocked(g, p),
                completed: projects::completed(g, &p.id),
                costs: p.costs.clone(),
                progress: projects::progress(g, p).cloned().unwrap_or_default(),
                rates: g.run.project_rates.get(&p.id).cloned().unwrap_or_default(),
            })
            .collect(),
        quest,
        completed_quests: c
            .events
            .iter()
            .filter(|e| g.career.completed_quests.contains(&e.id))
            .map(|e| e.id.clone())
            .collect(),
        events: c
            .events
            .iter()
            .filter(|e| !e.is_quest() && !e.silent && g.run.events.contains(&e.id))
            .map(|e| e.id.clone())
            .collect(),
        achievements: g.career.achievements.iter().cloned().collect(),
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
        },
        boredom: BoredomView {
            amount: g.amount(economy::BOREDOM),
            cap: boredom::cap(r, g),
            rate: boredom::rate(r, g),
            phase: boredom::phase(r, g.run.day),
        },
        panels: g.run.enabled_panels.iter().cloned().collect(),
        can_retire: g.run.enabled_panels.contains(career::RETIREMENT_PANEL),
        log: g.career.log.clone(),
    }
}
