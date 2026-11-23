//! `handsoff`: how far automation alone carries a run. Its first run (every
//! run until `if` opens, then that one: 049) is the author's (with its
//! knobs), which writes and saves the condition programs.
//! From run 2 on, at each new run it loads them from the library by name
//! (ship → Program 1, Dream watch → 2, fund → 3, market → 4, build order → 5)
//! and assigns processors, and makes no other decision: no purchase, research,
//! project, pad good, pause, processor or retirement by hand. What were its
//! last hands are lines: its build order seats a processor gained mid-run by
//! the reset's rule ([`seat_lines`]); its first side program (the Dream watch)
//! funds projects ([`fund_lines`]). One hand stays: it retires by hand once the run has
//! bought nothing for `stale_days` days (Dream could otherwise hold boredom
//! off and stall the career; programs have no reading for it since 059).
//! A
//! processor an unpaid Data Center stopped is no longer reseated (it was by
//! hand): the shortfall comes off the build order, last in line.
//! docs/agents.md says the same; docs/experiments.md has the measure.

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


/// The build order's lead, the stock lines: a maker whose product sits at
/// its cap is paused (all of it) so its input builds up for the next
/// purchase, and resumed once the product is under 90% of its cap; and while
/// titanium sits at its cap, a Storage Depot once credits reach three times
/// its price now (`depot`, a number: the right side is never a reading).
/// With no reading for running counts, `pause` and `resume` are done steps
/// while the product holds. (Before 059 the lead also moved processors
/// between the build order and the ship program on whether a pad stood:
/// no reading says so now; [`seat_lines`] seats them instead.)
fn processor_lines(depot: Fx) -> Vec<Entry> {
    let pct = |res: &str, cmp, v: i64| clause(Reading::Percent { resource: res.to_string() }, cmp, v);
    let act = |verb, target: Option<&str>, amount| Entry::new(Op::Act { verb, target: target.map(str::to_string), tab: None, amount }).indent();
    let when = |c, and: Option<Clause>| Entry::new(Op::If { when: Condition { first: c, rest: and.map(|clause| Joined { join: Join::And, clause }) } });
    let mut lines = Vec::new();
    for (maker, product) in STOCK {
        lines.push(when(pct(product, Cmp::Gt, 99), None));
        lines.push(act(Verb::Pause, Some(maker), Some(Amount::All)));
        lines.push(when(pct(product, Cmp::Lt, 90), None));
        lines.push(act(Verb::Resume, Some(maker), Some(Amount::All)));
    }
    let rich = Clause { reading: Reading::Amount { resource: "credits".to_string() }, cmp: Cmp::Gt, value: depot * Fx::from_int(3) - Fx::EPSILON };
    lines.push(when(pct("titanium", Cmp::Gt, 99), Some(rich)));
    lines.push(Entry::new(Op::Build { building: "storage_depot".to_string(), up_to: None }).indent());
    lines
}

fn clause(reading: Reading, cmp: Cmp, v: i64) -> Clause {
    Clause { reading, cmp, value: Fx::from_int(v) }
}

fn assign(tab: usize, amount: Amount) -> Entry {
    Entry::new(Op::Act { verb: Verb::AssignProcessors, target: None, tab: Some(tab as u32), amount: Some(amount) }).indent()
}

/// The build order's seat lines, what was the reset's processor rule as
/// lines for a processor gained mid-run (it starts free): while one is free,
/// the build order keeps one, the main program takes one, then each side
/// program in `sides` one, the rest to the main program. An `assign` short
/// of processors fails (a step), so with few the earlier ones win.
fn seat_lines(sides: &[usize]) -> Vec<Entry> {
    let mut lines = vec![
        Entry::new(Op::If { when: Condition { first: clause(Reading::FreeProcessors, Cmp::Gt, 0), rest: None } }),
        assign(BUILD, Amount::N(1)),
        assign(MAIN, Amount::N(0)),
        assign(MAIN, Amount::N(1)),
    ];
    lines.extend(sides.iter().map(|&s| assign(s, Amount::N(1))));
    lines.push(assign(MAIN, Amount::All));
    lines
}

/// Projects it funds, all at once (drained in file order): the author's
/// order without the unreachable Lunar Cartography and Deep Core Survey, and
/// without Chemical Energy and Grid Recalibration, which drain the propellant
/// shipping burns and energy (mirrors content/projects.json).
const PROJECTS: &[&str] = &[
    "foundation_grant",
    "ai_consciousness",
    "research_archive",
    "microwave_power",
    "predictive_maintenance",
    "speculator_dossier",
    "market_cornering",
];
/// The daily rate per cost resource (the author's `project_rate`) and the
/// credits it waits for (strategy).
const FUND_RATE: u32 = 3;
const FUND_AT: i64 = 1000;

