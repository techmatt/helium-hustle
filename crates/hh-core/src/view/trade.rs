//! The Shipments panel: launch pads, the loading order and the markets.

use super::{MarketView, PadView};
use crate::content::content;
use crate::modules::*;
use crate::rules::Rules;
use crate::state::GameState;

/// Each tradeable good with a market this run, its demand shown only once research reveals it.
pub(super) fn markets(r: &Rules, g: &GameState) -> Vec<MarketView> {
    let c = content();
    let aware = research::reveals_demand(g);
    shipments::goods(r)
        .into_iter()
        .filter_map(|good| {
            let m = g.run.markets.get(&good)?;
            Some(MarketView {
                name: c.resource(&good).map_or(good.clone(), |x| x.name.clone()),
                demand: aware.then_some(m.demand),
                tier: demand::TIERS[demand::tier(r, m.demand)],
                speculators: m.speculators_seen.then_some(m.speculators),
                good,
            })
        })
        .collect()
}

/// Every launch pad, in pad order.
pub(super) fn pads(r: &Rules, g: &GameState) -> Vec<PadView> {
    (0..g.run.pads.len())
        .map(|i| {
            let p = &g.run.pads[i];
            PadView {
                good: p.good.clone(),
                paused: p.paused,
                active: shipments::pad_active(g, i),
                unpaid: i < g.active(shipments::LAUNCH_PAD) as usize && !shipments::pad_active(g, i),
                cargo: p.cargo,
                capacity: r.p.shipments.pad_cargo_capacity,
                cooldown: p.cooldown,
                fuel: r.p.shipments.fuel_per_pad,
                load: shipments::load_amount(r, g),
                payout: shipments::payout(r, g, i),
                can_launch: shipments::can_launch(r, g, i),
                fueled: g.amount(shipments::PROPELLANT) >= r.p.shipments.fuel_per_pad,
                ready: shipments::ready(r, g, i),
                switch_returns: shipments::switch_returns(r, g, i),
            }
        })
        .collect()
}

/// The loading order's goods a pad may take now.
pub(super) fn pad_goods(g: &GameState) -> Vec<String> {
    g.run.loading_priority.iter().filter(|x| shipments::allowed(g, x)).cloned().collect()
}
