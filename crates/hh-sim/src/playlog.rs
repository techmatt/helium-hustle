//! `hh-sim playlog FILE`: replays a human's play log (the web client writes
//! one per career, `web/README.md`, *Play logs*) and measures the career as
//! the ladder measures a sim player's ([`crate::career::Meter`]), with real
//! time taken from the log's wall clock instead of a speed profile; then
//! reports where the human's real time went (speed mix, pauses, sessions).
//!
//! A log is JSON lines, in the order the client sent them. The first is the
//! header (`kind: "header"`, `format: "hh-playlog/1"`): rules revision, config
//! hash and the config itself, seed, scenario, the repo commit (stamped by the
//! dev server), and any actions taken before the log began (`prior_actions`).
//! Every other line has `t` (wall-clock ms), `kind`, `tick`, `run`, `day` and
//! `speed`; `action` lines carry the action as applied at that tick (the
//! game's record), `checkpoint` lines the state's fingerprint.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use hh_core::{Action, Config, Scenario, Session, Stamped, RULES_REVISION};
use serde::Serialize;
use serde_json::Value;

use crate::career::{GameResult, Meter};
use crate::experiment::{self, Cell, Loaded, Spec};
use crate::milestones::Milestones;
use crate::profile::Profiles;

pub const FORMAT: &str = "hh-playlog/1";

/// A line with a place in time: wall-clock ms, the career tick, the speed.
#[derive(Debug, Clone)]
struct Point {
    t: f64,
    tick: u64,
    speed: u32,
    kind: String,
    hidden: bool,
}

/// A parsed log.
pub struct Log {
    pub career: String,
    pub rules_revision: u32,
    pub config_hash: String,
    pub commit: String,
    pub config: Config,
    pub seed: u64,
    pub scenario: Option<Scenario>,
    pub actions: Vec<Stamped>,
    /// The last tick the log reached, and the fingerprint logged there, if any.
    pub last_tick: u64,
    pub last_fingerprint: Option<String>,
    points: Vec<Point>,
    /// Lines by kind, and actions the engine refused, by type.
    pub kinds: BTreeMap<String, u64>,
    pub rejected: BTreeMap<String, u64>,
    pub panels: BTreeMap<String, u64>,
    /// Lines dropped as repeats of an earlier line's `id`.
    pub duplicates: u64,
}

fn field<'a>(v: &'a Value, k: &str, line: usize) -> Result<&'a Value, String> {
    v.get(k).ok_or_else(|| format!("line {line}: no {k:?}"))
}

fn as_u64(v: &Value, k: &str, line: usize) -> Result<u64, String> {
    field(v, k, line)?.as_u64().ok_or_else(|| format!("line {line}: {k:?} is not a whole number"))
}

