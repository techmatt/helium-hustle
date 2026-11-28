//! `hh-sim`, the native command line (docs/simulator.md). It reads files and
//! prints; every rule is `hh-core`'s.

use hh_core::{
    content_hash, play, scenario, scenarios, Config, Policy, Record, Scenario, Session,
    RULES_REVISION, VERSION,
};
use std::fmt::Write as _;

pub mod agents;
pub mod autopilot;
pub mod axes;
pub mod career;
pub mod experiment;
pub mod gates;
pub mod human;
pub mod lab;
pub mod milestones;
pub mod objectives;
pub mod params;
pub mod pool;
pub mod playlog;
pub mod playrecord;
pub mod summary;
pub mod vs;

// The unit tests' binary (main.rs and tests/cli.rs set their own): see main.rs.
#[cfg(test)]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub const USAGE: &str = "usage:
  hh-sim --help                                    this usage (also printed, as an error, with no command)
  hh-sim run [--config F] [--seed S] [--ticks N] [--policy P] [--scenario NAME | --scenario-file F] [--save F] [--json]
  hh-sim replay [--summary] RECORD.json...   --summary: each end state as `run` prints it
  hh-sim show [--config F]
  hh-sim scenarios
  hh-sim gates                                     what each quest opens (unlocks, requires, research shown, projects, Earth techs)
  hh-sim golden
  hh-sim params [--config F] [--write]             each module's param table from the registry (--write: into docs/rules/)
  hh-sim objectives SCENARIO.json [--config F] [--seed S] [--ticks N]
  hh-sim ladder [SPEC.json] [GRID]                 the agent ladder in game time (default experiments/ladder.json)
  hh-sim experiment SPEC.json [GRID]               any spec
  hh-sim compare [--config A] [--config-b B] [--set module.key=value]... [GRID]
  hh-sim balance [GRID]                            balance: lines for the judge's missed targets
  hh-sim trace [GRID] [--lines | --at T]           one career's actions and milestones (--lines: each day's program lines run;
                                                   --at T: the state at career tick T instead, what walls a run in)
  hh-sim summarize GAMES.jsonl [--milestones F]    a read's games: each run, the milestones, the career at the end
  hh-sim agents                                    the ladder's rungs
  hh-sim axes [--seeds N] [--first-seed S] [--skills skilled,medium] [--axes a,b] [--runs N] [--max-ticks N]
              [--config F] [--set module.key=value]... [--threads T]
                                                   the committed ideology players: run and day each first reaches rank 5
  hh-sim speed                                     native engine speed (ticks/s)
  hh-sim playlog LOG.jsonl [--json] [--milestones F] [--human-models F] [--today]   replay and measure a human's play log (--today: best-effort on today's rules)
  hh-sim vs-human LOG.jsonl [--agents a,b] [--humans a,b] [--ticks N] [--json] [--threads T] [--milestones F] [--human-models F] [--include-accelerated] [--skilled: the log's record beside the skilled player alone, no replay]
                                                   the sim players on the log's game, beside the human
  hh-sim accelerate [--seed S] [--config F] [--agent SPEC] [--boost P,R,B | --no-boost] [--target milestone|quest|end]
                    [--budget TICKS] [--save RECORD]   debug: a fresh career on the autopilot (docs/simulator.md, Accelerators)
GRID: [SPEC.json] [--agents a,b] [--humans a,b] [--seeds N] [--first-seed S]
      [--retirements N] [--max-ticks N] [--config F] [--set module.key=value]... [--threads T]
      [--out DIR (ladder, experiment, compare)] [--human-models F] [--milestones F]
players: [name=]agent[:knob=value,...]; --humans adds real time projected by those human models
defaults: --config configs/baseline.json --seed 0 --ticks 1000 (objectives: 5000) --policy idle
policies: idle, greedy (buy whatever is affordable every tick), greedy:every=N, script";

/// A failure: bad input (exit 2) or a failed check (exit 1).
#[derive(Debug)]
pub enum Fail {
    Usage(String),
    Check(String),
}

/// Default files, relative to the repo root (the CLI runs from there).
pub(crate) const BASELINE: &str = "configs/baseline.json";
pub(crate) const MILESTONES: &str = "configs/sim/milestones.json";
pub(crate) const HUMANS: &str = "configs/sim/humans.json";
pub(crate) const LADDER: &str = "experiments/ladder.json";
pub(crate) const BALANCE: &str = "experiments/balance.json";

pub(crate) fn usage<T>(e: impl Into<String>) -> Result<T, Fail> {
    Err(Fail::Usage(e.into()))
}

/// The default `--threads` of the commands that run careers in parallel.
pub(crate) const THREADS: usize = 6;

/// A file's text, the error a plain string (the form `experiment::load` takes).
pub(crate) fn read_text(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}

pub(crate) fn read(path: &str) -> Result<String, Fail> {
    read_text(path).map_err(Fail::Usage)
}

/// A comma-separated list, empty pieces dropped.
pub(crate) fn list(s: &str) -> Vec<String> {
    s.split(',').filter(|x| !x.is_empty()).map(String::from).collect()
}

/// The milestones file at `path` (errors name it).
pub(crate) fn load_milestones(path: &str) -> Result<milestones::Milestones, Fail> {
    milestones::Milestones::from_json(&read(path)?).map_err(|e| Fail::Usage(format!("{path}: {e}")))
}

/// The human time models file at `path` (errors name it).
pub(crate) fn load_humans(path: &str) -> Result<human::Humans, Fail> {
    human::Humans::from_json(&read(path)?).map_err(|e| Fail::Usage(format!("{path}: {e}")))
}

/// The play log at `path` (errors name it).
pub(crate) fn load_log(path: &str) -> Result<playlog::Log, Fail> {
    let text = read(path)?;
    // Tolerant (Matt, 2026-11-27): a config key the log lacks takes today's default.
    // (This build's own baseline, embedded: the defaults are the rules it plays.)
    let today: Option<serde_json::Value> = serde_json::from_str(include_str!("../../../configs/baseline.json")).ok();
    playlog::Log::parse_with(&text, today.as_ref()).map_err(|e| {
        // A log from another rules revision says so, without the usage text (134: every log
        // Matt had, a revision or more old, failed on a missing config key under 40 lines of usage).
        let logged = text
            .lines()
            .next()
            .and_then(|l| serde_json::from_str::<serde_json::Value>(l).ok())
            .and_then(|h| h.get("rules_revision").and_then(serde_json::Value::as_u64));
        match logged {
            Some(n) if n != u64::from(hh_core::RULES_REVISION) => Fail::Check(format!(
                "{path}: {e}\nThe log is rules revision {n}, this build {}: replay it on a build of that revision (scripts/baseline.py, docs/simulator.md).",
                hh_core::RULES_REVISION
            )),
            _ => Fail::Usage(format!("{path}: {e}")),
        }
    })
}

/// The config at `path` (errors name it).
pub(crate) fn load_config(path: &str) -> Result<Config, Fail> {
    Config::from_json(&read(path)?).map_err(|e| Fail::Usage(format!("{path}: {e}")))
}

struct Opts {
    config: String,
    seed: u64,
    ticks: u64,
    ticks_given: bool,
    policy: Policy,
    scenario: Option<Scenario>,
    save: Option<String>,
    json: bool,
    summary: bool,
    rest: Vec<String>,
    /// The options given, in order (checked against what the command reads).
    given: Vec<String>,
}

/// The options each single-game command reads (`parse` takes them all).
fn reads(cmd: &str) -> &'static [&'static str] {
    match cmd {
        "run" => &["--config", "--seed", "--ticks", "--policy", "--scenario", "--scenario-file", "--save", "--json"],
        "replay" => &["--summary"],
        "show" => &["--config"],
        "objectives" => &["--config", "--seed", "--ticks"],
        _ => &[],
    }
}

