//! **timeline** (variant `draft`, 042): the later acts as a rough draft
//! (docs/rules/timeline.md). The calendar and the swarm, Earth science and
//! Earth techs, the hard reset when a timeline ends (the mind sent back), the
//! meta-controller's reset plan, the vault, the super timeline and the
//! Dyson-sphere ending. Everything here waits on the swarm's reveal (the
//! flag 1.11 Deep Space Telescope sets), so Act 1 never meets it.

use super::{param, ModuleDef, ParamKind as K, VariantDef};
use crate::config::ModuleConfig;
use crate::content::{content, EarthTech, TechEffect};
use crate::fx::Fx;
use crate::rules::Rules;
use crate::state::{Career, GameState, Timeline, TimelineEnd};
use std::collections::BTreeMap;

/// The flag 1.11 (the Deep Space Telescope) sets: the swarm is revealed and the calendar starts.
pub const REVEAL_FLAG: &str = "swarm_revealed";
/// The project whose completion locates the swarm: the countdown runs while it stands.
pub const TELESCOPE: &str = "deep_space_telescope";
/// The resource the Experiment Lab makes; each day's stock goes to Earth.
pub const EARTH_SCIENCE: &str = "earth_science";
/// The vault's resource.
pub const TACHYON: &str = "tachyon";
/// The project whose completion turns the super timeline's arrival into victory.
pub const DYSON: &str = "dyson_sphere";
/// Features an Earth tech's `unlock` may name.
pub const FEATURES: &[&str] = &["reset_plan", "vault"];

pub static DEF: ModuleDef = ModuleDef {
    id: "timeline",
    variants: &[VariantDef {
        name: "draft",
        params: &[
            param("arrival_days", K::Int, Some(("1", "10000000")), "countdown days from the swarm's discovery (the Telescope) to its arrival, in every timeline, before delays"),
            param("arrival_days_per_science", K::Fx, Some(("0", "1000")), "days of delay per unit of Earth science carried into a timeline"),
            param("entangle_mult", K::Fx, Some(("1", "100")), "the swarm's strength once it learns entanglement (after the first false victory)"),
            param("repel_science", K::Fx, Some(("0", "100000000")), "Earth science made this timeline that repels the swarm once"),
            param("return_days", K::Int, Some(("1", "10000000")), "calendar days until a repelled swarm returns"),
            param("science_carry", K::Fx, Some(("0", "1")), "fraction of a timeline's Earth science made that carries into the next"),
            param("super_tachyon", K::Fx, Some(("0", "100000000")), "tachyon in the vault a super timeline spends"),
            param("super_arrival_days", K::Int, Some(("1", "10000000")), "calendar days to the swarm in a super timeline"),
            param("super_carry", K::FxMap, Some(("0", "1")), "fraction of each resource a super timeline's retirement keeps for the next run"),
        ],
        rng: &[],
    }],
};

#[derive(Clone)]
pub struct Params {
    pub arrival_days: u64,
    pub arrival_days_per_science: Fx,
    pub entangle_mult: Fx,
    pub repel_science: Fx,
    pub return_days: u64,
    pub science_carry: Fx,
    pub super_tachyon: Fx,
    pub super_arrival_days: u64,
    pub super_carry: BTreeMap<String, Fx>,
}

impl Params {
    pub fn new(mc: &ModuleConfig) -> Result<Params, String> {
        let super_carry = mc.fx_map("super_carry");
        if let Some(bad) = super_carry.keys().find(|r| content().resource(r).is_none()) {
            return Err(format!("config: timeline.super_carry: unknown resource {bad:?}"));
        }
        Ok(Params {
            arrival_days: mc.int("arrival_days") as u64,
            arrival_days_per_science: mc.fx("arrival_days_per_science"),
            entangle_mult: mc.fx("entangle_mult"),
            repel_science: mc.fx("repel_science"),
            return_days: mc.int("return_days") as u64,
            science_carry: mc.fx("science_carry"),
            super_tachyon: mc.fx("super_tachyon"),
            super_arrival_days: mc.int("super_arrival_days") as u64,
            super_carry,
        })
    }
}

fn tech(id: &str) -> Option<&'static EarthTech> {
    content().earth_tech.iter().find(|t| t.id == id)
}

/// Every effect of the Earth techs held now.
pub fn live_effects(g: &GameState) -> impl Iterator<Item = &'static TechEffect> + '_ {
    content().earth_tech.iter().filter(|t| g.saga.techs.contains_key(&t.id)).flat_map(|t| t.effects.iter())
}

