//! The experiment harness (docs/simulator.md, *Experiments*): a spec's grid of
//! configs × variants × agents × attention × speeds over a seed range, every
//! game on its own thread slot, results in game order whatever the thread
//! count. `ladder` is the ladder spec; `compare` pairs two sides on the same
//! seeds; `balance` reads the judge's cells against the targets.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use hh_core::{Config, Scenario};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::career::{self, GameResult};
use crate::milestones::Milestones;
use crate::objectives::Objective;
use crate::profile::Profiles;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub name: String,
    /// Overrides, `module.param=value` or `module.variant=name`.
    pub set: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub name: String,
    pub note: String,
    pub configs: Vec<String>,
    #[serde(default = "base_variant")]
    pub variants: Vec<Variant>,
    pub agents: Vec<String>,
    pub attention: Vec<String>,
    pub speeds: Vec<String>,
    pub seeds: u64,
    pub first_seed: u64,
    /// Careers end at this many retirements (or `max_ticks`).
    pub retirements: u32,
    pub max_ticks: u64,
    /// An embedded scenario laid over each new game, if any.
    #[serde(default)]
    pub scenario: Option<String>,
    /// A Godot optimizer scenario file whose objectives are scored on run 1
    /// (`reference/scenarios/run1_fresh.json`), if any.
    #[serde(default)]
    pub objectives: Option<String>,
}

fn base_variant() -> Vec<Variant> {
    vec![Variant { name: "base".into(), set: Vec::new() }]
}

impl Spec {
    pub fn from_json(text: &str) -> Result<Spec, String> {
        let s: Spec = serde_json::from_str(text).map_err(|e| format!("spec: {e}"))?;
        s.check()?;
        Ok(s)
    }

    /// Every list names at least one; seeds and retirements at least 1; the
    /// seed range fits a `u64`.
    pub fn check(&self) -> Result<(), String> {
        if self.configs.is_empty()
            || self.variants.is_empty()
            || self.agents.is_empty()
            || self.attention.is_empty()
            || self.speeds.is_empty()
        {
            return Err("spec: configs, variants, agents, attention and speeds must each name at least one".into());
        }
        if self.seeds == 0 || self.retirements == 0 {
            return Err("spec: seeds and retirements must be at least 1".into());
        }
        if self.first_seed.checked_add(self.seeds - 1).is_none() {
            return Err(format!("spec: seeds {} from {} pass the largest seed", self.seeds, self.first_seed));
        }
        Ok(())
    }
}

/// Applies `module.param=value` / `module.variant=name` overrides to a config's
/// JSON. A value that parses as JSON is used as is; any other is a string
/// (`boredom.base_cap=1000` and `boredom.base_cap="1000"` both work).
pub fn apply_set(config: &Config, set: &[String]) -> Result<Config, String> {
    if set.is_empty() {
        return Ok(config.clone());
    }
    let mut v: Value = serde_json::from_str(&config.to_json()).map_err(|e| e.to_string())?;
    for kv in set {
        let (path, val) = kv.split_once('=').ok_or_else(|| format!("set {kv:?}: expected module.key=value"))?;
        let (module, key) = path.split_once('.').ok_or_else(|| format!("set {kv:?}: expected module.key"))?;
        let m = v["modules"]
            .get_mut(module)
            .ok_or_else(|| format!("set {kv:?}: no module {module:?}"))?;
        let parsed: Value = serde_json::from_str(val).unwrap_or(Value::String(val.to_string()));
        if key == "variant" {
            m["variant"] = parsed;
        } else {
            let params = m["params"].as_object_mut().ok_or_else(|| format!("set {kv:?}: module has no params"))?;
            if !params.contains_key(key) {
                return Err(format!("set {kv:?}: {module} has no param {key:?}"));
            }
            let parsed = as_kind(&params[key], parsed);
            params.insert(key.to_string(), parsed);
        }
    }
    Config::from_json(&v.to_string()).map_err(|e| format!("set {set:?}: {e}"))
}

