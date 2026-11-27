//! **demand**: each tradeable good's demand, from gradient noise, speculator
//! pools, rival dumps, launch saturation, Promote and coupling, times the
//! Nationalist bonus (docs/rules/demand.md).

use super::{param, ModuleDef, ParamKind as K, VariantDef};
use crate::config::ModuleConfig;
use crate::content::{content, BuildingEffect};
use crate::fx::Fx;
use crate::hash::lattice;
use crate::modifiers::modifier;
use crate::rules::Rules;
use crate::state::{GameState, Market};

const NOISE: &str = "demand_noise";
const SPECULATORS: &str = "speculators";
const RIVALS: &str = "rivals";
const SATURATION: &str = "launch_saturation";
const DISRUPT: &str = "disrupt";

pub static DEF: ModuleDef = ModuleDef {
    id: "demand",
    variants: &[VariantDef {
        name: "arc1",
        params: &[
            param("min_demand", K::Fx, Some(("0", "10")), "demand floor"),
            param("max_demand", K::Fx, Some(("0", "10")), "demand ceiling before demand_ceiling modifiers"),
            param("base_demand", K::Fx, Some(("0", "10")), "demand with every force at zero"),
            param("noise_amplitude", K::Fx, Some(("0", "10")), "the noise term's largest swing either way"),
            param("noise_freq_min", K::Fx, Some(("0", "100")), "lowest per-good noise frequency (cycles per day)"),
            param("noise_freq_max", K::Fx, Some(("0", "100")), "highest per-good noise frequency"),
            param("noise_octave_freqs", K::FxList, Some(("0", "1000")), "each octave's frequency multiplier"),
            param("noise_octave_gains", K::FxList, Some(("0", "1000")), "each octave's weight"),
            param("speculator_max_suppression", K::Fx, Some(("0", "10")), "demand a pool suppresses as it grows without bound"),
            param("speculator_half_point", K::Fx, Some(("0.000001", "1000000")), "pool size that suppresses half the maximum"),
            param("speculator_decay", K::Fx, Some(("0", "1")), "fraction of each pool gone per day"),
            param("burst_interval_min", K::Int, Some(("1", "100000")), "fewest days between speculator bursts"),
            param("burst_interval_max", K::Int, Some(("1", "100000")), "most days between speculator bursts"),
            param("burst_size_min", K::Int, Some(("0", "1000000")), "smallest burst before growth"),
            param("burst_size_max", K::Int, Some(("0", "1000000")), "largest burst before growth"),
            param("burst_growth", K::Fx, Some(("1", "10")), "each burst this run multiplies the next one's size by this"),
            param("disrupt_min", K::Fx, Some(("0", "1000")), "fewest speculators one Disrupt removes"),
            param("disrupt_max", K::Fx, Some(("0", "1000")), "most speculators one Disrupt removes"),
            param("arbitrage_decay_per_engine", K::Fx, Some(("0", "1000")), "speculators each working (active, upkeep paid) Arbitrage Engine removes from every pool per day"),
            param("promote_effect", K::Fx, Some(("0", "10")), "demand one Promote adds, before promote_effectiveness_mult"),
            param("promote_decay", K::Fx, Some(("0", "10")), "promote effect lost per day"),
            param("promote_speculator_dampening", K::Fx, Some(("0", "1")), "a Promote multiplies its good's speculator pool by this"),
            param("rival_decay", K::Fx, Some(("0", "10")), "rival pressure lost per day"),
            param("saturation_min", K::Fx, Some(("0", "10")), "least demand a launch takes from its good"),
            param("saturation_max", K::Fx, Some(("0", "10")), "most demand a launch takes from its good"),
            param("saturation_decay", K::Fx, Some(("0", "10")), "launch saturation lost per day"),
            param("coupling_fraction", K::Fx, Some(("0", "10")), "share of the other goods' promote effect a good gains"),
            param("tier_bounds", K::FxList, Some(("0", "10")), "demand bounds between the LOW / MEDIUM / HIGH / VERY HIGH labels"),
        ],
        rng: &[NOISE, SPECULATORS, RIVALS, SATURATION, DISRUPT],
    }],
};

#[derive(Clone)]
pub struct Params {
    pub min_demand: Fx,
    pub max_demand: Fx,
    pub base_demand: Fx,
    pub noise_amplitude: Fx,
    pub noise_freq_min: Fx,
    pub noise_freq_max: Fx,
    pub noise_octave_freqs: Vec<Fx>,
    pub noise_octave_gains: Vec<Fx>,
    pub speculator_max_suppression: Fx,
    pub speculator_half_point: Fx,
    pub speculator_decay: Fx,
    pub burst_interval_min: u32,
    pub burst_interval_max: u32,
    pub burst_size_min: u32,
    pub burst_size_max: u32,
    pub burst_growth: Fx,
    pub disrupt_min: Fx,
    pub disrupt_max: Fx,
    pub arbitrage_decay_per_engine: Fx,
    pub promote_effect: Fx,
    pub promote_decay: Fx,
    pub promote_speculator_dampening: Fx,
    pub rival_decay: Fx,
    pub saturation_min: Fx,
    pub saturation_max: Fx,
    pub saturation_decay: Fx,
    pub coupling_fraction: Fx,
    pub tier_bounds: Vec<Fx>,
}

