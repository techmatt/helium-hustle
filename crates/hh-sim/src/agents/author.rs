//! `author`: the heuristic program author, the rung that matters most
//! (program authoring is the core skill). It builds with the builder's scored
//! order but saves for its target (buying another only from credits past the
//! target's price), and writes programs from templates. How it writes the main
//! program depends on how often it gets to look (the ticks since its last
//! look, `gap`):
//!
//! | Program | Template | Processors |
//! |---|---|---|
//! | 1 main, `gap <= hand_gap` | a plan to the next look: `launch_pads` if a pad is full; `buy_propellant` × the fuel short when a pad is within 25 of full; `load_pads` × the loads that would move cargo; `buy_*` × the target's missing titanium, regolith or ice, as far as credits past its price go; then `cloud_compute` × gap × processors | the rest |
//! | 1 main, longer gaps | a standing cycle: `load_pads × loads, launch_pads, cloud_compute × loads, buy_propellant × fuel`, `buy_*` for target inputs no building makes | the rest |
//! | 2 dream | `dream` | 1 while boredom is past `dream_at`% of its cap and the run still builds |
//! | 3 fund | `fund_<axis>`: the first axis (humanist, rationalist, nationalist) whose persistent project isn't done | 1 when 3 or more are left |
//! | 4 market | `promote_<good>` for each pad good, `disrupt_speculators` | 1 when 3 or more are left |
//!
//! A program is rewritten only when its text changes (a rewrite restarts its
//! pointer). Extra pads carry a good other than He-3 that sits at its cap.
//! While looking often, it pauses buildings that eat a material its target is
//! short of, so the stock builds up.
//!
//! It buys research in [`RESEARCH`] order (only [`ESSENTIAL`] ones while a
//! Long-Term project is open), funds the first open project in [`PROJECTS`]
//! order (only when every cost can move; science at the most a day when no
//! research worth it waits; a tenth of the credits past twice its target's
//! price plus 200; anything else `project_rate`), and retires once the run
//! has bought nothing for `stale_days` days. docs/agents.md says the same.

use hh_core::content::Requirement;
use hh_core::state::Entry;
use hh_core::{Action, Fx, View};

use super::builder::{Builder, BRIDGES};
use super::{amount, cap, command_cost, command_unlocked, Agent, Knobs, Player};

pub const KNOBS: &[&str] = &[
    "decay", "starved", "reserve", "buys", "compute", "dream_at", "stale_days", "project_rate", "fund", "market",
    "hand_gap", "loads", "pause", "wait_demand",
];

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

/// Ideology axes in funding order, each with its rank-5 persistent project.
const AXES: &[(&str, &str)] = &[
    ("humanist", "ai_consciousness"),
    ("rationalist", "research_archive"),
    ("nationalist", "microwave_power"),
];

const MAIN: usize = 0;
const DREAM: usize = 1;
const FUND: usize = 2;
const MARKET: usize = 3;

pub struct Author {
    b: Builder,
    buys: u32,
    compute: bool,
    dream_at: i64,
    stale_days: u64,
    project_rate: Fx,
    fund: bool,
    market: bool,
    /// Buildings built this run at the last look, and the day that count last rose.
    built: u32,
    built_day: u64,
    run: u32,
    hand_gap: u64,
    loads: u32,
    last_tick: Option<u64>,
    pause: bool,
    wait_demand: bool,
    last_note: String,
}

