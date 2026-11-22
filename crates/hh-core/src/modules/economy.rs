//! **economy**: resources, storage caps, buildings (buy, sell, enable, the
//! multi-pass daily resolution with partial production and stall tracking), land
//! and the end-of-tick clamp (docs/rules/economy.md).

use std::collections::BTreeMap;

use super::{param, ModuleDef, ParamKind as K, VariantDef};
use crate::config::ModuleConfig;
use crate::content::{content, Amounts, Building, BuildingEffect, Requirement};
use crate::fx::Fx;
use crate::modifiers::modifier;
use crate::rules::Rules;
use crate::state::{Flow, GameState, Holding, Short, Stall};

pub const ENERGY: &str = "energy";
pub const CREDITS: &str = "credits";
pub const LAND: &str = "land";
pub const PROCESSORS: &str = "processors";
pub const BOREDOM: &str = "boredom";
pub const SCIENCE: &str = "science";

pub static DEF: ModuleDef = ModuleDef {
    id: "economy",
    variants: &[VariantDef {
        name: "arc1",
        params: &[
            param("starting_resources", K::FxMap, Some(("0", "1000000000")), "resources at the start of every run, by id"),
            param("starting_buildings", K::IntMap, Some(("0", "1000")), "buildings owned (and active) at the start of every run"),
            param("land_base_cost", K::Fx, Some(("0", "1000000000")), "credits for the first land purchase"),
            param("land_cost_scaling", K::Fx, Some(("1", "10")), "each land purchase multiplies the next one's price by this"),
            param("land_per_purchase", K::Fx, Some(("0", "1000000")), "land gained per purchase"),
            param("upkeep_epsilon", K::Fx, Some(("0", "1")), "a building pays full upkeep if short by no more than this"),
            param("flow_average_days", K::Int, Some(("1", "1000")), "days in the resource breakdown's rolling average (display only)"),
        ],
        rng: &[],
    }],
};

#[derive(Clone)]
pub struct Params {
    pub starting_resources: BTreeMap<String, Fx>,
    pub starting_buildings: BTreeMap<String, u32>,
    pub land_base_cost: Fx,
    pub land_cost_scaling: Fx,
    pub land_per_purchase: Fx,
    pub upkeep_epsilon: Fx,
    pub flow_average_days: i64,
}

impl Params {
    pub fn new(mc: &ModuleConfig) -> Result<Params, String> {
        let c = content();
        let starting_resources = mc.fx_map("starting_resources");
        let starting_buildings: BTreeMap<String, u32> = mc
            .int_map("starting_buildings")
            .into_iter()
            .map(|(k, v)| (k, u32::try_from(v).expect("range-checked")))
            .collect();
        let mut bad: Vec<String> = starting_resources
            .keys()
            .filter(|r| c.resource(r).is_none())
            .map(|r| format!("modules.economy.params.starting_resources.{r}: unknown resource"))
            .collect();
        bad.extend(
            starting_buildings
                .keys()
                .filter(|b| c.building(b).is_none())
                .map(|b| format!("modules.economy.params.starting_buildings.{b}: unknown building")),
        );
        if !bad.is_empty() {
            return Err(format!("config: {}", bad.join("; ")));
        }
        Ok(Params {
            starting_resources,
            starting_buildings,
            land_base_cost: mc.fx("land_base_cost"),
            land_cost_scaling: mc.fx("land_cost_scaling"),
            land_per_purchase: mc.fx("land_per_purchase"),
            upkeep_epsilon: mc.fx("upkeep_epsilon"),
            flow_average_days: mc.int("flow_average_days"),
        })
    }
}

/// Whether every product is at or over its cap (nothing to make room for); never for
/// no products. A building so is "output at cap"; a command so is skipped.
pub fn all_capped(r: &Rules, g: &GameState, products: &Amounts) -> bool {
    !products.is_empty() && products.keys().all(|res| cap(r, g, res).is_some_and(|c| g.amount(res) >= c))
}

