//! The rivals and the adversaries (speculators, rivals): who is around and how close each is.

use super::{AdversaryView, RivalView};
use crate::content::{content, Condition};
use crate::modules::attention;
use crate::rules::Rules;
use crate::state::GameState;

/// Every rival, with how many of its dumps this run's activity feed shows.
pub(super) fn rivals(g: &GameState) -> Vec<RivalView> {
    content()
        .rivals
        .iter()
        .map(|rv| RivalView {
            id: &rv.id,
            name: &rv.name,
            good: &rv.good,
            dumps_seen: g.run.activity.iter().filter(|a| a.kind == "rival_dump" && a.id == rv.id).count() as u32,
        })
        .collect()
}

/// Every adversary, in `ADVERSARIES` order.
pub(super) fn adversaries(r: &Rules, g: &GameState) -> Vec<AdversaryView> {
    attention::ADVERSARIES
        .iter()
        .map(|&id| AdversaryView {
            id,
            attention: attention::attention(g, id),
            threshold: attention::threshold_of(r, id),
            present: attention::present(r, g, id),
            seen: adversary_seen(r, g, id),
        })
        .collect()
}

/// Present now, or a seen event's condition was its presence.
fn adversary_seen(r: &Rules, g: &GameState, id: &str) -> bool {
    attention::present(r, g, id)
        || content().events.iter().any(|e| {
            matches!(&e.condition, Condition::AdversaryPresent { adversary } if adversary == id)
                && (g.career.seen_events.contains(&e.id) || g.career.completed_quests.contains(&e.id))
        })
}
