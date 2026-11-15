//! `oracle`: the ceiling (docs/agents.md, *The oracle*). A two-level rollout
//! planner over the author, which plays every look. The oracle tries goals on
//! copies of the game (its privilege: [`Player::fork`], the future included)
//! and keeps what scores best:
//!
//! - **Strategy** (each new run, and every `replan` ticks): a bundle of
//!   standing goals ([`strategies`]: the author's own; trade, i.e. Trade
//!   Promotion research, the market program and a different good on each
//!   pad, with the main program run last; an ideology axis funded; Dream
//!   held against boredom; Foundation Grant pushed; all research), each
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
//! profile's weights ([`profile`]) on what the career
//! keeps (ideology best scores, Long-Term project progress and completions,
//! building types, commands, quests and sub-objectives, achievements),
//! research this run, each open milestone met (more the sooner) and each
//! open milestone's progress squared ([`progress`]: the pull grows as a goal
//! nears).
//!
//! **Runs.** At each strategy plan from day `min_run`, the oracle plays the
//! run out on a copy and retires once its average credits a day so far is
//! the best the run will see (the prestige rule: a later day's average beats
//! it by no more than `retire_margin` per mille), except while holding
//! boredom for the Open Horizons quest's ten years. The author also retires
//! a run that has bought nothing for `stale_days`; holding boredom with Dream
//! it doesn't, so the oracle does, the ten years again excepted.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use hh_core::modules::{boredom, events};
use hh_core::{content, Action, Fx, GameState, Session};

use super::author::{Author, Directives};
use super::{progress, Agent, Knobs, Player};
use crate::milestones::{Milestones, Tracker, When};

pub const KNOBS: &[&str] = &[
    "profile", "every", "horizon", "long", "replan", "roll_gap", "tail", "margin", "max_gap", "stale_days", "beam", "width",
    "hoard_at", "strategy", "retire", "min_run", "retire_margin", "early",
    "first_margin", "ideology",
];

/// The milestones a profile scores (the simulator's default set).
pub fn milestones() -> &'static Milestones {
    static MS: OnceLock<Milestones> = OnceLock::new();
    MS.get_or_init(|| {
        Milestones::from_json(include_str!("../../../../configs/sim/milestones.json")).expect("the default milestones load")
    })
}

/// The Open Horizons quest and its ten-year sub-objective.
const HORIZON: &str = "qhorizon_open_horizons";
const SURVIVE: &str = "survive_10_years";

const AXES: &[&str] = &["humanist", "rationalist", "nationalist"];

/// Ticks between retirement checks (each plays the run out on a copy).
const RETIRE_EVERY: u64 = 100;

/// What a profile values beyond credits, in credits per unit.
#[derive(Debug, Clone, Copy)]
pub struct Weights {
    /// Per point of an axis's career-best score.
    pub ideology: i64,
    /// Per Long-Term project's worth of progress (fractions paid, summed),
    /// and per completion.
    pub project: i64,
    pub project_done: i64,
    /// Per building type, command, quest or sub-objective, achievement first had.
    pub building: i64,
    pub command: i64,
    pub quest: i64,
    pub achievement: i64,
    /// Per research bought this run.
    pub research: i64,
    /// Per open milestone's progress squared at the end (0 to 1, [`progress`]).
    pub progress: i64,
    /// Per milestone met, plus `early` per tick it came before the rollout's end.
    pub milestone: i64,
    pub early: i64,
}

