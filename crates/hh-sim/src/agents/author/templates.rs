//! The author's program templates (docs/agents.md, *The program author*):
//! the main program's hand plan and standing cycle, the side programs, the
//! condition programs it saves by name (`ship`, `dream watch`, `build order`)
//! and the line builders they share. Free functions of the view: the
//! author's state and choices stay in `author.rs`.

use super::*;

/// The main program while looks come often (`gap <= hand_gap`): a plan for the
/// next gap, what is due now, then `fill` of the filler to the next look.
/// `load_pads` and each bridge buy are capped at `loads` (the knob).
pub(super) fn hand_plan(
    v: &View,
    pads: &[&PadView],
    target: Option<&str>,
    filler: &str,
    fill: u32,
    wait_demand: bool,
    loads: u32,
) -> Vec<Entry> {
    let e = Entry::command;
    let mut lines = Vec::new();
    let credits = amount(v, "credits");
    let keep = target.map_or(Fx::ZERO, |t| price(v, t, "credits"));
    if let Some(pad) = pads.first() {
        let full = pad.cargo >= pad.capacity;
        // Launch into good demand: HIGH or better, or MEDIUM once
        // the good piles at its cap (waiting would waste it).
        let tier = v.markets.iter().find(|m| m.good == pad.good).map_or("", |m| m.tier);
        let piled = cap(v, &pad.good).is_some_and(|c| amount(v, &pad.good) >= c);
        let good_demand = match tier {
            "HIGH" | "VERY HIGH" => true,
            "MEDIUM" => piled || !wait_demand,
            _ => !wait_demand,
        };
        if full && pad.can_launch && good_demand {
            lines.push(e("launch_pads", 1));
        }
        let short = (pad.fuel - amount(v, "propellant")).floor_int();
        let fuel_cost = command_cost(v, "buy_propellant", "credits");
        // Strategy: fuel once the pad is within 25 cargo of full.
        let near = pad.cargo + Fx::from_int(25) >= pad.capacity;
        if near && short > 0 && fuel_cost > Fx::ZERO {
            let n = short.min((credits / fuel_cost).floor_int());
            if n > 0 {
                lines.push(e("buy_propellant", n as u32));
            }
        }
        // Loads worth running (a partial load still counts).
        let per = pad.load.floor_int().max(1);
        let stock = (amount(v, &pad.good).floor_int() + per - 1) / per;
        let room = ((pad.capacity - pad.cargo).floor_int() + per - 1) / per;
        if !full && stock > 0 && room > 0 {
            lines.push(e("load_pads", stock.min(room).min(i64::from(loads)) as u32));
        }
    }
    if let Some(b) = target.and_then(|t| v.buildings.iter().find(|b| b.id == t)) {
        let mut spare = credits - keep;
        for &(res, cmd) in INPUT_BRIDGES {
            let need = b.cost.get(res).copied().unwrap_or(Fx::ZERO) - amount(v, res);
            let each = command_cost(v, cmd, "credits");
            if need > Fx::ZERO && each > Fx::ZERO && command_unlocked(v, cmd) {
                let n = need.floor_int().saturating_add(1).min((spare / each).floor_int());
                if n > 0 {
                    lines.push(e(cmd, n as u32));
                    spare -= each.mul_int(n);
                }
            }
        }
    }
    lines.push(e(filler, fill));
    lines
}

/// The main program for longer gaps: a standing cycle that ships and earns
/// unattended. It earns before it spends, so a cycle started broke still pays
/// its way.
pub(super) fn standing_cycle(v: &View, pads: &[&PadView], target: Option<&str>, filler: &str, loads: u32) -> Vec<Entry> {
    let e = Entry::command;
    let mut lines = Vec::new();
    let shipping = pads.first().filter(|_| command_unlocked(v, "load_pads"));
    if shipping.is_some() {
        lines.push(e("load_pads", loads));
        lines.push(e("launch_pads", 1));
    }
    lines.push(e(filler, loads));
    if let Some(pad) = shipping {
        lines.push(e("buy_propellant", pad.fuel.floor_int().max(1) as u32));
    }
    // Bridges for the target's inputs no building makes (`loads` caps the
    // overbuy). Bridging what a building makes too, as far as spare credits
    // go, got the casual author its second processor in run 1 but cost it
    // the first shipment there (4 seeds).
    if let Some(b) = target.and_then(|t| v.buildings.iter().find(|b| b.id == t)) {
        for &(res, cmd) in INPUT_BRIDGES {
            let need = b.cost.get(res).copied().unwrap_or(Fx::ZERO) - amount(v, res);
            let made = made(v, res);
            if need > Fx::ZERO && !made && command_unlocked(v, cmd) {
                lines.push(e(cmd, need.floor_int().saturating_add(1).min(i64::from(loads)) as u32));
            }
        }
    }
    lines
}

