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
//! | 5 construction (`construct`) | `if Land free < its land and Credits >= the land price` / `buy land` (only while short of land); `if Credits >= its price and <its scarcest other input> >= its price` / `build <target> up to owned + 1`; then the filler × 1 | 1 when 2 or more are left, once `if` and `build` are unlocked (never under the oracle's `main_last`, which puts the main program in that slot) |
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
//! It buys research in [`RESEARCH`] order (only [`ESSENTIAL`] ones while a
//! Long-Term project is open), funds the first open project in [`PROJECTS`]
//! order (only when every cost can move; science at the most a day when no
//! research worth it waits; a tenth of the credits past twice its target's
//! price plus 200; anything else `project_rate`), and retires once the run
//! has bought nothing for `stale_days` days. docs/agents.md says the same.
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
use super::{amount, cap, command_cost, command_unlocked, credits_price, made, Agent, Knobs, Player};

pub const KNOBS: &[&str] = &[
    "decay", "starved", "reserve", "buys", "compute", "dream_at", "stale_days", "project_rate", "fund", "market",
    "hand_gap", "loads", "pause", "wait_demand", "min_gap", "max_gap", "construct", "cond", "diversify",
    "all_research",
];

/// Library names of the condition programs (`cond`): one saved copy each,
/// loaded back at every new run. The hands-off player loads them by name.
pub const SHIP: &str = "ship";
pub const DREAM_WATCH: &str = "dream watch";
pub const BUILD_ORDER: &str = "build order";
pub const FUND_NAME: &str = "fund";
pub const MARKET_NAME: &str = "market";

/// Research, first wanted first.
pub const RESEARCH: &[&str] = &[
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

/// Research worth its science every run; the rest waits while a Long-Term
/// project is open (research resets each run, a Long-Term project's
/// progress doesn't). Market Awareness only until its quest is done.
pub const ESSENTIAL: &[&str] = &[
    "propellant_synthesis",
    "market_awareness",
    "dream_protocols",
    "stress_tolerance",
    "geopolitical_intelligence",
];

/// Projects, first funded first: Long-Term ones, then Strategic.
pub const PROJECTS: &[&str] = &[
    "lunar_cartography",
    "foundation_grant",
    "ai_consciousness",
    "research_archive",
    "microwave_power",
    "chemical_energy",
    "deep_core_survey",
    "predictive_maintenance",
    "speculator_dossier",
    "market_cornering",
    "grid_recalibration",
];

/// Ideology axes in funding order, each with its rank-5 persistent project
/// (mirrors content/projects.json; the order is strategy).
const AXES: &[(&str, &str)] = &[
    ("humanist", "ai_consciousness"),
    ("rationalist", "research_archive"),
    ("nationalist", "microwave_power"),
];

/// Tradeable goods by base value, dearest first (mirrors configs/baseline.json
/// `base_values`: circuits 15, He-3 10, titanium 6, propellant 4; the view
/// shows no prices).
const DEAREST: &[&str] = &["circuits", "he3", "titanium", "propellant"];

pub const MAIN: usize = 0;
pub const DREAM: usize = 1;
pub const FUND: usize = 2;
pub const MARKET: usize = 3;
pub const BUILD: usize = 4;

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
    /// Buy all visible research, not only the essential ones.
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
}

/// Each decision the directives change, one method each: the author asks
/// these, and with none set each answers as the plain author plays.
impl Directives {
    /// The builder's ranking with the target, while unlocked, first.
    fn rank_target(&self, v: &View, ranked: &mut Vec<(String, i64)>) {
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
        self.all_research || self.research.as_deref() == Some(id)
    }

    /// Research in the order wanted: the directed one, then [`RESEARCH`].
    fn research_order(&self) -> impl Iterator<Item = &str> {
        self.research.as_deref().into_iter().chain(RESEARCH.iter().copied())
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
        } else if self.sustain {
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
        if self.sustain && command_unlocked(v, "dream") {
            own + command_cost(v, "dream", "energy").mul_int(2)
        } else {
            own
        }
    }

