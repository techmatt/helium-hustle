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
    let mut lines = Vec::new();
    let land = Fx::from_int(i64::from(b.land));
    if amount(v, "land") < land {
        let short = Clause { reading: Reading::FreeLand, cmp: Cmp::Lt, value: land };
        lines.push(when(short, Some(stock_at_least(v, "credits", v.land_price))));
        lines.push(Entry::new(Op::BuyLand {}).indent());
    }
    let credits = credits_price(b);
    // The other input furthest from its price (as a share of it).
    let other = b
        .cost
        .iter()
        .filter(|(res, _)| res.as_str() != "credits")
        .min_by_key(|(res, &need)| if need > Fx::ZERO { (amount(v, res) / need).steps() } else { i64::MAX });
    let rest = other.map(|(res, &need)| stock_at_least(v, res, need));
    lines.push(when(stock_at_least(v, "credits", credits), rest));
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
    Clause { reading, cmp, value: Fx::from_int(value) }
}

/// The reading for a resource stock: its fill percent when capped, else its
/// amount (one reading per thing).
fn stock_reading(v: &View, resource: &str) -> Reading {
    if cap(v, resource).is_some() {
        Reading::Percent { resource: resource.to_string() }
    } else {
        Reading::Amount { resource: resource.to_string() }
    }
}

/// `need` in the stock reading's unit: a percent of the cap now when capped,
/// rounded as the reading rounds (`need / cap`, then × 100), so `need` held
/// reads exactly this. A cap that moves after the program is written moves
/// what the percent asks for.
fn stock_value(v: &View, resource: &str, need: Fx) -> Fx {
    match cap(v, resource).filter(|c| *c > Fx::ZERO) {
        Some(c) => need / c * Fx::from_int(100),
        None => need,
    }
}

/// `<resource> above <need less a hair>`: at least `need` (one `Fx` step under
/// it, as `above` is strict).
pub(super) fn stock_at_least(v: &View, resource: &str, need: Fx) -> Clause {
    Clause { reading: stock_reading(v, resource), cmp: Cmp::Gt, value: stock_value(v, resource, need) - Fx::EPSILON }
}

/// `<resource> below <need>`.
pub(super) fn stock_below(v: &View, resource: &str, need: Fx) -> Clause {
    Clause { reading: stock_reading(v, resource), cmp: Cmp::Lt, value: stock_value(v, resource, need) }
}

