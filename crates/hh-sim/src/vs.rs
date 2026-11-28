//! `hh-sim vs-human LOG`: the sim players on a human's game, side by side
//! (docs/simulator.md, *Against a human*). It replays the play log
//! ([`crate::playlog`]), plays each sim player on the log's config, seed and
//! scenario for as many ticks as the human played (`--ticks` for more), and
//! prints them beside the human in game time: milestones in ticks and
//! retirements, run lengths, credits a run, buildings and commands used,
//! decisions a run; real time from the log's clock for the human and from a
//! human model for the players (`--humans`). Then where the human first
//! diverged from the best player (most milestones, then most credits) in what
//! was built, and when.
//!
//! `--skilled` (131; from the record since 135) reads the log's play record
//! ([`crate::playrecord`]) instead of replaying it, so it works on a log from any rules
//! revision: the human's milestones, long goals and credits a day as the log recorded
//! them, beside the skilled player's career on today's rules, the log's (tolerantly
//! loaded) config and seed, for as many ticks.

use std::fmt::Write as _;

use serde::Serialize;

use crate::career::{self, Build, GameResult, RunResult};
use crate::human::Humans;
use std::collections::BTreeMap;

use crate::milestones::{Hit, Milestones, VsHuman};
use crate::playlog;
use crate::playrecord::{PlayRecord, RunCredits};
use crate::{list, read, usage, Fail, HUMANS, LADDER, MILESTONES, THREADS};

/// A row: the human or a sim player, by name.
#[derive(Debug, Clone, Serialize)]
pub struct Row {
    pub name: String,
    pub spec: String,
    pub game: GameResult,
}

/// Where the human first parted from the best player.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Divergence {
    pub best: String,
    /// The first purchase (run, its number in the run) where the two bought
    /// different buildings, or one bought and the other didn't.
    pub what: Option<Parting>,
    /// The first purchase, before `what`, of the same building far apart in time.
    pub when: Option<Parting>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Parting {
    pub run: u32,
    /// 1-based, within the run.
    pub nth: usize,
    pub human: Option<Build>,
    pub best: Option<Build>,
    /// The next few purchases on each side from here.
    pub human_next: Vec<Build>,
    pub best_next: Vec<Build>,
}

fn by_run(builds: &[Build], run: u32) -> Vec<&Build> {
    builds.iter().filter(|b| b.run == run).collect()
}

/// Matches the two build orders purchase by purchase, run by run, up to the
/// runs both played; `at` says how far apart in time counts as parting.
pub fn diverge(human: &GameResult, best: &GameResult, best_name: &str, at: &VsHuman) -> Divergence {
    let mut d = Divergence { best: best_name.to_string(), ..Default::default() };
    let runs = (human.runs.len() + 1).min(best.runs.len() + 1) as u32;
    let next = |v: &[&Build], k: usize| v.iter().skip(k).take(5).map(|b| (*b).clone()).collect::<Vec<_>>();
    for run in 1..=runs {
        let (h, b) = (by_run(&human.builds, run), by_run(&best.builds, run));
        // In the last run both played, compare only as far as both got.
        let last = run == runs;
        let n = if last { h.len().min(b.len()) } else { h.len().max(b.len()) };
        for k in 0..n {
            let (x, y) = (h.get(k), b.get(k));
            let part = || Parting {
                run,
                nth: k + 1,
                human: x.map(|v| (*v).clone()),
                best: y.map(|v| (*v).clone()),
                human_next: next(&h, k + 1),
                best_next: next(&b, k + 1),
            };
            match (x, y) {
                (Some(x), Some(y)) if x.building == y.building => {
                    let far = at.when_ticks.max(y.tick.saturating_sub(best.start_tick) * at.when_percent / 100);
                    if d.when.is_none() && x.tick.abs_diff(y.tick) > far {
                        d.when = Some(part());
                    }
                }
                _ => {
                    d.what = Some(part());
                    return d;
                }
            }
        }
    }
    d
}

/// The best player's index in `rows`: the most milestones, then the most
/// credits earned; ties go to the first listed.
pub fn best(rows: &[Row]) -> Option<usize> {
    let credits = |g: &GameResult| g.all_runs().map(|r| r.credits_earned).sum::<f64>();
    let mut best: Option<usize> = None;
    for (i, r) in rows.iter().enumerate() {
        let better = best.is_none_or(|b| {
            let (x, y) = (&r.game, &rows[b].game);
            x.milestones.len().cmp(&y.milestones.len()).then(credits(x).total_cmp(&credits(y))).is_gt()
        });
        if better {
            best = Some(i);
        }
    }
    best
}

