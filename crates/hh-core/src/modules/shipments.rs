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

/// The tradeable goods, in content resource order.
pub fn goods(r: &Rules) -> Vec<String> {
    content()
        .resources
        .iter()
        .filter(|res| r.p.shipments.base_values.contains_key(&res.id))
        .map(|res| res.id.clone())
        .collect()
}

/// One pad per Launch Pad owned: new pads take the first good in loading
/// priority; selling drops the last pad (and its cargo).
pub fn sync_pads(g: &mut GameState) {
    let n = g.owned(LAUNCH_PAD) as usize;
    let good = g.run.loading_priority.first().cloned().unwrap_or_default();
    g.run.pads.truncate(n);
    while g.run.pads.len() < n {
        g.run.pads.push(Pad { good: good.clone(), paused: false, cargo: Fx::ZERO, cooldown: 0 });
    }
}

/// A pad runs if it is among the first `active` Launch Pads.
pub fn pad_active(g: &GameState, i: usize) -> bool {
    i < g.active(LAUNCH_PAD) as usize
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
/// to the higher-priority good's pads first.
pub fn load(r: &Rules, g: &mut GameState) {
    let per = load_amount(r, g);
    let cap = r.p.shipments.pad_cargo_capacity;
    let rank = |good: &str| g.run.loading_priority.iter().position(|x| x == good).unwrap_or(usize::MAX);
    let mut order: Vec<usize> = (0..g.run.pads.len()).collect();
    order.sort_by_key(|&i| (rank(&g.run.pads[i].good), i));
    for i in order {
        if !pad_active(g, i) || g.run.pads[i].paused {
            continue;
        }
        let good = g.run.pads[i].good.clone();
        let take = per.min(cap - g.run.pads[i].cargo).min(g.amount(&good)).max(Fx::ZERO);
        if take > Fx::ZERO {
            *g.run.resources.get_mut(&good).expect("content good") -= take;
            g.run.pads[i].cargo += take;
        }
    }
}

/// A pad can launch now: active, cooled down, cargo aboard and fuel in stock.
pub fn can_launch(r: &Rules, g: &GameState, i: usize) -> bool {
    let p = &g.run.pads[i];
    pad_active(g, i) && p.cooldown == 0 && p.cargo > Fx::ZERO && g.amount(PROPELLANT) >= r.p.shipments.fuel_per_pad
}

/// Launch Full Pads: every active, unpaused, full pad that can launch does, in pad order.
pub fn launch_full(r: &Rules, g: &mut GameState) {
    for i in 0..g.run.pads.len() {
        let p = &g.run.pads[i];
        if !p.paused && p.cargo >= r.p.shipments.pad_cargo_capacity && can_launch(r, g, i) {
            launch(r, g, i);
        }
    }
}

/// The credits a pad's cargo fetches now.
pub fn payout(r: &Rules, g: &GameState, i: usize) -> Fx {
    let p = &g.run.pads[i];
    let base = r.p.shipments.base_values.get(&p.good).copied().unwrap_or(Fx::ZERO);
    let demand = g.run.markets.get(&p.good).map_or(Fx::ZERO, |m| m.demand);
    base * demand * p.cargo * modifier(r, g, "shipment_credit_mult")
}

/// Launches pad `i` (checked by the caller): burns fuel, pays `base value ×
/// demand × cargo × shipment_credit_mult`, saturates the good's demand and
/// starts the cooldown.
pub fn launch(r: &Rules, g: &mut GameState, i: usize) {
    let fuel = r.p.shipments.fuel_per_pad;
    g.consume(PROPELLANT, fuel);
    let pay = payout(r, g, i);
    let good = g.run.pads[i].good.clone();
    let demand = g.run.markets.get(&good).map_or(Fx::ZERO, |m| m.demand);
    g.produce(CREDITS, pay);
    g.run.stats.shipments += 1;
    let cargo = g.run.pads[i].cargo;
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

/// Sets the good a pad carries. A loaded pad switching goods returns its cargo
/// to the stockpile; what exceeds the stockpile's cap is lost, and a
/// `cargo_lost` activity line says so.
pub fn set_pad_good(r: &Rules, g: &mut GameState, pad: u32, good: &str) -> Result<(), String> {
    if !r.p.shipments.base_values.contains_key(good) {
        return Err(format!("pad {pad}: {good:?} is not a tradeable good"));
    }
    let p = g.run.pads.get_mut(pad as usize).ok_or_else(|| format!("pad {pad}: no such pad"))?;
    if p.good == good {
        return Ok(());
    }
    let (old, cargo) = (p.good.clone(), p.cargo);
    let back = switch_returns(r, g, pad as usize);
    let p = &mut g.run.pads[pad as usize];
    p.good = good.to_string();
    p.cargo = Fx::ZERO;
    g.add(&old, back);
    if cargo > back {
        g.activity("cargo_lost", &old, Some(Fx::from_int(i64::from(pad) + 1)), Some(cargo - back));
    }
    Ok(())
}

/// How much of pad `i`'s cargo a switch to another good returns to the
/// stockpile: all of it, up to the room under the stockpile's cap.
pub fn switch_returns(r: &Rules, g: &GameState, i: usize) -> Fx {
    let p = &g.run.pads[i];
    let room = super::economy::cap(r, g, &p.good).map_or(p.cargo, |c| (c - g.amount(&p.good)).max(Fx::ZERO));
    p.cargo.min(room)
}

pub fn set_pad_paused(g: &mut GameState, pad: u32, paused: bool) -> Result<(), String> {
    let p = g.run.pads.get_mut(pad as usize).ok_or_else(|| format!("pad {pad}: no such pad"))?;
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
