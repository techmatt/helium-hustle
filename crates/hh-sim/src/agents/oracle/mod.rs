//! `oracle`: the ceiling (docs/agents.md, *The oracle*). A two-level rollout
//! planner over the author, which plays every look. The oracle tries goals on
//! copies of the game (its privilege: [`Player::fork`], the future included)
//! and keeps what scores best:
//!
//! - **Strategy** (each new run, and every `replan` ticks): a bundle of
//!   standing goals ([`strategies`], a fixed table: the author's own; trade,
//!   i.e. Trade Promotion research, the market program and a different good
//!   on each pad, with the main program run last; an ideology axis funded;
//!   Dream held against boredom; Foundation Grant pushed; all research), each
//!   played for `long` ticks, through retirements.
//! - **Tactics** (every `every` ticks, and once a target is bought): within
//!   the strategy, one change each to the building it saves for, the research
//!   it buys first (when the strategy's is done), the project it pushes (when
//!   the strategy names none), or hoarding; screened over a third of
//!   `horizon`, the best `width` played over all of it, the two best changes
//!   together (a two-wide beam). A change must beat the current goals by
//!   `margin` per mille.
//!
//! **Score**, in credits: credits earned in the rollout; plus the closing
//! income of the run under way at its end (over the later of the rollout's
//! second half and that run's start) times the days that run has left (room
//! to the boredom cap at its closing growth, at most `tail`); plus the
//! profile's weights ([`Profile`]) on what the career
//! keeps (ideology best scores, Long-Term project progress and completions,
//! building types, commands, quests and sub-objectives, achievements),
//! research this run, each open milestone met (more the sooner) and each
//! open milestone's progress squared ([`super::progress`]: the pull grows as a goal
//! nears). The milestones are the career's (the CLI's `--milestones`).
//!
//! **Runs.** Every `RETIRE_EVERY` (100) ticks from day `min_run`, the oracle plays the
//! run out on a copy and retires once its average credits a day so far is
//! the best the run will see (the prestige rule: a later day's average beats
//! it by no more than `retire_margin` per mille), except while holding
//! boredom for the Open Horizons quest's ten years. The author also retires
//! a run that has bought nothing for `stale_days`; holding boredom with Dream
//! it doesn't, so the oracle does, the ten years again excepted.
//!
//! Files: `score.rs` (profiles, rollouts, the score), `strategy.rs` (the
//! strategy table, tactics), `retire.rs` (the retirement rule); this one the
//! knobs, the real game's milestones as its looks see them, and each look.

mod retire;
mod score;
mod strategy;

use std::collections::BTreeSet;

use hh_core::{Action, Fx, GameState};

use super::author::{Author, Directives};
use super::{Agent, Knobs, Player};
use crate::milestones::{Milestones, Tracker, When};

pub use score::{Profile, Weights};
pub use strategy::{campaign, strategies, strategy_index, Strategy};

pub const KNOBS: &[&str] = &[
    "profile", "every", "horizon", "long", "replan", "roll_gap", "tail", "margin", "max_gap", "stale_days", "beam", "width",
    "hoard_at", "strategy", "retire", "min_run", "retire_margin", "early",
    "first_margin", "ideology",
];

/// The Open Horizons quest and its ten-year sub-objective.
const HORIZON: &str = "qhorizon_open_horizons";
const SURVIVE: &str = "survive_10_years";

/// Ticks between retirement checks (each plays the run out on a copy).
const RETIRE_EVERY: u64 = 100;

/// What the oracle has seen of the real game at its looks: credits earned by
/// tick and goods launched, this run (for the history milestones).
///
/// Not `milestones::Tracker`, which judges the same milestones differently: a rate
/// here is any stretch of at least `days` between looks (Tracker: an exact window of
/// daily samples), and a good counts as launched when its saturation rises (Tracker:
/// revenue rises or a pad starts cooling down). Merging them changes the oracle's play.
#[derive(Debug, Clone, Default)]
struct Looks {
    run: u32,
    earned: Vec<(u64, Fx)>,
    saturation: Vec<(String, Fx)>,
    shipped: BTreeSet<String>,
}

