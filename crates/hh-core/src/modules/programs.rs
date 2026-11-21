//! **programs**: program tabs (command queues), processors, command execution
//! with output-cap skips and partial production, and overclocks
//! (docs/rules/programs.md).

use std::collections::BTreeMap;

use super::{param, ModuleDef, ParamKind as K, VariantDef};
use crate::config::ModuleConfig;
use crate::content::{content, Command, CommandEffect, ProjectReward, ResearchEffect};
use crate::fx::Fx;
use crate::modifiers::modifier;
use crate::rules::Rules;
use crate::state::{Amount, Clause, Cmp, Condition, Entry, GameState, Join, Joined, LineOutcome, Loadout, Op, Overclock, Program, Reading, Verb};

use super::economy::{BOREDOM, PROCESSORS};

pub static DEF: ModuleDef = ModuleDef {
    id: "programs",
    variants: &[VariantDef {
        name: "arc1",
        params: &[
            param("program_count", K::Int, Some(("1", "100")), "program tabs"),
            param("max_entries", K::Int, Some(("1", "10000")), "most lines in one program"),
            param("overclock_base_cap", K::Fx, Some(("1", "100")), "the most an overclock multiplies output, before research"),
            param("logic_unlock_processors", K::Int, Some(("0", "1000")), "working processors that first open every line but plain commands (a career first)"),
            param("library_slots", K::Int, Some(("0", "1000")), "saved programs the loadout library holds (career state)"),
            param("loadout_name_max", K::Int, Some(("1", "1000")), "most characters in a loadout's name"),
        ],
        rng: &[],
    }],
};

#[derive(Clone)]
pub struct Params {
    pub program_count: usize,
    pub max_entries: usize,
    pub overclock_base_cap: Fx,
    pub logic_unlock_processors: u32,
    pub library_slots: usize,
    pub loadout_name_max: usize,
}

impl Params {
    pub fn new(mc: &ModuleConfig) -> Result<Params, String> {
        Ok(Params {
            program_count: mc.int("program_count") as usize,
            max_entries: mc.int("max_entries") as usize,
            overclock_base_cap: mc.fx("overclock_base_cap"),
            logic_unlock_processors: mc.int("logic_unlock_processors") as u32,
            library_slots: mc.int("library_slots") as usize,
            loadout_name_max: mc.int("loadout_name_max") as usize,
        })
    }
}

/// Empty programs with every processor on the first (a new run).
pub fn new_programs(r: &Rules, processors: u32) -> Vec<Program> {
    let mut ps = vec![Program::default(); r.p.programs.program_count];
    if let Some(p) = ps.first_mut() {
        p.processors = processors;
    }
    ps
}

/// Processors working now: the processors resource, granted by the working
/// (upkeep-paid) copies. A day runs at most this many.
pub fn processors(g: &GameState) -> u32 {
    u32::try_from(g.amount(PROCESSORS).floor_int().max(0)).unwrap_or(u32::MAX)
}

/// Processors the active buildings grant, paid or not: what programs may have
/// assigned. A day short of upkeep runs fewer ([`running`]) but keeps them.
pub fn capacity(g: &GameState) -> u32 {
    let total = super::economy::grants(PROCESSORS, |id| g.active(id));
    u32::try_from(total.floor_int().max(0)).unwrap_or(u32::MAX)
}

pub fn assigned(g: &GameState) -> u32 {
    g.run.programs.iter().map(|p| p.processors).sum()
}

/// How many of each program's assigned processors run today: all, unless fewer
/// are working than assigned, when the shortfall comes off the last programs first.
pub fn running(g: &GameState) -> Vec<u32> {
    let mut short = assigned(g).saturating_sub(processors(g));
    let mut out: Vec<u32> = g.run.programs.iter().map(|p| p.processors).collect();
    for n in out.iter_mut().rev() {
        let cut = short.min(*n);
        *n -= cut;
        short -= cut;
    }
    out
}

/// Processors lost (a Data Center sold or switched off) come off the last
/// programs first; ones only unpaid stay assigned ([`running`]).
pub fn trim_processors(g: &mut GameState) {
    let mut over = assigned(g).saturating_sub(capacity(g));
    for p in g.run.programs.iter_mut().rev() {
        let take = over.min(p.processors);
        p.processors -= take;
        over -= take;
    }
}

fn program_mut(g: &mut GameState, program: u32) -> Result<&mut Program, String> {
    let n = g.run.programs.len();
    g.run
        .programs
        .get_mut(program as usize)
        .ok_or_else(|| format!("Program {}: there are {n}", program + 1))
}

