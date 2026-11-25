//! **shipments**: launch pads, loading, launching, payouts and cooldowns
//! (docs/rules/shipments.md).

use std::collections::BTreeMap;

use super::{param, ModuleDef, ParamKind as K, VariantDef};
use crate::config::ModuleConfig;
use crate::content::{content, ResearchEffect};
use crate::fx::Fx;
use crate::modifiers::modifier;
use crate::rules::Rules;
use crate::state::{GameState, Pad};

use super::economy::CREDITS;

pub const LAUNCH_PAD: &str = "launch_pad";
pub const PROPELLANT: &str = "propellant";

pub static DEF: ModuleDef = ModuleDef {
    id: "shipments",
    variants: &[VariantDef {
        name: "arc1",
        params: &[
            param("base_values", K::FxMap, Some(("0", "1000000")), "the tradeable goods and each one's credits per unit at demand 1"),
            param("pad_cargo_capacity", K::Fx, Some(("1", "1000000")), "cargo a pad holds when full"),
            param("fuel_per_pad", K::Fx, Some(("0", "1000000")), "propellant burned per launch"),
            param("load_per_execution", K::Fx, Some(("0", "1000000")), "units Load Launch Pads puts on each pad, before research"),
            param("launch_cooldown", K::Int, Some(("0", "100000")), "days after a launch before the pad can launch again"),
        ],
        rng: &[],
    }],
};

#[derive(Clone)]
pub struct Params {
    pub base_values: BTreeMap<String, Fx>,
    pub pad_cargo_capacity: Fx,
    pub fuel_per_pad: Fx,
    pub load_per_execution: Fx,
    pub launch_cooldown: u32,
}

impl Params {
    pub fn new(mc: &ModuleConfig) -> Result<Params, String> {
        let base_values = mc.fx_map("base_values");
        if let Some(g) = base_values.keys().find(|g| content().resource(g).is_none()) {
            return Err(format!("config: modules.shipments.params.base_values.{g}: unknown resource"));
        }
        if base_values.is_empty() {
            return Err("config: modules.shipments.params.base_values: no tradeable goods".into());
        }
        Ok(Params {
            base_values,
            pad_cargo_capacity: mc.fx("pad_cargo_capacity"),
            fuel_per_pad: mc.fx("fuel_per_pad"),
            load_per_execution: mc.fx("load_per_execution"),
            launch_cooldown: mc.int("launch_cooldown") as u32,
        })
    }
}

/// A pad may be set to `good` now: the active quest's `pad_goods`, if it
/// names any, hold every pad to them (docs/rules/shipments.md).
pub fn allowed(g: &GameState, good: &str) -> bool {
    super::events::active_quest(g)
        .and_then(|q| q.pad_goods.as_ref())
        .is_none_or(|goods| goods.iter().any(|x| x == good))
}

/// The tradeable goods, in content resource order.
pub fn goods(r: &Rules) -> Vec<String> {
    content()
        .resources
        .iter()
        .filter(|res| r.p.shipments.base_values.contains_key(&res.id))
        .map(|res| res.id.clone())
        .collect()
}

/// One pad per Launch Pad owned: new pads start with No cargo (the player
/// picks the good); selling drops the last pad (and its cargo).
pub fn sync_pads(g: &mut GameState) {
    let n = g.owned(LAUNCH_PAD) as usize;
    g.run.pads.truncate(n);
    while g.run.pads.len() < n {
        g.run.pads.push(Pad { good: None, paused: false, cargo: Fx::ZERO, cooldown: 0 });
    }
}

/// The good pad `i` carries, `None` for No cargo.
pub fn pad_good(g: &GameState, i: usize) -> Option<&str> {
    g.run.pads[i].good.as_deref()
}

/// A pad runs if it is among the first working Launch Pads (active, upkeep paid).
pub fn pad_active(g: &GameState, i: usize) -> bool {
    i < g.working(LAUNCH_PAD) as usize
}

/// Units one Load puts on a pad: the research value if bought, else the
/// config's, times `cargo_capacity_mult`.
pub fn load_amount(r: &Rules, g: &GameState) -> Fx {
    let mut per = r.p.shipments.load_per_execution;
    for effect in super::research::live_effects(g) {
        if let ResearchEffect::LoadPerExecution { value } = effect {
            per = *value;
        }
    }
    per * modifier(r, g, "cargo_capacity_mult")
}

