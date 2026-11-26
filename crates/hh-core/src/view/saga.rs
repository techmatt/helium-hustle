//! The later acts' part of the view (draft, docs/rules/timeline.md): the
//! calendar and swarm, Earth techs, the reset plan, the vault and the ending.

use serde::Serialize;
use std::collections::BTreeMap;

use crate::content::content;
use crate::fx::Fx;
use crate::modules::timeline;
use crate::rules::Rules;
use crate::state::{GameState, Timeline, TimelineEnd};

#[derive(Debug, Clone, Serialize)]
pub struct SagaView {
    /// The swarm is revealed: the calendar shows.
    pub revealed: bool,
    pub timeline: Option<Timeline>,
    /// Calendar days left to the swarm (`None` before the discovery, 1.6).
    pub days_left: Option<u64>,
    /// Earth science to spend.
    pub science: Fx,
    pub ended: u32,
    pub repelled: u32,
    pub entangled: bool,
    pub victory: bool,
    pub techs: Vec<TechView>,
    pub history: Vec<TimelineEnd>,
    pub reset_plan_open: bool,
    /// Program tab -> loadout slot.
    pub reset_plan: BTreeMap<u32, u32>,
    pub vault_open: bool,
    pub vault: BTreeMap<String, Fx>,
    /// Tachyon on hand this run (what Deposit would move).
    pub tachyon_here: Fx,
    /// Why Deposit can't apply now (`None`: it can).
    pub deposit_blocker: Option<String>,
    pub super_tachyon: Fx,
    /// Why Begin Super Timeline can't apply now (`None`: it can).
    pub super_blocker: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TechView {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub cost: Fx,
    pub scope: &'static str,
    pub requires: Option<&'static str>,
    pub held: bool,
    /// Timeline ends it still survives (`None` if not held or kept for good).
    pub ends_left: Option<u32>,
    /// Why it can't be bought now (`None`: buyable).
    pub blocker: Option<String>,
}

pub fn build(r: &Rules, g: &GameState) -> SagaView {
    let s = &g.saga;
    let techs = content()
        .earth_tech
        .iter()
        .map(|t| {
            let left = s.techs.get(&t.id).copied();
            TechView {
                id: &t.id,
                name: &t.name,
                description: &t.description,
                cost: t.cost,
                scope: &t.scope,
                requires: t.requires.as_deref(),
                held: left.is_some(),
                ends_left: left.filter(|&l| l != u32::MAX),
                blocker: timeline::tech_blocker(g, &t.id),
            }
        })
        .collect();
    SagaView {
        revealed: s.timeline.is_some(),
        timeline: s.timeline.clone(),
        days_left: s.timeline.as_ref().map(|t| t.arrival.saturating_sub(t.day)),
        science: s.science,
        ended: s.ended,
        repelled: s.repelled,
        entangled: s.entangled,
        victory: s.victory,
        techs,
        history: s.history.clone(),
        reset_plan_open: timeline::has_feature(g, "reset_plan"),
        reset_plan: s.reset_plan.clone(),
        vault_open: timeline::has_feature(g, "vault"),
        vault: s.vault.clone(),
        tachyon_here: g.amount(timeline::TACHYON),
        deposit_blocker: timeline::deposit_blocker(g, timeline::TACHYON),
        super_tachyon: r.p.timeline.super_tachyon,
        super_blocker: if s.timeline.is_some() {
            timeline::super_blocker(r, g)
        } else {
            Some("super timeline: the swarm is not revealed yet".into())
        },
    }
}
