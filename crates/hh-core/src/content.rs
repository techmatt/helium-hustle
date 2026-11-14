//! Game data, embedded at compile time from `content/` (`hh-core` reads no
//! files). Ids are unique across all content and are what states, actions and
//! records name, so reordering a file never breaks a save.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use crate::fx::Fx;
use crate::hash::{fnv1a64, hex};

/// The embedded files, in hash order: `(name, text)`.
const FILES: &[(&str, &str)] = &[
    ("resources.json", include_str!("../../../content/resources.json")),
    ("buildings.json", include_str!("../../../content/buildings.json")),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Building {
    pub id: String,
    pub name: String,
    /// Price of the first one, by resource id.
    pub cost: BTreeMap<String, Fx>,
    /// Made per day by each one, by resource id.
    pub produces: BTreeMap<String, Fx>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub resources: Vec<Resource>,
    pub buildings: Vec<Building>,
}

impl Content {
    fn load() -> Result<Content, String> {
        let parse = |i: usize| FILES[i].1;
        let resources: Vec<Resource> = serde_json::from_str(parse(0))
            .map_err(|e| format!("content/{}: {e}", FILES[0].0))?;
        let buildings: Vec<Building> = serde_json::from_str(parse(1))
            .map_err(|e| format!("content/{}: {e}", FILES[1].0))?;
        let c = Content {
            resources,
            buildings,
        };
        c.check()?;
        Ok(c)
    }

    /// Ids unique across all content; every reference names a resource.
    pub fn check(&self) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        let all = self
            .resources
            .iter()
            .map(|r| &r.id)
            .chain(self.buildings.iter().map(|b| &b.id));
        for id in all {
            if !ids.insert(id.as_str()) {
                return Err(format!("content: duplicate id {id:?}"));
            }
        }
        for b in &self.buildings {
            for r in b.cost.keys().chain(b.produces.keys()) {
                if self.resource(r).is_none() {
                    return Err(format!("content: building {:?} names unknown resource {r:?}", b.id));
                }
            }
        }
        Ok(())
    }

    pub fn resource(&self, id: &str) -> Option<&Resource> {
        self.resources.iter().find(|r| r.id == id)
    }

    pub fn building(&self, id: &str) -> Option<&Building> {
        self.buildings.iter().find(|b| b.id == id)
    }
}

/// The embedded content (checked once; a broken file panics at first use and
/// fails every test).
pub fn content() -> &'static Content {
    static C: OnceLock<Content> = OnceLock::new();
    C.get_or_init(|| Content::load().unwrap_or_else(|e| panic!("{e}")))
}

/// FNV-1a over the embedded files' names and raw bytes.
pub fn content_hash() -> String {
    let mut bytes = Vec::new();
    for (name, text) in FILES {
        bytes.extend_from_slice(name.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(text.as_bytes());
        bytes.push(0);
    }
    hex(fnv1a64(&bytes))
}
