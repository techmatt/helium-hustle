//! The Projects panel and the Stats panel's live rewards: each project's
//! unlock condition and rewards as text, written here from the row so the
//! client holds no rules (display only: no rule reads it).

use serde::Serialize;
use std::collections::BTreeMap;

use super::{names, Text};
use crate::content::{content, Amounts, Condition, Project, ProjectReward, Requirement};
use crate::fx::Fx;
use crate::modules::*;
use crate::rules::Rules;
use crate::state::GameState;

#[derive(Debug, Clone, Serialize)]
pub struct ProjectView {
    pub id: Text,
    pub name: Text,
    pub description: Text,
    /// `run`, `timeline` or `forever`.
    pub scope: Text,
    pub unlocked: bool,
    pub completed: bool,
    /// Completions so far in its scope.
    pub completions: u32,
    pub repeatable: bool,
    /// The next completion's costs.
    pub costs: Amounts,
    pub progress: BTreeMap<String, Fx>,
    pub rates: BTreeMap<String, Fx>,
    /// The most each rate may be (per resource, per day).
    pub max_rates: BTreeMap<String, Fx>,
    /// Days to the next completion at today's rates; `None` while a cost
    /// still owed has no rate.
    pub days_to_done: Option<u64>,
    /// What opens it.
    pub unlock_text: String,
    /// What it gives, one line per reward.
    pub reward_text: Vec<String>,
}

pub(super) fn projects(r: &Rules, g: &GameState) -> Vec<ProjectView> {
    content()
        .projects
        .iter()
        .map(|p| ProjectView {
            id: &p.id,
            name: &p.name,
            description: &p.description,
            scope: p.scope.name(),
            unlocked: projects::unlocked(r, g, p),
            completed: projects::completed(g, &p.id),
            completions: projects::completions(g, p),
            repeatable: p.repeat.is_some(),
            costs: projects::costs(g, p),
            progress: projects::progress(g, p).cloned().unwrap_or_default(),
            rates: g.run.project_rates.get(&p.id).cloned().unwrap_or_default(),
            max_rates: p.costs.keys().map(|res| (res.clone(), projects::max_rate(g, p, res))).collect(),
            days_to_done: projects::days_to_done(g, p),
            unlock_text: condition_text(&p.unlock),
            reward_text: reward_text(p),
        })
        .collect()
}

/// Every live project reward, as `Project: what it gives` (content order).
pub(super) fn live_rewards(g: &GameState) -> Vec<String> {
    content()
        .projects
        .iter()
        .filter(|p| projects::completions(g, p) > 0)
        .flat_map(|p| reward_text(p).into_iter().map(move |t| format!("{}: {t}", p.name)))
        .collect()
}

/// A number to two places, trailing zeros dropped.
pub(super) fn short(x: Fx) -> String {
    let full = x.to_string();
    match full.split_once('.') {
        Some((whole, frac)) => {
            let frac = frac[..frac.len().min(2)].trim_end_matches('0');
            if frac.is_empty() { whole.to_string() } else { format!("{whole}.{frac}") }
        }
        None => full,
    }
}

fn name(id: &str) -> String {
    names().get(id).map_or_else(|| id.to_string(), |n| n.to_string())
}

/// A modifier key as words: `land_cost_mult` is "Land cost".
fn modifier_name(key: &str) -> String {
    let words = key.trim_end_matches("_mult").replace('_', " ");
    let mut cs = words.chars();
    cs.next().map_or_else(String::new, |f| f.to_uppercase().chain(cs).collect())
}

