//! **projects**: big builds that drain resources over time at player-set
//! rates. A row's `scope` says where its progress and completions live: the
//! run, the timeline (the career) or forever (docs/rules/projects.md).

use std::collections::BTreeMap;

use super::{param, ModuleDef, ParamKind as K, VariantDef};
use crate::config::ModuleConfig;
use crate::content::{content, Amounts, Project, ProjectReward, Scope};
use crate::fx::Fx;
use crate::rules::Rules;
use crate::state::{GameState, ProjectBook};

use super::economy::CREDITS;

pub static DEF: ModuleDef = ModuleDef {
    id: "projects",
    variants: &[VariantDef {
        name: "drain",
        params: &[
            param("credits_per_day", K::Fx, Some(("0.000001", "1000000")), "the most credits a project takes a day"),
            param("other_per_day", K::Fx, Some(("0.000001", "1000000")), "the most of each other resource a project takes a day"),
        ],
        rng: &[],
    }],
};

#[derive(Clone)]
pub struct Params {
    pub credits_per_day: Fx,
    pub other_per_day: Fx,
}

impl Params {
    pub fn new(mc: &ModuleConfig) -> Result<Params, String> {
        Ok(Params { credits_per_day: mc.fx("credits_per_day"), other_per_day: mc.fx("other_per_day") })
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

/// The most of a resource the project may take a day: the funding cap
/// (`credits_per_day` for credits, `other_per_day` for every other resource),
/// and on a row with `min_days` at most its cost over those days, rounded up a
/// step when inexact, so funding at the most takes no fewer than `min_days`.
pub fn max_rate(r: &Rules, g: &GameState, p: &Project, resource: &str) -> Fx {
    let cap = if resource == CREDITS { r.p.projects.credits_per_day } else { r.p.projects.other_per_day };
    let Some(min_days) = p.min_days else { return cap };
    let cost = cost(g, p, resource);
    let days = i64::from(min_days);
    let floor = cost.mul_ratio(1, days);
    cap.min(if floor.mul_int(days) < cost { floor.plus_steps(1) } else { floor })
}

/// The rate a project actually gets of a resource: the last drain's take
/// when the stock couldn't cover the set rate, else the set rate.
pub fn actual_rate(g: &GameState, p: &Project, resource: &str) -> Fx {
    let set = g.run.project_rates.get(&p.id).and_then(|m| m.get(resource)).copied().unwrap_or(Fx::ZERO);
    let short = g.run.project_short.get(&p.id).and_then(|m| m.get(resource)).copied();
    short.map_or(set, |s| s.min(set))
}

/// Progress toward the next completion, by resource.
pub fn progress<'a>(g: &'a GameState, p: &Project) -> Option<&'a BTreeMap<String, Fx>> {
    book(g, p.scope).progress.get(&p.id)
}

fn paid(g: &GameState, p: &Project, resource: &str) -> Fx {
    progress(g, p).and_then(|m| m.get(resource)).copied().unwrap_or(Fx::ZERO)
}

/// Days until every cost is met at the actual rates ([`actual_rate`]: the set
/// rate, or the last drain's take where the stock fell short); `None` when a
/// cost still owed gets nothing.
pub fn days_to_done(g: &GameState, p: &Project) -> Option<u64> {
    let mut days = 0u64;
    for res in p.costs.keys() {
        let owed = cost(g, p, res) - paid(g, p, res);
        if owed <= Fx::ZERO {
            continue;
        }
        let rate = actual_rate(g, p, res);
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
    let max = max_rate(r, g, p, resource);
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
    g.run.project_short.clear();
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
                // Funding the stock couldn't cover counts as unmet demand (the day's rate).
                let u = g.run.today.unmet.entry(res.clone()).or_insert(Fx::ZERO);
                *u = u.saturating_add(want - take);
                g.run.project_short.entry(p.id.clone()).or_default().insert(res.clone(), take);
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
/// for good drops its rates (a repeatable one keeps them), and its starting
/// research counts as bought at once.
fn complete(g: &mut GameState, p: &Project) {
    let book = book_mut(g, p.scope);
    *book.completions.entry(p.id.clone()).or_insert(0) += 1;
    book.progress.remove(&p.id);
    if p.repeat.is_none() {
        g.run.project_rates.remove(&p.id);
    }
    grant_starting_research(g);
    g.note("project", &p.id);
}

/// A scenario's start: the project counts as completed (once) in its scope,
/// with no note and no grants (the caller grants starting research).
pub(crate) fn mark_completed(g: &mut GameState, id: &str) {
    let p = content().project(id).expect("a scenario's projects are checked");
    let n = book_mut(g, p.scope).completions.entry(p.id.clone()).or_insert(0);
    *n = (*n).max(1);
}

/// A live project reward holds the flag (a `flag` reward lives only while
/// its project counts as completed).
pub fn flag_live(g: &GameState, flag: &str) -> bool {
    live_rewards(g).any(|r| matches!(r, ProjectReward::Flag { flag: f, .. } if f == flag))
}

/// Every live `starting_research` reward's items count as bought this run
/// (and known to the career): at completion and at each run's start.
pub fn grant_starting_research(g: &mut GameState) {
    let items: Vec<&'static String> = live_rewards(g)
        .filter_map(|r| if let ProjectReward::StartingResearch { research } = r { Some(research) } else { None })
        .flatten()
        .collect();
    for id in items {
        g.run.research.insert(id.clone());
        g.career.researched.insert(id.clone());
    }
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
