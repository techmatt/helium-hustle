//! **career**: runs and retirement. What persists, what resets, how a run
//! starts, and the career bonuses earned from personal bests
//! (docs/rules/career.md).

use super::{param, ModuleDef, ParamKind as K, VariantDef};
use crate::config::ModuleConfig;
use crate::content::content;
use crate::fx::Fx;
use crate::math;
use crate::rules::Rules;
use crate::state::{GameState, Run};

use super::economy::CREDITS;

pub const RETIREMENT_PANEL: &str = "retirement";

pub static DEF: ModuleDef = ModuleDef {
    id: "career",
    variants: &[VariantDef {
        name: "arc1",
        params: &[
            param("credits_bonus_fraction", K::Fx, Some(("0", "10")), "starting credits: this fraction of the best run's credits earned (whole credits)"),
            param("boredom_resilience_base", K::Fx, Some(("0.000001", "1")), "boredom growth multiplier per resilience period of the longest run"),
            param("boredom_resilience_period", K::Fx, Some(("1", "1000000")), "days of the longest run per resilience step"),
            param("gap_days", K::Int, Some(("0", "100000")), "days of the retirement gap, simulated day by day (slow systems only)"),
            param("buy_power_peak_threshold", K::Fx, Some(("0", "1000000")), "peak energy production per day above which Buy Power scales"),
            param("buy_power_per_energy", K::Fx, Some(("0", "10")), "Buy Power scaling per unit of peak production over the threshold"),
        ],
        rng: &[],
    }],
};

#[derive(Clone)]
pub struct Params {
    pub credits_bonus_fraction: Fx,
    pub boredom_resilience_base: Fx,
    pub boredom_resilience_period: Fx,
    pub gap_days: u64,
    pub buy_power_peak_threshold: Fx,
    pub buy_power_per_energy: Fx,
}

impl Params {
    pub fn new(mc: &ModuleConfig) -> Result<Params, String> {
        Ok(Params {
            credits_bonus_fraction: mc.fx("credits_bonus_fraction"),
            boredom_resilience_base: mc.fx("boredom_resilience_base"),
            boredom_resilience_period: mc.fx("boredom_resilience_period"),
            gap_days: mc.int("gap_days") as u64,
            buy_power_peak_threshold: mc.fx("buy_power_peak_threshold"),
            buy_power_per_energy: mc.fx("buy_power_per_energy"),
        })
    }
}

/// Boredom growth multiplier from the longest run:
/// `base ^ (best_run_days / period)` (through libm: a fractional power).
pub fn boredom_resilience(r: &Rules, g: &GameState) -> Fx {
    let p = &r.p.career;
    let steps = Fx::from_int(g.career.best_run_days as i64) / p.boredom_resilience_period;
    math::pow_frac(p.boredom_resilience_base, steps).unwrap_or(Fx::ONE)
}

/// Buy Power scaling from peak production: `1 + max(0, peak − threshold) × per_energy`.
pub fn buy_power_mult(r: &Rules, g: &GameState) -> Fx {
    let p = &r.p.career;
    Fx::ONE + (g.career.peak_power - p.buy_power_peak_threshold).max(Fx::ZERO) * p.buy_power_per_energy
}

/// Starting credits from the best run: `floor(best_run_credits × fraction)`.
pub fn starting_credits(r: &Rules, g: &GameState) -> Fx {
    let v = g.career.best_run_credits * r.p.career.credits_bonus_fraction;
    Fx::from_int(v.floor_int())
}

/// A flag holds: a career flag (an event's or a command's, kept for the
/// career), or a live project reward's (held while its project counts as
/// completed: docs/rules/projects.md).
pub fn flag_set(g: &GameState, flag: &str) -> bool {
    g.career.flags.contains(flag) || super::projects::flag_live(g, flag)
}

/// Starts a run on the current career (a new game, or after a retirement):
/// the config's starting state, career bonuses, bonus buildings, the seen
/// events' unlocks, live starting research, empty programs and fresh markets.
pub fn new_run(r: &Rules, g: &mut GameState) {
    g.run = Run {
        resources: content().resources.iter().map(|res| (res.id.clone(), Fx::ZERO)).collect(),
        buildings: super::economy::starting_buildings(r),
        loading_priority: super::shipments::goods(r),
        ..Run::default()
    };
    for (res, &v) in &r.p.economy.starting_resources {
        g.run.resources.insert(res.clone(), v);
    }
    super::ideology::carry_in(g);
    let credits = starting_credits(r, g);
    g.add(CREDITS, credits);
    let mut bonus = super::projects::starting_bonus(g);
    bonus.extend(super::achievements::starting_bonus(g));
    for (b, n) in bonus {
        super::economy::grant_bonus(g, &b, n);
    }
    let owned: Vec<String> = g.run.buildings.keys().cloned().collect();
    g.career.owned_buildings.extend(owned);
    super::events::reapply(g);
    super::projects::grant_starting_research(g);
    super::economy::refresh(g);
    let procs = super::programs::capacity(g);
    g.run.programs = super::programs::new_programs(r, procs);
    super::demand::init(r, g);
    g.run.boredom_phase = super::boredom::phase(r, 0);
    super::timeline::run_start(r, g);
}