impl Log {
    pub fn parse(text: &str) -> Result<Log, String> {
        let mut lines = text.lines().enumerate().filter(|(_, l)| !l.trim().is_empty());
        let (_, first) = lines.next().ok_or("empty log")?;
        let h: Value = serde_json::from_str(first).map_err(|e| format!("line 1: {e}"))?;
        if h.get("kind").and_then(Value::as_str) != Some("header") || h.get("format").and_then(Value::as_str) != Some(FORMAT) {
            return Err(format!("line 1: not a {FORMAT} header"));
        }
        let config = Config::from_value(field(&h, "config", 1)?).map_err(|e| format!("header config: {e}"))?;
        let seed = field(&h, "seed", 1)?
            .as_str()
            .and_then(|s| s.parse().ok())
            .ok_or("header: seed is not a decimal string")?;
        let scenario: Option<Scenario> = match h.get("scenario") {
            None | Some(Value::Null) => None,
            Some(s) => Some(serde_json::from_value(s.clone()).map_err(|e| format!("header scenario: {e}"))?),
        };
        let mut actions: Vec<Stamped> = match h.get("prior_actions") {
            None | Some(Value::Null) => Vec::new(),
            Some(a) => serde_json::from_value(a.clone()).map_err(|e| format!("header prior_actions: {e}"))?,
        };
        let text_of = |k: &str| h.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        let mut log = Log {
            career: text_of("career"),
            rules_revision: h.get("rules_revision").and_then(Value::as_u64).unwrap_or(0) as u32,
            config_hash: text_of("config_hash"),
            commit: text_of("commit"),
            config,
            seed,
            scenario,
            actions: Vec::new(),
            last_tick: 0,
            last_fingerprint: None,
            points: Vec::new(),
            kinds: BTreeMap::new(),
            rejected: BTreeMap::new(),
            panels: BTreeMap::new(),
            duplicates: 0,
        };
        // In tick, then wall-clock, order: a page's closing beacon can land
        // before a send still in flight, so file order may swap neighbors (ticks
        // never fall within a career; the clock could step back).
        let mut parsed: Vec<(usize, u64, f64, Value)> = Vec::new();
        // A line can reach the file twice (a send the page saw fail but the
        // server took): the second copy of an `id` is dropped.
        let mut seen = std::collections::BTreeSet::new();
        for (i, l) in lines {
            let n = i + 1;
            let v: Value = serde_json::from_str(l).map_err(|e| format!("line {n}: {e}"))?;
            if let Some(id) = v.get("id").and_then(Value::as_str) {
                if !seen.insert(id.to_string()) {
                    log.duplicates += 1;
                    continue;
                }
            }
            let kind = field(&v, "kind", n)?.as_str().ok_or(format!("line {n}: kind"))?.to_string();
            *log.kinds.entry(kind).or_default() += 1;
            // Written at a later visit (a discarded save's career_end): no place in time.
            if v.get("offline").and_then(Value::as_bool) == Some(true) {
                continue;
            }
            let t = field(&v, "t", n)?.as_f64().ok_or(format!("line {n}: t"))?;
            parsed.push((n, as_u64(&v, "tick", n)?, t, v));
        }
        parsed.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.total_cmp(&b.2)));
        let mut hidden = false;
        for (n, tick, t, v) in parsed {
            let kind = field(&v, "kind", n)?.as_str().ok_or(format!("line {n}: kind"))?.to_string();
            match kind.as_str() {
                "action" => {
                    let a: Action = serde_json::from_value(field(&v, "action", n)?.clone())
                        .map_err(|e| format!("line {n}: action: {e}"))?;
                    actions.push(Stamped { tick, action: a });
                    // A fingerprint at this tick came before this action: stale.
                    if tick == log.last_tick {
                        log.last_fingerprint = None;
                    }
                }
                "rejected" => {
                    let t = v.pointer("/action/type").and_then(Value::as_str).unwrap_or("?");
                    *log.rejected.entry(t.to_string()).or_default() += 1;
                }
                "panel" => {
                    let p = v.get("panel").and_then(Value::as_str).unwrap_or("?");
                    *log.panels.entry(p.to_string()).or_default() += 1;
                }
                "hidden" => hidden = true,
                "visible" | "session_start" => hidden = false,
                _ => {}
            }
            // The fingerprint checks the replay only if logged at the last tick.
            if tick > log.last_tick {
                log.last_tick = tick;
                log.last_fingerprint = None;
            }
            if tick == log.last_tick {
                if let Some(f) = v.get("fingerprint").and_then(Value::as_str) {
                    log.last_fingerprint = Some(f.to_string());
                }
            }
            let speed = v.get("speed").and_then(Value::as_u64).unwrap_or(0) as u32;
            log.points.push(Point { t, tick, speed, kind, hidden });
        }
        log.actions = actions;
        Ok(log)
    }

    /// Active play seconds at each point: wall time from the first point,
    /// leaving out the closed time before each `session_start`.
    fn active_s(&self) -> Vec<f64> {
        let mut out = Vec::with_capacity(self.points.len());
        let mut s = 0.0;
        for (i, p) in self.points.iter().enumerate() {
            if i > 0 && p.kind != "session_start" {
                s += ((p.t - self.points[i - 1].t) / 1000.0).max(0.0);
            }
            out.push(s);
        }
        out
    }
}

