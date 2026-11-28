//! The shared modifier framework (docs/rules/README.md, *Modifiers*): a
//! modifier key's value is the product of every live source naming it
//! (earned achievements, completed quests' `modifier` unlocks, completed
//! projects, research done this run, career bonuses), 1 with none.
//! Derived on demand, never stored, so it can't go stale.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::content::{content, AchievementReward, Project, ProjectReward, ResearchEffect, Unlock};
use crate::fx::Fx;
use crate::rules::Rules;
use crate::state::GameState;

/// One key's possible sources, each list in content order (the order the product is
/// taken in, so it rounds as a scan of all content would).
#[derive(Default)]
struct Sources {
    achievements: Vec<(&'static str, Fx)>,
    quests: Vec<(&'static str, Fx)>,
    projects: Vec<(&'static Project, Fx)>,
    research: Vec<(&'static str, Fx)>,
}

/// Every key's sources, found once (content is fixed for the process): `modifier`
/// is asked many times a day (each cap, rate and price), and scanning every
/// achievement, quest unlock, project and research item for each ask was a large
/// share of a tick and a view (137).
fn sources(key: &str) -> Option<&'static Sources> {
    static INDEX: OnceLock<BTreeMap<&'static str, Sources>> = OnceLock::new();
    INDEX
        .get_or_init(|| {
            let c = content();
            let mut ix: BTreeMap<&'static str, Sources> = BTreeMap::new();
            for a in &c.achievements {
                if let AchievementReward::Modifier { key: k, value } = &a.reward {
                    ix.entry(k.as_str()).or_default().achievements.push((&a.id, *value));
                }
            }
            for e in &c.events {
                for u in &e.unlocks {
                    if let Unlock::Modifier { key: k, value } = u {
                        ix.entry(k.as_str()).or_default().quests.push((&e.id, *value));
                    }
                }
            }
            for p in &c.projects {
                for r in &p.rewards {
                    if let ProjectReward::Modifier { key: k, value } = r {
                        ix.entry(k.as_str()).or_default().projects.push((p, *value));
                    }
                }
            }
            for item in &c.research {
                if let Some(ResearchEffect::Modifier { key: k, value }) = &item.effect {
                    ix.entry(k.as_str()).or_default().research.push((&item.id, *value));
                }
            }
            ix
        })
        .get(key)
}

pub fn modifier(r: &Rules, g: &GameState, key: &str) -> Fx {
    // An unknown key would silently read 1: catch a typo at a call site in tests.
    debug_assert!(crate::content::MODIFIER_KEYS.contains(&key), "unknown modifier key {key:?}");
    let mut m = Fx::ONE;
    if let Some(s) = sources(key) {
        for &(id, value) in &s.achievements {
            if g.career.achievements.contains(id) {
                m = m * value;
            }
        }
        // A completed quest's `modifier` unlock (a side quest's permanent reward).
        for &(id, value) in &s.quests {
            if g.career.completed_quests.contains(id) {
                m = m * value;
            }
        }
        for &(p, value) in &s.projects {
            let n = crate::modules::projects::completions(g, p);
            if n > 0 {
                m = m * crate::modules::projects::stacked(value, n);
            }
        }
        for &(id, value) in &s.research {
            if g.run.research.contains(id) {
                m = m * value;
            }
        }
    }
    m = m * crate::modules::timeline::modifier(g, key);
    if key == "buy_power_mult" {
        m = m * crate::modules::career::buy_power_mult(r, g);
    }
    m
}
