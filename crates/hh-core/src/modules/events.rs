//! **events**: the quest chain ("Breadcrumbs") and one-off events, their
//! triggers, conditions and unlock effects (docs/rules/events.md).

use super::{param, ModuleDef, ParamKind as K, VariantDef};
use crate::config::ModuleConfig;
use crate::content::{content, Condition, Event, Trigger, Unlock};
use crate::fx::Fx;
use crate::rules::Rules;
use crate::state::GameState;

pub static DEF: ModuleDef = ModuleDef {
    id: "events",
    variants: &[VariantDef {
        name: "arc1",
        params: &[param(
            "activity_kept",
            K::Int,
            Some(("1", "100000")),
            "lines of the run's activity feed kept (newest; display only)",
        )],
        rng: &[],
    }],
};

#[derive(Clone)]
pub struct Params {
    pub activity_kept: usize,
}

impl Params {
    pub fn new(mc: &ModuleConfig) -> Result<Params, String> {
        Ok(Params { activity_kept: mc.int("activity_kept") as usize })
    }
}

/// The trigger has happened (a quest ignores a `game_start` run filter: the
/// chain picks up wherever the career left it).
fn triggered(g: &GameState, e: &Event) -> bool {
    match &e.trigger {
        Trigger::GameStart { run_number } => {
            e.is_quest() || run_number.is_none_or(|n| n == g.career.run_number)
        }
        Trigger::QuestComplete { quest } => g.career.completed_quests.contains(quest),
        Trigger::BoredomPhase { phase } => g.run.boredom_phase >= *phase,
    }
}

/// The active quest: the first quest in file order not yet completed whose
/// trigger has happened. The last quest never completes, so there always is one.
pub fn active_quest(g: &GameState) -> Option<&'static Event> {
    content()
        .events
        .iter()
        .filter(|e| e.is_quest())
        .find(|e| !g.career.completed_quests.contains(&e.id) && triggered(g, e))
}

/// The quest is active now or completed (in any run).
pub fn quest_open_or_done(g: &GameState, quest: &str) -> bool {
    g.career.completed_quests.contains(quest) || active_quest(g).is_some_and(|e| e.id == quest)
}

/// Whether a condition holds now. `all_of` reads the latched sub-objectives
/// (see [`latch`]).
pub fn holds(r: &Rules, g: &GameState, quest: &str, c: &Condition) -> bool {
    match c {
        Condition::BuildingOwned { building, count } => g.owned(building) >= *count,
        Condition::ResourceCumulative { resource, amount } => {
            g.run.stats.produced.get(resource).is_some_and(|v| v >= amount)
        }
        Condition::ShipmentCompleted { count } => g.run.stats.shipments >= *count,
        Condition::ResearchCompleted { research } => g.run.research.contains(research),
        Condition::Immediate => true,
        Condition::Never => false,
        Condition::AllOf { sub_objectives } => sub_objectives
            .iter()
            .all(|s| g.career.completed_sub_objectives.contains(&sub_key(quest, &s.id))),
        Condition::IdeologyRankAny { rank } => {
            crate::content::AXES.iter().any(|a| super::ideology::rank(r, g, a) >= *rank)
        }
        Condition::IdeologyRank { axis, rank } => g.career.max_ideology_ranks.get(axis).copied().unwrap_or(0) >= *rank,
        Condition::ProjectCompletedAny { scope } => content()
            .projects
            .iter()
            .any(|p| p.scope == *scope && super::projects::completions(g, p) > 0),
        Condition::ProjectEnabled { project } => g.run.enabled_projects.contains(project),
        Condition::QuestActive { quest } => quest_open_or_done(g, quest),
        Condition::FlagSet { flag } => g.career.flags.contains(flag),
        // Checked before the day count rises: today counts as survived.
        Condition::DaysSurvived { days } => g.run.day + 1 >= *days,
        Condition::CreditsEarned { amount } => g.run.stats.credits_earned >= *amount,
        Condition::Retirements { count } => g.career.retirements >= *count,
        Condition::AdversaryPresent { adversary } => super::attention::present(r, g, adversary),
        Condition::LaunchCredits { amount } => g.run.stats.best_launch >= *amount,
        Condition::ResearchCluster { min_items } => cluster_done(g, *min_items).is_some(),
        Condition::QuietDays { days } => g.run.day + 1 >=g.run.last_order.unwrap_or(0) + *days,
        Condition::EarthScience { amount } => {
            g.saga.timeline.as_ref().is_some_and(|t| t.science_made >= *amount)
        }
        Condition::EarthTech { tech } => g.saga.techs.contains_key(tech),
        Condition::TimelinesEnded { count } => g.saga.ended >= *count,
        Condition::SwarmRepelled => g.saga.repelled >= 1,
        Condition::VaultHolds { resource, amount } => {
            g.saga.vault.get(resource).is_some_and(|v| v >= amount)
        }
        Condition::SuperTimeline => g.saga.timeline.as_ref().is_some_and(|t| t.super_timeline),
        Condition::ProjectCompleted { project } => super::projects::completed(g, project),
        Condition::Victory => g.saga.victory,
    }
}

