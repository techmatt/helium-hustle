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
            param("rank_multiplier", K::Fx, Some(("1", "10")), "an axis's modifier m is this to the power of its rank: tagged buildings cost ÷m, its boredom tool works ×m"),
            param("major_rank", K::Int, Some(("1", "1000")), "the first major rank: at or past it an axis's discrete major effect is on"),
            param("half_life_days", K::Fx, Some(("1", "1000000")), "days of the retirement gap that halve every score (proportional decay toward 0)"),
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
    pub rank_multiplier: Fx,
    pub major_rank: i64,
    pub half_life_days: Fx,
}

impl Params {
    pub fn new(mc: &ModuleConfig) -> Result<Params, String> {
        Ok(Params {
            rank_base_score: mc.fx("rank_base_score"),
            rank_growth: mc.fx("rank_growth"),
            max_rank: mc.int("max_rank"),
            fund_push: mc.fx("fund_push"),
            fund_cross_penalty: mc.fx("fund_cross_penalty"),
            rank_multiplier: mc.fx("rank_multiplier"),
            major_rank: mc.int("major_rank"),
            half_life_days: mc.fx("half_life_days"),
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

/// The axis is at or past its first major rank (`major_rank`, positive):
/// its discrete major effect is on (ideology.md, *Major ranks*).
pub fn major(r: &Rules, g: &GameState, axis: &str) -> bool {
    rank(r, g, axis) >= r.p.ideology.major_rank
}

/// The retirement gap's daily decay factor on every score:
/// `0.5 ^ (1 / (half_life_days × ideology_half_life_mult))` (fractional,
/// through `libm`; the modifier is Three Voices' reward).
pub fn gap_decay_factor(r: &Rules, g: &GameState) -> Fx {
    let half_life = r.p.ideology.half_life_days * crate::modifiers::modifier(r, g, "ideology_half_life_mult");
    let exp = Fx::ONE / half_life;
    math::pow_frac(Fx::ONE / Fx::from_int(2), exp).unwrap_or(Fx::ZERO)
}

pub fn score(g: &GameState, axis: &str) -> Fx {
    // An unknown axis would silently score 0: catch a typo at a call site in tests.
    debug_assert!(AXES.contains(&axis), "unknown ideology axis {axis:?}");
    g.run.ideology.get(axis).copied().unwrap_or(Fx::ZERO)
}

pub fn rank(r: &Rules, g: &GameState, axis: &str) -> i64 {
    rank_for_score(r, score(g, axis))
}

/// The axis's one modifier, `m = rank_multiplier ^ rank` (below 1 for a
/// negative rank, never zero). It does two things only: the axis's tagged
/// buildings cost ÷m, and its boredom tool works ×m (docs/rules/ideology.md).
pub fn modifier(r: &Rules, g: &GameState, axis: &str) -> Fx {
    modifier_at(r, rank(r, g, axis))
}

/// `rank_multiplier ^ rank` for a given rank.
pub fn modifier_at(r: &Rules, rank: i64) -> Fx {
    r.p.ideology.rank_multiplier.pow_int(rank).unwrap_or(Fx::MAX).max(Fx::EPSILON)
}

/// A boredom tool's use nudges its axis's score by `amount` (no cross penalty).
pub fn nudge(g: &mut GameState, axis: &str, amount: Fx) {
    let v = g.run.ideology.entry(axis.to_string()).or_insert(Fx::ZERO);
    *v += amount;
}

/// The modifier key multiplying an axis's own growth (`None`: none yet).
fn growth_key(axis: &str) -> Option<&'static str> {
    match axis {
        "nationalist" => Some("nationalist_growth_mult"),
        _ => None,
    }
}

/// A Fund command: `fund_push` (times the axis's growth modifier) to its
/// axis, `fund_push × fund_cross_penalty` from each other.
pub fn fund(r: &Rules, g: &mut GameState, axis: &str) {
    let p = &r.p.ideology;
    // The Consensus Engine (draft Earth tech) sets this to 0.
    let cross = p.fund_push * p.fund_cross_penalty * crate::modifiers::modifier(r, g, "ideology_cross_mult");
    let push = growth_key(axis).map_or(p.fund_push, |k| p.fund_push * crate::modifiers::modifier(r, g, k));
    for &a in AXES {
        let v = g.run.ideology.entry(a.to_string()).or_insert(Fx::ZERO);
        if a == axis {
            *v += push;
        } else {
            *v -= cross;
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

/// A new run's starting scores: the ones the career carried out of the last
/// run, decayed through the gap (none in run 1 or a timeline's first run).
pub fn carry_in(g: &mut GameState) {
    for &a in AXES {
        let s = g.career.ideology.get(a).copied().unwrap_or(Fx::ZERO);
        g.run.ideology.insert(a.to_string(), s);
    }
}