/// Load Launch Pads: each active, unpaused pad takes up to the load amount
/// from its good's stockpile, to its capacity; pads load in loading-priority
/// order of their goods (pad order within a good), so a short stockpile goes
/// to the higher-priority good's pads first. A No-cargo pad loads nothing.
/// With `only`, just that good's pads (`load pads <good>`).
pub fn load(r: &Rules, g: &mut GameState, only: Option<&str>) {
    let per = load_amount(r, g);
    let cap = r.p.shipments.pad_cargo_capacity;
    let rank = |good: Option<&str>| g.run.loading_priority.iter().position(|x| Some(x.as_str()) == good).unwrap_or(usize::MAX);
    let mut order: Vec<usize> = (0..g.run.pads.len()).filter(|&i| on(g, i, only)).collect();
    order.sort_by_key(|&i| (rank(pad_good(g, i)), i));
    for i in order {
        let take = load_take(g, i, per, cap);
        if take > Fx::ZERO {
            let good = pad_good(g, i).expect("only a pad with a good loads").to_string();
            *g.run.resources.get_mut(&good).expect("content good") -= take;
            g.track("pads", &good, Fx::ZERO, take);
            g.run.pads[i].cargo += take;
        }
    }
}

/// What Load puts aboard pad `i` now (0 when No cargo, paused, inactive, full
/// or out of the good).
fn load_take(g: &GameState, i: usize, per: Fx, cap: Fx) -> Fx {
    let Some(good) = pad_good(g, i) else { return Fx::ZERO };
    if !pad_active(g, i) || g.run.pads[i].paused {
        return Fx::ZERO;
    }
    per.min(cap - g.run.pads[i].cargo).min(g.amount(good)).max(Fx::ZERO)
}

/// Whether Load would put anything aboard `only`'s pads now (a `load pads`
/// line with nothing to load is done).
pub fn would_load(r: &Rules, g: &GameState, only: Option<&str>) -> bool {
    let (per, cap) = (load_amount(r, g), r.p.shipments.pad_cargo_capacity);
    (0..g.run.pads.len()).any(|i| on(g, i, only) && load_take(g, i, per, cap) > Fx::ZERO)
}

/// Whether Launch Full Pads would launch any of `only`'s pads now (a `launch
/// pads` line with none is done).
pub fn would_launch(r: &Rules, g: &GameState, only: Option<&str>) -> bool {
    (0..g.run.pads.len()).any(|i| launches(r, g, i, only))
}

/// Pad `i` launches under Launch Full Pads for `only`: its good, unpaused,
/// ready.
pub fn launches(r: &Rules, g: &GameState, i: usize, only: Option<&str>) -> bool {
    on(g, i, only) && !g.run.pads[i].paused && ready(r, g, i)
}

/// A pad can launch now: active, cooled down, cargo aboard and fuel in stock
/// (only a pad with a good holds cargo).
pub fn can_launch(r: &Rules, g: &GameState, i: usize) -> bool {
    let p = &g.run.pads[i];
    pad_active(g, i) && p.good.is_some() && p.cooldown == 0 && p.cargo > Fx::ZERO && g.amount(PROPELLANT) >= r.p.shipments.fuel_per_pad
}

/// A pad is ready: full and able to launch now. The one rule behind Launch Full
/// Pads (unpaused pads: [`launches`]) and the view's Ready badge.
pub fn ready(r: &Rules, g: &GameState, i: usize) -> bool {
    g.run.pads[i].cargo >= r.p.shipments.pad_cargo_capacity && can_launch(r, g, i)
}

/// Pad `i` carries `only` (`None`: every pad, No cargo included).
pub fn on(g: &GameState, i: usize, only: Option<&str>) -> bool {
    only.is_none_or(|good| pad_good(g, i) == Some(good))
}

/// Launch Full Pads: every active, unpaused, full pad that can launch does, in
/// pad order. With `only`, just that good's pads (`launch pads <good>`).
pub fn launch_full(r: &Rules, g: &mut GameState, only: Option<&str>) {
    for i in 0..g.run.pads.len() {
        if launches(r, g, i, only) {
            launch(r, g, i);
        }
    }
}

/// Pad `i` is idle: no cargo aboard (a No-cargo pad always is). `set pads`
/// switches only idle pads (of another good or No cargo), and the `idle pads`
/// reading counts them.
pub fn idle(g: &GameState, i: usize) -> bool {
    g.run.pads[i].cargo == Fx::ZERO
}

/// `set pads <good> up to N`: while fewer than `n` pads carry `good`, an idle
/// pad carrying another good or No cargo switches to it, in pad order. Returns
/// how many switched.
pub fn point_idle(r: &Rules, g: &mut GameState, good: &str, n: u32) -> u32 {
    let mut have = (0..g.run.pads.len()).filter(|&i| on(g, i, Some(good))).count() as u32;
    let mut moved = 0;
    for i in 0..g.run.pads.len() {
        if have >= n {
            break;
        }
        if !on(g, i, Some(good)) && idle(g, i) && set_pad_good(r, g, i as u32, Some(good)).is_ok() {
            have += 1;
            moved += 1;
        }
    }
    moved
}

/// The credits a pad's cargo fetches now.
pub fn payout(r: &Rules, g: &GameState, i: usize) -> Fx {
    let Some(good) = pad_good(g, i) else { return Fx::ZERO };
    let base = r.p.shipments.base_values.get(good).copied().unwrap_or(Fx::ZERO);
    let demand = g.run.markets.get(good).map_or(Fx::ZERO, |m| m.demand);
    base * demand * g.run.pads[i].cargo * modifier(r, g, "shipment_credit_mult")
}

