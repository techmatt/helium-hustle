//! Thin wasm-bindgen layer over `hh-core`. No rules live here.
//!
//! Configs, states, actions, views and records cross the boundary as JSON
//! strings; seeds are decimal strings (any `u64`). Errors (bad JSON, a bad
//! seed, an invalid config, an illegal action) are thrown as JavaScript
//! strings. A state carries its config, so calls after `new_game` take only it.

use std::cell::RefCell;

use hh_core::{Action, Config, GameState, Record, Rules, Scenario};
use serde::Serialize;
use wasm_bindgen::prelude::*;

fn to_json<T: Serialize + ?Sized>(value: &T) -> String {
    serde_json::to_string(value).expect("engine types serialize")
}

fn err(e: String) -> JsValue {
    JsValue::from_str(&e)
}

fn parse_seed(seed: &str) -> Result<u64, JsValue> {
    seed.trim()
        .parse()
        .map_err(|_| err(format!("bad seed {seed:?}: expected a decimal u64 string")))
}

fn config(json: &str) -> Result<Config, JsValue> {
    Config::from_json(json).map_err(err)
}

// Two caches, so a page wake (advance_run, then view on the state it returned) neither
// rebuilds the rules nor parses the state twice (094: about a third of a wake's engine
// time). Neither changes a result: the rules are rebuilt from an equal config, and a
// state's JSON parses back to an equal state (`cached_state_matches_a_fresh_parse`).
thread_local! {
    /// The rules last built, reused while states carry an equal config.
    static RULES: RefCell<Option<Rules>> = const { RefCell::new(None) };
    /// The last state handed out as JSON, with that JSON: a call on the same string
    /// takes a copy instead of parsing it.
    static LAST: RefCell<Option<(String, GameState)>> = const { RefCell::new(None) };
}

/// The rules for a state's config, built once per config.
fn rules_for(g: &GameState) -> Result<Rules, JsValue> {
    RULES.with_borrow_mut(|cached| {
        if let Some(r) = cached.as_ref().filter(|r| *r.config == *g.config) {
            return Ok(r.clone());
        }
        let r = Rules::for_state(g).map_err(err)?;
        *cached = Some(r.clone());
        Ok(r)
    })
}

/// A state's JSON, kept with the state for the next call on that string.
fn hand_out(g: GameState) -> String {
    let json = g.to_json();
    LAST.with_borrow_mut(|last| *last = Some((json.clone(), g)));
    json
}

/// A state and the rules it plays under, checked to fit.
fn load(state_json: &str) -> Result<(Rules, GameState), JsValue> {
    let cached = LAST.with_borrow(|last| last.as_ref().filter(|(j, _)| j == state_json).map(|(_, g)| g.clone()));
    let g = match cached {
        Some(g) => g,
        None => GameState::from_json(state_json).map_err(err)?,
    };
    let r = rules_for(&g)?;
    r.check_state(&g).map_err(err)?;
    Ok((r, g))
}

#[wasm_bindgen]
pub fn version() -> String {
    hh_core::VERSION.to_string()
}

/// `hh_core::RULES_REVISION`: a save from another revision is discarded.
#[wasm_bindgen]
pub fn rules_revision() -> u32 {
    hh_core::RULES_REVISION
}

#[wasm_bindgen]
pub fn content_hash() -> String {
    hh_core::content_hash()
}

/// Checks a config. Returns `{"config": <resolved>, "hash": "<16 hex>"}`.
#[wasm_bindgen]
pub fn resolve_config(config_json: &str) -> Result<String, JsValue> {
    let c = config(config_json)?;
    Ok(to_json(&serde_json::json!({ "hash": c.hash(), "config": c })))
}

/// The embedded scenarios: `[{name, description, start}]`.
#[wasm_bindgen]
pub fn scenarios() -> String {
    to_json(hh_core::scenarios())
}

/// A new game at tick 0, or from an embedded scenario when `scenario` is
/// non-empty.
#[wasm_bindgen]
pub fn new_game(config_json: &str, seed: &str, scenario: &str) -> Result<String, JsValue> {
    let r = Rules::new(&config(config_json)?).map_err(err)?;
    let s: Option<&Scenario> = if scenario.is_empty() {
        None
    } else {
        Some(hh_core::scenario(scenario).map_err(err)?)
    };
    Ok(hand_out(r.start(parse_seed(seed)?, s)))
}

/// Runs `ticks` ticks; returns the new state.
#[wasm_bindgen]
pub fn advance(state_json: &str, ticks: u32) -> Result<String, JsValue> {
    let (r, mut g) = load(state_json)?;
    r.advance(&mut g, ticks as u64);
    Ok(hand_out(g))
}

