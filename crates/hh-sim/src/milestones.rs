//! Arc 1 milestones as measurable objectives (`configs/sim/milestones.json`,
//! docs/principles.md). The tracker reads the state after each tick: that is
//! observation for scoring, never an agent's input.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use hh_core::modules::economy;
use hh_core::{content, Fx, GameState, Rules, Session};
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
    /// Any project of the scope completed.
    ProjectCompleted { scope: content::Scope },
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
    /// Checks switched off until the balance work rebases them (Matt, 2026-11-27): milestone
    /// ids, and `run1` for run 1's length bands (ticks and real minutes). Their lines are
    /// not written; the milestones are still tracked and reported.
    pub paused: Vec<String>,
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
        if let Some(p) = m.judge.paused.iter().find(|p| *p != "run1" && !ids.contains(*p)) {
            return Err(format!("milestones: judge.paused names {p}, neither a milestone nor run1"));
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
        When::ProjectCompleted { scope } if !c.projects.iter().any(|p| p.scope == *scope) => {
            Err(format!("no project of scope {}", scope.name()))
        }
        When::CreditRate { days: 0, .. } => Err("credit_rate days must be at least 1".into()),
        When::AllOf { of } => of.iter().try_for_each(check),
        _ => Ok(()),
    }
}

/// When a milestone was first met.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, Deserialize)]
pub struct Hit {
    pub tick: u64,
    /// The run it was met in (1-based).
    pub run: u32,
    /// Ticks into that run.
    pub day: u64,
}

/// Run-scoped observations the milestones need, updated each tick. Serializable: the web
/// client's live play record ([`crate::playrecord`]) keeps one across page loads.
#[derive(Debug, Default, Clone, serde::Serialize, Deserialize)]
pub struct Tracker {
    run: u32,
    /// Credits earned this run at the end of each recent run day (`days`).
    earned: VecDeque<Fx>,
    days: VecDeque<u64>,
    shipped: BTreeSet<String>,
    revenue: BTreeMap<String, Fx>,
    /// Each pad's good and cooldown before the tick.
    pads: Vec<(Option<String>, u32)>,
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
        self.before_state(&s.state);
    }

    /// [`Tracker::before`] on a bare state.
    pub fn before_state(&mut self, g: &GameState) {
        self.revenue = g.run.markets.iter().map(|(g, m)| (g.clone(), m.revenue)).collect();
        self.pads = g.run.pads.iter().map(|p| (p.good.clone(), p.cooldown)).collect();
    }

    /// Reads the state after a tick (or after a look's actions).
    pub fn after(&mut self, s: &Session) {
        self.after_state(&s.state);
    }

    /// [`Tracker::after`] on a bare state.
    pub fn after_state(&mut self, g: &GameState) {
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
            // A launched pad had cargo, so a good.
            let launched = *cooldown == 0 && g.run.pads.get(i).is_some_and(|p| p.cooldown > 0);
            if let Some(good) = good.as_ref().filter(|_| launched) {
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
        self.met_state(w, &s.state)
    }

    /// [`Tracker::met`] on a bare state.
    pub fn met_state(&self, w: &When, g: &GameState) -> bool {
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
            When::ProjectCompleted { scope } => content()
                .projects
                .iter()
                .filter(|p| p.scope == *scope)
                .any(|p| hh_core::modules::projects::completed(g, &p.id)),
            When::IdeologyRank { rank } => g.career.max_ideology_ranks.values().any(|r| r >= rank),
            When::Quest { quest } => g.career.completed_quests.contains(quest),
            When::AllOf { of } => of.iter().all(|x| self.met_state(x, g)),
        }
    }
}

/// The career log's latest retirement, chosen or forced.
pub fn last_retirement(g: &GameState) -> Option<&hh_core::state::LogEntry> {
    g.career.log.iter().rev().find(|e| e.kind == "retired" || e.kind == "forced_retirement")
}