fn runs_line(g: &GameResult, f: impl Fn(&RunResult) -> String) -> String {
    let mut parts: Vec<String> = g.runs.iter().map(&f).collect();
    if let Some(c) = &g.current {
        parts.push(format!("{}+", f(c)));
    }
    if parts.len() > 8 {
        let n = parts.len();
        parts.drain(4..n - 3);
        parts.insert(4, "…".into());
    }
    parts.join(", ")
}

fn used(all: usize, never: &[String]) -> usize {
    all - never.len()
}

fn build_text(b: &Option<Build>) -> String {
    b.as_ref().map_or("nothing".into(), |b| format!("{} at tick {}", b.building, b.tick))
}

fn next_text(v: &[Build]) -> String {
    if v.is_empty() {
        "-".into()
    } else {
        v.iter().map(|b| format!("{} t{}", b.building, b.tick)).collect::<Vec<_>>().join(", ")
    }
}

/// The side-by-side report.
pub fn report(rows: &[Row], div: &Divergence, ms: &Milestones, humans: &Humans, models: &[String]) -> String {
    let c = hh_core::content();
    let (nb, nc) = (c.buildings.len(), c.commands.len());
    let mut t = String::new();
    let _ = writeln!(
        t,
        "## Side by side in game time (`+`: the run under way at the end)\n\n| Player | Ticks | Ret | Run lengths | Credits a run | Decisions a run | Decisions | Ticks/dec | Buildings used | Commands used |\n|---|---|---|---|---|---|---|---|---|---|"
    );
    for r in rows {
        let g = &r.game;
        let _ = writeln!(
            t,
            "| {} | {} | {} | {} | {} | {} | {} | {:.1} | {}/{nb} | {}/{nc} |",
            r.name,
            g.ticks,
            g.retirements,
            runs_line(g, |r| r.ticks.to_string()),
            runs_line(g, |r| format!("{:.0}", r.credits_earned)),
            runs_line(g, |r| r.decisions.to_string()),
            g.decisions,
            g.ticks as f64 / g.decisions.max(1) as f64,
            used(nb, &g.never_built),
            used(nc, &g.never_used),
        );
    }
    let _ = writeln!(t, "\n## Milestones (`t` career tick, `r` retirements before it)\n");
    let _ = writeln!(t, "{}", crate::experiment::milestone_head(ms, "| Player |"));
    for r in rows {
        let mut line = format!("| {} |", r.name);
        for m in &ms.milestones {
            match r.game.milestones.get(&m.id) {
                Some(h) => {
                    let _ = write!(line, " t{} r{} |", h.tick, h.run.saturating_sub(1));
                }
                None => line.push_str(" - |"),
            }
        }
        let _ = writeln!(t, "{line}");
    }
    let _ = writeln!(t, "\nTargets (run windows): {}", crate::experiment::targets_line(ms));
    let _ = writeln!(
        t,
        "\n## Real time (the human's from the log's clock; a sim player's projected by a human model)\n"
    );
    let _ = writeln!(t, "{}", crate::experiment::milestone_head(ms, "| Player | Clock | Minutes | Min/dec | Run 1 min |"));
    for r in rows {
        let clocks: Vec<(String, Option<crate::human::Clock>)> = if r.game.own_clock.is_some() {
            vec![("log".into(), r.game.clock(None))]
        } else {
            models.iter().map(|m| (m.clone(), r.game.clock(humans.get(m).ok()))).collect()
        };
        for (name, clock) in clocks {
            let Some(clock) = clock else { continue };
            let total = clock.total() / 60.0;
            let run1 = r.game.runs.first().map_or("-".into(), |x| {
                format!("{:.1}", clock.span(x.start_tick, x.end_tick) / 60.0)
            });
            let mut line = format!(
                "| {} | {name} | {total:.1} | {:.2} | {run1} |",
                r.name,
                total / r.game.decisions.max(1) as f64
            );
            for m in &ms.milestones {
                match r.game.milestones.get(&m.id) {
                    Some(h) => {
                        let _ = write!(line, " {:.0}m |", clock.at(h.tick) / 60.0);
                    }
                    None => line.push_str(" - |"),
                }
            }
            let _ = writeln!(t, "{line}");
        }
    }
    if let (Some(h), Some(b)) = (rows.first(), rows.iter().find(|r| r.name == div.best)) {
        let only = |a: &GameResult, x: &GameResult, never: fn(&GameResult) -> &Vec<String>| -> String {
            let v: Vec<String> =
                never(x).iter().filter(|id| !never(a).contains(id)).cloned().collect();
            crate::experiment::joined(&v)
        };
        let _ = writeln!(t, "\n## The human against the best player, {}\n", div.best);
        let _ = writeln!(t, "- Buildings only the human built: {}", only(&h.game, &b.game, |g| &g.never_built));
        let _ = writeln!(t, "- Buildings only {} built: {}", div.best, only(&b.game, &h.game, |g| &g.never_built));
        let _ = writeln!(t, "- Commands only the human used: {}", only(&h.game, &b.game, |g| &g.never_used));
        let _ = writeln!(t, "- Commands only {} used: {}", div.best, only(&b.game, &h.game, |g| &g.never_used));
        match &div.when {
            Some(p) => {
                let _ = writeln!(
                    t,
                    "- First apart in when: run {}, purchase {}: the human bought {}, {} {}.",
                    p.run,
                    p.nth,
                    build_text(&p.human),
                    div.best,
                    build_text(&p.best)
                );
            }
            None => {
                let VsHuman { when_ticks, when_percent } = &ms.vs_human;
                let _ = writeln!(t, "- First apart in when: never, before the build orders part (within {when_ticks} ticks or {when_percent}% of the tick).");
            }
        }
        match &div.what {
            Some(p) => {
                let _ = writeln!(
                    t,
                    "- First apart in what: run {}, purchase {}: the human bought {}, {} {}.\n  - The human next: {}\n  - {} next: {}",
                    p.run,
                    p.nth,
                    build_text(&p.human),
                    div.best,
                    build_text(&p.best),
                    next_text(&p.human_next),
                    div.best,
                    next_text(&p.best_next)
                );
            }
            None => {
                let _ = writeln!(t, "- First apart in what: never, as far as both played (the same buildings in the same order).");
            }
        }
    }
    t
}

