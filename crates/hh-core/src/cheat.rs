//! Debug cheats (the web debug menu's Cheats section). They are actions like
//! any other, so a cheated game replays exactly; any cheat marks the game
//! (`GameState::cheated`) for good. No rule reads the mark.

use crate::action::Cheat;
use crate::content::content;
use crate::fx::Fx;
use crate::modules::economy::{self, BOREDOM, PROCESSORS};
use crate::rules::Rules;
use crate::state::GameState;

/// The grant cheat may grant this content resource: not processors (they
/// come from Data Centers) nor boredom (set it instead).
pub fn grantable(resource: &str) -> bool {
    resource != PROCESSORS && resource != BOREDOM
}

pub fn apply(r: &Rules, g: &mut GameState, c: &Cheat) -> Result<(), String> {
    match c {
        Cheat::Grant { resource, amount } => {
            if content().resource(resource).is_none() {
                return Err(format!("cheat: unknown resource {resource:?}"));
            }
            if !grantable(resource) {
                return Err(format!("cheat: can't grant {resource} (processors come from Data Centers; set boredom instead)"));
            }
            if *amount < Fx::ZERO {
                return Err("cheat: grant a positive amount".into());
            }
            let mut v = g.amount(resource).saturating_add(*amount);
            if let Some(cap) = economy::cap(r, g, resource) {
                v = v.min(cap.max(g.amount(resource)));
            }
            g.run.resources.insert(resource.clone(), v);
        }
        Cheat::SetBoredom { amount } => {
            if *amount < Fx::ZERO {
                return Err("cheat: boredom can't be negative".into());
            }
            let cap = economy::cap(r, g, BOREDOM).unwrap_or(Fx::MAX);
            g.run.resources.insert(BOREDOM.to_string(), (*amount).min(cap));
        }
        Cheat::JumpAhead { days } => {
            if *days == 0 {
                return Err("cheat: jump at least one day".into());
            }
        }
    }
    g.cheated = true;
    Ok(())
}
