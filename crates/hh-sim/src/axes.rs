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
//! share of the rank-5 score, with the days 1.4 Trade was done (`trade@`), Fund
//! opened (`fund@`) and each project completed that run (`ba@1012`, from the
//! career log), then the global projects done at its end (`[ba lc]`). Cases run
//! in parallel (`--threads`).

use std::fmt::Write as _;

use hh_core::content::AXES;
use hh_core::modules::ideology;
use hh_core::scenario::Start;
use hh_core::{Config, Fx, Scenario, Session};

use crate::agents::{self, Player};
use crate::career;
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
    /// Cut short by `--max-ticks` with the run under way (134: such a run was dropped).
    cut: bool,
    /// The day its Fund command was first open at a look.
    fund_day: Option<u64>,
    /// The day quest 1.4 Trade was first seen done at a look (A Voice's start).
    trade_day: Option<u64>,
    /// Global projects completed in the career at its end.
    projects: Vec<String>,
    /// Projects completed during the run, each with its day, in order.
    done: Vec<(String, u64)>,
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

/// A case's watch over [`career::drive`]: each run's length, best score and
/// days, until rank 5 is reached or `max_runs` runs end.
struct Race {
    axis: &'static str,
    fund: String,
    max_runs: u32,
    case: Case,
    best: Fx,
    last_day: u64,
    fund_day: Option<u64>,
    trade_day: Option<u64>,
    /// The career log's length when the run began: its `project` entries
    /// since are the run's completions (run numbers restart at a timeline's end).
    from: usize,
}

impl Race {
    /// Ends the run in `case.runs`: the career's global projects done now
    /// (they outlive the run's end) and the run's completions.
    fn end_run(&mut self, s: &Session, forced: bool) {
        let v = s.rules.view(&s.state);
        let projects = v.projects.iter().filter(|x| x.completed && x.scope != "run").map(|x| x.id.to_string()).collect();
        let log = &s.state.career.log;
        let done = log[self.from..].iter().filter(|e| e.kind == "project").map(|e| (e.id.clone(), e.day)).collect();
        self.from = log.len();
        let (fund_day, trade_day) = (self.fund_day.take(), self.trade_day.take());
        self.case.runs.push(Run { days: self.last_day, best: self.best, forced, cut: false, fund_day, trade_day, projects, done });
        self.best = Fx::ZERO;
    }
}

impl career::Watch for Race {
    fn before(&mut self, p: &mut Player) {
        if self.fund_day.is_none() && agents::command_unlocked(p.view(), &self.fund) {
            self.fund_day = Some(p.view().day);
        }
        if self.trade_day.is_none() && p.state().career.completed_quests.contains(TRADE) {
            self.trade_day = Some(p.view().day);
        }
    }

    /// A retirement the agent chose.
    fn retired(&mut self, p: &mut Player) {
        self.end_run(p.session(), false);
    }

    fn tick(&mut self, s: &mut Session) -> Result<(), String> {
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
            self.end_run(s, forced);
            return Ok(());
        }
        self.last_day = g.run.day;
        self.best = self.best.max(ideology::score(g, self.axis));
        if ideology::rank(&s.rules, g, self.axis) >= RANK {
            let v = s.rules.view(g);
            let done: Vec<String> = v.projects.iter().filter(|p| p.completed && p.scope != "run").map(|p| p.id.to_string()).collect();
            self.case.reached = Some((g.career.run_number, g.run.day, done));
            self.end_run(s, false);
        }
        Ok(())
    }

    fn done(&self, _s: &Session) -> bool {
        self.case.reached.is_some() || self.case.runs.len() as u32 >= self.max_runs
    }
}

fn play(config: &Config, axis: &'static str, k: usize, skill: &'static str, seed: u64, max_runs: u32, max_ticks: u64) -> Result<Case, String> {
    let start = std::time::Instant::now();
    let ms = load_milestones(MILESTONES).map_err(|e| format!("{e:?}"))?;
    let scenario = (skill == "medium").then(medium_start);
    let mut s = Session::new(config, seed, scenario.as_ref())?;
    let mut agent = agents::build(&spec(skill, k), seed, &ms)?;
    let case = Case { axis, skill, seed, runs: Vec::new(), reached: None, wall_s: 0.0 };
    let from = s.state.career.log.len();
    let mut race = Race { axis, fund: format!("fund_{axis}"), max_runs, case, best: Fx::ZERO, last_day: 0, fund_day: None, trade_day: None, from };
    career::drive(&mut s, agent.as_mut(), max_ticks, &mut race)?;
    if !career::Watch::done(&race, &s) && race.last_day > 0 {
        race.end_run(&s, false);
        if let Some(r) = race.case.runs.last_mut() {
            r.cut = true;
        }
    }
    race.case.wall_s = start.elapsed().as_secs_f64();
    Ok(race.case)
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
    let _ = writeln!(out, "{} (config {}), rank {RANK} at score {bar}; runs: days (best score as % of rank {RANK}, f forced, c cut by --max-ticks; x@day: done that day)", config.name, config.hash());
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
                let at: String = r.done.iter().map(|(id, d)| format!(" {}@{d}", short(id))).collect();
                let fd = format!("{td}{fd}{at} [{}]", r.projects.iter().map(|x| short(x)).collect::<Vec<_>>().join(" "));
                format!("{}{} ({pct}%{fd})", r.days, if r.forced { "f" } else if r.cut { "c" } else { "" })
            })
            .collect();
        let projects = c.reached.as_ref().map_or("-".to_string(), |(_, _, p)| p.join(", "));
        let _ = writeln!(out, "| {} | {} | {} | {reached} | {} | {projects} | {:.0} |", c.skill, c.axis, c.seed, runs.join(", "), c.wall_s);
    }
    Ok(out)
}