/// Dream processors while holding boredom down (`sustain`): enough to
/// outpace growth and a command's boredom a processor once boredom passes a
/// third of its cap; one more past two thirds.
pub(super) fn sustain_dreamers(v: &View, total: u32) -> u32 {
    let two = v.boredom.amount.mul_int(3) >= v.boredom.cap.mul_int(2);
    // An allowance of 0.2 boredom a processor a tick (mirrors no one value:
    // twice Cloud Compute's 0.1, content/commands.json).
    let cmds = Fx::from_int(i64::from(total)).mul_ratio(1, 5);
    // Halved: Dream's relief is 2 a processor (mirrors configs/baseline.json
    // `dream.relief`, before the Humanist bonus).
    (v.boredom.rate + cmds).mul_ratio(1, 2).floor_int() as u32 + 1 + u32::from(two)
}

/// The author's own axis to fund: the first in [`AXES`] whose persistent
/// project isn't done.
pub(super) fn own_axis(v: &View) -> Option<&'static str> {
    AXES.iter().find(|(_, proj)| !v.projects.iter().any(|p| p.id == *proj && p.completed)).map(|(a, _)| *a)
}

/// The market program: promote what ships, disrupt speculators (empty when
/// none is unlocked).
pub(super) fn market_program(v: &View, pads: &[&PadView]) -> Vec<Entry> {
    let e = Entry::command;
    let mut lines = Vec::new();
    for pad in pads {
        let cmd = format!("promote_{}", pad.good);
        if command_unlocked(v, &cmd) && !lines.iter().any(|l: &Entry| l.command_id() == Some(cmd.as_str())) {
            lines.push(e(&cmd, 1));
        }
    }
    if command_unlocked(v, "disrupt_speculators") {
        lines.push(e("disrupt_speculators", 1));
    }
    lines
}

/// The construction program for `b`: land when it is short of it and the
/// credits are there, then one more `b` once its credits and its scarcest
/// other input are in stock, then a filler line for the days neither holds.
pub(super) fn construction(v: &View, b: &hh_core::view::BuildingView, filler: &str) -> Vec<Entry> {
    let at_least = |resource: &str, value: Fx| Clause {
        reading: Reading::Amount { resource: resource.to_string() },
        cmp: Cmp::Ge,
        value,
        against: None,
    };
    let mut lines = Vec::new();
    let land = Fx::from_int(i64::from(b.land));
    if amount(v, "land") < land {
        let short = Clause { reading: Reading::Amount { resource: "land".to_string() }, cmp: Cmp::Lt, value: land, against: None };
        let rest = Some(Joined { join: Join::And, clause: at_least("credits", v.land_price) });
        lines.push(Entry::new(Op::If { when: Condition { first: short, rest } }));
        lines.push(Entry::new(Op::BuyLand {}).indent());
    }
    let credits = credits_price(b);
    // The other input furthest from its price (as a share of it).
    let other = b
        .cost
        .iter()
        .filter(|(res, _)| res.as_str() != "credits")
        .min_by_key(|(res, &need)| if need > Fx::ZERO { (amount(v, res) / need).steps() } else { i64::MAX });
    let rest = other.map(|(res, &need)| Joined { join: Join::And, clause: at_least(res, need) });
    lines.push(Entry::new(Op::If { when: Condition { first: at_least("credits", credits), rest } }));
    lines.push(Entry::new(Op::Build { building: b.id.to_string(), up_to: Some(b.owned + 1) }).indent());
    lines.push(Entry::command(filler, 1));
    lines
}

/// Whether the target is short of an input a `buy_*` command brings, with
/// credits past its price for at least one (what a hand plan would buy).
pub(super) fn bridge_due(v: &View, target: Option<&str>) -> bool {
    let Some(b) = target.and_then(|t| v.buildings.iter().find(|b| b.id == t)) else { return false };
    let spare = amount(v, "credits") - credits_price(b);
    INPUT_BRIDGES.iter().any(|&(res, cmd)| {
        let each = command_cost(v, cmd, "credits");
        b.cost.get(res).is_some_and(|&need| need > amount(v, res))
            && command_unlocked(v, cmd)
            && each > Fx::ZERO
            && spare >= each
    })
}

/// An input a purchase bought with `buy_*`, how much of it its price asked,
/// and the credits that buy one past its credits price.
pub(super) type Bridge = (String, Fx, Fx);

pub(super) fn clause(reading: Reading, cmp: Cmp, value: i64) -> Clause {
    Clause { reading, cmp, value: Fx::from_int(value), against: None }
}

/// `<reading> <cmp> <against> × <times>`.
pub(super) fn clause_times(reading: Reading, cmp: Cmp, against: Reading, times: i64) -> Clause {
    Clause { reading, cmp, value: Fx::from_int(times), against: Some(against) }
}

