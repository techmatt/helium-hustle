//! The agent ladder (docs/agents.md): sim players that read only the engine's
//! view and act only through its actions, as the web client does.
//!
//! An agent never sees the state: it gets a [`Player`], which hands out the
//! view and applies actions. Each agent's randomness is its own stream, keyed
//! by the game seed and the agent's kind, so the same seed, config and agent
//! give the same game. Agents use integers and `Fx` only (no floats), so a
//! decision can't differ between native and wasm builds.

use std::collections::BTreeMap;

use hh_core::{Action, Fx, Session, View};

pub mod author;
pub mod builder;
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

    /// Applies an action; an illegal one changes nothing and returns its error.
    pub fn apply(&mut self, a: &Action) -> Result<(), String> {
        let r = self.s.apply(a);
        if r.is_ok() {
            self.view = None;
            self.applied += 1;
        }
        r
    }
}

/// A sim player.
pub trait Agent: Send {
    /// One look at the game: read the view, act. Called at each attention
    /// point (docs/simulator.md, *Profiles*).
    fn act(&mut self, p: &mut Player);

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

/// The rungs, weakest first. `Planner` and `Oracle` are hooks for later
/// items: a strategic player that plans across retirements (through
/// [`Agent::new_run`]), and a full-information search as the ceiling.
pub const RUNGS: &[(&str, &str)] = &[
    ("random", "uniform over the actions the view offers"),
    ("greedy", "scored build order, one hand-picked command at a time"),
    ("author", "the builder plus programs written from templates, research, projects, Dream"),
    ("planner", "not built: plans across retirements (hook: Agent::new_run)"),
    ("oracle", "not built: full-information search, the ceiling"),
];

/// Builds an agent from its spec (`kind[:key=value,...]`) for a game seed.
pub fn build(spec: &str, seed: u64) -> Result<Box<dyn Agent>, String> {
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
        "planner" | "oracle" => Err(format!("agent {kind:?} is a hook for a later rung, not built yet")),
        _ => Err(format!(
            "unknown agent {kind:?} (rungs: {})",
            RUNGS.iter().map(|r| r.0).collect::<Vec<_>>().join(", ")
        )),
    }
}

// Shared view readers.

pub fn amount(v: &View, id: &str) -> Fx {
    v.resources.iter().find(|r| r.id == id).map_or(Fx::ZERO, |r| r.amount)
}

pub fn cap(v: &View, id: &str) -> Option<Fx> {
    v.resources.iter().find(|r| r.id == id).and_then(|r| r.cap)
}

pub fn owned(v: &View, id: &str) -> u32 {
    v.buildings.iter().find(|b| b.id == id).map_or(0, |b| b.owned)
}

pub fn command_unlocked(v: &View, id: &str) -> bool {
    v.commands.iter().any(|c| c.id == id && c.unlocked)
}

pub fn command_cost(v: &View, id: &str, res: &str) -> Fx {
    v.commands
        .iter()
        .find(|c| c.id == id)
        .and_then(|c| c.costs.get(res).copied())
        .unwrap_or(Fx::ZERO)
}

pub fn researched(v: &View, id: &str) -> bool {
    v.research.iter().any(|r| r.id == id && r.done)
}
