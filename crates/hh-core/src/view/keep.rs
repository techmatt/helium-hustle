//! Scope icons (display only, docs/rulings/ui.md): how long a card's effect lasts,
//! read from the rules that decide it, so the client shows a badge without knowing
//! them. `legacy`: kept when the AI retires. `eternal`: kept through everything, a
//! timeline's end included. Run-only things have none. Before the swarm's reveal
//! every durable thing shows `legacy`: the difference needs timelines to explain.

use crate::content::{Event, Research, ResearchEffect, Scope};
use crate::modules::{career, timeline};
use crate::state::GameState;

pub const LEGACY: &str = "legacy";
pub const ETERNAL: &str = "eternal";

/// `eternal` shown as `legacy` until the reveal.
fn shown(g: &GameState, eternal: bool) -> &'static str {
    if eternal && career::flag_set(g, timeline::REVEAL_FLAG) {
        ETERNAL
    } else {
        LEGACY
    }
}

/// A research item: bought research resets at retirement, but a career flag
/// (Programming) is kept through every reset (research.md).
pub fn research(g: &GameState, item: &Research) -> Option<&'static str> {
    match item.effect {
        Some(ResearchEffect::CareerFlag { .. }) => Some(shown(g, true)),
        _ => None,
    }
}

/// A project, by its scope (projects.md): `timeline` lasts until a timeline's end,
/// `forever` past it.
pub fn project(g: &GameState, scope: Scope) -> Option<&'static str> {
    match scope {
        Scope::Run => None,
        Scope::Timeline => Some(LEGACY),
        Scope::Forever => Some(shown(g, true)),
    }
}

/// A side quest with a reward: its completion outlives every reset and its unlocks
/// re-apply at each run's start (events.md). One with no unlock of its own has none.
pub fn side_quest(g: &GameState, e: &Event) -> Option<&'static str> {
    (!e.unlocks.is_empty()).then(|| shown(g, true))
}
