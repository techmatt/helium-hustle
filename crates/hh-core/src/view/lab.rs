//! The Research panel: each research item's price, state and scope icon.

use super::{keep, ResearchView};
use crate::content::content;
use crate::modules::*;
use crate::rules::Rules;
use crate::state::GameState;

/// The research items still on offer, in content order.
pub(super) fn research(r: &Rules, g: &GameState) -> Vec<ResearchView> {
    content()
        .research
        .iter()
        // A career-flag item once its flag is set (Programming bought) is gone for
        // good, the run it was bought in included.
        .filter(|item| !research::flag_held(g, item))
        .map(|item| {
            let cost = research::price(r, g, item);
            ResearchView {
                id: &item.id,
                name: &item.name,
                description: &item.description,
                category: &item.category,
                cost,
                requires: item.requires.as_deref(),
                done: g.run.research.contains(&item.id),
                visible: research::visible(g, item),
                affordable: research::blocker(r, g, item).is_none(),
                keep: keep::research(g, item),
            }
        })
        .collect()
}
