//! **projects**: big builds that drain resources over time at player-set
//! rates. A row's `scope` says where its progress and completions live: the
//! run, the timeline (the career) or forever (docs/rules/projects.md).

use std::collections::BTreeMap;

use super::{ModuleDef, VariantDef};
use crate::config::ModuleConfig;
use crate::content::{content, Amounts, Project, ProjectReward, Scope};
use crate::fx::Fx;
use crate::rules::Rules;
use crate::state::{GameState, ProjectBook};

pub static DEF: ModuleDef = ModuleDef {
    id: "projects",
    variants: &[VariantDef { name: "drain", params: &[], rng: &[] }],
};

#[derive(Clone)]
pub struct Params;

impl Params {
    pub fn new(_: &ModuleConfig) -> Result<Params, String> {
        Ok(Params)
    }
}

/// The book a scope's progress and completions live in.
pub fn book(g: &GameState, scope: Scope) -> &ProjectBook {
    match scope {
        Scope::Run => &g.run.projects,
        Scope::Timeline => &g.career.projects,
        Scope::Forever => &g.career.forever_projects,
    }
}

fn book_mut(g: &mut GameState, scope: Scope) -> &mut ProjectBook {
    match scope {
        Scope::Run => &mut g.run.projects,
        Scope::Timeline => &mut g.career.projects,
        Scope::Forever => &mut g.career.forever_projects,
    }
}

/// How many times the project has completed in its scope.
pub fn completions(g: &GameState, p: &Project) -> u32 {
    book(g, p.scope).completions.get(&p.id).copied().unwrap_or(0)
}

/// Completed at least once in its scope (an unknown id: no).
pub fn completed(g: &GameState, id: &str) -> bool {
    content().project(id).is_some_and(|p| completions(g, p) > 0)
}

/// The rewards of every completed project, in content order (the order their
/// multipliers apply in, so the products floor the same way everywhere).
pub fn live_rewards(g: &GameState) -> impl Iterator<Item = &'static ProjectReward> + '_ {
    content().projects.iter().filter(|p| completions(g, p) > 0).flat_map(|p| p.rewards.iter())
}

/// A completed project's multiplier reward, stacked per completion by the
/// bonus-stacking ruling: one source's bonuses add, so `n` completions of a
/// ×`v` reward give `1 + (v − 1) × n` (a one-off row: `v`).
pub fn stacked(v: Fx, n: u32) -> Fx {
    (Fx::ONE + (v - Fx::ONE).mul_int(i64::from(n))).max(Fx::ZERO)
}

/// Like [`live_rewards`], with each reward's project's completion count.
pub fn live_rewards_counted(g: &GameState) -> impl Iterator<Item = (&'static ProjectReward, u32)> + '_ {
    content()
        .projects
        .iter()
        .map(move |p| (p, completions(g, p)))
        .filter(|(_, n)| *n > 0)
        .flat_map(|(p, n)| p.rewards.iter().map(move |r| (r, n)))
}

/// Fundable now: its `unlock` holds and it is not done (a repeatable one is
/// never done).
pub fn unlocked(r: &Rules, g: &GameState, p: &Project) -> bool {
    (p.repeat.is_some() || completions(g, p) == 0) && super::events::holds(r, g, &p.id, &p.unlock)
}

/// The next completion's cost of one resource: the row's cost times
/// `cost_growth` once per completion so far.
pub fn cost(g: &GameState, p: &Project, resource: &str) -> Fx {
    let base = p.costs.get(resource).copied().unwrap_or(Fx::ZERO);
    let Some(rep) = &p.repeat else { return base };
    let n = i64::from(completions(g, p));
    rep.cost_growth.pow_int(n).and_then(|f| base.checked_mul(f)).unwrap_or(Fx::MAX)
}

/// Every cost of the next completion.
pub fn costs(g: &GameState, p: &Project) -> Amounts {
    p.costs.keys().map(|res| (res.clone(), cost(g, p, res))).collect()
}

/// The most of a resource the project may take a day: its cost over
/// `min_days`, rounded up a step when inexact, so funding at the most takes
/// exactly `min_days` days.
pub fn max_rate(g: &GameState, p: &Project, resource: &str) -> Fx {
    let cost = cost(g, p, resource);
    let days = i64::from(p.min_days);
    let floor = cost.mul_ratio(1, days);
    if floor.mul_int(days) < cost { floor.plus_steps(1) } else { floor }
}

/// Progress toward the next completion, by resource.
pub fn progress<'a>(g: &'a GameState, p: &Project) -> Option<&'a BTreeMap<String, Fx>> {
    book(g, p.scope).progress.get(&p.id)
}