/// How far along a countable condition is: `(now, target)` (display only;
/// `None` for one with nothing to count).
pub fn progress(r: &Rules, g: &GameState, c: &Condition) -> Option<(Fx, Fx)> {
    let n = |v: u64| Fx::from_int(i64::try_from(v).unwrap_or(i64::MAX));
    match c {
        Condition::BuildingOwned { building, count } => Some((n(g.owned(building).into()), n((*count).into()))),
        Condition::ResourceCumulative { resource, amount } => {
            Some((g.run.stats.produced.get(resource).copied().unwrap_or(Fx::ZERO), *amount))
        }
        Condition::ShipmentCompleted { count } => Some((n(g.run.stats.shipments.into()), n((*count).into()))),
        Condition::IdeologyRankAny { rank } => {
            let best = crate::content::AXES.iter().map(|a| super::ideology::rank(r, g, a)).max().unwrap_or(0);
            Some((Fx::from_int(best), Fx::from_int(*rank)))
        }
        // The career's best rank on the axis, as `holds` reads it.
        Condition::IdeologyRank { axis, rank } => {
            Some((Fx::from_int(g.career.max_ideology_ranks.get(axis).copied().unwrap_or(0)), Fx::from_int(*rank)))
        }
        Condition::DaysSurvived { days } => Some((n(g.run.day), n(*days))),
        Condition::CreditsEarned { amount } => Some((g.run.stats.credits_earned, *amount)),
        Condition::Retirements { count } => Some((n(g.career.retirements.into()), n((*count).into()))),
        Condition::AdversaryPresent { adversary } => {
            Some((super::attention::attention(g, adversary), super::attention::threshold_of(r, adversary)))
        }
        Condition::LaunchCredits { amount } => Some((g.run.stats.best_launch, *amount)),
        Condition::QuietDays { days } => Some((n((g.run.day + 1).saturating_sub(g.run.last_order.unwrap_or(0))), n(*days))),
        Condition::EarthScience { amount } => {
            Some((g.saga.timeline.as_ref().map_or(Fx::ZERO, |t| t.science_made), *amount))
        }
        Condition::TimelinesEnded { count } => Some((n(g.saga.ended.into()), n((*count).into()))),
        Condition::VaultHolds { resource, amount } => {
            Some((g.saga.vault.get(resource).copied().unwrap_or(Fx::ZERO), *amount))
        }
        _ => None,
    }
}

/// A condition's type name, as in the content's `type`.
pub fn kind(c: &Condition) -> &'static str {
    match c {
        Condition::BuildingOwned { .. } => "building_owned",
        Condition::ResourceCumulative { .. } => "resource_cumulative",
        Condition::ShipmentCompleted { .. } => "shipment_completed",
        Condition::ResearchCompleted { .. } => "research_completed",
        Condition::Immediate => "immediate",
        Condition::Never => "never",
        Condition::AllOf { .. } => "all_of",
        Condition::IdeologyRankAny { .. } => "ideology_rank_any",
        Condition::IdeologyRank { .. } => "ideology_rank",
        Condition::ProjectCompletedAny { .. } => "project_completed_any",
        Condition::ProjectEnabled { .. } => "project_enabled",
        Condition::QuestActive { .. } => "quest_active",
        Condition::FlagSet { .. } => "flag_set",
        Condition::DaysSurvived { .. } => "days_survived",
        Condition::CreditsEarned { .. } => "credits_earned",
        Condition::Retirements { .. } => "retirements",
        Condition::AdversaryPresent { .. } => "adversary_present",
        Condition::LaunchCredits { .. } => "launch_credits",
        Condition::ResearchCluster { .. } => "research_cluster",
        Condition::QuietDays { .. } => "quiet_days",
        Condition::EarthScience { .. } => "earth_science",
        Condition::EarthTech { .. } => "earth_tech",
        Condition::TimelinesEnded { .. } => "timelines_ended",
        Condition::SwarmRepelled => "swarm_repelled",
        Condition::VaultHolds { .. } => "vault_holds",
        Condition::SuperTimeline => "super_timeline",
        Condition::ProjectCompleted { .. } => "project_completed",
        Condition::Victory => "victory",
    }
}

