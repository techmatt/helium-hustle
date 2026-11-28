//! `author`: the heuristic program author, the rung that matters most
//! (program authoring is the core skill). It builds with the builder's scored
//! order but saves for its target (buying another only from credits past the
//! target's price), and writes programs from templates. After acting it picks
//! the ticks to its next look, `gap` (`plan_gap`: the soonest its
//! target or next research is affordable, boredom reaches `dream_at`, the pad
//! cools down or a stale run would retire; `min_gap`..`max_gap`). How it writes
//! the main program depends on that gap:
//!
//! | Program | Template | Processors |
//! |---|---|---|
//! | 1 main, `gap <= hand_gap` | a plan to the next look: `launch_pads` if a pad is full; `buy_propellant` × the fuel short when a pad is within 25 of full; `load_pads` × the loads that would move cargo, at most `loads`; `buy_*` × the target's missing titanium, regolith or ice, as far as credits past its price go; then `cloud_compute` × 2 × `max_gap` × processors | the rest |
//! | 1 main, longer gaps | a standing cycle: `load_pads × loads, launch_pads, cloud_compute × loads, buy_propellant × fuel`, `buy_*` × the target's missing inputs no building makes, at most `loads` | the rest |
//! | 2 dream | `dream` | 1 while boredom is past `dream_at`% of its cap and the run still builds |
//! | 3 fund | `fund_<axis>`: the first axis (humanist, rationalist, nationalist) whose persistent project isn't done | 1 when 3 or more are left |
//! | 4 market | `promote_<good>` for each pad good, `disrupt_speculators` | 1 when 3 or more are left |
//! | 5 construction (`construct`) | `if Free land < its land and Credits > the land price` / `buy land` (only while short of land); `if Credits > its price and <its scarcest other input> > its price` (a capped input as a percent of its cap) / `build <target> up to owned + 1`; then the filler × 1 | 1 when 2 or more are left, once `if` and `build` are unlocked (never under the oracle's `main_last`, which puts the main program in that slot) |
//!
//! With `cond` (default 1; the oracle runs 0) and program logic open, the
//! table gives way to condition programs, one text each, saved by name
//! ([`SHIP`], [`DREAM_WATCH`], [`FUND_NAME`], [`MARKET_NAME`], [`BUILD_ORDER`])
//! and loaded back (side programs at each new run, where they keep their
//! lines while without a processor): the main program is [`ship_cycle`]
//! (the hand plan, unsaved, while the target waits on a bought input),
//! Program 2 the [`dream_watch`] on one processor once Dream is unlocked,
//! Program 5 the [`build_order_program`] learned from the last run (rewritten
//! only when a run bought past it; run 1 writes the one-target program).
//!
//! A program is rewritten only when its text changes (a rewrite restarts its
//! pointer), or, for the main plan, to run its due lines again once the
//! pointer has passed into the filler; a plan with nothing due isn't written
//! over one already in its filler. Extra pads carry a good other than He-3
//! that sits at its cap.
//! While looking often, it pauses buildings that eat a material its target is
//! short of, so the stock builds up.
//!
//! It buys research in [`RESEARCH`] order, funds the cheapest open `run`
//! project whose costs can move beside the first open other one in
//! [`PROJECTS`] order (each only when every cost can move; science at the most a
//! day when no research waits, else `science_share` percent of the most; a
//! tenth of the credits past twice its target's price plus 200; anything else
//! `project_rate`), and retires once the run has bought nothing for
//! `stale_days` days. docs/agents.md says the same.
//!
//! The oracle steers it through [`Directives`] (a target, research, a
//! project, funding, Dream held, market, pads, hoarding, program order); each
//! one set changes the rules above where its doc says. Unset, as for the
//! `author` and `casual` rungs, they change nothing.

use std::collections::BTreeMap;

use hh_core::content::Requirement;
use hh_core::state::{Amount, Clause, Cmp, Condition, Entry, Join, Joined, Op, Reading, Verb};
use hh_core::view::PadView;
use hh_core::{Action, Fx, View};

use super::builder::{Builder, BRIDGES, INPUT_BRIDGES};

mod templates;
use templates::*;
use super::{amount, cap, ceil_int, command_cost, command_unlocked, credits_price, made, Agent, Knobs, Player};

pub const KNOBS: &[&str] = &[
    "decay", "starved", "reserve", "buys", "compute", "dream_at", "stale_days", "project_rate", "fund", "market",
    "hand_gap", "loads", "pause", "wait_demand", "min_gap", "max_gap", "construct", "cond", "diversify",
    "all_research", "science_share", "ship_loads", "fill", "dream_fill", "commit", "fund_procs",
];

/// Library names of the condition programs (`cond`): one saved copy each,
/// loaded back at every new run.
pub(crate) const SHIP: &str = "ship";
pub(crate) const DREAM_WATCH: &str = "dream watch";
pub const BUILD_ORDER: &str = "build order";
pub(crate) const FUND_NAME: &str = "fund";
pub(crate) const MARKET_NAME: &str = "market";

/// Research, first wanted first.
pub const RESEARCH: &[&str] = &[
    "basic_blueprints",
    "advanced_blueprints",
    "propellant_synthesis",
    "market_awareness",
    "dream_protocols",
    "stress_tolerance",
    "shipping_efficiency",
    "geopolitical_intelligence",
    "efficient_dreaming",
    "trade_promotion",
    "speculator_analysis",
    "overclock_protocols",
    "overclock_boost",
];

/// Research the main plan buys by program line every run, once science
/// covers its price (the rest the author buys on its looks).
pub(crate) const PLAN_RESEARCH: &[&str] = &[
    "propellant_synthesis",
    "market_awareness",
    "dream_protocols",
    "stress_tolerance",
    "geopolitical_intelligence",
];