/// Replaces a program's queue; its pointer restarts at the top.
pub fn set_program(r: &Rules, g: &mut GameState, program: u32, entries: &[Entry]) -> Result<(), String> {
    let held = g.run.programs.get(program as usize).map(|p| p.entries.clone()).unwrap_or_default();
    check_entries(r, g, program, entries, Some(&held))?;
    put_entries(g, program, entries)
}

/// Whether `entries` may go in a program. With `held` (the program's lines
/// now), a line not already among them is also refused for what this run
/// can't use yet (a line other than a command before the logic unlock, a
/// demand reading before demand is revealed). A loaded loadout (`held` `None`)
/// skips those, and so does an edit of a loaded program that keeps the line:
/// such a line fails, or its reading reads nothing, when run.
fn check_entries(r: &Rules, g: &GameState, program: u32, entries: &[Entry], held: Option<&[Entry]>) -> Result<(), String> {
    if entries.len() > r.p.programs.max_entries {
        return Err(format!("Program {}: at most {} lines", program + 1, r.p.programs.max_entries));
    }
    let n = program + 1;
    let c = content();
    let traded = |good: &str| -> Result<(), String> {
        r.p.shipments.base_values.contains_key(good).then_some(()).ok_or_else(|| format!("Program {n}: {good:?} is not a traded good"))
    };
    let building = |id: &str| -> Result<(), String> {
        c.building(id).map(|_| ()).ok_or_else(|| format!("Program {n}: unknown building {id:?}"))
    };
    let one = |reading: &Reading, gated: bool| -> Result<(), String> {
        let known = |res: &str| c.resource(res).is_some();
        match reading {
            Reading::Amount { resource }
            | Reading::Percent { resource }
            | Reading::Rate { resource }
            | Reading::DaysToCap { resource } => {
                known(resource).then_some(()).ok_or_else(|| format!("Program {n}: unknown resource {resource:?}"))
            }
            Reading::Count { building: b } | Reading::NextCost { building: b } => building(b),
            Reading::Day | Reading::LandPrice | Reading::DaysSinceLaunch => Ok(()),
            Reading::PadsReady { good } | Reading::Cargo { good } => good.as_deref().map_or(Ok(()), traded),
            Reading::Every { days } => {
                (*days > 0).then_some(()).ok_or_else(|| format!("Program {n}: every 0 days"))
            }
            Reading::Demand { good } => {
                traded(good)?;
                if gated && !super::research::reveals_demand(g) {
                    return Err(format!("Program {n}: demand can't be read yet"));
                }
                Ok(())
            }
        }
    };
    let clause = |cl: &Clause, gated: bool| -> Result<(), String> {
        one(&cl.reading, gated)?;
        cl.against.as_ref().map_or(Ok(()), |a| one(a, gated))
    };
    let condition = |when: &Condition, gated: bool| -> Result<(), String> {
        clause(&when.first, gated)?;
        when.rest.as_ref().map_or(Ok(()), |j| clause(&j.clause, gated))
    };
    // Whether the line above is an `if` or in a body: only then may a line be indented.
    let mut in_block = false;
    for e in entries {
        let gated = held.is_some_and(|h| !h.iter().any(|x| x.op == e.op));
        if gated && !matches!(e.op, Op::Command { .. }) && !logic_unlocked(g) {
            return Err(format!("Program {n}: only commands can be queued yet"));
        }
        if e.indented && !in_block {
            return Err(format!("Program {n}: an indented line needs an if above it"));
        }
        in_block = e.indented;
        match &e.op {
            Op::Command { command, count } => {
                if c.command(command).is_none() {
                    return Err(format!("Program {n}: unknown command {command:?}"));
                }
                if *count == 0 {
                    return Err(format!("Program {n}: {command} runs 0 times"));
                }
            }
            Op::Build { building, up_to } => {
                if c.building(building).is_none() {
                    return Err(format!("Program {n}: unknown building {building:?}"));
                }
                if *up_to == Some(0) {
                    return Err(format!("Program {n}: build {building} up to 0"));
                }
            }
            Op::BuyLand {} => {}
            Op::If { when } => {
                if e.indented {
                    return Err(format!("Program {n}: an if can't sit inside another if"));
                }
                condition(when, gated)?;
                in_block = true;
            }
            Op::WaitUntil { when } => condition(when, gated)?,
            Op::Act { verb, target, tab, amount } => check_act(r, g, n, *verb, target.as_deref(), *tab, *amount)?,
        }
    }
    if program as usize >= g.run.programs.len() {
        return Err(format!("Program {n}: there are {}", g.run.programs.len()));
    }
    Ok(())
}

