//! Thin wasm-bindgen layer over `hh-core`. No rules live here.
//!
//! Configs, states, actions, views and records cross the boundary as JSON
//! strings; seeds are decimal strings (any `u64`). Errors (bad JSON, a bad
//! seed, an invalid config, an illegal action) are thrown as JavaScript
//! strings. A state carries its config, so calls after `new_game` take only it.

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

/// A state and the rules it plays under, checked to fit.
fn load(state_json: &str) -> Result<(Rules, GameState), JsValue> {
    let g = GameState::from_json(state_json).map_err(err)?;
    let r = Rules::for_state(&g).map_err(err)?;
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
    Ok(to_json(&r.start(parse_seed(seed)?, s)))
}

/// Runs `ticks` ticks; returns the new state.
#[wasm_bindgen]
pub fn advance(state_json: &str, ticks: u32) -> Result<String, JsValue> {
    let (r, mut g) = load(state_json)?;
    r.advance(&mut g, ticks as u64);
    Ok(g.to_json())
}

/// Runs up to `ticks` ticks, stopping after a tick that retired the run
/// (forced), so the client can show the retirement before the next run plays
/// on. Returns `{"state": <the new state's JSON, as a string>, "lines": <the
/// program lines run over the ticks, hh_core::view::LinesRun>}`: a client
/// drawing once per several days colors each line by the whole span, not by
/// its last day. The ticks run are the state's `tick` minus the old one.
#[wasm_bindgen]
pub fn advance_run(state_json: &str, ticks: u32) -> Result<String, JsValue> {
    let (r, mut g) = load(state_json)?;
    let run = g.career.run_number;
    let mut lines = hh_core::view::LinesRun::default();
    for _ in 0..ticks {
        r.advance(&mut g, 1);
        lines.note(&g);
        if g.career.run_number != run {
            break;
        }
    }
    #[derive(Serialize)]
    struct Advanced {
        state: String,
        lines: hh_core::view::LinesRun,
    }
    Ok(to_json(&Advanced { state: g.to_json(), lines }))
}

/// Applies an action (`{"type": "buy", "building": "solar_panel"}`) between
/// ticks; returns the new state or throws why it can't.
#[wasm_bindgen]
pub fn apply(state_json: &str, action_json: &str) -> Result<String, JsValue> {
    let (r, mut g) = load(state_json)?;
    let a: Action =
        serde_json::from_str(action_json).map_err(|e| err(format!("bad action JSON: {e}")))?;
    r.apply(&mut g, &a).map_err(err)?;
    Ok(g.to_json())
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
/// policy, on `config_json`) in wasm. Returns `{"fingerprint", "record"}`:
/// the parity check compares the fingerprint with the native pin and replays
/// the record through this API.
#[wasm_bindgen]
pub fn play_golden(config_json: &str) -> Result<String, JsValue> {
    use hh_core::driver::{play, GOLDEN_POLICY, GOLDEN_SEED, GOLDEN_TICKS};
    let mut s = hh_core::Session::new(&config(config_json)?, GOLDEN_SEED, None).map_err(err)?;
    play(&mut s, GOLDEN_TICKS, GOLDEN_POLICY);
    Ok(to_json(&serde_json::json!({
        "fingerprint": s.state.fingerprint(),
        "record": s.record(),
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
