//! The measuring commands: `ladder`, `experiment`, `compare`, `balance`,
//! `agents`, `speed` (docs/simulator.md).

use std::fmt::Write as _;
use std::time::Instant;

use hh_core::{driver, play, Config, Session};

use crate::experiment::{self, Spec, Variant};
use crate::Fail;

pub const COMMANDS: &[&str] = &["ladder", "experiment", "compare", "balance", "trace", "agents", "speed"];

const LADDER: &str = "experiments/ladder.json";
const BALANCE: &str = "experiments/balance.json";
const HUMANS: &str = "configs/sim/humans.json";
const MILESTONES: &str = "configs/sim/milestones.json";

fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}

fn usage<T>(e: impl Into<String>) -> Result<T, Fail> {
    Err(Fail::Usage(e.into()))
}

#[derive(Default)]
struct Grid {
    spec: Option<String>,
    agents: Option<Vec<String>>,
    humans: Option<Vec<String>>,
    seeds: Option<u64>,
    first_seed: Option<u64>,
    retirements: Option<u32>,
    max_ticks: Option<u64>,
    threads: usize,
    out: Option<String>,
    human_models: String,
    milestones: String,
    config: Option<String>,
    config_b: Option<String>,
    set: Vec<String>,
    /// `trace --lines`: each day's program lines run.
    lines: bool,
}

/// Players, comma-separated; a piece with `=` but no `:` after a spec with
/// knobs is another knob of it (`author:a=1,b=2,greedy` is two players;
/// `casual=author:min_gap=120` one, named).
pub fn agent_list(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for piece in s.split(',').filter(|x| !x.is_empty()) {
        match out.last_mut() {
            Some(prev) if piece.contains('=') && !piece.contains(':') && prev.contains(':') => {
                prev.push(',');
                prev.push_str(piece);
            }
            _ => out.push(piece.to_string()),
        }
    }
    out
}

fn parse(args: &[String]) -> Result<Grid, Fail> {
    let mut g = Grid {
        threads: 6,
        human_models: HUMANS.into(),
        milestones: MILESTONES.into(),
        ..Default::default()
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = |name: &str| it.next().cloned().ok_or_else(|| Fail::Usage(format!("{name} needs a value")));
        let list = |s: String| s.split(',').filter(|x| !x.is_empty()).map(String::from).collect::<Vec<_>>();
        fn num<T: std::str::FromStr>(name: &str, v: String) -> Result<T, Fail> {
            v.parse().map_err(|_| Fail::Usage(format!("bad {name} {v:?}")))
        }
        match a.as_str() {
            "--lines" => g.lines = true,
            "--agents" => g.agents = Some(agent_list(&val(a)?)),
            "--humans" => g.humans = Some(list(val(a)?)),
            "--seeds" => g.seeds = Some(num(a, val(a)?)?),
            "--first-seed" => g.first_seed = Some(num(a, val(a)?)?),
            "--retirements" => g.retirements = Some(num(a, val(a)?)?),
            "--max-ticks" => g.max_ticks = Some(num(a, val(a)?)?),
            "--threads" => g.threads = num(a, val(a)?)?,
            "--out" => g.out = Some(val(a)?),
            "--human-models" => g.human_models = val(a)?,
            "--milestones" => g.milestones = val(a)?,
            "--config" => g.config = Some(val(a)?),
            "--config-b" => g.config_b = Some(val(a)?),
            "--set" => g.set.push(val(a)?),
            s if s.starts_with("--") => return usage(format!("unknown option {s}")),
            s if g.spec.is_none() => g.spec = Some(s.to_string()),
            s => return usage(format!("unexpected argument {s:?}")),
        }
    }
    if g.threads == 0 {
        return usage("--threads must be at least 1");
    }
    Ok(g)
}

