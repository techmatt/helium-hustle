//! **boredom**: once Burnout has fired in the career, boredom grows each day by
//! the phase of the run's day count, times the stacked rate modifiers; its cap is a base plus Recreation Domes;
//! reaching the cap forces retirement (`career`) (docs/rules/boredom.md).

use super::{param, ModuleDef, ParamKind as K, VariantDef};
use crate::config::ModuleConfig;
use crate::content::{content, Building, BuildingEffect, ProjectReward, ResearchEffect};
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
            param("escalation_per_year", K::Fx, Some(("1", "100")), "after the last phase starts, the rate is multiplied by this each full year"),
        ],
        rng: &[],
    }],
};

#[derive(Clone)]
pub struct Params {
    pub phase_days: Vec<u64>,
    pub phase_rates: Vec<Fx>,
    pub base_cap: Fx,
    pub escalation_per_year: Fx,
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
        Ok(Params {
            phase_days,
            phase_rates,
            base_cap: mc.fx("base_cap"),
            escalation_per_year: mc.fx("escalation_per_year"),
        })
    }
}

/// The 1-based phase for a run day.
pub fn phase(r: &Rules, day: u64) -> u32 {
    r.p.boredom.phase_days.iter().filter(|&&d| d <= day).count() as u32
}

/// The boredom cap: the base plus every working (active, upkeep-paid) building's
/// `store` on boredom, each × its tagged axis's modifier (the Recreation Dome).
pub fn cap(r: &Rules, g: &GameState) -> Fx {
    let mut cap = r.p.boredom.base_cap;
    for b in &content().buildings {
        let n = g.working(&b.id) as i64;
        if n == 0 {
            continue;
        }
        for e in &b.effects {
            if let BuildingEffect::Store { resource, amount } = e {
                if resource == BOREDOM {
                    cap = cap.saturating_add(amount.mul_int(n) * axis_mod(r, g, b));
                }
            }
        }
    }
    cap
}

/// A building's tagged axis's modifier (1 untagged).
fn axis_mod(r: &Rules, g: &GameState, b: &Building) -> Fx {
    b.ideology.as_deref().map_or(Fx::ONE, |a| super::ideology::modifier(r, g, a))
}

/// One working building's daily boredom relief now (its `boredom_relief` × its
/// axis's modifier); zero for a building without one.
pub fn building_relief(r: &Rules, g: &GameState, b: &Building) -> Fx {
    let mut v = Fx::ZERO;
    for e in &b.effects {
        if let BuildingEffect::BoredomRelief { amount } = e {
            v += *amount;
        }
    }
    if v == Fx::ZERO {
        Fx::ZERO
    } else {
        v * axis_mod(r, g, b)
    }
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
    for (reward, n) in super::projects::live_rewards_counted(g) {
        if let ProjectReward::BoredomModifiers { base_rate_mult, .. } = reward {
            m = m * super::projects::stacked(*base_rate_mult, n);
        }
    }
    m = m * crate::modifiers::modifier(r, g, "boredom_growth_mult");
    m * super::career::boredom_resilience(r, g) * g.boost.map_or(Fx::ONE, |b| b.boredom)
}

/// The career flag Burnout's event sets (`content/events.json`, `boredom_bites`).
pub const BURNOUT_FLAG: &str = "burnout";

/// Whether boredom grows yet: not until Burnout fires, once a career (Matt,
/// 2026-11-26). Before it, the day's growth and command boredom are zero.
pub fn started(g: &GameState) -> bool {
    g.career.flags.contains(BURNOUT_FLAG)
}

/// Full years since the last phase started (0 before it).
pub fn years_past_last_phase(r: &Rules, day: u64) -> i64 {
    let last = *r.p.boredom.phase_days.last().expect("checked at load");
    if day < last {
        return 0;
    }
    ((day - last) / r.p.economy.days_per_year as u64) as i64
}

/// The escalation factor: `escalation_per_year ^ full years past the last phase`.
pub fn escalation(r: &Rules, day: u64) -> Fx {
    r.p.boredom.escalation_per_year.pow_int(years_past_last_phase(r, day)).unwrap_or(Fx::MAX)
}

/// Today's growth rate (per day, after modifiers and escalation); zero before Burnout.
pub fn rate(r: &Rules, g: &GameState) -> Fx {
    if !started(g) {
        return Fx::ZERO;
    }
    let i = phase(r, g.run.day) as usize - 1;
    let base = r.p.boredom.phase_rates[i] * multiplier(r, g);
    base.checked_mul(escalation(r, g.run.day)).unwrap_or(Fx::MAX)
}

/// The first step of each tick: the phase is set from the day and boredom grows.
pub fn grow(r: &Rules, g: &mut GameState) {
    g.run.boredom_phase = phase(r, g.run.day);
    let v = rate(r, g);
    g.add(BOREDOM, v);
    g.track("growth", BOREDOM, v, Fx::ZERO);
    // Working buildings' daily relief (the Recreation Dome), never below zero;
    // none before Burnout, when there is nothing to relieve.
    for b in &content().buildings {
        let n = g.working(&b.id) as i64;
        if n == 0 {
            continue;
        }
        let want = building_relief(r, g, b).mul_int(n);
        let off = want.min(g.amount(BOREDOM));
        if off > Fx::ZERO {
            g.add(BOREDOM, -off);
            g.track(&format!("building:{}", b.id), BOREDOM, Fx::ZERO, off);
        }
    }
}

/// Boredom reached its cap: the run must end (checked after the tick's clamp).
pub fn at_cap(r: &Rules, g: &GameState) -> bool {
    g.amount(BOREDOM) >= cap(r, g)
}
