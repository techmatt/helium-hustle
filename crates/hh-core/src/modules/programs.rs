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
            param("fund_verb", K::Int, Some(("0", "1")), "1: the `fund <project>` verb is on; 0: off (a line with it fails when run)"),
            param("line_text_max", K::Int, Some(("1", "10000")), "most characters in a note or unparsed line's text"),
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
    pub fund_verb: bool,
    pub line_text_max: usize,
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
            fund_verb: mc.int("fund_verb") == 1,
            line_text_max: mc.int("line_text_max") as usize,
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

/// Processors the active buildings grant that no program holds (the view's
/// `processors_free`, the `free processors` reading): what `assign processors
/// <tab> all` would add.
pub fn free(g: &GameState) -> u32 {
    capacity(g).saturating_sub(assigned(g))
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

/// Replaces a program's queue; its pointer restarts at the top. What this run
/// hasn't unlocked yet is accepted: such a line fails, or its `if` reads
/// false, when run ([`locked`] says why).
pub fn set_program(r: &Rules, g: &mut GameState, program: u32, entries: &[Entry]) -> Result<(), String> {
    check_entries(r, g, program, entries)?;
    put_entries(g, program, entries)
}

/// Whether `entries` may go in a program: on their form only, never on what
/// this run has unlocked.
fn check_entries(r: &Rules, g: &GameState, program: u32, entries: &[Entry]) -> Result<(), String> {
    let n = program + 1;
    if entries.len() > r.p.programs.max_entries {
        return Err(format!("Program {n}: at most {} lines", r.p.programs.max_entries));
    }
    if program as usize >= g.run.programs.len() {
        return Err(format!("Program {n}: there are {}", g.run.programs.len()));
    }
    // Whether the line above is an `if` or in a body: only then may a line be indented.
    let mut in_block = false;
    for (i, e) in entries.iter().enumerate() {
        check_line(r, g, e, in_block).map_err(|why| format!("Program {n}, line {}: {why}", i + 1))?;
        in_block = e.indented || matches!(e.op, Op::If { .. });
    }
    Ok(())
}

/// Whether one line may stand where it is (`in_block`: the line above is an
/// `if` or in a body), on its form: `Err` says why not, in the player's words.
pub fn check_line(r: &Rules, g: &GameState, e: &Entry, in_block: bool) -> Result<(), String> {
    let c = content();
    // One reading per thing: a capped resource (and boredom) reads as a fill
    // percent or an amount (the editor's unit toggle), an uncapped one as an
    // amount; land and processors have their own `free` readings.
    let one = |reading: &Reading| -> Result<(), String> {
        let (resource, percent) = match reading {
            Reading::Amount { resource } => (resource, false),
            Reading::Percent { resource } => (resource, true),
            Reading::FreeProcessors | Reading::FreeLand | Reading::IdlePads | Reading::EnergyPerDay | Reading::DaysSincePurchase => {
                return Ok(())
            }
        };
        let Some(res) = c.resource(resource) else {
            return Err(format!("unknown resource {resource:?}"));
        };
        let capped = res.cap.is_some() || resource == BOREDOM;
        let own = resource == super::economy::LAND || resource == PROCESSORS;
        match (percent, capped) {
            (true, false) => Err(format!("{} has no cap to read a percent of", res.name)),
            _ if own => Err(format!("{resource} reads as free {resource}")),
            _ => Ok(()),
        }
    };
    if e.indented && !in_block {
        return Err("an indented line needs an if above it".to_string());
    }
    match &e.op {
        Op::Command { command, count } => {
            if c.command(command).is_none() {
                return Err(format!("unknown command {command:?}"));
            }
            if *count == 0 {
                return Err(format!("{command} runs 0 times"));
            }
        }
        Op::Build { building, up_to } => {
            if c.building(building).is_none() {
                return Err(format!("unknown building {building:?}"));
            }
            if *up_to == Some(0) {
                return Err(format!("build {building} up to 0"));
            }
        }
        Op::BuyLand {} => {}
        Op::If { when } => {
            if e.indented {
                return Err("an if can't sit inside another if".to_string());
            }
            one(&when.first.reading)?;
            if let Some(j) = &when.rest {
                one(&j.clause.reading)?;
            }
        }
        Op::Act { verb, target, tab, amount } => check_act(r, g, *verb, target.as_deref(), *tab, *amount)?,
        Op::Note { text } | Op::Unparsed { text } => {
            if text.contains('\n') {
                return Err("a line's text is one line".to_string());
            }
            let max = r.p.programs.line_text_max;
            if text.chars().count() > max {
                return Err(format!("at most {max} characters"));
            }
        }
    }
    Ok(())
}

/// Why a line won't run yet this run, if it won't: what it needs isn't
/// unlocked (program logic, `if`, a researched reading, the fund verb, a
/// command, building, research item or project). Such a line is kept; run,
/// it fails, or its `if` reads false.
pub fn locked(r: &Rules, g: &GameState, e: &Entry) -> Option<String> {
    let c = content();
    match &e.op {
        Op::Note { .. } | Op::Unparsed { .. } => return None,
        Op::Command { command, .. } => {
            return c.command(command).filter(|x| !unlocked(g, x)).map(|x| format!("{} isn't unlocked this run", x.name));
        }
        _ if !logic_unlocked(g) => {
            return Some(format!("program logic opens at {} working processors", r.p.programs.logic_unlock_processors));
        }
        _ => {}
    }
    match &e.op {
        Op::If { when } => {
            if !if_unlocked(g) {
                return Some("if isn't open yet (Conditional Logic)".to_string());
            }
            let researched = when.first.reading.researched() || when.rest.as_ref().is_some_and(|j| j.clause.reading.researched());
            if researched && !super::research::opens_program_readings(g) {
                return Some("its reading needs research first".to_string());
            }
            None
        }
        Op::Build { building, .. } | Op::Act { verb: Verb::Pause | Verb::Resume, target: Some(building), .. } => c
            .building(building)
            .filter(|b| !super::economy::unlocked(g, b))
            .map(|b| format!("{} isn't unlocked this run", b.name)),
        Op::Act { verb: Verb::Fund, target, .. } => {
            if !r.p.programs.fund_verb {
                return Some("the fund verb is off".to_string());
            }
            let p = target.as_deref().and_then(|t| c.project(t))?;
            (!super::projects::unlocked(r, g, p)).then(|| format!("{} isn't open to funding", p.name))
        }
        Op::Act { verb: v @ (Verb::LaunchPads | Verb::LoadPads), .. } => {
            pad_command(*v).filter(|x| !unlocked(g, x)).map(|x| format!("{} isn't unlocked this run", x.name))
        }
        Op::Act { verb: Verb::Research, target: Some(item), .. } => c
            .research_item(item)
            .filter(|it| !super::research::visible(g, it))
            .map(|it| format!("{} isn't shown yet", it.name)),
        _ => None,
    }
}

/// What each verb's slots take (docs/rules/programs.md, *Verbs*): a slot the
/// verb has no use for must be empty, and an amount is never 0.
fn check_act(r: &Rules, g: &GameState, verb: Verb, target: Option<&str>, tab: Option<u32>, amount: Option<Amount>) -> Result<(), String> {
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
        Verb::Fund => (target.is_some_and(|p| c.project(p).is_some()), false, Takes::Optional),
    };
    let what = format!("{verb:?}");
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
        (_, Some(Amount::N(0))) if verb != Verb::AssignProcessors && verb != Verb::Fund => Err(format!("{what}: an amount of 0")),
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
    check_entries(r, g, program, &entries)?;
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
    super::timeline::loadout_deleted(g, loadout);
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
    // The Humanist major effect: Dream costs only its energy.
    if matches!(c.effect, Some(CommandEffect::Dream)) && super::ideology::major(r, g, "humanist") {
        costs.retain(|res, _| res == super::economy::ENERGY);
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

/// Runs one execution of a command (docs/rules/commands.md, *Running a
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
    // An effect command needs its full cost, and so does one that relieves
    // boredom (Distract, draft); a pure conversion runs at the fraction its
    // scarcest input allows.
    let relieves = boredom_cost(g, c) < Fx::ZERO;
    let failed = frac <= Fx::ZERO || ((c.effect.is_some() || relieves) && frac < Fx::ONE);
    if frac < Fx::ONE {
        let ran = if failed { Fx::ZERO } else { frac };
        if let Some(s) = super::economy::scarcest(g, &costs, ran) {
            g.run.today.shorts.insert(short.to_string(), s);
        }
        // What it wanted counts, met or not (the day's rate, `Run.day_net`).
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
    } else if b < Fx::ZERO {
        // A negative flat boredom relieves it (Distract, draft), never below zero.
        let relief = (-b).min(g.amount(BOREDOM));
        g.add(BOREDOM, -relief);
        g.track(src, BOREDOM, Fx::ZERO, relief);
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
        Some(CommandEffect::Neutralize { adversary }) => super::attention::neutralize(g, adversary),
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

/// The career flag that opens `if` (a global project's reward).
pub const IF_FLAG: &str = "logic_if";

/// `if` lines are open: program logic and the `if` flag.
pub fn if_unlocked(g: &GameState) -> bool {
    logic_unlocked(g) && g.career.flags.contains(IF_FLAG)
}

/// What a reading reads now (`None`: nothing to read, the clause is false).
pub fn reading(r: &Rules, g: &GameState, what: &Reading) -> Option<Fx> {
    match what {
        // A capped resource's amount is at most its cap, as the panel shows it
        // (below: programs run before the day's clamp).
        Reading::Amount { resource } => {
            let held = g.amount(resource);
            Some(super::economy::cap(r, g, resource).map_or(held, |cap| held.min(cap)))
        }
        Reading::Percent { resource } => {
            // At most 100: programs run before the day's clamp, so a stock
            // may stand past its cap then; the fill reads as the panel shows it.
            let cap = super::economy::cap(r, g, resource).filter(|c| *c > Fx::ZERO)?;
            Some((g.amount(resource).min(cap) / cap).mul_int(PERCENT))
        }
        Reading::FreeProcessors => Some(Fx::from_int(i64::from(free(g)))),
        Reading::FreeLand => Some(g.amount(super::economy::LAND)),
        Reading::IdlePads => {
            let n = (0..g.run.pads.len()).filter(|&i| super::shipments::idle(g, i)).count();
            Some(Fx::from_int(n as i64))
        }
        // Researched readings read nothing (the clause is false) until their
        // research is done this run.
        Reading::EnergyPerDay | Reading::DaysSincePurchase if !super::research::opens_program_readings(g) => None,
        Reading::EnergyPerDay => Some(g.run.day_net.get(super::economy::ENERGY).copied().unwrap_or(Fx::ZERO)),
        Reading::DaysSincePurchase => {
            let since = g.run.day.saturating_sub(g.run.last_purchase.unwrap_or(0));
            Some(Fx::from_int(since as i64))
        }
    }
}

/// A percent reading is this many at the cap.
const PERCENT: i64 = 100;

pub fn clause_holds(r: &Rules, g: &GameState, c: &Clause) -> bool {
    let Some(v) = reading(r, g, &c.reading) else {
        return false;
    };
    match c.cmp {
        Cmp::Lt => v < c.value,
        Cmp::Gt => v > c.value,
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
/// The command a pad verb runs: Launch Full Pads or Load Launch Pads.
fn pad_command(verb: Verb) -> Option<&'static Command> {
    let is: fn(&CommandEffect) -> bool = if verb == Verb::LaunchPads {
        |e| matches!(e, CommandEffect::LaunchFullPads)
    } else {
        |e| matches!(e, CommandEffect::LoadPads)
    };
    content().commands.iter().find(|c| c.effect.as_ref().is_some_and(is))
}

/// Runs one `act` line (docs/rules/programs.md, *Verbs*). `Done`: nothing was
/// left to do (the step only, like a finished `build ... up to N`).
#[allow(clippy::too_many_arguments)]
fn act(r: &Rules, g: &mut GameState, verb: Verb, target: Option<&str>, tab: Option<u32>, amount: Option<Amount>, src: &str, short: &str) -> LineOutcome {
    let ok = |res: Result<(), String>| if res.is_ok() { LineOutcome::Ran } else { LineOutcome::Failed };
    match verb {
        Verb::LaunchPads | Verb::LoadPads => {
            // Locked, it fails; with nothing to load or launch, done: the
            // step only, no costs or boredom. Else the command's row as the
            // command runs it.
            let Some(c) = pad_command(verb).filter(|c| unlocked(g, c)) else {
                return LineOutcome::Failed;
            };
            let busy = if verb == Verb::LaunchPads {
                super::shipments::would_launch(r, g, target)
            } else {
                super::shipments::would_load(r, g, target)
            };
            if !busy {
                return LineOutcome::Done;
            }
            execute(r, g, c, src, short, target)
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
        Verb::Fund => {
            let Some(p) = target.and_then(|id| content().project(id)) else {
                return LineOutcome::Failed;
            };
            if !r.p.programs.fund_verb || !super::projects::unlocked(r, g, p) {
                return LineOutcome::Failed;
            }
            // The Projects panel's rate, on every cost: N a day (at most the
            // resource's most a day), `all` or none the most.
            let rate = |res: &str| {
                let max = super::projects::max_rate(g, p, res);
                match amount {
                    Some(Amount::N(k)) => Fx::from_int(i64::from(k)).min(max),
                    _ => max,
                }
            };
            let want: Vec<(String, Fx)> = p.costs.keys().map(|res| (res.clone(), rate(res))).collect();
            let now = g.run.project_rates.get(&p.id);
            if want.iter().all(|(res, x)| now.and_then(|m| m.get(res)).copied().unwrap_or(Fx::ZERO) == *x) {
                return LineOutcome::Done;
            }
            for (res, rate) in want {
                if super::projects::set_rate(r, g, &p.id, &res, rate).is_err() {
                    return LineOutcome::Failed;
                }
            }
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

/// One processor's step of program `i`: free lines (`if`s) are passed over
/// until a line that costs the step runs, done or not. A whole loop of free
/// lines idles the step.
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
            // Free, never run, no outcome: passed over like a true `if`.
            Op::Note { .. } | Op::Unparsed { .. } => {
                let p = &mut g.run.programs[i];
                p.step = 0;
                p.entry = next as u32;
                continue;
            }
            // Before program logic opens, every line but a command fails
            // (a held or loaded one; programs.md, *Locked lines*).
            Op::Build { .. } | Op::BuyLand {} | Op::Act { .. } if !logic_unlocked(g) => (LineOutcome::Failed, next, true),
            Op::If { when } => {
                // A loaded `if` this career hasn't opened is false.
                if if_unlocked(g) && holds(r, g, when) {
                    (LineOutcome::True, next, false)
                } else {
                    let to = past_body(&g.run.programs[i], line);
                    let end = if to > line { to } else { n };
                    // A body line that ran earlier today keeps that outcome.
                    for l in line + 1..end {
                        // Note and unparsed lines never have an outcome.
                        if g.run.programs[i].entries[l].op.is_text() {
                            continue;
                        }
                        g.run.today.lines.entry(i as u32).or_default().entry(l as u32).or_insert(LineOutcome::Jumped);
                    }
                    (LineOutcome::False, to, false)
                }
            }
            // Done: nothing to buy, the step only.
            Op::Build { building, up_to: Some(most) } if g.owned(building) >= *most => (LineOutcome::Done, next, true),
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
            Op::Act { verb, target, tab, amount } => (act(r, g, *verb, target.as_deref(), *tab, *amount, src, &short), next, true),
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
