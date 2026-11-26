//! The shared modifier framework (docs/rules/README.md, *Modifiers*): a
//! modifier key's value is the product of every live source naming it
//! (earned achievements, completed quests' `modifier` unlocks, completed
//! projects, research done this run, career bonuses), 1 with none.
//! Derived on demand, never stored, so it can't go stale.

use crate::content::{content, AchievementReward, ProjectReward, ResearchEffect, Unlock};
use crate::fx::Fx;
use crate::rules::Rules;
use crate::state::GameState;

pub fn modifier(r: &Rules, g: &GameState, key: &str) -> Fx {
    // An unknown key would silently read 1: catch a typo at a call site in tests.
    debug_assert!(crate::content::MODIFIER_KEYS.contains(&key), "unknown modifier key {key:?}");
    let mut m = Fx::ONE;
    for a in &content().achievements {
        if let AchievementReward::Modifier { key: k, value } = &a.reward {
            if k == key && g.career.achievements.contains(&a.id) {
                m = m * *value;
            }
        }
    }
    // A completed quest's `modifier` unlock (a side quest's permanent reward).
    for e in &content().events {
        for u in &e.unlocks {
            if let Unlock::Modifier { key: k, value } = u {
                if k == key && g.career.completed_quests.contains(&e.id) {
                    m = m * *value;
                }
            }
        }
    }
    for (reward, n) in crate::modules::projects::live_rewards_counted(g) {
        if let ProjectReward::Modifier { key: k, value } = reward {
            if k == key {
                m = m * crate::modules::projects::stacked(*value, n);
            }
        }
    }
    for effect in crate::modules::research::live_effects(g) {
        if let ResearchEffect::Modifier { key: k, value } = effect {
            if k == key {
                m = m * *value;
            }
        }
    }
    m = m * crate::modules::timeline::modifier(g, key);
    if key == "buy_power_mult" {
        m = m * crate::modules::career::buy_power_mult(r, g);
    }
    m
}
