//! **Placeholder rules**, just big enough to exercise the engine (config,
//! content, `Fx`, the libm boundary, an RNG stream, an action, a cap). Not
//! game design: the rules port replaces this module whole.
//!
//! Each day: credits rise by `credits_per_day`; every building makes its
//! content `produces` per copy, times a sunlight factor drawn uniformly from
//! `1 - sunlight_spread ..= 1 + sunlight_spread`; energy is then capped at
//! `energy_cap`. A building costs its content `cost` times
//! `cost_growth ^ owned`.

use std::collections::BTreeMap;

use super::{Module, ModuleDef, ParamDef, ParamKind, VariantDef};
use crate::action::Action;
use crate::config::ModuleConfig;
use crate::content::{content, Building};
use crate::fx::Fx;
use crate::math;
use crate::state::GameState;
use crate::view::{BuildingView, View};

const SUNLIGHT: &str = "sunlight";
const ENERGY: &str = "energy";
const CREDITS: &str = "credits";

pub static DEF: ModuleDef = ModuleDef {
    id: "placeholder",
    variants: &[VariantDef {
        name: "basic",
        params: &[
            ParamDef {
                name: "start_credits",
                kind: ParamKind::Fx,
                range: Some(("0", "1000000000")),
                doc: "credits at day 0",
            },
            ParamDef {
                name: "credits_per_day",
                kind: ParamKind::Fx,
                range: Some(("0", "1000000000")),
                doc: "flat credit income per day",
            },
            ParamDef {
                name: "energy_cap",
                kind: ParamKind::Fx,
                range: Some(("0", "1000000000")),
                doc: "most energy that can be held",
            },
            ParamDef {
                name: "sunlight_spread",
                kind: ParamKind::Fx,
                range: Some(("0", "1")),
                doc: "daily production varies by up to this fraction either way",
            },
            ParamDef {
                name: "cost_growth",
                kind: ParamKind::Fx,
                range: Some(("1", "10")),
                doc: "each building owned multiplies the next one's cost by this",
            },
        ],
        rng: &[SUNLIGHT],
    }],
};

pub struct Placeholder {
    start_credits: Fx,
    credits_per_day: Fx,
    energy_cap: Fx,
    sunlight_spread: Fx,
    cost_growth: Fx,
}

impl Placeholder {
    pub fn new(mc: &ModuleConfig) -> Self {
        Placeholder {
            start_credits: mc.fx("start_credits"),
            credits_per_day: mc.fx("credits_per_day"),
            energy_cap: mc.fx("energy_cap"),
            sunlight_spread: mc.fx("sunlight_spread"),
            cost_growth: mc.fx("cost_growth"),
        }
    }

    /// The price of the next copy of `b` when `owned` are held. A price past
    /// `Fx`'s range is `Fx::MAX`: shown, never affordable, never a panic.
    fn cost(&self, b: &Building, owned: u32) -> BTreeMap<String, Fx> {
        let factor = math::pow_int(self.cost_growth, owned);
        b.cost
            .iter()
            .map(|(r, &base)| {
                let price = factor.and_then(|f| base.checked_mul(f)).unwrap_or(Fx::MAX);
                (r.clone(), price)
            })
            .collect()
    }

    fn affordable(g: &GameState, cost: &BTreeMap<String, Fx>) -> bool {
        cost.iter().all(|(r, &c)| c < Fx::MAX && g.amount(r) >= c)
    }

    fn sunlight(&self, g: &mut GameState) -> Fx {
        let steps = self.sunlight_spread.steps();
        let k = g.stream(SUNLIGHT).below(
            u32::try_from(2 * steps + 1).expect("sunlight_spread <= 1 keeps the draw in u32"),
        );
        (Fx::ONE - self.sunlight_spread).plus_steps(k as i64)
    }
}

impl Module for Placeholder {
    fn init(&self, g: &mut GameState) {
        *g.resources.get_mut(CREDITS).expect("content has credits") = self.start_credits;
    }

    fn tick(&self, g: &mut GameState) {
        let sun = self.sunlight(g);
        // Uncapped totals saturate at Fx::MAX rather than panic in a very long game.
        let c = g.resources.get_mut(CREDITS).expect("content has credits");
        *c = c.saturating_add(self.credits_per_day);
        for b in &content().buildings {
            let n = g.owned(&b.id);
            if n == 0 {
                continue;
            }
            for (r, &per) in &b.produces {
                let made = per.mul_int(n as i64) * sun;
                let v = g.resources.get_mut(r).expect("content checked");
                *v = v.saturating_add(made);
            }
        }
        let e = g.resources.get_mut(ENERGY).expect("content has energy");
        *e = (*e).min(self.energy_cap);
    }

    fn apply(&self, g: &mut GameState, a: &Action) -> Option<Result<(), String>> {
        let Action::Buy { building } = a;
        Some((|| {
            let b = content()
                .building(building)
                .ok_or_else(|| format!("buy: unknown building {building:?}"))?;
            let cost = self.cost(b, g.owned(&b.id));
            if !Self::affordable(g, &cost) {
                return Err(format!("buy {}: can't afford it", b.name));
            }
            for (r, c) in cost {
                *g.resources.get_mut(&r).expect("content checked") -= c;
            }
            *g.buildings.entry(b.id.clone()).or_insert(0) += 1;
            Ok(())
        })())
    }

    fn view(&self, g: &GameState, v: &mut View) {
        for r in &mut v.resources {
            if r.id == ENERGY {
                r.cap = Some(self.energy_cap);
            }
            if r.id == CREDITS {
                r.per_day += self.credits_per_day;
            }
        }
        for b in &content().buildings {
            let n = g.owned(&b.id);
            for (res, &per) in &b.produces {
                if let Some(r) = v.resources.iter_mut().find(|r| &r.id == res) {
                    r.per_day += per.mul_int(n as i64);
                }
            }
            let cost = self.cost(b, n);
            v.buildings.push(BuildingView {
                id: b.id.clone(),
                name: b.name.clone(),
                owned: n,
                affordable: Self::affordable(g, &cost),
                cost,
                buy: Action::Buy {
                    building: b.id.clone(),
                },
            });
        }
    }
}