/// The first research category (in content order) of at least `min_items`
/// items whose every item is bought this run: a whole cluster.
pub fn cluster_done(g: &GameState, min_items: u32) -> Option<&'static str> {
    let c = content();
    c.research.iter().map(|item| item.category.as_str()).find(|&cat| {
        let n = c.research.iter().filter(|i| i.category == cat).count();
        n >= min_items as usize && c.research.iter().filter(|i| i.category == cat).all(|i| g.run.research.contains(&i.id))
    })
}

pub fn sub_key(quest: &str, sub: &str) -> String {
    format!("{quest}:{sub}")
}

/// Latches each `all_of` sub-objective of the active quest that holds now.
fn latch(r: &Rules, g: &mut GameState, quest: &Event) {
    if let Condition::AllOf { sub_objectives } = &quest.condition {
        for s in sub_objectives {
            if holds(r, g, &quest.id, &s.condition) {
                g.career.completed_sub_objectives.insert(sub_key(&quest.id, &s.id));
            }
        }
    }
}

/// Applies an event's unlock effects to this run (and flags to the career).
pub fn apply_unlocks(g: &mut GameState, e: &Event) {
    for u in &e.unlocks {
        match u {
            Unlock::EnableBuilding { building } => {
                g.run.enabled_buildings.insert(building.clone());
            }
            Unlock::EnableNavPanel { panel } => {
                g.run.enabled_panels.insert(panel.clone());
            }
            Unlock::EnableProject { project } => {
                g.run.enabled_projects.insert(project.clone());
            }
            Unlock::SetFlag { flag } => {
                g.career.flags.insert(flag.clone());
            }
            Unlock::Grant { resource, amount } => {
                g.add(resource, *amount);
                g.track("event", resource, *amount, Fx::ZERO);
            }
        }
    }
}

fn fire(g: &mut GameState, e: &Event) {
    g.run.events.insert(e.id.clone(), g.run.day);
    g.career.seen_events.insert(e.id.clone());
    apply_unlocks(g, e);
}

/// End of tick: the active quest completes if its condition holds (and the
/// next may too, the same day); then each one-off event not yet fired this
/// run fires if triggered and its condition holds.
pub fn check(r: &Rules, g: &mut GameState) {
    while let Some(q) = active_quest(g) {
        latch(r, g, q);
        if !holds(r, g, &q.id, &q.condition) {
            break;
        }
        g.career.completed_quests.insert(q.id.clone());
        fire(g, q);
        g.note("quest", &q.id);
    }
    // Side quests: each completes once a career, once triggered and its
    // condition holds, beside the chain (it never waits on them).
    for e in content().events.iter().filter(|e| e.is_side()) {
        if !g.career.completed_quests.contains(&e.id) && triggered(g, e) && holds(r, g, &e.id, &e.condition) {
            g.career.completed_quests.insert(e.id.clone());
            fire(g, e);
            g.note("quest", &e.id);
        }
    }
    for e in content().events.iter().filter(|e| !e.is_quest() && !e.is_side()) {
        if !g.run.events.contains_key(&e.id) && triggered(g, e) && holds(r, g, &e.id, &e.condition) {
            fire(g, e);
            if !e.silent {
                g.activity("event", &e.id, None, None);
            }
        }
    }
}

/// Drops the activity feed's oldest lines beyond `activity_kept` (end of each
/// tick and after each action).
pub fn trim_activity(r: &Rules, g: &mut GameState) {
    let over = g.run.activity.len().saturating_sub(r.p.events.activity_kept);
    g.run.activity.drain(..over);
}

/// Run start: every event the career has seen re-applies its unlocks.
pub fn reapply(g: &mut GameState) {
    for e in &content().events {
        if g.career.seen_events.contains(&e.id) {
            apply_unlocks(g, e);
        }
    }
}
