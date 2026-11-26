//! The builder: a scored build order (roughly the Godot optimizer's greedy
//! over purchase orderings), shared by `greedy` and `author`, and the `greedy`
//! agent itself.
//!
//! **Scoring.** Each unlocked building scores its base weight ([`WEIGHTS`])
//! times its keep percent per copy owned (`decay` sets one for all), with
//! needs read from the view:
//! - a building whose every product sits at its cap scores 0 (no more of a
//!   capped pile);
//! - an input a building is starved of raises its maker by `starved`%;
//! - an energy user that would leave less than `reserve` spare energy a day
//!   gives way to a Solar Panel.
//!
//! A target priced past a cap gives way to a Storage Depot (a Battery for
//! energy); one short of land buys land first when credits allow.
//!
//! **`greedy`** buys the best-scored affordable building, up to `buys` a look,
//! never saving for a dearer one, buys the cheapest affordable research, and
//! runs one command on the first program with every processor, chosen afresh
//! each look ([`hand_command`]). It funds no project and never retires.

use std::collections::BTreeSet;

use hh_core::state::Entry;
use hh_core::{Action, Fx, View};

use super::{amount, command_cost, command_unlocked, credits_price, Agent, Knobs, Player};

pub const KNOBS: &[&str] = &["decay", "starved", "reserve", "buys", "min_gap", "max_gap"];

/// Base weights (points) and the percent kept per copy owned, by building id.
/// A building not listed scores 0. Strategy, hand-tuned: they mirror no game value.
pub const WEIGHTS: &[(&str, i64, i64)] = &[
    ("data_center", 90, 85),
    ("refinery", 70, 70),
    ("launch_pad", 65, 40),
    ("excavator", 60, 70),
    ("smelter", 50, 60),
    ("research_lab", 60, 80),
    ("solar_panel", 40, 75),
    ("electrolysis", 35, 60),
    ("fabricator", 35, 50),
    ("ice_extractor", 30, 60),
    ("recreation_dome", 30, 75),
    ("fuel_cell", 25, 75),
    ("arbitrage_engine", 20, 50),
    ("microwave_receiver", 20, 50),
    ("storage_depot", 0, 70),
    ("battery", 0, 50),
];

/// The building that makes a starved input (mirrors content/buildings.json
/// `produces`).
const MAKERS: &[(&str, &str)] = &[
    ("energy", "solar_panel"),
    ("regolith", "excavator"),
    ("ice", "ice_extractor"),
    ("circuits", "fabricator"),
    ("propellant", "electrolysis"),
];

/// Commands that bridge a missing resource, by resource: [`INPUT_BRIDGES`], then
/// propellant (pads' fuel). Mirrors content/commands.json's `buy_*` commands.
pub const BRIDGES: &[(&str, &str)] = &[
    ("titanium", "buy_titanium"),
    ("regolith", "buy_regolith"),
    ("ice", "buy_ice"),
    ("propellant", "buy_propellant"),
];

/// The bridges for a building's missing inputs (not propellant: pads burn it).
pub const INPUT_BRIDGES: &[(&str, &str)] = &[("titanium", "buy_titanium"), ("regolith", "buy_regolith"), ("ice", "buy_ice")];

#[derive(Debug, Clone)]
pub struct Builder {
    /// Percent of the weight kept per copy owned, for every building
    /// (the knob); else each building's own ([`WEIGHTS`]).
    pub decay: Option<i64>,
    /// Percent added to a starved input's maker.
    pub starved: i64,
    /// Spare energy a day an energy user must leave.
    pub reserve: Fx,
}

impl Builder {
    pub fn new(k: &Knobs) -> Self {
        Builder {
            decay: k.0.get("decay").map(|d| (*d).clamp(1, 100)),
            starved: k.get("starved", 100).max(0),
            reserve: Fx::from_int(k.get("reserve", 3).max(0)),
        }
    }

