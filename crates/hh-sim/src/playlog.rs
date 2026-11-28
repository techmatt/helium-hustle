//! `hh-sim playlog FILE`: replays a human's play log (the web client writes
//! one per career, `web/README.md`, *Play logs*) and measures the career as
//! the ladder measures a sim player's ([`crate::career::Meter`]), in game
//! time, with real time from the log's wall clock (a sim player's is a human
//! model's projection); then reports where the human's real time went (speed
//! mix, pauses, sessions). `hh-sim vs-human` ([`crate::vs`]) sets the same
//! career beside the sim players'.
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

use hh_core::{Action, Cheat, Config, Scenario, Session, Stamped, RULES_REVISION};
use serde::Serialize;
use serde_json::Value;

use crate::career::{GameResult, Meter};
use crate::experiment::{self, Cell, Loaded, Spec};
use crate::human::{Clock, Humans};
use crate::milestones::Milestones;
use crate::playrecord::PlayRecord;

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
    /// The career used an accelerator (a boost, the autopilot or a debug
    /// speed): no human's pace, so `vs-human` refuses it.
    pub accelerated: bool,
    /// What loading had to bend: a config key the log lacks (today's default used) or
    /// one today's config dropped ([`tolerant_config`]).
    pub warnings: Vec<String>,
    /// Action lines this build can't read (an action since renamed or reshaped): their
    /// tick and why. Exact replay refuses a log with any; [`replay_today`] skips them.
    pub unreadable: Vec<(u64, String)>,
    /// The play record the log carries (logs from 135 on), if any.
    pub record: Option<PlayRecord>,
}

/// A logged config, read tolerantly (Matt, 2026-11-27): as it is if this build takes it;
/// else laid over today's config (`defaults`, configs/baseline.json: the one home of a
/// key's default), each key the log lacks taking today's value and each key today's
/// config no longer has dropped, with a warning for each. A module whose variant
/// changed keeps its logged parameters whole (another variant's keys aren't its).
pub fn tolerant_config(logged: &Value, defaults: Option<&Value>, warnings: &mut Vec<String>) -> Result<Config, String> {
    let err = match Config::from_value(logged) {
        Ok(c) => return Ok(c),
        Err(e) => e,
    };
    let Some(today) = defaults else { return Err(err) };
    let mut w = Vec::new();
    let merged = merge(logged, today, "", 0, &mut w);
    let c = Config::from_value(&merged).map_err(|e| format!("{err}\n(laid over today's defaults, still: {e})"))?;
    warnings.extend(w);
    Ok(c)
}

/// Merges by the config's own shape: the top, `modules`, each module and its `params`
/// (depths 0 to 3). A parameter's value is the log's whole: a map in one
/// (`starting_resources`) is data keyed by content id, not more keys.
fn merge(logged: &Value, today: &Value, path: &str, depth: u32, w: &mut Vec<String>) -> Value {
    let (Value::Object(l), Value::Object(t)) = (logged, today) else { return logged.clone() };
    if depth > 3 {
        return logged.clone();
    }
    if l.contains_key("variant") && l.get("variant") != t.get("variant") {
        return logged.clone();
    }
    let at = |k: &str| if path.is_empty() { k.to_string() } else { format!("{path}.{k}") };
    let mut out = serde_json::Map::new();
    for (k, tv) in t {
        match l.get(k) {
            Some(lv) => {
                out.insert(k.clone(), merge(lv, tv, &at(k), depth + 1, w));
            }
            None => {
                w.push(format!("config {}: not in the log; today's default {tv} used", at(k)));
                out.insert(k.clone(), tv.clone());
            }
        }
    }
    for (k, lv) in l {
        if !t.contains_key(k) {
            w.push(format!("config {}: no longer a key; the log's {lv} dropped", at(k)));
        }
    }
    Value::Object(out)
}

/// The page's fastest speed but a debug one (`web/src/clock.ts`, `SPEEDS`): a log
/// that played faster used a debug speed, an accelerator.
pub const MAX_SPEED: u32 = 200;