fn parse(args: &[String]) -> Result<Opts, Fail> {
    let mut o = Opts {
        config: BASELINE.into(),
        seed: 0,
        ticks: 1000,
        ticks_given: false,
        policy: Policy::Idle,
        scenario: None,
        save: None,
        json: false,
        summary: false,
        rest: Vec::new(),
        given: Vec::new(),
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = |name: &str| {
            it.next()
                .cloned()
                .ok_or_else(|| Fail::Usage(format!("{name} needs a value")))
        };
        if a.starts_with("--") {
            o.given.push(a.clone());
        }
        match a.as_str() {
            "--config" => o.config = val("--config")?,
            "--seed" => {
                let v = val("--seed")?;
                o.seed = v.parse().map_err(|_| Fail::Usage(format!("bad --seed {v:?}")))?;
            }
            "--ticks" => {
                let v = val("--ticks")?;
                o.ticks = v.parse().map_err(|_| Fail::Usage(format!("bad --ticks {v:?}")))?;
                o.ticks_given = true;
            }
            "--policy" => o.policy = Policy::parse(&val("--policy")?).map_err(Fail::Usage)?,
            "--scenario" => {
                o.scenario = Some(scenario(&val("--scenario")?).map_err(Fail::Usage)?.clone())
            }
            "--scenario-file" => {
                let f = val("--scenario-file")?;
                o.scenario = Some(
                    Scenario::from_json(&read(&f)?).map_err(|e| Fail::Usage(format!("{f}: {e}")))?,
                );
            }
            "--save" => o.save = Some(val("--save")?),
            "--json" => o.json = true,
            "--summary" => o.summary = true,
            s if s.starts_with("--") => return usage(format!("unknown option {s}")),
            s => o.rest.push(s.to_string()),
        }
    }
    Ok(o)
}