impl Grid {
    /// The spec at `path` with the grid's overrides.
    fn spec(&self, path: &str) -> Result<Spec, Fail> {
        let mut s = Spec::from_json(&read(path).map_err(Fail::Usage)?).map_err(|e| Fail::Usage(format!("{path}: {e}")))?;
        if let Some(v) = &self.agents {
            s.agents = v.clone();
        }
        if let Some(v) = &self.humans {
            s.humans = v.clone();
        }
        if let Some(v) = self.seeds {
            s.seeds = v;
        }
        if let Some(v) = self.first_seed {
            s.first_seed = v;
        }
        if let Some(v) = self.retirements {
            s.retirements = v;
        }
        if let Some(v) = self.max_ticks {
            s.max_ticks = v;
        }
        if let Some(v) = &self.config {
            s.configs = vec![v.clone()];
        }
        // `--set` lands on every variant (for compare, on side B only: see there).
        for v in &mut s.variants {
            v.set.extend(self.set.iter().cloned());
        }
        s.check().map_err(Fail::Usage)?;
        Ok(s)
    }

    /// Refuses options the command doesn't read.
    fn only(&self, cmd: &str, out: bool, config_b: bool) -> Result<(), Fail> {
        if self.out.is_some() && !out {
            return usage(format!("{cmd} writes no files: --out not taken"));
        }
        if self.config_b.is_some() && !config_b {
            return usage("--config-b is compare's");
        }
        if self.lines && cmd != "trace" {
            return usage("--lines is trace's");
        }
        Ok(())
    }

    fn load(&self, spec: Spec) -> Result<experiment::Loaded, Fail> {
        experiment::load(
            spec,
            &read(&self.human_models).map_err(Fail::Usage)?,
            &read(&self.milestones).map_err(Fail::Usage)?,
            &read,
        )
            .map_err(Fail::Usage)
    }
}