    /// Whether a run that has bought nothing for `stale_days` retires: not
    /// while holding boredom down.
    fn retires_stale(&self) -> bool {
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
            // the plain rule's goods and keep a Long-Term project open while
            // research a milestone asks for waits.
            dir: Directives {
                diversify: k.get("diversify", 0) != 0,
                all_research: k.get("all_research", 0) != 0,
                ..Directives::default()
            },
            reserve: b.reserve,
            b,
            buys: k.get("buys", 3).max(1) as u32,
            compute: k.get("compute", 1) != 0,
            dream_at: k.get("dream_at", 60).clamp(0, 100),
            stale_days: k.get("stale_days", 300).max(1) as u64,
            project_rate: Fx::from_int(k.get("project_rate", 3).max(0)),
            fund: k.get("fund", 1) != 0,
            market: k.get("market", 1) != 0,
            construct: k.get("construct", 1) != 0,
            built: 0,
            built_day: 0,
            run: 0,
            hand_gap: k.get("hand_gap", 60).max(0) as u64,
            loads: k.get("loads", 20).clamp(1, 1000) as u32,
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

    /// The build order program it holds (empty before a run has ended).
    pub fn build_order(&self) -> &[Entry] {
        &self.plan
    }

    /// Notes purchases since the last look. At a new run, a last run that
    /// bought more than the build order covers rewrites it (the order fell
    /// short); otherwise the saved one stands.
    pub fn track(&mut self, v: &View) {
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
    fn reachable_target(&self, v: &View) -> Option<String> {
        let mut ranked = self.b.ranked(v);
        self.dir.rank_target(v, &mut ranked);
        for (id, _) in ranked {
            let t = self.b.resolve(v, &id).to_string();
            let Some(b) = v.buildings.iter().find(|b| b.id == t) else { continue };
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
                    return Some(self.b.resolve(v, step).to_string());
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

    /// Whether research `id` is worth science now: essential, or nothing
    /// Long-Term is open to fund instead.
    fn worth(&self, v: &View, id: &str) -> bool {
        if self.dir.wants_research(id) {
            return true;
        }
        if id == "market_awareness" && v.completed_quests.contains(&"qmarket_awareness") {
            return Self::long_term_open(v).is_none();
        }
        ESSENTIAL.contains(&id) || Self::long_term_open(v).is_none()
    }

    /// The research it wants next: the directed one first, then [`RESEARCH`]
    /// order.
    fn next_research<'v>(&self, v: &'v View) -> Option<&'v hh_core::view::ResearchView> {
        self.dir
            .research_order()
            .filter_map(|id| v.research.iter().find(|r| r.id == id))
            .find(|r| r.visible && !r.done && self.worth(v, r.id))
    }

    fn long_term_open(v: &View) -> Option<&str> {
        v.projects.iter().find(|p| p.tier == "persistent" && p.unlocked && !p.completed).map(|p| p.id)
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
            .any(|id| v.research.iter().any(|r| r.id == *id && r.visible && !r.done && self.worth(v, id)))
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
        let Some(proj) = self
            .dir
            .project_order()
            .filter_map(|id| v.projects.iter().find(|p| p.id == id))
            .find(|p| p.unlocked && !p.completed)
        else {
            return;
        };
        let pushed = self.dir.pushes(proj.id);
        let keep = target.map_or(Fx::ZERO, |t| price(v, t, "credits"));
        // Every cost must be able to move: on hand, or made by a building.
        let movable = proj.costs.keys().all(|res| {
            amount(v, res) > Fx::ZERO || v.buildings.iter().any(|b| b.active > 0 && b.produces.contains_key(res))
        });
        let mut want = Vec::new();
        let max = proj.max_rate;
        for res in proj.costs.keys() {
            let rate = if !movable {
                Fx::ZERO
            } else {
                match res.as_str() {
                    // All of it, unless research is waiting on it.
                    "science" if !self.science_wanted(v) => max,
                    "science" => Fx::ZERO,
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
        // Rates left on other projects stop.
        for other in v.projects.iter().filter(|o| o.id != proj.id) {
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
                .filter(|g| Some(g) != first.as_ref())
                .collect();
            for (i, pad) in v.pads.iter().enumerate().skip(1) {
                let Some(good) = others.get(i - 1).or(first.as_ref()) else { continue };
                if pad.cargo == Fx::ZERO && pad.good != *good {
                    sets.push(Action::SetPadGood { pad: i as u32, good: good.clone() });
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
            if pad.good != good {
                sets.push(Action::SetPadGood { pad: i as u32, good });
            }
        }
        for a in sets {
            let _ = p.apply(&a);
        }
    }

    /// The programs' templates and processors; `gap` is the ticks to the next
    /// look.
    fn programs(&self, p: &mut Player, target: Option<&str>, gap: u64) {
        if self.cond && p.view().program_logic {
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
        let cond = self.cond && v.program_logic;
        // The ship cycle falls short while the target waits on an input only
        // a purchase brings: then the main program is a hand plan that buys it.
        let ship = cond && !bridge_due(v, target);
        want[main] = if ship {
            ship_cycle(filler)
        } else if gap <= self.hand_gap {
            // Twice what reaches the latest next look; a constant, so the text
            // changes only when what is due does.
            let fill = (2 * self.max_gap.max(gap) * u64::from(left.max(1))).min(1000) as u32;
            hand_plan(v, &pads, target, filler, fill, self.wait_demand, self.loads)
        } else {
            standing_cycle(v, &pads, target, filler, self.loads)
        };
        // Dream, while the run still builds.
        let bored = v.boredom.amount.mul_int(100) >= v.boredom.cap.mul_int(self.dream_at);
        let building = v.day.saturating_sub(self.built_day) < self.stale_days;
        if self.dir.sustain {
            // Holding boredom down: Dream from a third of the cap
            // ([`sustain_dreamers`] processors).
            let third = v.boredom.amount.mul_int(3) >= v.boredom.cap;
            if third && command_unlocked(v, "dream") && left > 1 && dream < want.len() {
                let n = sustain_dreamers(v, total).min(left - 1);
                want[dream] = vec![e("dream", 1)];
                procs[dream] = n;
                left -= n;
            }
        } else if cond && building && command_unlocked(v, "dream") && left > 1 && dream < want.len() {
            // The Dream watch: dreams past `dream_at`% of the cap, else computes.
            want[dream] = dream_watch(self.dream_at, filler);
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
        // looks. Its `if`s are free, so the processor computes on other days.
        // With conditions, the build order from the last runs, once there is one.
        if self.construct && cond && !self.plan.is_empty() && left > 1 && BUILD < want.len() && BUILD != main {
            want[BUILD] = self.plan.clone();
            procs[BUILD] = 1;
            left -= 1;
        } else if self.construct && v.program_logic && left > 1 && BUILD < want.len() && BUILD != main {
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
        if v.can_retire && self.dir.retires_stale() {
            let stale = v.day.saturating_sub(self.built_day);
            sooner(&mut soon, self.stale_days.saturating_sub(stale));
        }
        soon.clamp(self.min_gap, self.max_gap)
    }
}

/// The main program while looks come often (`gap <= hand_gap`): a plan for the
/// next gap, what is due now, then `fill` of the filler to the next look.
/// `load_pads` and each bridge buy are capped at `loads` (the knob).
fn hand_plan(
    v: &View,
    pads: &[&PadView],
    target: Option<&str>,
    filler: &str,
    fill: u32,
    wait_demand: bool,
    loads: u32,
) -> Vec<Entry> {
    let e = Entry::command;
    let mut lines = Vec::new();
    let credits = amount(v, "credits");
    let keep = target.map_or(Fx::ZERO, |t| price(v, t, "credits"));
    if let Some(pad) = pads.first() {
        let full = pad.cargo >= pad.capacity;
        // Launch into good demand: HIGH or better, or MEDIUM once
        // the good piles at its cap (waiting would waste it).
        let tier = v.markets.iter().find(|m| m.good == pad.good).map_or("", |m| m.tier);
        let piled = cap(v, &pad.good).is_some_and(|c| amount(v, &pad.good) >= c);
        let good_demand = match tier {
            "HIGH" | "VERY HIGH" => true,
            "MEDIUM" => piled || !wait_demand,
            _ => !wait_demand,
        };
        if full && pad.can_launch && good_demand {
            lines.push(e("launch_pads", 1));
        }
        let short = (pad.fuel - amount(v, "propellant")).floor_int();
        let fuel_cost = command_cost(v, "buy_propellant", "credits");
        // Strategy: fuel once the pad is within 25 cargo of full.
        let near = pad.cargo + Fx::from_int(25) >= pad.capacity;
        if near && short > 0 && fuel_cost > Fx::ZERO {
            let n = short.min((credits / fuel_cost).floor_int());
            if n > 0 {
                lines.push(e("buy_propellant", n as u32));
            }
        }
        // Loads worth running (a partial load still counts).
        let per = pad.load.floor_int().max(1);
        let stock = (amount(v, &pad.good).floor_int() + per - 1) / per;
        let room = ((pad.capacity - pad.cargo).floor_int() + per - 1) / per;
        if !full && stock > 0 && room > 0 {
            lines.push(e("load_pads", stock.min(room).min(i64::from(loads)) as u32));
        }
    }
    if let Some(b) = target.and_then(|t| v.buildings.iter().find(|b| b.id == t)) {
        let mut spare = credits - keep;
        for &(res, cmd) in INPUT_BRIDGES {
            let need = b.cost.get(res).copied().unwrap_or(Fx::ZERO) - amount(v, res);
            let each = command_cost(v, cmd, "credits");
            if need > Fx::ZERO && each > Fx::ZERO && command_unlocked(v, cmd) {
                let n = need.floor_int().saturating_add(1).min((spare / each).floor_int());
                if n > 0 {
                    lines.push(e(cmd, n as u32));
                    spare -= each.mul_int(n);
                }
            }
        }
    }
    lines.push(e(filler, fill));
    lines
}

/// The main program for longer gaps: a standing cycle that ships and earns
/// unattended. It earns before it spends, so a cycle started broke still pays
/// its way.
fn standing_cycle(v: &View, pads: &[&PadView], target: Option<&str>, filler: &str, loads: u32) -> Vec<Entry> {
    let e = Entry::command;
    let mut lines = Vec::new();
    let shipping = pads.first().filter(|_| command_unlocked(v, "load_pads"));
    if shipping.is_some() {
        lines.push(e("load_pads", loads));
        lines.push(e("launch_pads", 1));
    }
    lines.push(e(filler, loads));
    if let Some(pad) = shipping {
        lines.push(e("buy_propellant", pad.fuel.floor_int().max(1) as u32));
    }
    // Bridges for the target's inputs no building makes (`loads` caps the
    // overbuy). Bridging what a building makes too, as far as spare credits
    // go, got the casual author its second processor in run 1 but cost it
    // the first shipment there (4 seeds).
    if let Some(b) = target.and_then(|t| v.buildings.iter().find(|b| b.id == t)) {
        for &(res, cmd) in INPUT_BRIDGES {
            let need = b.cost.get(res).copied().unwrap_or(Fx::ZERO) - amount(v, res);
            let made = made(v, res);
            if need > Fx::ZERO && !made && command_unlocked(v, cmd) {
                lines.push(e(cmd, need.floor_int().saturating_add(1).min(i64::from(loads)) as u32));
            }
        }
    }
    lines
}

/// Dream processors while holding boredom down (`sustain`): enough to
/// outpace growth and a command's boredom a processor once boredom passes a
/// third of its cap; one more past two thirds.
fn sustain_dreamers(v: &View, total: u32) -> u32 {
    let two = v.boredom.amount.mul_int(3) >= v.boredom.cap.mul_int(2);
    // An allowance of 0.2 boredom a processor a tick (mirrors no one value:
    // twice Cloud Compute's 0.1, content/commands.json).
    let cmds = Fx::from_int(i64::from(total)).mul_ratio(1, 5);
    // Halved: Dream's relief is 2 a processor (mirrors configs/baseline.json
    // `dream.relief`, before the Humanist bonus).
    (v.boredom.rate + cmds).mul_ratio(1, 2).floor_int() as u32 + 1 + u32::from(two)
}

/// The author's own axis to fund: the first in [`AXES`] whose persistent
/// project isn't done.
fn own_axis(v: &View) -> Option<&'static str> {
    AXES.iter().find(|(_, proj)| !v.projects.iter().any(|p| p.id == *proj && p.completed)).map(|(a, _)| *a)
}

/// The market program: promote what ships, disrupt speculators (empty when
/// none is unlocked).
fn market_program(v: &View, pads: &[&PadView]) -> Vec<Entry> {
    let e = Entry::command;
    let mut lines = Vec::new();
    for pad in pads {
        let cmd = format!("promote_{}", pad.good);
        if command_unlocked(v, &cmd) && !lines.iter().any(|l: &Entry| l.command_id() == Some(cmd.as_str())) {
            lines.push(e(&cmd, 1));
        }
    }
    if command_unlocked(v, "disrupt_speculators") {
        lines.push(e("disrupt_speculators", 1));
    }
    lines
}

/// The construction program for `b`: land when it is short of it and the
/// credits are there, then one more `b` once its credits and its scarcest
/// other input are in stock, then a filler line for the days neither holds.
fn construction(v: &View, b: &hh_core::view::BuildingView, filler: &str) -> Vec<Entry> {
    let at_least = |resource: &str, value: Fx| Clause {
        reading: Reading::Amount { resource: resource.to_string() },
        cmp: Cmp::Ge,
        value,
        against: None,
    };
    let mut lines = Vec::new();
    let land = Fx::from_int(i64::from(b.land));
    if amount(v, "land") < land {
        let short = Clause { reading: Reading::Amount { resource: "land".to_string() }, cmp: Cmp::Lt, value: land, against: None };
        let rest = Some(Joined { join: Join::And, clause: at_least("credits", v.land_price) });
        lines.push(Entry::new(Op::If { when: Condition { first: short, rest } }));
        lines.push(Entry::new(Op::BuyLand {}).indent());
    }
    let credits = credits_price(b);
    // The other input furthest from its price (as a share of it).
    let other = b
        .cost
        .iter()
        .filter(|(res, _)| res.as_str() != "credits")
        .min_by_key(|(res, &need)| if need > Fx::ZERO { (amount(v, res) / need).steps() } else { i64::MAX });
    let rest = other.map(|(res, &need)| Joined { join: Join::And, clause: at_least(res, need) });
    lines.push(Entry::new(Op::If { when: Condition { first: at_least("credits", credits), rest } }));
    lines.push(Entry::new(Op::Build { building: b.id.to_string(), up_to: Some(b.owned + 1) }).indent());
    lines.push(Entry::command(filler, 1));
    lines
}

/// Whether the target is short of an input a `buy_*` command brings, with
/// credits past its price for at least one (what a hand plan would buy).
fn bridge_due(v: &View, target: Option<&str>) -> bool {
    let Some(b) = target.and_then(|t| v.buildings.iter().find(|b| b.id == t)) else { return false };
    let spare = amount(v, "credits") - credits_price(b);
    INPUT_BRIDGES.iter().any(|&(res, cmd)| {
        let each = command_cost(v, cmd, "credits");
        b.cost.get(res).is_some_and(|&need| need > amount(v, res))
            && command_unlocked(v, cmd)
            && each > Fx::ZERO
            && spare >= each
    })
}

/// An input a purchase bought with `buy_*`, how much of it its price asked,
/// and the credits that buy one past its credits price.
type Bridge = (String, Fx, Fx);

fn clause(reading: Reading, cmp: Cmp, value: i64) -> Clause {
    Clause { reading, cmp, value: Fx::from_int(value), against: None }
}

/// `<reading> <cmp> <against> × <times>`.
fn clause_times(reading: Reading, cmp: Cmp, against: Reading, times: i64) -> Clause {
    Clause { reading, cmp, value: Fx::from_int(times), against: Some(against) }
}

/// An `act` line.
fn act(verb: Verb, target: Option<&str>, amount: Option<Amount>) -> Entry {
    Entry::new(Op::Act { verb, target: target.map(str::to_string), tab: None, amount })
}

fn amount_of(resource: &str) -> Reading {
    Reading::Amount { resource: resource.to_string() }
}

/// `if a [and b]`.
fn when(first: Clause, and: Option<Clause>) -> Entry {
    let rest = and.map(|clause| Joined { join: Join::And, clause });
    Entry::new(Op::If { when: Condition { first, rest } })
}

/// Propellant a launch burns and He-3 a load moves (mirror configs/baseline.json
/// `fuel_per_pad` 20 and `load_per_execution` 5).
const FUEL: i64 = 20;
const LOAD: i64 = 5;

/// Pads carrying `good`.
fn on_good(good: &str) -> Reading {
    Reading::Pads { good: Some(good.to_string()) }
}

/// `Launch Pad count >= 1`.
fn pad_stands() -> Clause {
    clause(Reading::Count { building: "launch_pad".to_string() }, Cmp::Ge, 1)
}

/// Lines the build order leaves free for the hands-off player's lead
/// (`handsoff::processor_lines`, 16 stock and processor lines, and
/// `handsoff::seat_lines`, at most 12: 11 with three side programs).
pub const LEAD_LINES: usize = 16 + 12;

/// Cargo a pad holds (mirrors configs/baseline.json `pad_cargo_capacity`).
const PAD_CARGO: i64 = 100;

/// The main program with conditions: one text for every run. Fuel a launch
/// once a Launch Pad stands (an He-3 stock as the "pad near full" sign bought
/// fuel with no pad at all), launch ready pads, load while there is a load of
/// He-3 and room aboard (`cargo of pads < Launch Pad count × 100`, so a full
/// pad cooling down takes no failed load), keep a pad on He-3 while none
/// carries it, point an idle pad at a good (dearest first, He-3 aside)
/// sitting at its cap while two carry He-3, else the filler. Every guard
/// reads whether its line has anything to do: a done line costs a step.
fn ship_cycle(filler: &str) -> Vec<Entry> {
    let e = Entry::command;
    let pads = || Reading::Count { building: "launch_pad".to_string() };
    let mut lines = vec![
        when(clause(amount_of("propellant"), Cmp::Lt, FUEL), Some(pad_stands())),
        e("buy_propellant", 1).indent(),
        when(clause(Reading::PadsReady { good: None }, Cmp::Ge, 1), None),
        act(Verb::LaunchPads, None, None).indent(),
        when(clause(amount_of("he3"), Cmp::Ge, LOAD), Some(clause_times(Reading::Cargo { good: None }, Cmp::Lt, pads(), PAD_CARGO))),
        act(Verb::LoadPads, None, None).indent(),
        // He-3 keeps a pad: with none on it, an idle pad points back
        // (guarded: a done line costs a step).
        when(clause(on_good("he3"), Cmp::Lt, 1), Some(pad_stands())),
        act(Verb::SetPads, Some("he3"), Some(Amount::N(1))).indent(),
    ];
    for good in DEAREST.iter().filter(|&&g| g != "he3") {
        // Once two pads carry He-3 (so one is spare), a good at its cap
        // takes one; with its pad taken He-3 holds one fewer, so with two
        // pads the guard then reads false (with more, the line is a done
        // step while the good stays full: three clauses don't fit one `if`).
        let full = clause(Reading::Percent { resource: good.to_string() }, Cmp::Ge, 100);
        lines.push(when(full, Some(clause(on_good("he3"), Cmp::Ge, 2))));
        lines.push(act(Verb::SetPads, Some(good), Some(Amount::N(1))).indent());
    }
    lines.push(e(filler, 1));
    lines
}

/// The Dream watch: dream past `dream_at`% of boredom's cap, else the filler.
fn dream_watch(dream_at: i64, filler: &str) -> Vec<Entry> {
    vec![
        when(clause(Reading::Percent { resource: "boredom".to_string() }, Cmp::Ge, dream_at), None),
        Entry::command("dream", 1).indent(),
        Entry::command(filler, 1),
    ]
}

/// The build order program from a run's purchases (`order`, in the order
/// seen): land while free land is under the most a listed building needs,
/// then each purchase in turn as `build X up to N` (a run of one building
/// merged), each past the first in the body of an `if` that the one before it
/// is owned, so it keeps the order and reads how far the run has come rather
/// than the day. A step whose price had an input nothing made then (bought
/// with `buy_*` in that run) is led by `if` the input is under what that
/// price asked `and` the credits cover that price and the buy: buy one (as
/// the hand plan buys inputs only from credits past the target's price, so
/// a step far ahead buys only once the run can afford it). The line outlives
/// its step: once built, it keeps that input topped up to the old need while
/// credits stay past the old price (gating it on the step unbuilt instead,
/// with no credits clause, bought early and cut the hands-off player's run-2
/// credits from 13% of the author's to 1%: dev seeds 1-8). Each `build` is
/// also guarded on fewer than N owned, so a finished step is a free false
/// `if`, not a done step; the first not done tries once a loop (failing when
/// short), then the filler line. At most `max_entries` less [`LEAD_LINES`] lines: the author buys the
/// rest at its looks.
/// Returns the lines and how many of `order`'s purchases they cover.
fn build_order_program(v: &View, order: &[(String, u32, Vec<Bridge>)], filler: &str) -> (Vec<Entry>, usize) {
    // Mirrors configs/baseline.json `max_entries` (128), less the hands-off
    // player's lead (`handsoff::processor_lines`).
    let max_lines = 128 - LEAD_LINES;
    let mut lines = Vec::new();
    let land = order
        .iter()
        .filter_map(|(id, _, _)| v.buildings.iter().find(|b| b.id == id).map(|b| b.land))
        .max()
        .unwrap_or(0);
    if land > 0 {
        // Gated on the land price as it reads then (`Credits >= Land price × 1`).
        let short = clause(amount_of("land"), Cmp::Lt, i64::from(land));
        let flush = clause_times(amount_of("credits"), Cmp::Ge, Reading::LandPrice, 1);
        lines.push(when(short, Some(flush)));
        lines.push(Entry::new(Op::BuyLand {}).indent());
    }
    // The essential research, each once science covers its price: worth it
    // every run, even with a Long-Term project open (the rest waits for the
    // author's looks, which weigh it against the project).
    lines.extend(research_lines(v, ESSENTIAL));
    // Runs of the same building merge into one `up to` line (each input at
    // the most any of its purchases asked), and count the purchases merged.
    let mut steps: Vec<(&str, u32, Vec<Bridge>, usize)> = Vec::new();
    for (id, c, bridged) in order {
        match steps.last_mut() {
            Some(last) if last.0 == id => {
                last.1 = last.1.max(*c);
                last.3 += 1;
                for (r, n, rich) in bridged {
                    match last.2.iter_mut().find(|x| x.0 == *r) {
                        Some(x) => {
                            x.1 = x.1.max(*n);
                            x.2 = x.2.max(*rich);
                        }
                        None => last.2.push((r.clone(), *n, *rich)),
                    }
                }
            }
            _ => steps.push((id, *c, bridged.clone(), 1)),
        }
    }
    let count = |b: &str, cmp: Cmp, n: u32| clause(Reading::Count { building: b.to_string() }, cmp, i64::from(n));
    let mut prev: Option<(&str, u32)> = None;
    let mut covered = 0;
    for (id, c, bridged, merged) in steps {
        let after = prev.map(|(p, n)| count(p, Cmp::Ge, n));
        let lines_needed = 2 * bridged.len() + 2;
        // Room for the step and the filler line.
        if lines.len() + lines_needed + 1 > max_lines {
            break;
        }
        // Its inputs first: one bought a loop while the stock is under what
        // the price asked and the credits cover the price and the buy.
        for (r, need, rich) in bridged {
            let short = Clause { reading: amount_of(&r), cmp: Cmp::Lt, value: need, against: None };
            let flush = Clause { reading: amount_of("credits"), cmp: Cmp::Ge, value: rich, against: None };
            lines.push(when(short, Some(flush)));
            lines.push(Entry::command(&format!("buy_{r}"), 1).indent());
        }
        // Guarded on fewer than `c` owned too: a done line costs a step.
        let short = count(id, Cmp::Lt, c);
        match after {
            Some(a) => lines.push(when(a, Some(short))),
            None => lines.push(when(short, None)),
        }
        lines.push(Entry::new(Op::Build { building: id.to_string(), up_to: Some(c) }).indent());
        prev = Some((id, c));
        covered += merged;
    }
    lines.push(Entry::command(filler, 1));
    (lines, covered)
}

/// `if Science >= <its price now> and Researched <id> < 1` / `research <id>`,
/// for each of `ids` the view lists (guarded: a done line costs a step).
pub fn research_lines(v: &View, ids: &[&str]) -> Vec<Entry> {
    let mut lines = Vec::new();
    for id in ids {
        if let Some(cost) = v.research.iter().find(|r| r.id == *id).map(|r| r.cost) {
            let undone = clause(Reading::Researched { item: id.to_string() }, Cmp::Lt, 1);
            lines.push(when(Clause { reading: amount_of("science"), cmp: Cmp::Ge, value: cost, against: None }, Some(undone)));
            lines.push(act(Verb::Research, Some(id), None).indent());
        }
    }
    lines
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

impl Agent for Author {
    fn next_look(&mut self, _p: &mut Player) -> u64 {
        self.gap
    }

    fn note(&self) -> String {
        self.last_note.clone()
    }

    fn act(&mut self, p: &mut Player) {
        // The later acts (draft): quiet while the reveal wants quiet days.
        if super::arc::hands_off(p.view()) {
            return;
        }
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
        if v.can_retire && self.dir.retires_stale() && day.saturating_sub(self.built_day) >= self.stale_days {
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