/// Runs a command line (without the program name); returns what to print.
pub fn run(args: &[String]) -> Result<String, Fail> {
    let Some((cmd, rest)) = args.split_first() else {
        return usage("no command");
    };
    if matches!(cmd.as_str(), "--help" | "-h" | "help") {
        return Ok(USAGE.to_string());
    }
    if lab::COMMANDS.contains(&cmd.as_str()) {
        return lab::run(cmd, rest);
    }
    if cmd == "playlog" {
        return playlog_cmd(rest);
    }
    if cmd == "vs-human" {
        return vs::run(rest);
    }
    if cmd == "accelerate" {
        return autopilot::cli(rest);
    }
    if cmd == "params" {
        return params::run(rest);
    }
    if cmd == "summarize" {
        return summarize_cmd(rest);
    }
    if cmd == "axes" {
        return axes::run(rest);
    }
    let o = parse(rest)?;
    if cmd != "replay" && cmd != "objectives" && !o.rest.is_empty() {
        return usage(format!("unexpected argument {:?}", o.rest[0]));
    }
    if let Some(opt) = o.given.iter().find(|g| !reads(cmd).contains(&g.as_str())) {
        return usage(format!("{cmd} does not take {opt}"));
    }
    match cmd.as_str() {
        "run" => run_game(&o),
        "replay" => replay(&o.rest, o.summary),
        "show" => {
            let c = load_config(&o.config)?;
            Ok(format!(
                "{}\nconfig hash {}\ncontent hash {}\nrules revision {RULES_REVISION}\n",
                c.to_json(),
                c.hash(),
                content_hash()
            ))
        }
        "gates" => Ok(gates::run()),
        "scenarios" => Ok(scenarios()
            .iter()
            .map(|s| format!("{}: {}\n", s.name, s.description))
            .collect()),
        "objectives" => {
            let [path] = o.rest.as_slice() else {
                return usage("objectives: give one scenario file");
            };
            let c = load_config(&o.config)?;
            let ticks = if o.ticks_given { o.ticks } else { 5000 };
            objectives::run(path, &read(path)?, &c, o.seed, ticks).map_err(Fail::Usage)
        }
        "golden" => {
            let c = load_config(BASELINE)?;
            let s = hh_core::driver::golden_session(&c).map_err(Fail::Usage)?;
            Ok(format!(
                "revision {RULES_REVISION} fingerprint {} content {} config {}\n",
                s.state.fingerprint(),
                content_hash(),
                c.hash()
            ))
        }
        other => usage(format!("unknown command {other:?}")),
    }
}