/// Runs up to `ticks` ticks, stopping after a tick that retired the run
/// (forced) or changed the active quest, so the client can show the retirement
/// or the story modal before playing on. Returns `{"state": <the new state's
/// JSON, as a string>, "lines": <the program lines run over the ticks,
/// hh_core::view::LinesRun>, "stopped": <it stopped early>, "ran": <the ticks
/// run>}`: a client
/// drawing once per several days colors each line by the whole span, not by
/// its last day. The ticks run are the state's `tick` minus the old one.
#[wasm_bindgen]
pub fn advance_run(state_json: &str, ticks: u32) -> Result<String, JsValue> {
    let (r, mut g) = load(state_json)?;
    let run = g.career.run_number;
    let quest = active_quest_id(&g);
    let mut lines = hh_core::view::LinesRun::default();
    let mut stopped = false;
    let mut ran = 0u32;
    for _ in 0..ticks {
        r.advance(&mut g, 1);
        ran += 1;
        lines.note(&g);
        if g.career.run_number != run || active_quest_id(&g) != quest {
            stopped = true;
            break;
        }
    }
    #[derive(Serialize)]
    struct Advanced {
        state: String,
        lines: hh_core::view::LinesRun,
        stopped: bool,
        ran: u32,
    }
    Ok(to_json(&Advanced { state: hand_out(g), lines, stopped, ran }))
}

/// The active quest's id: a change is a main-story transition, whose modal pauses the
/// game on the day it happened (a fast speed's wake would otherwise play on past it).
fn active_quest_id(g: &GameState) -> Option<&'static str> {
    hh_core::modules::events::active_quest(g).map(|e| e.id.as_str())
}

/// Applies an action (`{"type": "buy", "building": "solar_panel"}`) between
/// ticks; returns the new state or throws why it can't.
#[wasm_bindgen]
pub fn apply(state_json: &str, action_json: &str) -> Result<String, JsValue> {
    let (r, mut g) = load(state_json)?;
    let a: Action =
        serde_json::from_str(action_json).map_err(|e| err(format!("bad action JSON: {e}")))?;
    r.apply(&mut g, &a).map_err(err)?;
    Ok(hand_out(g))
}

/// A program's lines (a JSON entries array) as the text editor's text
/// (`hh_core::program_text::to_text`).
#[wasm_bindgen]
pub fn program_text(entries_json: &str) -> Result<String, JsValue> {
    let entries: Vec<hh_core::state::Entry> =
        serde_json::from_str(entries_json).map_err(|e| err(format!("bad entries JSON: {e}")))?;
    Ok(hh_core::program_text::to_text(&entries))
}

/// Reads the text editor's text for the state's run: `{"entries": [...],
/// "lines": [{"entry", "status", "reason"}]}`, one `lines` item per text
/// line (`hh_core::program_text::parse`).
#[wasm_bindgen]
pub fn parse_program_text(state_json: &str, text: &str) -> Result<String, JsValue> {
    let (r, g) = load(state_json)?;
    Ok(to_json(&hh_core::program_text::parse(&r, &g, text)))
}

/// The text editor's instruction panel for the state's run, `cursor` the text
/// line the caret is on: `[{"name", "items": [{"text", "locked", "card"}]}]`
/// (`hh_core::program_text::instructions`).
#[wasm_bindgen]
pub fn program_instructions(state_json: &str, text: &str, cursor: u32) -> Result<String, JsValue> {
    let (r, g) = load(state_json)?;
    Ok(to_json(&hh_core::program_text::instructions(&r, &g, text, cursor as usize)))
}

/// Display data (`hh_core::View`, docs/architecture.md *The view*): everything a
/// client shows and can do.
#[wasm_bindgen]
pub fn view(state_json: &str) -> Result<String, JsValue> {
    let (r, g) = load(state_json)?;
    Ok(to_json(&r.view(&g)))
}

/// The state's fingerprint (16 hex digits).
#[wasm_bindgen]
pub fn fingerprint(state_json: &str) -> Result<String, JsValue> {
    Ok(load(state_json)?.1.fingerprint())
}

/// The state's config hash (what a save is tagged with).
#[wasm_bindgen]
pub fn config_hash(state_json: &str) -> Result<String, JsValue> {
    Ok(load(state_json)?.0.config_hash)
}

/// Plays the golden game (`hh_core::driver`: the golden seed, length and
/// policy, on `config_json`) in wasm. Returns `{"fingerprint", "config_hash",
/// "record"}`: the parity check compares the fingerprint and config hash with
/// the native pins and replays the record through this API.
#[wasm_bindgen]
pub fn play_golden(config_json: &str) -> Result<String, JsValue> {
    let c = config(config_json)?;
    let s = hh_core::driver::golden_session(&c).map_err(err)?;
    Ok(to_json(&serde_json::json!({
        "fingerprint": s.state.fingerprint(),
        "config_hash": c.hash(),
        "record": s.record(),
    })))
}

/// The debug autopilot (`hh_sim::autopilot`, an accelerator): a sim player
/// plays the client's game on, honestly, toward a target. The client calls
/// [`Autopilot::step`] in chunks (so the page stays live), then takes the state
/// and the stamped actions to append to its record.
#[wasm_bindgen]
pub struct Autopilot {
    ap: hh_sim::autopilot::Autopilot,
    /// Actions already handed out by [`Autopilot::take_actions`].
    taken: usize,
}