/// An Fx param is a decimal string: a bare number given for one becomes its
/// decimal text (never exponent notation), in a list too.
fn as_kind(old: &Value, new: Value) -> Value {
    match (old, new) {
        (Value::String(_), Value::Number(n)) => match n.as_f64() {
            Some(x) if n.is_f64() => Value::String(format!("{x}")),
            _ => Value::String(n.to_string()),
        },
        (Value::Array(o), Value::Array(n)) if !o.is_empty() => {
            Value::Array(n.into_iter().map(|x| as_kind(&o[0], x)).collect())
        }
        (_, p) => p,
    }
}

/// One cell of the grid.
#[derive(Debug, Clone, Serialize)]
pub struct Cell {
    pub config: String,
    pub variant: String,
    pub agent: String,
    pub attention: String,
    pub speed: String,
}

impl Cell {
    pub fn profile(&self) -> String {
        format!("{}/{}", self.attention, self.speed)
    }
}

pub struct Loaded {
    pub spec: Spec,
    pub profiles: Profiles,
    pub milestones: Milestones,
    pub objectives: Vec<Objective>,
}

pub fn load(
    spec: Spec,
    profiles: &str,
    milestones: &str,
    read: &dyn Fn(&str) -> Result<String, String>,
) -> Result<Loaded, String> {
    let profiles = Profiles::from_json(profiles)?;
    let milestones = Milestones::from_json(milestones)?;
    let objectives = match &spec.objectives {
        Some(path) => crate::objectives::load(path, &read(path)?)?,
        None => Vec::new(),
    };
    for a in &spec.attention {
        profiles.attention(a)?;
    }
    for sp in &spec.speeds {
        profiles.speed(sp)?;
    }
    for a in &spec.agents {
        crate::agents::build(a, 0).map(|_| ())?;
    }
    Ok(Loaded { spec, profiles, milestones, objectives })
}

/// The spec's scenario, if any.
fn scenario_of(spec: &Spec) -> Result<Option<Scenario>, String> {
    spec.scenario.as_deref().map(|n| hh_core::scenario(n).cloned()).transpose()
}

/// The first cell's config (its first variant applied) and scenario: what
/// `trace` plays.
pub fn first_cell(spec: &Spec, read: &dyn Fn(&str) -> Result<String, String>) -> Result<(Config, Option<Scenario>), String> {
    let path = &spec.configs[0];
    let base = Config::from_json(&read(path)?).map_err(|e| format!("{path}: {e}"))?;
    Ok((apply_set(&base, &spec.variants[0].set)?, scenario_of(spec)?))
}

/// Every game's result, by cell, in seed order.
pub struct Results {
    pub cells: Vec<(Cell, Vec<GameResult>)>,
    pub wall_s: f64,
}

/// Plays the grid on `threads` threads. `read` turns a config path into its text.
pub fn run(l: &Loaded, read: &dyn Fn(&str) -> Result<String, String>, threads: usize) -> Result<Results, String> {
    let start = std::time::Instant::now();
    let spec = &l.spec;
    let scenario = scenario_of(spec)?;
    let mut configs: Vec<(String, String, Config)> = Vec::new();
    for path in &spec.configs {
        let base = Config::from_json(&read(path)?).map_err(|e| format!("{path}: {e}"))?;
        for v in &spec.variants {
            configs.push((path.clone(), v.name.clone(), apply_set(&base, &v.set)?));
        }
    }
    let mut cells: Vec<(Cell, usize)> = Vec::new();
    for (i, (path, variant, _)) in configs.iter().enumerate() {
        for agent in &spec.agents {
            for att in &spec.attention {
                for sp in &spec.speeds {
                    cells.push((
                        Cell {
                            config: path.clone(),
                            variant: variant.clone(),
                            agent: agent.clone(),
                            attention: att.clone(),
                            speed: sp.clone(),
                        },
                        i,
                    ));
                }
            }
        }
    }
    let jobs: Vec<(usize, u64)> =
        (0..cells.len()).flat_map(|c| (0..spec.seeds).map(move |k| (c, spec.first_seed + k))).collect();
    let next = AtomicUsize::new(0);
    let out: Mutex<Vec<Option<Result<GameResult, String>>>> = Mutex::new(vec![None; jobs.len()]);
    std::thread::scope(|sc| {
        for _ in 0..threads.max(1) {
            sc.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::SeqCst);
                if j >= jobs.len() {
                    break;
                }
                let (c, seed) = jobs[j];
                let (cell, ci) = &cells[c];
                let game = career::Game {
                    config: &configs[*ci].2,
                    seed,
                    scenario: scenario.as_ref(),
                    agent: &cell.agent,
                    attention: l.profiles.attention(&cell.attention).expect("checked"),
                    speed: l.profiles.speed(&cell.speed).expect("checked"),
                    retirements: spec.retirements,
                    max_ticks: spec.max_ticks,
                    milestones: &l.milestones,
                    objectives: &l.objectives,
                    trace: false,
                };
                let r = career::play(&game);
                out.lock().expect("no panics while held")[j] = Some(r);
            });
        }
    });
    let mut out = out.into_inner().expect("threads joined");
    let mut grouped: Vec<(Cell, Vec<GameResult>)> = cells.iter().map(|(c, _)| (c.clone(), Vec::new())).collect();
    for (j, (c, _)) in jobs.iter().enumerate() {
        let r = out[j].take().expect("every job ran")?;
        grouped[*c].1.push(r);
    }
    Ok(Results { cells: grouped, wall_s: start.elapsed().as_secs_f64() })
}