/// When each milestone and long goal was first met, read as a career plays: one home
/// for the sim's measure ([`crate::career::Meter`]) and the web client's live play
/// record ([`crate::playrecord`]), so `vs-human --skilled` sets like beside like.
/// Call [`Marks::step`] after each tick and each round of actions, with
/// [`Tracker::before_state`] before each tick.
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct Marks {
    pub tracker: Tracker,
    /// Milestones by id.
    pub hits: BTreeMap<String, Hit>,
    /// Long goals (`project:<id>`, `rank5:<axis>`), ticks from `start_tick`
    /// ([`crate::career::GameResult::landmarks`]).
    pub landmarks: BTreeMap<String, Hit>,
    start_tick: u64,
    book_sizes: (usize, usize),
    /// Projects completed when the career started (a scenario's): no landmark while they
    /// stay in their book (134: they read as first completed on run 1's first day).
    at_start: BTreeSet<String>,
}

impl Marks {
    pub fn new(ms: &Milestones, g: &GameState) -> Marks {
        let mut tracker = Tracker::new(ms);
        tracker.after_state(g);
        let books = [&g.career.projects.completions, &g.career.forever_projects.completions];
        Marks {
            tracker,
            hits: BTreeMap::new(),
            landmarks: BTreeMap::new(),
            start_tick: g.tick,
            book_sizes: (books[0].len(), books[1].len()),
            at_start: books.iter().flat_map(|b| b.keys()).cloned().collect(),
        }
    }

    /// Records each milestone met now; returns the ids newly met. `ended`: the run (and
    /// its last day) a retirement closed in this step, if any. A milestone first seen then
    /// counts in that run (on its last day): a retirement, or a career-wide condition (a
    /// quest, lifetime shipments, a command used) the run's last tick met. A run-scoped one
    /// the new run already reset (a building count, a launch on that tick) is not seen
    /// (docs/principles.md).
    pub fn check(&mut self, ms: &Milestones, g: &GameState, ended: Option<(u32, u64)>) -> Vec<String> {
        let mut new = Vec::new();
        for m in &ms.milestones {
            if !self.hits.contains_key(&m.id) && self.tracker.met_state(&m.when, g) {
                let (run, day) = match (&m.when, ended) {
                    (_, Some(e)) => e,
                    (When::Retirements { .. }, None) => {
                        last_retirement(g).map_or((g.career.run_number, g.run.day), |e| (e.run, e.day))
                    }
                    _ => (g.career.run_number, g.run.day),
                };
                self.hits.insert(m.id.clone(), Hit { tick: g.tick, run, day });
                new.push(m.id.clone());
            }
        }
        new
    }

    /// Records the long goals first met by now; returns the keys newly met. `ended` as
    /// for [`Marks::check`]: one met on a retirement's tick counts in the ended run, on
    /// its last day (134: they went to the new run's day 0).
    pub fn landmarks(&mut self, r: &Rules, g: &GameState, ended: Option<(u32, u64)>) -> Vec<String> {
        let (run, day) = ended.unwrap_or((g.career.run_number, g.run.day));
        let hit = Hit { tick: g.tick - self.start_tick, run, day };
        let mut new = Vec::new();
        // Scan the books only when one changed size (a timeline's end empties one).
        let books = [&g.career.projects.completions, &g.career.forever_projects.completions];
        let sizes = (books[0].len(), books[1].len());
        if sizes != self.book_sizes {
            self.book_sizes = sizes;
            self.at_start.retain(|id| books.iter().any(|b| b.contains_key(id)));
            for id in books.iter().flat_map(|b| b.keys()) {
                let key = format!("project:{id}");
                if !self.at_start.contains(id) && !self.landmarks.contains_key(&key) {
                    self.landmarks.insert(key.clone(), hit);
                    new.push(key);
                }
            }
        }
        // On a retirement tick the run's ranks are the ended run's, before the gap's decay.
        let ended_ranks = ended.and(g.career.last_run.as_ref()).map(|l| &l.ideology_ranks);
        for axis in content::AXES {
            let key = format!("rank5:{axis}");
            if self.landmarks.contains_key(&key) {
                continue;
            }
            let rank = match ended_ranks {
                Some(ranks) => ranks.get(*axis).copied().unwrap_or(0),
                None => hh_core::modules::ideology::rank(r, g, axis),
            };
            if rank >= crate::career::LANDMARK_RANK {
                self.landmarks.insert(key.clone(), hit);
                new.push(key);
            }
        }
        new
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