#[wasm_bindgen]
impl Autopilot {
    /// `target`: `milestone`, `quest` or `end`; `agent`: a sim player's spec
    /// (empty: the default); `boost`: `production,research,boredom` (empty: no
    /// boost action); `budget`: ticks before it gives up; `met_json`: the step
    /// ids (`["milestone:M1", ...]`) already met in the career, kept by the
    /// client from earlier runs of the autopilot.
    #[wasm_bindgen(constructor)]
    pub fn new(
        state_json: &str,
        target: &str,
        agent: &str,
        boost: &str,
        budget: u32,
        met_json: &str,
    ) -> Result<Autopilot, JsValue> {
        use hh_sim::autopilot as a;
        let g = GameState::from_json(state_json).map_err(err)?;
        let ms = hh_sim::milestones::Milestones::from_json(include_str!("../../../configs/sim/milestones.json"))
            .map_err(err)?;
        let target = a::Target::parse(target).map_err(err)?;
        let agent = if agent.trim().is_empty() { a::DEFAULT_AGENT } else { agent.trim() };
        let boost = if boost.trim().is_empty() { None } else { Some(a::parse_boost(boost.trim()).map_err(err)?) };
        let met: Vec<String> = serde_json::from_str(met_json).map_err(|e| err(format!("bad met JSON: {e}")))?;
        let ap = a::Autopilot::new(g, ms, agent, target, boost, budget as u64, &met).map_err(err)?;
        Ok(Autopilot { ap, taken: 0 })
    }

    /// Plays up to `ticks` more; returns its status (`hh_sim::autopilot::Status`).
    pub fn step(&mut self, ticks: u32) -> String {
        to_json(&self.ap.step(ticks as u64))
    }

    pub fn state(&self) -> String {
        self.ap.state().to_json()
    }

    /// The stamped actions (`[{tick, action}]`) applied since the last call.
    pub fn take_actions(&mut self) -> String {
        let all = self.ap.actions();
        let new = &all[self.taken..];
        self.taken = all.len();
        to_json(new)
    }
}

/// The autopilot's defaults and every step it knows (milestones, then
/// main-quest steps): `{"agent", "boost", "steps": [{id, name}]}`.
#[wasm_bindgen]
pub fn autopilot_info() -> Result<String, JsValue> {
    let ms = hh_sim::milestones::Milestones::from_json(include_str!("../../../configs/sim/milestones.json"))
        .map_err(err)?;
    Ok(to_json(&serde_json::json!({
        "agent": hh_sim::autopilot::DEFAULT_AGENT,
        "boost": hh_sim::autopilot::DEFAULT_BOOST,
        "steps": hh_sim::autopilot::steps(&ms),
    })))
}

/// Replays a record (`hh_core::Record` JSON). Returns
/// `{"state": <state>, "fingerprint": "...", "warnings": [...]}` or throws.
#[wasm_bindgen]
pub fn replay(record_json: &str) -> Result<String, JsValue> {
    let rec = Record::from_json(record_json).map_err(err)?;
    let (g, warnings) = rec.replay().map_err(err)?;
    Ok(to_json(&serde_json::json!({
        "fingerprint": g.fingerprint(),
        "warnings": warnings,
        "state": g,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASELINE: &str = include_str!("../../../configs/baseline.json");

    /// The state cache's premise: a state's JSON parses back to an equal state, early,
    /// late and across a retirement (the golden game).
    #[test]
    fn cached_state_matches_a_fresh_parse() {
        let c = Config::from_json(BASELINE).unwrap();
        let r = Rules::new(&c).unwrap();
        let mut states = Vec::new();
        let mut g = r.start(1, None);
        for _ in 0..12 {
            r.advance(&mut g, 50);
            states.push(g.clone());
        }
        let mut late = r.start(1, Some(hh_core::scenario("everything").unwrap()));
        r.advance(&mut late, 60);
        states.push(late);
        states.push(hh_core::driver::golden_session(&c).unwrap().state);
        for g in states {
            assert_eq!(GameState::from_json(&g.to_json()).unwrap(), g, "tick {}", g.tick);
        }
    }

    /// A wake through the caches reads as one through fresh parses.
    #[test]
    fn cached_calls_match_uncached_ones() {
        let s = new_game(BASELINE, "1", "everything").unwrap();
        let a = advance_run(&s, 20).unwrap();
        let a: serde_json::Value = serde_json::from_str(&a).unwrap();
        let next = a["state"].as_str().unwrap().to_string();
        let ran = a["ran"].as_u64().unwrap();
        let cached = view(&next).unwrap();
        LAST.with_borrow_mut(|l| *l = None);
        RULES.with_borrow_mut(|r| *r = None);
        assert_eq!(view(&next).unwrap(), cached);
        let fresh = {
            let g = GameState::from_json(&s).unwrap();
            let r = Rules::for_state(&g).unwrap();
            let mut g2 = g.clone();
            r.advance(&mut g2, ran);
            g2.to_json()
        };
        assert_eq!(next, fresh);
    }
}