impl Looks {
    fn look(&mut self, g: &GameState) {
        if g.career.run_number != self.run {
            *self = Looks { run: g.career.run_number, ..Default::default() };
        }
        self.earned.push((g.tick, g.run.stats.credits_earned));
        for (good, m) in &g.run.markets {
            let before = self.saturation.iter().find(|s| s.0 == *good).map_or(Fx::ZERO, |s| s.1);
            if m.saturation > before {
                self.shipped.insert(good.clone());
            }
        }
        self.saturation = g.run.markets.iter().map(|(k, m)| (k.clone(), m.saturation)).collect();
    }

    /// Whether a history milestone held over some stretch it saw.
    fn met(&self, w: &When) -> bool {
        match w {
            When::CreditRate { per_day, days } => {
                let need = |dt: u64| per_day.mul_int(dt as i64);
                self.earned.iter().enumerate().any(|(i, &(t1, e1))| {
                    self.earned[..i].iter().rev().find(|(t0, _)| t1 - t0 >= *days).is_some_and(|&(t0, e0)| e1 - e0 >= need(t1 - t0))
                })
            }
            When::GoodsShipped { count } => self.shipped.len() as u32 >= *count,
            When::AllOf { of } => of.iter().all(|x| self.met(x)),
            _ => false,
        }
    }
}

pub struct Oracle {
    base: Author,
    w: Weights,
    pub profile: Profile,
    /// The milestones it scores (the career's).
    ms: Milestones,
    every: u64,
    horizon: u64,
    long: u64,
    replan: u64,
    roll_gap: u64,
    tail: i64,
    margin: i64,
    stale_days: u64,
    beam: bool,
    width: usize,
    hoard_at: Fx,
    /// One strategy only (its index in [`strategies`]), or -1 to search.
    pin: i64,
    /// Try retiring every `RETIRE_EVERY` ticks, from day `min_run` of a run.
    retire: bool,
    min_run: u64,
    /// It retires once no later day of the run (played out) beats its
    /// average credits a day so far by more than this, per mille.
    retire_margin: i64,
    /// The same before the career's first retirement, for a profile that
    /// scores milestones (the first retirement is one).
    first_margin: i64,
    /// The strategy in force, and the ticks of the next tactics and strategy plans.
    strategy: Directives,
    strategy_label: String,
    next_plan: u64,
    next_strategy: u64,
    next_retire: u64,
    run: u32,
    /// Buildings built this run at the last look, and the day that rose.
    built: u32,
    built_day: u64,
    /// The target's count when it was set: once it rises, the goal is met.
    target_owned: u32,
    /// Milestones met in the real game (as far as its looks show).
    done: BTreeSet<String>,
    seen: Looks,
    plans: u32,
    rollouts: u64,
    note: String,
}

impl Oracle {
    /// An oracle scoring milestones `ms`. A `strategy` knob past the table's
    /// end is refused.
    pub fn new(k: &Knobs, ms: &Milestones) -> Result<Self, String> {
        let pin = k.get("strategy", -1);
        let rows = strategies().len();
        if usize::try_from(pin).is_ok_and(|i| i >= rows) {
            return Err(format!("agent oracle: strategy={pin} is past the table's {rows} rows (-1 searches them all)"));
        }
        let profile = Profile::from_knob(k.get("profile", 0))?;
        let mut w = profile.weights();
        w.early = k.get("early", w.early).max(0);
        w.ideology = k.get("ideology", w.ideology).max(0);
        let stale_days = k.get("stale_days", 300).max(1) as u64;
        let mut base = Author::new(&Knobs::default());
        base.set_gaps(1, k.get("max_gap", 10).max(1) as u64);
        base.set_stale_days(stale_days);
        Ok(Oracle {
            base,
            w,
            profile,
            ms: ms.clone(),
            every: k.get("every", 50).max(1) as u64,
            // 600 tried against 300 and 900 (dev seeds 1-3, 4 runs): 300
            // steered off slow chains (circuits before a second Data Center).
            horizon: k.get("horizon", 600).max(20) as u64,
            long: k.get("long", 2000).max(20) as u64,
            replan: k.get("replan", 500).max(1) as u64,
            // Rollouts look as often as the real game does, so keeping the
            // goals plays out in a rollout just as it will for real.
            roll_gap: k.get("roll_gap", k.get("max_gap", 10)).max(1) as u64,
            tail: k.get("tail", 400).max(0),
            margin: k.get("margin", 20).max(0),
            stale_days,
            beam: k.get("beam", 1) != 0,
            width: k.get("width", 5).max(1) as usize,
            hoard_at: Fx::from_int(k.get("hoard_at", 5000).max(0)),
            pin,
            retire: k.get("retire", 1) != 0,
            // 400 let a young run retire at day 625-730 on 5,000 credits (the
            // played-out run, its goals fixed, underestimates its growth).
            min_run: k.get("min_run", 1000).max(0) as u64,
            retire_margin: k.get("retire_margin", 0).max(0),
            first_margin: k.get("first_margin", 0).max(0),
            strategy: Directives::default(),
            strategy_label: "author".into(),
            next_plan: 0,
            next_strategy: 0,
            next_retire: 0,
            run: 0,
            built: 0,
            built_day: 0,
            target_owned: 0,
            done: BTreeSet::new(),
            seen: Looks::default(),
            plans: 0,
            rollouts: 0,
            note: String::new(),
        })
    }

