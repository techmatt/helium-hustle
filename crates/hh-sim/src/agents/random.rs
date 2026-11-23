//! `random`: the floor. Each look it takes `actions` actions (knob, default 2),
//! each uniform over what the view offers: buy an affordable building, buy
//! affordable research, buy land, write a random program (1-4 lines of
//! unlocked commands, counts 1-3) to a random program, move free processors
//! to a random program, or start funding a random open project. It retires
//! voluntarily with chance 1 in `retire_odds` (default 500) per look, and
//! points a No-cargo pad at the first good in loading priority
//! ([`super::pick_cargo`]).

use hh_core::rng::Stream;
use hh_core::state::Entry;
use hh_core::{Action, Fx};

use super::{amount, Agent, Knobs, Player};

pub const KNOBS: &[&str] = &["actions", "retire_odds", "every"];

pub struct Random {
    rng: Stream,
    actions: u32,
    retire_odds: u32,
    every: u64,
}

impl Random {
    pub fn new(seed: u64, k: &Knobs) -> Self {
        Random {
            rng: Stream::new(seed, "agent:random"),
            actions: k.get("actions", 2).max(1) as u32,
            retire_odds: k.get("retire_odds", 500).max(1) as u32,
            every: k.get("every", 5).max(1) as u64,
        }
    }

    fn pick<T: Clone>(&mut self, xs: &[T]) -> Option<T> {
        if xs.is_empty() {
            None
        } else {
            Some(xs[self.rng.below(xs.len() as u32) as usize].clone())
        }
    }
}

impl Agent for Random {
    fn next_look(&mut self, _p: &mut Player) -> u64 {
        self.every
    }

    fn act(&mut self, p: &mut Player) {
        if p.view().can_retire && self.rng.below(self.retire_odds) == 0 {
            let _ = p.apply(&Action::Retire);
            return;
        }
        for _ in 0..self.actions {
            let v = p.view();
            let mut options: Vec<Action> = Vec::new();
            options.extend(v.buildings.iter().filter(|b| b.affordable).map(|b| b.buy.clone()));
            options.extend(
                v.research
                    .iter()
                    .filter(|r| r.affordable)
                    .map(|r| Action::Research { research: r.id.to_string() }),
            );
            if amount(v, "credits") >= v.land_price {
                options.push(Action::BuyLand);
            }
            let commands: Vec<String> = v.commands.iter().filter(|c| c.unlocked).map(|c| c.id.to_string()).collect();
            let programs = v.programs.len() as u32;
            let free = v.processors_free;
            let assigned: Vec<u32> = v.programs.iter().map(|p| p.processors).collect();
            let projects: Vec<(String, Vec<String>)> = v
                .projects
                .iter()
                .filter(|p| p.unlocked && !p.completed)
                .map(|p| (p.id.to_string(), p.costs.keys().cloned().collect()))
                .collect();
            // Write a program.
            let lines = 1 + self.rng.below(4);
            let mut entries = Vec::new();
            for _ in 0..lines {
                if let Some(c) = self.pick(&commands) {
                    entries.push(Entry::command(&c, 1 + self.rng.below(3)));
                }
            }
            options.push(Action::SetProgram { program: self.rng.below(programs), entries });
            if free > 0 {
                let prog = self.rng.below(programs);
                options.push(Action::SetProcessors { program: prog, processors: assigned[prog as usize] + free });
            }
            if let Some((id, costs)) = self.pick(&projects) {
                if let Some(res) = self.pick(&costs) {
                    let rate = Fx::from_int(1 + self.rng.below(10) as i64);
                    options.push(Action::SetProjectRate { project: id, resource: res, rate });
                }
            }
            if let Some(a) = self.pick(&options) {
                let _ = p.apply(&a);
            }
        }
        // Its one fixed hand besides retiring, after its buys: a new pad gets a cargo.
        super::pick_cargo(p);
    }
}