fn run_game(o: &Opts) -> Result<String, Fail> {
    let c = load_config(&o.config)?;
    let mut s = Session::new(&c, o.seed, o.scenario.as_ref()).map_err(Fail::Usage)?;
    play(&mut s, o.ticks, o.policy);
    let rec = s.record();
    let (replayed, warnings) = rec.replay().map_err(Fail::Check)?;
    let replay_ok = replayed == s.state && warnings.is_empty();
    if let Some(path) = &o.save {
        std::fs::write(path, rec.to_json() + "\n").map_err(|e| Fail::Usage(format!("{path}: {e}")))?;
    }
    let out = if o.json {
        serde_json::json!({
            "engine": VERSION,
            "rules_revision": RULES_REVISION,
            "config": c.name,
            "config_hash": c.hash(),
            "content_hash": content_hash(),
            "seed": o.seed.to_string(),
            "scenario": o.scenario.as_ref().map(|s| &s.name),
            "actions": rec.actions.len(),
            "fingerprint": s.state.fingerprint(),
            "replay_ok": replay_ok,
            "state": s.state,
        })
        .to_string()
            + "\n"
    } else {
        let mut t = String::new();
        let _ = writeln!(
            t,
            "{} (config {}, content {}), seed {}{}, rules revision {RULES_REVISION}",
            c.name,
            c.hash(),
            content_hash(),
            o.seed,
            o.scenario.as_ref().map(|s| format!(", scenario {}", s.name)).unwrap_or_default()
        );
        t.push_str(&state_summary(&s.rules, &s.state, rec.actions.len()));
        let _ = writeln!(t, "fingerprint {}", s.state.fingerprint());
        let _ = writeln!(t, "replay {}", if replay_ok { "ok" } else { "MISMATCH" });
        t
    };
    if replay_ok {
        Ok(out)
    } else {
        Err(Fail::Check(format!("{out}replay mismatch: {warnings:?}")))
    }
}

/// `hh-sim playlog LOG [--json] [--milestones F] [--human-models F] [--today]`: replays
/// a human's play log and measures it as the ladder measures an agent; `--today` plays
/// its actions best-effort on today's rules instead ([`playlog::replay_today`]).
fn playlog_cmd(args: &[String]) -> Result<String, Fail> {
    let (mut path, mut json, mut today) = (None, false, false);
    let mut milestones = MILESTONES.to_string();
    let mut humans = HUMANS.to_string();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => json = true,
            "--today" => today = true,
            "--milestones" => milestones = it.next().ok_or(Fail::Usage("--milestones needs a value".into()))?.clone(),
            "--human-models" => humans = it.next().ok_or(Fail::Usage("--human-models needs a value".into()))?.clone(),
            f if f.starts_with("--") => return usage(format!("playlog: unknown option {f:?}")),
            f if path.is_none() => path = Some(f.to_string()),
            f => return usage(format!("playlog: one log at a time (also given {f:?})")),
        }
    }
    let Some(path) = path else {
        return usage("playlog: give a log file");
    };
    let log = load_log(&path)?;
    if today {
        let r = playlog::replay_today(&log).map_err(|e| Fail::Check(format!("{path}: {e}")))?;
        return Ok(if json {
            serde_json::json!({ "log": path, "warnings": log.warnings, "today": r }).to_string() + "\n"
        } else {
            playlog::today_report(&path, &log, &r)
        });
    }
    let ms = load_milestones(&milestones)?;
    let pf = load_humans(&humans)?;
    let r = playlog::replay(&log, &ms).map_err(|e| Fail::Check(format!("{path}: replay failed: {e}")))?;
    let out = if json {
        serde_json::json!({
            "log": path,
            "career": log.career,
            "rules_revision": log.rules_revision,
            "config_hash": log.config_hash,
            "commit": log.commit,
            "fingerprint": r.fingerprint,
            "reproduced": r.matches,
            "accelerated": log.accelerated,
            "game": r.game,
            "time": r.time,
        })
        .to_string()
            + "\n"
    } else {
        playlog::report(&path, &log, &r, ms, pf)
    };
    match r.matches {
        Some(false) => Err(Fail::Check(out)),
        _ => Ok(out),
    }
}

