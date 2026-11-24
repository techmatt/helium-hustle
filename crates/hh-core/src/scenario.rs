//! Scenarios: named starting states laid over a new game, for the sim
//! (`hh-sim run --scenario NAME`) and the web debug menu. The embedded ones
//! live in `scenarios/` (one file each, listed in `FILES`); any file in the
//! same format loads with [`Scenario::from_json`].
//!
//! ```json
//! { "name": "rich", "description": "...",
//!   "start": { "tick": 0, "resources": { "credits": "1000" }, "buildings": { "solar_panel": 3 },
//!              "panels": [], "programs": [[{ "op": "command", "command": "idle", "count": 1 }]] } }
//! ```
//!
//! Every key is required (`resources`, `buildings`, `panels` and `programs` may
//! be empty); a listed resource or building replaces the new game's value, the
//! rest stay; panels add to the run's; programs replace queues from the first.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::content::{content, NavCondition};
use crate::fx::Fx;
use crate::state::{Entry, GameState, Holding};

const FILES: &[(&str, &str)] = &[
    // First: the debug menu's default pick.
    ("everything.json", include_str!("../../../scenarios/everything.json")),
    ("rich.json", include_str!("../../../scenarios/rich.json")),
    ("at_cap.json", include_str!("../../../scenarios/at_cap.json")),
    ("late_unlocks.json", include_str!("../../../scenarios/late_unlocks.json")),
    ("pad_loading.json", include_str!("../../../scenarios/pad_loading.json")),
    ("golden_start.json", include_str!("../../../scenarios/golden_start.json")),
    ("story_acts.json", include_str!("../../../scenarios/story_acts.json")),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Start {
    pub tick: u64,
    pub resources: BTreeMap<String, Fx>,
    pub buildings: BTreeMap<String, u32>,
    /// Nav panels enabled this run (`projects`, `ideologies`, `retirement`:
    /// the nav tiles shown by a panel).
    pub panels: Vec<String>,
    /// Program queues from the first program on (the rest stay empty).
    pub programs: Vec<Vec<Entry>>,
    /// Career flags set at the start.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flags: Vec<String>,
    /// Projects counted as completed once in their scope, their rewards
    /// live (`conditional_logic`: `if` open; the golden game's start, README
    /// *Rules changes*).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projects: Vec<String>,
    /// Events taken as already fired, in order: each one's unlocks apply,
    /// it counts as seen, and a quest or side quest among them as completed
    /// (the story beats done).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<String>,
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
        for p in &self.start.panels {
            if !c.nav.iter().any(|t| matches!(&t.visible, NavCondition::Panel { panel } if panel == p)) {
                errors.push(format!("start.panels: {p:?} is no nav tile's panel"));
            }
        }
        for e in &self.start.events {
            if !c.events.iter().any(|x| &x.id == e) {
                errors.push(format!("start.events: {e:?} is no event"));
            }
        }
        for p in &self.start.projects {
            if c.project(p).is_none() {
                errors.push(format!("start.projects: {p:?} is no project"));
            }
        }
        for (i, entries) in self.start.programs.iter().enumerate() {
            for e in entries {
                // A scenario's programs skip `set_program`: command lines only.
                if e.indented || e.command_id().is_none_or(|id| c.command(id).is_none()) || e.count() == 0 {
                    errors.push(format!("start.programs[{i}]: bad line {:?}", e.describe()));
                }
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
                g.run.buildings.insert(b.clone(), Holding::starting(n));
                g.career.owned_buildings.insert(b.clone());
            }
        }
        for id in &self.start.projects {
            crate::modules::projects::mark_completed(g, id);
        }
        crate::modules::projects::grant_starting_research(g);
        crate::modules::economy::refresh(g);
        let free = crate::modules::programs::capacity(g) - crate::modules::programs::assigned(g);
        if let Some(p) = g.run.programs.first_mut() {
            p.processors += free;
        }
        g.run.enabled_panels.extend(self.start.panels.iter().cloned());
        g.career.flags.extend(self.start.flags.iter().cloned());
        for id in &self.start.events {
            let e = content().events.iter().find(|x| &x.id == id).expect("checked");
            if e.is_quest() || e.is_side() {
                g.career.completed_quests.insert(e.id.clone());
            }
            g.run.events.insert(e.id.clone(), g.run.day);
            g.career.seen_events.insert(e.id.clone());
            crate::modules::events::apply_unlocks(g, e);
        }
        for (p, entries) in g.run.programs.iter_mut().zip(&self.start.programs) {
            p.entries = entries.clone();
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