/// The fraction of `needs` the stock covers, at most 1: the scarcest input's (a
/// partial run of a building or a conversion command).
pub fn input_fraction(g: &GameState, needs: &Amounts) -> Fx {
    let mut frac = Fx::ONE;
    for (res, &need) in needs {
        if need > Fx::ZERO {
            frac = frac.min(g.amount(res).max(Fx::ZERO) / need);
        }
    }
    frac
}

/// Notes the demand a run of `needs` at `paid` (0 to 1) leaves unmet, for the
/// `rate` reading: on each input held short of its full need, the need less
/// what is paid. An input held in full but left unused (another input ran
/// short) is no shortage of it and counts nothing. Call before paying.
pub fn note_unmet(g: &mut GameState, needs: &Amounts, paid: Fx) {
    for (res, &need) in needs {
        if need > Fx::ZERO && g.amount(res) < need {
            let left = need - need * paid;
            if left > Fx::ZERO {
                let u = g.run.today.unmet.entry(res.clone()).or_insert(Fx::ZERO);
                *u = u.saturating_add(left);
            }
        }
    }
}

/// The input that limits a run of `needs` to `ran` (its first scarcest, in id
/// order), for the breakdown's shortfalls.
pub fn scarcest(g: &GameState, needs: &Amounts, ran: Fx) -> Option<Short> {
    let mut best: Option<(Fx, Short)> = None;
    for (res, &need) in needs {
        if need > Fx::ZERO {
            let had = g.amount(res).max(Fx::ZERO);
            let f = had / need;
            if f < Fx::ONE && best.as_ref().is_none_or(|(b, _)| f < *b) {
                best = Some((f, Short { resource: res.clone(), need, had, ran }));
            }
        }
    }
    best.map(|(_, s)| s)
}

/// `resource`'s storage cap now: its base plus every working (active, upkeep-paid)
/// building's `store` effect, times `storage_cap_mult` for everything but energy.
/// Boredom's is the boredom module's. `None` is uncapped.
pub fn cap(r: &Rules, g: &GameState, resource: &str) -> Option<Fx> {
    if resource == BOREDOM {
        return Some(super::boredom::cap(r, g));
    }
    let base = content().resource(resource)?.cap?;
    let cap = base + stored(g, resource);
    Some(if resource == ENERGY { cap } else { cap * modifier(r, g, "storage_cap_mult") })
}

/// The sum of working buildings' `store` effects on `resource`.
pub fn stored(g: &GameState, resource: &str) -> Fx {
    let mut total = Fx::ZERO;
    for b in &content().buildings {
        let n = g.working(&b.id);
        for e in &b.effects {
            if let BuildingEffect::Store { resource: res, amount } = e {
                if res == resource && n > 0 {
                    total += amount.mul_int(n as i64);
                }
            }
        }
    }
    total
}

/// Whether a content requirement holds now.
pub fn requirement_met(g: &GameState, req: &Requirement) -> bool {
    match req {
        Requirement::None => true,
        Requirement::Building { building, count } => g.owned(building) >= *count,
        Requirement::Quest { quest } => super::events::quest_open_or_done(g, quest),
        Requirement::Research { research } => g.run.research.contains(research),
        Requirement::Flag { flag } => g.career.flags.contains(flag),
        Requirement::Project { project } => super::projects::completed(g, project),
    }
}

/// Buyable now, prices aside: its requirement holds and, if an event gates it,
/// the event enabled it this run.
pub fn unlocked(g: &GameState, b: &Building) -> bool {
    requirement_met(g, &b.requires)
        && (!content().event_gated(&b.id) || g.run.enabled_buildings.contains(&b.id))
}

/// Shown to the player: unlocked, or (for a building-count requirement only)
/// owned in any run.
pub fn visible(g: &GameState, b: &Building) -> bool {
    unlocked(g, b)
        || (matches!(b.requires, Requirement::Building { .. })
            && !content().event_gated(&b.id)
            && g.career.owned_buildings.contains(&b.id))
}