/// `hh-sim summarize GAMES.jsonl [--milestones F]`: a read's per-game tables.
fn summarize_cmd(args: &[String]) -> Result<String, Fail> {
    let mut path = None;
    let mut milestones = MILESTONES.to_string();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--milestones" => milestones = it.next().ok_or(Fail::Usage("--milestones needs a value".into()))?.clone(),
            f if f.starts_with("--") => return usage(format!("summarize: unknown option {f:?}")),
            f if path.is_none() => path = Some(f.to_string()),
            f => return usage(format!("summarize: one games.jsonl at a time (also given {f:?})")),
        }
    }
    let Some(path) = path else {
        return usage("summarize: give a games.jsonl");
    };
    let games = summary::parse(&read(&path)?).map_err(|e| Fail::Usage(format!("{path}: {e}")))?;
    let ms = load_milestones(&milestones)?;
    Ok(summary::tables(&games, &ms))
}

/// The end of a game as `run` prints it: tick, run and day, each resource,
/// the buildings owned and the career log.
fn state_summary(rules: &hh_core::Rules, g: &hh_core::GameState, actions: usize) -> String {
    let mut t = String::new();
    let v = rules.view(g);
    let _ = writeln!(t, "tick {}, run {} day {}, {actions} actions", g.tick, v.run, v.day);
    for r in &v.resources {
        let cap = r.cap.map(|c| format!(" / {c}")).unwrap_or_default();
        let _ = writeln!(t, "  {:<16} {}{cap}  ({} per day)", r.name, r.amount, r.per_day);
    }
    for b in v.buildings.iter().filter(|b| b.owned > 0) {
        let _ = writeln!(t, "  {:<20} {} owned, {} active", b.name, b.owned, b.active);
    }
    t.push_str(&objectives::career_log(g));
    t
}

fn replay(paths: &[String], summary: bool) -> Result<String, Fail> {
    if paths.is_empty() {
        return usage("replay: no record given");
    }
    let mut out = String::new();
    let mut bad = false;
    for p in paths {
        let rec = Record::from_json(&read(p)?);
        match rec.as_ref().map_err(Clone::clone).and_then(|r| r.replay()) {
            Ok((g, warnings)) => {
                let _ = writeln!(out, "{p}: tick {}, fingerprint {}", g.tick, g.fingerprint());
                if let (true, Ok(r)) = (summary, &rec) {
                    let rules = hh_core::Rules::new(&r.config).map_err(Fail::Usage)?;
                    out.push_str(&state_summary(&rules, &g, r.actions.len()));
                }
                for w in &warnings {
                    let _ = writeln!(out, "  warning: {w}");
                }
                bad |= !warnings.is_empty();
            }
            Err(e) => {
                let _ = writeln!(out, "{p}: FAILED: {e}");
                bad = true;
            }
        }
    }
    if bad {
        Err(Fail::Check(out))
    } else {
        Ok(out)
    }
}
