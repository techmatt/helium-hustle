//! Game configs: which modules play, in which variant, with every tunable.
//!
//! A config is JSON (`configs/baseline.json`):
//!
//! ```json
//! { "name": "baseline",
//!   "modules": { "<module>": { "variant": "<name>", "params": { "<param>": "<value>" } } } }
//! ```
//!
//! **Every parameter must be given**: no defaults in code. Unknown, missing,
//! wrong-kind and out-of-range keys are errors naming them (all of them, not just
//! the first). `Fx` values are decimal strings (`"1.15"`), integers JSON integers.
//! The tick plays modules in the fixed order of `Rules::tick` (`rules.rs`),
//! whatever the config or registry order.

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
    IntList(Vec<i64>),
    FxList(Vec<Fx>),
    IntMap(BTreeMap<String, i64>),
    FxMap(BTreeMap<String, Fx>),
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

    /// A checked list of integers. (An empty list reads back from JSON as
    /// either kind, so both empty forms are accepted.)
    pub fn int_list(&self, name: &str) -> Vec<i64> {
        match self.params.get(name) {
            Some(Param::IntList(v)) => v.clone(),
            Some(Param::FxList(v)) if v.is_empty() => Vec::new(),
            other => panic!("param {name}: expected a checked integer list, found {other:?}"),
        }
    }

    /// A checked list of decimals.
    pub fn fx_list(&self, name: &str) -> Vec<Fx> {
        match self.params.get(name) {
            Some(Param::FxList(v)) => v.clone(),
            Some(Param::IntList(v)) if v.is_empty() => Vec::new(),
            other => panic!("param {name}: expected a checked decimal list, found {other:?}"),
        }
    }

    /// A checked map of integers by content id.
    pub fn int_map(&self, name: &str) -> BTreeMap<String, i64> {
        match self.params.get(name) {
            Some(Param::IntMap(v)) => v.clone(),
            Some(Param::FxMap(v)) if v.is_empty() => BTreeMap::new(),
            other => panic!("param {name}: expected a checked integer map, found {other:?}"),
        }
    }

    /// A checked map of decimals by content id.
    pub fn fx_map(&self, name: &str) -> BTreeMap<String, Fx> {
        match self.params.get(name) {
            Some(Param::FxMap(v)) => v.clone(),
            Some(Param::IntMap(v)) if v.is_empty() => BTreeMap::new(),
            other => panic!("param {name}: expected a checked decimal map, found {other:?}"),
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
        let Some(v) = params.get(p.name) else {
            errors.push(format!("{path}: missing"));
            continue;
        };
        match parse_param(v, p.kind) {
            Ok(val) => {
                if let Some((lo, hi)) = p.range {
                    if !in_range(&val, lo, hi) {
                        errors.push(format!("{path}: out of range {lo}..={hi}"));
                        continue;
                    }
                }
                out.insert(p.name.to_string(), val);
            }
            Err(e) => errors.push(format!("{path}: {e}")),
        }
    }
    (errors.len() == before).then_some(ModuleConfig {
        variant,
        params: out,
    })
}

fn parse_fx(v: &Value) -> Result<Fx, String> {
    match v {
        Value::String(s) => Fx::parse(s),
        _ => Err("expected a decimal string such as \"1.5\"".into()),
    }
}

fn parse_int(v: &Value) -> Result<i64, String> {
    v.as_i64().ok_or_else(|| "expected an integer".to_string())
}

fn parse_list<T>(v: &Value, each: fn(&Value) -> Result<T, String>) -> Result<Vec<T>, String> {
    let Value::Array(a) = v else {
        return Err("expected a list".into());
    };
    a.iter()
        .enumerate()
        .map(|(i, x)| each(x).map_err(|e| format!("[{i}]: {e}")))
        .collect()
}

fn parse_map<T>(v: &Value, each: fn(&Value) -> Result<T, String>) -> Result<BTreeMap<String, T>, String> {
    let Value::Object(o) = v else {
        return Err("expected an object".into());
    };
    o.iter()
        .map(|(k, x)| each(x).map(|t| (k.clone(), t)).map_err(|e| format!("{k}: {e}")))
        .collect()
}

fn parse_param(v: &Value, kind: ParamKind) -> Result<Param, String> {
    Ok(match kind {
        ParamKind::Fx => Param::Fx(parse_fx(v)?),
        ParamKind::Int => Param::Int(parse_int(v)?),
        ParamKind::FxList => Param::FxList(parse_list(v, parse_fx)?),
        ParamKind::IntList => Param::IntList(parse_list(v, parse_int)?),
        ParamKind::FxMap => Param::FxMap(parse_map(v, parse_fx)?),
        ParamKind::IntMap => Param::IntMap(parse_map(v, parse_int)?),
    })
}

/// Every value (each element of a list or map) within `lo..=hi`.
fn in_range(v: &Param, lo: &str, hi: &str) -> bool {
    let fx = |x: &Fx| Fx::parse(lo).expect("range bound") <= *x && *x <= Fx::parse(hi).expect("range bound");
    let int = |i: &i64| {
        lo.parse::<i64>().expect("range bound") <= *i && *i <= hi.parse::<i64>().expect("range bound")
    };
    match v {
        Param::Fx(x) => fx(x),
        Param::Int(i) => int(i),
        Param::FxList(l) => l.iter().all(fx),
        Param::IntList(l) => l.iter().all(int),
        Param::FxMap(m) => m.values().all(fx),
        Param::IntMap(m) => m.values().all(int),
    }
}