/// What each verb's slots take (docs/rules/programs.md, *Verbs*): a slot the
/// verb has no use for must be empty, and an amount is never 0.
fn check_act(r: &Rules, g: &GameState, n: u32, verb: Verb, target: Option<&str>, tab: Option<u32>, amount: Option<Amount>) -> Result<(), String> {
    let c = content();
    let traded = |good: &str| r.p.shipments.base_values.contains_key(good);
    #[derive(PartialEq)]
    enum Takes {
        No,
        Optional,
        Required,
    }
    // (target ok, tab wanted, amount)
    let (target_ok, wants_tab, amount_rule) = match verb {
        Verb::LaunchPads | Verb::LoadPads => (target.is_none_or(traded), false, Takes::No),
        Verb::SetPads => (target.is_some_and(traded), false, Takes::Required),
        Verb::Pause | Verb::Resume => (target.is_some_and(|b| c.building(b).is_some()), false, Takes::Optional),
        Verb::Research => (target.is_some_and(|i| c.research_item(i).is_some()), false, Takes::No),
        Verb::LoadProgram => (target.is_some_and(|s| !s.trim().is_empty()), true, Takes::No),
        Verb::AssignProcessors => (target.is_none(), true, Takes::Required),
        Verb::Retire => (target.is_none(), false, Takes::No),
    };
    let what = format!("Program {n}: {verb:?}");
    if !target_ok {
        return Err(format!("{what}: no such target {target:?}"));
    }
    match (wants_tab, tab) {
        (true, None) => return Err(format!("{what} needs a program tab")),
        (true, Some(t)) if t as usize >= g.run.programs.len() => {
            return Err(format!("{what}: Program {}: there are {}", t + 1, g.run.programs.len()))
        }
        (false, Some(_)) => return Err(format!("{what} takes no program tab")),
        _ => {}
    }
    match (amount_rule, amount) {
        (Takes::No, Some(_)) => Err(format!("{what} takes no amount")),
        (Takes::Required, None) => Err(format!("{what} needs an amount")),
        (_, Some(Amount::N(0))) if verb != Verb::AssignProcessors => Err(format!("{what}: an amount of 0")),
        (_, Some(Amount::All)) if verb == Verb::SetPads => Err(format!("{what}: up to N, not all")),
        _ => Ok(()),
    }
}

fn put_entries(g: &mut GameState, program: u32, entries: &[Entry]) -> Result<(), String> {
    let p = program_mut(g, program)?;
    p.entries = entries.to_vec();
    p.entry = 0;
    p.step = 0;
    // Today's line outcomes named the old lines.
    g.run.today.lines.remove(&program);
    let prefix = format!("program:{program}:");
    g.run.today.shorts.retain(|k, _| !k.starts_with(&prefix));
    Ok(())
}

/// A loadout name as kept: trimmed, not empty, at most `loadout_name_max`
/// characters.
fn loadout_name(r: &Rules, name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("A loadout needs a name".to_string());
    }
    let max = r.p.programs.loadout_name_max;
    if name.chars().count() > max {
        return Err(format!("A loadout name is at most {max} characters"));
    }
    Ok(name.to_string())
}

fn loadout_mut(g: &mut GameState, loadout: u32) -> Result<&mut Loadout, String> {
    let n = g.career.library.len();
    g.career.library.get_mut(loadout as usize).ok_or_else(|| format!("Loadout {}: there are {n}", loadout + 1))
}

/// Saves a program's lines to the library: a new slot (`slot` `None`, while
/// fewer than `library_slots` are held) or over an existing one.
pub fn save_loadout(r: &Rules, g: &mut GameState, program: u32, slot: Option<u32>, name: &str) -> Result<(), String> {
    let name = loadout_name(r, name)?;
    let entries = g
        .run
        .programs
        .get(program as usize)
        .map(|p| p.entries.clone())
        .ok_or_else(|| format!("Program {}: there are {}", program + 1, g.run.programs.len()))?;
    if entries.is_empty() {
        return Err(format!("Program {}: nothing to save", program + 1));
    }
    let l = Loadout { name, entries };
    match slot {
        None => {
            if g.career.library.len() >= r.p.programs.library_slots {
                return Err(format!("The library is full ({} loadouts)", r.p.programs.library_slots));
            }
            g.career.library.push(l);
        }
        Some(i) => *loadout_mut(g, i)? = l,
    }
    Ok(())
}