/// An `act` line.
pub(super) fn act(verb: Verb, target: Option<&str>, amount: Option<Amount>) -> Entry {
    Entry::new(Op::Act { verb, target: target.map(str::to_string), tab: None, amount })
}

pub(super) fn amount_of(resource: &str) -> Reading {
    Reading::Amount { resource: resource.to_string() }
}

/// `if a [and b]`.
pub(super) fn when(first: Clause, and: Option<Clause>) -> Entry {
    let rest = and.map(|clause| Joined { join: Join::And, clause });
    Entry::new(Op::If { when: Condition { first, rest } })
}

/// Propellant a launch burns and He-3 a load moves (mirror configs/baseline.json
/// `fuel_per_pad` 20 and `load_per_execution` 5).
pub(super) const FUEL: i64 = 20;
pub(super) const LOAD: i64 = 5;

/// Pads carrying `good`.
pub(super) fn on_good(good: &str) -> Reading {
    Reading::Pads { good: Some(good.to_string()) }
}

/// `Launch Pad count >= 1`.
pub(super) fn pad_stands() -> Clause {
    clause(Reading::Count { building: "launch_pad".to_string() }, Cmp::Ge, 1)
}

/// Lines the build order leaves free for the hands-off player's lead
/// (`handsoff::processor_lines`, 16 stock and processor lines, and
/// `handsoff::seat_lines`, at most 12: 11 with three side programs).
pub const LEAD_LINES: usize = 16 + 12;

/// Cargo a pad holds (mirrors configs/baseline.json `pad_cargo_capacity`).
pub(super) const PAD_CARGO: i64 = 100;

/// The main program with conditions: one text for every run. Fuel a launch
/// once a Launch Pad stands (an He-3 stock as the "pad near full" sign bought
/// fuel with no pad at all), launch ready pads, load while there is a load of
/// He-3 and room aboard (`cargo of pads < Launch Pad count × 100`, so a full
/// pad cooling down takes no failed load), keep a pad on He-3 while none
/// carries it, point an idle pad at a good (dearest first, He-3 aside)
/// sitting at its cap while two carry He-3, else the filler. Every guard
/// reads whether its line has anything to do: a done line costs a step.
pub(super) fn ship_cycle(filler: &str) -> Vec<Entry> {
    let e = Entry::command;
    let pads = || Reading::Count { building: "launch_pad".to_string() };
    let mut lines = vec![
        when(clause(amount_of("propellant"), Cmp::Lt, FUEL), Some(pad_stands())),
        e("buy_propellant", 1).indent(),
        when(clause(Reading::PadsReady { good: None }, Cmp::Ge, 1), None),
        act(Verb::LaunchPads, None, None).indent(),
        when(clause(amount_of("he3"), Cmp::Ge, LOAD), Some(clause_times(Reading::Cargo { good: None }, Cmp::Lt, pads(), PAD_CARGO))),
        act(Verb::LoadPads, None, None).indent(),
        // He-3 keeps a pad: with none on it, an idle pad points back
        // (guarded: a done line costs a step).
        when(clause(on_good("he3"), Cmp::Lt, 1), Some(pad_stands())),
        act(Verb::SetPads, Some("he3"), Some(Amount::N(1))).indent(),
    ];
    for good in DEAREST.iter().filter(|&&g| g != "he3") {
        // Once two pads carry He-3 (so one is spare), a good at its cap
        // takes one; with its pad taken He-3 holds one fewer, so with two
        // pads the guard then reads false (with more, the line is a done
        // step while the good stays full: three clauses don't fit one `if`).
        let full = clause(Reading::Percent { resource: good.to_string() }, Cmp::Ge, 100);
        lines.push(when(full, Some(clause(on_good("he3"), Cmp::Ge, 2))));
        lines.push(act(Verb::SetPads, Some(good), Some(Amount::N(1))).indent());
    }
    lines.push(e(filler, 1));
    lines
}

/// The Dream watch: dream past `dream_at`% of boredom's cap, else the filler.
pub(super) fn dream_watch(dream_at: i64, filler: &str) -> Vec<Entry> {
    vec![
        when(clause(Reading::Percent { resource: "boredom".to_string() }, Cmp::Ge, dream_at), None),
        Entry::command("dream", 1).indent(),
        Entry::command(filler, 1),
    ]
}