/// An `act` line.
pub(super) fn act(verb: Verb, target: Option<&str>, amount: Option<Amount>) -> Entry {
    Entry::new(Op::Act { verb, target: target.map(str::to_string), tab: None, amount })
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

/// The main program with conditions, written from the view: its numbers are
/// percents of the caps now and its pad lines read the pads now, so the text
/// changes (and is rewritten) when a cap or the pad count does. Every `if`
/// costs a step (rev 19), so it guards only what a bare line would waste:
/// with no pad it is the filler alone. Once a pad stands, fuel while
/// propellant is under a launch's worth and a pad holds cargo (fewer than all
/// idle); `launch pads all` bare (done when none is full: the step a guard
/// would take); `load pads all` `ship_loads` times once He-3 holds that many
/// loads (at most its cap now): one `if` a batch, not one a load. With two pads or
/// more, `set pads he3 up to 1` bare (done while one carries it) and an idle
/// pad pointed at a good (dearest first, He-3 aside) at its cap while two are
/// idle (it may take the He-3 pad, which the He-3 line then takes back). Then
/// the filler × `fill`.
pub(super) fn ship_cycle(v: &View, filler: &str, ship_loads: i64, fill: u32) -> Vec<Entry> {
    let e = Entry::command;
    let pct = |res: &str, cmp, v| clause(Reading::Percent { resource: res.to_string() }, cmp, v);
    let idle = |cmp, n: i64| clause(Reading::IdlePads, cmp, n);
    let mut lines = Vec::new();
    let pads = v.pads.len() as i64;
    if pads > 0 {
        let batch = cap(v, "he3").map_or(Fx::from_int(LOAD * ship_loads), |c| c.min(Fx::from_int(LOAD * ship_loads)));
        let loads = (batch.floor_int() / LOAD).max(1);
        lines.extend([
            when(stock_below(v, "propellant", Fx::from_int(FUEL)), Some(idle(Cmp::Lt, pads))),
            e("buy_propellant", 1).indent(),
            act(Verb::LaunchPads, None, None),
            when(stock_at_least(v, "he3", Fx::from_int(LOAD * loads)), None),
        ]);
        // The verb, not the command: with no pad to take cargo (one full,
        // cooling) it is done, a step and no energy.
        lines.extend((0..loads).map(|_| act(Verb::LoadPads, None, None).indent()));
    }
    if pads > 1 {
        // He-3 keeps a pad: an idle pad points back (done when one carries it).
        lines.push(act(Verb::SetPads, Some("he3"), Some(Amount::N(1))));
        for good in DEAREST.iter().filter(|&&g| g != "he3") {
            lines.push(when(pct(good, Cmp::Gt, 99), Some(idle(Cmp::Gt, 1))));
            lines.push(act(Verb::SetPads, Some(good), Some(Amount::N(1))).indent());
        }
    }
    lines.push(e(filler, fill));
    lines
}

/// The Dream watch: dream at `dream_at`% of boredom's cap or more, else the
/// filler × `fill`.
pub(super) fn dream_watch(dream_at: i64, filler: &str, fill: u32) -> Vec<Entry> {
    let at = Clause { reading: Reading::Percent { resource: "boredom".to_string() }, cmp: Cmp::Gt, value: Fx::from_int(dream_at) - Fx::EPSILON };
    vec![when(at, None), Entry::command("dream", 1).indent(), Entry::command(filler, fill)]
}

/// The build order program from a run's purchases (`order`, in the order
/// seen): land while free land is under the most a listed building needs,
/// then each building in the order first bought as `build X up to N` (every
/// purchase of it merged), bare: short of credits it fails in one step, the
/// step a credits guard would take (rev 19), so a guard would only add one.
/// With no count reading, the steps are not gated on the one before: each
/// tries every loop it can pay for (done once N are owned), so the cheaper
/// steps fill first. The plan's research comes first, bare too. A step whose price had an input
/// nothing made then (bought with `buy_*` in that run) is led by `if` the
/// input is under what that price asked `and` the credits cover that price
/// and the buy: buy one (as the hand plan buys inputs only from credits past
/// the target's price). At most `max_entries` lines: the author buys the
/// rest at its looks.
/// Returns the lines and how many of `order`'s purchases they cover.
pub(super) fn build_order_program(v: &View, order: &[(String, u32, Vec<Bridge>)], filler: &str) -> (Vec<Entry>, usize) {
    // The config's `max_entries`.
    let max_lines = v.max_program_lines;
    let mut lines = Vec::new();
    let land = order
        .iter()
        .filter_map(|(id, _, _)| v.buildings.iter().find(|b| b.id == id).map(|b| b.land))
        .max()
        .unwrap_or(0);
    if land > 0 {
        // A short `buy land` fails, a step: no credits clause, as the land
        // price moves with every purchase.
        lines.push(when(clause(Reading::FreeLand, Cmp::Lt, i64::from(land)), None));
        lines.push(Entry::new(Op::BuyLand {}).indent());
    }
    // The plan's research, each bought once science covers it (the rest
    // waits for the author's looks).
    lines.extend(research_lines(v, PLAN_RESEARCH));
    // Every purchase of a building merges into one `up to` line, at its first
    // place (each input at the most any of its purchases asked), and counts
    // the purchases merged: with no count reading the steps can't wait on
    // each other, so one line a building keeps the loop short.
    let mut steps: Vec<(&str, u32, Vec<Bridge>, usize)> = Vec::new();
    for (id, c, bridged) in order {
        match steps.iter_mut().find(|s| s.0 == id) {
            Some(last) => {
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
    let mut covered = 0;
    for (id, c, bridged, merged) in steps {
        let lines_needed = 2 * bridged.len() + 1;
        // Room for the step and the filler line.
        if lines.len() + lines_needed + 1 > max_lines {
            break;
        }
        // Its inputs first: one bought a loop while the stock is under what
        // the price asked and the credits cover the price and the buy.
        for (r, need, rich) in bridged {
            lines.push(when(stock_below(v, &r, need), Some(stock_at_least(v, "credits", rich))));
            lines.push(Entry::command(&format!("buy_{r}"), 1).indent());
        }
        // Bare (rev 19): short, it fails in one step, as a false credits
        // guard would take one; affordable or finished, it saves the guard's.
        lines.push(Entry::new(Op::Build { building: id.to_string(), up_to: Some(c) }));
        covered += merged;
    }
    lines.push(Entry::command(filler, 1));
    (lines, covered)
}

/// `research <id>`, bare, for each of `ids` the view lists: short of science
/// it fails in one step, as a false guard would take one; once researched, a
/// done step.
pub fn research_lines(v: &View, ids: &[&str]) -> Vec<Entry> {
    ids.iter().filter(|id| v.research.iter().any(|r| r.id == **id)).map(|id| act(Verb::Research, Some(id), None)).collect()
}
