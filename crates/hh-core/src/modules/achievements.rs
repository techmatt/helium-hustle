//! **achievements**: optional accomplishments with permanent rewards
//! (modifiers, bonus buildings), kept by the career (docs/rules/achievements.md).

use super::{ModuleDef, VariantDef};
use crate::config::ModuleConfig;
use crate::content::{content, Achievement, AchievementCondition, AchievementReward};
use crate::fx::Fx;
use crate::state::GameState;

pub static DEF: ModuleDef = ModuleDef {
    id: "achievements",
    variants: &[VariantDef { name: "arc1", params: &[], rng: &[] }],
};

pub struct Params;

impl Params {
    pub fn new(_: &ModuleConfig) -> Result<Params, String> {
        Ok(Params)
    }
}

fn earn(g: &mut GameState, a: &Achievement) {
    if g.career.achievements.insert(a.id.clone()) {
        g.note("achievement", &a.id);
    }
}

fn flow(map: &std::collections::BTreeMap<String, Fx>, r: &str) -> Fx {
    map.get(r).copied().unwrap_or(Fx::ZERO)
}

/// End of tick: the day's production, use and stockpile conditions.
pub fn check(g: &mut GameState) {
    for a in &content().achievements {
        if g.career.achievements.contains(&a.id) {
            continue;
        }
        let met = match &a.condition {
            AchievementCondition::ProducedInADay { resource, at_least } => {
                flow(&g.run.today.produced, resource) >= *at_least
            }
            AchievementCondition::Stockpile { resource, at_least } => g.amount(resource) >= *at_least,
            AchievementCondition::ConsumedInADay { resource, more_than } => {
                flow(&g.run.today.consumed, resource) > *more_than
            }
            _ => false,
        };
        if met {
            earn(g, a);
        }
    }
}

/// At a launch: the shipment conditions.
pub fn on_shipment(g: &mut GameState, revenue: Fx, demand: Fx) {
    for a in &content().achievements {
        let met = match &a.condition {
            AchievementCondition::ShipmentRevenue { at_least } => revenue >= *at_least,
            AchievementCondition::ShipmentsThisRun { at_least } => g.run.stats.shipments >= *at_least,
            AchievementCondition::ShipmentDemand { more_than } => demand > *more_than,
            _ => false,
        };
        if met {
            earn(g, a);
        }
    }
}

/// Free buildings every run from earned achievements.
pub fn starting_bonus(g: &GameState) -> Vec<(String, u32)> {
    let mut out = Vec::new();
    for a in &content().achievements {
        if let AchievementReward::BonusBuildings { building, count } = &a.reward {
            if g.career.achievements.contains(&a.id) {
                out.push((building.clone(), *count));
            }
        }
    }
    out
}