impl Params {
    pub fn new(mc: &ModuleConfig) -> Result<Params, String> {
        let u = |n: &str| mc.int(n) as u32;
        let p = Params {
            min_demand: mc.fx("min_demand"),
            max_demand: mc.fx("max_demand"),
            base_demand: mc.fx("base_demand"),
            noise_amplitude: mc.fx("noise_amplitude"),
            noise_freq_min: mc.fx("noise_freq_min"),
            noise_freq_max: mc.fx("noise_freq_max"),
            noise_octave_freqs: mc.fx_list("noise_octave_freqs"),
            noise_octave_gains: mc.fx_list("noise_octave_gains"),
            speculator_max_suppression: mc.fx("speculator_max_suppression"),
            speculator_half_point: mc.fx("speculator_half_point"),
            speculator_decay: mc.fx("speculator_decay"),
            burst_interval_min: u("burst_interval_min"),
            burst_interval_max: u("burst_interval_max"),
            burst_size_min: u("burst_size_min"),
            burst_size_max: u("burst_size_max"),
            burst_growth: mc.fx("burst_growth"),
            disrupt_min: mc.fx("disrupt_min"),
            disrupt_max: mc.fx("disrupt_max"),
            arbitrage_decay_per_engine: mc.fx("arbitrage_decay_per_engine"),
            promote_effect: mc.fx("promote_effect"),
            promote_decay: mc.fx("promote_decay"),
            promote_speculator_dampening: mc.fx("promote_speculator_dampening"),
            rival_decay: mc.fx("rival_decay"),
            saturation_min: mc.fx("saturation_min"),
            saturation_max: mc.fx("saturation_max"),
            saturation_decay: mc.fx("saturation_decay"),
            coupling_fraction: mc.fx("coupling_fraction"),
            tier_bounds: mc.fx_list("tier_bounds"),
        };
        let mut bad = Vec::new();
        let gains: Fx = p.noise_octave_gains.iter().copied().sum();
        if p.noise_octave_freqs.len() != p.noise_octave_gains.len() || gains <= Fx::ZERO {
            bad.push("noise_octave_freqs and noise_octave_gains: one gain per octave, some positive");
        }
        let ordered = [
            (p.min_demand, p.max_demand),
            (p.noise_freq_min, p.noise_freq_max),
            (p.disrupt_min, p.disrupt_max),
            (p.saturation_min, p.saturation_max),
        ];
        if ordered.iter().any(|(lo, hi)| lo > hi)
            || p.burst_interval_min > p.burst_interval_max
            || p.burst_size_min > p.burst_size_max
        {
            bad.push("each *_min must not exceed its *_max");
        }
        if !p.tier_bounds.windows(2).all(|w| w[0] < w[1]) {
            bad.push("tier_bounds must rise");
        }
        if p.tier_bounds.len() != TIERS.len() - 1 {
            bad.push("tier_bounds needs one bound between each two of the four tier labels (3)");
        }
        if bad.is_empty() {
            Ok(p)
        } else {
            Err(format!("config: modules.demand.params: {}", bad.join("; ")))
        }
    }
}

/// 1D gradient noise with quintic fade for one octave at position `x`
/// (fixed point throughout): about -1..=1.
fn gradient_noise(key: u64, octave: u64, x: Fx) -> Fx {
    let i = x.floor_int();
    let t = x - Fx::from_int(i);
    // Lattice gradients in -2..=2, which makes one octave span about -1..=1.
    let grad = |j: i64| {
        let span = Fx::from_int(4).steps() as u64 + 1;
        Fx::from_int(-2).plus_steps((lattice(key, octave, j) % span) as i64)
    };
    let v0 = grad(i) * t;
    let v1 = grad(i + 1) * (t - Fx::ONE);
    // Quintic fade: t^3 (t (6t - 15) + 10).
    let fade = t * t * t * (t * (t.mul_int(6) - Fx::from_int(15)) + Fx::from_int(10));
    v0 + fade * (v1 - v0)
}

