//! How far a game is toward a milestone, 0 to 1: the oracle's milestone
//! profile scores each open milestone's progress at a rollout's end, so goals
//! longer than its horizon still pull (docs/oracle.md).
//!
//! Progress reads the state, plus what only a rollout saw: its closing credit
//! income and the goods it shipped. A milestone met is 1.

use hh_core::content::{content, Condition, Scope};
use hh_core::modules::{events, ideology, projects};
use hh_core::{Fx, Rules};

use crate::milestones::When;

/// What a rollout saw beyond its final state.
pub struct Seen<'a> {
    /// Credits earned a day over its second half.
    pub rate: Fx,
    pub shipped: usize,
    pub tracker_met: &'a dyn Fn(&When) -> bool,
}

fn frac(have: Fx, need: Fx) -> Fx {
    if need <= Fx::ZERO {
        return Fx::ONE;
    }
    (have / need).max(Fx::ZERO).min(Fx::ONE)
}

fn count(have: u64, need: u64) -> Fx {
    frac(Fx::from_int(have as i64), Fx::from_int(need as i64))
}

/// The best fraction paid of a project of the scope (progress over its
/// summed costs; 1 for a completed one).
pub fn best_paid(g: &hh_core::GameState, scope: Scope) -> Fx {
    content()
        .projects
        .iter()
        .filter(|p| p.scope == scope)
        .map(|p| {
            if projects::completed(g, &p.id) {
                return Fx::ONE;
            }
            let (paid, total) = paid(g, p);
            frac(paid, total)
        })
        .max()
        .unwrap_or(Fx::ZERO)
}

/// A project's paid amount toward its next completion and that completion's
/// cost, summed over its resources.
pub fn paid(g: &hh_core::GameState, p: &hh_core::content::Project) -> (Fx, Fx) {
    let total = projects::costs(g, p).values().fold(Fx::ZERO, |s, x| s + *x);
    let paid = projects::progress(g, p).map_or(Fx::ZERO, |m| m.values().fold(Fx::ZERO, |s, x| s + *x));
    (paid, total)
}

/// Progress toward a quest condition (a sub-objective's, or the quest's own).
fn condition(r: &Rules, g: &hh_core::GameState, quest: &str, c: &Condition) -> Fx {
    match c {
        Condition::IdeologyRankAny { rank } => {
            let need = ideology::score_for_rank(r, *rank);
            let best = hh_core::content::AXES.iter().map(|a| ideology::score(g, a)).max().unwrap_or(Fx::ZERO);
            frac(best, need)
        }
        Condition::ProjectCompletedAny { scope } => best_paid(g, *scope),
        Condition::DaysSurvived { days } => count(g.run.day + 1, *days),
        Condition::CreditsEarned { amount } => frac(g.run.stats.credits_earned, *amount),
        Condition::ResearchCompleted { research } => Fx::from_int(i64::from(g.run.research.contains(research))),
        Condition::BuildingOwned { building, count: n } => count(u64::from(g.owned(building)), u64::from(*n)),
        Condition::ShipmentCompleted { count: n } => count(u64::from(g.run.stats.shipments), u64::from(*n)),
        Condition::AllOf { sub_objectives } => {
            let n = sub_objectives.len().max(1) as i64;
            let sum = sub_objectives.iter().fold(Fx::ZERO, |s, sub| {
                let latched = g.career.completed_sub_objectives.contains(&events::sub_key(quest, &sub.id));
                s + if latched { Fx::ONE } else { condition(r, g, quest, &sub.condition) }
            });
            sum.mul_ratio(1, n)
        }
        Condition::IdeologyRank { axis, rank } => {
            let best = g.career.max_ideology_scores.get(axis).copied().unwrap_or(Fx::ZERO);
            frac(best.max(ideology::score(g, axis)), ideology::score_for_rank(r, *rank))
        }
        // Each axis as `IdeologyRank` reads it, averaged.
        Condition::IdeologyRankEach { rank } => {
            let need = ideology::score_for_rank(r, *rank);
            let sum = hh_core::content::AXES.iter().fold(Fx::ZERO, |s, a| {
                let best = g.career.max_ideology_scores.get(*a).copied().unwrap_or(Fx::ZERO);
                s + frac(best.max(ideology::score(g, a)), need)
            });
            sum.mul_ratio(1, hh_core::content::AXES.len() as i64)
        }
        // The rest: the engine's own countable progress, where it has one.
        _ => events::progress(r, g, c).map_or(Fx::ZERO, |(now, target)| frac(now, target)),
    }
}

/// Progress toward a milestone at a rollout's end, 0 to 1.
pub fn toward(r: &Rules, g: &hh_core::GameState, w: &When, seen: &Seen) -> Fx {
    if (seen.tracker_met)(w) {
        return Fx::ONE;
    }
    match w {
        When::Building { building, count: n } => count(u64::from(g.owned(building)), u64::from(*n)),
        When::Shipments { count: n } => count(u64::from(g.career.lifetime.shipments + g.run.stats.shipments), u64::from(*n)),
        When::Retirements { count: n } => count(u64::from(g.career.retirements), u64::from(*n)),
        When::CreditRate { per_day, .. } => frac(seen.rate, *per_day),
        When::Stockpile { resource, at_least } => frac(g.amount(resource), *at_least),
        When::GoodsShipped { count: n } => count(seen.shipped as u64, u64::from(*n)),
        When::CommandsUsed { groups } => {
            let used = groups.iter().filter(|grp| grp.iter().any(|c| g.career.used_commands.contains(c))).count();
            count(used as u64, groups.len() as u64)
        }
        When::Research { research } => {
            // Science on hand toward it, short of buying it.
            let cost = content().research.iter().find(|x| x.id == *research).map_or(Fx::ONE, |x| x.cost);
            frac(g.amount("science"), cost).min(Fx::from_int(9).mul_ratio(1, 10))
        }
        When::ProjectCompleted { scope } => best_paid(g, *scope),
        When::IdeologyRank { rank } => {
            let need = ideology::score_for_rank(r, *rank);
            let best = g.career.max_ideology_scores.values().copied().max().unwrap_or(Fx::ZERO);
            frac(best, need)
        }
        When::Quest { quest } => {
            if g.career.completed_quests.contains(quest) {
                return Fx::ONE;
            }
            content()
                .events
                .iter()
                .find(|e| e.id == *quest)
                .map_or(Fx::ZERO, |e| condition(r, g, quest, &e.condition))
        }
        When::AllOf { of } => {
            let n = of.len().max(1) as i64;
            of.iter().fold(Fx::ZERO, |s, x| s + toward(r, g, x, seen)).mul_ratio(1, n)
        }
    }
}