/// Projects the author funds: the first open non-`run` one in this order,
/// beside the cheapest open `run` one whose costs can move (order only
/// breaks ties).
pub(crate) const PROJECTS: &[&str] = &[
    "blueprint_archive",
    "conditional_logic",
    "deep_space_telescope",
    "lunar_cartography",
    "foundation_grant",
    "ai_consciousness",
    "research_archive",
    "microwave_power",
    "chemical_energy",
    "deep_core_survey",
    "speculator_dossier",
    "market_cornering",
    "grid_recalibration",
];

/// Ideology axes in funding order, each with its rank-5 persistent project
/// (mirrors content/projects.json; the order is strategy).
pub(crate) const AXES: &[(&str, &str)] = &[
    ("humanist", "ai_consciousness"),
    ("rationalist", "research_archive"),
    ("nationalist", "microwave_power"),
];

/// Tradeable goods by base value, dearest first (mirrors configs/baseline.json
/// `base_values`: circuits 15, He-3 10, titanium 6, propellant 4; the view
/// shows no prices).
const DEAREST: &[&str] = &["circuits", "he3", "titanium", "propellant"];

pub(crate) const MAIN: usize = 0;
pub(crate) const DREAM: usize = 1;
pub(crate) const FUND: usize = 2;
pub(crate) const MARKET: usize = 3;
pub(crate) const BUILD: usize = 4;

/// Goals a planner sets on top of the author's own choices (the oracle,
/// docs/agents.md). Unset, the author plays as itself.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Directives {
    /// The building to save for, in place of the best-scored one.
    pub target: Option<String>,
    /// Research to buy first, saving science for it.
    pub research: Option<String>,
    /// The project to fund, in place of the first open one in order; funded
    /// with everything to spare (credits past the target's price, all science).
    pub project: Option<String>,
    /// The axis to fund, and processors on it (0: the author's own rule).
    pub axis: Option<String>,
    pub fund_procs: u32,
    /// Hold boredom down with Dream (enough processors to outpace growth,
    /// from a third of the cap), and never retire for a stale run.
    pub sustain: bool,
    /// Meant to buy all visible research, not only [`RESEARCH`], but a no-op
    /// today: `next_research` and `science_wanted` look only at the directed
    /// id and [`RESEARCH`], so it plays as the plain author (kept for the
    /// oracle's strategy indices and the autopilot's second driver; whether
    /// to make it work is open, docs/agents.md).
    pub all_research: bool,
    /// Run the market program (Promote what ships, Disrupt) on one processor
    /// whenever two are free, not only three.
    pub market: bool,
    /// Extra pads each carry a different good with a maker running, dearest
    /// first, not only a good at its cap.
    pub diversify: bool,
    /// Buy no buildings and fund no project: let credits pile up.
    pub hoard: bool,
    /// The main program runs last (programs run in order), so Dream, Fund
    /// and the market get energy before Cloud Compute's filler takes it.
    pub main_last: bool,
    /// Committed to one ideology axis (130): its boredom tool is the only
    /// one used (Dream for Humanist, Theorize for Rationalist, held as
    /// `sustain` holds Dream; Recreation Domes for Nationalist, bought from
    /// half the boredom cap), its tool's research comes first, its tagged
    /// buildings score ×3/2, and while 1.5 A Voice is under way no run
    /// retires for being stale (boredom ends it: `retires_stale`).
    /// Funding itself is `axis` (the committed players set both).
    pub commit: Option<String>,
}

/// Quest 1.4 Trade, whose completion starts 1.5 A Voice (rank 5 on any
/// axis this run), content/events.json.
const TRADE_QUEST: &str = "qtrade";
const VOICE_QUEST: &str = "qa_voice";