/// Launches pad `i` (checked by the caller): burns fuel, pays `base value ×
/// demand × cargo × shipment_credit_mult`, saturates the good's demand and
/// starts the cooldown.
pub fn launch(r: &Rules, g: &mut GameState, i: usize) {
    let fuel = r.p.shipments.fuel_per_pad;
    g.consume("launch", PROPELLANT, fuel);
    let pay = payout(r, g, i);
    let good = pad_good(g, i).expect("a pad with cargo has a good").to_string();
    let demand = g.run.markets.get(&good).map_or(Fx::ZERO, |m| m.demand);
    g.produce("launch", CREDITS, pay);
    g.run.stats.shipments += 1;
    let cargo = g.run.pads[i].cargo;
    g.run.stats.best_launch = g.run.stats.best_launch.max(pay);
    g.run.stats.trade_credits = g.run.stats.trade_credits.saturating_add(pay);
    let best = g.run.stats.best_cargo.entry(good.clone()).or_insert(Fx::ZERO);
    *best = (*best).max(cargo);
    super::attention::on_sale(r, g, cargo);
    g.activity("shipment", &good, Some(pay), Some(cargo));
    let p = &mut g.run.pads[i];
    p.cargo = Fx::ZERO;
    p.cooldown = r.p.shipments.launch_cooldown;
    super::demand::on_launch(r, g, &good, pay);
    super::achievements::on_shipment(g, pay, demand);
}

/// A player's manual launch: works on a paused pad, needs only some cargo.
pub fn launch_action(r: &Rules, g: &mut GameState, pad: u32) -> Result<(), String> {
    let i = pad as usize;
    if i >= g.run.pads.len() {
        return Err(format!("launch: no pad {}", pad + 1));
    }
    if !can_launch(r, g, i) {
        return Err(format!("launch: pad {} can't launch (inactive, cooling down, empty or no fuel)", pad + 1));
    }
    launch(r, g, i);
    Ok(())
}

/// Sets the good a pad carries (`None`: No cargo). A loaded pad switching
/// goods returns its cargo to the stockpile; what exceeds the stockpile's cap
/// is lost, and a `cargo_lost` activity line says so.
pub fn set_pad_good(r: &Rules, g: &mut GameState, pad: u32, good: Option<&str>) -> Result<(), String> {
    if let Some(good) = good.filter(|x| !r.p.shipments.base_values.contains_key(*x)) {
        return Err(format!("pad {}: {good:?} is not a tradeable good", pad + 1));
    }
    if let Some(good) = good.filter(|x| !allowed(g, x)) {
        return Err(format!("pad {}: {good:?} can't be picked during this quest", pad + 1));
    }
    let p = g.run.pads.get_mut(pad as usize).ok_or_else(|| format!("pad {}: no such pad", pad + 1))?;
    if p.good.as_deref() == good {
        return Ok(());
    }
    let (old, cargo) = (p.good.clone(), p.cargo);
    let back = switch_returns(r, g, pad as usize);
    let p = &mut g.run.pads[pad as usize];
    p.good = good.map(str::to_string);
    p.cargo = Fx::ZERO;
    // A No-cargo pad holds no cargo: nothing to return.
    if let Some(old) = old {
        g.add(&old, back);
        g.track("pads", &old, back, Fx::ZERO);
        if cargo > back {
            g.activity("cargo_lost", &old, Some(Fx::from_int(i64::from(pad) + 1)), Some(cargo - back));
        }
    }
    Ok(())
}

/// How much of pad `i`'s cargo a switch to another good returns to the
/// stockpile: all of it, up to the room under the stockpile's cap.
pub fn switch_returns(r: &Rules, g: &GameState, i: usize) -> Fx {
    let p = &g.run.pads[i];
    let Some(good) = p.good.as_deref() else { return Fx::ZERO };
    let room = super::economy::cap(r, g, good).map_or(p.cargo, |c| (c - g.amount(good)).max(Fx::ZERO));
    p.cargo.min(room)
}

pub fn set_pad_paused(g: &mut GameState, pad: u32, paused: bool) -> Result<(), String> {
    let p = g.run.pads.get_mut(pad as usize).ok_or_else(|| format!("pad {}: no such pad", pad + 1))?;
    p.paused = paused;
    Ok(())
}

/// Sets the loading priority: every tradeable good once.
pub fn set_loading_priority(r: &Rules, g: &mut GameState, order: &[String]) -> Result<(), String> {
    let mut sorted = order.to_vec();
    sorted.sort();
    let mut want = goods(r);
    want.sort();
    if sorted != want {
        return Err(format!("loading priority: list each tradeable good once ({})", want.join(", ")));
    }
    g.run.loading_priority = order.to_vec();
    Ok(())
}

/// Each day, pads cool down.
pub fn tick(g: &mut GameState) {
    for p in &mut g.run.pads {
        p.cooldown = p.cooldown.saturating_sub(1);
    }
}
