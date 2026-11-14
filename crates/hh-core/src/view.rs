//! What a client shows: derived display data (prices, rates, caps, what is
//! affordable), so the UI never computes a rule. Never read by the rules, not
//! part of the state, the fingerprint or a replay.

use serde::Serialize;
use std::collections::BTreeMap;

use crate::action::Action;
use crate::fx::Fx;

#[derive(Debug, Clone, Serialize)]
pub struct ResourceView {
    pub id: String,
    pub name: String,
    pub amount: Fx,
    /// Most that can be held, if capped.
    pub cap: Option<Fx>,
    /// Expected change per day (before any random factor or cap).
    pub per_day: Fx,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildingView {
    pub id: String,
    pub name: String,
    pub owned: u32,
    /// Price of the next one.
    pub cost: BTreeMap<String, Fx>,
    pub affordable: bool,
    /// The action that buys one.
    pub buy: Action,
}

#[derive(Debug, Clone, Serialize)]
pub struct View {
    pub tick: u64,
    pub resources: Vec<ResourceView>,
    pub buildings: Vec<BuildingView>,
}