/// Ends the run: its stats fold into the career (lifetime totals, personal
/// bests), the retirement is logged, and the next run starts.
pub fn retire(r: &Rules, g: &mut GameState, forced: bool) {
    close_run(r, g, forced);
    super::timeline::on_retire(r, g);
    new_run(r, g);
}

/// What the next run would start with if this one retired now: the career as a
/// retirement leaves it, with the carried ideology in the run, without the new run's
/// setup (a third of a late view's cost, 094). Every next-run number (starting
/// credits, resilience, Buy Power, ideology ranks) reads only these.
pub fn next_run_preview(r: &Rules, g: &GameState) -> GameState {
    let mut h = g.clone();
    close_run(r, &mut h, false);
    super::ideology::carry_in(&mut h);
    h
}

/// A retirement's first part: the run's stats fold into the career, then the gap.
fn close_run(r: &Rules, g: &mut GameState, forced: bool) {
    let s = g.run.stats.clone();
    let day = g.run.day;
    g.note(if forced { "forced_retirement" } else { "retired" }, "");
    let ideology_ranks = crate::content::AXES
        .iter()
        .map(|&a| (a.to_string(), super::ideology::rank(r, g, a)))
        .collect();
    g.career.last_run = Some(crate::state::LastRun {
        run: g.career.run_number,
        days: day,
        forced,
        stats: s.clone(),
        ideology_ranks,
    });
    let c = &mut g.career;
    c.lifetime.credits_earned = c.lifetime.credits_earned.saturating_add(s.credits_earned);
    c.lifetime.shipments += s.shipments;
    c.lifetime.days += day;
    c.lifetime.buildings_built += s.buildings_built;
    c.lifetime.research_completed += s.research_completed;
    c.lifetime.trade_credits = c.lifetime.trade_credits.saturating_add(s.trade_credits);
    c.best_run_days = c.best_run_days.max(day);
    c.best_run_credits = c.best_run_credits.max(s.credits_earned);
    c.best_run_shipments = c.best_run_shipments.max(s.shipments);
    c.peak_power = c.peak_power.max(s.peak_energy);
    c.retirements += 1;
    c.run_number += 1;
    c.ideology = g.run.ideology.clone();
    gap(r, g);
}

/// The retirement gap (draft): `gap_days` days, simulated one by one, in which
/// only the slow systems run: every carried ideology score decays
/// proportionally toward zero (one half-life for all three axes), and every
/// adversary's Attention decays linearly. Unseen in Act 1 (no calendar).
///
/// The scores and the Attention decay apart (neither reads the other), so each runs
/// its own days; and a score's day is a function of its value alone, so once one
/// leaves it as it was, so would every later one: the loop stops there. The Attention's
/// linear decay takes its days at once (`attention::decay_days`). The same numbers as
/// day by day, much faster: the view previews a retirement on every look.
fn gap(r: &Rules, g: &mut GameState) {
    let (f, days) = (super::ideology::gap_decay_factor(r, g), r.p.career.gap_days);
    for v in g.career.ideology.values_mut() {
        for _ in 0..days {
            let next = *v * f;
            if next == *v {
                break;
            }
            *v = next;
        }
    }
    super::attention::decay_days(r, g, days);
    g.career.gap_days += days;
}

/// Why the player can't retire now, if they can't: retiring needs the retirement panel
/// unlocked. The view's `can_retire` and the Retire action both ask this.
pub fn retire_blocker(g: &GameState) -> Option<&'static str> {
    (!g.run.enabled_panels.contains(RETIREMENT_PANEL)).then_some("retire: not unlocked yet")
}

/// Voluntary retirement (the Retire action).
pub fn retire_action(r: &Rules, g: &mut GameState) -> Result<(), String> {
    if let Some(e) = retire_blocker(g) {
        return Err(e.into());
    }
    retire(r, g, false);
    Ok(())
}
