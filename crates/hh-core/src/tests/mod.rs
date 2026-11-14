//! hh-core's tests. `golden_game` pins one fixed game (README, *Rules changes*).

mod config;
mod fx;
mod play;
mod rng;

use crate::driver::{play, GOLDEN_POLICY, GOLDEN_SEED, GOLDEN_TICKS};
use crate::{content_hash, Config, Session, RULES_REVISION};

pub(crate) const BASELINE: &str = include_str!("../../../../configs/baseline.json");

pub(crate) fn baseline() -> Config {
    Config::from_json(BASELINE).expect("baseline loads")
}

// The golden constants: update all three together on a `RULES_REVISION` bump
// (and `GOLDEN_*` in scripts/wasm-smoke.mjs), copying the failing assertion's
// left side.
const GOLDEN_REVISION: u32 = 1;
const GOLDEN_FINGERPRINT: &str = "36d8cc14b1f9de2a";
const GOLDEN_CONTENT_HASH: &str = "8f392297a9a3af48";

pub(crate) fn golden_session() -> Session {
    let mut s = Session::new(&baseline(), GOLDEN_SEED, None).unwrap();
    play(&mut s, GOLDEN_TICKS, GOLDEN_POLICY);
    s
}

#[test]
fn golden_game() {
    let s = golden_session();
    assert_eq!(s.state.tick, GOLDEN_TICKS);
    assert!(s.state.owned("solar_panel") > 0, "the golden game buys something");
    assert_eq!(
        (RULES_REVISION, s.state.fingerprint().as_str()),
        (GOLDEN_REVISION, GOLDEN_FINGERPRINT)
    );
    assert_eq!(content_hash(), GOLDEN_CONTENT_HASH);
}

#[test]
fn golden_game_replays() {
    let s = golden_session();
    let rec = s.record();
    let (g, warnings) = crate::Record::from_json(&rec.to_json()).unwrap().replay().unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(g, s.state);
}

#[test]
fn wasm_smoke_expects_the_golden_constants() {
    let smoke = include_str!("../../../../scripts/wasm-smoke.mjs");
    assert!(
        smoke.contains(&format!("'{GOLDEN_FINGERPRINT}'")),
        "scripts/wasm-smoke.mjs holds another golden fingerprint"
    );
    assert!(
        smoke.contains(&format!("'{GOLDEN_CONTENT_HASH}'")),
        "scripts/wasm-smoke.mjs holds another content hash"
    );
    assert!(
        smoke.contains(&format!("GOLDEN_REVISION = {GOLDEN_REVISION};")),
        "scripts/wasm-smoke.mjs holds another golden revision"
    );
    // The golden game's definition, which the smoke script mirrors.
    let crate::Policy::Greedy { every } = GOLDEN_POLICY else {
        panic!("the smoke script mirrors a greedy golden policy only")
    };
    for line in [
        format!("const SEED = '{GOLDEN_SEED}';"),
        format!("const TICKS = {GOLDEN_TICKS};"),
        format!("const EVERY = {every};"),
    ] {
        assert!(smoke.contains(&line), "scripts/wasm-smoke.mjs lacks `{line}`");
    }
}
