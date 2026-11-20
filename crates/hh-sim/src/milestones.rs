//! Arc 1 milestones as measurable objectives (`configs/sim/milestones.json`,
//! docs/principles.md). The tracker reads the state after each tick: that is
//! observation for scoring, never an agent's input.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use hh_core::modules::economy;
use hh_core::{content, Fx, Rules, Session};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum When {
    /// Capability: at least `count` of a building owned in the current run.
    Building { building: String, count: u32 },
    /// Pipeline output: shipments launched in the career.
    Shipments { count: u32 },
    Retirements { count: u32 },
    /// Scaling: credits earned in the run over the trailing `days`, at least
    /// `per_day` a day on average.
    CreditRate { per_day: Fx, days: u64 },
    /// Stockpile, uncapped resources only.
    Stockpile { resource: String, at_least: Fx },
    /// Distinct goods launched in one run.
    GoodsShipped { count: u32 },
    /// Each group has a command used at least once in the career.
    CommandsUsed { groups: Vec<Vec<String>> },
    /// Bought in the current run.
    Research { research: String },
    /// Any project of the tier completed (persistent: in the career).
    ProjectCompleted { tier: String },
    /// The career's best rank on any axis.
    IdeologyRank { rank: i64 },
    Quest { quest: String },
    AllOf { of: Vec<When> },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub runs: [u32; 2],
    #[serde(default)]
    pub ticks: Option<[u64; 2]>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Milestone {
    pub id: String,
    pub name: String,
    pub says: String,
    pub when: When,
    pub target: Target,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run1 {
    /// Run-1 length bands by player name (a spec's `name=agent`).
    pub ticks: BTreeMap<String, [u64; 2]>,
    /// Run-1 real minutes under one human time model.
    pub minutes: Run1Minutes,
}

/// The model is the judge's spec's first human model
/// (`experiments/balance.json` `humans`, its one home).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run1Minutes {
    pub band: [u64; 2],
    /// The players it's read for (by name).
    pub players: Vec<String>,
}

/// The judge's own thresholds (`hh-sim balance`).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Judge {
    /// A milestone reached by at most this percent of the seeds is late.
    pub late_percent: u64,
}

/// When `vs-human` counts the same purchase as parted in time: more than
/// the larger of `when_ticks` and `when_percent` of the best player's tick.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VsHuman {
    pub when_ticks: u64,
    pub when_percent: u64,
}

/// The judge's cells (players, seeds, human model) are `experiments/balance.json`'s.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Milestones {
    #[serde(rename = "_doc")]
    pub doc: String,
    pub milestones: Vec<Milestone>,
    pub run1: Run1,
    pub judge: Judge,
    pub vs_human: VsHuman,
}

impl Milestones {
    pub fn from_json(text: &str) -> Result<Milestones, String> {
        let m: Milestones = serde_json::from_str(text).map_err(|e| format!("milestones: {e}"))?;
        let mut ids = BTreeSet::new();
        for x in &m.milestones {
            if !ids.insert(x.id.clone()) {
                return Err(format!("milestones: duplicate id {}", x.id));
            }
            check(&x.when).map_err(|e| format!("milestone {}: {e}", x.id))?;
            if x.target.runs[0] < 1 || x.target.runs[0] > x.target.runs[1] {
                return Err(format!("milestone {}: bad run window {:?}", x.id, x.target.runs));
            }
        }
        Ok(m)
    }
}

/// Checks ids against content and the design rules (no stockpile threshold
/// on a capped resource).
fn check(w: &When) -> Result<(), String> {
    let c = content();
    match w {
        When::Building { building, .. } if c.building(building).is_none() => Err(format!("unknown building {building}")),
        When::Stockpile { resource, .. } => match c.resources.iter().find(|r| r.id == *resource) {
            None => Err(format!("unknown resource {resource}")),
            Some(r) if r.cap.is_some() => {
                Err(format!("stockpile threshold on {resource}, a capped resource (design rules: use a rate)"))
            }
            Some(_) => Ok(()),
        },
        When::CommandsUsed { groups } => {
            for id in groups.iter().flatten() {
                if !c.commands.iter().any(|x| x.id == *id) {
                    return Err(format!("unknown command {id}"));
                }
            }
            Ok(())
        }
        When::Research { research } if !c.research.iter().any(|x| x.id == *research) => {
            Err(format!("unknown research {research}"))
        }
        When::Quest { quest } if !c.events.iter().any(|x| x.id == *quest) => Err(format!("unknown quest {quest}")),
        When::ProjectCompleted { tier } if !c.projects.iter().any(|p| p.tier == *tier) => {
            Err(format!("unknown project tier {tier}"))
        }
        When::CreditRate { days: 0, .. } => Err("credit_rate days must be at least 1".into()),
        When::AllOf { of } => of.iter().try_for_each(check),
        _ => Ok(()),
    }
}

/// When a milestone was first met.
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Hit {
    pub tick: u64,
    /// The run it was met in (1-based).
    pub run: u32,
    /// Ticks into that run.
    pub day: u64,
}