/// The good's noise term on `day`: the octaves' weighted mean, times the amplitude.
pub fn noise(r: &Rules, m: &Market, day: u64) -> Fx {
    let p = &r.p.demand;
    let x = m.noise_freq.mul_int(day as i64);
    let mut sum = Fx::ZERO;
    let mut weight = Fx::ZERO;
    for (o, (&f, &w)) in p.noise_octave_freqs.iter().zip(&p.noise_octave_gains).enumerate() {
        sum += gradient_noise(m.noise_key, o as u64, x * f) * w;
        weight += w;
    }
    (sum / weight).max(-Fx::ONE).min(Fx::ONE) * p.noise_amplitude
}

/// Demand a pool of `n` speculators suppresses: `max × n / (n + half_point)`.
pub fn suppression(r: &Rules, n: Fx) -> Fx {
    let p = &r.p.demand;
    p.speculator_max_suppression * (n / (n + p.speculator_half_point))
}

/// The demand ceiling: `max_demand` times every `demand_ceiling` modifier.
pub fn ceiling(r: &Rules, g: &GameState) -> Fx {
    r.p.demand.max_demand * modifier(r, g, "demand_ceiling")
}

/// Sets every good's demand for the day from its forces.
fn compute(r: &Rules, g: &mut GameState) {
    let p = &r.p.demand;
    let ceil = ceiling(r, g);
    let day = g.run.day;
    let promotes: Vec<(String, Fx)> = g.run.markets.iter().map(|(k, m)| (k.clone(), m.promote)).collect();
    let goods: Vec<String> = g.run.markets.keys().cloned().collect();
    for good in goods {
        let coupling: Fx = promotes.iter().filter(|(k, _)| *k != good).map(|(_, v)| *v).sum();
        let m = &g.run.markets[&good];
        let raw = p.base_demand + noise(r, m, day) - suppression(r, m.speculators) - m.rival_pressure
            - m.saturation
            + m.promote
            + coupling * p.coupling_fraction;
        let d = raw.max(p.min_demand).min(ceil.max(p.min_demand));
        g.run.markets.get_mut(&good).expect("listed").demand = d;
    }
}

/// A new run's markets: fresh noise frequencies and keys, empty pools, the
/// first burst and rival dumps scheduled, and day 0's demand.
pub fn init(r: &Rules, g: &mut GameState) {
    let p = &r.p.demand;
    g.run.markets.clear();
    for good in super::shipments::goods(r) {
        let noise_freq = g.stream(NOISE).fx_in(p.noise_freq_min, p.noise_freq_max);
        let noise_key = g.stream(NOISE).below(u32::MAX) as u64;
        g.run.markets.insert(
            good,
            Market {
                demand: Fx::ZERO,
                noise_freq,
                noise_key,
                speculators: Fx::ZERO,
                speculators_seen: false,
                rival_pressure: Fx::ZERO,
                saturation: Fx::ZERO,
                promote: Fx::ZERO,
                revenue: Fx::ZERO,
            },
        );
    }
    g.run.bursts = 0;
    g.run.next_burst_day = g.run.day + burst_interval(r, g);
    g.run.rival_next.clear();
    for rv in &content().rivals {
        let next = g.run.day + g.stream(RIVALS).int_in(rv.interval_min, rv.interval_max) as u64;
        g.run.rival_next.insert(rv.id.clone(), next);
    }
    compute(r, g);
}

/// Days to the next burst: uniform in the interval, times `speculator_burst_interval_mult`.
fn burst_interval(r: &Rules, g: &mut GameState) -> u64 {
    let p = &r.p.demand;
    let days = g.stream(SPECULATORS).int_in(p.burst_interval_min, p.burst_interval_max);
    let m = modifier(r, g, "speculator_burst_interval_mult");
    (Fx::from_int(days as i64) * m).floor_int().max(1) as u64
}

/// The daily demand update (before any building or program runs): pools decay (proportionally,
/// plus each Arbitrage Engine's flat amount, both times the Nationalist
/// bonus), promote, rival pressure and saturation fade, then demand is set.
pub fn update(r: &Rules, g: &mut GameState) {
    super::attention::decay_day(r, g);
    let p = &r.p.demand;
    let mut engines = 0i64;
    for b in &content().buildings {
        if b.effects.iter().any(|e| matches!(e, BuildingEffect::SpeculatorDecay)) {
            engines += g.working(&b.id) as i64;
        }
    }
    let flat = p.arbitrage_decay_per_engine.mul_int(engines);
    for m in g.run.markets.values_mut() {
        let decay = m.speculators * p.speculator_decay + flat;
        m.speculators = (m.speculators - decay).max(Fx::ZERO);
        m.promote = (m.promote - p.promote_decay).max(Fx::ZERO);
        m.rival_pressure = (m.rival_pressure - p.rival_decay).max(Fx::ZERO);
        m.saturation = (m.saturation - p.saturation_decay).max(Fx::ZERO);
    }
    compute(r, g);
}