/// The profiles: `profile=0` milestones (the default), 1 credits, 2
/// ideology, 3 builder, 4 quests (the milestones' weights, and while Open
/// Horizons is open, its campaign as the only strategy: [`campaign`]).
pub fn profile(n: i64) -> (&'static str, Weights) {
    let w = |ideology, project, project_done, building, command, quest, achievement, research, progress, milestone, early| {
        Weights { ideology, project, project_done, building, command, quest, achievement, research, progress, milestone, early }
    };
    match n {
        1 => ("credits", w(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)),
        2 => ("ideology", w(30, 3000, 3000, 0, 0, 1000, 0, 100, 0, 0, 0)),
        3 => ("builder", w(2, 3000, 3000, 3000, 100, 1000, 300, 100, 0, 0, 0)),
        4 => ("quests", w(3, 2000, 2000, 300, 200, 2000, 300, 300, 4000, 4000, 100)),
        // `early` 100 against 20 (dev seeds 1-3, run 1): the same M2 and M3,
        // M7 in run 1 on 2 seeds of 3 against 1.
        _ => ("milestones", w(3, 2000, 2000, 300, 200, 2000, 300, 300, 4000, 4000, 100)),
    }
}

/// The Open Horizons campaign (the quests profile): one run held long by
/// Dream for its ten years, funding Humanist toward rank 5 (cheaper boredom,
/// stronger Dream) and then its AI Consciousness project, trading meanwhile.
pub fn campaign() -> (String, Directives) {
    let d = Directives {
        research: Some("trade_promotion".into()),
        market: true,
        diversify: true,
        main_last: true,
        sustain: true,
        axis: Some("humanist".into()),
        fund_procs: 2,
        project: Some("ai_consciousness".into()),
        ..Directives::default()
    };
    ("open horizons".into(), d)
}

/// Whether the Open Horizons quest is the one under way.
fn horizon_open(g: &GameState) -> bool {
    g.career.completed_quests.contains("qmarket_awareness") && !g.career.completed_quests.contains(HORIZON)
}

/// The run strategies worth trying now: standing goals, each a label and
/// the directives it sets.
pub fn strategies(g: &GameState) -> Vec<(String, Directives)> {
    let trade = Directives {
        research: Some("trade_promotion".into()),
        market: true,
        diversify: true,
        main_last: true,
        ..Directives::default()
    };
    let mut out = vec![
        ("author".to_string(), Directives::default()),
        ("trade".into(), trade.clone()),
        ("trade, main first".into(), Directives { main_last: false, ..trade.clone() }),
    ];
    out.push(("trade+sustain".into(), Directives { sustain: true, ..trade.clone() }));
    out.push(("trade+all".into(), Directives { all_research: true, ..trade.clone() }));
    out.push(("trade+grant".into(), Directives { project: Some("foundation_grant".into()), ..trade.clone() }));
    // Funding needs Geopolitical Intelligence, visible once the market quest is done.
    if g.career.completed_quests.contains("qmarket_awareness") {
        for a in AXES {
            let fund = Directives { axis: Some(a.to_string()), fund_procs: 2, ..trade.clone() };
            out.push((format!("trade+fund {a}"), fund.clone()));
            out.push((format!("trade+fund {a}+sustain"), Directives { sustain: true, ..fund }));
        }
    }
    out
}

