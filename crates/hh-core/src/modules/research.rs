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

/// The panel an event opens before any research can be bought (1.1 Online's
/// completion).
pub const RESEARCH_PANEL: &str = "research";

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

/// Shown (and so buyable): its `reveal_when` (this run's trigger for its group)
/// holds, and it was bought in any run or its `visible_when` holds; never a
/// career-flag item once its flag is set (Programming: bought once, for good).
pub fn visible(g: &GameState, item: &Research) -> bool {
    !flag_held(g, item)
        && shows(g, &item.reveal_when)
        && (g.career.researched.contains(&item.id) || shows(g, &item.visible_when))
}

/// A career-flag item whose flag is set: bought for good, never shown again.
pub fn flag_held(g: &GameState, item: &Research) -> bool {
    matches!(&item.effect, Some(ResearchEffect::CareerFlag { flag }) if g.career.flags.contains(flag))
}

/// A state saved before career-flag research existed (`Career::flag_research`
/// false): each career-flag item whose `visible_when` already holds counts as
/// bought, its flag set (Programming for a career past A Voice's start). Called
/// on load only.
pub fn grant_past_flags(g: &mut GameState) {
    if g.career.flag_research.0 {
        return;
    }
    grant_flags_due(g);
    g.career.flag_research.0 = true;
}

/// Sets the flag of each career-flag item whose `visible_when` holds now: a
/// state brought up to date, or a scenario that starts past its quest.
pub fn grant_flags_due(g: &mut GameState) {
    for item in &content().research {
        if let Some(ResearchEffect::CareerFlag { flag }) = &item.effect {
            if shows(g, &item.visible_when) {
                g.career.flags.insert(flag.clone());
            }
        }
    }
}

fn shows(g: &GameState, v: &Visibility) -> bool {
    match v {
        Visibility::Always => true,
        Visibility::Never => false,
        Visibility::EventCompleted { event } => g.run.events.contains_key(event),
        Visibility::BoredomAbove { threshold } => g.amount(BOREDOM) > *threshold,
        Visibility::ResearchPurchased { research } => g.run.research.contains(research),
        Visibility::BuildingCount { building, count } => g.owned(building) >= *count,
        Visibility::ShipmentsCompleted { count } => {
            g.career.lifetime.shipments + g.run.stats.shipments >= *count
        }
        Visibility::QuestCompleted { quest } => g.career.completed_quests.contains(quest),
        Visibility::QuestActive { quest } => super::events::quest_open_or_done(g, quest),
        Visibility::Flag { flag } => g.career.flags.contains(flag),
        Visibility::AllOf { all } => all.iter().all(|x| shows(g, x)),
        Visibility::AnyOf { any } => any.iter().any(|x| shows(g, x)),
    }
}

/// Its price in science: `cost`, times a
/// live research-discount project reward's multiplier (each whose category, if
/// it names one, is the item's) if bought in an earlier run.
pub fn price(r: &Rules, g: &GameState, item: &Research) -> Fx {
    let mut c = item.cost;
    if g.career.researched.contains(&item.id) {
        // The Rationalist major effect: research done in an earlier run is free.
        if super::ideology::major(r, g, "rationalist") {
            return Fx::ZERO;
        }
        for reward in super::projects::live_rewards(g) {
            if let ProjectReward::ResearchDiscount { mult, category } = reward {
                if category.as_ref().is_none_or(|cat| *cat == item.category) {
                    c = c * *mult;
                }
            }
        }
    }
    c
}

/// The effects of every research item done this run, in content order (the order
/// their multipliers apply in, so the products floor the same way everywhere).
pub fn live_effects(g: &GameState) -> impl Iterator<Item = &'static ResearchEffect> + '_ {
    content().research.iter().filter(|item| g.run.research.contains(&item.id)).filter_map(|item| item.effect.as_ref())
}

/// A research item revealing exact demand is done this run.
pub fn reveals_demand(g: &GameState) -> bool {
    live_effects(g).any(|e| matches!(e, ResearchEffect::RevealDemand))
}

/// A research item opening programs' researched readings is done this run.
pub fn opens_program_readings(g: &GameState) -> bool {
    live_effects(g).any(|e| matches!(e, ResearchEffect::ProgramReadings))
}

/// Why `item` can't be bought now, if it can't.
pub fn blocker(r: &Rules, g: &GameState, item: &Research) -> Option<String> {
    if !g.run.enabled_panels.contains(RESEARCH_PANEL) {
        return Some("research: not open yet".into());
    }
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
    g.track("research", SCIENCE, Fx::ZERO, cost);
    g.run.research.insert(item.id.clone());
    g.career.researched.insert(item.id.clone());
    if let Some(ResearchEffect::CareerFlag { flag }) = &item.effect {
        g.career.flags.insert(flag.clone());
    }
    g.run.stats.research_completed += 1;
    g.run.last_purchase = Some(g.run.day);
    g.note("research", &item.id);
    Ok(())
}