/// Where the human's real time went.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TimeUse {
    pub sessions: u64,
    pub active_s: f64,
    /// Seconds with the page closed between sessions (not play time).
    pub closed_s: f64,
    pub hidden_s: f64,
    /// Real seconds and ticks at each speed (0 is paused).
    pub at_speed_s: BTreeMap<u32, f64>,
    pub ticks_at_speed: BTreeMap<u32, u64>,
    /// Times the game was paused by the player (not by a run summary).
    pub pauses: u64,
    /// Ticks passed by the debug menu's jump ahead (not counted at any speed).
    pub jumped_ticks: u64,
}

fn time_use(log: &Log) -> TimeUse {
    let mut u = TimeUse::default();
    for (i, p) in log.points.iter().enumerate() {
        if p.kind == "session_start" {
            u.sessions += 1;
        }
        if p.kind == "speed" && p.speed == 0 {
            u.pauses += 1;
        }
        let Some(next) = log.points.get(i + 1) else { continue };
        let dt = ((next.t - p.t) / 1000.0).max(0.0);
        if next.kind == "session_start" {
            u.closed_s += dt;
            continue;
        }
        u.active_s += dt;
        if p.hidden {
            u.hidden_s += dt;
        }
        *u.at_speed_s.entry(p.speed).or_default() += dt;
        let ticks = next.tick.saturating_sub(p.tick);
        if next.kind == "jump_end" {
            u.jumped_ticks += ticks;
        } else {
            *u.ticks_at_speed.entry(p.speed).or_default() += ticks;
        }
    }
    u
}

/// The replayed career: its measures, how it ended, and the time use.
pub struct Replayed {
    pub game: GameResult,
    pub time: TimeUse,
    pub fingerprint: String,
    /// The log's last fingerprint matched the replay's at the last tick.
    pub matches: Option<bool>,
}

/// Replays the log tick by tick under a [`Meter`]; each tick's real seconds
/// come from the log's wall clock (time at a tick before the next tick runs,
/// a pause included, counts toward that next tick).
pub fn replay(log: &Log, milestones: &Milestones) -> Result<Replayed, String> {
    let wall = std::time::Instant::now();
    let mut s = Session::new(&log.config, log.seed, log.scenario.as_ref())?;
    let mut meter = Meter::new(&s, milestones, &[]);
    let active = log.active_s();
    // The real seconds at which the career reached tick k: between the last
    // point before k and the first point at or after it, in proportion.
    let mut j = 0usize;
    let mut reached = |k: u64| -> f64 {
        while j + 1 < log.points.len() && log.points[j + 1].tick < k {
            j += 1;
        }
        let a = j;
        let b = (a..log.points.len()).find(|&x| log.points[x].tick >= k);
        match b {
            Some(b) if log.points[a].tick < k && log.points[b].tick > log.points[a].tick => {
                let (pa, pb) = (&log.points[a], &log.points[b]);
                active[a] + (active[b] - active[a]) * (k - pa.tick) as f64 / (pb.tick - pa.tick) as f64
            }
            Some(b) => active[b],
            None => active.last().copied().unwrap_or(0.0),
        }
    };
    let start = s.state.tick;
    let mut prev = 0.0f64;
    let mut looks = 0u64;
    let mut i = 0usize;
    loop {
        // Every action stamped at this tick, then a look.
        let mut acted = false;
        while i < log.actions.len() && log.actions[i].tick == s.state.tick {
            s.apply(&log.actions[i].action).map_err(|e| format!("action {i} at tick {}: {e}", s.state.tick))?;
            i += 1;
            acted = true;
        }
        if acted {
            looks += 1;
            meter.look(&s);
        }
        if i < log.actions.len() && log.actions[i].tick < s.state.tick {
            return Err(format!("action {i}: tick {} is before tick {}", log.actions[i].tick, s.state.tick));
        }
        if s.state.tick >= log.last_tick && i >= log.actions.len() {
            break;
        }
        let k = s.state.tick + 1;
        let now = if k <= start { 0.0 } else { reached(k) };
        meter.tick(&mut s, (now - prev).max(0.0))?;
        prev = now;
    }
    let fingerprint = s.state.fingerprint();
    let matches = log.last_fingerprint.as_ref().map(|f| *f == fingerprint);
    let game = meter.finish(&s, log.seed, "human", looks, Vec::new(), wall.elapsed().as_secs_f64());
    Ok(Replayed { game, time: time_use(log), fingerprint, matches })
}

