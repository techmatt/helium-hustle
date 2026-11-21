//! Modules: one rules area each; a variant is one way of doing it.
//!
//! A module is `modules/<id>.rs`: its `ModuleDef` (variants, each with its
//! parameters and RNG purposes) in [`REGISTRY`], a typed `Params` built from its
//! config entry, and the functions that play its rules. Arc 1's systems read
//! each other every tick (boredom reads research, ideology, projects and the
//! career), so modules are not isolated objects: [`crate::Rules`] holds every
//! module's params ([`Params`]) and calls their functions in one tick order
//! (`rules.rs`). Every module in the registry must be in the config.

use crate::config::ModuleConfig;

pub mod achievements;
pub mod boredom;
pub mod career;
pub mod demand;
pub mod dream;
pub mod economy;
pub mod events;
pub mod ideology;
pub mod programs;
pub mod projects;
pub mod research;
pub mod shipments;
pub mod timeline;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    /// A decimal string, read as `Fx`.
    Fx,
    /// A JSON integer.
    Int,
    /// A list of decimal strings.
    FxList,
    /// A list of integers.
    IntList,
    /// An object of decimal strings by content id.
    FxMap,
    /// An object of integers by content id.
    IntMap,
}

pub struct ParamDef {
    pub name: &'static str,
    pub kind: ParamKind,
    /// Inclusive bounds, written as the param is (`"0"`, `"1.5"`), if any; for a
    /// list or map, each value's.
    pub range: Option<(&'static str, &'static str)>,
    pub doc: &'static str,
}

pub struct VariantDef {
    pub name: &'static str,
    pub params: &'static [ParamDef],
    /// RNG purposes this variant draws from; never shared with another module.
    pub rng: &'static [&'static str],
}

pub struct ModuleDef {
    pub id: &'static str,
    pub variants: &'static [VariantDef],
}

/// Shorthand for a `ParamDef`.
pub(crate) const fn param(
    name: &'static str,
    kind: ParamKind,
    range: Option<(&'static str, &'static str)>,
    doc: &'static str,
) -> ParamDef {
    ParamDef { name, kind, range, doc }
}

/// Every module. This order only orders config checks and their errors; the tick
/// plays modules in the fixed order of `Rules::tick` (`rules.rs`).
pub static REGISTRY: &[&ModuleDef] = &[
    &boredom::DEF,
    &economy::DEF,
    &demand::DEF,
    &programs::DEF,
    &dream::DEF,
    &projects::DEF,
    &shipments::DEF,
    &research::DEF,
    &ideology::DEF,
    &events::DEF,
    &achievements::DEF,
    &career::DEF,
    &timeline::DEF,
];

/// Every module's params, compiled from a config.
#[derive(Clone)]
pub struct Params {
    pub boredom: boredom::Params,
    pub economy: economy::Params,
    pub demand: demand::Params,
    pub programs: programs::Params,
    pub dream: dream::Params,
    pub projects: projects::Params,
    pub shipments: shipments::Params,
    pub research: research::Params,
    pub ideology: ideology::Params,
    pub events: events::Params,
    pub achievements: achievements::Params,
    pub career: career::Params,
    pub timeline: timeline::Params,
}

impl Params {
    /// From a checked config; `Err` names each registry module it lacks.
    pub fn new(get: impl Fn(&str) -> Option<ModuleConfig>) -> Result<Params, String> {
        let missing: Vec<_> = REGISTRY.iter().filter(|d| get(d.id).is_none()).map(|d| d.id).collect();
        if !missing.is_empty() {
            return Err(format!(
                "config: missing module(s) {} (every module plays)",
                missing.join(", ")
            ));
        }
        let m = |id: &str| get(id).expect("checked above");
        Ok(Params {
            boredom: boredom::Params::new(&m("boredom"))?,
            economy: economy::Params::new(&m("economy"))?,
            demand: demand::Params::new(&m("demand"))?,
            programs: programs::Params::new(&m("programs"))?,
            dream: dream::Params::new(&m("dream"))?,
            projects: projects::Params::new(&m("projects"))?,
            shipments: shipments::Params::new(&m("shipments"))?,
            research: research::Params::new(&m("research"))?,
            ideology: ideology::Params::new(&m("ideology"))?,
            events: events::Params::new(&m("events"))?,
            achievements: achievements::Params::new(&m("achievements"))?,
            career: career::Params::new(&m("career"))?,
            timeline: timeline::Params::new(&m("timeline"))?,
        })
    }
}

/// The RNG purposes a module's variant declares.
pub(crate) fn rng_purposes(id: &str, variant: &str) -> &'static [&'static str] {
    REGISTRY
        .iter()
        .find(|d| d.id == id)
        .and_then(|d| d.variants.iter().find(|v| v.name == variant))
        .map(|v| v.rng)
        .unwrap_or(&[])
}
