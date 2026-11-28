//! `hh-sim axes`: the committed ideology players (130), one per axis at each
//! skill, each playing a career until this run's rank on its axis first
//! reaches 5 (quest 1.5 A Voice's bar) or `--runs` runs end.
//!
//! - **skilled**: the oracle (quests profile) committed to the axis
//!   (`oracle:profile=4,commit=k`), from a new game.
//! - **medium**: the author committed to the axis (`author:commit=k`), from a
//!   new game with Blueprint Archive, Lunar Cartography and Foundation Grant
//!   already completed.
//!
//! Each case prints the run and day rank 5 was first reached, the projects
//! completed by then, and each run's length and best score on the axis as a
//! share of the rank-5 score. Cases run in parallel (`--threads`).

use std::fmt::Write as _;

use hh_core::content::AXES;
use hh_core::modules::ideology;
use hh_core::scenario::Start;
use hh_core::{Config, Fx, Scenario, Session};

use crate::agents::{self, Player};
use crate::{load_config, load_milestones, usage, Fail, BASELINE, MILESTONES, THREADS};

/// The rank the players race to (mirrors content/events.json 1.5 A Voice's
/// `ideology_rank_any` 5).
const RANK: i64 = 5;

/// Quest 1.4 Trade, whose completion starts 1.5 A Voice.
const TRADE: &str = "qtrade";

/// A project's initials, for the runs column (`blueprint_archive`: `ba`).
fn short(id: &str) -> String {
    id.split('_').filter_map(|w| w.chars().next()).collect()
}

/// Projects the medium player starts with completed (Matt's ruling, 130).
const MEDIUM_START: &[&str] = &["blueprint_archive", "lunar_cartography", "foundation_grant"];

/// One run of a case: its length and the best this-run score on the axis.
struct Run {
    days: u64,
    best: Fx,
    forced: bool,
    /// The day its Fund command was first open at a look.
    fund_day: Option<u64>,
    /// The day quest 1.4 Trade was first seen done at a look (A Voice's start).
    trade_day: Option<u64>,
    /// Global projects completed in the career at its end.
    projects: Vec<String>,
}

/// One case's career.
struct Case {
    axis: &'static str,
    skill: &'static str,
    seed: u64,
    runs: Vec<Run>,
    /// Run and day rank 5 was first reached, and the projects done by then.
    reached: Option<(u32, u64, Vec<String>)>,
    wall_s: f64,
}

/// The agent spec of a skill on an axis (`k` its `commit` knob value).
fn spec(skill: &str, k: usize) -> String {
    match skill {
        "skilled" => format!("oracle:profile=4,commit={k}"),
        _ => format!("author:commit={k}"),
    }
}

fn medium_start() -> Scenario {
    Scenario {
        name: "medium start".into(),
        description: "Blueprint Archive, Lunar Cartography and Foundation Grant completed".into(),
        start: Start {
            tick: 0,
            resources: Default::default(),
            buildings: Default::default(),
            panels: Vec::new(),
            programs: Vec::new(),
            flags: Vec::new(),
            projects: MEDIUM_START.iter().map(|s| s.to_string()).collect(),
            events: Vec::new(),
        },
    }
}

fn play(config: &Config, axis: &'static str, k: usize, skill: &'static str, seed: u64, max_runs: u32, max_ticks: u64) -> Result<Case, String> {
    let start = std::time::Instant::now();
    let ms = load_milestones(MILESTONES).map_err(|e| format!("{e:?}"))?;
    let scenario = (skill == "medium").then(medium_start);
    let mut s = Session::new(config, seed, scenario.as_ref())?;
    let mut agent = agents::build(&spec(skill, k), seed, &ms)?;
    let mut case = Case { axis, skill, seed, runs: Vec::new(), reached: None, wall_s: 0.0 };
    let mut seen_run = 0u32;
    let (mut best, mut last_day, mut fund_day, mut trade_day) = (Fx::ZERO, 0u64, None, None);
    // The career's global projects done now (they outlive the run's end).
    let global = |s: &Session| -> Vec<String> {
        let v = s.rules.view(&s.state);
        v.projects.iter().filter(|x| x.completed && x.scope != "run").map(|x| x.id.to_string()).collect()
    };
    let fund = format!("fund_{axis}");
    let end = s.state.tick.saturating_add(max_ticks);
    'career: while s.state.tick < end {
        let gap = {
            let mut p = Player::new(&mut s);
            let run = p.view().run;
            if run != seen_run {
                if seen_run != 0 {
                    agent.new_run(&mut p);
                }
                seen_run = run;
            }
            if fund_day.is_none() && agents::command_unlocked(p.view(), &fund) {
                fund_day = Some(p.view().day);
            }
            if trade_day.is_none() && p.state().career.completed_quests.contains(TRADE) {
                trade_day = Some(p.view().day);
            }
            agent.act(&mut p);
            let run = p.view().run;
            if run != seen_run {
                agent.new_run(&mut p);
                seen_run = run;
                // A retirement the agent chose.
                let projects = global(p.session());
                case.runs.push(Run { days: last_day, best, forced: false, fund_day: fund_day.take(), trade_day: trade_day.take(), projects });
                best = Fx::ZERO;
            }
            agent.next_look(&mut p).max(1)
        };
        if case.runs.len() as u32 >= max_runs {
            break 'career;
        }
        for _ in 0..gap {
            let run = s.state.career.run_number;
            s.advance(1);
            let g = &s.state;
            if g.career.run_number != run {
                // The run ended (chosen or forced): `retirements` rose either way.
                let forced = g
                    .career
                    .log
                    .iter()
                    .rev()
                    .find(|e| e.kind == "retired" || e.kind == "forced_retirement")
                    .is_some_and(|e| e.kind == "forced_retirement");
                let projects = global(&s);
                case.runs.push(Run { days: last_day, best, forced, fund_day: fund_day.take(), trade_day: trade_day.take(), projects });
                best = Fx::ZERO;
                if case.runs.len() as u32 >= max_runs {
                    break 'career;
                }
                break;
            }
            last_day = g.run.day;
            best = best.max(ideology::score(g, axis));
            if ideology::rank(&s.rules, g, axis) >= RANK {
                let v = s.rules.view(g);
                let done: Vec<String> = v.projects.iter().filter(|p| p.completed && p.scope != "run").map(|p| p.id.to_string()).collect();
                let projects = done.clone();
                case.reached = Some((g.career.run_number, g.run.day, done));
                case.runs.push(Run { days: last_day, best, forced: false, fund_day: fund_day.take(), trade_day: trade_day.take(), projects });
                break 'career;
            }
            if s.state.tick >= end {
                break 'career;
            }
        }
    }
    case.wall_s = start.elapsed().as_secs_f64();
    Ok(case)
}

