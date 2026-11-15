//! **ideology**: three axes funded zero-sum by the Fund commands; geometric
//! ranks; per-rank bonuses and discounts; the career's best ranks
//! (docs/rules/ideology.md).

use super::{param, ModuleDef, ParamKind as K, VariantDef};
use crate::config::ModuleConfig;
use crate::content::AXES;
use crate::fx::Fx;
use crate::math;
use crate::rules::Rules;
use crate::state::GameState;

pub static DEF: ModuleDef = ModuleDef {
    id: "ideology",
    variants: &[VariantDef {
        name: "geometric",
        params: &[
            param("rank_base_score", K::Fx, Some(("0.000001", "1000000")), "score rank 1 costs"),
            param("rank_growth", K::Fx, Some(("1.000001", "10")), "each rank costs this times the one before"),
            param("max_rank", K::Int, Some(("1", "1000")), "highest rank either way"),
            param("fund_push", K::Fx, Some(("0", "1000")), "score a Fund command adds to its axis"),
            param("fund_cross_penalty", K::Fx, Some(("0", "10")), "fraction of the push taken from each other axis"),
            param("bonus_factor", K::Fx, Some(("0.000001", "10")), "per-rank multiplier on bonuses (demand, Dream, science, speculator decay)"),
            param("discount_factor", K::Fx, Some(("0.000001", "10")), "per-rank multiplier on costs and boredom growth"),
            param("overclock_duration_factor", K::Fx, Some(("0.000001", "10")), "per Rationalist rank multiplier on overclock duration"),
        ],
        rng: &[],
    }],
};

#[derive(Clone)]
pub struct Params {
    pub rank_base_score: Fx,
    pub rank_growth: Fx,
    pub max_rank: i64,
    pub fund_push: Fx,
    pub fund_cross_penalty: Fx,
    pub bonus_factor: Fx,
    pub discount_factor: Fx,
    pub overclock_duration_factor: Fx,
}

impl Params {
    pub fn new(mc: &ModuleConfig) -> Result<Params, String> {
        Ok(Params {
            rank_base_score: mc.fx("rank_base_score"),
            rank_growth: mc.fx("rank_growth"),
            max_rank: mc.int("max_rank"),
            fund_push: mc.fx("fund_push"),
            fund_cross_penalty: mc.fx("fund_cross_penalty"),
            bonus_factor: mc.fx("bonus_factor"),
            discount_factor: mc.fx("discount_factor"),
            overclock_duration_factor: mc.fx("overclock_duration_factor"),
        })
    }
}

/// Score for rank `n`: `base × (growth^n − 1) / (growth − 1)` (rank 1 costs
/// `base`, each next one `growth` times more).
pub fn score_for_rank(r: &Rules, n: i64) -> Fx {
    let p = &r.p.ideology;
    match p.rank_growth.pow_int(n) {
        Some(g) => p.rank_base_score.checked_mul(g - Fx::ONE).map_or(Fx::MAX, |v| v / (p.rank_growth - Fx::ONE)),
        None => Fx::MAX,
    }
}

/// The whole rank a score reaches (negative for a negative score), capped at `max_rank`.
pub fn rank_for_score(r: &Rules, score: Fx) -> i64 {
    let a = score.max(-score);
    let mut n = 0;
    while n < r.p.ideology.max_rank && score_for_rank(r, n + 1) <= a {
        n += 1;
    }
    if score.is_negative() {
        -n
    } else {
        n
    }
}

/// The fractional rank a score reaches: `log_growth(score × (growth − 1) / base + 1)`.
pub fn continuous_rank(r: &Rules, score: Fx) -> Fx {
    let p = &r.p.ideology;
    if score <= Fx::ZERO {
        return Fx::ZERO;
    }
    let x = score * (p.rank_growth - Fx::ONE) / p.rank_base_score + Fx::ONE;
    math::log_in(x, p.rank_growth).unwrap_or(Fx::ZERO)
}

/// Score for a fractional rank (the inverse of [`continuous_rank`]).
pub fn score_for_continuous_rank(r: &Rules, rank: Fx) -> Fx {
    let p = &r.p.ideology;
    match math::pow_frac(p.rank_growth, rank) {
        Some(g) => p.rank_base_score * (g - Fx::ONE) / (p.rank_growth - Fx::ONE),
        None => Fx::ZERO,
    }
}

pub fn score(g: &GameState, axis: &str) -> Fx {
    g.run.ideology.get(axis).copied().unwrap_or(Fx::ZERO)
}

pub fn rank(r: &Rules, g: &GameState, axis: &str) -> i64 {
    rank_for_score(r, score(g, axis))
}

fn per_rank(r: &Rules, g: &GameState, axis: &str, factor: Fx) -> Fx {
    factor.pow_int(rank(r, g, axis)).unwrap_or(Fx::MAX)
}

/// `bonus_factor ^ rank` on `axis` (below 1 for a negative rank).
pub fn bonus(r: &Rules, g: &GameState, axis: &str) -> Fx {
    per_rank(r, g, axis, r.p.ideology.bonus_factor)
}

/// `discount_factor ^ rank` on `axis` (above 1 for a negative rank).
pub fn discount(r: &Rules, g: &GameState, axis: &str) -> Fx {
    per_rank(r, g, axis, r.p.ideology.discount_factor)
}

/// Overclock duration multiplier: `overclock_duration_factor ^ rationalist rank`.
pub fn overclock_duration(r: &Rules, g: &GameState) -> Fx {
    per_rank(r, g, "rationalist", r.p.ideology.overclock_duration_factor)
}

/// A Fund command: `fund_push` to its axis, `fund_push × fund_cross_penalty` from each other.
pub fn fund(r: &Rules, g: &mut GameState, axis: &str) {
    let p = &r.p.ideology;
    for &a in AXES {
        let v = g.run.ideology.entry(a.to_string()).or_insert(Fx::ZERO);
        if a == axis {
            *v += p.fund_push;
        } else {
            *v -= p.fund_push * p.fund_cross_penalty;
        }
    }
}

/// End of tick: the career's best rank and score per axis.
pub fn track(r: &Rules, g: &mut GameState) {
    for &a in AXES {
        let s = score(g, a);
        let n = rank_for_score(r, s);
        let best = g.career.max_ideology_ranks.entry(a.to_string()).or_insert(0);
        *best = (*best).max(n);
        let best = g.career.max_ideology_scores.entry(a.to_string()).or_insert(Fx::ZERO);
        *best = (*best).max(s);
    }
}

/// A new run's starting scores: the score of `fraction ×` the continuous
/// rank of the career's best score on each axis.
pub fn head_start(r: &Rules, g: &mut GameState, fraction: Fx) {
    for &a in AXES {
        let best = g.career.max_ideology_scores.get(a).copied().unwrap_or(Fx::ZERO);
        let start = score_for_continuous_rank(r, continuous_rank(r, best) * fraction);
        g.run.ideology.insert(a.to_string(), start);
    }
}