/// The build order program from a run's purchases (`order`, in the order
/// seen): land while free land is under the most a listed building needs,
/// then each purchase in turn as `build X up to N` (a run of one building
/// merged), each past the first in the body of an `if` that the one before it
/// is owned, so it keeps the order and reads how far the run has come rather
/// than the day. A step whose price had an input nothing made then (bought
/// with `buy_*` in that run) is led by `if` the input is under what that
/// price asked `and` the credits cover that price and the buy: buy one (as
/// the hand plan buys inputs only from credits past the target's price, so
/// a step far ahead buys only once the run can afford it). The line outlives
/// its step: once built, it keeps that input topped up to the old need while
/// credits stay past the old price (gating it on the step unbuilt instead,
/// with no credits clause, bought early and cut the hands-off player's run-2
/// credits from 13% of the author's to 1%: dev seeds 1-8). Each `build` is
/// also guarded on fewer than N owned, so a finished step is a free false
/// `if`, not a done step; the first not done tries once a loop (failing when
/// short), then the filler line. At most `max_entries` less [`LEAD_LINES`] lines: the author buys the
/// rest at its looks.
/// Returns the lines and how many of `order`'s purchases they cover.
pub(super) fn build_order_program(v: &View, order: &[(String, u32, Vec<Bridge>)], filler: &str) -> (Vec<Entry>, usize) {
    // The config's `max_entries`, less the hands-off player's lead
    // (`handsoff::processor_lines`).
    let max_lines = v.max_program_lines.saturating_sub(LEAD_LINES);
    let mut lines = Vec::new();
    let land = order
        .iter()
        .filter_map(|(id, _, _)| v.buildings.iter().find(|b| b.id == id).map(|b| b.land))
        .max()
        .unwrap_or(0);
    if land > 0 {
        // Gated on the land price as it reads then (`Credits >= Land price × 1`).
        let short = clause(amount_of("land"), Cmp::Lt, i64::from(land));
        let flush = clause_times(amount_of("credits"), Cmp::Ge, Reading::LandPrice, 1);
        lines.push(when(short, Some(flush)));
        lines.push(Entry::new(Op::BuyLand {}).indent());
    }
    // The plan's research, each once science covers its price (the rest
    // waits for the author's looks).
    lines.extend(research_lines(v, PLAN_RESEARCH));
    // Runs of the same building merge into one `up to` line (each input at
    // the most any of its purchases asked), and count the purchases merged.
    let mut steps: Vec<(&str, u32, Vec<Bridge>, usize)> = Vec::new();
    for (id, c, bridged) in order {
        match steps.last_mut() {
            Some(last) if last.0 == id => {
                last.1 = last.1.max(*c);
                last.3 += 1;
                for (r, n, rich) in bridged {
                    match last.2.iter_mut().find(|x| x.0 == *r) {
                        Some(x) => {
                            x.1 = x.1.max(*n);
                            x.2 = x.2.max(*rich);
                        }
                        None => last.2.push((r.clone(), *n, *rich)),
                    }
                }
            }
            _ => steps.push((id, *c, bridged.clone(), 1)),
        }
    }
    let count = |b: &str, cmp: Cmp, n: u32| clause(Reading::Count { building: b.to_string() }, cmp, i64::from(n));
    let mut prev: Option<(&str, u32)> = None;
    let mut covered = 0;
    for (id, c, bridged, merged) in steps {
        let after = prev.map(|(p, n)| count(p, Cmp::Ge, n));
        let lines_needed = 2 * bridged.len() + 2;
        // Room for the step and the filler line.
        if lines.len() + lines_needed + 1 > max_lines {
            break;
        }
        // Its inputs first: one bought a loop while the stock is under what
        // the price asked and the credits cover the price and the buy.
        for (r, need, rich) in bridged {
            let short = Clause { reading: amount_of(&r), cmp: Cmp::Lt, value: need, against: None };
            let flush = Clause { reading: amount_of("credits"), cmp: Cmp::Ge, value: rich, against: None };
            lines.push(when(short, Some(flush)));
            lines.push(Entry::command(&format!("buy_{r}"), 1).indent());
        }
        // Guarded on fewer than `c` owned too: a done line costs a step.
        let short = count(id, Cmp::Lt, c);
        match after {
            Some(a) => lines.push(when(a, Some(short))),
            None => lines.push(when(short, None)),
        }
        lines.push(Entry::new(Op::Build { building: id.to_string(), up_to: Some(c) }).indent());
        prev = Some((id, c));
        covered += merged;
    }
    lines.push(Entry::command(filler, 1));
    (lines, covered)
}

/// `if Science >= <its price now> and Researched <id> < 1` / `research <id>`,
/// for each of `ids` the view lists (guarded: a done line costs a step).
pub fn research_lines(v: &View, ids: &[&str]) -> Vec<Entry> {
    let mut lines = Vec::new();
    for id in ids {
        if let Some(cost) = v.research.iter().find(|r| r.id == *id).map(|r| r.cost) {
            let undone = clause(Reading::Researched { item: id.to_string() }, Cmp::Lt, 1);
            lines.push(when(Clause { reading: amount_of("science"), cmp: Cmp::Ge, value: cost, against: None }, Some(undone)));
            lines.push(act(Verb::Research, Some(id), None).indent());
        }
    }
    lines
}