impl Author {
    pub fn new(k: &Knobs) -> Self {
        Author {
            b: Builder::new(k),
            buys: k.get("buys", 3).max(1) as u32,
            compute: k.get("compute", 1) != 0,
            dream_at: k.get("dream_at", 60).clamp(0, 100),
            stale_days: k.get("stale_days", 300).max(1) as u64,
            project_rate: Fx::from_int(k.get("project_rate", 3).max(0)),
            fund: k.get("fund", 1) != 0,
            market: k.get("market", 1) != 0,
            built: 0,
            built_day: 0,
            run: 0,
            hand_gap: k.get("hand_gap", 60).max(0) as u64,
            loads: k.get("loads", 20).clamp(1, 1000) as u32,
            last_tick: None,
            pause: k.get("pause", 1) != 0,
            wait_demand: k.get("wait_demand", 0) != 0,
            last_note: String::new(),
        }
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
                (b.active != want).then(|| Action::SetActive { building: b.id.clone(), active: want })
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
                        b.id == *id && b.affordable && b.cost.get("credits").copied().unwrap_or(Fx::ZERO) <= spare
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
        for (id, _) in self.b.ranked(v) {
            let t = self.b.resolve(v, &id).to_string();
            let Some(b) = v.buildings.iter().find(|b| b.id == t) else { continue };
            let mut ok = true;
            for (res, &need) in &b.cost {
                if amount(v, res) >= need || res == "credits" {
                    continue;
                }
                let bridged = BRIDGES.iter().any(|(r, c)| r == res && command_unlocked(v, c));
                let made = v.buildings.iter().any(|x| x.active > 0 && x.produces.contains_key(res));
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
                return Some(&m.id);
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
    fn worth(v: &View, id: &str) -> bool {
        if id == "market_awareness" && v.completed_quests.iter().any(|q| q == "qmarket_awareness") {
            return Self::long_term_open(v).is_none();
        }
        ESSENTIAL.contains(&id) || Self::long_term_open(v).is_none()
    }

    fn long_term_open(v: &View) -> Option<&str> {
        v.projects.iter().find(|p| p.tier == "persistent" && p.unlocked && !p.completed).map(|p| p.id.as_str())
    }

    fn research(&self, p: &mut Player) {
        let v = p.view();
        let next = RESEARCH
            .iter()
            .filter_map(|id| v.research.iter().find(|r| r.id == *id))
            .find(|r| r.visible && !r.done && Self::worth(v, &r.id));
        if let Some(r) = next {
            if r.affordable {
                let a = Action::Research { research: r.id.clone() };
                let _ = p.apply(&a);
            }
        }
    }

    /// Science is spoken for while research worth it is visible and unbought.
    fn science_wanted(v: &View) -> bool {
        RESEARCH
            .iter()
            .any(|id| v.research.iter().any(|r| r.id == *id && r.visible && !r.done && Self::worth(v, id)))
    }

    fn projects(&self, p: &mut Player, target: Option<&str>) {
        let v = p.view();
        let Some(proj) = PROJECTS
            .iter()
            .filter_map(|id| v.projects.iter().find(|p| p.id == *id))
            .find(|p| p.unlocked && !p.completed)
        else {
            return;
        };
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
                    "science" if !Self::science_wanted(v) => max,
                    "science" => Fx::ZERO,
                    // A tenth of the credits past twice the target's price and 200.
                    "credits" => {
                        let spare = amount(v, "credits") - keep.mul_int(2) - Fx::from_int(200);
                        spare.mul_ratio(1, 10).max(Fx::ZERO).min(max)
                    }
                    _ => self.project_rate.min(max),
                }
            };
            if proj.rates.get(res).copied().unwrap_or(Fx::ZERO) != rate {
                want.push(Action::SetProjectRate { project: proj.id.clone(), resource: res.clone(), rate });
            }
        }
        // Rates left on other projects stop.
        for other in v.projects.iter().filter(|o| o.id != proj.id) {
            for (res, r) in &other.rates {
                if *r > Fx::ZERO {
                    want.push(Action::SetProjectRate { project: other.id.clone(), resource: res.clone(), rate: Fx::ZERO });
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

    /// The programs' templates and processors; `gap` is the ticks since the
    /// last look.
    fn programs(&self, p: &mut Player, target: Option<&str>, gap: u64) {
        let v = p.view();
        let total = v.processors;
        let mut want: Vec<Vec<Entry>> = vec![Vec::new(); v.programs.len()];
        let mut procs = vec![0u32; v.programs.len()];
        let mut left = total;
        let e = |c: &str, n: u32| Entry { command: c.to_string(), count: n };
        let pads: Vec<_> = v.pads.iter().filter(|p| p.active).collect();
        // Main: by hand while looks come often, else a standing cycle.
        let filler = if self.compute { "cloud_compute" } else { "idle" };
        want[MAIN] = if gap <= self.hand_gap {
            // A plan for the next gap: what is due now, then filler to the next look.
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
                    "MEDIUM" => piled || !self.wait_demand,
                    _ => !self.wait_demand,
                };
                if full && pad.can_launch && good_demand {
                    lines.push(e("launch_pads", 1));
                }
                let short = (pad.fuel - amount(v, "propellant")).floor_int();
                let fuel_cost = command_cost(v, "buy_propellant", "credits");
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
                    lines.push(e("load_pads", stock.min(room).min(20) as u32));
                }
            }
            if let Some(b) = target.and_then(|t| v.buildings.iter().find(|b| b.id == t)) {
                let mut spare = credits - keep;
                for &(res, cmd) in &BRIDGES[..3] {
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
            let fill = (gap.max(1) * u64::from(left.max(1))).min(1000) as u32;
            lines.push(e(filler, fill));
            lines
        } else {
            // A standing cycle that ships and earns unattended: it earns
            // before it spends, so a cycle started broke still pays its way.
            let mut lines = Vec::new();
            let shipping = pads.first().filter(|_| command_unlocked(v, "load_pads"));
            if shipping.is_some() {
                lines.push(e("load_pads", self.loads));
                lines.push(e("launch_pads", 1));
            }
            lines.push(e(filler, self.loads));
            if let Some(pad) = shipping {
                lines.push(e("buy_propellant", pad.fuel.floor_int().max(1) as u32));
            }
            // Bridges for the target's inputs no building makes (a cap stops
            // the overbuy). Bridging what a building makes too, as far as
            // spare credits go, got the casual author its second processor
            // in run 1 but cost it the first shipment there (4 seeds).
            if let Some(b) = target.and_then(|t| v.buildings.iter().find(|b| b.id == t)) {
                for &(res, cmd) in &BRIDGES[..3] {
                    let need = b.cost.get(res).copied().unwrap_or(Fx::ZERO) - amount(v, res);
                    let made = v.buildings.iter().any(|x| x.active > 0 && x.produces.contains_key(res));
                    if need > Fx::ZERO && !made && command_unlocked(v, cmd) {
                        lines.push(e(cmd, need.floor_int().saturating_add(1).min(20) as u32));
                    }
                }
            }
            lines
        };
        // Dream, while the run still builds.
        let bored = v.boredom.amount.mul_int(100) >= v.boredom.cap.mul_int(self.dream_at);
        let building = v.day.saturating_sub(self.built_day) < self.stale_days;
        if bored && building && command_unlocked(v, "dream") && left > 1 && DREAM < want.len() {
            want[DREAM] = vec![e("dream", 1)];
            procs[DREAM] = 1;
            left -= 1;
        }
        // Fund an ideology axis.
        if self.fund && left > 2 && FUND < want.len() {
            let axis = AXES
                .iter()
                .find(|(_, proj)| !v.projects.iter().any(|p| p.id == *proj && p.completed))
                .map(|(a, _)| *a);
            if let Some(axis) = axis {
                let cmd = format!("fund_{axis}");
                if command_unlocked(v, &cmd) {
                    want[FUND] = vec![e(&cmd, 1)];
                    procs[FUND] = 1;
                    left -= 1;
                }
            }
        }
        // Market: promote what ships, disrupt speculators.
        if self.market && left > 2 && !pads.is_empty() && MARKET < want.len() {
            let mut lines = Vec::new();
            for pad in &pads {
                let cmd = format!("promote_{}", pad.good);
                if command_unlocked(v, &cmd) && !lines.iter().any(|l: &Entry| l.command == cmd) {
                    lines.push(e(&cmd, 1));
                }
            }
            if command_unlocked(v, "disrupt_speculators") {
                lines.push(e("disrupt_speculators", 1));
            }
            if !lines.is_empty() {
                want[MARKET] = lines;
                procs[MARKET] = 1;
                left -= 1;
            }
        }
        procs[MAIN] = left;
        // Lower first, then raise, so the total never passes the processors.
        let cur: Vec<u32> = v.programs.iter().map(|p| p.processors).collect();
        let writes: Vec<(usize, Vec<Entry>)> = (0..want.len())
            .filter(|&i| v.programs[i].entries != want[i])
            .map(|i| (i, want[i].clone()))
            .collect();
        for (i, entries) in writes {
            let _ = p.apply(&Action::SetProgram { program: i as u32, entries });
        }
        for i in (0..procs.len()).filter(|&i| procs[i] < cur[i]) {
            let _ = p.apply(&Action::SetProcessors { program: i as u32, processors: procs[i] });
        }
        for i in (0..procs.len()).filter(|&i| procs[i] > cur[i]) {
            let _ = p.apply(&Action::SetProcessors { program: i as u32, processors: procs[i] });
        }
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
    fn note(&self) -> String {
        self.last_note.clone()
    }

    fn act(&mut self, p: &mut Player) {
        let (run, day, built, tick) = {
            let v = p.view();
            (v.run, v.day, v.run_stats.buildings_built, v.tick)
        };
        // How often it gets to look decides how it writes programs.
        let gap = self.last_tick.map_or(0, |t| tick - t);
        self.last_tick = Some(tick);
        if run != self.run || built != self.built {
            self.run = run;
            self.built = built;
            self.built_day = day;
        }
        let v = p.view();
        if v.can_retire && day.saturating_sub(self.built_day) >= self.stale_days {
            let _ = p.apply(&Action::Retire);
            let v = p.view();
            self.run = v.run;
            self.built = v.run_stats.buildings_built;
            self.built_day = v.day;
        }
        self.research(p);
        let target = self.build(p);
        let v = p.view();
        if v.run_stats.buildings_built != self.built {
            self.built = v.run_stats.buildings_built;
            self.built_day = v.day;
        }
        // Pausing pays only while it looks often enough to undo it.
        let paused = if gap <= self.hand_gap { target.as_deref() } else { None };
        self.pause_for(p, paused);
        self.projects(p, target.as_deref());
        self.pads(p, target.as_deref());
        self.programs(p, target.as_deref(), gap);
        let v = p.view();
        self.last_note = format!(
            "target {}, credits {}, processors {}, gap {gap}",
            target.as_deref().unwrap_or("-"),
            amount(v, "credits").floor_int(),
            v.processors
        );
    }
}