/// Each decision the directives change, one method each: the author asks
/// these, and with none set each answers as the plain author plays.
impl Directives {
    /// The command holding boredom down: the committed axis's tool (none
    /// for Nationalist, whose tool is a building), else Dream with `sustain`.
    pub fn tool_command(&self) -> Option<&'static str> {
        match self.commit.as_deref() {
            Some("humanist") => Some("dream"),
            Some("rationalist") => Some("theorize"),
            Some(_) => None,
            None => self.sustain.then_some("dream"),
        }
    }

    /// The research that opens the committed axis's tool.
    fn tool_research(&self) -> Option<&'static str> {
        match self.commit.as_deref() {
            Some("humanist") => Some("dream_protocols"),
            Some("rationalist") => Some("thought_experiments"),
            Some("nationalist") => Some("recreation_dome_blueprints"),
            _ => None,
        }
    }

    /// Committed, the research opening Fund (shown once A Voice starts),
    /// after only the Blueprints that open the base's science (before them
    /// it waited all run 2 on science it couldn't make).
    fn fund_research(&self) -> &'static [&'static str] {
        if self.commit.is_some() {
            &["basic_blueprints", "advanced_blueprints", "geopolitical_intelligence"]
        } else {
            &[]
        }
    }

    /// The builder's ranking with the target, while unlocked, first; for a
    /// committed player its tagged buildings scored up, and a Recreation
    /// Dome first from half the boredom cap (Nationalist).
    fn rank_target(&self, v: &View, ranked: &mut Vec<(String, i64)>) {
        if let Some(axis) = self.commit.as_deref() {
            let tagged = |id: &str| v.buildings.iter().any(|b| b.id == id && b.ideology == Some(axis));
            for (id, s) in ranked.iter_mut() {
                if tagged(id) {
                    *s = s.saturating_mul(3) / 2;
                }
            }
            ranked.sort_by_key(|x| std::cmp::Reverse(x.1));
            let dome = "recreation_dome";
            let half = v.boredom.amount.mul_int(2) >= v.boredom.cap;
            if axis == "nationalist" && half && v.buildings.iter().any(|b| b.id == dome && b.unlocked) {
                ranked.retain(|(id, _)| id != dome);
                ranked.insert(0, (dome.to_string(), i64::MAX));
            }
        }
        if let Some(t) = &self.target {
            if v.buildings.iter().any(|b| b.id == t && b.unlocked) {
                ranked.retain(|(id, _)| id != t);
                ranked.insert(0, (t.clone(), i64::MAX));
            }
        }
    }

    /// Research worth its science whatever else is open: all of it, or the
    /// directed one.
    fn wants_research(&self, id: &str) -> bool {
        self.all_research || self.research.as_deref() == Some(id) || self.tool_research() == Some(id) || self.fund_research().contains(&id)
    }

    /// Research in the order wanted: committed, Fund's; the directed one,
    /// the committed tool's, then [`RESEARCH`].
    fn research_order(&self) -> impl Iterator<Item = &str> {
        self.fund_research()
            .iter()
            .copied()
            .chain(self.research.as_deref())
            .chain(self.tool_research())
            .chain(RESEARCH.iter().copied())
    }

    /// Projects in the order funded: the pushed one, then [`PROJECTS`].
    fn project_order(&self) -> impl Iterator<Item = &str> {
        self.project.as_deref().into_iter().chain(PROJECTS.iter().copied())
    }

    /// Whether project `id` is pushed (funded with everything to spare).
    fn pushes(&self, id: &str) -> bool {
        self.project.as_deref() == Some(id)
    }

    /// The program slots of the main program and Dream, of `len`: main last
    /// with `main_last`, Dream first while holding boredom, else in order.
    fn program_slots(&self, len: usize) -> (usize, usize) {
        let last = len.saturating_sub(1);
        if self.main_last && last > MARKET {
            (last, DREAM)
        } else if self.tool_command().is_some() {
            (DREAM, MAIN)
        } else {
            (MAIN, DREAM)
        }
    }

    /// The axis to fund and its processors, when directed (else the
    /// author's own rule).
    fn funded_axis(&self) -> Option<(&str, u32)> {
        self.axis.as_deref().filter(|_| self.fund_procs > 0).map(|a| (a, self.fund_procs))
    }

    /// Whether the market program runs with `left` processors free: the
    /// author's own rule (`own`, from 3 free), or directed from 2.
    fn market_runs(&self, own: bool, left: u32) -> bool {
        (own && left > 2) || (self.market && left > 1)
    }

    /// The builder's energy reserve: holding boredom down, room for two
    /// Dreams a day once Dream is unlocked (before, the room starved the run).
    fn energy_reserve(&self, own: Fx, v: &View) -> Fx {
        match self.tool_command().filter(|c| command_unlocked(v, c)) {
            Some(c) => own + command_cost(v, c, "energy").mul_int(2),
            None => own,
        }
    }

    /// Whether a run that has bought nothing for `stale_days` retires: not
    /// while holding boredom down; committed, not while 1.5 A Voice (rank 5
    /// this run) is under way: each such run plays out until boredom ends it.
    pub fn retires_stale(&self, v: &View) -> bool {
        if self.commit.is_some() {
            let done = |q: &str| v.completed_quests.contains(&q);
            return !(done(TRADE_QUEST) && !done(VOICE_QUEST));
        }
        !self.sustain
    }
}

#[derive(Clone)]
pub struct Author {
    /// The planner's goals (unset for the plain author).
    pub dir: Directives,
    b: Builder,
    /// The builder's own energy reserve (its `reserve` knob).
    reserve: Fx,
    buys: u32,
    compute: bool,
    dream_at: i64,
    stale_days: u64,
    project_rate: Fx,
    /// Percent of a project's most a day of science while research waits.
    science_share: i64,
    fund: bool,
    market: bool,
    /// Writes the construction program (Program 5) once the game allows it.
    construct: bool,
    /// Buildings built this run at the last look, and the day that count last rose.
    built: u32,
    built_day: u64,
    run: u32,
    hand_gap: u64,
    loads: u32,
    /// The ship cycle's loads a batch ([`ship_cycle`]), and the filler's count
    /// a loop in the ship cycle (`fill`) and the Dream watch (`dream_fill`):
    /// each `if` costs a step (rev 19), so a longer filler spends fewer on
    /// guards (and dreams less often in the watch).
    ship_loads: i64,
    fill: u32,
    dream_fill: u32,
    min_gap: u64,
    max_gap: u64,
    /// The ticks to its next look, chosen at this one.
    gap: u64,
    income: super::Income,
    pause: bool,
    wait_demand: bool,
    last_note: String,
    /// Writes condition programs (once program logic is open): the ship
    /// cycle, the Dream watch and the build order, kept in the library.
    cond: bool,
    /// This run's purchases as seen look to look: each building's count, and
    /// each rise in order (`(building, owned after)`).
    seen_run: u32,
    owned: BTreeMap<String, u32>,
    order: Vec<(String, u32, Vec<Bridge>)>,
    /// Per building, the buyable inputs of its price that no building made
    /// or that were short at the last look ([`Bridge`]).
    unmade: BTreeMap<String, Vec<Bridge>>,
    /// The build order program, from the last run that bought past what it
    /// covers, and how many of that run's purchases its lines cover (64
    /// lines may stop short of them all).
    plan: Vec<Entry>,
    plan_from: usize,
    /// Ends the look at a retirement (the hands-off player plays the new run).
    pub stop_at_retire: bool,
}

