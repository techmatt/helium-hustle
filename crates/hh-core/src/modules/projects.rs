//! **projects**: big builds that drain resources over time at player-set
//! rates; Long-Term (persistent) progress lives in the career, Strategic
//! (personal) in the run (docs/rules/projects.md).

use std::collections::BTreeMap;

use super::{param, ModuleDef, ParamKind as K, VariantDef};
use crate::config::ModuleConfig;
use crate::content::{content, Project, ProjectReward, ProjectUnlock};
use crate::fx::Fx;
use crate::rules::Rules;
use crate::state::GameState;

pub static DEF: ModuleDef = ModuleDef {
    id: "projects",
    variants: &[VariantDef {
        name: "drain",
        params: &[param("max_drain_rate", K::Fx, Some(("0", "1000000")), "most of one resource a project may take per day")],
        rng: &[],
    }],
};

#[derive(Clone)]
pub struct Params {
    pub max_drain_rate: Fx,
}

impl Params {
    pub fn new(mc: &ModuleConfig) -> Result<Params, String> {
        Ok(Params { max_drain_rate: mc.fx("max_drain_rate") })
    }
}

/// Completed: in the career (Long-Term) or this run (Strategic).
pub fn completed(g: &GameState, id: &str) -> bool {
    g.career.completed_projects.contains(id) || g.run.completed_projects.contains(id)
}

/// The rewards of every completed project, in content order (the order their
/// multipliers apply in, so the products floor the same way everywhere).
pub fn live_rewards(g: &GameState) -> impl Iterator<Item = &'static ProjectReward> + '_ {
    content().projects.iter().filter(|p| completed(g, &p.id)).map(|p| &p.reward)
}

/// Fundable now (and not yet complete).
pub fn unlocked(g: &GameState, p: &Project) -> bool {
    if completed(g, &p.id) {
        return false;
    }
    match &p.unlock {
        ProjectUnlock::EventUnlocked => g.run.enabled_projects.contains(&p.id),
        ProjectUnlock::IdeologyRank { axis, rank } => {
            g.career.max_ideology_ranks.get(axis).copied().unwrap_or(0) >= *rank
        }
        ProjectUnlock::QuestActive { quest } => super::events::quest_open_or_done(g, quest),
        ProjectUnlock::ResearchCompleted { research } => g.run.research.contains(research),
        ProjectUnlock::FlagSet { flag } => g.career.flags.contains(flag),
    }
}

/// Progress so far, by resource.
pub fn progress<'a>(g: &'a GameState, p: &Project) -> Option<&'a BTreeMap<String, Fx>> {
    if p.persistent() {
        g.career.project_progress.get(&p.id)
    } else {
        g.run.project_progress.get(&p.id)
    }
}

/// Sets how much of `resource` a project takes per day (0 stops it).
pub fn set_rate(r: &Rules, g: &mut GameState, id: &str, resource: &str, rate: Fx) -> Result<(), String> {
    let p = content().project(id).ok_or_else(|| format!("project: unknown project {id:?}"))?;
    if !unlocked(g, p) {
        return Err(format!("project {}: not available", p.name));
    }
    if !p.costs.contains_key(resource) {
        return Err(format!("project {}: takes no {resource}", p.name));
    }
    if rate.is_negative() || rate > r.p.projects.max_drain_rate {
        return Err(format!("project {}: rate must be 0..={}", p.name, r.p.projects.max_drain_rate));
    }
    let rates = g.run.project_rates.entry(p.id.clone()).or_default();
    if rate == Fx::ZERO {
        rates.remove(resource);
    } else {
        rates.insert(resource.to_string(), rate);
    }
    if rates.is_empty() {
        g.run.project_rates.remove(&p.id);
    }
    Ok(())
}

/// Each day (after programs): every funded, unlocked project takes up to its
/// rate of each resource it still needs (as much as is in stock); one whose
/// every cost is met completes.
pub fn drain(g: &mut GameState) {
    for p in &content().projects {
        let Some(rates) = g.run.project_rates.get(&p.id).cloned() else {
            continue;
        };
        if !unlocked(g, p) {
            continue;
        }
        for (res, rate) in rates {
            let done = progress(g, p).and_then(|m| m.get(&res)).copied().unwrap_or(Fx::ZERO);
            let need = p.costs[&res] - done;
            let take = rate.min(need).min(g.amount(&res)).max(Fx::ZERO);
            if take == Fx::ZERO {
                continue;
            }
            g.consume(&res, take);
            let book = if p.persistent() { &mut g.career.project_progress } else { &mut g.run.project_progress };
            *book.entry(p.id.clone()).or_default().entry(res.clone()).or_insert(Fx::ZERO) += take;
        }
        let funded = p
            .costs
            .iter()
            .all(|(res, &c)| progress(g, p).and_then(|m| m.get(res)).is_some_and(|&v| v >= c));
        if funded {
            complete(g, p);
        }
    }
}

fn complete(g: &mut GameState, p: &Project) {
    if p.persistent() {
        g.career.completed_projects.insert(p.id.clone());
    } else {
        g.run.completed_projects.insert(p.id.clone());
    }
    g.run.project_rates.remove(&p.id);
    if let ProjectReward::SetFlag { flag } = &p.reward {
        g.career.flags.insert(flag.clone());
    }
    g.note("project", &p.id);
}

/// Free buildings every run from completed Long-Term projects.
pub fn starting_bonus(g: &GameState) -> Vec<(String, u32)> {
    let mut out = Vec::new();
    for p in &content().projects {
        if let ProjectReward::StartingBuildings { buildings } = &p.reward {
            if g.career.completed_projects.contains(&p.id) {
                out.extend(buildings.iter().map(|(b, &n)| (b.clone(), n)));
            }
        }
    }
    out
}
