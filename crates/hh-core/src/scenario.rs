//! Scenarios: named starting states laid over a new game, for the sim
//! (`hh-sim run --scenario NAME`) and the web debug menu. The embedded ones
//! live in `scenarios/` (one file each, listed in `FILES`); any file in the
//! same format loads with [`Scenario::from_json`].
//!
//! ```json
//! { "name": "rich", "description": "...",
//!   "start": { "tick": 0, "resources": { "credits": "1000" }, "buildings": { "solar_panel": 3 } } }
//! ```
//!
//! Every key is required (`resources` and `buildings` may be empty); a listed
//! resource or building replaces the new game's value, the rest stay.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::content::content;
use crate::fx::Fx;
use crate::state::{GameState, Holding};

const FILES: &[(&str, &str)] = &[
    ("rich.json", include_str!("../../../scenarios/rich.json")),
    ("at_cap.json", include_str!("../../../scenarios/at_cap.json")),
    ("pad_loading.json", include_str!("../../../scenarios/pad_loading.json")),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Start {
    pub tick: u64,
    pub resources: BTreeMap<String, Fx>,
    pub buildings: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub name: String,
    pub description: String,
    pub start: Start,
}

impl Scenario {
    /// Parses and checks a scenario against the content.
    pub fn from_json(json: &str) -> Result<Scenario, String> {
        let s: Scenario = serde_json::from_str(json).map_err(|e| format!("scenario: {e}"))?;
        s.check()?;
        Ok(s)
    }

    pub fn check(&self) -> Result<(), String> {
        let c = content();
        let mut errors = Vec::new();
        for r in self.start.resources.keys() {
            if c.resource(r).is_none() {
                errors.push(format!("start.resources.{r}: unknown resource"));
            }
        }
        for b in self.start.buildings.keys() {
            if c.building(b).is_none() {
                errors.push(format!("start.buildings.{b}: unknown building"));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("scenario {:?}: {}", self.name, errors.join("; ")))
        }
    }

    /// Lays the start over a new game: the tick (and the run's day), resources
    /// and buildings (owned and active); processors gained go to the first program.
    pub(crate) fn apply_to(&self, g: &mut GameState) {
        g.tick = self.start.tick;
        g.run.day = self.start.tick;
        // The run's schedules start from the scenario's day, not day 0.
        g.run.next_burst_day += self.start.tick;
        for d in g.run.rival_next.values_mut() {
            *d += self.start.tick;
        }
        for (r, &v) in &self.start.resources {
            g.run.resources.insert(r.clone(), v);
        }
        for (b, &n) in &self.start.buildings {
            if n == 0 {
                g.run.buildings.remove(b);
            } else {
                g.run.buildings.insert(b.clone(), Holding { owned: n, active: n, bonus: 0, free: n });
                g.career.owned_buildings.insert(b.clone());
            }
        }
        crate::modules::economy::refresh(g);
        let free = crate::modules::programs::processors(g) - crate::modules::programs::assigned(g);
        if let Some(p) = g.run.programs.first_mut() {
            p.processors += free;
        }
    }
}

/// The embedded scenarios, in `FILES` order (checked once; a broken file
/// panics at first use and fails every test).
pub fn scenarios() -> &'static [Scenario] {
    static S: OnceLock<Vec<Scenario>> = OnceLock::new();
    S.get_or_init(|| {
        let list: Vec<Scenario> = FILES
            .iter()
            .map(|(f, text)| {
                Scenario::from_json(text).unwrap_or_else(|e| panic!("scenarios/{f}: {e}"))
            })
            .collect();
        for (i, s) in list.iter().enumerate() {
            assert!(
                !list[..i].iter().any(|t| t.name == s.name),
                "scenarios: duplicate name {:?}",
                s.name
            );
        }
        list
    })
}

/// An embedded scenario by name.
pub fn scenario(name: &str) -> Result<&'static Scenario, String> {
    scenarios().iter().find(|s| s.name == name).ok_or_else(|| {
        let names: Vec<_> = scenarios().iter().map(|s| s.name.as_str()).collect();
        format!("unknown scenario {name:?} (known: {})", names.join(", "))
    })
}
