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
        Condition::PersistentProjectCompletedAny => content()
            .projects
            .iter()
            .any(|p| p.persistent() && g.career.completed_projects.contains(&p.id)),
        // Checked before the day count rises: today counts as survived.
        Condition::DaysSurvived { days } => g.run.day + 1 >= *days,
        Condition::CreditsEarned { amount } => g.run.stats.credits_earned >= *amount,
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
        Condition::DaysSurvived { days } => Some((n(g.run.day), n(*days))),
        Condition::CreditsEarned { amount } => Some((g.run.stats.credits_earned, *amount)),
        _ => None,
    }
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
        }
    }
}

fn fire(g: &mut GameState, e: &Event) {
    g.run.events.insert(e.id.clone());
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
    for e in content().events.iter().filter(|e| !e.is_quest()) {
        if !g.run.events.contains(&e.id) && triggered(g, e) && holds(r, g, &e.id, &e.condition) {
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
