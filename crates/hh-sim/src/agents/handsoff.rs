//! `handsoff`: how far automation alone carries a run. Its first run is the
//! author's (with its knobs), which writes and saves the condition programs.
//! From run 2 on, at each new run it loads them from the library by name
//! (ship → Program 1, Dream watch → 2, fund → 3, market → 4, build order → 5)
//! and assigns processors, and makes no other decision: no purchase, research,
//! project, pad good or pause by hand. The two exceptions, both a player's
//! hands on the controls rather than a choice of play: a processor gained
//! mid-run (a Data Center the build order bought) is assigned at its next look
//! by the same rule, and a run that has bought nothing for `stale_days` days
//! retires (Dream could otherwise hold boredom off and stall the career).
//! docs/agents.md says the same; docs/simulator.md has the measure.

use hh_core::state::{Amount, Clause, Cmp, Condition, Entry, Join, Joined, Op, Reading, Verb};
use hh_core::{Action, Fx, View};

use super::author::{self, Author, BUILD, BUILD_ORDER, DREAM, DREAM_WATCH, FUND, FUND_NAME, MAIN, MARKET, MARKET_NAME, SHIP};
use super::{Agent, Knobs, Player};

/// The author's knobs (its first run), and how often it looks afterwards.
pub fn knobs() -> Vec<&'static str> {
    let mut k = author::KNOBS.to_vec();
    k.push("every");
    k
}

/// Program slots and the loadout each loads, in the order a spare processor
/// goes to them (the main program takes the rest).
const SLOTS: &[(usize, &str)] = &[(BUILD, BUILD_ORDER), (DREAM, DREAM_WATCH), (FUND, FUND_NAME), (MARKET, MARKET_NAME)];

/// Most lines in a program (mirrors configs/baseline.json `max_entries`).
const MAX_LINES: usize = 128;

/// The build order's lead: while no Launch Pad stands, the ship program has
/// nothing to ship, so the build order takes every processor (its buy of a
/// short input and the build it serves then fall on one day, before the
/// day's clamp cuts the input back to its cap); once one stands, it keeps one
/// and the ship program takes the rest. Done lines are free.
///
/// Then the stock lines: a maker whose product sits at its cap is paused (all
/// of it) so its input builds up for the next purchase, and resumed once the
/// product is under 90% of its cap; and while titanium sits at its cap, a
/// Storage Depot once credits reach three times its price.
fn processor_lines() -> Vec<Entry> {
    let pads = |cmp| Clause { reading: Reading::Count { building: "launch_pad".to_string() }, cmp, value: Fx::ONE, against: None };
    let pct = |res: &str, cmp, v: i64| Clause { reading: Reading::Percent { resource: res.to_string() }, cmp, value: Fx::from_int(v), against: None };
    let act = |verb, target: Option<&str>, tab: Option<usize>, amount| {
        Entry::new(Op::Act { verb, target: target.map(str::to_string), tab: tab.map(|t| t as u32), amount }).indent()
    };
    let when = |c, and: Option<Clause>| Entry::new(Op::If { when: Condition { first: c, rest: and.map(|clause| Joined { join: Join::And, clause }) } });
    let mut lines = vec![
        when(pads(Cmp::Lt), None),
        act(Verb::AssignProcessors, None, Some(MAIN), Some(Amount::N(0))),
        act(Verb::AssignProcessors, None, Some(BUILD), Some(Amount::All)),
        when(pads(Cmp::Ge), None),
        act(Verb::AssignProcessors, None, Some(BUILD), Some(Amount::N(1))),
        act(Verb::AssignProcessors, None, Some(MAIN), Some(Amount::All)),
    ];
    for (maker, product) in STOCK {
        lines.push(when(pct(product, Cmp::Ge, 100), None));
        lines.push(act(Verb::Pause, Some(maker), None, Some(Amount::All)));
        lines.push(when(pct(product, Cmp::Lt, 90), None));
        lines.push(act(Verb::Resume, Some(maker), None, Some(Amount::All)));
    }
    let depot = Clause {
        reading: Reading::Amount { resource: "credits".to_string() },
        cmp: Cmp::Ge,
        value: Fx::from_int(3),
        against: Some(Reading::NextCost { building: "storage_depot".to_string() }),
    };
    lines.push(when(pct("titanium", Cmp::Ge, 100), Some(depot)));
    lines.push(Entry::new(Op::Build { building: "storage_depot".to_string(), up_to: None }).indent());
    lines
}

/// The research the author's build order leaves to its looks (it weighs it
/// against a Long-Term project; this player funds none), to go after the
/// build order's own lines, so the essential research there comes first.
fn research_tail(v: &View) -> Vec<Entry> {
    let rest: Vec<&str> = author::RESEARCH.iter().copied().filter(|id| !author::ESSENTIAL.contains(id)).collect();
    author::research_lines(v, &rest)
}

