//! `handsoff`: how far automation alone carries a run. Its first run is the
//! author's (with its knobs), which writes and saves the condition programs.
//! From run 2 on, at each new run it loads them from the library by name
//! (ship → Program 1, Dream watch → 2, fund → 3, market → 4, build order → 5)
//! and assigns processors, and makes no other decision: no purchase, research,
//! project, pad good, pause, processor or retirement by hand. What were its
//! last hands are lines: its build order seats a processor gained mid-run by
//! the reset's rule ([`seat_lines`]); its first side program (the Dream watch)
//! hands a free one to the main program and `retire`s once the run has bought
//! nothing for `stale_days` days ([`hand_lines`]; Dream could otherwise hold
//! boredom off and stall the career), and funds projects ([`fund_lines`]).
//! With no side program saved, the hand lines go in the build order. A
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


/// The build order's lead: while no Launch Pad stands, the ship program has
/// nothing to ship, so the build order takes every processor (its buy of a
/// short input and the build it serves then fall on one day, before the
/// day's clamp cuts the input back to its cap); once one stands, it keeps one
/// and the ship program takes the rest. Every line is guarded on having
/// something to do: a done line costs a step.
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
    // Each guard also reads whether its lines have anything to do (a done
    // line costs a step): the build order short of every working processor
    // (one gained before a pad stands is taken at once), or holding more than
    // one; a maker still running, or not all running.
    let on = |tab: usize, cmp, v: i64| Clause { reading: Reading::Processors { tab: tab as u32 }, cmp, value: Fx::from_int(v), against: None };
    let short = Clause {
        reading: Reading::Processors { tab: BUILD as u32 },
        cmp: Cmp::Lt,
        value: Fx::ONE,
        against: Some(Reading::Amount { resource: "processors".to_string() }),
    };
    let running = |b: &str, cmp, v: i64| Clause { reading: Reading::Running { building: b.to_string() }, cmp, value: Fx::from_int(v), against: None };
    let mut lines = vec![
        when(pads(Cmp::Lt), Some(short)),
        act(Verb::AssignProcessors, None, Some(MAIN), Some(Amount::N(0))),
        act(Verb::AssignProcessors, None, Some(BUILD), Some(Amount::All)),
        when(pads(Cmp::Ge), Some(on(BUILD, Cmp::Ge, 2))),
        act(Verb::AssignProcessors, None, Some(BUILD), Some(Amount::N(1))),
        act(Verb::AssignProcessors, None, Some(MAIN), Some(Amount::All)),
    ];
    for (maker, product) in STOCK {
        lines.push(when(pct(product, Cmp::Ge, 100), Some(running(maker, Cmp::Ge, 1))));
        lines.push(act(Verb::Pause, Some(maker), None, Some(Amount::All)));
        let all = Clause {
            reading: Reading::Running { building: maker.to_string() },
            cmp: Cmp::Lt,
            value: Fx::ONE,
            against: Some(Reading::Count { building: maker.to_string() }),
        };
        lines.push(when(pct(product, Cmp::Lt, 90), Some(all)));
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

fn clause(reading: Reading, cmp: Cmp, v: i64) -> Clause {
    Clause { reading, cmp, value: Fx::from_int(v), against: None }
}

fn when2(a: Clause, b: Clause) -> Entry {
    Entry::new(Op::If { when: Condition { first: a, rest: Some(Joined { join: Join::And, clause: b }) } })
}

fn assign(tab: usize, amount: Amount) -> Entry {
    Entry::new(Op::Act { verb: Verb::AssignProcessors, target: None, tab: Some(tab as u32), amount: Some(amount) }).indent()
}

/// The build order's seat lines, what was the reset's processor rule as
/// lines for a processor gained mid-run: the first side program in `sides`
/// takes one once 3 work (the build order's 1, its own, one for the main
/// program), the rest together once `2 + sides` do; each taken from the
/// build order (before a pad) or the main program (after), the rest handed
/// back to the main program (the lead takes them back for the build order
/// while no pad stands). Two blocks, not one a side, to fit the build
/// order's room for its lead (`author::LEAD_LINES`).
fn seat_lines(sides: &[usize]) -> Vec<Entry> {
    let mut lines = Vec::new();
    let groups: Vec<(&[usize], i64)> = match sides.split_first() {
        None => Vec::new(),
        Some((first, [])) => vec![(std::slice::from_ref(first), 3)],
        Some((first, rest)) => vec![(std::slice::from_ref(first), 3), (rest, 2 + sides.len() as i64)],
    };
    for (group, at) in groups {
        let last = *group.last().expect("a group is never empty");
        lines.push(when2(
            clause(Reading::Processors { tab: last as u32 }, Cmp::Lt, 1),
            clause(Reading::Amount { resource: "processors".to_string() }, Cmp::Ge, at),
        ));
        lines.push(assign(BUILD, Amount::N(1)));
        lines.push(assign(MAIN, Amount::N(0)));
        lines.extend(group.iter().map(|&s| assign(s, Amount::N(1))));
        lines.push(assign(MAIN, Amount::All));
    }
    lines
}

/// The side program's tail, the rest of what were hands: every `FREE_EVERY` days, once
/// a pad stands, a processor left free goes to the main program; and a run
/// that has bought nothing for `stale_days` days retires.
fn hand_lines(stale_days: u64) -> Vec<Entry> {
    vec![
        when2(clause(Reading::Every { days: FREE_EVERY }, Cmp::Eq, 1), clause(Reading::Count { building: "launch_pad".to_string() }, Cmp::Ge, 1)),
        assign(MAIN, Amount::All),
        Entry::new(Op::If { when: Condition { first: clause(Reading::DaysSincePurchase, Cmp::Ge, stale_days as i64), rest: None } }),
        Entry::new(Op::Act { verb: Verb::Retire, target: None, tab: None, amount: None }).indent(),
    ]
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
/// The daily rate per cost resource (the author's `project_rate`), the
/// credits it waits for, and how often it looks (strategy). The period is
/// odd: a one-processor program whose Dream runs every other day reads a
/// guard on alternate days only, and an even period would then never land.
const FUND_RATE: u32 = 3;
const FUND_AT: i64 = 1000;
const FREE_EVERY: u32 = 9;

/// From the day each project opens: while it is not completed and credits
/// are at `FUND_AT` or more, it is funded at `FUND_RATE` a day per cost. A
/// rate set stays (the drain takes only what is in stock), so it needs no
/// stop; a project not open fails, one already at the rate is done, each a
/// step; a completed one's guard skips it for free.
fn fund_lines() -> Vec<Entry> {
    let fund = |id: &str| Entry::new(Op::Act { verb: Verb::Fund, target: Some(id.to_string()), tab: None, amount: Some(Amount::N(FUND_RATE)) }).indent();
    let mut lines = Vec::new();
    for id in PROJECTS {
        lines.push(when2(
            clause(Reading::Completed { project: id.to_string() }, Cmp::Lt, 1),
            clause(Reading::Amount { resource: "credits".to_string() }, Cmp::Ge, FUND_AT),
        ));
        lines.push(fund(id));
    }
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
        // Between resets it does nothing: its programs hold the rest.
        if self.loaded_run == run {
            return;
        }
        if self.loaded_run == 0 {
            // The author's last act, once: its build order from run 1,
            // saved to the library (the author writes it at a new run).
            self.author.track(v);
            let mut plan = self.author.build_order().to_vec();
            if !plan.is_empty() {
                let v = p.view();
                let sides: Vec<usize> =
                    SLOTS.iter().filter(|(s, name)| *s != BUILD && v.library.iter().any(|l| l.name == *name)).map(|(s, _)| *s).collect();
                let mut lead = processor_lines();
                lead.extend(seat_lines(&sides));
                // The rest goes to the first side program (the Dream watch
                // when saved): the build order has room for its lead only.
                // Funding first: a true block's body takes the day's step, so
                // a periodic guard after it misses that day (the free
                // processor's misses a look now and then).
                let mut extra = fund_lines();
                extra.extend(hand_lines(self.stale_days));
                extra.extend(research_tail(v));
                let side = sides.first().and_then(|&s| {
                    let name = SLOTS.iter().find(|(x, _)| *x == s)?.1;
                    let held = &v.library.iter().find(|l| l.name == name)?.entries;
                    Some((s, name, held.clone()))
                });
                let side = side.filter(|(_, _, h)| !h.is_empty() && h.len() + extra.len() <= v.max_program_lines);
                if side.is_none() {
                    // No side program to carry them: the hand lines (retire
                    // included) lead the build order when they fit.
                    let hands = hand_lines(self.stale_days);
                    if plan.len() + lead.len() + hands.len() <= v.max_program_lines {
                        lead.extend(hands);
                    }
                }
                self.lines = [plan.len(), lead.len(), side.as_ref().map_or(0, |(_, _, h)| h.len()), extra.len()];
                if plan.len() + lead.len() <= v.max_program_lines {
                    plan.splice(0..0, lead);
                }
                p.put_program(BUILD as u32, plan, Some(BUILD_ORDER));
                if let Some((s, name, mut held)) = side {
                    // Before its last line (the Dream watch's filler), so a
                    // step reads every guard each loop: after it, a
                    // one-processor loop would read them every other day
                    // only, and an even period on one parity never.
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
        let lead = processor_lines().len() + seat_lines(&[DREAM, FUND, MARKET]).len();
        assert!(lead <= author::LEAD_LINES, "{lead} > {}", author::LEAD_LINES);
    }
}