    /// Unlocked buildings by score, best first (ties in content order);
    /// zero scores dropped.
    pub fn ranked(&self, v: &View) -> Vec<(String, i64)> {
        let mut starved: BTreeSet<&str> = BTreeSet::new();
        for b in &v.buildings {
            if b.active > 0 && b.stall == Some(hh_core::state::Stall::InputStarved) {
                for (res, need) in b.upkeep {
                    let short = amount(v, res) < need.mul_int(b.active as i64);
                    if short {
                        starved.insert(res.as_str());
                    }
                }
            }
        }
        let glut = v.markets.iter().any(|m| super::cap(v, &m.good).is_some_and(|c| amount(v, &m.good) >= c));
        let mut out: Vec<(String, i64)> = Vec::new();
        for b in v.buildings.iter().filter(|b| b.unlocked) {
            let (base, keep) = WEIGHTS.iter().find(|w| w.0 == b.id).map_or((0, 100), |w| (w.1, w.2));
            let keep = self.decay.unwrap_or(keep);
            let mut s = base * 1000;
            for _ in 0..b.owned.min(64) {
                s = s * keep / 100;
            }
            let capped = !b.produces.is_empty()
                && b.produces.keys().all(|res| super::cap(v, res).is_some_and(|c| amount(v, res) >= c));
            if capped {
                s = 0;
            }
            if MAKERS.iter().any(|(res, maker)| *maker == b.id && starved.contains(res)) {
                s += s * self.starved / 100;
            }
            // A good piling up at its cap with no pad to ship it.
            if b.id == "launch_pad" && b.owned == 0 && glut {
                s += s * self.starved / 100;
            }
            if s > 0 {
                out.push((b.id.to_string(), s));
            }
        }
        out.sort_by_key(|x| std::cmp::Reverse(x.1));
        out
    }

    /// Energy active buildings make a day minus what they use (content rates).
    pub fn energy_spare(v: &View) -> Fx {
        let mut net = Fx::ZERO;
        for b in &v.buildings {
            let n = b.active as i64;
            net += b.produces.get("energy").copied().unwrap_or(Fx::ZERO).mul_int(n);
            net -= b.upkeep.get("energy").copied().unwrap_or(Fx::ZERO).mul_int(n);
        }
        net
    }

    /// What to build for `id` now: itself, or what it needs first (a Solar
    /// Panel for energy, storage for a price past a cap), followed a few steps.
    pub fn resolve<'v>(&self, v: &'v View, id: &'v str) -> &'v str {
        let mut at = id;
        for _ in 0..4 {
            let next = self.resolve_once(v, at);
            if next == at {
                break;
            }
            at = next;
        }
        at
    }

    fn resolve_once<'v>(&self, v: &'v View, id: &'v str) -> &'v str {
        let Some(b) = v.buildings.iter().find(|b| b.id == id) else { return id };
        let uses = b.upkeep.get("energy").copied().unwrap_or(Fx::ZERO);
        if uses > Fx::ZERO && Self::energy_spare(v) - uses < self.reserve && unlocked(v, "solar_panel") {
            return "solar_panel";
        }
        for (res, &price) in &b.cost {
            if super::cap(v, res).is_some_and(|c| price > c) {
                let store = if res == "energy" { "battery" } else { "storage_depot" };
                if unlocked(v, store) && store != id {
                    return store;
                }
            }
        }
        id
    }

    /// The building to work toward: the best-scored one, resolved.
    pub fn target(&self, v: &View) -> Option<String> {
        let best = self.ranked(v).into_iter().next()?;
        Some(self.resolve(v, &best.0).to_string())
    }

    /// Buys `id` if it can (land first when short and credits allow).
    /// Returns whether a building was bought.
    pub fn try_buy(p: &mut Player, id: &str) -> bool {
        let v = p.view();
        let Some(b) = v.buildings.iter().find(|b| b.id == id) else { return false };
        if amount(v, "land") < Fx::from_int(b.land as i64) {
            let price_credits = credits_price(b);
            if amount(v, "credits") >= v.land_price + price_credits {
                let _ = p.apply(&Action::BuyLand);
            }
        }
        let v = p.view();
        let Some(b) = v.buildings.iter().find(|b| b.id == id) else { return false };
        if b.affordable {
            let a = b.buy.clone();
            return p.apply(&a).is_ok();
        }
        false
    }
}

fn unlocked(v: &View, id: &str) -> bool {
    v.buildings.iter().any(|b| b.id == id && b.unlocked)
}