/// The next one's price: `cost × cost_scaling ^ (owned − bonus) × discount ^
/// rank` (its ideology axis), each power exact (`Fx::pow_int`). A price past
/// `Fx`'s range is `Fx::MAX`: shown, never affordable.
pub fn price(r: &Rules, g: &GameState, b: &Building) -> BTreeMap<String, Fx> {
    let h = g.run.buildings.get(&b.id).copied().unwrap_or_default();
    let bought = h.owned.saturating_sub(h.bonus) as i64;
    let mut factor = b.cost_scaling.pow_int(bought);
    if let Some(axis) = &b.ideology {
        let d = super::ideology::discount(r, g, axis);
        factor = factor.and_then(|f| f.checked_mul(d));
    }
    b.cost
        .iter()
        .map(|(res, &base)| (res.clone(), factor.and_then(|f| base.checked_mul(f)).unwrap_or(Fx::MAX)))
        .collect()
}

pub fn affordable(g: &GameState, cost: &BTreeMap<String, Fx>) -> bool {
    cost.iter().all(|(r, &c)| c < Fx::MAX && g.amount(r) >= c)
}

/// `b` has a limit and owns that many.
pub fn at_limit(g: &GameState, b: &Building) -> bool {
    b.max_count.is_some_and(|m| g.owned(&b.id) >= m)
}

/// Why `b` can't be bought now, if it can't.
pub fn buy_blocker(r: &Rules, g: &GameState, b: &Building) -> Option<String> {
    if !unlocked(g, b) {
        return Some(format!("buy {}: locked", b.name));
    }
    if at_limit(g, b) {
        return Some(format!("buy {}: at its limit", b.name));
    }
    if g.amount(LAND) < Fx::from_int(b.land as i64) {
        return Some(format!("buy {}: not enough land", b.name));
    }
    if !affordable(g, &price(r, g, b)) {
        return Some(format!("buy {}: can't afford it", b.name));
    }
    None
}

pub fn buy(r: &Rules, g: &mut GameState, id: &str) -> Result<(), String> {
    let b = content().building(id).ok_or_else(|| format!("buy: unknown building {id:?}"))?;
    if let Some(e) = buy_blocker(r, g, b) {
        return Err(e);
    }
    for (res, c) in price(r, g, b) {
        *g.run.resources.get_mut(&res).expect("content checked") -= c;
        g.track("buy", &res, Fx::ZERO, c);
    }
    *g.run.resources.get_mut(LAND).expect("content has land") -= Fx::from_int(b.land as i64);
    g.track("buy", LAND, Fx::ZERO, Fx::from_int(b.land as i64));
    let h = g.run.buildings.entry(b.id.clone()).or_default();
    h.owned += 1;
    h.active += 1;
    g.run.stats.buildings_built += 1;
    g.run.last_purchase = Some(g.run.day);
    g.career.owned_buildings.insert(b.id.clone());
    refresh(g);
    Ok(())
}

/// Sells one: no credits back. A copy that took land (bought) goes first and
/// refunds its land; once only free copies (starting, bonus) remain, one goes
/// with no refund. An idle copy goes before an active one.
pub fn sell(g: &mut GameState, id: &str) -> Result<(), String> {
    let b = content().building(id).ok_or_else(|| format!("sell: unknown building {id:?}"))?;
    let h = g.run.buildings.get_mut(&b.id).filter(|h| h.owned > 0);
    let Some(h) = h else {
        return Err(format!("sell {}: none owned", b.name));
    };
    let paid = h.owned > h.free;
    h.owned -= 1;
    h.free = h.free.min(h.owned);
    let was = h.active;
    h.active = h.active.min(h.owned);
    // A copy taken out of service is an unpaid one first: the paid ones keep working.
    h.unpaid = h.unpaid.saturating_sub(was - h.active);
    h.bonus = h.bonus.min(h.owned);
    if h.owned == 0 {
        g.run.buildings.remove(&b.id);
    }
    if paid {
        g.add(LAND, Fx::from_int(b.land as i64));
        g.track("sell", LAND, Fx::from_int(b.land as i64), Fx::ZERO);
    }
    refresh(g);
    Ok(())
}

