//! The agent ladder (docs/agents.md): sim players that read only the engine's
//! view and act only through its actions, as the web client does.
//!
//! An agent never sees the state: it gets a [`Player`], which hands out the
//! view and applies actions. Each agent's randomness is its own stream, keyed
//! by the game seed and the agent's kind, so the same seed, config and agent
//! give the same game. Agents use integers and `Fx` only (no floats), so a
//! decision can't differ between native and wasm builds.

use std::collections::BTreeMap;

use hh_core::state::Entry;
use hh_core::{Action, Fx, Session, View};

use crate::milestones::Milestones;

pub mod author;
pub mod builder;
pub mod oracle;
pub mod progress;
pub mod random;

/// The seat an agent plays from: the view, and actions. Nothing else.
pub struct Player<'a> {
    s: &'a mut Session,
    view: Option<View>,
    /// Actions applied (accepted) during this look.
    pub applied: u32,
}

impl<'a> Player<'a> {
    pub fn new(s: &'a mut Session) -> Self {
        Player { s, view: None, applied: 0 }
    }

    /// The current view (rebuilt after an action changed the game).
    pub fn view(&mut self) -> &View {
        if self.view.is_none() {
            self.view = Some(self.s.rules.view(&self.s.state));
        }
        self.view.as_ref().expect("just built")
    }

    /// The oracle's seat only (docs/agents.md): the whole state, and a copy of
    /// the game to try moves on (its future included: seeded draws, events).
    /// The other rungs read the view alone.
    pub fn state(&self) -> &hh_core::GameState {
        &self.s.state
    }

    /// The oracle's seat only, like [`Player::state`]: the session (its milestone tracker).
    pub fn session(&self) -> &Session {
        self.s
    }

    /// The oracle's seat only: a copy of the game to play ahead on.
    pub fn fork(&self) -> Session {
        self.s.fork()
    }

    /// Applies an action; an illegal one changes nothing and returns its error.
    pub fn apply(&mut self, a: &Action) -> Result<(), String> {
        let r = self.s.apply(a);
        if r.is_ok() {
            self.view = None;
            self.applied += 1;
        }
        r
    }

    /// Puts `entries` in a program through the loadout library, as a player
    /// keeping their programs across retirements would: a saved loadout with
    /// these lines is loaded; otherwise the lines are written, and, when
    /// `keep` names a role, saved to the slot of that name (overwritten when it
    /// differs, a new slot while there is room). Returns whether it loaded.
    pub fn put_program(&mut self, program: u32, entries: Vec<Entry>, keep: Option<&str>) -> bool {
        let v = self.view();
        if let Some(i) = v.library.iter().position(|l| l.entries == entries) {
            if self.apply(&Action::LoadLoadout { loadout: i as u32, program }).is_ok() {
                return true;
            }
        }
        let _ = self.apply(&Action::SetProgram { program, entries: entries.clone() });
        if let Some(name) = keep.filter(|_| !entries.is_empty()) {
            let v = self.view();
            if v.programs.get(program as usize).is_some_and(|p| p.entries == entries) {
                let slot = v.library.iter().position(|l| l.name == name).map(|i| i as u32);
                let room = slot.is_some() || v.library.len() < v.library_slots;
                if room {
                    let _ = self.apply(&Action::SaveLoadout { program, slot, name: name.to_string() });
                }
            }
        }
        false
    }
}

/// A sim player. It plays in game time only: at each look it acts, then says
/// how many ticks to skip before its next look ([`Agent::next_look`]), so it
/// looks whenever a decision is useful and skips stretches that need none
/// (docs/agents.md, *When it looks*).
pub trait Agent: Send {
    /// One look at the game: read the view, act.
    fn act(&mut self, p: &mut Player);

    /// Ticks until the next look (at least 1), asked once after each `act`.
    fn next_look(&mut self, p: &mut Player) -> u64;

    /// Hook for the strategic rung: called once for each new run, at the
    /// first look after a forced retirement (before `act`), or right after
    /// the look in which the agent retired. A cross-retirement planner sets
    /// its plan for the run here. The v0 agents keep nothing across runs.
    fn new_run(&mut self, _p: &mut Player) {}

    /// What it is working toward, for `hh-sim trace` (empty: nothing to say).
    fn note(&self) -> String {
        String::new()
    }
}

/// Knobs from a spec (`kind:key=value,...`), integers only.
#[derive(Debug, Clone, Default)]
pub struct Knobs(pub BTreeMap<String, i64>);

impl Knobs {
    /// The knob's value, or `default`. (A knob not in the agent's `KNOBS`
    /// list is refused when the agent is built.)
    pub fn get(&self, key: &str, default: i64) -> i64 {
        self.0.get(key).copied().unwrap_or(default)
    }

    fn check(&self, known: &[&str], kind: &str) -> Result<(), String> {
        match self.0.keys().find(|k| !known.contains(&k.as_str())) {
            Some(k) => Err(format!("agent {kind}: unknown knob {k:?} (known: {})", known.join(", "))),
            None => Ok(()),
        }
    }
}

/// The rungs, weakest first. `planner` is a hook for a later item: a player
/// that plans across retirements without peeking (through
/// [`Agent::new_run`]); `oracle` is the ceiling, a full-information planner.
pub const RUNGS: &[(&str, &str)] = &[
    ("random", "uniform over the actions the view offers"),
    ("greedy", "scored build order, one hand-picked command at a time"),
    ("author", "the builder plus programs written from templates, research, projects, Dream"),
    ("planner", "not built: plans across retirements without peeking (hook: Agent::new_run)"),
    ("oracle", "the author steered by rollouts on copies of the game (the future included): run strategies, then tactics"),
];