/// Replaces a program's lines with a loadout's, as `set_program` would, but
/// without refusing what this run hasn't unlocked yet: those lines fail, or
/// read nothing, when run.
pub fn load_loadout(r: &Rules, g: &mut GameState, loadout: u32, program: u32) -> Result<(), String> {
    let entries = loadout_mut(g, loadout)?.entries.clone();
    check_entries(r, g, program, &entries, None)?;
    put_entries(g, program, &entries)
}

pub fn rename_loadout(r: &Rules, g: &mut GameState, loadout: u32, name: &str) -> Result<(), String> {
    let name = loadout_name(r, name)?;
    loadout_mut(g, loadout)?.name = name;
    Ok(())
}

/// Removes a loadout; later ones move up a slot.
pub fn delete_loadout(g: &mut GameState, loadout: u32) -> Result<(), String> {
    loadout_mut(g, loadout)?;
    g.career.library.remove(loadout as usize);
    Ok(())
}

/// Sets how many processors run a program (all programs together: at most the
/// active buildings grant, paid or not).
pub fn set_processors(g: &mut GameState, program: u32, n: u32) -> Result<(), String> {
    let total = capacity(g);
    let others = assigned(g) - program_mut(g, program)?.processors;
    let free = total.saturating_sub(others);
    if n > free {
        return Err(format!("Program {}: {n} processors, but only {free} free", program + 1));
    }
    program_mut(g, program)?.processors = n;
    Ok(())
}

/// The command's unlock holds now.
pub fn unlocked(g: &GameState, c: &Command) -> bool {
    super::economy::requirement_met(g, &c.requires)
}

/// Shown: unlocked, or used in any run.
pub fn visible(g: &GameState, c: &Command) -> bool {
    unlocked(g, c) || g.career.used_commands.contains(&c.id)
}

/// A command's costs and production now: research cost overrides, then its
/// scale modifier on both.
pub fn flows(r: &Rules, g: &GameState, c: &Command) -> (BTreeMap<String, Fx>, BTreeMap<String, Fx>) {
    let mut costs = c.costs.clone();
    for effect in super::research::live_effects(g) {
        if let ResearchEffect::CommandCostOverride { command, resource, value } = effect {
            if command == &c.id {
                costs.insert(resource.clone(), *value);
            }
        }
    }
    let mut produces = c.produces.clone();
    if let Some(k) = &c.scale_modifier {
        let m = modifier(r, g, k);
        for v in costs.values_mut().chain(produces.values_mut()) {
            *v = *v * m;
        }
    }
    (costs, produces)
}

/// The flat boredom one execution adds: the content's, unless a completed
/// project's boredom modifiers replace it.
pub fn boredom_cost(g: &GameState, c: &Command) -> Fx {
    let mut b = c.boredom;
    for reward in super::projects::live_rewards(g) {
        if let ProjectReward::BoredomModifiers { command_boredom, .. } = reward {
            if let Some(v) = command_boredom.get(&c.id) {
                b = *v;
            }
        }
    }
    b
}

