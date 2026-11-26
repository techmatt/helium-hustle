//! The Stats panel and retirement: the career's records, the next run's preview and the achievements.

use super::{axes, AchievementView, CareerView, NextRunView};
use crate::content::content;
use crate::modules::*;
use crate::rules::Rules;
use crate::state::GameState;

/// The next run if this one retired now.
pub(super) fn next_run(r: &Rules, g: &GameState) -> NextRunView {
    let after = career::next_run_preview(r, g);
    NextRunView {
        starting_credits: career::starting_credits(r, &after),
        boredom_resilience: career::boredom_resilience(r, &after),
        buy_power_mult: career::buy_power_mult(r, &after),
        ideology: axes(r, &after),
    }
}

/// Every achievement, earned or not, in content order.
pub(super) fn achievements(g: &GameState) -> Vec<AchievementView> {
    content()
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
        .collect()
}

/// The career's records and what it gives every run.
pub(super) fn career(r: &Rules, g: &GameState) -> CareerView {
    CareerView {
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
    }
}
