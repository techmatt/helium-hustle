//! Debug cheats (the web debug menu's Cheats section). They are actions like
//! any other, so a cheated game replays exactly; any cheat marks the game
//! (`GameState::cheated`) for good. No rule reads the mark. The accelerators
//! (boost, autopilot, cheat building, complete research) also mark it
//! `accelerated`; the boost is read through
//! [`boosted`] and the boredom multiplier, the cheat building through [`tick`].

use crate::action::{Boost, Cheat};
use crate::content::content;
use crate::fx::Fx;
use crate::modules::economy::{self, BOREDOM, CREDITS, LAND, PROCESSORS, SCIENCE};
use crate::rules::Rules;
use crate::state::GameState;

/// The largest boost factor (debug only, so not a config number).
pub const BOOST_MAX: i64 = 1000;
/// A cheat building's price in credits (debug only, so not a config number).
pub const CHEAT_BUILDING_COST: i64 = 1;
/// What each cheat building adds a day to every shown income and takes off
/// boredom (debug only, so not a config number).
pub const CHEAT_BUILDING_RATE: i64 = 10;
/// The source name a cheat building's flows show under.
pub const CHEAT_SOURCE: &str = "cheat_building";

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
        Cheat::Boost { production, research, boredom } => {
            if *production <= Fx::ZERO || *research <= Fx::ZERO || *boredom < Fx::ZERO {
                return Err("cheat: boost factors must be positive (boredom may be 0)".into());
            }
            // Past this a product can overflow `Fx` (a panic by design): no debug
            // run needs more.
            let most = Fx::from_int(BOOST_MAX);
            if *production > most || *research > most || *boredom > most {
                return Err(format!("cheat: boost factors are at most {BOOST_MAX}"));
            }
            let b = Boost { production: *production, research: *research, boredom: *boredom };
            g.boost = Some(b);
            g.accelerated = true;
        }
        Cheat::Autopilot { target } => {
            if target.is_empty() {
                return Err("cheat: autopilot needs a target".into());
            }
            g.accelerated = true;
        }
        Cheat::CheatBuilding => {
            let cost = Fx::from_int(CHEAT_BUILDING_COST);
            if g.amount(CREDITS) < cost {
                return Err(format!("cheat: a cheat building costs {CHEAT_BUILDING_COST} credit"));
            }
            g.consume(CHEAT_SOURCE, CREDITS, cost);
            g.run.cheat_buildings += 1;
            g.accelerated = true;
        }
        Cheat::CompleteResearch => {
            for item in &content().research {
                if g.run.research.insert(item.id.clone()) {
                    g.career.researched.insert(item.id.clone());
                    g.run.stats.research_completed += 1;
                    g.note("research", &item.id);
                }
            }
            g.accelerated = true;
        }
    }
    g.cheated = true;
    Ok(())
}

/// After boredom grows each day: each cheat building makes `CHEAT_BUILDING_RATE`
/// of every resource the Resources panel shows (but land, processors and
/// boredom) and takes as much off boredom (down to 0). Nothing without one.
pub fn tick(g: &mut GameState) {
    let n = g.run.cheat_buildings;
    if n == 0 {
        return;
    }
    let v = Fx::from_int(CHEAT_BUILDING_RATE) * Fx::from_int(i64::from(n));
    for res in &content().resources {
        if [LAND, PROCESSORS, BOREDOM].contains(&res.id.as_str()) || !economy::shown(g, res) {
            continue;
        }
        g.produce_unboosted(CHEAT_SOURCE, &res.id, v);
    }
    let off = v.min(g.amount(BOREDOM).max(Fx::ZERO));
    g.consume(CHEAT_SOURCE, BOREDOM, off);
}

/// `v` of `resource` made, times the debug boost (`research` for science,
/// `production` for the rest); `v` itself with no boost set.
pub fn boosted(g: &GameState, resource: &str, v: Fx) -> Fx {
    match g.boost {
        None => v,
        Some(b) if resource == SCIENCE => v * b.research,
        Some(b) => v * b.production,
    }
}