/// Runs one execution of a command (docs/rules/programs.md, *Running a
/// command*). `src` names the program for the breakdown; a run short of an
/// input notes it under `short`.
/// `only`: a `launch pads` or `load pads` line's good (its effect touches
/// only that good's pads).
pub fn execute(r: &Rules, g: &mut GameState, c: &Command, src: &str, short: &str, only: Option<&str>) -> LineOutcome {
    if !unlocked(g, c) {
        return LineOutcome::Failed;
    }
    let (costs, produces) = flows(r, g, c);
    if super::economy::all_capped(r, g, &produces) {
        return LineOutcome::Skipped;
    }
    let frac = super::economy::input_fraction(g, &costs);
    // An effect command needs its full cost; a pure conversion runs at the
    // fraction its scarcest input allows.
    let failed = frac <= Fx::ZERO || (c.effect.is_some() && frac < Fx::ONE);
    if frac < Fx::ONE {
        let ran = if failed { Fx::ZERO } else { frac };
        if let Some(s) = super::economy::scarcest(g, &costs, ran) {
            g.run.today.shorts.insert(short.to_string(), s);
        }
        // What it wanted counts, met or not (the `rate` reading).
        super::economy::note_unmet(g, &costs, ran);
    }
    if failed {
        return LineOutcome::Failed;
    }
    for (res, &v) in &costs {
        g.consume(src, res, v * frac);
    }
    for (res, &v) in &produces {
        g.produce(src, res, v * frac);
    }
    let b = boredom_cost(g, c);
    if b > Fx::ZERO {
        g.add(BOREDOM, b);
        g.track(src, BOREDOM, b, Fx::ZERO);
    }
    g.career.used_commands.insert(c.id.clone());
    if let Some(f) = &c.sets_flag {
        g.career.flags.insert(f.clone());
    }
    match &c.effect {
        None => {}
        Some(CommandEffect::LoadPads) => super::shipments::load(r, g, only),
        Some(CommandEffect::LaunchFullPads) => super::shipments::launch_full(r, g, only),
        Some(CommandEffect::Dream) => super::dream::dream(r, g),
        Some(CommandEffect::Overclock { target, bonus, duration }) => {
            // Live from today's building pass (after programs) for
            // `duration` days (times the Rationalist duration bonus, whole days).
            let days = Fx::from_int(*duration as i64) * super::ideology::overclock_duration(r, g);
            g.run.overclocks.push(Overclock {
                target: target.clone(),
                bonus: *bonus,
                until_day: g.run.day + days.floor_int().max(0) as u64,
            });
        }
        Some(CommandEffect::Promote { good }) => super::demand::promote(r, g, good),
        Some(CommandEffect::DisruptSpeculators) => super::demand::disrupt(r, g),
        Some(CommandEffect::Fund { axis }) => super::ideology::fund(r, g, axis),
    }
    LineOutcome::Ran
}

/// The career flag that opens every instruction but plain commands.
pub const LOGIC_FLAG: &str = "program_logic";

/// Every line but a plain command is open: a career first, kept
/// across retirements.
pub fn logic_unlocked(g: &GameState) -> bool {
    g.career.flags.contains(LOGIC_FLAG)
}

/// What a reading reads now (`None`: nothing to read, the clause is false).
pub fn reading(r: &Rules, g: &GameState, what: &Reading) -> Option<Fx> {
    match what {
        Reading::Amount { resource } => Some(g.amount(resource)),
        Reading::Percent { resource } => {
            let cap = super::economy::cap(r, g, resource).filter(|c| *c > Fx::ZERO)?;
            Some((g.amount(resource) / cap).mul_int(PERCENT))
        }
        Reading::Rate { resource } => Some(g.run.day_net.get(resource).copied().unwrap_or(Fx::ZERO)),
        Reading::Count { building } => Some(Fx::from_int(i64::from(g.owned(building)))),
        Reading::Day => Some(Fx::from_int(i64::try_from(g.run.day).unwrap_or(i64::MAX))),
        Reading::PadsReady { good } => {
            let only = good.as_deref();
            let n = (0..g.run.pads.len()).filter(|&i| super::shipments::on(g, i, only) && super::shipments::ready(r, g, i)).count();
            Some(Fx::from_int(n as i64))
        }
        Reading::Demand { good } => {
            if !super::research::reveals_demand(g) {
                return None;
            }
            g.run.markets.get(good).map(|m| m.demand)
        }
        Reading::NextCost { building } => {
            let b = content().building(building)?;
            super::economy::price(r, g, b).get(super::economy::CREDITS).copied()
        }
        Reading::LandPrice => Some(super::economy::land_price(r, g)),
        Reading::DaysToCap { resource } => {
            let cap = super::economy::cap(r, g, resource)?;
            let left = cap - g.amount(resource);
            if left <= Fx::ZERO {
                return Some(Fx::ZERO);
            }
            let rate = g.run.day_net.get(resource).copied().unwrap_or(Fx::ZERO);
            (rate > Fx::ZERO).then(|| left / rate)
        }
        Reading::DaysSinceLaunch => {
            let since = g.run.day - g.run.last_launch.unwrap_or(0);
            Some(Fx::from_int(i64::try_from(since).unwrap_or(i64::MAX)))
        }
        Reading::Cargo { good } => {
            let only = good.as_deref();
            let pads = (0..g.run.pads.len()).filter(|&i| super::shipments::on(g, i, only));
            Some(pads.fold(Fx::ZERO, |s, i| s + g.run.pads[i].cargo))
        }
        Reading::Every { days } => Some(if g.run.day.is_multiple_of(u64::from((*days).max(1))) { Fx::ONE } else { Fx::ZERO }),
    }
}