/// Run-scoped observations the milestones need, updated each tick.
#[derive(Default)]
pub struct Tracker {
    run: u32,
    /// Credits earned this run at the end of each recent run day (`days`).
    earned: VecDeque<Fx>,
    days: VecDeque<u64>,
    shipped: BTreeSet<String>,
    revenue: BTreeMap<String, Fx>,
    /// Each pad's good and cooldown before the tick.
    pads: Vec<(String, u32)>,
    window: usize,
    best_rate: Fx,
}

impl Tracker {
    pub fn new(ms: &Milestones) -> Tracker {
        fn days(w: &When) -> u64 {
            match w {
                When::CreditRate { days, .. } => *days,
                When::AllOf { of } => of.iter().map(days).max().unwrap_or(0),
                _ => 0,
            }
        }
        let window = ms.milestones.iter().map(|m| days(&m.when)).max().unwrap_or(0) as usize;
        Tracker { window, ..Default::default() }
    }

    /// Reads markets and pads before a tick: a launch shows as its good's
    /// revenue rising, or (on a speculator burst's day, which zeroes revenue)
    /// as a ready pad cooling down after the tick.
    pub fn before(&mut self, s: &Session) {
        self.revenue = s.state.run.markets.iter().map(|(g, m)| (g.clone(), m.revenue)).collect();
        self.pads = s.state.run.pads.iter().map(|p| (p.good.clone(), p.cooldown)).collect();
    }

    /// Reads the state after a tick (or after a look's actions).
    pub fn after(&mut self, s: &Session) {
        let g = &s.state;
        if g.career.run_number != self.run {
            self.run = g.career.run_number;
            self.earned.clear();
            self.days.clear();
            self.shipped.clear();
            return;
        }
        for (good, m) in &g.run.markets {
            if m.revenue > self.revenue.get(good).copied().unwrap_or(Fx::ZERO) {
                self.shipped.insert(good.clone());
            }
        }
        for (i, (good, cooldown)) in self.pads.iter().enumerate() {
            if *cooldown == 0 && g.run.pads.get(i).is_some_and(|p| p.cooldown > 0) {
                self.shipped.insert(good.clone());
            }
        }
        // One sample a run day: a look between ticks doesn't add one.
        if self.days.back() == Some(&g.run.day) {
            *self.earned.back_mut().expect("paired") = g.run.stats.credits_earned;
            return;
        }
        self.days.push_back(g.run.day);
        self.earned.push_back(g.run.stats.credits_earned);
        while self.earned.len() > self.window + 1 {
            self.days.pop_front();
            self.earned.pop_front();
        }
        if self.window > 0 && self.earned.len() > self.window {
            let rate = (self.earned[self.window] - self.earned[0]).mul_ratio(1, self.window as i64);
            self.best_rate = self.best_rate.max(rate);
        }
    }

    /// The run's best credits earned a day over the milestones' longest
    /// window so far, and starts the next run's at zero.
    pub fn take_best_rate(&mut self) -> Fx {
        std::mem::replace(&mut self.best_rate, Fx::ZERO)
    }

    /// Goods seen shipped this run.
    pub fn shipped(&self) -> usize {
        self.shipped.len()
    }

    pub fn met(&self, w: &When, s: &Session) -> bool {
        let g = &s.state;
        match w {
            When::Building { building, count } => g.owned(building) >= *count,
            When::Shipments { count } => g.career.lifetime.shipments + g.run.stats.shipments >= *count,
            When::Retirements { count } => g.career.retirements >= *count,
            When::CreditRate { per_day, days } => {
                let d = *days as usize;
                self.earned.len() > d && {
                    let now = self.earned[self.earned.len() - 1];
                    let then = self.earned[self.earned.len() - 1 - d];
                    now - then >= per_day.mul_int(*days as i64)
                }
            }
            When::Stockpile { resource, at_least } => g.amount(resource) >= *at_least,
            When::GoodsShipped { count } => self.shipped.len() as u32 >= *count,
            When::CommandsUsed { groups } => {
                groups.iter().all(|grp| grp.iter().any(|c| g.career.used_commands.contains(c)))
            }
            When::Research { research } => g.run.research.contains(research),
            When::ProjectCompleted { tier } => content()
                .projects
                .iter()
                .filter(|p| p.tier == *tier)
                .any(|p| g.career.completed_projects.contains(&p.id) || g.run.completed_projects.contains(&p.id)),
            When::IdeologyRank { rank } => g.career.max_ideology_ranks.values().any(|r| r >= rank),
            When::Quest { quest } => g.career.completed_quests.contains(quest),
            When::AllOf { of } => of.iter().all(|x| self.met(x, s)),
        }
    }
}

/// Capped resources other than boredom, with their caps now.
pub fn capped(r: &Rules, s: &Session) -> Vec<(String, Fx)> {
    content()
        .resources
        .iter()
        .filter(|res| res.id != economy::BOREDOM)
        .filter_map(|res| economy::cap(r, &s.state, &res.id).map(|c| (res.id.clone(), c)))
        .collect()
}