impl Author {
    pub fn new(k: &Knobs) -> Self {
        let min_gap = k.get("min_gap", 1).max(1) as u64;
        let b = Builder::new(k);
        Author {
            // `diversify` and `all_research` (default 0): the directives of those
            // names, for the debug autopilot, whose boosted runs fill no cap with
            // the plain rule's goods (`all_research` is a no-op today: see
            // `Directives::all_research`).
            dir: Directives {
                diversify: k.get("diversify", 0) != 0,
                all_research: k.get("all_research", 0) != 0,
                // `commit` (130): one axis (1 Nationalist, 2 Humanist, 3
                // Rationalist), funded on `fund_procs` processors (default 2).
                axis: super::axis_knob(k.get("commit", 0)).map(String::from),
                fund_procs: k.get("fund_procs", 2).clamp(1, 64) as u32,
                commit: super::axis_knob(k.get("commit", 0)).map(String::from),
                ..Directives::default()
            },
            reserve: b.reserve,
            b,
            buys: k.get("buys", 3).max(1) as u32,
            compute: k.get("compute", 1) != 0,
            dream_at: k.get("dream_at", 60).clamp(0, 100),
            stale_days: k.get("stale_days", 300).max(1) as u64,
            project_rate: Fx::from_int(k.get("project_rate", 3).max(0)),
            science_share: k.get("science_share", 50).clamp(0, 100),
            fund: k.get("fund", 1) != 0,
            market: k.get("market", 1) != 0,
            construct: k.get("construct", 1) != 0,
            built: 0,
            built_day: 0,
            run: 0,
            hand_gap: k.get("hand_gap", 60).max(0) as u64,
            loads: k.get("loads", 20).clamp(1, 1000) as u32,
            ship_loads: k.get("ship_loads", 2).clamp(1, 100),
            fill: k.get("fill", 8).clamp(1, 1000) as u32,
            dream_fill: k.get("dream_fill", 1).clamp(1, 1000) as u32,
            min_gap,
            max_gap: (k.get("max_gap", 10).max(1) as u64).max(min_gap),
            gap: min_gap,
            income: super::Income::default(),
            pause: k.get("pause", 1) != 0,
            wait_demand: k.get("wait_demand", 0) != 0,
            last_note: String::new(),
            cond: k.get("cond", 1) != 0,
            seen_run: 0,
            owned: BTreeMap::new(),
            order: Vec::new(),
            unmade: BTreeMap::new(),
            plan: Vec::new(),
            plan_from: 0,
            stop_at_retire: false,
        }
    }

    /// Notes purchases since the last look. At a new run, a last run that
    /// bought more than the build order covers rewrites it (the order fell
    /// short); otherwise the saved one stands.
    pub(crate) fn track(&mut self, v: &View) {
        if !self.cond {
            return;
        }
        if v.run != self.seen_run {
            if self.seen_run != 0 && self.order.len() > self.plan_from {
                let filler = if self.compute { "cloud_compute" } else { "idle" };
                // What it covers (the line cap may cut it short): a later run
                // that buys past that rewrites it.
                (self.plan, self.plan_from) = build_order_program(v, &self.order, filler);
            }
            self.seen_run = v.run;
            self.order.clear();
            self.owned = v.buildings.iter().map(|b| (b.id.to_string(), b.owned)).collect();
        } else {
            for b in &v.buildings {
                let had = self.owned.get(b.id).copied().unwrap_or(0);
                let bridged = self.unmade.get(b.id).cloned().unwrap_or_default();
                for n in had + 1..=b.owned {
                    self.order.push((b.id.to_string(), n, bridged.clone()));
                }
                self.owned.insert(b.id.to_string(), b.owned);
            }
        }
        // Each building's buyable inputs that nothing makes now or that are
        // short of its price: the purchase seen at the next look may have
        // needed them bought. (Remembering an input short at any look since
        // the last purchase bought more and earned less: dev seeds 1-8.)
        self.unmade = v
            .buildings
            .iter()
            .map(|b| {
                let res = INPUT_BRIDGES
                    .iter()
                    .filter_map(|&(r, cmd)| {
                        let rich = credits_price(b) + command_cost(v, cmd, "credits");
                        b.cost
                            .get(r)
                            .filter(|&&n| n > Fx::ZERO && (!made(v, r) || amount(v, r) < n))
                            .map(|&n| (r.to_string(), n, rich))
                    })
                    .collect();
                (b.id.to_string(), res)
            })
            .collect();
    }

    /// Its look gaps (the oracle sets its own and its rollouts').
    pub fn set_gaps(&mut self, min_gap: u64, max_gap: u64) {
        self.min_gap = min_gap.max(1);
        self.max_gap = max_gap.max(self.min_gap);
    }

    /// Days a run may buy nothing before it retires.
    pub fn set_stale_days(&mut self, days: u64) {
        self.stale_days = days.max(1);
    }

    /// Pauses buildings that eat a material the target is short of (regolith,
    /// ice, titanium, circuits), so the stock can build up; runs everything
    /// else at full.
    fn pause_for(&self, p: &mut Player, target: Option<&str>) {
        let v = p.view();
        let short: Vec<String> = target
            .and_then(|t| v.buildings.iter().find(|b| b.id == t))
            .map(|b| {
                b.cost
                    .iter()
                    .filter(|(res, need)| *res != "credits" && *res != "energy" && amount(v, res) < **need)
                    .map(|(res, _)| res.clone())
                    .collect()
            })
            .unwrap_or_default();
        let sets: Vec<Action> = v
            .buildings
            .iter()
            .filter(|b| b.owned > 0)
            .filter_map(|b| {
                let pause = self.pause && b.upkeep.keys().any(|res| short.contains(res));
                let want = if pause { 0 } else { b.owned };
                (b.active != want).then(|| Action::SetActive { building: b.id.to_string(), active: want })
            })
            .collect();
        for a in sets {
            let _ = p.apply(&a);
        }
    }

    /// Builds toward the target; buys something else only from credits
    /// beyond the target's price. Returns the target.
    fn build(&mut self, p: &mut Player) -> Option<String> {
        let mut target = None;
        if self.dir.hoard {
            return None;
        }
        for _ in 0..self.buys {
            let v = p.view();
            let t = self.reachable_target(v);
            target = t.clone();
            let Some(t) = t else { break };
            if Builder::try_buy(p, &t) {
                continue;
            }
            // Credits to spare beyond the target: the best affordable other.
            let v = p.view();
            let keep = price(v, &t, "credits");
            let spare = amount(v, "credits") - keep;
            let other = self.b.ranked(v).into_iter().map(|(id, _)| id).find(|id| {
                *id != t
                    && v.buildings.iter().any(|b| {
                        b.id == *id && b.affordable && credits_price(b) <= spare
                    })
                    && self.b.resolve(v, id) == id.as_str()
            });
            match other {
                Some(id) if Builder::try_buy(p, &id) => {}
                _ => break,
            }
        }
        target
    }