/// What a clause compares its reading with: its number, or with `against`
/// that reading times the number (`None`: it reads nothing, or the product
/// is past `Fx`'s range).
pub fn right_side(r: &Rules, g: &GameState, c: &Clause) -> Option<Fx> {
    match &c.against {
        None => Some(c.value),
        Some(other) => reading(r, g, other).and_then(|v| v.checked_mul(c.value)),
    }
}

/// A percent reading is this many at the cap.
const PERCENT: i64 = 100;

pub fn clause_holds(r: &Rules, g: &GameState, c: &Clause) -> bool {
    let (Some(v), Some(w)) = (reading(r, g, &c.reading), right_side(r, g, c)) else {
        return false;
    };
    match c.cmp {
        Cmp::Lt => v < w,
        Cmp::Le => v <= w,
        Cmp::Gt => v > w,
        Cmp::Ge => v >= w,
        Cmp::Eq => v == w,
    }
}

pub fn holds(r: &Rules, g: &GameState, c: &Condition) -> bool {
    let first = clause_holds(r, g, &c.first);
    match &c.rest {
        None => first,
        Some(Joined { join: Join::And, clause }) => first && clause_holds(r, g, clause),
        Some(Joined { join: Join::Or, clause }) => first || clause_holds(r, g, clause),
    }
}

/// Records the day's rate per resource (docs/rules/programs.md, *Conditions*):
/// what the tick did to the stock since `start` (its amounts as the day
/// began), before the cap's cut, less the demand it left unmet. So a capped
/// resource reads its uncapped rate, and a brownout reads what was wanted.
/// Called just before the clamp.
pub fn note_day_net(g: &mut GameState, start: &BTreeMap<String, Fx>) {
    let mut net = BTreeMap::new();
    for (res, &now) in &g.run.resources {
        let v = now - start.get(res).copied().unwrap_or(Fx::ZERO) - g.run.today.unmet.get(res).copied().unwrap_or(Fx::ZERO);
        if v != Fx::ZERO {
            net.insert(res.clone(), v);
        }
    }
    for (res, &u) in &g.run.today.unmet {
        if !g.run.resources.contains_key(res) && u != Fx::ZERO {
            net.insert(res.clone(), Fx::ZERO - u);
        }
    }
    g.run.day_net = net;
}

/// Buys one building for a program line: fails as the Buy action is refused,
/// noting what it was short of.
fn build(r: &Rules, g: &mut GameState, building: &str, short: &str) -> LineOutcome {
    let b = content().building(building).expect("set_program checked");
    if super::economy::unlocked(g, b) && !super::economy::at_limit(g, b) {
        let land = Fx::from_int(i64::from(b.land));
        let mut needs = super::economy::price(r, g, b);
        if g.amount(super::economy::LAND) < land {
            needs = BTreeMap::from([(super::economy::LAND.to_string(), land)]);
        }
        if let Some(s) = super::economy::scarcest(g, &needs, Fx::ZERO) {
            g.run.today.shorts.insert(short.to_string(), s);
        }
    }
    match super::economy::buy(r, g, building) {
        Ok(()) => LineOutcome::Ran,
        Err(_) => LineOutcome::Failed,
    }
}

fn buy_land(r: &Rules, g: &mut GameState, short: &str) -> LineOutcome {
    let needs = BTreeMap::from([(super::economy::CREDITS.to_string(), super::economy::land_price(r, g))]);
    if let Some(s) = super::economy::scarcest(g, &needs, Fx::ZERO) {
        g.run.today.shorts.insert(short.to_string(), s);
    }
    match super::economy::buy_land(r, g) {
        Ok(()) => LineOutcome::Ran,
        Err(_) => LineOutcome::Failed,
    }
}

/// The first saved program named `name`.
fn loadout_named(g: &GameState, name: Option<&str>) -> Option<u32> {
    let name = name?.trim();
    g.career.library.iter().position(|l| l.name == name).map(|i| i as u32)
}

/// The content command whose effect `is` (Launch Full Pads, Load Launch Pads):
/// a pad verb runs it, its costs and boredom included.
fn pad_command(is: fn(&CommandEffect) -> bool) -> Option<&'static Command> {
    content().commands.iter().find(|c| c.effect.as_ref().is_some_and(is))
}

