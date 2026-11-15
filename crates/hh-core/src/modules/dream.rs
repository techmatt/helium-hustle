//! **dream**: what the Dream command does to boredom (docs/rules/dream.md). A
//! module of its own so another Dream (Dream Echo) can be a variant.

use super::{param, ModuleDef, ParamKind as K, VariantDef};
use crate::config::ModuleConfig;
use crate::fx::Fx;
use crate::rules::Rules;
use crate::state::GameState;

use super::economy::BOREDOM;

pub static DEF: ModuleDef = ModuleDef {
    id: "dream",
    variants: &[VariantDef {
        name: "flat",
        params: &[param("relief", K::Fx, Some(("0", "1000000")), "boredom removed per Dream, before the Humanist bonus")],
        rng: &[],
    }],
};

#[derive(Clone)]
pub struct Params {
    pub relief: Fx,
}

impl Params {
    pub fn new(mc: &ModuleConfig) -> Result<Params, String> {
        Ok(Params { relief: mc.fx("relief") })
    }
}

/// Boredom removed by one Dream now: `relief × bonus ^ humanist rank`.
pub fn relief(r: &Rules, g: &GameState) -> Fx {
    r.p.dream.relief * super::ideology::bonus(r, g, "humanist")
}

/// Flat Dream: boredom falls by [`relief`], not below zero.
pub fn dream(r: &Rules, g: &mut GameState) {
    let v = g.amount(BOREDOM) - relief(r, g);
    g.run.resources.insert(BOREDOM.to_string(), v.max(Fx::ZERO));
}
