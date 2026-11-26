//! **attention**: the minimal Attention model that brings the adversaries in
//! (docs/rules/attention.md, draft). Each adversary has one Attention number,
//! kept in the career: its driver raises it (speculators: units sold; rivals:
//! land claimed), it decays linearly each day (and through the retirement
//! gap), and the adversary is present while it is at or above its threshold
//! (one tier). Neutralizing sets it to zero.

use super::{param, ModuleDef, ParamKind as K, VariantDef};
use crate::config::ModuleConfig;
use crate::fx::Fx;
use crate::rules::Rules;
use crate::state::GameState;

/// The adversaries, fixed ids (like the ideology axes).
pub const SPECULATORS: &str = "speculators";
pub const RIVALS: &str = "rivals";
pub const ADVERSARIES: &[&str] = &[SPECULATORS, RIVALS];

pub static DEF: ModuleDef = ModuleDef {
    id: "attention",
    variants: &[VariantDef {
        name: "minimal",
        params: &[
            param("speculator_per_unit", K::Fx, Some(("0", "1000")), "speculator Attention per unit of cargo launched"),
            param("speculator_threshold", K::Fx, Some(("0", "100000000")), "speculator Attention at which speculators are present"),
            param("speculator_decay", K::Fx, Some(("0", "1000000")), "speculator Attention lost per day (linear)"),
            param("rival_per_land", K::Fx, Some(("0", "1000")), "rival Attention per unit of land claimed"),
            param("rival_threshold", K::Fx, Some(("0", "100000000")), "rival Attention at which the rival AIs are present"),
            param("rival_decay", K::Fx, Some(("0", "1000000")), "rival Attention lost per day (linear)"),
            param("rival_land_price_mult", K::Fx, Some(("1", "1000")), "land price multiplier while the rivals are present"),
            param("disrupt_attention", K::Fx, Some(("0", "1000000")), "speculator Attention one Disrupt Speculators takes off"),
        ],
        rng: &[],
    }],
};

#[derive(Clone)]
pub struct Params {
    pub speculator_per_unit: Fx,
    pub speculator_threshold: Fx,
    pub speculator_decay: Fx,
    pub rival_per_land: Fx,
    pub rival_threshold: Fx,
    pub rival_decay: Fx,
    pub rival_land_price_mult: Fx,
    pub disrupt_attention: Fx,
}

impl Params {
    pub fn new(mc: &ModuleConfig) -> Result<Params, String> {
        Ok(Params {
            speculator_per_unit: mc.fx("speculator_per_unit"),
            speculator_threshold: mc.fx("speculator_threshold"),
            speculator_decay: mc.fx("speculator_decay"),
            rival_per_land: mc.fx("rival_per_land"),
            rival_threshold: mc.fx("rival_threshold"),
            rival_decay: mc.fx("rival_decay"),
            rival_land_price_mult: mc.fx("rival_land_price_mult"),
            disrupt_attention: mc.fx("disrupt_attention"),
        })
    }
}

fn threshold(r: &Rules, who: &str) -> Fx {
    if who == SPECULATORS {
        r.p.attention.speculator_threshold
    } else {
        r.p.attention.rival_threshold
    }
}

fn decay(r: &Rules, who: &str) -> Fx {
    if who == SPECULATORS {
        r.p.attention.speculator_decay
    } else {
        r.p.attention.rival_decay
    }
}

/// The adversary's Attention now.
pub fn attention(g: &GameState, who: &str) -> Fx {
    g.career.attention.get(who).copied().unwrap_or(Fx::ZERO)
}

/// The Attention at which the adversary is present.
pub fn threshold_of(r: &Rules, who: &str) -> Fx {
    threshold(r, who)
}

/// The adversary is present: its Attention is at or above its threshold.
pub fn present(r: &Rules, g: &GameState, who: &str) -> bool {
    attention(g, who) >= threshold(r, who)
}

fn add(g: &mut GameState, who: &str, v: Fx) {
    let a = g.career.attention.entry(who.to_string()).or_insert(Fx::ZERO);
    *a = a.saturating_add(v).max(Fx::ZERO);
}

/// A launch sold `units` of cargo.
pub fn on_sale(r: &Rules, g: &mut GameState, units: Fx) {
    add(g, SPECULATORS, units * r.p.attention.speculator_per_unit);
}

/// The career flag that opens Act 2, where the rivals come in: before it,
/// land claimed draws no rival Attention (attention.md).
pub const RIVALS_FROM_FLAG: &str = "swarm_revealed";

/// `land` units of land were claimed.
pub fn on_land(r: &Rules, g: &mut GameState, land: Fx) {
    if g.career.flags.contains(RIVALS_FROM_FLAG) {
        add(g, RIVALS, land * r.p.attention.rival_per_land);
    }
}

/// Disrupt Speculators ran once.
pub fn on_disrupt(r: &Rules, g: &mut GameState) {
    add(g, SPECULATORS, Fx::ZERO - r.p.attention.disrupt_attention);
}

/// One day's linear decay for every adversary (each day, and each day of
/// the retirement gap).
pub fn decay_day(r: &Rules, g: &mut GameState) {
    for &who in ADVERSARIES {
        add(g, who, Fx::ZERO - decay(r, who));
    }
}

/// `days` of [`decay_day`] at once (the retirement gap): the decay is linear and
/// floored at zero, so `days` of it are `max(0, a − days × decay)`, exact. The view
/// previews a retirement on every look, and the gap's 1,825 days one by one were
/// most of a late view's cost (094).
pub fn decay_days(r: &Rules, g: &mut GameState, days: u64) {
    if days == 0 {
        return;
    }
    // In range: gap_days ≤ 100,000 and a decay ≤ 1,000,000 a day.
    let n = i64::try_from(days).expect("a gap's days fit an i64");
    for &who in ADVERSARIES {
        let lost = decay(r, who).mul_int(n);
        let a = g.career.attention.entry(who.to_string()).or_insert(Fx::ZERO);
        *a = if lost >= *a { Fx::ZERO } else { *a - lost };
    }
}

/// The neutralize stub: the adversary's Attention goes to zero.
pub fn neutralize(g: &mut GameState, who: &str) {
    g.career.attention.insert(who.to_string(), Fx::ZERO);
}

/// The land price factor from the adversaries (the rivals' while present).
pub fn land_price_mult(r: &Rules, g: &GameState) -> Fx {
    if present(r, g, RIVALS) {
        r.p.attention.rival_land_price_mult
    } else {
        Fx::ONE
    }
}