/// From the day each project opens: while credits are above `FUND_AT`, it
/// is funded at `FUND_RATE` a day per cost. A rate set stays (the drain takes
/// only what is in stock), so it needs no stop; a project not open (or
/// completed) fails, one already at the rate is done, each a step.
fn fund_lines() -> Vec<Entry> {
    let fund = |id: &str| Entry::new(Op::Act { verb: Verb::Fund, target: Some(id.to_string()), tab: None, amount: Some(Amount::N(FUND_RATE)) }).indent();
    let mut lines = vec![Entry::new(Op::If {
        when: Condition { first: clause(Reading::Amount { resource: "credits".to_string() }, Cmp::Gt, FUND_AT), rest: None },
    })];
    lines.extend(PROJECTS.iter().map(|id| fund(id)));
    lines
}

/// The research the author's build order leaves to its looks (this player
/// has no looks to buy it on), to go after the build order's own lines, so
/// the plan's research ([`author::PLAN_RESEARCH`]) there comes first.
fn research_tail(v: &View) -> Vec<Entry> {
    let rest: Vec<&str> = author::RESEARCH.iter().copied().filter(|id| !author::PLAN_RESEARCH.contains(id)).collect();
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
    stale_days: u64,
    every: u64,
    /// Lines written: the build order's plan and lead, the side program's own
    /// and what it gained (hands, funding, research).
    lines: [usize; 4],
    /// What it has bought this run (buildings owned and research done, and
    /// the land price, which rises with each land bought) and the day that
    /// last changed: the stale-run retirement's clock.
    bought: ((u32, Fx), u64),
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
            stale_days: k.get("stale_days", 300).max(1) as u64,
            every: k.get("every", 10).max(1) as u64,
            lines: [0; 4],
            bought: ((0, Fx::ZERO), 0),
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
        // Its saved programs guard with `if`: the author plays every run
        // until `if` opens (a global project, Act 1 beat 1.7), and that run
        // counts as its first.
        if v.if_open {
            self.first_run.get_or_insert(run);
        }
        if !self.hands_off(v) {
            self.author.act(p);
            return;
        }
        // Between resets it does nothing but the stale-run retirement: its
        // programs hold the rest.
        if self.loaded_run == run {
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
        if self.loaded_run == 0 {
            // The author's last act, once: its build order from run 1,
            // saved to the library (the author writes it at a new run).
            self.author.track(v);
            let mut plan = self.author.build_order().to_vec();
            if !plan.is_empty() {
                let v = p.view();
                let sides: Vec<usize> =
                    SLOTS.iter().filter(|(s, name)| *s != BUILD && v.library.iter().any(|l| l.name == *name)).map(|(s, _)| *s).collect();
                let depot = v.buildings.iter().find(|b| b.id == "storage_depot").map_or(Fx::ZERO, super::credits_price);
                let mut lead = processor_lines(depot);
                lead.extend(seat_lines(&sides));
                // The rest goes to the first side program (the Dream watch
                // when saved): the build order has room for its lead only.
                // (A free processor is seated by the build order's lead alone:
                // a second taker in a shorter loop would always win it.)
                let mut extra = fund_lines();
                extra.extend(research_tail(v));
                let side = sides.first().and_then(|&s| {
                    let name = SLOTS.iter().find(|(x, _)| *x == s)?.1;
                    let held = &v.library.iter().find(|l| l.name == name)?.entries;
                    Some((s, name, held.clone()))
                });
                let side = side.filter(|(_, _, h)| !h.is_empty() && h.len() + extra.len() <= v.max_program_lines);
                self.lines = [plan.len(), lead.len(), side.as_ref().map_or(0, |(_, _, h)| h.len()), extra.len()];
                if plan.len() + lead.len() <= v.max_program_lines {
                    plan.splice(0..0, lead);
                }
                p.put_program(BUILD as u32, plan, Some(BUILD_ORDER));
                if let Some((s, name, mut held)) = side {
                    // Before its last line (the Dream watch's filler), so a
                    // step reads every guard each loop.
                    let at = held.len() - 1;
                    held.splice(at..at, extra);
                    p.put_program(s as u32, held, Some(name));
                }
            }
        }
        self.loaded_run = run;
        self.load(p);
        self.assign(p);
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
            let [plan, lead, side, extra] = self.lines;
            format!("hands off: loaded slots {:?}; lines: build order {plan} + lead {lead}, side {side} + {extra}", self.loaded)
        } else {
            self.author.note()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lead fits the room the author's build order leaves it, with every
    /// side program saved; else a full build order would drop it whole.
    #[test]
    fn the_lead_fits_the_build_orders_room() {
        let lead = processor_lines(Fx::ONE).len() + seat_lines(&[DREAM, FUND, MARKET]).len();
        assert!(lead <= author::LEAD_LINES, "{lead} > {}", author::LEAD_LINES);
    }
}
