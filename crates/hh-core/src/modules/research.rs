//! **research**: items bought with science, gated by visibility; reset each
//! run (docs/rules/research.md). Their effects are read where they apply
//! (boredom, programs, shipments); unlocks are the commands' and buildings'
//! own `requires`.

use super::{ModuleDef, VariantDef};
use crate::config::ModuleConfig;
use crate::content::{content, ProjectReward, Research, ResearchEffect, Visibility};
use crate::fx::Fx;
use crate::rules::Rules;
use crate::state::GameState;

use super::economy::{BOREDOM, SCIENCE};

pub static DEF: ModuleDef = ModuleDef {
    id: "research",
    variants: &[VariantDef { name: "arc1", params: &[], rng: &[] }],
};

#[derive(Clone)]
pub struct Params;

impl Params {
    pub fn new(_: &ModuleConfig) -> Result<Params, String> {
        Ok(Params)
    }
}

/// Shown (and so buyable): bought in any run, or its `visible_when` holds.
pub fn visible(g: &GameState, item: &Research) -> bool {
    if g.career.researched.contains(&item.id) {
        return true;
    }
    match &item.visible_when {
        Visibility::Always => true,
        Visibility::EventCompleted { event } => g.run.events.contains(event),
        Visibility::BoredomAbove { threshold } => g.amount(BOREDOM) > *threshold,
        Visibility::ResearchPurchased { research } => g.run.research.contains(research),
        Visibility::BuildingCount { building, count } => g.owned(building) >= *count,
        Visibility::ShipmentsCompleted { count } => {
            g.career.lifetime.shipments + g.run.stats.shipments >= *count
        }
        Visibility::QuestCompleted { quest } => g.career.completed_quests.contains(quest),
    }
}

/// Its price in science: `cost × discount ^ rationalist rank`, times a
/// completed research-discount project's multiplier if bought in an earlier run.
pub fn price(r: &Rules, g: &GameState, item: &Research) -> Fx {
    let mut c = item.cost * super::ideology::discount(r, g, "rationalist");
    if g.career.researched.contains(&item.id) {
        for p in &content().projects {
            if let ProjectReward::ResearchDiscount { mult } = &p.reward {
                if super::projects::completed(g, &p.id) {
                    c = c * *mult;
                }
            }
        }
    }
    c
}

/// A research item revealing exact demand is done this run.
pub fn reveals_demand(g: &GameState) -> bool {
    content()
        .research
        .iter()
        .any(|item| matches!(item.effect, Some(ResearchEffect::RevealDemand)) && g.run.research.contains(&item.id))
}

/// Why `item` can't be bought now, if it can't.
pub fn blocker(r: &Rules, g: &GameState, item: &Research) -> Option<String> {
    if g.run.research.contains(&item.id) {
        return Some(format!("research {}: already done", item.name));
    }
    if !visible(g, item) {
        return Some(format!("research {}: not available yet", item.name));
    }
    if let Some(req) = &item.requires {
        if !g.run.research.contains(req) {
            return Some(format!("research {}: needs {req} first", item.name));
        }
    }
    if g.amount(SCIENCE) < price(r, g, item) {
        return Some(format!("research {}: not enough science", item.name));
    }
    None
}

pub fn buy(r: &Rules, g: &mut GameState, id: &str) -> Result<(), String> {
    let item = content().research_item(id).ok_or_else(|| format!("research: unknown item {id:?}"))?;
    if let Some(e) = blocker(r, g, item) {
        return Err(e);
    }
    let cost = price(r, g, item);
    *g.run.resources.get_mut(SCIENCE).expect("content has science") -= cost;
    g.run.research.insert(item.id.clone());
    g.career.researched.insert(item.id.clone());
    g.run.stats.research_completed += 1;
    g.note("research", &item.id);
    Ok(())
}