/// A player's name and its agent spec: `name=kind[:key=value,...]` names a
/// player (`casual=author:min_gap=120`); a bare spec is its own name.
pub fn named(player: &str) -> (&str, &str) {
    let head = player.split(':').next().unwrap_or(player);
    match head.split_once('=') {
        Some((name, _)) => (name, &player[name.len() + 1..]),
        None => (player, player),
    }
}

/// A player's agent kind: `author` for `casual=author:min_gap=120`.
pub fn kind(player: &str) -> &str {
    let spec = named(player).1;
    spec.split_once(':').map_or(spec, |x| x.0)
}

/// Builds an agent from its spec (`[name=]kind[:key=value,...]`) for a game
/// seed. `ms` are the milestones the career is measured against (the oracle
/// scores them in its rollouts).
pub fn build(player: &str, seed: u64, ms: &Milestones) -> Result<Box<dyn Agent>, String> {
    let spec = named(player).1;
    let (kind, rest) = spec.split_once(':').unwrap_or((spec, ""));
    let mut knobs = BTreeMap::new();
    for kv in rest.split(',').filter(|s| !s.is_empty()) {
        let (k, v) = kv.split_once('=').ok_or_else(|| format!("agent {spec:?}: expected key=value, got {kv:?}"))?;
        let v: i64 = v.parse().map_err(|_| format!("agent {spec:?}: {k}={v} is not an integer"))?;
        knobs.insert(k.to_string(), v);
    }
    let knobs = Knobs(knobs);
    match kind {
        "random" => {
            knobs.check(random::KNOBS, kind)?;
            Ok(Box::new(random::Random::new(seed, &knobs)))
        }
        "greedy" => {
            knobs.check(builder::KNOBS, kind)?;
            Ok(Box::new(builder::Greedy::new(&knobs)))
        }
        "author" => {
            knobs.check(author::KNOBS, kind)?;
            Ok(Box::new(author::Author::new(&knobs)))
        }
        "oracle" => {
            knobs.check(oracle::KNOBS, kind)?;
            Ok(Box::new(oracle::Oracle::new(&knobs, ms)?))
        }
        "planner" => Err(format!("agent {kind:?} is a hook for a later rung, not built yet")),
        _ => Err(format!(
            "unknown agent {kind:?} (rungs: {})",
            RUNGS.iter().map(|r| r.0).collect::<Vec<_>>().join(", ")
        )),
    }
}

// Shared view readers (`amount` and `command_cost` wrap `View`'s own, shared with
// `driver.rs`).

pub fn amount(v: &View, id: &str) -> Fx {
    v.amount(id)
}

/// A building's next credits price (0 if it costs none).
pub fn credits_price(b: &hh_core::view::BuildingView) -> Fx {
    b.cost.get("credits").copied().unwrap_or(Fx::ZERO)
}

/// Whether some active building makes `res`.
pub fn made(v: &View, res: &str) -> bool {
    v.buildings.iter().any(|x| x.active > 0 && x.produces.contains_key(res))
}

pub fn cap(v: &View, id: &str) -> Option<Fx> {
    v.resources.iter().find(|r| r.id == id).and_then(|r| r.cap)
}

pub fn command_unlocked(v: &View, id: &str) -> bool {
    v.commands.iter().any(|c| c.id == id && c.unlocked)
}

pub fn command_cost(v: &View, id: &str, res: &str) -> Fx {
    v.command_cost(id, res)
}

/// Whole ticks for `have` to reach `need` growing `rate` a day: 0 if it
/// already has, `None` if it doesn't grow.
pub fn ticks_to(have: Fx, need: Fx, rate: Fx) -> Option<u64> {
    if have >= need {
        return Some(0);
    }
    if rate <= Fx::ZERO {
        return None;
    }
    // Whole ticks, rounded up (at least one: it isn't there yet).
    let q = (need - have) / rate;
    let whole = q.floor_int().max(0);
    let up = if Fx::from_int(whole) < q { whole + 1 } else { whole };
    Some(up.max(1) as u64)
}

/// Ticks until every cost of `cost` is on hand: credits at `credit_rate`
/// (the player's own reading of its income), anything else at its view rate.
/// `None` when some cost doesn't grow.
pub fn ticks_to_afford(v: &View, cost: &BTreeMap<String, Fx>, credit_rate: Fx) -> Option<u64> {
    let mut most = 0u64;
    for (res, &need) in cost {
        let rate = if res == "credits" {
            credit_rate
        } else {
            v.resources.iter().find(|r| r.id == *res).map_or(Fx::ZERO, |r| r.change_per_day)
        };
        most = most.max(ticks_to(amount(v, res), need, rate)?);
    }
    Some(most)
}

/// A player's reading of its credit income: credits earned this run between
/// two looks, a day.
#[derive(Debug, Clone, Default)]
pub struct Income {
    last: Option<(u32, u64, Fx)>,
    pub rate: Fx,
}

impl Income {
    pub fn read(&mut self, v: &View) {
        let now = (v.run, v.tick, v.run_stats.credits_earned);
        if let Some((run, tick, earned)) = self.last {
            if run == now.0 && now.1 > tick {
                self.rate = (now.2 - earned) / Fx::from_int((now.1 - tick) as i64);
            } else if run != now.0 {
                self.rate = Fx::ZERO;
            }
        }
        self.last = Some(now);
    }
}