    /// The best-scored building whose missing inputs can be made or bought.
    /// A target short of an input nothing makes and no command buys (circuits)
    /// works toward that input's maker, or the building that unlocks it.
    /// A step costing more of a resource than its cap holds can never be
    /// bought, and waiting on it deadlocks: [`Builder::resolve`] already
    /// plans storage for an over-cap building, so this is the storage itself
    /// over its cap (a Storage Depot's cost grows 1.5x a buy, the cap it adds
    /// linearly). Then the building itself is the target if it fits its caps
    /// (an Arbitrage Engine resolved to a Solar Panel that needs a ninth
    /// depot), else it is skipped.
    fn reachable_target(&self, v: &View) -> Option<String> {
        let mut ranked = self.b.ranked(v);
        self.dir.rank_target(v, &mut ranked);
        for (id, _) in ranked {
            let Some(b) = [self.b.resolve(v, &id), id.as_str()]
                .into_iter()
                .filter_map(|t| v.buildings.iter().find(|b| b.id == t))
                .find(|b| !over_cap(v, b))
            else {
                continue;
            };
            let t = b.id.to_string();
            let mut ok = true;
            for (res, &need) in &b.cost {
                if amount(v, res) >= need || res == "credits" {
                    continue;
                }
                let bridged = BRIDGES.iter().any(|(r, c)| r == res && command_unlocked(v, c));
                let made = made(v, res);
                if bridged || made {
                    continue;
                }
                ok = false;
                if let Some(step) = Self::toward_maker(v, res) {
                    let step = self.b.resolve(v, step);
                    if v.buildings.iter().any(|m| m.id == step && !over_cap(v, m)) {
                        return Some(step.to_string());
                    }
                }
            }
            if ok {
                return Some(t);
            }
        }
        None
    }