/// What the oracle has seen of the real game at its looks: credits earned by
/// tick and goods launched, this run (for the history milestones).
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
    pub profile: &'static str,
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
    /// Try retiring at each strategy plan, from day `min_run` of a run.
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
    pub fn new(k: &Knobs) -> Self {
        let (name, mut w) = profile(k.get("profile", 0));
        w.early = k.get("early", w.early).max(0);
        w.ideology = k.get("ideology", w.ideology).max(0);
        let stale_days = k.get("stale_days", 300).max(1) as u64;
        let mut base = Author::new(&Knobs::default());
        base.set_gaps(1, k.get("max_gap", 10).max(1) as u64);
        base.set_stale_days(stale_days);
        Oracle {
            base,
            w,
            profile: name,
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
            pin: k.get("strategy", -1),
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
        }
    }

    /// One tactical change from `cur` each, within the strategy.
    fn candidates(&self, p: &mut Player, cur: &Directives) -> Vec<(String, Directives)> {
        let v = p.view();
        let mut out: Vec<(String, Directives)> = Vec::new();
        let mut push = |label: String, d: Directives| {
            if d != *cur && !out.iter().any(|(_, x)| *x == d) {
                out.push((label, d));
            }
        };
        let c = || cur.clone();
        push("target -".into(), Directives { target: None, ..c() });
        for b in v.buildings.iter().filter(|b| b.unlocked) {
            push(format!("target {}", b.id), Directives { target: Some(b.id.to_string()), ..c() });
        }
        // The strategy's research first; once it is done, any.
        let fixed = self.strategy.research.as_deref().filter(|r| v.research.iter().any(|x| x.id == *r && !x.done));
        if fixed.is_none() {
            for r in v.research.iter().filter(|r| r.visible && !r.done) {
                push(format!("research {}", r.id), Directives { research: Some(r.id.to_string()), ..c() });
            }
        }
        if self.strategy.project.is_none() {
            for pr in v.projects.iter().filter(|x| x.unlocked && !x.completed) {
                push(format!("project {}", pr.id), Directives { project: Some(pr.id.to_string()), ..c() });
            }
            push("project -".into(), Directives { project: None, ..c() });
        }
        // Hoarding only once credits on hand are within reach of the
        // stockpile it would serve (`hoard_at`), or to stop one.
        if cur.hoard || super::amount(v, "credits") >= self.hoard_at {
            push("hoard".into(), Directives { hoard: !cur.hoard, ..c() });
        }
        out
    }

    /// Plays the author with goals `dir` on a copy of the game for `horizon`
    /// ticks and scores it; `through` plays on past a retirement (with the
    /// same goals, the target dropped), else the rollout stops there.
    fn rollout(&mut self, p: &Player, dir: &Directives, horizon: u64, through: bool) -> i64 {
        self.rollouts += 1;
        let ms = milestones();
        let mut s: Session = p.fork();
        let mut a = self.base.clone();
        a.dir = dir.clone();
        a.set_gaps(1, self.roll_gap);
        let start = s.state.clone();
        let mut run = s.state.career.run_number;
        let t0 = s.state.tick;
        let end = t0 + horizon;
        let half = t0 + horizon / 2;
        let scored = self.w.milestone != 0 || self.w.progress != 0;
        let mut tracker = Tracker::new(ms);
        tracker.after(&s);
        // Each open milestone's first tick met in the rollout.
        let mut hits: Vec<(usize, u64)> = Vec::new();
        let open: Vec<usize> = (0..ms.milestones.len()).filter(|&i| scored && !self.done.contains(&ms.milestones[i].id)).collect();
        let mut sample: Option<(u64, Fx, Fx, u32)> = None;
        'play: while s.state.tick < end {
            let gap = {
                let mut pl = Player::new(&mut s);
                a.act(&mut pl);
                a.next_look(&mut pl)
            };
            for _ in 0..gap.max(1) {
                if s.state.career.run_number != run {
                    if !through {
                        break 'play;
                    }
                    run = s.state.career.run_number;
                    a.dir.target = None;
                    break;
                }
                if s.state.tick >= end {
                    break 'play;
                }
                // The closing income is sampled from the later of the
                // second half and the start of the run under way.
                if s.state.tick >= half && sample.is_none_or(|x| x.3 != run) {
                    sample = Some((s.state.tick, earned(&s.state), s.state.amount("boredom"), run));
                }
                if scored {
                    tracker.before(&s);
                }
                s.advance(1);
                if scored {
                    tracker.after(&s);
                    for &i in &open {
                        if !hits.iter().any(|h| h.0 == i) && tracker.met(&ms.milestones[i].when, &s) {
                            hits.push((i, s.state.tick));
                        }
                    }
                }
            }
        }
        let sample = sample.filter(|x| x.3 == s.state.career.run_number).map(|x| (x.0, x.1, x.2));
        let mut total = self.score(&start, &s, sample);
        for &(_, t) in &hits {
            let early = end.saturating_sub(t) as i64;
            total += self.w.milestone + self.w.early * early;
        }
        if self.w.progress != 0 {
            // Each open milestone not met here, by how far it got (squared).
            let rate = sample.map_or(Fx::ZERO, |(t, e, _)| (earned(&s.state) - e).mul_ratio(1, (s.state.tick - t).max(1) as i64));
            let met = |w: &When| tracker.met(w, &s);
            let seen = progress::Seen { rate, shipped: tracker.shipped(), tracker_met: &met };
            for &i in open.iter().filter(|i| !hits.iter().any(|h| h.0 == **i)) {
                let f = progress::toward(&s.rules, &s.state, &ms.milestones[i].when, &seen);
                total += (f * f).mul_int(self.w.progress).floor_int();
            }
        }
        total
    }

    fn score(&self, a: &GameState, s: &Session, sample: Option<(u64, Fx, Fx)>) -> i64 {
        let w = &self.w;
        let b = &s.state;
        let mut total = (earned(b) - earned(a)).floor_int();
        // The closing income for the days the run under way has left (a
        // sample from before a retirement reads the new run's start).
        if let Some((t, e, bored)) = sample {
            let dt = (b.tick - t).max(1) as i64;
            let rate = (earned(b) - e).mul_ratio(1, dt);
            let growth = (b.amount("boredom") - bored).mul_ratio(1, dt);
            let room = boredom::cap(&s.rules, b) - b.amount("boredom");
            let left = if growth > Fx::ZERO { (room / growth).floor_int().clamp(0, self.tail) } else { self.tail };
            total += rate.mul_int(left).floor_int();
        }
        let ideo = |g: &GameState| g.career.max_ideology_scores.values().fold(Fx::ZERO, |s, x| s + *x);
        total += (ideo(b) - ideo(a)).mul_int(w.ideology).floor_int();
        total += (projects(b) - projects(a)).mul_int(w.project).floor_int();
        let n = |x: usize, y: usize| y.saturating_sub(x) as i64;
        total += w.project_done * n(a.career.completed_projects.len(), b.career.completed_projects.len());
        total += w.building * n(a.career.owned_buildings.len(), b.career.owned_buildings.len());
        total += w.command * n(a.career.used_commands.len(), b.career.used_commands.len());
        let quests = |g: &GameState| g.career.completed_quests.len() + g.career.completed_sub_objectives.len();
        total += w.quest * n(quests(a), quests(b));
        total += w.achievement * n(a.career.achievements.len(), b.career.achievements.len());
        let research = |g: &GameState| g.career.lifetime.research_completed + g.run.stats.research_completed;
        total += w.research * i64::from(research(b).saturating_sub(research(a)));
        total
    }

    /// Plays each strategy for `long` ticks and keeps the best.
    fn choose_strategy(&mut self, p: &mut Player) {
        let mut best: Option<(i64, String, Directives)> = None;
        let all = if self.profile == "quests" && horizon_open(p.state()) {
            vec![campaign()]
        } else {
            strategies(p.state())
        };
        let pinned = usize::try_from(self.pin).ok().filter(|&i| i < all.len());
        for (i, (label, d)) in all.into_iter().enumerate() {
            if pinned.is_some_and(|k| k != i) {
                continue;
            }
            let s = self.rollout(p, &d, self.long, true);
            if best.as_ref().is_none_or(|b| s > b.0) {
                best = Some((s, label, d));
            }
        }
        if let Some((s, label, d)) = best {
            if d != self.strategy {
                self.note = format!("strategy {label}: {s}");
            }
            self.strategy_label = label;
            // The tactics kept where the strategy leaves them free.
            let cur = &self.base.dir;
            let mut dir = d.clone();
            dir.target = cur.target.clone();
            dir.hoard = cur.hoard;
            if d.project.is_none() {
                dir.project = cur.project.clone();
            }
            if d.research.is_none() {
                dir.research = cur.research.clone();
            }
            self.strategy = d;
            self.base.dir = dir;
        }
    }

    /// Retires once the run's average credits a day is the best it will see
    /// (played out under the current goals), from day `min_run` and not
    /// while holding boredom for the ten years. Returns whether it retired.
    fn maybe_retire(&mut self, p: &mut Player) -> bool {
        // Not before `min_run` days: a retirement at once re-rolls the
        // markets for free, which a peeking player could farm, burning runs.
        let can = p.view().can_retire;
        let g = p.state();
        let open = self.retire && can && g.run.day >= self.min_run;
        if !open || (self.base.dir.sustain && Self::surviving(g)) {
            return false;
        }
        let dir = self.base.dir.clone();
        // The first retirement is a milestone: a profile that scores
        // milestones takes it once the run is within `first_margin`.
        let first = self.w.milestone != 0 && p.state().career.retirements == 0;
        let margin = if first { self.first_margin } else { self.retire_margin };
        let (now, best_later) = self.run_rates(p, &dir);
        if now > Fx::ZERO && best_later.mul_int(1000) <= now.mul_int(1000 + margin) {
            self.note = format!("retire: {now} a day so far, at most {best_later} later");
            let _ = p.apply(&Action::Retire);
            return true;
        }
        false
    }

    /// The run's average credits earned a day so far, and the best that
    /// average reaches later in the run, played out on a copy under goals
    /// `dir` (to the run's end, or `long` ticks).
    fn run_rates(&mut self, p: &Player, dir: &Directives) -> (Fx, Fx) {
        self.rollouts += 1;
        let avg = |g: &GameState| g.run.stats.credits_earned.mul_ratio(1, g.run.day.max(1) as i64);
        let now = avg(p.state());
        let mut s: Session = p.fork();
        let mut a = self.base.clone();
        a.dir = dir.clone();
        a.set_gaps(1, self.roll_gap);
        let run = s.state.career.run_number;
        let end = s.state.tick + self.long;
        let mut best = Fx::ZERO;
        'play: while s.state.tick < end {
            let gap = {
                let mut pl = Player::new(&mut s);
                a.act(&mut pl);
                a.next_look(&mut pl)
            };
            for _ in 0..gap.max(1) {
                if s.state.career.run_number != run || s.state.tick >= end {
                    break 'play;
                }
                s.advance(1);
                if s.state.career.run_number == run {
                    best = best.max(avg(&s.state));
                }
            }
        }
        (now, best)
    }

    /// Tries the tactical candidates and keeps the best goals.
    fn plan(&mut self, p: &mut Player) {
        self.plans += 1;
        let cur = self.base.dir.clone();
        let (short, long) = ((self.horizon / 3).max(60), self.horizon);
        // A screen: every candidate over a third of the horizon.
        let mut screened: Vec<(i64, String, Directives)> = Vec::new();
        for (label, d) in self.candidates(p, &cur) {
            let s = self.rollout(p, &d, short, false);
            screened.push((s, label, d));
        }
        // Best first; ties keep candidate order (the sort is stable).
        screened.sort_by_key(|x| std::cmp::Reverse(x.0));
        // The best `width` over the whole horizon, beside keeping.
        let keep = self.rollout(p, &cur, long, false);
        let mut scored: Vec<(i64, String, Directives)> = Vec::new();
        for (_, label, d) in screened.into_iter().take(self.width) {
            let s = self.rollout(p, &d, long, false);
            scored.push((s, label, d));
        }
        scored.sort_by_key(|x| std::cmp::Reverse(x.0));
        // The two best changes in different goals, together.
        if self.beam && scored.len() >= 2 {
            let (a, b) = (&scored[0], &scored[1]);
            let field = |l: &str| l.split(' ').next().unwrap_or("").to_string();
            if field(&a.1) != field(&b.1) && a.0 > keep && b.0 > keep {
                let both = merge(&cur, &a.2, &b.2);
                let label = format!("{} + {}", a.1, b.1);
                let s = self.rollout(p, &both, long, false);
                scored.push((s, label, both));
                scored.sort_by_key(|x| std::cmp::Reverse(x.0));
            }
        }
        let need = keep + keep.abs() * self.margin / 1000;
        match scored.into_iter().next() {
            Some((s, label, d)) if s > need => {
                self.note = format!("{}: {label}: {s} over keep {keep}", self.strategy_label);
                self.target_owned = d.target.as_deref().map_or(0, |t| p.state().owned(t));
                self.base.dir = d;
            }
            _ => self.note = format!("{}: keep {keep}", self.strategy_label),
        }
    }

    /// Records the milestones the real game has met, as far as it can see.
    fn observe(&mut self, p: &Player) {
        let g = p.state();
        self.seen.look(g);
        let ms = milestones();
        // A tracker that saw no ticks answers the state milestones exactly
        // and the history ones never; `seen` answers those.
        let blank = Tracker::new(ms);
        for m in &ms.milestones {
            if !self.done.contains(&m.id) && (self.seen.met(&m.when) || self.blank_met(&blank, &m.when, p)) {
                self.done.insert(m.id.clone());
            }
        }
    }

    fn blank_met(&self, t: &Tracker, w: &When, p: &Player) -> bool {
        match w {
            When::CreditRate { .. } | When::GoodsShipped { .. } => false,
            When::AllOf { of } => of.iter().all(|x| self.blank_met(t, x, p) || self.seen.met(x)),
            _ => t.met(w, p.session()),
        }
    }

    /// Whether the run should go on past a stale stretch: the Open Horizons
    /// quest is active and its ten years aren't yet survived.
    fn surviving(g: &GameState) -> bool {
        horizon_open(g) && !g.career.completed_sub_objectives.contains(&events::sub_key(HORIZON, SURVIVE))
    }
}