/// Sets how many of a type run (the rest stay owned, idle, still on their land).
pub fn set_active(g: &mut GameState, id: &str, active: u32) -> Result<(), String> {
    let b = content().building(id).ok_or_else(|| format!("set active: unknown building {id:?}"))?;
    let owned = g.owned(&b.id);
    if active > owned {
        return Err(format!("set active {}: only {owned} owned", b.name));
    }
    if let Some(h) = g.run.buildings.get_mut(&b.id) {
        // Copies switched off are unpaid ones first: the paid ones keep working.
        h.unpaid = h.unpaid.saturating_sub(h.active.saturating_sub(active));
        h.active = active;
    }
    refresh(g);
    Ok(())
}

/// Grants free (bonus) buildings at run start: owned, active and bonus all rise.
pub fn grant_bonus(g: &mut GameState, id: &str, n: u32) {
    let h = g.run.buildings.entry(id.to_string()).or_default();
    h.owned += n;
    h.active += n;
    h.bonus += n;
    h.free += n;
}

/// The next land purchase's price in credits.
pub fn land_price(r: &Rules, g: &GameState) -> Fx {
    let p = &r.p.economy;
    let factor = p
        .land_cost_scaling
        .pow_int(g.run.land_bought as i64)
        .and_then(|f| f.checked_mul(modifier(r, g, "land_cost_mult")))
        .and_then(|f| f.checked_mul(super::ideology::discount(r, g, "nationalist")))
        .and_then(|f| f.checked_mul(super::attention::land_price_mult(r, g)));
    factor.and_then(|f| p.land_base_cost.checked_mul(f)).unwrap_or(Fx::MAX)
}

/// Why Buy Land wouldn't apply now, if it wouldn't (the view's `can_buy_land`,
/// without playing the action on a copy of the state).
pub fn land_blocker(r: &Rules, g: &GameState) -> Option<String> {
    let price = land_price(r, g);
    (price == Fx::MAX || g.amount(CREDITS) < price).then(|| "buy land: can't afford it".into())
}

pub fn buy_land(r: &Rules, g: &mut GameState) -> Result<(), String> {
    if let Some(e) = land_blocker(r, g) {
        return Err(e);
    }
    let price = land_price(r, g);
    *g.run.resources.get_mut(CREDITS).expect("content has credits") -= price;
    g.add(LAND, r.p.economy.land_per_purchase);
    g.track("land", CREDITS, Fx::ZERO, price);
    g.track("land", LAND, r.p.economy.land_per_purchase, Fx::ZERO);
    g.run.land_bought += 1;
    g.run.last_purchase = Some(g.run.day);
    super::attention::on_land(r, g, r.p.economy.land_per_purchase);
    Ok(())
}

/// After any change to buildings (or to how many the day's pass paid):
/// derived resources (processors) from working buildings' `grant` effects, launch pads matched to Launch Pads owned, and processor
/// assignments trimmed to the processors there are (last program first).
/// The sum of every building's `grant` of `resource`, times `count(building)`
/// copies: working copies for the amount held, active ones for processor
/// assignment limits.
pub fn grants(resource: &str, count: impl Fn(&str) -> u32) -> Fx {
    let mut total = Fx::ZERO;
    for b in &content().buildings {
        for e in &b.effects {
            if let BuildingEffect::Grant { resource: r, amount } = e {
                if r == resource {
                    total += amount.mul_int(i64::from(count(&b.id)));
                }
            }
        }
    }
    total
}