/// An accelerator's action (`Cheat::Boost`, `Cheat::Autopilot`).
pub fn is_accelerator(a: &Action) -> bool {
    matches!(
        a,
        Action::Cheat {
            cheat: Cheat::Boost { .. } | Cheat::Autopilot { .. } | Cheat::CheatBuilding | Cheat::CompleteResearch
        }
    )
}

fn field<'a>(v: &'a Value, k: &str, line: usize) -> Result<&'a Value, String> {
    v.get(k).ok_or_else(|| format!("line {line}: no {k:?}"))
}

fn as_u64(v: &Value, k: &str, line: usize) -> Result<u64, String> {
    field(v, k, line)?.as_u64().ok_or_else(|| format!("line {line}: {k:?} is not a whole number"))
}

impl Log {
    /// A log read strictly: its config as logged, or an error.
    pub fn parse(text: &str) -> Result<Log, String> {
        Log::parse_with(text, None)
    }

    /// A log read tolerantly against today's config (`defaults`, [`tolerant_config`]).
    pub fn parse_with(text: &str, defaults: Option<&Value>) -> Result<Log, String> {
        let mut lines = text.lines().enumerate().filter(|(_, l)| !l.trim().is_empty());
        let (_, first) = lines.next().ok_or("empty log")?;
        let h: Value = serde_json::from_str(first).map_err(|e| format!("line 1: {e}"))?;
        if h.get("kind").and_then(Value::as_str) != Some("header") || h.get("format").and_then(Value::as_str) != Some(FORMAT) {
            return Err(format!("line 1: not a {FORMAT} header"));
        }
        let mut warnings = Vec::new();
        let config = tolerant_config(field(&h, "config", 1)?, defaults, &mut warnings).map_err(|e| format!("header config: {e}"))?;
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
            accelerated: actions.iter().any(|a| is_accelerator(&a.action)),
            warnings,
            unreadable: Vec::new(),
            record: None,
        };
        let mut record = PlayRecord::default();
        let mut has_record = false;
        // In tick, then wall-clock, order: a page's closing beacon can land
        // before a send still in flight, so file order may swap neighbors (ticks
        // never fall within a career; the clock could step back).
        let mut parsed: Vec<(usize, u64, f64, Value)> = Vec::new();
        // A line can reach the file twice (a send the page saw fail but the
        // server took): the second copy of an `id` is dropped.
        let mut seen = std::collections::BTreeSet::new();
        // The header too: a resent first batch repeats it.
        if let Some(id) = h.get("id").and_then(Value::as_str) {
            seen.insert(id.to_string());
        }
        for (i, l) in lines {
            let n = i + 1;
            let v: Value = serde_json::from_str(l).map_err(|e| format!("line {n}: {e}"))?;
            if let Some(id) = v.get("id").and_then(Value::as_str) {
                if !seen.insert(id.to_string()) {
                    log.duplicates += 1;
                    continue;
                }
            }
            if v.get("kind").and_then(Value::as_str) == Some("header") {
                log.duplicates += 1;
                continue;
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
                "record_start" | "snapshot" | "landmark" | "run_end" => {
                    has_record = true;
                    record.read(&v);
                }
                "action" => {
                    let a: Action = match serde_json::from_value(field(&v, "action", n)?.clone()) {
                        Ok(a) => a,
                        Err(e) => {
                            log.unreadable.push((tick, format!("line {n}: {e}")));
                            // An accelerator still counts, by its kind, when its shape changed.
                            let k = v.pointer("/action/cheat/kind").and_then(Value::as_str);
                            if matches!(k, Some("boost" | "autopilot" | "cheat_building" | "complete_research")) {
                                log.accelerated = true;
                            }
                            continue;
                        }
                    };
                    if is_accelerator(&a) {
                        log.accelerated = true;
                    }
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
            if speed > MAX_SPEED {
                log.accelerated = true;
            }
            // A jump ahead's or an autopilot's action starts the ticks its `jump_end` closes.
            let cheat = v.pointer("/action/cheat/kind").and_then(Value::as_str);
            let kind = if kind == "action" && matches!(cheat, Some("jump_ahead" | "autopilot")) {
                "jump_start".to_string()
            } else {
                kind
            };
            log.points.push(Point { t, tick, speed, kind, hidden });
        }
        log.actions = actions;
        log.record = has_record.then_some(record);
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
    // Between a jump's action and its `jump_end` (a forced retirement's lines may come
    // between): every tick is jumped.
    let mut jumping = false;
    for (i, p) in log.points.iter().enumerate() {
        if p.kind == "jump_start" {
            jumping = true;
        } else if matches!(p.kind.as_str(), "jump_end" | "speed" | "continue" | "session_start" | "session_end") {
            // (A log from before `jump_end` existed never closes its jump: the player's
            // next move does.)
            jumping = false;
        }
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
        if jumping || next.kind == "jump_end" {
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

/// Replays the log tick by tick under a [`Meter`]; the career's real clock
/// comes from the log's wall clock (time at a tick before the next tick runs,
/// a pause included, counts toward that next tick). A decision is a tick
/// where the human applied actions.
pub fn replay(log: &Log, milestones: &Milestones) -> Result<Replayed, String> {
    if let Some((tick, e)) = log.unreadable.first() {
        return Err(format!(
            "{} action lines this build can't read (first at tick {tick}, {e}): replay it on its own revision (scripts/baseline.py), or best-effort (--today)",
            log.unreadable.len()
        ));
    }
    // Exact replay plays the log's own config: one patched from today's defaults is
    // another game (the tolerant load is for the record and --today).
    if let Some(w) = log.warnings.first() {
        return Err(format!(
            "the log's config needed today's defaults ({w}{}): the log is rules revision {}, this build {RULES_REVISION}: replay it on a build of that revision (scripts/baseline.py), best-effort on today's rules (playlog --today), or read its record (vs-human --skilled)",
            if log.warnings.len() > 1 { format!(", and {} more", log.warnings.len() - 1) } else { String::new() },
            log.rules_revision
        ));
    }
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
    let mut clock = Clock { points: vec![(start, 0.0)] };
    let mut i = 0usize;
    loop {
        // Every action stamped at this tick, then a look.
        let mut acted = 0u32;
        while i < log.actions.len() && log.actions[i].tick == s.state.tick {
            s.apply(&log.actions[i].action).map_err(|e| format!("action {i} at tick {}: {e}", s.state.tick))?;
            i += 1;
            acted += 1;
        }
        if acted > 0 {
            meter.look(&s, acted);
        }
        if i < log.actions.len() && log.actions[i].tick < s.state.tick {
            return Err(format!("action {i}: tick {} is before tick {}", log.actions[i].tick, s.state.tick));
        }
        if s.state.tick >= log.last_tick && i >= log.actions.len() {
            break;
        }
        let k = s.state.tick + 1;
        let prev = clock.total();
        clock.points.push((k, reached(k).max(prev)));
        meter.tick(&mut s)?;
    }
    // Time at the last tick (deciding, paused) is played too.
    let end = active.last().copied().unwrap_or(0.0);
    if end > clock.total() {
        clock.points.push((s.state.tick, end));
    }
    let fingerprint = s.state.fingerprint();
    let matches = log.last_fingerprint.as_ref().map(|f| *f == fingerprint);
    let mut game = meter.finish(&s, log.seed, "human", Vec::new(), wall.elapsed().as_secs_f64());
    game.real_s = Some(clock.total());
    game.own_clock = Some(clock);
    Ok(Replayed { game, time: time_use(log), fingerprint, matches })
}

/// An action the best-effort replay couldn't apply.
#[derive(Debug, Clone, Serialize)]
pub struct Skip {
    pub tick: u64,
    pub run: u32,
    pub day: u64,
    pub action: String,
    pub why: String,
}

/// The log's actions played on today's rules ([`replay_today`]).
#[derive(Debug, Clone, Serialize)]
pub struct Today {
    pub applied: u64,
    pub skips: Vec<Skip>,
    /// Where the best-effort career ended: tick, run, retirements.
    pub end: (u64, u32, u32),
}

/// An action as a player reads it: `Buy Smelter`, `Retire`, `Set program ...`.
fn describe(a: &Action) -> String {
    let c = hh_core::content();
    let v = serde_json::to_value(a).unwrap_or(Value::Null);
    let ty = v.get("type").and_then(Value::as_str).unwrap_or("action");
    let mut ty = ty.replace('_', " ");
    if let Some(f) = ty.get_mut(0..1) {
        f.make_ascii_uppercase();
    }
    let arg = v.as_object().and_then(|o| o.iter().find(|(k, _)| *k != "type").map(|(_, x)| x.clone()));
    match (a, arg) {
        (Action::Buy { building }, _) => {
            format!("Buy {}", c.buildings.iter().find(|b| b.id == *building).map_or(building.as_str(), |b| b.name.as_str()))
        }
        (Action::Research { research }, _) => {
            format!("Research {}", c.research.iter().find(|r| r.id == *research).map_or(research.as_str(), |r| r.name.as_str()))
        }
        (_, Some(Value::String(s))) => format!("{ty} {s}"),
        _ => ty,
    }
}

/// Why `a` was refused, with a purchase's shortfall (`22 credits short`) where the
/// refusal was its price.
fn why(s: &Session, a: &Action, e: &str) -> String {
    let credits = |price: hh_core::Fx| {
        let have = s.state.amount(hh_core::modules::economy::CREDITS);
        format!("{} credits short", (price - have).to_f64().ceil())
    };
    if let (Action::BuyLand, true) = (a, e.ends_with("can't afford it")) {
        return credits(hh_core::modules::economy::land_price(&s.rules, &s.state));
    }
    if let (Action::Buy { building }, true) = (a, e.ends_with("can't afford it")) {
        if let Some(b) = hh_core::content().buildings.iter().find(|b| b.id == *building) {
            let short: Vec<String> = hh_core::modules::economy::price(&s.rules, &s.state, b)
                .iter()
                .filter(|(r, c)| s.state.amount(r) < **c)
                .map(|(r, c)| format!("{} {r} short", (*c - s.state.amount(r)).to_f64().ceil()))
                .collect();
            if !short.is_empty() {
                return short.join(", ");
            }
        }
    }
    e.to_string()
}

/// Best-effort replay (Matt, 2026-11-27): the log's actions on today's rules and its
/// (tolerantly loaded) config, each at its logged tick; an action today's game refuses
/// (or this build can't read) is skipped and noted, and play goes on. The first skip is
/// where the log stops working on today's rules. Exact replay at the log's own revision
/// is `scripts/baseline.py`'s.
pub fn replay_today(log: &Log) -> Result<Today, String> {
    let mut s = Session::new(&log.config, log.seed, log.scenario.as_ref())?;
    let mut skips: Vec<Skip> = Vec::new();
    let mut unread = log.unreadable.iter().peekable();
    let mut applied = 0u64;
    let note = |s: &Session, skips: &mut Vec<Skip>, action: String, why: String| {
        let g = &s.state;
        skips.push(Skip { tick: g.tick, run: g.career.run_number, day: g.run.day, action, why });
    };
    for st in &log.actions {
        if st.tick > s.state.tick {
            while let Some((t, e)) = unread.peek().filter(|(t, _)| *t <= st.tick).cloned() {
                s.advance(t - s.state.tick);
                note(&s, &mut skips, "an action".into(), format!("unreadable on this build ({e})"));
                unread.next();
            }
            s.advance(st.tick - s.state.tick);
        }
        match s.apply(&st.action) {
            Ok(()) => applied += 1,
            Err(e) => {
                let w = why(&s, &st.action, &e);
                note(&s, &mut skips, describe(&st.action), w);
            }
        }
    }
    for (t, e) in unread {
        s.advance(t.saturating_sub(s.state.tick));
        note(&s, &mut skips, "an action".into(), format!("unreadable on this build ({e})"));
    }
    s.advance(log.last_tick.saturating_sub(s.state.tick));
    Ok(Today { applied, skips, end: (s.state.tick, s.state.career.run_number, s.state.career.retirements) })
}

/// The best-effort replay's report: the first point the log stops working, then the
/// skips by action.
pub fn today_report(path: &str, log: &Log, r: &Today) -> String {
    let mut t = String::new();
    let _ = writeln!(
        t,
        "{path}: best-effort replay on today's rules (revision {RULES_REVISION}; the log's {}), {} of {} actions applied",
        log.rules_revision,
        r.applied,
        log.actions.len() + log.unreadable.len()
    );
    match r.skips.first() {
        None => {
            let _ = writeln!(t, "Every action applied: the log still plays on today's rules.");
        }
        Some(k) => {
            let _ = writeln!(t, "First stops working: run {} day {} (tick {}): {} skipped, {}", k.run, k.day, k.tick, k.action, k.why);
            let mut by: BTreeMap<&str, u64> = BTreeMap::new();
            for k in &r.skips {
                *by.entry(k.action.as_str()).or_default() += 1;
            }
            let _ = writeln!(
                t,
                "{} skipped in all: {}",
                r.skips.len(),
                by.iter().map(|(a, n)| format!("{a} {n}")).collect::<Vec<_>>().join(", ")
            );
            for k in r.skips.iter().skip(1).take(9) {
                let _ = writeln!(t, "- run {} day {}: {} skipped, {}", k.run, k.day, k.action, k.why);
            }
        }
    }
    let (tick, run, ret) = r.end;
    let _ = writeln!(t, "Ended at tick {tick}: run {run}, {ret} retirements (the log's own: tick {}).", log.last_tick);
    if !log.warnings.is_empty() {
        let _ = writeln!(t, "\nThe log's config, laid over today's ({} keys):", log.warnings.len());
        for w in &log.warnings {
            let _ = writeln!(t, "warning: {w}");
        }
    }
    t
}

/// One line on the log's play record: what it holds, or that the log has none.
pub fn record_line(log: &Log) -> String {
    match &log.record {
        None => "play record: none (logs written before 135 keep none)".to_string(),
        Some(r) => format!(
            "play record: {} snapshots, {} milestones, {} landmarks, {} runs ended{}",
            r.snapshots,
            r.milestones.len(),
            r.landmarks.len(),
            r.runs.len(),
            match r.start {
                Some((_, run, day, true)) => format!("; begun mid-career at run {run} day {day} (a save from before records)"),
                _ => String::new(),
            }
        ),
    }
}

fn mins(s: f64) -> String {
    format!("{:.1}", s / 60.0)
}

/// The report: the log's stamp, whether the replay reproduces it, the
/// ladder's tables for this one career, and the time use.
pub fn report(path: &str, log: &Log, r: &Replayed, milestones: Milestones, humans: Humans) -> String {
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
    if log.accelerated {
        let _ = writeln!(t, "ACCELERATED: an accelerator (boost, autopilot or debug speed) was used: not a human's pace");
    }
    for w in &log.warnings {
        let _ = writeln!(t, "warning: {w}");
    }
    let _ = writeln!(t, "{}", record_line(log));
    let cell = Cell::new(&log.config.name, "human", "human");
    let sum = experiment::summarize(&cell, std::slice::from_ref(&r.game), &milestones, &[("log".into(), None)]);
    let spec = Spec {
        name: "playlog".into(),
        note: String::new(),
        configs: Vec::new(),
        variants: Vec::new(),
        agents: Vec::new(),
        humans: vec!["log".into()],
        seeds: 1,
        first_seed: log.seed,
        retirements: r.game.retirements,
        max_ticks: log.last_tick,
        scenario: None,
        objectives: None,
    };
    let loaded = Loaded { spec, humans, milestones, objectives: Vec::new() };
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
