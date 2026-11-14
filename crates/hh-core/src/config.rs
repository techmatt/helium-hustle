//! Game configs: which modules play, in which variant, with every tunable.
//!
//! A config is JSON (`configs/baseline.json`):
//!
//! ```json
//! { "name": "baseline",
//!   "modules": { "<module>": { "variant": "<name>", "params": { "<param>": "<value>" } } } }
//! ```
//!
//! **Every parameter must be given**: no defaults in code. Unknown, missing and
//! wrong-kind keys are errors naming them (all of them, not just the first).
//! `Fx` values are decimal strings (`"1.15"`), integers JSON integers.
//! Modules play in registry order (`modules::REGISTRY`), not config order.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::fx::Fx;
use crate::hash::{fnv1a64, hex};
use crate::modules::{self, ParamKind};

/// A parameter's value, as resolved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Param {
    Int(i64),
    Fx(Fx),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleConfig {
    pub variant: String,
    pub params: BTreeMap<String, Param>,
}

impl ModuleConfig {
    /// A parameter `Config::from_json` checked to be an `Fx`.
    pub fn fx(&self, name: &str) -> Fx {
        match self.params.get(name) {
            Some(Param::Fx(v)) => *v,
            other => panic!("param {name}: expected a checked Fx, found {other:?}"),
        }
    }

    /// A parameter `Config::from_json` checked to be an integer.
    pub fn int(&self, name: &str) -> i64 {
        match self.params.get(name) {
            Some(Param::Int(v)) => *v,
            other => panic!("param {name}: expected a checked integer, found {other:?}"),
        }
    }
}

/// A resolved, checked config. Build it with [`Config::from_json`] or
/// [`Config::check`]; its JSON form is canonical (every map ordered, every
/// `Fx` with six places), so it hashes the same however it was written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub name: String,
    pub modules: BTreeMap<String, ModuleConfig>,
}

impl Config {
    /// Parses and checks a config, naming every bad key.
    pub fn from_json(json: &str) -> Result<Config, String> {
        let v: Value = serde_json::from_str(json).map_err(|e| format!("config: bad JSON: {e}"))?;
        Self::from_value(&v)
    }

    pub fn from_value(v: &Value) -> Result<Config, String> {
        let mut errors = Vec::new();
        let Some(top) = v.as_object() else {
            return Err("config: expected an object".into());
        };
        for k in top.keys() {
            if k != "name" && k != "modules" {
                errors.push(format!("{k}: unknown key"));
            }
        }
        let name = match top.get("name") {
            Some(Value::String(s)) => s.clone(),
            Some(_) => {
                errors.push("name: expected a string".into());
                String::new()
            }
            None => {
                errors.push("name: missing".into());
                String::new()
            }
        };
        let mut modules_out = BTreeMap::new();
        match top.get("modules") {
            None => errors.push("modules: missing".into()),
            Some(Value::Object(ms)) => {
                for (id, m) in ms {
                    if let Some(mc) = check_module(id, m, &mut errors) {
                        modules_out.insert(id.clone(), mc);
                    }
                }
            }
            Some(_) => errors.push("modules: expected an object".into()),
        }
        if errors.is_empty() {
            Ok(Config {
                name,
                modules: modules_out,
            })
        } else {
            Err(format!("config: {}", errors.join("; ")))
        }
    }

    /// Re-checks an already-typed config (one read back from a state or a
    /// record) the same way.
    pub fn check(&self) -> Result<(), String> {
        let v = serde_json::to_value(self).map_err(|e| e.to_string())?;
        Self::from_value(&v).map(|_| ())
    }

    /// FNV-1a over the canonical JSON, `name` excluded: two configs that play
    /// the same hash the same.
    pub fn hash(&self) -> String {
        let mut c = self.clone();
        c.name.clear();
        hex(fnv1a64(serde_json::to_string(&c).expect("config serializes").as_bytes()))
    }

    pub fn module(&self, id: &str) -> Option<&ModuleConfig> {
        self.modules.get(id)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("config serializes")
    }
}

fn check_module(id: &str, m: &Value, errors: &mut Vec<String>) -> Option<ModuleConfig> {
    let Some(def) = modules::REGISTRY.iter().find(|d| d.id == id) else {
        let known: Vec<_> = modules::REGISTRY.iter().map(|d| d.id).collect();
        errors.push(format!("modules.{id}: unknown module (known: {})", known.join(", ")));
        return None;
    };
    let Some(obj) = m.as_object() else {
        errors.push(format!("modules.{id}: expected an object"));
        return None;
    };
    for k in obj.keys() {
        if k != "variant" && k != "params" {
            errors.push(format!("modules.{id}.{k}: unknown key"));
        }
    }
    let variant = match obj.get("variant") {
        Some(Value::String(s)) => s.clone(),
        Some(_) => {
            errors.push(format!("modules.{id}.variant: expected a string"));
            return None;
        }
        None => {
            errors.push(format!("modules.{id}.variant: missing"));
            return None;
        }
    };
    let Some(vdef) = def.variants.iter().find(|v| v.name == variant) else {
        let known: Vec<_> = def.variants.iter().map(|v| v.name).collect();
        errors.push(format!(
            "modules.{id}.variant: unknown variant {variant:?} (known: {})",
            known.join(", ")
        ));
        return None;
    };
    let empty = serde_json::Map::new();
    let params = match obj.get("params") {
        Some(Value::Object(p)) => p,
        Some(_) => {
            errors.push(format!("modules.{id}.params: expected an object"));
            return None;
        }
        None => {
            errors.push(format!("modules.{id}.params: missing"));
            &empty
        }
    };
    let before = errors.len();
    for k in params.keys() {
        if !vdef.params.iter().any(|p| p.name == k) {
            errors.push(format!("modules.{id}.params.{k}: unknown parameter"));
        }
    }
    let mut out = BTreeMap::new();
    for p in vdef.params {
        let path = format!("modules.{id}.params.{}", p.name);
        match (params.get(p.name), p.kind) {
            (None, _) => errors.push(format!("{path}: missing")),
            (Some(Value::String(s)), ParamKind::Fx) => match Fx::parse(s) {
                Ok(v) => {
                    out.insert(p.name.to_string(), Param::Fx(v));
                }
                Err(e) => errors.push(format!("{path}: {e}")),
            },
            (Some(v), ParamKind::Int) if v.is_i64() => {
                out.insert(p.name.to_string(), Param::Int(v.as_i64().expect("checked")));
            }
            (Some(_), ParamKind::Fx) => {
                errors.push(format!("{path}: expected a decimal string such as \"1.5\""))
            }
            (Some(_), ParamKind::Int) => errors.push(format!("{path}: expected an integer")),
        }
    }
    for p in vdef.params {
        if let (Some(v), Some((lo, hi))) = (out.get(p.name), p.range) {
            let ok = match v {
                Param::Fx(x) => {
                    Fx::parse(lo).expect("range bound") <= *x && *x <= Fx::parse(hi).expect("range bound")
                }
                Param::Int(i) => {
                    lo.parse::<i64>().expect("range bound") <= *i
                        && *i <= hi.parse::<i64>().expect("range bound")
                }
            };
            if !ok {
                errors.push(format!("modules.{id}.params.{}: out of range {lo}..={hi}", p.name));
            }
        }
    }
    (errors.len() == before).then_some(ModuleConfig {
        variant,
        params: out,
    })
}