pub fn refresh(g: &mut GameState) {
    let mut granted: BTreeMap<String, Fx> = BTreeMap::new();
    for b in &content().buildings {
        for e in &b.effects {
            if let BuildingEffect::Grant { resource, .. } = e {
                granted.entry(resource.clone()).or_insert(Fx::ZERO);
            }
        }
    }
    for (res, v) in &mut granted {
        *v = grants(res, |id| g.working(id));
    }
    for (res, v) in granted {
        g.run.resources.insert(res, v);
    }
    super::shipments::sync_pads(g);
    super::programs::trim_processors(g);
}

/// The multiplier on `b`'s production now: its output modifiers times its
/// category's overclock (science's Rationalist bonus is applied in [`resolve`]).
fn output_factor(r: &Rules, g: &GameState, b: &Building) -> Fx {
    let mut f = super::programs::overclock_factor(r, g, &b.category);
    for k in &b.output_modifiers {
        f = f * modifier(r, g, k);
    }
    f
}

/// Buildings that take no upkeep produce (the day's first stage, before
/// programs pay); returns the energy they made, for [`resolve`].
pub fn produce_free(r: &Rules, g: &mut GameState) -> Fx {
    resolve_stage(r, g, true)
}

/// The day's building production after programs, in content order
/// (docs/rules/economy.md, *Daily resolution*): of the buildings that take
/// upkeep, passes in which every one that can pay its full upkeep (within
/// `upkeep_epsilon`) pays and produces, repeated while any succeeds; then one
/// pass where the rest run at the fraction their scarcest input allows
/// (`input_starved`). `free_energy`: what [`produce_free`] made today.
pub fn resolve(r: &Rules, g: &mut GameState, free_energy: Fx) {
    // Who goes unpaid is decided afresh each day: the partial pass sets it below.
    let before: Vec<u32> = g.run.buildings.values().map(|h| h.unpaid).collect();
    for h in g.run.buildings.values_mut() {
        h.unpaid = 0;
    }
    let energy_made = free_energy + resolve_stage(r, g, false);
    // Effects follow the copies paid for (processors, pads, caps, decay): from
    // the next day on, as the effects are read after this pass.
    if g.run.buildings.values().map(|h| h.unpaid).ne(before) {
        refresh(g);
    }
    let stats = &mut g.run.stats;
    stats.peak_energy = stats.peak_energy.max(energy_made);
}

/// One stage of the building pass: the upkeep-free buildings (`free`) or the
/// rest. Returns the energy made.
fn resolve_stage(r: &Rules, g: &mut GameState, free: bool) -> Fx {
    let eps = r.p.economy.upkeep_epsilon;
    let upkeep_mult = modifier(r, g, "building_upkeep_mult");
    let science = super::ideology::bonus(r, g, "rationalist");
    let mut energy_made = Fx::ZERO;
    // Per running building: (id, production, upkeep) for its active count.
    let mut pending: Vec<(String, Amounts, Amounts)> = Vec::new();
    for b in &content().buildings {
        let n = g.active(&b.id) as i64;
        if n == 0 || b.upkeep.is_empty() != free {
            continue;
        }
        let f = output_factor(r, g, b);
        let prod = b
            .produces
            .iter()
            .map(|(res, &v)| {
                let mut amt = v.mul_int(n) * f;
                if res == SCIENCE {
                    amt = amt * science;
                }
                (res.clone(), amt)
            })
            .collect::<BTreeMap<_, _>>();
        let upk = b.upkeep.iter().map(|(res, &v)| (res.clone(), v.mul_int(n) * upkeep_mult)).collect();
        if all_capped(r, g, &prod) {
            g.run.today.stalls.insert(b.id.clone(), Stall::OutputCapped);
        }
        pending.push((b.id.clone(), prod, upk));
    }
    loop {
        let mut progressed = false;
        let mut retry = Vec::new();
        for (id, prod, upk) in pending {
            if upk.iter().all(|(res, &need)| g.amount(res) >= need - eps) {
                let src = format!("building:{id}");
                for (res, &v) in &upk {
                    g.consume(&src, res, v);
                }
                for (res, &v) in &prod {
                    g.produce(&src, res, v);
                    if res == ENERGY {
                        energy_made += v;
                    }
                }
                progressed = true;
            } else {
                retry.push((id, prod, upk));
            }
        }
        pending = retry;
        if !progressed || pending.is_empty() {
            break;
        }
    }
    for (id, prod, upk) in pending {
        let frac = input_fraction(g, &upk);
        let src = format!("building:{id}");
        if let Some(s) = scarcest(g, &upk, frac) {
            g.run.today.shorts.insert(src.clone(), s);
        }
        // An earlier partial building may have made what this one lacked: run in full,
        // it is not starved.
        if frac < Fx::ONE {
            // Copies left unpaid: whole copies short of the fraction paid.
            if let Some(h) = g.run.buildings.get_mut(&id) {
                let paid = frac.mul_int(i64::from(h.active)).floor_int().clamp(0, i64::from(h.active));
                h.unpaid = h.active - paid as u32;
            }
            g.run.today.stalls.insert(id, Stall::InputStarved);
        }
        note_unmet(g, &upk, frac.max(Fx::ZERO));
        if frac <= Fx::ZERO {
            continue;
        }
        for (res, &v) in &upk {
            g.consume(&src, res, v * frac);
        }
        for (res, &v) in &prod {
            g.produce(&src, res, v * frac);
            if res == ENERGY {
                energy_made += v * frac;
            }
        }
    }
    energy_made
}

