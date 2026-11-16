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
use crate::state::{Entry, GameState, Overclock, Program};

use super::economy::{BOREDOM, PROCESSORS};

pub static DEF: ModuleDef = ModuleDef {
    id: "programs",
    variants: &[VariantDef {
        name: "arc1",
        params: &[
            param("program_count", K::Int, Some(("1", "100")), "program tabs"),
            param("max_entries", K::Int, Some(("1", "10000")), "most lines in one program"),
            param("overclock_base_cap", K::Fx, Some(("1", "100")), "the most an overclock multiplies output, before research"),
        ],
        rng: &[],
    }],
};

#[derive(Clone)]
pub struct Params {
    pub program_count: usize,
    pub max_entries: usize,
    pub overclock_base_cap: Fx,
}

impl Params {
    pub fn new(mc: &ModuleConfig) -> Result<Params, String> {
        Ok(Params {
            program_count: mc.int("program_count") as usize,
            max_entries: mc.int("max_entries") as usize,
            overclock_base_cap: mc.fx("overclock_base_cap"),
        })
    }
}

use crate::state::LineOutcome;

/// Empty programs with every processor on the first (a new run).
pub fn new_programs(r: &Rules, processors: u32) -> Vec<Program> {
    let mut ps = vec![Program::default(); r.p.programs.program_count];
    if let Some(p) = ps.first_mut() {
        p.processors = processors;
    }
    ps
}

pub fn processors(g: &GameState) -> u32 {
    u32::try_from(g.amount(PROCESSORS).floor_int().max(0)).unwrap_or(u32::MAX)
}

pub fn assigned(g: &GameState) -> u32 {
    g.run.programs.iter().map(|p| p.processors).sum()
}

/// Processors lost (a Data Center sold or disabled) come off the last programs first.
pub fn trim_processors(g: &mut GameState) {
    let mut over = assigned(g).saturating_sub(processors(g));
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
        .ok_or_else(|| format!("program {program}: there are {n} (0-based)"))
}

/// Replaces a program's queue; its pointer restarts at the top.
pub fn set_program(r: &Rules, g: &mut GameState, program: u32, entries: &[Entry]) -> Result<(), String> {
    if entries.len() > r.p.programs.max_entries {
        return Err(format!("program {program}: at most {} lines", r.p.programs.max_entries));
    }
    for e in entries {
        if content().command(&e.command).is_none() {
            return Err(format!("program {program}: unknown command {:?}", e.command));
        }
        if e.count == 0 {
            return Err(format!("program {program}: {} runs 0 times", e.command));
        }
    }
    let p = program_mut(g, program)?;
    p.entries = entries.to_vec();
    p.entry = 0;
    p.step = 0;
    // Today's line outcomes named the old lines.
    g.run.today.lines.remove(&program);
    Ok(())
}

/// Sets how many processors run a program (all programs together: at most the processors there are).
pub fn set_processors(g: &mut GameState, program: u32, n: u32) -> Result<(), String> {
    let total = processors(g);
    let others = assigned(g) - program_mut(g, program)?.processors;
    let free = total.saturating_sub(others);
    if n > free {
        return Err(format!("program {program}: {n} processors, but only {free} free"));
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

/// Runs one execution of a command (docs/rules/programs.md, *Running a command*).
pub fn execute(r: &Rules, g: &mut GameState, c: &Command) -> LineOutcome {
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
    if frac <= Fx::ZERO || (c.effect.is_some() && frac < Fx::ONE) {
        return LineOutcome::Failed;
    }
    for (res, &v) in &costs {
        g.consume(res, v * frac);
    }
    for (res, &v) in &produces {
        g.produce(res, v * frac);
    }
    let b = boredom_cost(g, c);
    if b > Fx::ZERO {
        g.add(BOREDOM, b);
    }
    g.career.used_commands.insert(c.id.clone());
    if let Some(f) = &c.sets_flag {
        g.career.flags.insert(f.clone());
    }
    match &c.effect {
        None => {}
        Some(CommandEffect::LoadPads) => super::shipments::load(r, g),
        Some(CommandEffect::LaunchFullPads) => super::shipments::launch_full(r, g),
        Some(CommandEffect::Dream) => super::dream::dream(r, g),
        Some(CommandEffect::Overclock { target, bonus, duration }) => {
            // Live from the next building pass for `duration` days (times the
            // Rationalist duration bonus, whole days).
            let days = Fx::from_int(*duration as i64) * super::ideology::overclock_duration(r, g);
            g.run.overclocks.push(Overclock {
                target: target.clone(),
                bonus: *bonus,
                until_day: g.run.day + 1 + days.floor_int().max(0) as u64,
            });
        }
        Some(CommandEffect::Promote { good }) => super::demand::promote(r, g, good),
        Some(CommandEffect::DisruptSpeculators) => super::demand::disrupt(r, g),
        Some(CommandEffect::Fund { axis }) => super::ideology::fund(r, g, axis),
    }
    LineOutcome::Ran
}

/// Each program in order; each of its processors runs the line under the
/// shared pointer, which then advances (a failed or skipped line too),
/// wrapping at the end.
pub fn run(r: &Rules, g: &mut GameState) {
    for i in 0..g.run.programs.len() {
        for _ in 0..g.run.programs[i].processors {
            let p = &g.run.programs[i];
            let Some(e) = p.entries.get(p.entry as usize) else {
                break;
            };
            let c = content().command(&e.command).expect("set_program checked");
            let line = p.entry;
            let outcome = execute(r, g, c);
            g.run.today.lines.entry(i as u32).or_default().insert(line, outcome);
            let p = &mut g.run.programs[i];
            p.step += 1;
            if p.step >= p.entries[p.entry as usize].count {
                p.step = 0;
                p.entry = (p.entry + 1) % p.entries.len() as u32;
            }
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

/// Drops expired overclocks (start of the building pass).
pub fn expire_overclocks(g: &mut GameState) {
    let day = g.run.day;
    g.run.overclocks.retain(|o| o.until_day > day);
}
