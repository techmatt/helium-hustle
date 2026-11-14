//! `hh-sim`, the native command line (docs/simulator.md). It reads files and
//! prints; every rule is `hh-core`'s.

use hh_core::{
    content_hash, play, scenario, scenarios, Config, Policy, Record, Scenario, Session,
    RULES_REVISION, VERSION,
};
use std::fmt::Write as _;

pub const USAGE: &str = "usage:
  hh-sim run [--config F] [--seed S] [--ticks N] [--policy P] [--scenario NAME | --scenario-file F] [--save F] [--json]
  hh-sim replay RECORD.json...
  hh-sim show [--config F]
  hh-sim scenarios
  hh-sim golden
defaults: --config configs/baseline.json --seed 0 --ticks 1000 --policy idle
policies: idle, greedy (buy whatever is affordable every tick), greedy:every=N";

/// A failure: bad input (exit 2) or a failed check (exit 1).
#[derive(Debug)]
pub enum Fail {
    Usage(String),
    Check(String),
}

fn usage<T>(e: impl Into<String>) -> Result<T, Fail> {
    Err(Fail::Usage(e.into()))
}

fn read(path: &str) -> Result<String, Fail> {
    std::fs::read_to_string(path).map_err(|e| Fail::Usage(format!("{path}: {e}")))
}

fn load_config(path: &str) -> Result<Config, Fail> {
    Config::from_json(&read(path)?).map_err(|e| Fail::Usage(format!("{path}: {e}")))
}

struct Opts {
    config: String,
    seed: u64,
    ticks: u64,
    policy: Policy,
    scenario: Option<Scenario>,
    save: Option<String>,
    json: bool,
    rest: Vec<String>,
}

fn parse(args: &[String]) -> Result<Opts, Fail> {
    let mut o = Opts {
        config: "configs/baseline.json".into(),
        seed: 0,
        ticks: 1000,
        policy: Policy::Idle,
        scenario: None,
        save: None,
        json: false,
        rest: Vec::new(),
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = |name: &str| {
            it.next()
                .cloned()
                .ok_or_else(|| Fail::Usage(format!("{name} needs a value")))
        };
        match a.as_str() {
            "--config" => o.config = val("--config")?,
            "--seed" => {
                let v = val("--seed")?;
                o.seed = v.parse().map_err(|_| Fail::Usage(format!("bad --seed {v:?}")))?;
            }
            "--ticks" => {
                let v = val("--ticks")?;
                o.ticks = v.parse().map_err(|_| Fail::Usage(format!("bad --ticks {v:?}")))?;
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
    let o = parse(rest)?;
    if cmd != "replay" && !o.rest.is_empty() {
        return usage(format!("unexpected argument {:?}", o.rest[0]));
    }
    match cmd.as_str() {
        "run" => run_game(&o),
        "replay" => replay(&o.rest),
        "show" => {
            let c = load_config(&o.config)?;
            Ok(format!(
                "{}\nconfig hash {}\ncontent hash {}\nrules revision {RULES_REVISION}\n",
                c.to_json(),
                c.hash(),
                content_hash()
            ))
        }
        "scenarios" => Ok(scenarios()
            .iter()
            .map(|s| format!("{}: {}\n", s.name, s.description))
            .collect()),
        "golden" => {
            let c = load_config("configs/baseline.json")?;
            let mut s = Session::new(&c, hh_core::driver::GOLDEN_SEED, None).map_err(Fail::Usage)?;
            play(&mut s, hh_core::driver::GOLDEN_TICKS, hh_core::driver::GOLDEN_POLICY);
            Ok(format!(
                "revision {RULES_REVISION} fingerprint {} content {}\n",
                s.state.fingerprint(),
                content_hash()
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
        let v = s.rules.view(&s.state);
        let _ = writeln!(
            t,
            "{} (config {}, content {}), seed {}{}, rules revision {RULES_REVISION}",
            c.name,
            c.hash(),
            content_hash(),
            o.seed,
            o.scenario.as_ref().map(|s| format!(", scenario {}", s.name)).unwrap_or_default()
        );
        let _ = writeln!(t, "day {}, {} actions", s.state.tick, rec.actions.len());
        for r in &v.resources {
            let cap = r.cap.map(|c| format!(" / {c}")).unwrap_or_default();
            let _ = writeln!(t, "  {:<12} {}{cap}  ({} per day)", r.name, r.amount, r.per_day);
        }
        for b in &v.buildings {
            let _ = writeln!(t, "  {:<12} {} owned", b.name, b.owned);
        }
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

fn replay(paths: &[String]) -> Result<String, Fail> {
    if paths.is_empty() {
        return usage("replay: no record given");
    }
    let mut out = String::new();
    let mut bad = false;
    for p in paths {
        match Record::from_json(&read(p)?).and_then(|r| r.replay()) {
            Ok((g, warnings)) => {
                let _ = writeln!(out, "{p}: day {}, fingerprint {}", g.tick, g.fingerprint());
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