/// Makers the stock lines pause while their product sits at its cap
/// (mirrors content/buildings.json: smelters make titanium, fabricators circuits).
const STOCK: &[(&str, &str)] = &[("smelter", "titanium"), ("fabricator", "circuits")];

pub struct HandsOff {
    author: Author,
    first_run: Option<u32>,
    loaded_run: u32,
    /// Slots it loaded this run.
    loaded: Vec<usize>,
    built: u32,
    built_day: u64,
    stale_days: u64,
    every: u64,
}

impl HandsOff {
    pub fn new(k: &Knobs) -> Self {
        let mut author = Author::new(k);
        author.stop_at_retire = true;
        HandsOff {
            author,
            first_run: None,
            loaded_run: 0,
            loaded: Vec::new(),
            built: 0,
            built_day: 0,
            stale_days: k.get("stale_days", 300).max(1) as u64,
            every: k.get("every", 10).max(1) as u64,
        }
    }

    fn hands_off(&self, v: &View) -> bool {
        self.first_run.is_some_and(|r| v.run != r)
    }

    /// Loads each saved program into its slot (the main one always gets the
    /// ship cycle's processors, loaded or not).
    fn load(&mut self, p: &mut Player) {
        self.loaded.clear();
        let mut wanted = vec![(MAIN, SHIP)];
        wanted.extend_from_slice(SLOTS);
        for (slot, name) in wanted {
            let v = p.view();
            if let Some(i) = v.library.iter().position(|l| l.name == name) {
                let a = Action::LoadLoadout { loadout: i as u32, program: slot as u32 };
                if p.apply(&a).is_ok() {
                    self.loaded.push(slot);
                }
            }
        }
    }

    /// One processor each to the loaded side programs, the rest to the main
    /// program (lowered first, then raised). The build order takes the first
    /// (with one, it is what grows the run: nothing ships yet); the others
    /// one each while two or more are left.
    fn assign(&self, p: &mut Player) {
        let v = p.view();
        let mut want = vec![0u32; v.programs.len()];
        let mut left = v.processors;
        for &(slot, _) in SLOTS {
            let room = if slot == BUILD { left > 0 } else { left > 1 };
            if self.loaded.contains(&slot) && room && slot < want.len() {
                want[slot] = 1;
                left -= 1;
            }
        }
        if let Some(m) = want.get_mut(MAIN) {
            *m = left;
        }
        let cur: Vec<u32> = v.programs.iter().map(|p| p.processors).collect();
        for i in (0..want.len()).filter(|&i| want[i] < cur[i]) {
            let _ = p.apply(&Action::SetProcessors { program: i as u32, processors: want[i] });
        }
        for i in (0..want.len()).filter(|&i| want[i] > cur[i]) {
            let _ = p.apply(&Action::SetProcessors { program: i as u32, processors: want[i] });
        }
    }
}

impl Agent for HandsOff {
    fn act(&mut self, p: &mut Player) {
        let v = p.view();
        let run = v.run;
        self.first_run.get_or_insert(run);
        if !self.hands_off(v) {
            self.author.act(p);
            return;
        }
        let (day, built) = (v.day, v.run_stats.buildings_built);
        if self.loaded_run != run {
            if self.loaded_run == 0 {
                // The author's last act, once: its build order from run 1,
                // saved to the library (the author writes it at a new run).
                self.author.track(v);
                let mut plan = self.author.build_order().to_vec();
                if !plan.is_empty() {
                    let (lead, tail) = (processor_lines(), research_tail(p.view()));
                    if plan.len() + lead.len() + tail.len() <= MAX_LINES {
                        // The tail goes before the build order's last line, its filler.
                        let at = plan.len() - 1;
                        plan.splice(at..at, tail);
                        plan.splice(0..0, lead);
                    }
                    p.put_program(BUILD as u32, plan, Some(BUILD_ORDER));
                }
            }
            self.loaded_run = run;
            self.built = built;
            self.built_day = day;
            self.load(p);
            self.assign(p);
        }
        if built != self.built {
            self.built = built;
            self.built_day = day;
        }
        let v = p.view();
        if v.can_retire && day.saturating_sub(self.built_day) >= self.stale_days {
            if p.apply(&Action::Retire).is_ok() {
                // The new run's reset: load at once.
                self.act(p);
            }
            return;
        }
        // Between resets, only a processor gained, or one an unpaid Data
        // Center stopped (the shortfall comes off the build order last in
        // line, whose own `assign processors` lines can't then run).
        let assigned: u32 = v.programs.iter().map(|x| x.processors).sum();
        if v.processors_free > 0 || v.processors_running < assigned {
            self.assign(p);
        }
    }

    fn next_look(&mut self, p: &mut Player) -> u64 {
        if self.hands_off(p.view()) {
            self.every
        } else {
            self.author.next_look(p)
        }
    }

    fn note(&self) -> String {
        if self.loaded_run != 0 {
            format!("hands off: loaded slots {:?}", self.loaded)
        } else {
            self.author.note()
        }
    }
}
