//! Player actions, applied between ticks. JSON: `{"type": "buy", "building": "solar_panel"}`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    /// Buy one building (a content id) at its current price.
    Buy { building: String },
}