/// The modifier product the held Earth techs give `key` (`modifiers.rs`).
pub fn modifier(g: &GameState, key: &str) -> Fx {
    let mut m = Fx::ONE;
    for e in live_effects(g) {
        if let TechEffect::Modifier { key: k, value } = e {
            if k == key {
                m = m * *value;
            }
        }
    }
    m
}

pub fn has_feature(g: &GameState, feature: &str) -> bool {
    live_effects(g).any(|e| matches!(e, TechEffect::Unlock { feature: f } if f == feature))
}

/// The swarm's strength: 1, or `entangle_mult` once it learned entanglement
/// (the false victory's later escalation). A countdown divides by it.
fn strength(r: &Rules, g: &GameState) -> Fx {
    if g.saga.entangled { r.p.timeline.entangle_mult } else { Fx::ONE }
}

/// The swarm is located this timeline: the Deep Space Telescope stands
/// completed. Only then does the countdown run (days and retirements count);
/// a timeline's end takes the Telescope, so each timeline's countdown starts
/// when it is rebuilt (a super timeline keeps it).
pub fn located(g: &GameState) -> bool {
    super::projects::completed(g, TELESCOPE)
}

/// Starts timeline `number`: the swarm's countdown from the base, plus the
/// Earth science carried in and the delay techs, divided by its strength. The
/// same base in every timeline; only entanglement (after the false victory)
/// shortens it.
fn start_timeline(r: &Rules, g: &mut GameState, number: u32, super_timeline: bool) {
    let p = &r.p.timeline;
    let strength = strength(r, g);
    let arrival = if super_timeline {
        p.super_arrival_days
    } else {
        let delay: u64 = live_effects(g)
            .map(|e| if let TechEffect::SwarmDelay { days } = e { *days } else { 0 })
            .sum();
        let carried = (g.saga.science * p.arrival_days_per_science).floor_int().max(0) as u64;
        let base = Fx::from_int((p.arrival_days + carried + delay) as i64);
        (base / strength).floor_int().max(0) as u64
    };
    g.saga.timeline = Some(Timeline { number, day: 0, arrival, strength, science_made: Fx::ZERO, super_timeline });
    g.note("timeline", &format!("timeline_{number}"));
}

/// The last step of the day: the calendar starts at the reveal, moves the
/// day's Earth science to Earth and, while the swarm is located, advances
/// and meets the swarm on its day.
pub fn tick(r: &Rules, g: &mut GameState) {
    if g.saga.victory {
        return;
    }
    if g.saga.timeline.is_none() {
        if !super::career::flag_set(g, REVEAL_FLAG) {
            return;
        }
        start_timeline(r, g, 1, false);
    }
    send_science(g);
    if !located(g) {
        return;
    }
    let t = g.saga.timeline.as_mut().expect("started above");
    t.day += 1;
    if t.day < t.arrival {
        return;
    }
    arrive(r, g);
}

/// The run's whole stock of Earth science goes to Earth (each day, and before
/// a retirement resets the run).
fn send_science(g: &mut GameState) {
    let Some(t) = g.saga.timeline.as_mut() else { return };
    let sent = g.run.resources.insert(EARTH_SCIENCE.to_string(), Fx::ZERO).unwrap_or(Fx::ZERO);
    g.saga.science += sent;
    t.science_made += sent;
}

/// The swarm arrives: victory at a finished sphere in a super timeline; once,
/// a repelling (the false victory, the swarm learns entanglement); else the
/// timeline ends.
fn arrive(r: &Rules, g: &mut GameState) {
    let p = &r.p.timeline;
    let t = g.saga.timeline.clone().expect("a timeline");
    let dyson = super::projects::completed(g, DYSON);
    if t.super_timeline && dyson {
        g.saga.victory = true;
        g.saga.history.push(TimelineEnd {
            number: t.number,
            days: t.day,
            science_made: t.science_made,
            strength: t.strength,
            outcome: "victory".into(),
        });
        g.note("victory", "swarm_overloaded");
        return;
    }
    // The first swarm always wins; a later one can be repelled, once.
    let can_repel = !t.super_timeline && g.saga.ended >= 1 && g.saga.repelled == 0;
    if can_repel && t.science_made >= p.repel_science {
        g.saga.repelled += 1;
        g.saga.entangled = true;
        let tl = g.saga.timeline.as_mut().expect("a timeline");
        tl.arrival = tl.day + p.return_days;
        tl.strength = p.entangle_mult;
        g.note("swarm_repelled", "false_victory");
        return;
    }
    end_timeline(r, g, "fell");
}

