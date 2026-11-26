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
//! loadout's `seat` line. A tab whose `if`s read a researched reading
//! (`energy per day`, `days since a purchase`) can't be written before that
//! research: the first time, it writes the tab without those blocks and
//! rewrites it whole once the research is done, as a player would (once a
//! career: the library holds it after). Its only other hand move is a
//! stale-run retirement: once the run has bought nothing (building, research
//! or land) for `stale_days` days, it retires (a backstop: the loadout's own
//! `retire` line, on `days since a purchase`, comes first once researched).
//! docs/agents.md says the same; docs/experiments.md has the measure.

use hh_core::state::{Amount, Entry, Op};
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

/// The lines without each `if` block reading a researched reading.
fn without_researched(entries: &[Entry]) -> Vec<Entry> {
    let mut out = Vec::new();
    let mut skip = false;
    for e in entries {
        if let Op::If { when } = &e.op {
            skip = when.first.reading.researched() || when.rest.as_ref().is_some_and(|j| j.clause.reading.researched());
        } else if !e.indented && !matches!(e.op, Op::Else {}) {
            skip = false;
        }
        if !skip {
            out.push(e.clone());
        }
    }
    out
}

pub struct HandsOff {
    author: Author,
    loadout: Loadout,
    /// The loadout's file, for a refused tab's message.
    path: String,
    first_run: Option<u32>,
    loaded_run: u32,
    stale_days: u64,
    every: u64,
    /// Tabs written without their researched blocks, to rewrite whole once
    /// the research is done.
    pending: Vec<usize>,
    /// What it has bought this run (buildings owned and research done, and
    /// the land price, which rises with each land bought) and the day that
    /// last changed: the stale-run retirement's clock.
    bought: ((u32, Fx), u64),
}

impl HandsOff {
    /// `file` is another loadout to play, as `(path, text)` (the native-only
    /// `loadout=<path>` knob, [`super::build`]); `None` plays [`LOADOUT`].
    pub fn new(k: &Knobs, file: Option<(&str, &str)>) -> Result<Self, String> {
        let (path, text) = file.unwrap_or(("configs/sim/handsoff_loadout.txt", LOADOUT));
        let loadout = loadout::parse(text).map_err(|e| format!("{path}: {e}"))?;
        let mut author = Author::new(k);
        author.stop_at_retire = true;
        Ok(HandsOff {
            author,
            loadout,
            path: path.to_string(),
            first_run: None,
            loaded_run: 0,
            stale_days: k.get("stale_days", 300).max(1) as u64,
            every: k.get("every", 10).max(1) as u64,
            bought: ((0, Fx::ZERO), 0),
            pending: Vec::new(),
        })
    }

    fn hands_off(&self, run: u32) -> bool {
        self.first_run.is_some_and(|r| run != r)
    }

    /// The reset: each of the loadout's tabs into its program, then the seats.
    fn load(&mut self, p: &mut Player) {
        self.pending.clear();
        for (tab, entries) in &self.loadout.programs {
            let _ = p.put_program(*tab as u32, entries.clone(), Some(&slot_name(*tab)));
            let held = |p: &mut Player, want: &[Entry]| p.view().programs.get(*tab).is_some_and(|x| x.entries == want);
            if held(p, entries) {
                continue;
            }
            // Not researched yet: without those blocks for now.
            let part = without_researched(entries);
            let _ = p.put_program(*tab as u32, part.clone(), None);
            // A tab the game refused would sit empty every run and the read
            // mean nothing: never silently.
            assert!(held(p, &part), "{}: Program {} refused at run {}", self.path, tab + 1, p.view().run);
            self.pending.push(*tab);
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
        // `if` opens (a global project offered from the first retirement), and that run.
        // A timeline's end closes `if` with Conditional Logic's completion: the
        // author plays again (and re-funds it) until it reopens, and that run.
        if v.if_open {
            self.first_run.get_or_insert(run);
        } else {
            // Run numbers start over in the new timeline: forget the last load too.
            self.first_run = None;
            self.loaded_run = 0;
            self.pending.clear();
        }
        if !self.hands_off(run) {
            self.author.act(p);
            return;
        }
        if self.loaded_run == run {
            // Between resets: a pending tab rewritten once its research is done.
            if !self.pending.is_empty() && v.readings.iter().any(|r| r.reading.researched()) {
                for tab in std::mem::take(&mut self.pending) {
                    let entries = self.loadout.programs.iter().find(|(t, _)| *t == tab).map(|(_, e)| e.clone()).unwrap_or_default();
                    let _ = p.put_program(tab as u32, entries.clone(), Some(&slot_name(tab)));
                    let held = p.view().programs.get(tab).is_some_and(|x| x.entries == entries);
                    assert!(held, "{}: Program {} refused once researched", self.path, tab + 1);
                }
                return;
            }
            // The stale-run retirement.
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

    /// `loadout=<path>` plays a file; a missing or bad one is a build error.
    #[test]
    fn the_loadout_knob_reads_a_file() {
        let ms = crate::milestones::Milestones::from_json(include_str!("../../../../configs/sim/milestones.json")).unwrap();
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
        let good = format!("handsoff:loadout={root}/experiments/results/060_loadout_exploration/loadouts/L3_one_tab.txt,every=5");
        assert!(super::super::build(&good, 1, &ms).is_ok());
        let missing = format!("handsoff:loadout={root}/no_such_loadout.txt");
        assert!(super::super::build(&missing, 1, &ms).err().unwrap().contains("no_such_loadout.txt"));
        assert!(super::super::build("author:loadout=x.txt", 1, &ms).is_err());
    }
}
