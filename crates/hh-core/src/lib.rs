//! Helium Hustle's rules engine. Every rule lives here; nothing else holds one.
//!
//! Invariants (docs/architecture.md): no I/O (content is embedded, configs are
//! handed in), no clock (time is a tick count), no global RNG (per-purpose
//! streams in the state), no hash-order leaks (ordered maps only), the whole
//! state serde-serializable, exact replay, native = wasm. `clippy.toml` refuses
//! platform float functions, the clock and hash maps.

pub mod action;
pub mod cheat;
pub mod config;
pub mod content;
pub mod driver;
pub mod fx;
pub mod hash;
pub mod math;
pub mod modifiers;
pub mod modules;
pub mod program_text;
pub mod record;
pub mod rng;
pub mod rules;
pub mod scenario;
pub mod state;
pub mod view;

#[cfg(test)]
mod tests;

pub use action::{Action, Boost, Cheat};
pub use config::Config;
pub use content::{content, content_hash, Content};
pub use driver::{play, Policy};
pub use fx::Fx;
pub use record::{Record, Session, Stamped};
pub use rules::Rules;
pub use scenario::{scenario, scenarios, Scenario};
pub use state::GameState;
pub use view::View;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Bump on **any** change to how games play out (a rule, the tick loop, RNG
/// use, rounding), so results and saves from different rules are never
/// mistaken for each other. The web client discards a save from another
/// revision. The `golden_game` test fails on any play change until the golden
/// constants move; bumping this with them is the executor's rule (CLAUDE.md),
/// which no test can enforce (README, *Rules changes*).
pub const RULES_REVISION: u32 = 32;
