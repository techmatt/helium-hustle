//! The Resources and Buildings panels: each resource's amount, cap and rates, and each building's price, state and action.

use super::breakdown::short_view;
use super::{BuildingView, ResourceView};
use crate::action::Action;
use crate::content::{content, BuildingEffect};
use crate::fx::Fx;
use crate::modules::*;
use crate::rules::Rules;
use crate::state::GameState;

/// Every resource, in content order. `next` is the state a day on (`r.peek(g)`);
/// its rates read 0 when `same_run` is false (the next day ends this run).
pub(super) fn resources(r: &Rules, g: &GameState, next: &GameState, same_run: bool) -> Vec<ResourceView> {
    content()
        .resources
        .iter()
        .map(|res| {
            ResourceView {
                id: &res.id,
                name: &res.name,
                amount: g.amount(&res.id),
                cap: economy::cap(r, g, &res.id),
                per_day: if same_run { next.run.day_net.get(&res.id).copied().unwrap_or(Fx::ZERO) } else { Fx::ZERO },
                change_per_day: if same_run { next.amount(&res.id) - g.amount(&res.id) } else { Fx::ZERO },
                overflow: if same_run { next.run.today.overflow.get(&res.id).copied().unwrap_or(Fx::ZERO) } else { Fx::ZERO },
                unmet: if same_run { next.run.today.unmet.get(&res.id).copied().unwrap_or(Fx::ZERO) } else { Fx::ZERO },
                visible: economy::shown(g, res),
                grantable: crate::cheat::grantable(&res.id),
            }
        })
        .collect()
}

/// Every building, in content order.
pub(super) fn buildings(r: &Rules, g: &GameState) -> Vec<BuildingView> {
    content()
        .buildings
        .iter()
        .map(|b| {
            let h = g.run.buildings.get(&b.id).copied().unwrap_or_default();
            let cost = economy::price(r, g, b);
            let mut short: Vec<String> =
                cost.iter().filter(|(res, &v)| g.amount(res) < v).map(|(res, _)| res.clone()).collect();
            if g.amount(economy::LAND) < Fx::from_int(b.land as i64) {
                short.push(economy::LAND.to_string());
            }
            let effect = |grant: bool| {
                b.effects
                    .iter()
                    .filter_map(|e| match e {
                        BuildingEffect::Store { resource, amount } if !grant => Some((resource.as_str(), *amount)),
                        BuildingEffect::Grant { resource, amount } if grant => Some((resource.as_str(), *amount)),
                        _ => None,
                    })
                    .collect()
            };
            BuildingView {
                id: &b.id,
                name: &b.name,
                description: &b.description,
                category: &b.category,
                ideology: b.ideology.as_deref(),
                owned: h.owned,
                active: h.active,
                working: g.working(&b.id),
                bonus: h.bonus,
                max_count: b.max_count,
                cost,
                land: b.land,
                short,
                produces: &b.produces,
                upkeep: &b.upkeep,
                requires: &b.requires,
                stores: effect(false),
                grants: effect(true),
                affordable: economy::buy_blocker(r, g, b).is_none(),
                at_limit: economy::at_limit(g, b),
                unlocked: economy::unlocked(g, b),
                visible: economy::visible(g, b),
                stall: g.run.today.stalls.get(&b.id).copied(),
                starved: {
                    let k = format!("building:{}", b.id);
                    g.run.today.shorts.get(&k).map(|s| short_view(g, &k, s))
                },
                buy: Action::Buy { building: b.id.clone() },
            }
        })
        .collect()
}