/// `hh-sim vs-human LOG [--agents a,b] [--humans a,b] [--ticks N] [--json]
/// [--threads T] [--milestones F] [--human-models F] [--include-accelerated]`.
/// `--skilled`'s player: the skilled tier, the oracle's quests profile (the
/// skilled side of `hh-sim axes`, uncommitted: it picks its own axis).
pub const SKILLED: &str = "skilled=oracle:profile=4";

/// `run N day D` for a hit, `-` for none.
fn at(h: Option<&Hit>) -> String {
    h.map_or("-".to_string(), |h| format!("run {} day {}", h.run, h.day))
}

/// One side of the skilled table: when each milestone and long goal was first met, and
/// credits a day by run.
#[derive(Debug, Clone, Serialize)]
pub struct Side {
    pub milestones: BTreeMap<String, Hit>,
    pub landmarks: BTreeMap<String, Hit>,
    pub runs: Vec<RunCredits>,
}

impl Side {
    /// A sim player's career, as measured.
    pub fn of_game(g: &GameResult) -> Side {
        let per = |r: &RunResult, open: bool| RunCredits {
            run: r.run,
            days: r.ticks,
            per_day: if r.ticks > 0 { r.credits_earned / r.ticks as f64 } else { 0.0 },
            open,
        };
        let runs = g.runs.iter().map(|r| per(r, false)).chain(g.current.iter().map(|r| per(r, true))).collect();
        // Landmark ticks from the career's start in a GameResult; career ticks, as a record's.
        let landmarks = g.landmarks.iter().map(|(k, h)| (k.clone(), Hit { tick: h.tick + g.start_tick, ..*h })).collect();
        Side { milestones: g.milestones.clone(), landmarks, runs }
    }

    /// A human's career, as its play log recorded it.
    pub fn of_record(r: &PlayRecord) -> Side {
        Side { milestones: r.milestones.clone(), landmarks: r.landmarks.clone(), runs: r.credits_by_run() }
    }
}

fn credits_line(s: &Side) -> String {
    if s.runs.is_empty() {
        return "-".into();
    }
    let mut parts: Vec<String> =
        s.runs.iter().map(|r| format!("{:.0}{}", r.per_day, if r.open { "+" } else { "" })).collect();
    if parts.len() > 8 {
        let n = parts.len();
        parts.drain(4..n - 3);
        parts.insert(4, "…".into());
    }
    parts.join(", ")
}

