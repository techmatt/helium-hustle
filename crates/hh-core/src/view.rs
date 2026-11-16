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
use crate::content::{content, Amounts, BuildingEffect, Condition, NavCondition, Requirement, AXES, MODIFIER_KEYS};
use crate::fx::Fx;
use crate::modifiers::modifier;
use crate::modules::*;
use crate::rules::Rules;
use crate::state::{Activity, Entry, GameState, LastRun, LineOutcome, LogEntry, Overclock, RunStats, Stall};

/// Text borrowed from the embedded content.
pub type Text = &'static str;

#[derive(Debug, Clone, Serialize)]
pub struct ResourceView {
    pub id: Text,
    pub name: Text,
    pub amount: Fx,
    /// Most that can be held, if capped.
    pub cap: Option<Fx>,
    /// Change over the next day if nothing else is done.
    pub per_day: Fx,
    pub visible: bool,
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

/// The program lines run over a span of days (a client drawing once per
/// several days, or a trace): per program, per line in queue order, its
/// outcome the last time it ran in the span (`None`: not run). Noted after
/// each tick from that day's outcomes; the span holds no actions, so line
/// indices name the same lines throughout. Display only, like the view.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct LinesRun {
    pub days: u64,
    pub lines: Vec<Vec<Option<LineOutcome>>>,
}

impl LinesRun {
    /// Adds the day `g` just played.
    pub fn note(&mut self, g: &GameState) {
        self.days += 1;
        let programs = &g.run.programs;
        // A retirement in the span leaves new (empty) programs: fit to the queues now.
        self.lines.resize(programs.len(), Vec::new());
        for (i, p) in programs.iter().enumerate() {
            let row = &mut self.lines[i];
            row.resize(p.entries.len(), None);
            if let Some(today) = g.run.today.lines.get(&(i as u32)) {
                for (&line, &o) in today {
                    if let Some(slot) = row.get_mut(line as usize) {
                        *slot = Some(o);
                    }
                }
            }
        }
    }

    /// The day `g` just played alone.
    pub fn today(g: &GameState) -> LinesRun {
        let mut l = LinesRun::default();
        l.note(g);
        l
    }
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

#[derive(Debug, Clone, Serialize)]
pub struct ProjectView {
    pub id: Text,
    pub name: Text,
    pub description: Text,
    pub tier: Text,
    pub unlocked: bool,
    pub completed: bool,
    pub costs: &'static Amounts,
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
    pub id: Text,
    pub label: Text,
    pub done: bool,
    pub progress: Option<ProgressView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuestView {
    pub id: Text,
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
pub struct NavView {
    pub id: Text,
    pub name: Text,
    pub visible: bool,
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
    /// Each axis's starting score and rank (the ideology head start).
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
    pub completed_quests: Vec<Text>,
    /// The completed quests' texts, in chain order.
    pub story: Vec<StoryView>,
    /// One-off events fired this run, in content order.
    pub events: Vec<Text>,
    /// Their texts, in content order.
    pub event_texts: Vec<StoryView>,
    pub achievements: Vec<String>,
    /// Every achievement, earned or not, in content order.
    pub achievement_list: Vec<AchievementView>,
    pub rivals: Vec<RivalView>,
    /// Live overclocks.
    pub overclocks: Vec<Overclock>,
    /// Every modifier key's value now (1 with no source).
    pub modifiers: BTreeMap<Text, Fx>,
    pub career: CareerView,
    pub next_run: NextRunView,
    /// The last finished run's summary, if any.
    pub last_run: Option<LastRun>,
    /// This run's running totals (the state's own).
    pub run_stats: RunStats,
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
        let full = (rates[i] * boredom::multiplier(r, g)).to_string();
        let shown = match full.split_once('.') {
            Some((whole, frac)) => {
                let frac = frac[..frac.len().min(2)].trim_end_matches('0');
                if frac.is_empty() { whole.to_string() } else { format!("{whole}.{frac}") }
            }
            None => full,
        };
        s.body = Cow::Owned(s.body.replace("{rate}", &shown));
    }
    s
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
                id: &res.id,
                name: &res.name,
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
                id: &cmd.id,
                name: &cmd.name,
                description: &cmd.description,
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
                tier: demand::TIERS[demand::tier(r, m.demand)],
                speculators: m.speculators_seen.then_some(m.speculators),
                good,
            })
        })
        .collect();
    let quest = events::active_quest(g).map(|q| QuestView {
        id: &q.id,
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
        can_buy_land: economy::land_blocker(r, g).is_none(),
        commands,
        programs: g
            .run
            .programs
            .iter()
            .zip(LinesRun::today(g).lines)
            .map(|(p, outcomes)| ProgramView {
                entries: p.entries.clone(),
                processors: p.processors,
                entry: p.entry,
                step: p.step,
                outcomes,
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
        projects: c
            .projects
            .iter()
            .map(|p| ProjectView {
                id: &p.id,
                name: &p.name,
                description: &p.description,
                tier: &p.tier,
                unlocked: projects::unlocked(g, p),
                completed: projects::completed(g, &p.id),
                costs: &p.costs,
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
            .map(|e| e.id.as_str())
            .collect(),
        story: c.events.iter().filter(|e| g.career.completed_quests.contains(&e.id)).map(story).collect(),
        events: c
            .events
            .iter()
            .filter(|e| !e.is_quest() && !e.silent && g.run.events.contains(&e.id))
            .map(|e| e.id.as_str())
            .collect(),
        event_texts: c
            .events
            .iter()
            .filter(|e| !e.is_quest() && !e.silent && g.run.events.contains(&e.id))
            .map(|e| event_story(r, g, e))
            .collect(),
        achievements: g.career.achievements.iter().cloned().collect(),
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
        overclocks: g.run.overclocks.clone(),
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
        panels: g.run.enabled_panels.iter().cloned().collect(),
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
        log: g.career.log.clone(),
        activity: g.run.activity.clone(),
        names: names(),
    }
}