/// `hh-sim axes [--seeds N] [--first-seed S] [--skills a,b] [--axes a,b]
/// [--runs N] [--max-ticks N] [--config F] [--set module.key=value]...
/// [--threads T]`.
pub fn run(args: &[String]) -> Result<String, Fail> {
    let (mut seeds, mut first, mut runs, mut max_ticks, mut threads) = (1u64, 1u64, 3u32, 40_000u64, THREADS);
    let mut skills: Vec<String> = vec!["skilled".into(), "medium".into()];
    let mut axes: Vec<String> = AXES.iter().map(|a| a.to_string()).collect();
    let mut config = BASELINE.to_string();
    let mut set: Vec<String> = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = || it.next().cloned().ok_or_else(|| Fail::Usage(format!("{a} needs a value")));
        let num = |s: String| s.parse::<u64>().map_err(|_| Fail::Usage(format!("{a}: {s} is not a number")));
        match a.as_str() {
            "--seeds" => seeds = num(val()?)?,
            "--first-seed" => first = num(val()?)?,
            "--runs" => runs = num(val()?)? as u32,
            "--max-ticks" => max_ticks = num(val()?)?,
            "--threads" => threads = num(val()?)?.max(1) as usize,
            "--skills" => skills = crate::list(&val()?),
            "--axes" => axes = crate::list(&val()?),
            "--config" => config = val()?,
            "--set" => set.push(val()?),
            other => return usage(format!("axes: unknown option {other:?}")),
        }
    }
    let base = load_config(&config)?;
    let config = crate::experiment::apply_set(&base, &set).map_err(Fail::Usage)?;
    let mut jobs: Vec<(&'static str, usize, &'static str, u64)> = Vec::new();
    for sk in &skills {
        let skill: &'static str = match sk.as_str() {
            "skilled" => "skilled",
            "medium" => "medium",
            other => return usage(format!("axes: unknown skill {other:?} (skilled, medium)")),
        };
        for ax in &axes {
            let Some(k) = AXES.iter().position(|a| a == ax) else {
                return usage(format!("axes: unknown axis {ax:?} ({})", AXES.join(", ")));
            };
            for seed in first..first + seeds {
                jobs.push((AXES[k], k + 1, skill, seed));
            }
        }
    }
    let next = std::sync::atomic::AtomicUsize::new(0);
    let results = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|sc| {
        for _ in 0..threads.min(jobs.len()) {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let Some(&(axis, k, skill, seed)) = jobs.get(i) else { break };
                let r = play(&config, axis, k, skill, seed, runs, max_ticks);
                results.lock().unwrap_or_else(|e| e.into_inner()).push((i, r));
            });
        }
    });
    let mut results = results.into_inner().unwrap_or_else(|e| e.into_inner());
    results.sort_by_key(|r| r.0);
    let bar = ideology::score_for_rank(&hh_core::Rules::new(&config).map_err(Fail::Usage)?, RANK);
    let mut out = String::new();
    let _ = writeln!(out, "{} (config {}), rank {RANK} at score {bar}; runs: days (best score as % of rank {RANK}, f forced)", config.name, config.hash());
    let _ = writeln!(out, "| Skill | Axis | Seed | Reached | Runs | Projects done by then | Wall s |");
    let _ = writeln!(out, "|---|---|---|---|---|---|---|");
    for (_, r) in results {
        let c = r.map_err(Fail::Check)?;
        let reached = c.reached.as_ref().map_or("no".to_string(), |(run, day, _)| format!("run {run} day {day}"));
        let runs: Vec<String> = c
            .runs
            .iter()
            .map(|r| {
                let pct = if bar > Fx::ZERO { (r.best.mul_int(100) / bar).floor_int() } else { 0 };
                let fd = r.fund_day.map_or(String::new(), |d| format!(" fund@{d}"));
                let td = r.trade_day.map_or(String::new(), |d| format!(" trade@{d}"));
                let fd = format!("{td}{fd} [{}]", r.projects.iter().map(|x| short(x)).collect::<Vec<_>>().join(" "));
                format!("{}{} ({pct}%{fd})", r.days, if r.forced { "f" } else { "" })
            })
            .collect();
        let projects = c.reached.as_ref().map_or("-".to_string(), |(_, _, p)| p.join(", "));
        let _ = writeln!(out, "| {} | {} | {} | {reached} | {} | {projects} | {:.0} |", c.skill, c.axis, c.seed, runs.join(", "), c.wall_s);
    }
    Ok(out)
}