/// The human against the skilled player (131): run and day of each milestone, each
/// global project's first completion, the first rank 5, and credits a day by run.
pub fn skilled_report(human: &Side, skilled: &Side, ms: &Milestones) -> String {
    let mut t = String::from("\n## Against the skilled player (`--skilled`: ");
    let _ = writeln!(t, "{SKILLED})\n\n| Goal | Human | Skilled |\n|---|---|---|");
    for m in &ms.milestones {
        let _ = writeln!(t, "| {} {} | {} | {} |", m.id, m.name, at(human.milestones.get(&m.id)), at(skilled.milestones.get(&m.id)));
    }
    let keys: std::collections::BTreeSet<&String> =
        human.landmarks.keys().chain(skilled.landmarks.keys()).filter(|k| k.starts_with("project:")).collect();
    for k in keys {
        let _ = writeln!(t, "| {k} | {} | {} |", at(human.landmarks.get(k)), at(skilled.landmarks.get(k)));
    }
    // Rank 5 on any axis, first (the axis named).
    let first = |g: &Side| {
        g.landmarks
            .iter()
            .filter(|(k, _)| k.starts_with("rank5:"))
            .min_by_key(|(_, h)| (h.run, h.day))
            .map_or("-".to_string(), |(k, h)| format!("{} ({})", at(Some(h)), &k["rank5:".len()..]))
    };
    let _ = writeln!(t, "| rank 5 this run, first | {} | {} |", first(human), first(skilled));
    let done = |g: &Side| g.landmarks.keys().filter(|k| k.starts_with("project:")).count();
    let _ = writeln!(t, "| global projects done | {} | {} |", done(human), done(skilled));
    let _ = writeln!(t, "| credits a day, by run (`+` under way) | {} | {} |", credits_line(human), credits_line(skilled));
    t
}

/// `vs-human LOG --skilled`: the log's record beside the skilled player, no replay.
fn skilled_from_record(path: &str, milestones: &str, ticks: Option<u64>, include_accelerated: bool, json: bool) -> Result<String, Fail> {
    let ms = crate::load_milestones(milestones)?;
    let log = crate::load_log(path)?;
    let Some(rec) = &log.record else {
        return Err(Fail::Check(format!(
            "{path}: no play record: logs written before 135 keep none, so --skilled can't read this one. A plain vs-human replays it on a build of its own revision (scripts/baseline.py)."
        )));
    };
    if log.accelerated && !include_accelerated {
        return usage(format!(
            "vs-human: {path} used an accelerator (boost, autopilot or debug speed): no human's pace (--include-accelerated to compare anyway)"
        ));
    }
    let start = hh_core::Session::new(&log.config, log.seed, log.scenario.as_ref()).map_err(Fail::Usage)?.state.tick;
    let span = ticks.unwrap_or(log.last_tick.saturating_sub(start)).max(1);
    let g = career::play(&career::Game {
        config: &log.config,
        seed: log.seed,
        scenario: log.scenario.as_ref(),
        agent: crate::agents::named(SKILLED).1,
        retirements: u32::MAX,
        max_ticks: span,
        milestones: &ms,
        objectives: &[],
        trace: false,
    })
    .map_err(Fail::Usage)?;
    let (human, skilled) = (Side::of_record(rec), Side::of_game(&g));
    if json {
        return Ok(serde_json::json!({
            "log": path,
            "seed": log.seed.to_string(),
            "rules_revision": log.rules_revision,
            "warnings": log.warnings,
            "ticks": span,
            "human": human,
            "skilled": skilled,
        })
        .to_string()
            + "\n");
    }
    let mut t = format!(
        "{path}: seed {}, config {}, rules revision {}{}: the human's play record over {span} ticks; the skilled player on today's rules\n",
        log.seed,
        log.config_hash,
        log.rules_revision,
        if log.rules_revision != hh_core::RULES_REVISION { format!(" (this engine plays {})", hh_core::RULES_REVISION) } else { String::new() },
    );
    for w in &log.warnings {
        let _ = writeln!(t, "warning: {w}");
    }
    let _ = writeln!(t, "{}", playlog::record_line(&log));
    t.push_str(&skilled_report(&human, &skilled, &ms));
    Ok(t)
}

