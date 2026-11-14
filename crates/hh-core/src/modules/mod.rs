//! Modules: one rules area each; a variant is one way of doing it.
//!
//! A module is a `ModuleDef` in [`REGISTRY`] (its variants, each with its
//! parameters and RNG purposes) and a [`Module`] impl built from its config
//! entry. Modules play in registry order. To add one: `modules/<id>.rs` with
//! `pub static DEF`, listed in `REGISTRY` and `build`, and its entry in the
//! configs that play it.

use crate::action::Action;
use crate::config::ModuleConfig;
use crate::state::GameState;
use crate::view::View;

pub mod placeholder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    /// A decimal string, read as `Fx`.
    Fx,
    /// A JSON integer.
    Int,
}

pub struct ParamDef {
    pub name: &'static str,
    pub kind: ParamKind,
    /// Inclusive bounds, written as the param is (`"0"`, `"1.5"`), if any.
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

/// Every module, in play order.
pub static REGISTRY: &[&ModuleDef] = &[&placeholder::DEF];

/// A module's rules, compiled from its config entry.
pub trait Module: Send + Sync {
    /// Sets the module's part of a new game's state.
    fn init(&self, g: &mut GameState);
    /// One in-game day.
    fn tick(&self, g: &mut GameState);
    /// `None` if the action isn't this module's. Must check legality before
    /// changing anything: an `Err` leaves the state untouched.
    fn apply(&self, g: &mut GameState, a: &Action) -> Option<Result<(), String>>;
    /// Adds this module's display data (never read by the rules).
    fn view(&self, g: &GameState, v: &mut View);
}

pub(crate) fn build(id: &str, mc: &ModuleConfig) -> Box<dyn Module> {
    match id {
        "placeholder" => Box::new(placeholder::Placeholder::new(mc)),
        _ => unreachable!("Config::check refuses unknown module {id:?}"),
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
