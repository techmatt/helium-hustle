//! `hh-sim objectives`: plays one of the Godot optimizer's scenario files
//! (`reference/scenarios/*.json`) with the scripted policy, limited to the
//! scenario's available buildings and commands, and reports each objective's
//! tick against its target window, the quests and the run's end. Reading the
//! state to score objectives is observation, not rules.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use hh_core::driver::{play_allowed, Allowed, Policy};
use hh_core::{Config, Fx, Session};
use serde::Deserialize;

/// The Godot data's short ids that the port renamed (docs/rules/README.md, *Content*).
const RENAMED: &[(&str, &str)] = &[
    ("panel", "solar_panel"),
    ("eng", "energy"),
    ("reg", "regolith"),
    ("ti", "titanium"),
    ("cir", "circuits"),
    ("prop", "propellant"),
    ("cred", "credits"),
    ("sci", "science"),
];

/// Commands the port added that the opening can't do without, allowed beside
/// a scenario's own list: Buy Science is the only science before a Research
/// Lab, and the Blueprints that unlock the base's buildings cost science (074).
const ADDED: &[&str] = &["buy_science"];

fn port_id(id: &str) -> String {
    RENAMED.iter().find(|(old, _)| *old == id).map_or(id, |(_, new)| new).to_string()
}

#[derive(Deserialize)]
struct Available {
    buildings: Vec<String>,
    commands: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Objective {
    pub id: String,
    #[serde(rename = "type")]
    kind: String,
    value: String,
    count: Option<u32>,
    threshold: Option<f64>,
    pub target: [u64; 2],
}

/// A scenario file's objectives, for scoring run 1 of any career (the
/// experiment harness: a spec's `objectives`).
pub fn load(path: &str, text: &str) -> Result<Vec<Objective>, String> {
    let sc: OptimizerScenario = serde_json::from_str(text).map_err(|e| format!("{path}: {e}"))?;
    Ok(sc.objectives)
}

#[derive(Deserialize)]
struct EndCondition {
    #[serde(rename = "type")]
    kind: String,
    threshold: f64,
}

#[derive(Deserialize)]
struct OptimizerScenario {
    name: String,
    available_actions: Available,
    objectives: Vec<Objective>,
    end_condition: EndCondition,
}

/// A threshold from the scenario file (a JSON number) as an `Fx`.
fn fx(v: f64) -> Result<Fx, String> {
    Fx::parse(&format!("{v}")).map_err(|e| format!("threshold {v}: {e}"))
}

/// Whether an objective holds now.
pub fn holds(o: &Objective, s: &Session) -> Result<bool, String> {
    let g = &s.state;
    Ok(match o.kind.as_str() {
        "building" => g.owned(&port_id(&o.value)) >= o.count.unwrap_or(1),
        "resource" => g.amount(&port_id(&o.value)) >= fx(o.threshold.unwrap_or(0.0))?,
        "event" if o.value == "shipment_complete" => g.run.stats.shipments >= 1,
        "event" if o.value.starts_with("boredom_") => {
            let n: i64 = o.value["boredom_".len()..].parse().map_err(|_| format!("objective {}: bad value", o.id))?;
            g.amount("boredom") >= Fx::from_int(n)
        }
        _ => return Err(format!("objective {}: unknown type {} {}", o.id, o.kind, o.value)),
    })
}

/// Plays the scenario on `config` from `seed` until the first run's forced
/// retirement (or `max_ticks`); returns the report.
pub fn run(path: &str, text: &str, config: &Config, seed: u64, max_ticks: u64) -> Result<String, String> {
    let sc: OptimizerScenario = serde_json::from_str(text).map_err(|e| format!("{path}: {e}"))?;
    if sc.end_condition.kind != "boredom" {
        return Err(format!("{path}: end condition {:?} unsupported", sc.end_condition.kind));
    }
    let allowed = Allowed {
        buildings: Some(sc.available_actions.buildings.iter().map(|b| port_id(b)).collect()),
        commands: Some(
            sc.available_actions.commands.iter().map(|c| port_id(c)).chain(ADDED.iter().map(|c| c.to_string())).collect(),
        ),
    };
    let mut s = Session::new(config, seed, None)?;
    let end_at = fx(sc.end_condition.threshold)?;
    let mut hit: BTreeMap<String, u64> = BTreeMap::new();
    let mut end_tick = None;
    while s.state.career.retirements == 0 && s.state.tick < max_ticks {
        play_allowed(&mut s, 1, Policy::Script, &allowed);
        if s.state.career.retirements > 0 {
            break;
        }
        for o in &sc.objectives {
            if !hit.contains_key(&o.id) && holds(o, &s)? {
                hit.insert(o.id.clone(), s.state.tick);
            }
        }
        if end_tick.is_none() && s.state.amount("boredom") >= end_at {
            end_tick = Some(s.state.tick);
        }
    }
    let mut t = String::new();
    let _ = writeln!(t, "{} (seed {seed}), scripted policy, {} actions", sc.name, s.record().actions.len());
    let _ = writeln!(t, "objectives (tick hit, target window):");
    for o in &sc.objectives {
        let [lo, hi] = o.target;
        let line = match hit.get(&o.id) {
            Some(&k) => {
                let mark = if (lo..=hi).contains(&k) { "in" } else if k < lo { "early" } else { "late" };
                format!("{k:>6}  [{lo}, {hi}] {mark}")
            }
            None => format!("{:>6}  [{lo}, {hi}] not hit", "-"),
        };
        let _ = writeln!(t, "  {:<14}{line}", o.id);
    }
    let end = end_tick.map_or("not reached".to_string(), |k| format!("tick {k}"));
    let _ = writeln!(t, "end condition (boredom >= {}): {end}", sc.end_condition.threshold);
    t.push_str(&career_log(&s.state));
    Ok(t)
}

/// The career log: quests, achievements, projects, research, retirements, with ticks.
/// (Not the M1-M14 milestones: those are `milestones.rs`.)
pub fn career_log(g: &hh_core::GameState) -> String {
    let mut t = String::from("career log (tick, run, day):\n");
    for e in &g.career.log {
        let what = if e.id.is_empty() { e.kind.clone() } else { format!("{} {}", e.kind, e.id) };
        let _ = writeln!(t, "  {:>6}  run {} day {:>5}  {what}", e.tick, e.run, e.day);
    }
    t
}
