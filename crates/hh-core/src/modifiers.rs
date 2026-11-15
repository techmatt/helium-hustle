//! The shared modifier framework (docs/rules/README.md, *Modifiers*): a
//! modifier key's value is the product of every live source naming it
//! (earned achievements, completed projects, career bonuses), 1 with none.
//! Derived on demand, never stored, so it can't go stale.

use crate::content::{content, AchievementReward, ProjectReward};
use crate::fx::Fx;
use crate::rules::Rules;
use crate::state::GameState;

pub fn modifier(r: &Rules, g: &GameState, key: &str) -> Fx {
    let mut m = Fx::ONE;
    for a in &content().achievements {
        if let AchievementReward::Modifier { key: k, value } = &a.reward {
            if k == key && g.career.achievements.contains(&a.id) {
                m = m * *value;
            }
        }
    }
    for p in &content().projects {
        if let ProjectReward::Modifier { key: k, value } = &p.reward {
            if k == key && crate::modules::projects::completed(g, &p.id) {
                m = m * *value;
            }
        }
    }
    if key == "buy_power_mult" {
        m = m * crate::modules::career::buy_power_mult(r, g);
    }
    m
}