    /// Records the milestones the real game has met, as far as it can see.
    fn observe(&mut self, p: &Player) {
        let g = p.state();
        self.seen.look(g);
        // A tracker that saw no ticks answers the state milestones exactly
        // and the history ones never; `seen` answers those.
        let blank = Tracker::new(&self.ms);
        let met: Vec<String> = self
            .ms
            .milestones
            .iter()
            .filter(|m| !self.done.contains(&m.id) && (self.seen.met(&m.when) || self.blank_met(&blank, &m.when, p)))
            .map(|m| m.id.clone())
            .collect();
        self.done.extend(met);
    }

    fn blank_met(&self, t: &Tracker, w: &When, p: &Player) -> bool {
        match w {
            When::CreditRate { .. } | When::GoodsShipped { .. } => false,
            When::AllOf { of } => of.iter().all(|x| self.blank_met(t, x, p) || self.seen.met(x)),
            _ => t.met(w, p.session()),
        }
    }
}

impl Agent for Oracle {
    fn act(&mut self, p: &mut Player) {
        let (tick, run, day, built) = {
            let g = p.state();
            (g.tick, g.career.run_number, g.run.day, g.run.stats.buildings_built)
        };
        let fresh = run != self.run;
        if fresh {
            self.run = run;
            self.base.dir = Directives::default();
            self.next_plan = tick;
            self.next_strategy = tick;
        }
        if fresh || built != self.built {
            self.built = built;
            self.built_day = day;
        }
        self.observe(p);
        // A stale run held by Dream retires, unless it is surviving its ten years.
        let stale = day.saturating_sub(self.built_day) >= self.stale_days;
        if self.base.dir.sustain && stale && p.view().can_retire && !retire::surviving(p.state()) {
            let _ = p.apply(&Action::Retire);
            return;
        }
        if tick >= self.next_retire {
            self.next_retire = tick + RETIRE_EVERY;
            if self.maybe_retire(p) {
                return;
            }
        }
        if tick >= self.next_strategy {
            self.choose_strategy(p);
            self.next_strategy = tick + self.replan;
        }
        // A target bought: its goal is met, plan again now.
        if let Some(t) = self.base.dir.target.clone() {
            if p.state().owned(&t) > self.target_owned {
                self.base.dir.target = None;
                self.next_plan = tick;
            }
        }
        if tick >= self.next_plan {
            self.plan(p);
            self.next_plan = tick + self.every;
        }
        self.base.act(p);
    }

    /// The author's own gap: plans fall due at its next look, so the real
    /// game looks when a rollout of the same goals does.
    fn next_look(&mut self, p: &mut Player) -> u64 {
        self.base.next_look(p).max(1)
    }

    fn note(&self) -> String {
        format!("{} [{} plans, {} rollouts] {}", self.profile.name(), self.plans, self.rollouts, self.note)
    }
}
