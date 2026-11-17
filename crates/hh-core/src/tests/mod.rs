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
const GOLDEN_REVISION: u32 = 5;
const GOLDEN_FINGERPRINT: &str = "fb2975e112b43ace";
const GOLDEN_CONTENT_HASH: &str = "aeb05f3e771182b1";

pub(crate) fn golden_session() -> Session {
    let mut s = Session::new(&baseline(), GOLDEN_SEED, None).unwrap();
    play(&mut s, GOLDEN_TICKS, GOLDEN_POLICY);
    s
}

#[test]
fn golden_game() {
    let s = golden_session();
    assert_eq!(s.state.tick, GOLDEN_TICKS);
    let c = &s.state.career;
    assert!(c.completed_quests.contains("qshipment_proof_of_concept"), "the golden game ships");
    assert!(c.retirements >= 1, "the golden game retires");
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
    // The smoke plays the golden game through the wasm build's own copy of
    // `driver.rs` (`play_golden`), then checks its record's seed and length.
    assert_eq!(GOLDEN_POLICY, crate::Policy::Script);
    for line in [
        "m.play_golden(".to_string(),
        format!("const SEED = '{GOLDEN_SEED}';"),
        format!("const TICKS = {GOLDEN_TICKS};"),
    ] {
        assert!(smoke.contains(&line), "scripts/wasm-smoke.mjs lacks `{line}`");
    }
}