pub fn run(cmd: &str, args: &[String]) -> Result<String, Fail> {
    let g = parse(args)?;
    match cmd {
        "ladder" | "experiment" => {
            let path = match (&g.spec, cmd) {
                (Some(p), _) => p.clone(),
                (None, "ladder") => LADDER.into(),
                (None, _) => return usage("experiment: give a spec file"),
            };
            g.only(cmd, true, false)?;
            let l = g.load(g.spec(&path)?)?;
            let res = experiment::run(&l, &read, g.threads).map_err(Fail::Usage)?;
            let sums: Vec<_> =
                res.cells.iter().map(|(c, games)| experiment::summarize(c, games, &l.milestones, &l.models())).collect();
            let mut table = experiment::tables(&sums, &l);
            let _ = writeln!(table, "\n{}", speed_line());
            let _ = writeln!(table, "{}", wasm_speed_line());
            let dir = g.out.clone().unwrap_or(format!("out/experiments/{}", l.spec.name));
            experiment::write_out(&dir, &l, &res, &sums, &table).map_err(Fail::Usage)?;
            let bad = sums.iter().any(|s| !s.replay_ok);
            let out = format!(
                "{} ({} games, {:.1} s wall)\n\n{table}\nwritten to {dir}/ (report.md, results.json, games.jsonl)\n",
                l.spec.name,
                res.cells.iter().map(|c| c.1.len()).sum::<usize>(),
                res.wall_s
            );
            if bad {
                Err(Fail::Check(format!("{out}a game's record did not replay to its state")))
            } else {
                Ok(out)
            }
        }
        "compare" => {
            let a_path = g.config.clone().unwrap_or("configs/baseline.json".into());
            let b_path = g.config_b.clone().unwrap_or(a_path.clone());
            if g.config_b.is_none() && g.set.is_empty() {
                return usage("compare: give --config-b B or --set module.key=value");
            }
            let base = g.spec.clone().unwrap_or(BALANCE.into());
            // Side A is the plain config, side B it or --config-b with --set;
            // the spec's own variants are dropped so the cells pair one to one.
            let mut sa = g.spec(&base)?;
            sa.name = "compare".into();
            sa.configs = vec![a_path.clone()];
            sa.variants = vec![Variant { name: "a".into(), set: Vec::new() }];
            let mut sb = sa.clone();
            sb.configs = vec![b_path.clone()];
            sb.variants = vec![Variant { name: "b".into(), set: g.set.clone() }];
            let la = g.load(sa)?;
            let lb = g.load(sb)?;
            let ra = experiment::run(&la, &read, g.threads).map_err(Fail::Usage)?;
            let rb = experiment::run(&lb, &read, g.threads).map_err(Fail::Usage)?;
            let name_b = if g.set.is_empty() { b_path.clone() } else { format!("{b_path} {}", g.set.join(" ")) };
            let models = la.models();
            let model = models.first().and_then(|(n, h)| Some((n.as_str(), (*h)?)));
            let mut t = experiment::compare_table(&ra, &rb, &la.milestones, (&a_path, &name_b), model);
            for (label, l, r) in [("A", &la, &ra), ("B", &lb, &rb)] {
                let sums: Vec<_> =
                    r.cells.iter().map(|(c, games)| experiment::summarize(c, games, &l.milestones, &l.models())).collect();
                let _ = write!(t, "\n# Side {label}\n\n{}", experiment::tables(&sums, l));
            }
            if let Some(dir) = &g.out {
                std::fs::create_dir_all(dir).map_err(|e| Fail::Usage(format!("{dir}: {e}")))?;
                std::fs::write(format!("{dir}/compare.md"), &t).map_err(|e| Fail::Usage(format!("{dir}: {e}")))?;
            }
            replayed(&[&ra, &rb], t)
        }
        "balance" => {
            g.only(cmd, false, false)?;
            let path = g.spec.clone().unwrap_or(BALANCE.into());
            let l = g.load(g.spec(&path)?)?;
            let res = experiment::run(&l, &read, g.threads).map_err(Fail::Usage)?;
            let sums: Vec<_> =
                res.cells.iter().map(|(c, games)| experiment::summarize(c, games, &l.milestones, &l.models())).collect();
            let lines = experiment::balance(&sums, &l.milestones);
            replayed(&[&res], lines.iter().map(|l| format!("{l}\n")).collect())
        }
        "trace" => {
            g.only(cmd, false, false)?;
            let path = g.spec.clone().unwrap_or(BALANCE.into());
            let l = g.load(g.spec(&path)?)?;
            let s = &l.spec;
            let (config, scenario) = experiment::first_cell(s, &read).map_err(Fail::Usage)?;
            let game = crate::career::Game {
                config: &config,
                seed: s.first_seed,
                scenario: scenario.as_ref(),
                agent: &s.agents[0],
                retirements: s.retirements,
                max_ticks: s.max_ticks,
                milestones: &l.milestones,
                objectives: &l.objectives,
                trace: true,
            };
            let (r, sess) = crate::career::play_session(&game).map_err(Fail::Usage)?;
            let mut t = String::new();
            // Actions and the agent's notes, by tick (a note after that look's actions).
            let rec = sess.record();
            let mut notes = r.notes.iter().peekable();
            // With --lines, each day's program lines before the actions taken after it.
            let days = if g.lines { day_lines(&rec).map_err(Fail::Usage)? } else { Vec::new() };
            let mut days = days.iter().peekable();
            for a in &rec.actions {
                // By tick; a day's lines before a note at its tick (the day played, then the look).
                loop {
                    let day = days.peek().map(|(tick, _)| *tick).filter(|&d| d <= a.tick);
                    let note = notes.peek().map(|(tick, _)| *tick).filter(|&n| n < a.tick);
                    match (day, note) {
                        (Some(d), n) if n.is_none_or(|n| d <= n) => {
                            let (tick, d) = days.next().expect("peeked");
                            let _ = writeln!(t, "{tick:>6}  {d}");
                        }
                        (_, Some(_)) => {
                            let (tick, n) = notes.next().expect("peeked");
                            let _ = writeln!(t, "{tick:>6}  # {n}");
                        }
                        _ => break,
                    }
                }
                let _ = writeln!(t, "{:>6}  {}", a.tick, serde_json::to_string(&a.action).expect("json"));
            }
            for (tick, d) in days {
                let _ = writeln!(t, "{tick:>6}  {d}");
            }
            for (tick, n) in notes {
                let _ = writeln!(t, "{tick:>6}  # {n}");
            }
            t.push_str(&crate::objectives::milestones(&sess));
            let c = &sess.state.career;
            let _ = writeln!(
                t,
                "career: lifetime credits earned {} over {} days, {} shipments; best run credits {}, best run shipments {}",
                c.lifetime.credits_earned, c.lifetime.days, c.lifetime.shipments, c.best_run_credits, c.best_run_shipments
            );
            let _ = writeln!(t, "{}", serde_json::to_string(&r.runs).expect("json"));
            let _ = writeln!(t, "{} looks, {} decisions, {} actions in {} ticks", r.looks, r.decisions, r.actions, r.ticks);
            let _ = writeln!(t, "{}", serde_json::to_string(&r.milestones).expect("json"));
            let _ = writeln!(t, "replay {}", if r.replay_ok { "ok" } else { "MISMATCH" });
            if r.replay_ok {
                Ok(t)
            } else {
                Err(Fail::Check(format!("{t}the game's record did not replay to its state")))
            }
        }
        "agents" => Ok(crate::agents::RUNGS.iter().map(|(k, w)| format!("{k:<8} {w}\n")).collect()),
        "speed" => Ok(format!("{}\n", speed_line())),
        _ => usage(format!("unknown command {cmd:?}")),
    }
}