    /// The first step toward a building that makes `res`: the maker if
    /// unlocked, else the building its unlock asks for.
    fn toward_maker<'v>(v: &'v View, res: &str) -> Option<&'v str> {
        for m in v.buildings.iter().filter(|m| m.produces.contains_key(res)) {
            if m.unlocked {
                return Some(m.id);
            }
            if let Requirement::Building { building, .. } = &m.requires {
                if v.buildings.iter().any(|x| x.id == *building && x.unlocked) {
                    return Some(building);
                }
            }
        }
        None
    }

    /// Whether research `id` is worth science: directed or in [`RESEARCH`]
    /// (an open project no longer holds research back; it takes a share).
    fn worth(&self, id: &str) -> bool {
        self.dir.wants_research(id) || RESEARCH.contains(&id)
    }

    /// The research it wants next: the directed one first, then [`RESEARCH`]
    /// order.
    fn next_research<'v>(&self, v: &'v View) -> Option<&'v hh_core::view::ResearchView> {
        self.dir
            .research_order()
            .filter_map(|id| v.research.iter().find(|r| r.id == id))
            .find(|r| r.visible && !r.done && self.worth(r.id))
    }

    fn research(&self, p: &mut Player) {
        let v = p.view();
        if let Some(r) = self.next_research(v) {
            if r.affordable {
                let a = Action::Research { research: r.id.to_string() };
                let _ = p.apply(&a);
            }
        }
    }

    /// Science is spoken for while research worth it is visible and unbought.
    fn science_wanted(&self, v: &View) -> bool {
        RESEARCH
            .iter()
            .any(|id| v.research.iter().any(|r| r.id == *id && r.visible && !r.done && self.worth(id)))
    }

    fn projects(&self, p: &mut Player, target: Option<&str>) {
        let v = p.view();
        if self.dir.hoard {
            let stops: Vec<Action> = v
                .projects
                .iter()
                .flat_map(|o| o.rates.iter().filter(|(_, r)| **r > Fx::ZERO).map(move |(res, _)| (o.id, res)))
                .map(|(id, res)| Action::SetProjectRate { project: id.to_string(), resource: res.clone(), rate: Fx::ZERO })
                .collect();
            for a in stops {
                let _ = p.apply(&a);
            }
            return;
        }
        // The pushed one alone; else the cheapest movable open `run` project
        // (its reward is live for the rest of this run) beside the first open
        // other one in order, so a slow `run` project never blocks a
        // `timeline` one.
        let open: Vec<&hh_core::view::ProjectView> = self
            .dir
            .project_order()
            .filter_map(|id| v.projects.iter().find(|p| p.id == id))
            .filter(|p| p.unlocked && !p.completed)
            .collect();
        // Every cost must be able to move: on hand, or made by a building.
        let can_move = |p: &hh_core::view::ProjectView| {
            p.costs.keys().all(|res| {
                amount(v, res) > Fx::ZERO || v.buildings.iter().any(|b| b.active > 0 && b.produces.contains_key(res))
            })
        };
        let total = |p: &hh_core::view::ProjectView| p.costs.values().fold(Fx::ZERO, |s, x| s + *x);
        let chosen: Vec<&hh_core::view::ProjectView> = match open.iter().find(|p| self.dir.pushes(p.id)) {
            Some(p) => vec![*p],
            None => {
                let cheap_run = open.iter().filter(|p| p.scope == "run" && can_move(p)).min_by_key(|p| total(p));
                let other = open.iter().find(|p| p.scope != "run");
                cheap_run.into_iter().chain(other).copied().collect()
            }
        };
        if chosen.is_empty() {
            return;
        }
        let keep = target.map_or(Fx::ZERO, |t| price(v, t, "credits"));
        let fund_waits = self.next_research(v).is_some_and(|r| self.dir.fund_research().contains(&r.id));
        let mut want = Vec::new();
        for proj in &chosen {
            let pushed = self.dir.pushes(proj.id);
            let movable = can_move(proj);
            for res in proj.costs.keys() {
                let max = proj.max_rates.get(res).copied().unwrap_or(Fx::ZERO);
                let rate = if !movable {
                    Fx::ZERO
                } else {
                    match res.as_str() {
                        // None while the Blueprints wait on science: the base's buildings come first.
                        "science" if super::science_short(v).is_some() => Fx::ZERO,
                        // Committed: none while Fund's research waits (the
                        // run-2 Blueprint Archive took it for 1,300 days: 130).
                        "science" if fund_waits => Fx::ZERO,
                        // All of it, unless research is waiting: then a share.
                        "science" if !self.science_wanted(v) => max,
                        "science" => max.mul_ratio(self.science_share, 100),
                        // Pushed: everything past the target's price.
                        "credits" if pushed => (amount(v, "credits") - keep).max(Fx::ZERO).min(max),
                        // A tenth of the credits past twice the target's price and 200.
                        "credits" => {
                            let spare = amount(v, "credits") - keep.mul_int(2) - Fx::from_int(200);
                            spare.mul_ratio(1, 10).max(Fx::ZERO).min(max)
                        }
                        _ => self.project_rate.min(max),
                    }
                };
                if proj.rates.get(res).copied().unwrap_or(Fx::ZERO) != rate {
                    want.push(Action::SetProjectRate { project: proj.id.to_string(), resource: res.clone(), rate });
                }
            }
        }
        // Rates left on other projects stop.
        for other in v.projects.iter().filter(|o| !chosen.iter().any(|c| c.id == o.id)) {
            for (res, r) in &other.rates {
                if *r > Fx::ZERO {
                    want.push(Action::SetProjectRate { project: other.id.to_string(), resource: res.clone(), rate: Fx::ZERO });
                }
            }
        }
        for a in want {
            let _ = p.apply(&a);
        }
    }

    /// Extra pads carry a good at its cap (not He-3, not the target's).
    fn pads(&self, p: &mut Player, target: Option<&str>) {
        let v = p.view();
        let mut sets = Vec::new();
        if self.dir.diversify {
            // Goods with a maker running, dearest first (the view has no
            // prices: [`DEAREST`], the baseline's order).
            let first = v.pads.first().map(|p| p.good.clone());
            let others: Vec<String> = DEAREST
                .iter()
                .filter(|g| v.markets.iter().any(|m| m.good == **g))
                .filter(|g| v.buildings.iter().any(|b| b.active > 0 && b.produces.contains_key(**g)))
                .map(|g| g.to_string())
                .filter(|g| Some(g) != first.as_ref().and_then(|f| f.as_ref()))
                .collect();
            for (i, pad) in v.pads.iter().enumerate().skip(1) {
                let Some(good) = others.get(i - 1).or(first.as_ref().and_then(|f| f.as_ref())) else { continue };
                if pad.cargo == Fx::ZERO && pad.good.as_ref() != Some(good) {
                    sets.push(Action::SetPadGood { pad: i as u32, good: Some(good.clone()) });
                }
            }
            for a in sets {
                let _ = p.apply(&a);
            }
            return;
        }
        for (i, pad) in v.pads.iter().enumerate().skip(1) {
            if pad.cargo > Fx::ZERO {
                continue;
            }
            let full = v
                .markets
                .iter()
                .map(|m| m.good.clone())
                .filter(|g| g != "he3" && target.is_none_or(|t| price(v, t, g) == Fx::ZERO))
                .find(|g| cap(v, g).is_some_and(|c| amount(v, g) >= c));
            let good = full.unwrap_or_else(|| "he3".to_string());
            if pad.good.as_ref() != Some(&good) {
                sets.push(Action::SetPadGood { pad: i as u32, good: Some(good) });
            }
        }
        for a in sets {
            let _ = p.apply(&a);
        }
    }

    /// The programs' templates and processors; `gap` is the ticks to the next
    /// look.
    fn programs(&self, p: &mut Player, target: Option<&str>, gap: u64) {
        if self.cond && p.view().if_open {
            self.load_at_reset(p);
        }
        let v = p.view();
        let total = v.processors;
        let mut want: Vec<Vec<Entry>> = vec![Vec::new(); v.programs.len()];
        let mut procs = vec![0u32; v.programs.len()];
        let mut left = total;
        let e = Entry::command;
        let pads: Vec<_> = v.pads.iter().filter(|p| p.active).collect();
        // Holding boredom down, Dream runs first (programs run in order), so
        // loading can't take the He-3 and energy it needs.
        // With `main_last`, every other program runs before the main one.
        let (main, dream) = self.dir.program_slots(want.len());
        // Main: by hand while looks come often, else a standing cycle.
        let filler = if self.compute { "cloud_compute" } else { "idle" };
        // Condition programs once the game allows them: one fixed text each.
        // (Its templates guard with `if`, a global project's unlock.)
        let cond = self.cond && v.if_open;
        // The ship cycle falls short while the target waits on an input only
        // a purchase brings: then the main program is a hand plan that buys it.
        let ship = cond && !bridge_due(v, target);
        want[main] = if ship {
            ship_cycle(v, filler, self.ship_loads, self.fill)
        } else if gap <= self.hand_gap {
            // Twice what reaches the latest next look; a constant, so the text
            // changes only when what is due does.
            let fill = (2 * self.max_gap.max(gap) * u64::from(left.max(1))).min(1000) as u32;
            hand_plan(v, &pads, target, filler, fill, self.wait_demand, self.loads)
        } else {
            standing_cycle(v, &pads, target, filler, self.loads)
        };
        // Science for the next Blueprints on sale, ahead of the rest, from the
        // credits past the target's price (as the bridges buy).
        if let Some(short) = super::science_short(v) {
            let each = command_cost(v, "buy_science", "credits");
            let spare = amount(v, "credits") - target.map_or(Fx::ZERO, |t| price(v, t, "credits"));
            let n = if each > Fx::ZERO { (spare / each).floor_int().min(i64::from(short)) } else { 0 };
            if n > 0 {
                want[main].insert(0, e("buy_science", n as u32));
            }
        }
        // Dream, while the run still builds.
        let bored = v.boredom.amount.mul_int(100) >= v.boredom.cap.mul_int(self.dream_at);
        let building = v.day.saturating_sub(self.built_day) < self.stale_days;
        if let Some(tool) = self.dir.tool_command() {
            // Holding boredom down: Dream (or the committed axis's Theorize)
            // from a third of the cap ([`sustain_dreamers`] processors).
            let third = v.boredom.amount.mul_int(3) >= v.boredom.cap;
            // Theorize spends science: not while Fund's research (or the
            // Blueprints before it) waits on it.
            let science_free = tool != "theorize"
                || self.next_research(v).is_none_or(|r| r.affordable || !self.dir.fund_research().contains(&r.id));
            if third && science_free && command_unlocked(v, tool) && left > 1 && dream < want.len() {
                let n = sustain_dreamers(v, total).min(left - 1);
                want[dream] = vec![e(tool, 1)];
                procs[dream] = n;
                left -= n;
            }
        } else if self.dir.commit.is_some() {
            // Committed to an axis whose tool is a building: no Dream.
        } else if cond && building && command_unlocked(v, "dream") && left > 1 && dream < want.len() {
            // The Dream watch: dreams past `dream_at`% of the cap, else computes.
            want[dream] = dream_watch(self.dream_at, filler, self.dream_fill);
            procs[dream] = 1;
            left -= 1;
        } else if !cond && bored && building && command_unlocked(v, "dream") && left > 1 && dream < want.len() {
            want[dream] = vec![e("dream", 1)];
            procs[dream] = 1;
            left -= 1;
        }
        // Fund an ideology axis.
        if let Some((axis, fund_procs)) = self.dir.funded_axis() {
            let cmd = format!("fund_{axis}");
            if command_unlocked(v, &cmd) && left > 1 && FUND < want.len() {
                let n = fund_procs.min(left - 1);
                want[FUND] = vec![e(&cmd, 1)];
                procs[FUND] = n;
                left -= n;
            }
        } else if self.fund && left > 2 && FUND < want.len() {
            if let Some(axis) = own_axis(v) {
                let cmd = format!("fund_{axis}");
                if command_unlocked(v, &cmd) {
                    want[FUND] = vec![e(&cmd, 1)];
                    procs[FUND] = 1;
                    left -= 1;
                }
            }
        }
        if self.dir.market_runs(self.market, left) && !pads.is_empty() && MARKET < want.len() {
            let lines = market_program(v, &pads);
            if !lines.is_empty() {
                want[MARKET] = lines;
                procs[MARKET] = 1;
                left -= 1;
            }
        }
        // Construction: buy the target the day it comes within reach, between
        // looks. Its `if`s cost a step each (one when false), so the filler
        // computes less than it did while they were free (rev 19).
        // With conditions, the build order from the last runs, once there is one.
        if self.construct && cond && !self.plan.is_empty() && left > 1 && BUILD < want.len() && BUILD != main {
            want[BUILD] = self.plan.clone();
            procs[BUILD] = 1;
            left -= 1;
        } else if self.construct && v.if_open && left > 1 && BUILD < want.len() && BUILD != main {
            if let Some(b) = target.and_then(|t| v.buildings.iter().find(|b| b.id == t && b.unlocked)) {
                want[BUILD] = construction(v, b, filler);
                procs[BUILD] = 1;
                left -= 1;
            }
        }
        procs[main] = left;
        // A side program that loses its processor keeps its lines (idle with
        // none), so it needn't be written or loaded again when one comes back.
        if cond {
            let n = want.len();
            for i in [dream, FUND, MARKET, BUILD].into_iter().filter(|&i| i != main && i < n) {
                if want[i].is_empty() && procs[i] == 0 {
                    want[i] = v.programs[i].entries.clone();
                }
            }
        }
        let hand = !ship && gap <= self.hand_gap;
        let due: u64 = if hand {
            want[main].iter().filter(|l| l.command_id() != Some(filler)).map(|l| u64::from(l.count())).sum()
        } else {
            0
        };
        // Lower first, then raise, so the total never passes the processors.
        let cur: Vec<u32> = v.programs.iter().map(|p| p.processors).collect();
        // A hand plan is rewritten when its text changes, or to run its due
        // lines again once the pointer has passed them into the filler. One
        // with nothing due isn't written over a plan in its filler with
        // enough filler left to reach the next look: it would do the same.
        let in_filler = |i: usize| {
            let p = &v.programs[i];
            p.entries.len() > 1 && p.entry as usize + 1 >= p.entries.len()
        };
        let idle_same = |i: usize| {
            let p = &v.programs[i];
            let left_steps = p.entries.last().map_or(0, |l| u64::from(l.count().saturating_sub(p.step)));
            hand && i == main
                && due == 0
                && in_filler(i)
                && p.entries.last() == want[i].last()
                && left_steps >= gap * u64::from(procs[main].max(1))
        };
        let rearm = |i: usize| hand && i == main && due > 0 && in_filler(i);
        let writes: Vec<(usize, Vec<Entry>)> = (0..want.len())
            .filter(|&i| (v.programs[i].entries != want[i] && !idle_same(i)) || rearm(i))
            .map(|i| (i, want[i].clone()))
            .collect();
        // Standing programs go through the loadout library, so a new run loads
        // them back; a hand plan is one look's, not worth keeping, nor is a
        // construction program for one target once conditions are open.
        let role = |i: usize| match i {
            _ if ship && i == main => SHIP,
            _ if cond && i == main => "author: main",
            _ if cond && i == dream => DREAM_WATCH,
            _ if cond && i == FUND => FUND_NAME,
            _ if cond && i == MARKET => MARKET_NAME,
            _ if cond => BUILD_ORDER,
            _ if i == main => "author: main",
            _ if i == dream => "author: dream",
            FUND => "author: fund",
            MARKET => "author: market",
            _ => "author: build",
        };
        let one_target = |i: usize| cond && i == BUILD && want[i] != self.plan;
        for (i, entries) in writes {
            let keep = if (hand && i == main) || one_target(i) { None } else { Some(role(i)) };
            p.put_program(i as u32, entries, keep);
        }
        for i in (0..procs.len()).filter(|&i| procs[i] < cur[i]) {
            let _ = p.apply(&Action::SetProcessors { program: i as u32, processors: procs[i] });
        }
        for i in (0..procs.len()).filter(|&i| procs[i] > cur[i]) {
            let _ = p.apply(&Action::SetProcessors { program: i as u32, processors: procs[i] });
        }
    }

    /// At a new run (its side programs still empty), loads each saved side
    /// program into its slot: the Dream watch, fund, market, and the build
    /// order unless the last run rewrote it. Processors come as it gains them.
    fn load_at_reset(&self, p: &mut Player) {
        // The slots `programs` uses (a directive can move main and Dream).
        let (main, dream) = self.dir.program_slots(p.view().programs.len());
        let slots = [(dream, DREAM_WATCH), (FUND, FUND_NAME), (MARKET, MARKET_NAME), (BUILD, BUILD_ORDER)];
        for (slot, name) in slots.into_iter().filter(|&(s, _)| s != main) {
            let v = p.view();
            if v.programs.get(slot).is_none_or(|p| !p.entries.is_empty()) {
                continue;
            }
            let Some(i) = v.library.iter().position(|l| l.name == name) else { continue };
            if slot == BUILD && v.library[i].entries != self.plan {
                continue;
            }
            let _ = p.apply(&Action::LoadLoadout { loadout: i as u32, program: slot as u32 });
        }
    }

    /// The ticks until a decision is next useful, from what it can see: its
    /// target or the next research coming within reach (at its credit income
    /// and the view's rates), boredom reaching `dream_at`, the pad's cooldown
    /// ending, and the day a stale run would retire; at least `min_gap`, at
    /// most `max_gap` (what it can't foresee: events, demand, a stall).
    fn plan_gap(&self, v: &View, target: Option<&str>) -> u64 {
        let mut soon = u64::MAX;
        if let Some(b) = target.and_then(|t| v.buildings.iter().find(|b| b.id == t)) {
            if let Some(t) = super::ticks_to_afford(v, &b.cost, self.income.rate) {
                sooner(&mut soon, t);
            }
        }
        if let Some(r) = self.next_research(v) {
            let rate = v.resources.iter().find(|x| x.id == "science").map_or(Fx::ZERO, |x| x.change_per_day);
            if let Some(t) = super::ticks_to(amount(v, "science"), r.cost, rate) {
                sooner(&mut soon, t);
            }
        }
        if command_unlocked(v, "dream") {
            let at = v.boredom.cap.mul_ratio(self.dream_at, 100);
            if let Some(t) = super::ticks_to(v.boredom.amount, at, v.boredom.rate) {
                sooner(&mut soon, t);
            }
        }
        if let Some(pad) = v.pads.iter().find(|p| p.active && p.cooldown > 0) {
            sooner(&mut soon, u64::from(pad.cooldown));
        }
        if v.can_retire && self.dir.retires_stale(v) {
            let stale = v.day.saturating_sub(self.built_day);
            sooner(&mut soon, self.stale_days.saturating_sub(stale));
        }
        soon.clamp(self.min_gap, self.max_gap)
    }
}