/// The one command a hand-player runs, chosen from the view: launch a full
/// fueled pad; fuel a full pad; load when goods wait; buy science for the
/// next Blueprints on sale; bridge the target's missing titanium, regolith or
/// ice; else Cloud Compute (Idle without energy).
pub fn hand_command(v: &View, target: Option<&str>) -> &'static str {
    let credits = amount(v, "credits");
    if let Some(pad) = v.pads.iter().find(|p| p.active) {
        let full = pad.cargo >= pad.capacity;
        if full && pad.can_launch && command_unlocked(v, "launch_pads") {
            return "launch_pads";
        }
        let fuel = command_cost(v, "buy_propellant", "credits");
        if full && amount(v, "propellant") < pad.fuel && credits >= fuel {
            return "buy_propellant";
        }
        if !full && pad.good.as_deref().is_some_and(|g| amount(v, g) >= pad.load) && command_unlocked(v, "load_pads") {
            return "load_pads";
        }
    }
    if super::science_short(v).is_some()
        && credits >= command_cost(v, "buy_science", "credits")
        && amount(v, "energy") >= command_cost(v, "buy_science", "energy")
    {
        return "buy_science";
    }
    if let Some(id) = target {
        if let Some(b) = v.buildings.iter().find(|b| b.id == id) {
            let price_credits = credits_price(b);
            for &(res, cmd) in INPUT_BRIDGES {
                let need = b.cost.get(res).copied().unwrap_or(Fx::ZERO);
                if amount(v, res) < need && credits - price_credits >= command_cost(v, cmd, "credits") {
                    return cmd;
                }
            }
        }
    }
    if amount(v, "energy") >= command_cost(v, "cloud_compute", "energy") {
        "cloud_compute"
    } else {
        "idle"
    }
}

/// Puts every processor on the first program.
pub fn all_on_first(p: &mut Player) {
    let v = p.view();
    if v.processors_free > 0 {
        let n = v.programs[0].processors + v.processors_free;
        let _ = p.apply(&Action::SetProcessors { program: 0, processors: n });
    }
}

pub struct Greedy {
    b: Builder,
    buys: u32,
    min_gap: u64,
    max_gap: u64,
    income: super::Income,
}

/// Affordable now, or short only of land that credits can buy along with it
/// (the view's `affordable` includes the land check).
fn buyable_with_land(v: &View, b: &hh_core::view::BuildingView) -> bool {
    if b.affordable {
        return true;
    }
    let short_land = amount(v, "land") < Fx::from_int(b.land as i64);
    let credits = credits_price(b);
    short_land
        && b.unlocked
        && amount(v, "credits") >= v.land_price + credits
        && b.cost.iter().all(|(res, need)| amount(v, res) >= *need)
}

impl Greedy {
    pub fn new(k: &Knobs) -> Self {
        let min_gap = k.get("min_gap", 1).max(1) as u64;
        Greedy {
            b: Builder::new(k),
            buys: k.get("buys", 3).max(1) as u32,
            min_gap,
            max_gap: (k.get("max_gap", 10).max(1) as u64).max(min_gap),
            income: super::Income::default(),
        }
    }
}

impl Agent for Greedy {
    /// Its hand command is chosen afresh each look, so it looks at least
    /// every `max_gap` ticks; sooner when its target comes within reach.
    fn next_look(&mut self, p: &mut Player) -> u64 {
        let v = p.view();
        self.income.read(v);
        let soon = self
            .b
            .target(v)
            .and_then(|t| v.buildings.iter().find(|b| b.id == t))
            .and_then(|b| super::ticks_to_afford(v, &b.cost, self.income.rate))
            .filter(|&t| t > 0)
            .unwrap_or(u64::MAX);
        soon.clamp(self.min_gap, self.max_gap)
    }

    fn act(&mut self, p: &mut Player) {
        all_on_first(p);
        for _ in 0..self.buys {
            let v = p.view();
            let pick = self
                .b
                .ranked(v)
                .into_iter()
                .map(|(id, _)| self.b.resolve(v, &id).to_string())
                .find(|id| v.buildings.iter().any(|b| b.id == *id && buyable_with_land(v, b)));
            match pick {
                Some(id) if Builder::try_buy(p, &id) => {}
                _ => break,
            }
        }
        // After the buys, so a pad bought this look gets its cargo now.
        super::pick_cargo(p);
        super::hand_launch(p);
        // The cheapest affordable research.
        let v = p.view();
        let cheapest = v.research.iter().filter(|r| r.affordable && super::buys_research(r.id)).min_by_key(|r| r.cost).map(|r| r.id);
        if let Some(id) = cheapest {
            let _ = p.apply(&Action::Research { research: id.to_string() });
        }
        let v = p.view();
        let target = self.b.target(v);
        let cmd = hand_command(v, target.as_deref());
        let want = vec![Entry::command(cmd, 1)];
        if v.programs[0].entries != want {
            let name = format!("builder: {cmd}");
            p.put_program(0, want, Some(&name));
        }
    }
}
