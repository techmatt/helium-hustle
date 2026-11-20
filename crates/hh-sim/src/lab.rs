//! The measuring commands: `ladder`, `experiment`, `compare`, `balance`, `trace`
//! (with its state dump and line replay), `agents`, `speed` (docs/simulator.md).
//! `lib.rs` dispatches these here, `vs-human` to `vs.rs`, and runs the rest itself.

use std::fmt::Write as _;
use std::time::Instant;

use hh_core::{driver, play, Config, Session};

use crate::experiment::{self, Spec, Variant};
use crate::{list, read, read_text, usage, Fail, BALANCE, BASELINE, HUMANS, LADDER, MILESTONES, THREADS};

pub const COMMANDS: &[&str] = &["ladder", "experiment", "compare", "balance", "trace", "agents", "speed"];

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
    /// `trace --at T`: the state at career tick T instead of the trace.
    at: Option<u64>,
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
        threads: THREADS,
        human_models: HUMANS.into(),
        milestones: MILESTONES.into(),
        ..Default::default()
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = |name: &str| it.next().cloned().ok_or_else(|| Fail::Usage(format!("{name} needs a value")));
        fn num<T: std::str::FromStr>(name: &str, v: String) -> Result<T, Fail> {
            v.parse().map_err(|_| Fail::Usage(format!("bad {name} {v:?}")))
        }
        match a.as_str() {
            "--lines" => g.lines = true,
            "--at" => g.at = Some(num(a, val(a)?)?),
            "--agents" => g.agents = Some(agent_list(&val(a)?)),
            "--humans" => g.humans = Some(list(&val(a)?)),
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
        let mut s = Spec::from_json(&read(path)?).map_err(|e| Fail::Usage(format!("{path}: {e}")))?;
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
        if (self.lines || self.at.is_some()) && cmd != "trace" {
            return usage("--lines and --at are trace's");
        }
        Ok(())
    }

    fn load(&self, spec: Spec) -> Result<experiment::Loaded, Fail> {
        experiment::load(
            spec,
            &read(&self.human_models)?,
            &read(&self.milestones)?,
            &read_text,
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
            let dir = g.out.clone().unwrap_or(format!("out/experiments/{}", l.spec.name));
            let res = experiment::run(&l, &read_text, g.threads, Some(&dir)).map_err(Fail::Usage)?;
            let sums = summaries(&res, &l);
            let mut table = experiment::tables(&sums, &l);
            let _ = writeln!(table, "\n{}", speed_line());
            let _ = writeln!(table, "{}", wasm_speed_line());
            experiment::write_out(&dir, &l, &res, &sums, &table).map_err(Fail::Usage)?;
            let out = format!(
                "{} ({} games, {:.1} s wall)\n\n{table}\nwritten to {dir}/ (report.md, results.json, games.jsonl)\n",
                l.spec.name,
                res.cells.iter().map(|c| c.1.len()).sum::<usize>(),
                res.wall_s
            );
            replayed(&[&res], out)
        }
        "compare" => {
            g.only(cmd, true, true)?;
            let a_path = g.config.clone().unwrap_or(BASELINE.into());
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
            let ra = experiment::run(&la, &read_text, g.threads, None).map_err(Fail::Usage)?;
            let rb = experiment::run(&lb, &read_text, g.threads, None).map_err(Fail::Usage)?;
            let name_b = if g.set.is_empty() { b_path.clone() } else { format!("{b_path} {}", g.set.join(" ")) };
            let models = la.models();
            let model = models.first().and_then(|(n, h)| Some((n.as_str(), (*h)?)));
            let mut t = experiment::compare_table(&ra, &rb, &la.milestones, (&a_path, &name_b), model);
            for (label, l, r) in [("A", &la, &ra), ("B", &lb, &rb)] {
                let _ = write!(t, "\n# Side {label}\n\n{}", experiment::tables(&summaries(r, l), l));
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
            let res = experiment::run(&l, &read_text, g.threads, None).map_err(Fail::Usage)?;
            let lines = experiment::balance(&summaries(&res, &l), &l.milestones);
            replayed(&[&res], lines.iter().map(|l| format!("{l}\n")).collect())
        }
        "trace" => {
            g.only(cmd, false, false)?;
            let path = g.spec.clone().unwrap_or(BALANCE.into());
            let l = g.load(g.spec(&path)?)?;
            let s = &l.spec;
            let (config, scenario) = experiment::first_cell(s, &read_text).map_err(Fail::Usage)?;
            // With --at, the career runs to that tick whatever its retirements.
            let game = crate::career::Game {
                config: &config,
                seed: s.first_seed,
                scenario: scenario.as_ref(),
                agent: &s.agents[0],
                retirements: if g.at.is_some() { u32::MAX } else { s.retirements },
                max_ticks: g.at.unwrap_or(s.max_ticks),
                milestones: &l.milestones,
                objectives: &l.objectives,
                trace: true,
            };
            let (r, sess) = crate::career::play_session(&game).map_err(Fail::Usage)?;
            if g.at.is_some() {
                let note = r.notes.last().map_or("", |n| n.1.as_str());
                return Ok(state_at(&sess, note));
            }
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
            t.push_str(&crate::objectives::career_log(&sess));
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
        "agents" | "speed" if !args.is_empty() => usage(format!("{cmd} takes no options")),
        "agents" => Ok(crate::agents::RUNGS.iter().map(|(k, w)| format!("{k:<8} {w}\n")).collect()),
        "speed" => Ok(format!("{}\n", speed_line())),
        _ => usage(format!("unknown command {cmd:?}")),
    }
}

/// `trace --at T`: the state's economy (resources against caps, buildings
/// with their next price, programs, markets), the agent's last note, and the
/// career's long goals: what walls a run in.
fn state_at(s: &Session, note: &str) -> String {
    let mut t = String::new();
    let v = s.rules.view(&s.state);
    let amount = |id: &str| v.resources.iter().find(|r| r.id == id).map_or(hh_core::Fx::ZERO, |r| r.amount);
    let _ = writeln!(t, "tick {} run {} day {} processors {} land {}", v.tick, v.run, v.day, v.processors, amount("land"));
    for r in &v.resources {
        let cap = r.cap.map_or("-".into(), |c| c.to_string());
        let _ = writeln!(t, "  {:12} {:>10} cap {:>10} per_day {:>8}", r.id, r.amount.to_string(), cap, r.per_day.to_string());
    }
    for b in v.buildings.iter().filter(|b| b.unlocked || b.owned > 0) {
        let cost: Vec<String> = b.cost.iter().map(|(k, x)| format!("{k} {}", x.floor_int())).collect();
        let _ = writeln!(
            t,
            "  {:20} owned {:>2} active {:>2} land {} next: {} short {:?}",
            b.id,
            b.owned,
            b.active,
            b.land,
            cost.join(", "),
            b.short
        );
    }
    for p in v.programs.iter().filter(|p| !p.entries.is_empty()) {
        let lines: Vec<String> = p.entries.iter().map(|e| e.describe()).collect();
        let _ = writeln!(t, "  program procs {}: {}", p.processors, lines.join(", "));
    }
    for m in &v.markets {
        let _ = writeln!(t, "  market {:10} demand {:?} tier {}", m.good, m.demand.map(|d| d.to_string()), m.tier);
    }
    let _ = writeln!(t, "  # {note}");
    let c = &s.state.career;
    let _ = writeln!(
        t,
        "career: retirements {} best run days {} best run credits {}",
        c.retirements, c.best_run_days, c.best_run_credits
    );
    let _ = writeln!(t, "  quests: {:?}", c.completed_quests);
    let _ = writeln!(t, "  sub-objectives: {:?}", c.completed_sub_objectives);
    let _ = writeln!(t, "  best ranks: {:?} best scores: {:?}", c.max_ideology_ranks, c.max_ideology_scores);
    let _ = writeln!(t, "  projects done: {:?} progress: {:?}", c.completed_projects, c.project_progress);
    let _ = writeln!(t, "  achievements: {:?}", c.achievements);
    t
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
                    let e = &g.run.programs[p].entries[i];
                    let cmd = e.command_id().map_or_else(|| e.describe(), str::to_string);
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

/// Each cell's summary, in the results' cell order.
fn summaries(r: &experiment::Results, l: &experiment::Loaded) -> Vec<experiment::Summary> {
    r.cells.iter().map(|(c, games)| experiment::summarize(c, games, &l.milestones, &l.models())).collect()
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
    let Ok(c) = read_text(BASELINE).and_then(|t| Config::from_json(&t)) else {
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
    if !std::path::Path::new("web/src/wasm/hh_wasm_bg.wasm").exists() {
        return "engine speed, wasm: not measured (build web/src/wasm: node web/scripts/build-wasm.mjs)".into();
    }
    match std::process::Command::new("node").arg("scripts/wasm-speed.mjs").output() {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Ok(o) => format!("engine speed, wasm: failed ({})", String::from_utf8_lossy(&o.stderr).trim()),
        Err(e) => format!("engine speed, wasm: not measured (node: {e})"),
    }
}