/// Keeps the sooner of `soon` and `t`; a `t` of 0 (it holds already, yet this
/// look didn't act on it) gives no reason to look again sooner.
fn sooner(soon: &mut u64, t: u64) {
    if t > 0 {
        *soon = (*soon).min(t);
    }
}

fn price(v: &View, id: &str, res: &str) -> Fx {
    v.buildings
        .iter()
        .find(|b| b.id == id)
        .and_then(|b| b.cost.get(res).copied())
        .unwrap_or(Fx::ZERO)
}

/// Whether `b` costs more of some capped resource than its cap holds: it
/// can't be bought until the cap rises.
fn over_cap(v: &View, b: &hh_core::view::BuildingView) -> bool {
    b.cost.iter().any(|(res, &need)| cap(v, res).is_some_and(|c| need > c))
}


impl Agent for Author {
    fn next_look(&mut self, _p: &mut Player) -> u64 {
        self.gap
    }

    fn note(&self) -> String {
        self.last_note.clone()
    }

    fn act(&mut self, p: &mut Player) {
        let (run, day, built) = {
            let v = p.view();
            self.income.read(v);
            self.b.reserve = self.dir.energy_reserve(self.reserve, v);
            (v.run, v.day, v.run_stats.buildings_built)
        };
        self.track(p.view());
        if run != self.run || built != self.built {
            self.run = run;
            self.built = built;
            self.built_day = day;
        }
        let v = p.view();
        if v.can_retire && self.dir.retires_stale(v) && day.saturating_sub(self.built_day) >= self.stale_days {
            let _ = p.apply(&Action::Retire);
            let v = p.view();
            self.run = v.run;
            self.built = v.run_stats.buildings_built;
            self.built_day = v.day;
            self.track(p.view());
            if self.stop_at_retire {
                return;
            }
        }
        self.research(p);
        let target = self.build(p);
        let v = p.view();
        if v.run_stats.buildings_built != self.built {
            self.built = v.run_stats.buildings_built;
            self.built_day = v.day;
        }
        // How long until it looks again decides how it writes programs.
        let gap = self.plan_gap(p.view(), target.as_deref());
        // Pausing pays only while it looks often enough to undo it.
        let paused = if gap <= self.hand_gap { target.as_deref() } else { None };
        self.pause_for(p, paused);
        self.projects(p, target.as_deref());
        super::pick_cargo(p);
        super::hand_launch(p);
        self.pads(p, target.as_deref());
        self.programs(p, target.as_deref(), gap);
        self.gap = gap;
        // Last, so its project funding (the Dyson Sphere) outlives the
        // author's own stop of every project but its pick.
        super::arc::act(p);
        let v = p.view();
        self.last_note = format!(
            "target {}, credits {}, processors {}, gap {gap}",
            target.as_deref().unwrap_or("-"),
            amount(v, "credits").floor_int(),
            v.processors
        );
    }
}