/// After shipments: a speculator burst when due (its target drawn in
/// proportion to each good's revenue since the last burst; no revenue, no
/// burst), and each rival's dump when due.
pub fn bursts(r: &Rules, g: &mut GameState) {
    let p = &r.p.demand;
    if g.run.day >= g.run.next_burst_day {
        let total: Fx = g.run.markets.values().map(|m| m.revenue).sum();
        // Bursts land only while the speculators are present (attention.md),
        // and never at a Nationalist major rank (ideology.md).
        let landing = super::attention::present(r, g, super::attention::SPECULATORS)
            && !super::ideology::major(r, g, "nationalist");
        if total > Fx::ZERO && landing {
            let one = u32::try_from(Fx::ONE.steps()).expect("scale fits u32");
            let k = g.stream(SPECULATORS).below(one);
            let at = total.mul_ratio(k as i64, one as i64);
            let mut acc = Fx::ZERO;
            let mut target = None;
            for (good, m) in &g.run.markets {
                acc += m.revenue;
                if m.revenue > Fx::ZERO && at < acc {
                    target = Some(good.clone());
                    break;
                }
            }
            let target = target.expect("a positive total has a target");
            let size = g.stream(SPECULATORS).int_in(p.burst_size_min, p.burst_size_max);
            let growth = p.burst_growth.pow_int(g.run.bursts as i64).unwrap_or(Fx::MAX);
            let add = Fx::from_int(size as i64).checked_mul(growth).unwrap_or(Fx::MAX);
            let m = g.run.markets.get_mut(&target).expect("drawn from markets");
            m.speculators = m.speculators.saturating_add(add);
            m.speculators_seen |= m.speculators > Fx::ZERO;
            g.run.bursts += 1;
            g.activity("speculator_burst", &target, Some(add), None);
            for m in g.run.markets.values_mut() {
                m.revenue = Fx::ZERO;
            }
        }
        g.run.next_burst_day = g.run.day + burst_interval(r, g);
    }
    let rivals = super::attention::present(r, g, super::attention::RIVALS);
    for rv in &content().rivals {
        if g.run.rival_next.get(&rv.id).is_some_and(|&d| g.run.day >= d) {
            // A rival dumps only while the rivals are present (attention.md).
            if let Some(m) = g.run.markets.get_mut(&rv.good).filter(|_| rivals) {
                m.rival_pressure += rv.demand_hit;
                g.activity("rival_dump", &rv.id, None, None);
            }
            let next = g.run.day + g.stream(RIVALS).int_in(rv.interval_min, rv.interval_max) as u64;
            g.run.rival_next.insert(rv.id.clone(), next);
        }
    }
}

/// A launch of `good` paying `pay`: its saturation rises by a draw in the
/// saturation range; the revenue counts toward the next burst's target.
pub fn on_launch(r: &Rules, g: &mut GameState, good: &str, pay: Fx) {
    let p = &r.p.demand;
    let hit = g.stream(SATURATION).fx_in(p.saturation_min, p.saturation_max);
    if let Some(m) = g.run.markets.get_mut(good) {
        m.saturation += hit;
        m.revenue = m.revenue.saturating_add(pay);
    }
}

/// Promote: the good's promote effect rises; its speculator pool is damped.
pub fn promote(r: &Rules, g: &mut GameState, good: &str) {
    let p = &r.p.demand;
    let add = p.promote_effect * modifier(r, g, "promote_effectiveness_mult");
    if let Some(m) = g.run.markets.get_mut(good) {
        m.promote += add;
        m.speculators = m.speculators * p.promote_speculator_dampening;
    }
}

/// Disrupt Speculators: a draw in the disrupt range comes off the first good
/// in loading priority whose pool is above zero (none: nothing happens).
pub fn disrupt(r: &Rules, g: &mut GameState) {
    let p = &r.p.demand;
    let amount = g.stream(DISRUPT).fx_in(p.disrupt_min, p.disrupt_max);
    super::attention::on_disrupt(r, g);
    let order = g.run.loading_priority.clone();
    for good in order {
        if let Some(m) = g.run.markets.get_mut(&good) {
            if m.speculators > Fx::ZERO {
                m.speculators = (m.speculators - amount).max(Fx::ZERO);
                return;
            }
        }
    }
}

/// The demand labels shown before Market Awareness, lowest first; the config's
/// `tier_bounds` holds one bound between each two (checked at load).
pub const TIERS: &[&str] = &["LOW", "MEDIUM", "HIGH", "VERY HIGH"];

/// The label shown before Market Awareness: the index in [`TIERS`] of the first
/// tier bound above `demand`.
pub fn tier(r: &Rules, demand: Fx) -> usize {
    r.p.demand.tier_bounds.iter().filter(|&&b| demand >= b).count()
}
