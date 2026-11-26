//! The navigation tabs: each tab and whether it shows.

use super::NavView;
use crate::content::{content, NavCondition};
use crate::modules::career;
use crate::state::GameState;

/// Every tab, in content order.
pub(super) fn nav(g: &GameState) -> Vec<NavView> {
    content()
        .nav
        .iter()
        .map(|t| NavView {
            id: &t.id,
            name: &t.name,
            visible: match &t.visible {
                NavCondition::Always => true,
                NavCondition::Panel { panel } => g.run.enabled_panels.contains(panel),
                NavCondition::BuildingKnown { building } => {
                    g.owned(building) > 0 || g.career.owned_buildings.contains(building)
                }
                NavCondition::Flag { flag } => career::flag_set(g, flag),
            },
        })
        .collect()
}
