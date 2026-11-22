//! **boredom**: boredom grows each day by the phase of the run's day count,
//! times the stacked rate modifiers; its cap is a base plus Recreation Domes;
//! reaching the cap forces retirement (`career`) (docs/rules/boredom.md).

use super::{param, ModuleDef, ParamKind as K, VariantDef};
use crate::config::ModuleConfig;
use crate::content::{ProjectReward, ResearchEffect};
use crate::fx::Fx;
use crate::rules::Rules;
use crate::state::GameState;

use super::economy::BOREDOM;

pub static DEF: ModuleDef = ModuleDef {
    id: "boredom",
    variants: &[VariantDef {
        name: "phased",
        params: &[
            param("phase_days", K::IntList, Some(("0", "1000000")), "the run day each phase starts (first 0, rising)"),
            param("phase_rates", K::FxList, Some(("0", "1000")), "boredom per day in each phase, before modifiers"),
            param("base_cap", K::Fx, Some(("1", "1000000")), "the boredom cap before Recreation Domes"),
        ],
        rng: &[],
    }],
};

#[derive(Clone)]
pub struct Params {
    pub phase_days: Vec<u64>,
    pub phase_rates: Vec<Fx>,
    pub base_cap: Fx,
}

impl Params {
    pub fn new(mc: &ModuleConfig) -> Result<Params, String> {
        let phase_days: Vec<u64> = mc.int_list("phase_days").into_iter().map(|d| d as u64).collect();
        let phase_rates = mc.fx_list("phase_rates");
        let rising = phase_days.windows(2).all(|w| w[0] < w[1]);
        if phase_days.first() != Some(&0) || !rising || phase_days.len() != phase_rates.len() {
            return Err("config: modules.boredom.params: phase_days must start at 0 and rise, \
                        one per phase_rates value"
                .into());
        }
        Ok(Params { phase_days, phase_rates, base_cap: mc.fx("base_cap") })
    }
}

/// The 1-based phase for a run day.
pub fn phase(r: &Rules, day: u64) -> u32 {
    r.p.boredom.phase_days.iter().filter(|&&d| d <= day).count() as u32
}

/// The boredom cap: the base plus every working (active, upkeep-paid) building's
/// `store` on boredom.
pub fn cap(r: &Rules, g: &GameState) -> Fx {
    r.p.boredom.base_cap + super::economy::stored(g, BOREDOM)
}

/// The stacked growth-rate multiplier: research (Stress Tolerance), Humanist
/// rank (discount ^ rank), completed projects (AI Consciousness Act) and the
/// career's boredom resilience; the debug boost's `boredom` factor when set.
pub fn multiplier(r: &Rules, g: &GameState) -> Fx {
    let mut m = Fx::ONE;
    for effect in super::research::live_effects(g) {
        if let ResearchEffect::BoredomRateMultiplier { value } = effect {
            m = m * *value;
        }
    }
    m = m * super::ideology::discount(r, g, "humanist");
    for (reward, n) in super::projects::live_rewards_counted(g) {
        if let ProjectReward::BoredomModifiers { base_rate_mult, .. } = reward {
            m = m * super::projects::stacked(*base_rate_mult, n);
        }
    }
    m = m * crate::modifiers::modifier(r, g, "boredom_growth_mult");
    m * super::career::boredom_resilience(r, g) * g.boost.map_or(Fx::ONE, |b| b.boredom)
}

/// Today's growth rate (per day, after modifiers).
pub fn rate(r: &Rules, g: &GameState) -> Fx {
    let i = phase(r, g.run.day) as usize - 1;
    r.p.boredom.phase_rates[i] * multiplier(r, g)
}

/// The first step of each tick: the phase is set from the day and boredom grows.
pub fn grow(r: &Rules, g: &mut GameState) {
    g.run.boredom_phase = phase(r, g.run.day);
    let v = rate(r, g);
    g.add(BOREDOM, v);
    g.track("growth", BOREDOM, v, Fx::ZERO);
}

/// Boredom reached its cap: the run must end (checked after the tick's clamp).
pub fn at_cap(r: &Rules, g: &GameState) -> bool {
    g.amount(BOREDOM) >= cap(r, g)
}