/// What a project gives: its rewards, then each building that requires it.
fn reward_text(p: &Project) -> Vec<String> {
    let mut out: Vec<String> = p
        .rewards
        .iter()
        .map(|reward| match reward {
            ProjectReward::StartingBuildings { buildings } => {
                let list: Vec<String> = buildings.iter().map(|(b, n)| format!("{n} {}", name(b))).collect();
                format!("Every run starts with {} free", list.join(" and "))
            }
            ProjectReward::Modifier { key, value } => format!("{} ×{}", modifier_name(key), short(*value)),
            ProjectReward::SetFlag { text, .. } => text.clone(),
            ProjectReward::BoredomModifiers { base_rate_mult, command_boredom } => {
                let cmds: Vec<String> =
                    command_boredom.iter().map(|(c, v)| format!("{} {}", name(c), short(*v))).collect();
                format!("Boredom growth ×{}; boredom per run of {}", short(*base_rate_mult), cmds.join(", "))
            }
            ProjectReward::ResearchDiscount { mult } => {
                format!("Research bought in an earlier run costs ×{}", short(*mult))
            }
        })
        .collect();
    for b in &content().buildings {
        if matches!(&b.requires, Requirement::Project { project } if *project == p.id) {
            out.push(format!("Opens {}", b.name));
        }
    }
    out
}

/// A condition as words (a project's unlock; display only).
pub fn condition_text(c: &Condition) -> String {
    let n = short;
    match c {
        Condition::BuildingOwned { building, count } => format!("Own {count} {}", name(building)),
        Condition::ResourceCumulative { resource, amount } => format!("Make {} {} this run", n(*amount), name(resource)),
        Condition::ShipmentCompleted { count } => format!("Ship {count} times this run"),
        Condition::ResearchCompleted { research } => format!("Research {}", name(research)),
        Condition::Immediate => "Open from the start".to_string(),
        Condition::Never => "Not available".to_string(),
        Condition::AllOf { sub_objectives } => {
            sub_objectives.iter().map(|s| s.label.as_str()).collect::<Vec<_>>().join("; ")
        }
        Condition::IdeologyRankAny { rank } => format!("Reach rank {rank} on any ideology axis"),
        Condition::IdeologyRank { axis, rank } => {
            format!("Reach rank {rank} on the {} axis (best in the career)", modifier_name(axis))
        }
        Condition::ProjectCompletedAny { scope } => format!("Complete any {} project", scope.name()),
        Condition::ProjectEnabled { project } => {
            let by = content().events.iter().find(|e| {
                e.unlocks.iter().any(|u| matches!(u, crate::content::Unlock::EnableProject { project: x } if x == project))
            });
            match by {
                Some(e) if !e.title.is_empty() => format!("Opened by {}", e.title),
                _ => "Opened by an event not in the game yet".to_string(),
            }
        }
        Condition::QuestActive { quest } => format!("Reach the quest {}", name(quest)),
        Condition::FlagSet { flag } => match content().commands.iter().find(|c| c.sets_flag.as_deref() == Some(flag)) {
            Some(cmd) => format!("Use {} once", cmd.name),
            None => format!("Needs {flag}"),
        },
        Condition::DaysSurvived { days } => format!("Last {days} days in a run"),
        Condition::CreditsEarned { amount } => format!("Earn {} credits this run", n(*amount)),
        Condition::Retirements { count } => format!("Retire {count} times"),
        Condition::AdversaryPresent { adversary } => format!("Draw the {adversary} in"),
        Condition::LaunchCredits { amount } => format!("Earn {} credits from one launch", n(*amount)),
        Condition::ResearchCluster { min_items } => {
            format!("Research every item of one cluster of {min_items} or more in one run")
        }
        Condition::QuietDays { days } => format!("{days} days without an order"),
        Condition::EarthScience { amount } => format!("{} Earth science this timeline", n(*amount)),
        Condition::EarthTech { tech } => {
            let t = content().earth_tech.iter().find(|t| &t.id == tech).map_or(tech.as_str(), |t| t.name.as_str());
            format!("Hold {t}")
        }
        Condition::TimelinesEnded { count } => format!("End {count} timelines"),
        Condition::SwarmRepelled => "Repel the swarm".to_string(),
        Condition::VaultHolds { resource, amount } => format!("The vault holds {} {}", n(*amount), name(resource)),
        Condition::SuperTimeline => "In a super timeline".to_string(),
        Condition::ProjectCompleted { project } => format!("Complete {}", name(project)),
        Condition::Victory => "Victory".to_string(),
    }
}