fn paid(g: &GameState, p: &Project, resource: &str) -> Fx {
    progress(g, p).and_then(|m| m.get(resource)).copied().unwrap_or(Fx::ZERO)
}

/// Days until every cost is met at today's rates (stock aside); `None` when a
/// cost still owed has no rate.
pub fn days_to_done(g: &GameState, p: &Project) -> Option<u64> {
    let rates = g.run.project_rates.get(&p.id);
    let mut days = 0u64;
    for res in p.costs.keys() {
        let owed = cost(g, p, res) - paid(g, p, res);
        if owed <= Fx::ZERO {
            continue;
        }
        let rate = rates.and_then(|m| m.get(res)).copied().unwrap_or(Fx::ZERO);
        if rate <= Fx::ZERO {
            return None;
        }
        // Whole days, rounded up: owed / rate in raw steps (no add, so a cost
        // saturated at Fx::MAX can't overflow).
        let (o, s) = (owed.steps(), rate.steps());
        let d = o / s + i64::from(o % s != 0);
        days = days.max(u64::try_from(d).unwrap_or(u64::MAX));
    }
    Some(days)
}

/// Sets how much of `resource` a project takes per day (0 stops it).
pub fn set_rate(r: &Rules, g: &mut GameState, id: &str, resource: &str, rate: Fx) -> Result<(), String> {
    let p = content().project(id).ok_or_else(|| format!("project: unknown project {id:?}"))?;
    if !unlocked(r, g, p) {
        return Err(format!("project {}: not available", p.name));
    }
    if !p.costs.contains_key(resource) {
        let shown = content().resource(resource).map_or(resource, |x| x.name.as_str());
        return Err(format!("project {}: takes no {shown}", p.name));
    }
    let max = max_rate(g, p, resource);
    if rate.is_negative() || rate > max {
        return Err(format!("project {}: rate must be 0..={max}", p.name));
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
pub fn drain(r: &Rules, g: &mut GameState) {
    for p in &content().projects {
        let Some(rates) = g.run.project_rates.get(&p.id).cloned() else {
            continue;
        };
        if !unlocked(r, g, p) {
            continue;
        }
        for (res, rate) in rates {
            let need = cost(g, p, &res) - paid(g, p, &res);
            let want = rate.min(need).max(Fx::ZERO);
            let take = want.min(g.amount(&res)).max(Fx::ZERO);
            if take < want {
                // Funding the stock couldn't cover counts as unmet demand (`rate`).
                let u = g.run.today.unmet.entry(res.clone()).or_insert(Fx::ZERO);
                *u = u.saturating_add(want - take);
            }
            if take == Fx::ZERO {
                continue;
            }
            g.consume("projects", &res, take);
            let book = book_mut(g, p.scope);
            *book.progress.entry(p.id.clone()).or_default().entry(res.clone()).or_insert(Fx::ZERO) += take;
        }
        if p.costs.keys().all(|res| paid(g, p, res) >= cost(g, p, res)) {
            complete(g, p);
        }
    }
}

/// One more completion: the count rises, progress starts over, a project done
/// for good drops its rates (a repeatable one keeps them), flags are set.
fn complete(g: &mut GameState, p: &Project) {
    let book = book_mut(g, p.scope);
    *book.completions.entry(p.id.clone()).or_insert(0) += 1;
    book.progress.remove(&p.id);
    if p.repeat.is_none() {
        g.run.project_rates.remove(&p.id);
    }
    for reward in &p.rewards {
        if let ProjectReward::SetFlag { flag } = reward {
            g.career.flags.insert(flag.clone());
        }
    }
    g.note("project", &p.id);
}

/// Free buildings at the start of every run from completed projects.
pub fn starting_bonus(g: &GameState) -> Vec<(String, u32)> {
    // Stacked per completion (bonus stacking): n completions give n × each.
    let mut out = Vec::new();
    for (reward, times) in live_rewards_counted(g) {
        if let ProjectReward::StartingBuildings { buildings } = reward {
            out.extend(buildings.iter().map(|(b, &n)| (b.clone(), n.saturating_mul(times))));
        }
    }
    out
}