fn mins(s: f64) -> String {
    format!("{:.1}", s / 60.0)
}

/// The report: the log's stamp, whether the replay reproduces it, the
/// ladder's tables for this one career, and the time use.
pub fn report(path: &str, log: &Log, r: &Replayed, milestones: Milestones, profiles: Profiles) -> String {
    let mut t = String::new();
    let _ = writeln!(
        t,
        "{path}: career {}, seed {}, rules revision {}{}, config {}, commit {}",
        log.career,
        log.seed,
        log.rules_revision,
        if log.rules_revision != RULES_REVISION { format!(" (this engine plays {RULES_REVISION})") } else { String::new() },
        log.config_hash,
        if log.commit.is_empty() { "unknown" } else { &log.commit }
    );
    let _ = writeln!(
        t,
        "replayed {} actions to tick {} (run {}, {} retirements): fingerprint {} {}",
        log.actions.len(),
        log.last_tick,
        r.game.runs.len() + 1,
        r.game.retirements,
        r.fingerprint,
        match r.matches {
            Some(true) => "= the log's: final state reproduced".to_string(),
            Some(false) => format!("!= the log's {}: MISMATCH", log.last_fingerprint.as_deref().unwrap_or("")),
            None => "(the log has no fingerprint at its last tick to check)".to_string(),
        }
    );
    let cell = Cell {
        config: log.config.name.clone(),
        variant: "human".into(),
        agent: "human".into(),
        attention: "log".into(),
        speed: "log".into(),
    };
    let sum = experiment::summarize(&cell, std::slice::from_ref(&r.game), &milestones);
    let spec = Spec {
        name: "playlog".into(),
        note: String::new(),
        configs: Vec::new(),
        variants: Vec::new(),
        agents: Vec::new(),
        attention: Vec::new(),
        speeds: Vec::new(),
        seeds: 1,
        first_seed: log.seed,
        retirements: r.game.retirements,
        max_ticks: log.last_tick,
        scenario: None,
        objectives: None,
    };
    let loaded = Loaded { spec, profiles, milestones, objectives: Vec::new() };
    t.push('\n');
    t.push_str(&experiment::tables(&[sum], &loaded));
    let u = &r.time;
    let _ = writeln!(t, "\n## Real time\n");
    let _ = writeln!(
        t,
        "{} sessions, {} min played ({} min of it hidden), {} min closed between sessions; {} pauses{}",
        u.sessions,
        mins(u.active_s),
        mins(u.hidden_s),
        mins(u.closed_s),
        u.pauses,
        if u.jumped_ticks > 0 { format!("; {} ticks jumped (cheat)", u.jumped_ticks) } else { String::new() }
    );
    let _ = writeln!(t, "\n| Speed | Real min | Share | Ticks |\n|---|---|---|---|");
    for (sp, secs) in &u.at_speed_s {
        let label = if *sp == 0 { "paused".to_string() } else { format!("{sp}x") };
        let _ = writeln!(
            t,
            "| {label} | {} | {:.0}% | {} |",
            mins(*secs),
            100.0 * secs / u.active_s.max(1e-9),
            u.ticks_at_speed.get(sp).copied().unwrap_or(0)
        );
    }
    let count = |m: &BTreeMap<String, u64>| -> String {
        if m.is_empty() {
            "-".into()
        } else {
            m.iter().map(|(k, n)| format!("{k} {n}")).collect::<Vec<_>>().join(", ")
        }
    };
    let _ = writeln!(t, "\nActions applied: {}; refused by the engine: {}", log.actions.len(), count(&log.rejected));
    let _ = writeln!(t, "Panels opened: {}", count(&log.panels));
    let _ = writeln!(
        t,
        "Log lines by kind: {}{}",
        count(&log.kinds),
        if log.duplicates > 0 { format!(" ({} repeated lines dropped)", log.duplicates) } else { String::new() }
    );
    t
}
