//! The navigation tabs: each tab and whether it shows.

use super::NavView;
use crate::content::{content, NavCondition};
use crate::modules::{career, events};
use crate::rules::Rules;
use crate::state::GameState;

/// Every tab, in content order. A panel an event will enable at this tick's
/// end (a research just bought, the clock paused) shows at once.
pub(super) fn nav(r: &Rules, g: &GameState) -> Vec<NavView> {
    let pending = events::pending_panels(r, g);
    content()
        .nav
        .iter()
        .map(|t| NavView {
            id: &t.id,
            name: &t.name,
            visible: match &t.visible {
                NavCondition::Always => true,
                NavCondition::Panel { panel } => {
                    g.run.enabled_panels.contains(panel) || pending.contains(&panel.as_str())
                }
                NavCondition::BuildingKnown { building } => {
                    g.owned(building) > 0 || g.career.owned_buildings.contains(building)
                }
                NavCondition::Flag { flag } => career::flag_set(g, flag),
            },
        })
        .collect()
}