pub fn run(args: &[String]) -> Result<String, Fail> {
    let mut path = None;
    let (mut json, mut threads, mut ticks) = (false, THREADS, None::<u64>);
    let (mut include_accelerated, mut skilled) = (false, false);
    let mut agents: Option<Vec<String>> = None;
    let mut models: Option<Vec<String>> = None;
    let mut milestones = MILESTONES.to_string();
    let mut human_models = HUMANS.to_string();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = || it.next().cloned().ok_or_else(|| Fail::Usage(format!("{a} needs a value")));
        match a.as_str() {
            "--json" => json = true,
            "--include-accelerated" => include_accelerated = true,
            "--skilled" => skilled = true,
            "--agents" => agents = Some(crate::lab::agent_list(&val()?)),
            "--humans" => models = Some(list(&val()?)),
            "--ticks" => ticks = Some(val()?.parse().map_err(|_| Fail::Usage("bad --ticks".into()))?),
            "--threads" => threads = val()?.parse().map_err(|_| Fail::Usage("bad --threads".into()))?,
            "--milestones" => milestones = val()?,
            "--human-models" => human_models = val()?,
            f if f.starts_with("--") => return usage(format!("vs-human: unknown option {f:?}")),
            f if path.is_none() => path = Some(f.to_string()),
            f => return usage(format!("vs-human: one log at a time (also given {f:?})")),
        }
    }
    if threads == 0 {
        return usage("--threads must be at least 1");
    }
    let Some(path) = path else { return usage("vs-human: give a play log") };
    if skilled {
        if agents.is_some() || models.is_some() {
            return usage("vs-human --skilled reads the log's record and plays the skilled player alone: no --agents or --humans");
        }
        return skilled_from_record(&path, &milestones, ticks, include_accelerated, json);
    }
    // The ladder's players and human models unless given; never the oracle by
    // default (its careers take minutes: `--agents oracle` asks for it).
    let (agents, models) = match (agents, models) {
        (Some(a), Some(m)) => (a, m),
        (a, m) => {
            let spec = crate::experiment::Spec::from_json(&read(LADDER)?).map_err(|e| Fail::Usage(format!("{LADDER}: {e}")))?;
            let ladder = spec.agents.into_iter().filter(|p| crate::agents::kind(p) != "oracle").collect();
            (a.unwrap_or(ladder), m.unwrap_or(spec.humans))
        }
    };
    // The best-player comparison needs one of these (134: `--agents ""` panicked after the
    // careers had played).
    if agents.is_empty() {
        return usage("vs-human: no players");
    }
    let humans = crate::load_humans(&human_models)?;
    for m in &models {
        humans.get(m).map_err(Fail::Usage)?;
    }
    let ms = crate::load_milestones(&milestones)?;
    let mut names = vec!["human"];
    for a in &agents {
        crate::agents::build(a, 0, &ms).map_err(Fail::Usage)?;
        let name = crate::agents::named(a).0;
        if names.contains(&name) {
            return usage(format!("vs-human: two rows named {name:?} (the log's row is \"human\")"));
        }
        names.push(name);
    }
    let log = crate::load_log(&path)?;
    if log.accelerated && !include_accelerated {
        return usage(format!(
            "vs-human: {path} used an accelerator (boost, autopilot or debug speed): no human's pace (--include-accelerated to compare anyway)"
        ));
    }
    let h = playlog::replay(&log, &ms).map_err(|e| Fail::Check(format!("{path}: replay failed: {e}")))?;
    let span = ticks.unwrap_or(h.game.ticks).max(1);
    let out = crate::experiment::par_map(agents.len(), threads, |j| {
        career::play(&career::Game {
            config: &log.config,
            seed: log.seed,
            scenario: log.scenario.as_ref(),
            agent: crate::agents::named(&agents[j]).1,
            retirements: u32::MAX,
            max_ticks: span,
            milestones: &ms,
            objectives: &[],
            trace: false,
        })
    });
    let mut rows = vec![Row { name: "human".into(), spec: "log".into(), game: h.game.clone() }];
    for (a, r) in agents.iter().zip(out) {
        let (name, spec) = crate::agents::named(a);
        rows.push(Row { name: name.into(), spec: spec.into(), game: r.map_err(Fail::Usage)? });
    }
    let b = 1 + best(&rows[1..]).expect("a player");
    let div = diverge(&rows[0].game, &rows[b].game, &rows[b].name, &ms.vs_human);
    let replay_bad = rows.iter().any(|r| !r.game.replay_ok);
    let text = if json {
        serde_json::json!({
            "log": path,
            "seed": log.seed.to_string(),
            "reproduced": h.matches,
            "ticks": span,
            "rows": rows,
            "divergence": div,
        })
        .to_string()
            + "\n"
    } else {
        let mut t = format!(
            "{path}: seed {}, config {}, {} ticks from tick {} (the human's {}){}\n\n",
            log.seed,
            log.config_hash,
            span,
            h.game.start_tick,
            h.game.ticks,
            match h.matches {
                Some(true) => ", the log's final state reproduced",
                Some(false) => ", FINGERPRINT MISMATCH: the replay doesn't reach the log's state",
                None => ", no fingerprint at the log's last tick to check",
            }
        );
        t.push_str(&report(&rows, &div, &ms, &humans, &models));
        t
    };
    if h.matches == Some(false) || replay_bad {
        Err(Fail::Check(text))
    } else {
        Ok(text)
    }
}
