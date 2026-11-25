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
    /// Save a program's lines to the loadout library: a new slot (`slot`
    /// null) or over slot `slot`.
    SaveLoadout { program: u32, slot: Option<u32>, name: String },
    /// Replace a program's lines with a saved loadout's.
    LoadLoadout { loadout: u32, program: u32 },
    RenameLoadout { loadout: u32, name: String },
    DeleteLoadout { loadout: u32 },
    /// The good a pad carries; `good` null returns it to No cargo.
    SetPadGood { pad: u32, good: Option<String> },
    SetPadPaused { pad: u32, paused: bool },
    /// Launch a pad now (works on a paused pad; needs cargo and fuel).
    Launch { pad: u32 },
    /// Every tradeable good once, first loaded first.
    SetLoadingPriority { goods: Vec<String> },
    Research { research: String },
    /// Run one command once, now, by hand (no processor): as a program line
    /// would run it (commands.md, *Running by hand*).
    RunCommand { command: String },
    /// A project's funding rate for one resource, per day (0 stops it).
    SetProjectRate { project: String, resource: String, rate: Fx },
    /// Voluntary retirement (once unlocked).
    Retire,
    /// A debug cheat (`cheat.rs`): marks the game as cheated for good.
    Cheat { cheat: Cheat },
    // Draft (042), the later acts (docs/rules/timeline.md):
    /// Buy an Earth tech with Earth science.
    BuyEarthTech { tech: String },
    /// The meta-controller's reset plan: load `loadout` into `program` at each
    /// new run (`null` clears the tab's entry).
    SetResetPlan { program: u32, loadout: Option<u32> },
    /// Move the run's whole stock of a resource into the vault.
    Deposit { resource: String },
    /// Spend the vault on a super timeline.
    BeginSuperTimeline,
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
    /// An accelerator (`cheat.rs`, docs/rules/README.md *Cheats*): sets the
    /// career's debug boost. Each factor multiplies for the rest of the career
    /// (1 is off): `production` every resource made but science (buildings,
    /// commands, launch payouts), `research` science made, `boredom` boredom
    /// growth. Marks the game accelerated too.
    Boost { production: Fx, research: Fx, boredom: Fx },
    /// An accelerator: marks the start of an autopilot run (a sim player
    /// playing on toward `target`); its actions follow as usual, so a replay
    /// plays them like any others. Marks the game accelerated.
    Autopilot { target: String },
    /// Buys one cheat building for `cheat::CHEAT_BUILDING_COST` credits: each
    /// adds `cheat::CHEAT_BUILDING_RATE` a day to every shown income and takes
    /// as much off boredom, for the rest of the run.
    CheatBuilding,
    /// Completes every research item not done this run, free.
    CompleteResearch,
}

/// The career's debug boost (`Cheat::Boost`): factors, 1 = off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Boost {
    pub production: Fx,
    pub research: Fx,
    pub boredom: Fx,
}