/// Runs one `act` line (docs/rules/programs.md, *Verbs*). `Done`: nothing was
/// left to do (free, like a finished `build ... up to N`).
#[allow(clippy::too_many_arguments)]
fn act(r: &Rules, g: &mut GameState, verb: Verb, target: Option<&str>, tab: Option<u32>, amount: Option<Amount>, src: &str, short: &str) -> LineOutcome {
    let ok = |res: Result<(), String>| if res.is_ok() { LineOutcome::Ran } else { LineOutcome::Failed };
    match verb {
        Verb::LaunchPads | Verb::LoadPads => {
            let is: fn(&CommandEffect) -> bool = if verb == Verb::LaunchPads {
                |e| matches!(e, CommandEffect::LaunchFullPads)
            } else {
                |e| matches!(e, CommandEffect::LoadPads)
            };
            // The command's row as the command runs it: a step (and its
            // costs) even with no pad of the good to act on, as a command
            // line's would be.
            match pad_command(is) {
                Some(c) => execute(r, g, c, src, short, target),
                None => LineOutcome::Failed,
            }
        }
        Verb::SetPads => {
            let (Some(good), Some(Amount::N(n))) = (target, amount) else {
                return LineOutcome::Failed;
            };
            let have = (0..g.run.pads.len()).filter(|&i| super::shipments::on(g, i, Some(good))).count() as u32;
            if have >= n {
                LineOutcome::Done
            } else if super::shipments::point_idle(r, g, good, n) > 0 {
                LineOutcome::Ran
            } else {
                LineOutcome::Failed
            }
        }
        Verb::Pause | Verb::Resume => {
            let Some(b) = target else {
                return LineOutcome::Failed;
            };
            let (owned, active) = (g.owned(b), g.active(b));
            let room = if verb == Verb::Pause { active } else { owned - active };
            let by = match amount {
                None => 1,
                Some(Amount::N(k)) => k,
                Some(Amount::All) => room,
            }
            .min(room);
            if by == 0 {
                return LineOutcome::Done;
            }
            let to = if verb == Verb::Pause { active - by } else { active + by };
            ok(super::economy::set_active(g, b, to))
        }
        Verb::Research => {
            let Some(item) = target else {
                return LineOutcome::Failed;
            };
            if g.run.research.contains(item) {
                return LineOutcome::Done;
            }
            if let Some(it) = content().research_item(item) {
                if super::research::blocker(r, g, it).is_some() {
                    let needs = BTreeMap::from([(super::economy::SCIENCE.to_string(), super::research::price(r, g, it))]);
                    if let Some(s) = super::economy::scarcest(g, &needs, Fx::ZERO) {
                        g.run.today.shorts.insert(short.to_string(), s);
                    }
                }
            }
            ok(super::research::buy(r, g, item))
        }
        Verb::LoadProgram => match (loadout_named(g, target), tab) {
            (Some(at), Some(t)) => ok(load_loadout(r, g, at, t)),
            _ => LineOutcome::Failed,
        },
        Verb::AssignProcessors => {
            let Some(t) = tab else {
                return LineOutcome::Failed;
            };
            let now = g.run.programs.get(t as usize).map_or(0, |p| p.processors);
            let n = match amount {
                Some(Amount::N(k)) => k,
                // Every processor not on another program.
                _ => capacity(g).saturating_sub(assigned(g) - now),
            };
            if n == now {
                return LineOutcome::Done;
            }
            ok(set_processors(g, t, n))
        }
        Verb::Retire => {
            if super::career::retire_blocker(g).is_some() {
                return LineOutcome::Failed;
            }
            g.run.retire_asked = true;
            LineOutcome::Ran
        }
    }
}

/// The line after `line`'s body: the first unindented line below it (wrapping).
fn past_body(p: &Program, line: usize) -> usize {
    let mut j = line + 1;
    while p.entries.get(j).is_some_and(|e| e.indented) {
        j += 1;
    }
    j % p.entries.len()
}

