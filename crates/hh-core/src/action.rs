//! Player actions, applied between ticks. JSON: `{"type": "buy", "building": "solar_panel"}`.
//! Program, pad and processor indices are 0-based.

use serde::{Deserialize, Serialize};

use crate::fx::Fx;
use crate::state::Entry;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    /// Buy one building (a content id) at its current price.
    Buy { building: String },
    /// Sell one: refunds its land only.
    Sell { building: String },
    /// Set how many of a building type run.
    SetActive { building: String, active: u32 },
    BuyLand,
    /// Replace a program's command queue.
    SetProgram { program: u32, entries: Vec<Entry> },
    /// Set how many processors run a program.
    SetProcessors { program: u32, processors: u32 },
    SetPadGood { pad: u32, good: String },
    SetPadPaused { pad: u32, paused: bool },
    /// Launch a pad now (works on a paused pad; needs cargo and fuel).
    Launch { pad: u32 },
    /// Every tradeable good once, first loaded first.
    SetLoadingPriority { goods: Vec<String> },
    Research { research: String },
    /// A project's funding rate for one resource, per day (0 stops it).
    SetProjectRate { project: String, resource: String, rate: Fx },
    /// Voluntary retirement (once unlocked).
    Retire,
    /// A debug cheat (`cheat.rs`): marks the game as cheated for good.
    Cheat { cheat: Cheat },
}

/// The debug menu's cheats. JSON: `{"type": "cheat", "cheat": {"kind": "grant",
/// "resource": "credits", "amount": "500"}}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Cheat {
    /// Adds to a resource, up to its cap (not processors or boredom).
    Grant { resource: String, amount: Fx },
    /// Sets boredom, up to its cap (at the cap the run retires on the next tick).
    SetBoredom { amount: Fx },
    /// Marks a jump of `days` days; the client then advances them as usual,
    /// so a replay plays them like any others.
    JumpAhead { days: u32 },
}