/// The end-of-tick clamp, the only place caps bind: anything over its cap is
/// cut to it (counted as today's overflow); anything below zero (an upkeep
/// tolerance) is lifted to zero.
pub fn clamp(r: &Rules, g: &mut GameState) {
    let ids: Vec<String> = g.run.resources.keys().cloned().collect();
    for id in ids {
        let cap = cap(r, g, &id);
        let v = g.run.resources.get_mut(&id).expect("listed");
        if let Some(c) = cap {
            if *v > c {
                let over = *v - c;
                *v = c;
                g.run.today.overflow.insert(id.clone(), over);
                g.track("cap", &id, Fx::ZERO, over);
            }
        }
        let v = g.run.resources.get_mut(&id).expect("listed");
        if v.is_negative() {
            *v = Fx::ZERO;
        }
    }
}

/// Folds the day just ended (`today.flows`) into the rolling average:
/// `avg += (day − avg) / flow_average_days` for every source already in it, a
/// source absent from the day counting zero; a source new to it starts at its
/// day's value. A step too small to move the average lands on the day's value,
/// so a source gone quiet reaches zero and drops out. Display only.
pub fn fold_flow_average(r: &Rules, g: &mut GameState) {
    let n = r.p.economy.flow_average_days;
    let step = |avg: Fx, day: Fx| {
        let next = avg + (day - avg).mul_ratio(1, n);
        if next == avg { day } else { next }
    };
    let today = &g.run.today.flows;
    let avg = &mut g.run.flow_average;
    for (res, a) in avg.iter_mut() {
        let day = today.get(res);
        for (src, f) in a.iter_mut() {
            let d = day.and_then(|d| d.get(src)).copied().unwrap_or_default();
            *f = Flow { produced: step(f.produced, d.produced), consumed: step(f.consumed, d.consumed) };
        }
        a.retain(|_, f| *f != Flow::default());
    }
    // A source new to the average starts at its day's value, not ramping up from zero.
    for (res, by) in today {
        let a = avg.entry(res.clone()).or_default();
        for (src, &d) in by {
            a.entry(src.clone()).or_insert(d);
        }
    }
    avg.retain(|_, a| !a.is_empty());
}

/// The building holding for a new run (the config's starting buildings).
pub fn starting_buildings(r: &Rules) -> BTreeMap<String, Holding> {
    r.p.economy
        .starting_buildings
        .iter()
        .filter(|(_, &n)| n > 0)
        .map(|(id, &n)| (id.clone(), Holding::starting(n)))
        .collect()
}
