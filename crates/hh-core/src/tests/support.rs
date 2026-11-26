//! Helpers the topical play tests share: owning buildings, setting a program, a buy action.

use crate::modules::economy;
use crate::state::{Entry, Holding};
use crate::{Action, GameState, Rules};

pub(crate) fn own(g: &mut GameState, b: &str, n: u32) {
    g.run.buildings.insert(b.into(), Holding { owned: n, active: n, bonus: 0, free: 0, unpaid: 0 });
    economy::refresh(g);
}

pub(crate) fn program(r: &Rules, g: &mut GameState, cmds: &[(&str, u32)]) {
    let entries = cmds.iter().map(|(c, n)| Entry::command(c, *n)).collect();
    r.apply(g, &Action::SetProgram { program: 0, entries }).unwrap();
}

pub(crate) fn buy(b: &str) -> Action {
    Action::Buy { building: b.into() }
}