/// `cur` with the goals `a` and `b` each changed.
fn merge(cur: &Directives, a: &Directives, b: &Directives) -> Directives {
    let pick = |x: bool| if x { a } else { b };
    let fund = a.axis != cur.axis || a.fund_procs != cur.fund_procs;
    Directives {
        target: pick(a.target != cur.target).target.clone(),
        research: pick(a.research != cur.research).research.clone(),
        project: pick(a.project != cur.project).project.clone(),
        axis: pick(fund).axis.clone(),
        fund_procs: pick(fund).fund_procs,
        sustain: pick(a.sustain != cur.sustain).sustain,
        all_research: pick(a.all_research != cur.all_research).all_research,
        market: pick(a.market != cur.market).market,
        diversify: pick(a.diversify != cur.diversify).diversify,
        hoard: pick(a.hoard != cur.hoard).hoard,
        main_last: pick(a.main_last != cur.main_last).main_last,
    }
}

/// Long-Term projects' fractions paid, summed (completed ones count 1).
fn projects(g: &GameState) -> Fx {
    content()
        .projects
        .iter()
        .filter(|p| p.persistent())
        .fold(Fx::ZERO, |s, p| {
            if g.career.completed_projects.contains(&p.id) {
                return s + Fx::ONE;
            }
            let total = p.costs.values().fold(Fx::ZERO, |s, x| s + *x);
            let paid = g.career.project_progress.get(&p.id).map_or(Fx::ZERO, |m| m.values().fold(Fx::ZERO, |s, x| s + *x));
            s + if total > Fx::ZERO { paid / total } else { Fx::ZERO }
        })
}

/// Credits earned in the career so far, the run under way included.
fn earned(g: &GameState) -> Fx {
    g.career.lifetime.credits_earned + g.run.stats.credits_earned
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
        if self.base.dir.sustain && stale && p.view().can_retire && !Self::surviving(p.state()) {
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

    fn new_run(&mut self, p: &mut Player) {
        self.base.dir = Directives::default();
        self.next_plan = p.state().tick;
        self.next_strategy = p.state().tick;
    }

    fn note(&self) -> String {
        format!("{} [{} plans, {} rollouts] {}", self.profile, self.plans, self.rollouts, self.note)
    }
}