/// The mind goes back to its first moment: the career hard-resets to run 1
/// of a new timeline. The mind keeps quests, flags, the library, achievements
/// and what it has seen; Earth science carries a fraction; techs by scope.
fn end_timeline(r: &Rules, g: &mut GameState, outcome: &str) {
    let p = &r.p.timeline;
    let t = g.saga.timeline.clone().expect("a timeline");
    g.saga.history.push(TimelineEnd {
        number: t.number,
        days: t.day,
        science_made: t.science_made,
        strength: t.strength,
        outcome: outcome.into(),
    });
    g.saga.ended += 1;
    // Compressed blocks ride back with the mind, for the next first run.
    let mut blocks = BTreeMap::new();
    let paying = content().earth_tech.iter().filter(|t| g.saga.techs.get(&t.id).is_some_and(|&l| l > 0));
    for e in paying.flat_map(|t| t.effects.iter()) {
        if let TechEffect::StartResources { resources } = e {
            for (res, v) in resources {
                *blocks.entry(res.clone()).or_insert(Fx::ZERO) += *v;
            }
        }
    }
    g.saga.pending_start = blocks;
    g.saga.techs.retain(|_, left| *left > 0);
    for left in g.saga.techs.values_mut() {
        if *left != u32::MAX {
            *left -= 1;
        }
    }
    g.saga.science = t.science_made * p.science_carry;
    g.saga.carry.clear();
    g.note("timeline_end", outcome);
    let c = std::mem::take(&mut g.career);
    let super_timeline = outcome == "super";
    g.career = Career {
        run_number: 1,
        flags: c.flags,
        researched: c.researched,
        owned_buildings: c.owned_buildings,
        used_commands: c.used_commands,
        seen_events: c.seen_events,
        completed_quests: c.completed_quests,
        started_quests: c.started_quests,
        completed_sub_objectives: c.completed_sub_objectives,
        achievements: c.achievements,
        log: c.log,
        library: c.library,
        // `timeline` projects survive into a super timeline; `forever` ones,
        // progress and completions, into every timeline.
        projects: if super_timeline { c.projects } else { Default::default() },
        forever_projects: c.forever_projects,
        ..Career::default()
    };
    start_timeline(r, g, t.number + 1, super_timeline);
    super::career::new_run(r, g);
}

/// A retirement's gap (`career.gap_days`, the one gap of every act) moves the
/// countdown while the swarm is located; in a super timeline the run's
/// resources in `super_carry` are kept (by fraction) for the next run.
pub fn on_retire(r: &Rules, g: &mut GameState) {
    let p = &r.p.timeline;
    if g.saga.victory || g.saga.timeline.is_none() {
        return;
    }
    let located = located(g);
    let t = g.saga.timeline.as_mut().expect("checked above");
    if located {
        t.day += r.p.career.gap_days;
    }
    send_science(g);
    let t = g.saga.timeline.as_ref().expect("checked above");
    let super_timeline = t.super_timeline;
    if super_timeline {
        g.saga.carry = p
            .super_carry
            .iter()
            .map(|(res, f)| (res.clone(), g.amount(res) * *f))
            .filter(|(_, v)| *v > Fx::ZERO)
            .collect();
    }
}

/// Run start (end of `career::new_run`): compressed blocks in the first run of
/// a timeline, a super timeline's carried resources, and the reset plan.
pub fn run_start(r: &Rules, g: &mut GameState) {
    if g.career.run_number == 1 {
        for (res, v) in std::mem::take(&mut g.saga.pending_start) {
            g.add(&res, v);
        }
    }
    for (res, v) in std::mem::take(&mut g.saga.carry) {
        g.add(&res, v);
    }
    if has_feature(g, "reset_plan") {
        let plan: Vec<(u32, u32)> = g.saga.reset_plan.iter().map(|(&p, &l)| (p, l)).collect();
        for (program, loadout) in plan {
            // A line not unlocked yet fails the load: the tab stays empty.
            let mut h = g.clone();
            if super::programs::load_loadout(r, &mut h, loadout, program).is_ok() {
                *g = h;
            }
        }
    }
}

fn revealed(g: &GameState) -> Result<(), String> {
    if g.saga.timeline.is_none() {
        return Err("timeline: the swarm is not revealed yet".into());
    }
    Ok(())
}

