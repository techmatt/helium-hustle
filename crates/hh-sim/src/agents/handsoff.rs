//! `handsoff`: how far the best known loadout carries a run with no hand on
//! it. Its programs are data: [`LOADOUT`] (`configs/sim/handsoff_loadout.txt`,
//! the form in [`super::loadout`]), updated when the rules or the best
//! loadout change.
//!
//! Its first runs, every run until `if` opens and that one (049), are the
//! author's (with its knobs): the loadout guards with `if`. From the next run
//! on, at each reset it puts the loadout's tabs in their programs through the
//! library, as a player keeping them would (written and saved as
//! `hands-off N` the first time, loaded after), and seats processors by the
//! loadout's `seat` line. Its only hand move after that is a stale-run
//! retirement: once the run has bought nothing (building, research or land)
//! for `stale_days` days, it retires (programs have no reading for it).
//! docs/agents.md says the same; docs/experiments.md has the measure.

use hh_core::state::Amount;
use hh_core::{Action, Fx};

use super::author::{self, Author};
use super::loadout::{self, Loadout};
use super::{Agent, Knobs, Player};

/// The loadout it plays (built in, so the native and wasm players match).
pub const LOADOUT: &str = include_str!("../../../../configs/sim/handsoff_loadout.txt");

/// The author's knobs (its first runs), how often it looks afterwards, and
/// the stale-run retirement's days (`stale_days`, shared with the author).
pub fn knobs() -> Vec<&'static str> {
    let mut k = author::KNOBS.to_vec();
    k.push("every");
    k
}

/// The library name a tab's lines are saved under.
fn slot_name(tab: usize) -> String {
    format!("hands-off {}", tab + 1)
}

pub struct HandsOff {
    author: Author,
    loadout: Loadout,
    first_run: Option<u32>,
    loaded_run: u32,
    stale_days: u64,
    every: u64,
    /// What it has bought this run (buildings owned and research done, and
    /// the land price, which rises with each land bought) and the day that
    /// last changed: the stale-run retirement's clock.
    bought: ((u32, Fx), u64),
}

impl HandsOff {
    pub fn new(k: &Knobs) -> Result<Self, String> {
        let loadout = loadout::parse(LOADOUT).map_err(|e| format!("configs/sim/handsoff_loadout.txt: {e}"))?;
        let mut author = Author::new(k);
        author.stop_at_retire = true;
        Ok(HandsOff {
            author,
            loadout,
            first_run: None,
            loaded_run: 0,
            stale_days: k.get("stale_days", 300).max(1) as u64,
            every: k.get("every", 10).max(1) as u64,
            bought: ((0, Fx::ZERO), 0),
        })
    }

    fn hands_off(&self, run: u32) -> bool {
        self.first_run.is_some_and(|r| run != r)
    }

    /// The reset: each of the loadout's tabs into its program, then the seats.
    fn load(&self, p: &mut Player) {
        for (tab, entries) in &self.loadout.programs {
            let _ = p.put_program(*tab as u32, entries.clone(), Some(&slot_name(*tab)));
            // A tab the game refused would sit empty every run and the read
            // mean nothing: never silently.
            let held = p.view().programs.get(*tab).is_some_and(|x| x.entries == *entries);
            assert!(held, "configs/sim/handsoff_loadout.txt: Program {} refused at run {}", tab + 1, p.view().run);
        }
        let v = p.view();
        let n = v.programs.len();
        let mut want = vec![0u32; n];
        let mut left = v.processors;
        for &(t, a) in self.loadout.seat.iter().filter(|(t, _)| *t < n) {
            let k = match a {
                Amount::All => left,
                Amount::N(k) => k.min(left),
            };
            want[t] += k;
            left -= k;
        }
        // Lowered first, then raised, so the free pool covers each raise.
        let cur: Vec<u32> = v.programs.iter().map(|p| p.processors).collect();
        for i in (0..n).filter(|&i| want[i] < cur[i]) {
            let _ = p.apply(&Action::SetProcessors { program: i as u32, processors: want[i] });
        }
        for i in (0..n).filter(|&i| want[i] > cur[i]) {
            let _ = p.apply(&Action::SetProcessors { program: i as u32, processors: want[i] });
        }
    }
}

impl Agent for HandsOff {
    fn act(&mut self, p: &mut Player) {
        let v = p.view();
        let run = v.run;
        // The loadout guards with `if`: the author plays every run until
        // `if` opens (a global project, Act 1 beat 1.7), and that run.
        if v.if_open {
            self.first_run.get_or_insert(run);
        }
        if !self.hands_off(run) {
            self.author.act(p);
            return;
        }
        if self.loaded_run == run {
            // Between resets: the stale-run retirement alone.
            let owned = v.buildings.iter().map(|b| b.owned).sum::<u32>() + v.research.iter().filter(|r| r.done).count() as u32;
            let have = (owned, v.land_price);
            if have != self.bought.0 {
                self.bought = (have, v.day);
            } else if v.day.saturating_sub(self.bought.1) >= self.stale_days {
                let _ = p.apply(&Action::Retire);
            }
            return;
        }
        self.bought = ((u32::MAX, Fx::ZERO), v.day);
        self.loaded_run = run;
        self.load(p);
    }

    fn next_look(&mut self, p: &mut Player) -> u64 {
        if self.hands_off(p.view().run) {
            self.every
        } else {
            self.author.next_look(p)
        }
    }

    fn note(&self) -> String {
        if self.loaded_run != 0 {
            let tabs: Vec<String> = self.loadout.programs.iter().map(|(t, e)| format!("{}: {} lines", t + 1, e.len())).collect();
            format!("hands off: loadout tabs {}", tabs.join(", "))
        } else {
            self.author.note()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped loadout reads, and every tab fits a program.
    #[test]
    fn the_loadout_reads() {
        let l = loadout::parse(LOADOUT).unwrap();
        assert!(!l.programs.is_empty());
        let cfg = hh_core::Config::from_json(include_str!("../../../../configs/baseline.json")).unwrap();
        let s = hh_core::Session::new(&cfg, 1, None).unwrap();
        let v = s.rules.view(&s.state);
        for (t, e) in &l.programs {
            assert!(*t < v.programs.len(), "tab {}", t + 1);
            assert!(e.len() <= v.max_program_lines, "tab {}: {} lines", t + 1, e.len());
        }
    }
}