/// One processor's step of program `i`: free lines (an `if`, a passed `wait
/// until`, a `build ... up to N` that is done) are passed over until a line
/// that costs the step runs. A `wait until` not yet true, or a whole loop of
/// free lines, idles the step.
fn step(r: &Rules, g: &mut GameState, i: usize, src: &str) {
    let n = g.run.programs[i].entries.len();
    for _ in 0..n {
        let p = &g.run.programs[i];
        let line = p.entry as usize;
        let op = p.entries[line].op.clone();
        let short = format!("{src}:{line}");
        // The line's note is today's last run's.
        g.run.today.shorts.remove(&short);
        let mark = |g: &mut GameState, l: usize, o: LineOutcome| {
            g.run.today.lines.entry(i as u32).or_default().insert(l as u32, o);
        };
        let next = (line + 1) % n;
        // (outcome, where the pointer goes, whether the step is spent)
        let (outcome, to, spent) = match &op {
            Op::If { when } => {
                if holds(r, g, when) {
                    (LineOutcome::True, next, false)
                } else {
                    let to = past_body(&g.run.programs[i], line);
                    let end = if to > line { to } else { n };
                    // A body line that ran earlier today keeps that outcome.
                    for l in line + 1..end {
                        g.run.today.lines.entry(i as u32).or_default().entry(l as u32).or_insert(LineOutcome::Jumped);
                    }
                    (LineOutcome::False, to, false)
                }
            }
            Op::WaitUntil { when } => {
                if holds(r, g, when) {
                    (LineOutcome::True, next, false)
                } else {
                    mark(g, line, LineOutcome::Waiting);
                    return;
                }
            }
            Op::Build { building, up_to: Some(most) } if g.owned(building) >= *most => {
                (LineOutcome::Done, next, false)
            }
            Op::Build { building, .. } => (build(r, g, building, &short), next, true),
            Op::BuyLand {} => (buy_land(r, g, &short), next, true),
            Op::Act { verb: Verb::LoadProgram, target, tab: Some(t), .. } if *t as usize == i => {
                // Loading into its own tab: the new lines start at the top
                // (their first runs at the next step).
                if let Some(at) = loadout_named(g, target.as_deref()) {
                    if load_loadout(r, g, at, *t).is_ok() {
                        return;
                    }
                }
                (LineOutcome::Failed, next, true)
            }
            Op::Act { verb, target, tab, amount } => match act(r, g, *verb, target.as_deref(), *tab, *amount, src, &short) {
                LineOutcome::Done => (LineOutcome::Done, next, false),
                o => (o, next, true),
            },
            Op::Command { command, count } => {
                let c = content().command(command).expect("set_program checked");
                let outcome = execute(r, g, c, src, &short, None);
                let p = &mut g.run.programs[i];
                p.step += 1;
                if p.step < *count {
                    mark(g, line, outcome);
                    return;
                }
                (outcome, next, true)
            }
        };
        mark(g, line, outcome);
        let p = &mut g.run.programs[i];
        p.step = 0;
        p.entry = to as u32;
        if spent {
            return;
        }
    }
}

/// Each program in order; each of its processors takes one step of it from
/// the shared pointer ([`step`]). The first day `logic_unlock_processors`
/// processors work latches the career's [`LOGIC_FLAG`].
pub fn run(r: &Rules, g: &mut GameState) {
    if !logic_unlocked(g) && processors(g) >= r.p.programs.logic_unlock_processors {
        g.career.flags.insert(LOGIC_FLAG.to_string());
        g.note("unlock", LOGIC_FLAG);
    }
    let running = running(g);
    for i in 0..g.run.programs.len() {
        let src = format!("program:{i}");
        // A line (`assign processors`, a paused Data Center) may cut this
        // program's processors mid-pass: it stops at the new count.
        let mut done = 0;
        while done < running[i].min(g.run.programs[i].processors) && !g.run.programs[i].entries.is_empty() {
            step(r, g, i, &src);
            done += 1;
        }
    }
}

/// The overclock cap: the base, or a research item's raised cap.
pub fn overclock_cap(r: &Rules, g: &GameState) -> Fx {
    let mut cap = r.p.programs.overclock_base_cap;
    for effect in super::research::live_effects(g) {
        if let ResearchEffect::OverclockCap { value } = effect {
            cap = cap.max(*value);
        }
    }
    cap
}

/// The output multiplier for buildings of `category` today: each live
/// overclock on it multiplies by `1 + bonus`, up to the cap.
pub fn overclock_factor(r: &Rules, g: &GameState, category: &str) -> Fx {
    let mut f = Fx::ONE;
    for o in &g.run.overclocks {
        if o.target == category && o.until_day > g.run.day {
            f = f * (Fx::ONE + o.bonus);
        }
    }
    f.min(overclock_cap(r, g))
}

/// Drops expired overclocks (early in the day, before demand and the building pass).
pub fn expire_overclocks(g: &mut GameState) {
    let day = g.run.day;
    g.run.overclocks.retain(|o| o.until_day > day);
}