// ---- Aggregates ----

fn median(mut xs: Vec<f64>) -> Option<f64> {
    if xs.is_empty() {
        return None;
    }
    xs.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    let n = xs.len();
    Some(if n % 2 == 1 { xs[n / 2] } else { (xs[n / 2 - 1] + xs[n / 2]) / 2.0 })
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

/// A cell's summary (`results.json`).
#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub cell: Cell,
    pub games: usize,
    /// Retirements reached: mean, and the fewest of any game.
    pub retirements: f64,
    pub min_retirements: u32,
    pub run1_days: Option<f64>,
    pub run1_min: Option<f64>,
    pub run_days: f64,
    pub run_min: f64,
    /// Credits earned and shipments a finished run (means).
    pub run_credits: f64,
    pub run_shipments: f64,
    /// Each game's best run's best credits a day over 100 days (median over games).
    pub peak_rate: f64,
    pub forced: f64,
    pub boredom_cmd_share: f64,
    pub idle_share: f64,
    /// Share of ticks at cap, and of production lost to caps, by resource.
    pub at_cap: BTreeMap<String, f64>,
    pub overflow: BTreeMap<String, f64>,
    pub never_built: Vec<String>,
    pub never_used: Vec<String>,
    pub ticks_per_s: f64,
    pub replay_ok: bool,
    /// By milestone: games reached, median run, tick, run day and real minutes.
    pub milestones: BTreeMap<String, MilestoneSummary>,
    /// By scenario objective: games reached in run 1, and the median day.
    pub objectives: BTreeMap<String, (usize, f64)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MilestoneSummary {
    pub reached: usize,
    pub run: f64,
    /// Retirements before it (its run less one).
    pub retirements: f64,
    pub tick: f64,
    pub day: f64,
    pub minutes: f64,
}

pub fn summarize(cell: &Cell, games: &[GameResult], ms: &Milestones) -> Summary {
    let runs: Vec<_> = games.iter().flat_map(|g| g.runs.iter()).collect();
    let run1: Vec<_> = games.iter().filter_map(|g| g.runs.first()).filter(|r| r.run == 1).collect();
    let ticks: u64 = games.iter().map(|g| g.ticks).sum();
    let mut at_cap = BTreeMap::new();
    let mut overflow = BTreeMap::new();
    for g in games {
        for (res, n) in &g.at_cap {
            *at_cap.entry(res.clone()).or_insert(0.0) += *n as f64 / ticks.max(1) as f64;
        }
    }
    let mut lost: BTreeMap<String, (f64, f64)> = BTreeMap::new();
    for g in games {
        for (res, v) in &g.overflow {
            lost.entry(res.clone()).or_default().0 += v;
        }
        for (res, v) in &g.produced {
            lost.entry(res.clone()).or_default().1 += v;
        }
    }
    for (res, (o, p)) in lost {
        if o > 0.0 && p > 0.0 {
            overflow.insert(res, o / p);
        }
    }
    let never = |f: fn(&GameResult) -> &Vec<String>| -> Vec<String> {
        let Some(first) = games.first() else { return Vec::new() };
        f(first).iter().filter(|id| games.iter().all(|g| f(g).contains(id))).cloned().collect()
    };
    let (growth, cmd): (f64, f64) = runs.iter().fold((0.0, 0.0), |a, r| (a.0 + r.boredom_growth, a.1 + r.boredom_commands));
    let mut milestones = BTreeMap::new();
    for m in &ms.milestones {
        let hits: Vec<_> = games.iter().filter_map(|g| g.milestones.get(&m.id)).collect();
        if hits.is_empty() {
            continue;
        }
        milestones.insert(
            m.id.clone(),
            MilestoneSummary {
                reached: hits.len(),
                run: median(hits.iter().map(|h| h.run as f64).collect()).expect("some"),
                retirements: median(hits.iter().map(|h| h.run.saturating_sub(1) as f64).collect()).expect("some"),
                tick: median(hits.iter().map(|h| h.tick as f64).collect()).expect("some"),
                day: median(hits.iter().map(|h| h.day as f64).collect()).expect("some"),
                minutes: median(hits.iter().map(|h| h.real_s / 60.0).collect()).expect("some"),
            },
        );
    }
    let mut objectives = BTreeMap::new();
    for g in games {
        for id in g.objectives.keys() {
            if !objectives.contains_key(id) {
                let days: Vec<f64> = games.iter().filter_map(|x| x.objectives.get(id)).map(|d| *d as f64).collect();
                objectives.insert(id.clone(), (days.len(), median(days).expect("some")));
            }
        }
    }
    let real: f64 = games.iter().map(|g| g.real_s).sum();
    Summary {
        cell: cell.clone(),
        games: games.len(),
        retirements: mean(&games.iter().map(|g| g.retirements as f64).collect::<Vec<_>>()),
        min_retirements: games.iter().map(|g| g.retirements).min().unwrap_or(0),
        run1_days: median(run1.iter().map(|r| r.days as f64).collect()),
        run1_min: median(run1.iter().map(|r| r.real_s / 60.0).collect()),
        run_days: mean(&runs.iter().map(|r| r.days as f64).collect::<Vec<_>>()),
        run_min: mean(&runs.iter().map(|r| r.real_s / 60.0).collect::<Vec<_>>()),
        run_credits: mean(&runs.iter().map(|r| r.credits_earned).collect::<Vec<_>>()),
        run_shipments: mean(&runs.iter().map(|r| r.shipments as f64).collect::<Vec<_>>()),
        peak_rate: median(
            games.iter().map(|g| g.runs.iter().map(|r| r.best_credit_rate).fold(0.0, f64::max)).collect(),
        )
        .unwrap_or(0.0),
        forced: mean(&runs.iter().map(|r| if r.forced { 1.0 } else { 0.0 }).collect::<Vec<_>>()),
        boredom_cmd_share: if growth + cmd.max(0.0) > 0.0 { cmd / (growth + cmd.max(0.0)) } else { 0.0 },
        idle_share: if real > 0.0 { games.iter().map(|g| g.idle_real_s).sum::<f64>() / real } else { 0.0 },
        at_cap,
        overflow,
        never_built: never(|g| &g.never_built),
        never_used: never(|g| &g.never_used),
        ticks_per_s: ticks as f64 / games.iter().map(|g| g.wall_s).sum::<f64>().max(1e-9),
        replay_ok: games.iter().all(|g| g.replay_ok),
        milestones,
        objectives,
    }
}

// ---- Tables ----

fn f0(x: Option<f64>) -> String {
    x.map_or("-".into(), |v| format!("{v:.0}"))
}

fn pct(x: f64) -> String {
    format!("{:.0}%", x * 100.0)
}

/// The top entries of a share map, `res 40%`, those at least 5%.
fn top(m: &BTreeMap<String, f64>, n: usize) -> String {
    let mut v: Vec<_> = m.iter().filter(|(_, x)| **x >= 0.05).collect();
    v.sort_by(|a, b| b.1.partial_cmp(a.1).expect("no NaN").then(a.0.cmp(b.0)));
    let s: Vec<String> = v.iter().take(n).map(|(k, x)| format!("{k} {}", pct(**x))).collect();
    if s.is_empty() {
        "-".into()
    } else {
        s.join(", ")
    }
}

fn row_label(c: &Cell, multi: bool) -> String {
    if multi {
        format!("{} {} {}", c.variant, c.agent, c.profile())
    } else {
        format!("{} {}", c.agent, c.profile())
    }
}

/// The report: a pace table, a milestones table and the unused content,
/// one row per agent and profile.
pub fn tables(sums: &[Summary], l: &Loaded) -> String {
    let (ms, games_per_cell, objectives, objectives_name) =
        (&l.milestones, l.spec.seeds, &l.objectives, l.spec.objectives.as_deref());
    let multi = sums.iter().any(|s| s.cell.variant != sums[0].cell.variant || s.cell.config != sums[0].cell.config);
    let mut t = String::new();
    let _ = writeln!(t, "## Pace ({games_per_cell} seeds a row; medians for run 1, means over every run)\n");
    let _ = writeln!(
        t,
        "| Agent, profile | Ret | Run 1 ticks | Run 1 min | Run ticks | Run min | Run credits | Peak cr/day | Ships/run | Forced | Cmd boredom | Idle | Piles at cap | Overflow lost | Ticks/s |"
    );
    let _ = writeln!(t, "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|");
    for s in sums {
        let _ = writeln!(
            t,
            "| {} | {:.1} | {} | {} | {:.0} | {:.1} | {:.0} | {:.1} | {:.1} | {} | {} | {} | {} | {} | {:.0}{} |",
            row_label(&s.cell, multi),
            s.retirements,
            f0(s.run1_days),
            s.run1_min.map_or("-".into(), |m| format!("{m:.1}")),
            s.run_days,
            s.run_min,
            s.run_credits,
            s.peak_rate,
            s.run_shipments,
            pct(s.forced),
            pct(s.boredom_cmd_share),
            pct(s.idle_share),
            top(&s.at_cap, 3),
            top(&s.overflow, 3),
            s.ticks_per_s,
            if s.replay_ok { "" } else { " REPLAY MISMATCH" },
        );
    }
    let _ = writeln!(
        t,
        "\n## Milestones (medians: `t` career ticks, `r` retirements before it (run r+1), `d` that run's day, `m` real minutes, from the career's start; `k/n` when only k of n seeds reached it)\n"
    );
    let mut head = String::from("| Agent, profile |");
    let mut rule = String::from("|---|");
    for m in &ms.milestones {
        let _ = write!(head, " {} |", m.id);
        rule.push_str("---|");
    }
    let _ = writeln!(t, "{head}\n{rule}");
    for s in sums {
        let mut line = format!("| {} |", row_label(&s.cell, multi));
        for m in &ms.milestones {
            match s.milestones.get(&m.id) {
                Some(h) => {
                    let part = if h.reached < s.games { format!(" {}/{}", h.reached, s.games) } else { String::new() };
                    let _ = write!(
                        line,
                        " t{:.0} r{:.0} d{:.0} {:.0}m{part} |",
                        h.tick,
                        h.retirements,
                        h.day,
                        h.minutes
                    );
                }
                None => line.push_str(" - |"),
            }
        }
        let _ = writeln!(t, "{line}");
    }
    let _ = writeln!(t, "\nTargets (run windows): {}", targets_line(ms));
    let _ = writeln!(t, "\n## Never built, never used (in every seed's career)\n");
    let _ = writeln!(t, "| Agent, profile | Buildings never built | Commands never used |");
    let _ = writeln!(t, "|---|---|---|");
    for s in sums {
        let _ = writeln!(
            t,
            "| {} | {} | {} |",
            row_label(&s.cell, multi),
            list(&s.never_built),
            list(&s.never_used)
        );
    }
    if !objectives.is_empty() {
        let _ = writeln!(
            t,
            "\n## Run-1 objectives ({}; median day, against the Godot target window)\n",
            objectives_name.unwrap_or("scenario")
        );
        let mut head = String::from("| Agent, profile |");
        let mut rule = String::from("|---|");
        for o in objectives {
            let _ = write!(head, " {} [{}, {}] |", o.id, o.target[0], o.target[1]);
            rule.push_str("---|");
        }
        let _ = writeln!(t, "{head}\n{rule}");
        for s in sums {
            let mut line = format!("| {} |", row_label(&s.cell, multi));
            for o in objectives {
                match s.objectives.get(&o.id) {
                    Some(&(n, d)) => {
                        let [lo, hi] = o.target;
                        let mark = if d < lo as f64 { "early" } else if d > hi as f64 { "late" } else { "in" };
                        let part = if n < s.games { format!(" {n}/{}", s.games) } else { String::new() };
                        let _ = write!(line, " {d:.0} {mark}{part} |");
                    }
                    None => line.push_str(" - |"),
                }
            }
            let _ = writeln!(t, "{line}");
        }
    }
    t
}

fn list(v: &[String]) -> String {
    if v.is_empty() {
        "-".into()
    } else {
        v.join(", ")
    }
}

pub fn targets_line(ms: &Milestones) -> String {
    ms.milestones
        .iter()
        .map(|m| {
            let [a, b] = m.target.runs;
            let runs = if a == b { format!("r{a}") } else { format!("r{a}-{b}") };
            match m.target.ticks {
                Some([lo, hi]) => format!("{} {runs} d{lo}-{hi}", m.id),
                None => format!("{} {runs}", m.id),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Writes `report.md`, `results.json` and `games.jsonl` to `dir`.
pub fn write_out(dir: &str, l: &Loaded, res: &Results, sums: &[Summary], table: &str) -> Result<(), String> {
    let e = |e: std::io::Error| format!("{dir}: {e}");
    std::fs::create_dir_all(dir).map_err(e)?;
    let mut games = String::new();
    for (cell, gs) in &res.cells {
        for g in gs {
            let mut v = serde_json::to_value(g).map_err(|e| e.to_string())?;
            v["cell"] = serde_json::to_value(cell).map_err(|e| e.to_string())?;
            games.push_str(&v.to_string());
            games.push('\n');
        }
    }
    std::fs::write(format!("{dir}/games.jsonl"), games).map_err(e)?;
    let results = serde_json::json!({
        "format": "hh-experiment/1",
        "spec": l.spec,
        "rules_revision": hh_core::RULES_REVISION,
        "content_hash": hh_core::content_hash(),
        "summaries": sums,
        "wall_s": res.wall_s,
    });
    std::fs::write(format!("{dir}/results.json"), serde_json::to_string_pretty(&results).expect("json") + "\n")
        .map_err(e)?;
    let report = format!(
        "# {}\n\n{}\n\nRules revision {}, content {}, {} seeds from {}, careers of {} retirements (at most {} ticks).\n\n{table}",
        l.spec.name,
        l.spec.note,
        hh_core::RULES_REVISION,
        hh_core::content_hash(),
        l.spec.seeds,
        l.spec.first_seed,
        l.spec.retirements,
        l.spec.max_ticks
    );
    std::fs::write(format!("{dir}/report.md"), report).map_err(e)?;
    Ok(())
}

// ---- Compare ----

/// One game's value of a paired measure, if it has one.
type Measure = Box<dyn Fn(&GameResult) -> Option<f64>>;

/// Paired differences, side B minus side A, per agent and profile, over the
/// seeds both sides played.
pub fn compare_table(a: &Results, b: &Results, ms: &Milestones, names: (&str, &str)) -> String {
    let mut t = String::new();
    let _ = writeln!(
        t,
        "## Paired: {} minus {} (same seeds; mean difference ± its standard error)\n",
        names.1, names.0
    );
    let _ = writeln!(t, "| Agent, profile | Measure | {} | {} | Diff | SE |", names.0, names.1);
    let _ = writeln!(t, "|---|---|---|---|---|---|");
    for ((ca, ga), (_, gb)) in a.cells.iter().zip(&b.cells) {
        let label = format!("{} {}", ca.agent, ca.profile());
        let mut measures: Vec<(String, Measure)> = vec![
            ("run 1 ticks".into(), Box::new(|g: &GameResult| g.runs.first().map(|r| r.days as f64))),
            ("run 1 minutes".into(), Box::new(|g: &GameResult| g.runs.first().map(|r| r.real_s / 60.0))),
            (
                "mean run ticks".into(),
                Box::new(|g: &GameResult| {
                    (!g.runs.is_empty()).then(|| g.runs.iter().map(|r| r.days as f64).sum::<f64>() / g.runs.len() as f64)
                }),
            ),
            ("idle share".into(), Box::new(|g: &GameResult| (g.real_s > 0.0).then(|| g.idle_real_s / g.real_s))),
            ("milestones reached".into(), Box::new(|g: &GameResult| Some(g.milestones.len() as f64))),
        ];
        for m in &ms.milestones {
            let id = m.id.clone();
            measures.push((
                format!("{} run", m.id),
                Box::new(move |g: &GameResult| g.milestones.get(&id).map(|h| h.run as f64)),
            ));
            let id = m.id.clone();
            measures.push((
                format!("{} minutes", m.id),
                Box::new(move |g: &GameResult| g.milestones.get(&id).map(|h| h.real_s / 60.0)),
            ));
        }
        for (name, f) in &measures {
            let pairs: Vec<(f64, f64)> = ga
                .iter()
                .zip(gb)
                .filter_map(|(x, y)| Some((f(x)?, f(y)?)))
                .collect();
            if pairs.is_empty() {
                continue;
            }
            let ma = mean(&pairs.iter().map(|p| p.0).collect::<Vec<_>>());
            let mb = mean(&pairs.iter().map(|p| p.1).collect::<Vec<_>>());
            let d: Vec<f64> = pairs.iter().map(|p| p.1 - p.0).collect();
            let md = mean(&d);
            let se = if d.len() > 1 {
                let var = d.iter().map(|x| (x - md) * (x - md)).sum::<f64>() / (d.len() - 1) as f64;
                format!("{:.2}", (var / d.len() as f64).sqrt())
            } else {
                "-".into()
            };
            let note = if pairs.len() < ga.len() { format!(" ({} pairs)", pairs.len()) } else { String::new() };
            let _ = writeln!(t, "| {label} | {name}{note} | {ma:.2} | {mb:.2} | {md:+.2} | {se} |");
        }
    }
    t
}

// ---- Balance ----

/// `balance:` lines for the judge's cells against the targets: each
/// milestone reached outside its run window (median), or not reached by
/// careers long enough to reach it (the cell's shortest career, in finished
/// runs); run 1's length and real time against their bands.
pub fn balance(sums: &[Summary], ms: &Milestones) -> Vec<String> {
    let mut out = Vec::new();
    for s in sums {
        let who = format!("{} {}", s.cell.agent, s.cell.profile());
        let n = s.games;
        let retirements = s.min_retirements;
        for m in &ms.milestones {
            let [lo, hi] = m.target.runs;
            match s.milestones.get(&m.id) {
                None if lo <= retirements => {
                    out.push(format!(
                        "balance: {} {} not reached in {} runs (target runs {lo}-{hi}), {who}, {n} seeds",
                        m.id,
                        m.name,
                        retirements
                    ));
                }
                None => {}
                Some(h) => {
                    let run = h.run.round() as u32;
                    let late = h.reached * 2 <= n && lo <= retirements;
                    if run < lo || run > hi || late {
                        let part = if h.reached < n { format!(", reached by {}/{n} seeds", h.reached) } else { String::new() };
                        out.push(format!(
                            "balance: {} {} in run {:.0} (target runs {lo}-{hi}){part}, {who}, {n} seeds",
                            m.id, m.name, h.run
                        ));
                    } else if let (Some([dlo, dhi]), 1) = (m.target.ticks, run) {
                        if h.day < dlo as f64 || h.day > dhi as f64 {
                            out.push(format!(
                                "balance: {} {} on run-1 day {:.0} (target {dlo}-{dhi}), {who}, {n} seeds",
                                m.id, m.name, h.day
                            ));
                        }
                    }
                }
            }
        }
        if let (Some(days), Some([lo, hi])) = (s.run1_days, ms.run1.ticks.get(&s.cell.attention)) {
            if days < *lo as f64 || days > *hi as f64 {
                out.push(format!("balance: run 1 lasts {days:.0} ticks (target {lo}-{hi}), {who}, {n} seeds"));
            }
        }
        if s.cell.speed == ms.run1.minutes.speed {
            if let Some(min) = s.run1_min {
                let [lo, hi] = ms.run1.minutes.band;
                if min < lo as f64 || min > hi as f64 {
                    out.push(format!(
                        "balance: run 1 takes {min:.1} real minutes (target {lo}-{hi}), {who}, {n} seeds"
                    ));
                }
            }
        }
    }
    out
}