/// The record replayed a day at a time: for each day a program line ran, the
/// tick and `lines p<program>: <line> <command> <outcome>, ...`, lines in queue
/// order (`trace --lines`).
pub fn day_lines(rec: &hh_core::Record) -> Result<Vec<(u64, String)>, String> {
    let rules = hh_core::Rules::new(&rec.config)?;
    let mut g = rules.start(rec.seed, rec.scenario.as_ref());
    let mut out = Vec::new();
    let mut actions = rec.actions.iter().peekable();
    loop {
        while let Some(a) = actions.next_if(|a| a.tick == g.tick) {
            rules.apply(&mut g, &a.action).map_err(|e| format!("tick {}: {e}", a.tick))?;
        }
        if g.tick >= rec.ticks {
            break;
        }
        rules.advance(&mut g, 1);
        let ran = hh_core::view::LinesRun::today(&g);
        let mut parts = Vec::new();
        for (p, row) in ran.lines.iter().enumerate() {
            let lines: Vec<String> = row
                .iter()
                .enumerate()
                .filter_map(|(i, o)| {
                    let o = (*o)?;
                    let cmd = &g.run.programs[p].entries[i].command;
                    Some(format!("{i} {cmd} {}", serde_json::to_value(o).expect("json").as_str().unwrap_or("?")))
                })
                .collect();
            if !lines.is_empty() {
                parts.push(format!("p{p}: {}", lines.join(", ")));
            }
        }
        if !parts.is_empty() {
            out.push((g.tick, format!("lines {}", parts.join("; "))));
        }
    }
    Ok(out)
}

/// `out`, or a check failure if any game's record didn't replay to its state.
fn replayed(results: &[&experiment::Results], out: String) -> Result<String, Fail> {
    let bad = results.iter().flat_map(|r| r.cells.iter()).flat_map(|c| c.1.iter()).any(|g| !g.replay_ok);
    if bad {
        Err(Fail::Check(format!("{out}a game's record did not replay to its state")))
    } else {
        Ok(out)
    }
}

/// Native engine speed: the golden game (the script reads a view each tick)
/// and a plain advance of a new game.
pub fn speed_line() -> String {
    let Ok(c) = read("configs/baseline.json").and_then(|t| Config::from_json(&t)) else {
        return "engine speed: configs/baseline.json not readable".into();
    };
    let t0 = Instant::now();
    let mut s = Session::new(&c, driver::GOLDEN_SEED, None).expect("baseline plays");
    play(&mut s, driver::GOLDEN_TICKS, driver::GOLDEN_POLICY);
    let scripted = driver::GOLDEN_TICKS as f64 / t0.elapsed().as_secs_f64();
    let t1 = Instant::now();
    let mut s = Session::new(&c, 1, None).expect("baseline plays");
    let n = 20_000;
    s.advance(n);
    let plain = n as f64 / t1.elapsed().as_secs_f64();
    format!("engine speed, native: {scripted:.0} ticks/s scripted (golden game, a view a tick), {plain:.0} ticks/s plain advance")
}

/// The same read in wasm, by `node scripts/wasm-speed.mjs` when the package is built.
pub fn wasm_speed_line() -> String {
    if !std::path::Path::new("crates/hh-wasm/pkg/hh_wasm_bg.wasm").exists() {
        return "engine speed, wasm: not measured (build crates/hh-wasm/pkg: README, Build and test)".into();
    }
    match std::process::Command::new("node").arg("scripts/wasm-speed.mjs").output() {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Ok(o) => format!("engine speed, wasm: failed ({})", String::from_utf8_lossy(&o.stderr).trim()),
        Err(e) => format!("engine speed, wasm: not measured (node: {e})"),
    }
}