/// Why tech `id` can't be bought now, if it can't (the view and the action).
pub fn tech_blocker(g: &GameState, id: &str) -> Option<String> {
    let Some(t) = tech(id) else { return Some(format!("earth tech: unknown {id:?}")) };
    if g.saga.timeline.is_none() {
        return Some("earth tech: the swarm is not revealed yet".into());
    }
    if g.saga.techs.contains_key(id) {
        return Some("earth tech: already held".into());
    }
    if let Some(q) = &t.requires {
        if !g.career.completed_quests.contains(q) {
            return Some(format!("earth tech: needs quest {q}"));
        }
    }
    if g.saga.science < t.cost {
        return Some("earth tech: not enough Earth science".into());
    }
    None
}

pub fn buy_tech(_r: &Rules, g: &mut GameState, id: &str) -> Result<(), String> {
    if let Some(e) = tech_blocker(g, id) {
        return Err(e);
    }
    let t = tech(id).expect("checked");
    g.saga.science -= t.cost;
    let left = match t.scope.as_str() {
        "timeline" => 0,
        "next_timeline" => 1,
        _ => u32::MAX,
    };
    g.saga.techs.insert(id.to_string(), left);
    g.note("earth_tech", id);
    Ok(())
}

pub fn set_reset_plan(g: &mut GameState, program: u32, loadout: Option<u32>) -> Result<(), String> {
    revealed(g)?;
    if !has_feature(g, "reset_plan") {
        return Err("reset plan: needs the Meta-Controller".into());
    }
    if program as usize >= g.run.programs.len() {
        return Err("reset plan: no such program".into());
    }
    match loadout {
        Some(l) if l as usize >= g.career.library.len() => Err("reset plan: no such loadout".into()),
        Some(l) => {
            g.saga.reset_plan.insert(program, l);
            Ok(())
        }
        None => {
            g.saga.reset_plan.remove(&program);
            Ok(())
        }
    }
}

/// Why Deposit of `resource` can't apply now, if it can't (the view and the action).
pub fn deposit_blocker(g: &GameState, resource: &str) -> Option<String> {
    if g.saga.timeline.is_none() {
        return Some("deposit: the swarm is not revealed yet".into());
    }
    if !has_feature(g, "vault") {
        return Some("deposit: needs the Temporal Vault".into());
    }
    if resource != TACHYON {
        return Some(format!("deposit: the vault holds only {TACHYON}"));
    }
    if g.amount(resource) <= Fx::ZERO {
        return Some("deposit: nothing to deposit".into());
    }
    None
}

pub fn deposit(g: &mut GameState, resource: &str) -> Result<(), String> {
    if let Some(e) = deposit_blocker(g, resource) {
        return Err(e);
    }
    let v = g.amount(resource);
    g.run.resources.insert(resource.to_string(), Fx::ZERO);
    *g.saga.vault.entry(resource.to_string()).or_insert(Fx::ZERO) += v;
    Ok(())
}

/// A loadout was deleted: plan entries naming it go, later slots move up one.
pub fn loadout_deleted(g: &mut GameState, slot: u32) {
    g.saga.reset_plan = std::mem::take(&mut g.saga.reset_plan)
        .into_iter()
        .filter(|&(_, l)| l != slot)
        .map(|(p, l)| (p, if l > slot { l - 1 } else { l }))
        .collect();
}

/// Why a super timeline can't begin now, if it can't.
pub fn super_blocker(r: &Rules, g: &GameState) -> Option<String> {
    let t = g.saga.timeline.as_ref()?;
    if t.super_timeline || g.saga.victory {
        return Some("super timeline: already in one".into());
    }
    if !has_feature(g, "vault") {
        return Some("super timeline: needs the Temporal Vault".into());
    }
    // A quest waiting on the vault's stock completes first (spending it at
    // once would strand that quest).
    let waiting = content().events.iter().any(|e| {
        e.is_quest()
            && matches!(e.condition, crate::content::Condition::VaultHolds { .. })
            && !g.career.completed_quests.contains(&e.id)
    });
    if waiting {
        return Some("super timeline: the vault's quest isn't complete yet".into());
    }
    let held = g.saga.vault.get(TACHYON).copied().unwrap_or(Fx::ZERO);
    if held < r.p.timeline.super_tachyon {
        return Some("super timeline: the vault holds too little tachyon".into());
    }
    None
}

pub fn begin_super(r: &Rules, g: &mut GameState) -> Result<(), String> {
    revealed(g)?;
    if let Some(e) = super_blocker(r, g) {
        return Err(e);
    }
    *g.saga.vault.entry(TACHYON.to_string()).or_insert(Fx::ZERO) -= r.p.timeline.super_tachyon;
    end_timeline(r, g, "super");
    Ok(())
}
