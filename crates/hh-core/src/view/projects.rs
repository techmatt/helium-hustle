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
    /// Its scope icon: `legacy`, `eternal` or none (`run`; `keep.rs`).
    pub keep: Option<Text>,
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
    /// What each set rate actually gets: the last day's take where the stock
    /// fell short of it, else the set rate (keys as `rates`).
    pub actual_rates: BTreeMap<String, Fx>,
    /// Days to the next completion at the actual rates; `None` while a cost
    /// still owed gets nothing.
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
            keep: super::keep::project(g, p.scope),
            unlocked: projects::unlocked(r, g, p),
            completed: projects::completed(g, &p.id),
            completions: projects::completions(g, p),
            repeatable: p.repeat.is_some(),
            costs: projects::costs(g, p),
            progress: projects::progress(g, p).cloned().unwrap_or_default(),
            rates: g.run.project_rates.get(&p.id).cloned().unwrap_or_default(),
            max_rates: p.costs.keys().map(|res| (res.clone(), projects::max_rate(r, g, p, res))).collect(),
            actual_rates: g
                .run
                .project_rates
                .get(&p.id)
                .map(|m| m.keys().map(|res| (res.clone(), projects::actual_rate(g, p, res))).collect())
                .unwrap_or_default(),
            days_to_done: projects::days_to_done(g, p),
            unlock_text: condition_text(&p.unlock),
            reward_text: reward_text(p),
        })
        .collect()
}

/// One project completion this run (its popup and its Events line).
#[derive(Debug, Clone, Serialize)]
pub struct ProjectDoneView {
    pub id: Text,
    pub name: Text,
    pub day: u64,
    /// Which completion in its scope it was (1 for the first).
    pub completion: u32,
    /// What that completion cost in total.
    pub costs: Amounts,
    /// What it gives, one line per reward.
    pub reward_text: Vec<String>,
}

/// The project completions this run, oldest first, from the career log. Each one's
/// number is its scope's count less the same project's later completions this run;
/// its cost is the cost after the completions before it.
pub(super) fn project_done(g: &GameState) -> Vec<ProjectDoneView> {
    let run = g.career.run_number;
    // The log is in time order: this run's entries are its tail, back to another run's
    // entry or a timeline's end (a new timeline starts again at run 1).
    let start = g.career.log.iter().rposition(|e| e.run != run || e.kind == "timeline_end").map_or(0, |i| i + 1);
    let notes: Vec<_> = g.career.log[start..].iter().filter(|e| e.kind == "project").collect();
    notes
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            let p = content().project(&e.id)?;
            let later = notes[i + 1..].iter().filter(|x| x.id == e.id).count() as u32;
            let completion = projects::completions(g, p).saturating_sub(later).max(1);
            Some(ProjectDoneView {
                id: &p.id,
                name: &p.name,
                day: e.day,
                completion,
                costs: p.costs.keys().map(|res| (res.clone(), projects::cost_after(p, res, completion - 1))).collect(),
                reward_text: reward_text(p),
            })
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
                // "Name ×n" (×1 left out): content names are singular (134: "2 Solar Panel").
                let list: Vec<String> =
                    buildings.iter().map(|(b, n)| if *n == 1 { name(b) } else { format!("{} ×{n}", name(b)) }).collect();
                format!("Every run starts with these free: {}", list.join(", "))
            }
            ProjectReward::Modifier { key, value } => format!("{} ×{}", modifier_name(key), short(*value)),
            ProjectReward::Flag { text, .. } => text.clone(),
            ProjectReward::StartingResearch { research } => {
                let list: Vec<String> = research.iter().map(|x| name(x)).collect();
                format!("Every run starts with {} researched", list.join(" and "))
            }
            ProjectReward::BoredomModifiers { base_rate_mult, command_boredom } => {
                // The change, from the command's own boredom (Matt, 2026-11-27: "Cloud
                // Compute boredom 0.1 → 0.15", never read as relief).
                let cmds: Vec<String> = command_boredom
                    .iter()
                    .map(|(c, v)| {
                        let was = content().command(c).map_or(Fx::ZERO, |x| x.boredom);
                        format!("{} boredom {} → {}", name(c), short(was), short(*v))
                    })
                    .collect();
                format!("Boredom growth ×{}; {}", short(*base_rate_mult), cmds.join(", "))
            }
            ProjectReward::ResearchDiscount { mult, category } => match category {
                Some(cat) => format!("{cat} research bought in an earlier run costs ×{}", short(*mult)),
                None => format!("Research bought in an earlier run costs ×{}", short(*mult)),
            },
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
        Condition::EventNotFired { event } => {
            let t = content().events.iter().find(|e| &e.id == event).map_or(event.as_str(), |e| e.title.as_str());
            format!("Not in a run where {t} happened")
        }
        Condition::DaysSurvived { days } => format!("Last {days} days in a run"),
        Condition::CreditsEarned { amount } => format!("Earn {} credits this run", n(*amount)),
        Condition::Retirements { count } => format!("Retire {count} times"),
        Condition::AdversaryPresent { adversary } => format!("Draw the {adversary} in"),
        Condition::LaunchCredits { amount } => format!("Earn {} credits from one launch", n(*amount)),
        Condition::TradeCredits { amount } => format!("Earn {} credits from trade", n(*amount)),
        Condition::ResearchCount { count } => format!("Research {count} items in one run"),
        Condition::IdeologyRankEach { rank } => format!("Reach rank {rank} on every axis"),
        Condition::LaunchCargo { resource, amount } => format!("Launch {} {} at once", n(*amount), name(resource)),
        Condition::LaunchFull { resource } => format!("Launch a full pad of {}", name(resource)),
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
