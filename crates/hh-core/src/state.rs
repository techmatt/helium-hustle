//! The game state: everything a saved game needs to reload to the same future.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::Config;
use crate::fx::Fx;
use crate::hash::{as_string, fnv1a64, hex};
use crate::rng::Stream;

/// One game. Every map is ordered, so its JSON is stable; the seed and RNG
/// positions are decimal strings (too big for JavaScript numbers).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameState {
    #[serde(with = "as_string")]
    pub seed: u64,
    /// The resolved config this game plays under.
    pub config: Config,
    /// Days elapsed (1 tick = 1 in-game day).
    pub tick: u64,
    /// Every content resource, by id.
    pub resources: BTreeMap<String, Fx>,
    /// Buildings owned, by content id (a count; absent is none).
    pub buildings: BTreeMap<String, u32>,
    /// One stream per declared purpose (`rng.rs`).
    pub rng: BTreeMap<String, Stream>,
}

impl GameState {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("state serializes")
    }

    pub fn from_json(json: &str) -> Result<GameState, String> {
        serde_json::from_str(json).map_err(|e| format!("bad state JSON: {e}"))
    }

    /// FNV-1a of the JSON form, as 16 hex digits: equal fingerprints, equal
    /// games (the replay and native = wasm checks).
    pub fn fingerprint(&self) -> String {
        hex(fnv1a64(self.to_json().as_bytes()))
    }

    pub fn amount(&self, resource: &str) -> Fx {
        self.resources.get(resource).copied().unwrap_or(Fx::ZERO)
    }

    pub fn owned(&self, building: &str) -> u32 {
        self.buildings.get(building).copied().unwrap_or(0)
    }

    pub fn stream(&mut self, purpose: &str) -> &mut Stream {
        self.rng
            .get_mut(purpose)
            .unwrap_or_else(|| panic!("rng purpose {purpose:?} not declared by any module"))
    }
}
