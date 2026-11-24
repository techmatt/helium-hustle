//! hh-core's tests. `golden_game` pins one fixed game (README, *Rules changes*).

mod config;
mod fx;
mod logic;
mod play;
mod rng;
mod text;
mod timeline;
mod verbs;

use crate::driver::{play, GOLDEN_POLICY, GOLDEN_SEED, GOLDEN_TICKS};
use crate::{content_hash, Config, Session, RULES_REVISION};

#[test]
fn the_golden_starts_with_if_open() {
    let s = crate::driver::golden_start(&baseline()).unwrap();
    assert!(s.state.career.flags.contains(crate::modules::programs::IF_FLAG));
    assert_eq!(s.state.tick, 0);
}

pub(crate) const BASELINE: &str = include_str!("../../../../configs/baseline.json");

pub(crate) fn baseline() -> Config {
    Config::from_json(BASELINE).expect("baseline loads")
}

pub(crate) fn rules() -> crate::Rules {
    crate::Rules::new(&baseline()).unwrap()
}

pub(crate) fn f(s: &str) -> crate::Fx {
    crate::Fx::parse(s).unwrap()
}

/// Sets `res`'s amount in the run. Caps don't bind here but in the day's
/// clamp: a new game's caps are each resource's `cap` in
/// `content/resources.json` (energy 100, regolith 50, ice and propellant 30,
/// titanium and He-3 20, circuits 10) plus storage buildings, so a `set` past
/// one is cut at the next tick's end. Energy in a brownout can sit a few
/// 10^-6 above 0, not at 0: assert it with a tolerance.
pub(crate) fn set(g: &mut crate::GameState, res: &str, v: &str) {
    g.run.resources.insert(res.into(), f(v));
}

/// Program 1's line outcomes today (`None`: no outcome: not reached, a note,
/// or a locked line skipped free). The step-rule tests read days with it.
pub(crate) fn outcomes(g: &crate::GameState) -> Vec<Option<crate::state::LineOutcome>> {
    crate::view::LinesRun::today(g).lines[0].clone()
}

// The golden constants: update all four together on a `RULES_REVISION` bump
// with `python scripts/set_golden.py` (it also sets `GOLDEN_*` in
// scripts/wasm-smoke.mjs); never type them (README, *Rules changes*).
const GOLDEN_REVISION: u32 = 25;
const GOLDEN_FINGERPRINT: &str = "ed42f770642dba38";
const GOLDEN_CONTENT_HASH: &str = "e5d15b56e0adc484";
const GOLDEN_CONFIG_HASH: &str = "c6f158e1d0485a48";

pub(crate) fn golden_session() -> Session {
    crate::driver::golden_session(&baseline()).unwrap()
}

#[test]
fn golden_game() {
    let s = golden_session();
    assert_eq!(s.state.tick, GOLDEN_TICKS);
    let c = &s.state.career;
    assert!(c.completed_quests.contains("qfirst_shipment"), "the golden game ships");
    assert!(c.retirements >= 1, "the golden game retires");
    assert_eq!(
        (RULES_REVISION, s.state.fingerprint().as_str()),
        (GOLDEN_REVISION, GOLDEN_FINGERPRINT)
    );
    assert_eq!(content_hash(), GOLDEN_CONTENT_HASH);
    assert_eq!(baseline().hash(), GOLDEN_CONFIG_HASH);
}

/// The golden script's condition program (`driver::guard_lines`) does its work
/// in the golden game (which starts with `if` open, the `golden_start`
/// scenario): each guard both holds and fails, the `assign` and the `build`
/// both run, and the `build` is also done (two pads owned), so the
/// fingerprint pins the interpreter and the readings, not only unit tests.
#[test]
fn golden_game_runs_its_condition_program() {
    use crate::state::{LineOutcome as O, Reading};
    use std::collections::BTreeSet;
    let mut s = crate::driver::golden_start(&baseline()).unwrap();
    let mut seen: [BTreeSet<String>; 4] = Default::default();
    let mut readings = BTreeSet::new();
    for _ in 0..GOLDEN_TICKS {
        play(&mut s, 1, GOLDEN_POLICY);
        let program = &s.state.run.programs[0];
        if program.entries.len() == 5 {
            for e in &program.entries {
                if let crate::state::Op::If { when } = &e.op {
                    readings.insert(when.first.reading.describe());
                }
            }
            for (&line, &o) in s.state.run.today.lines.get(&0).into_iter().flatten() {
                if let Some(set) = seen.get_mut(line as usize) {
                    set.insert(format!("{o:?}"));
                }
            }
        }
    }
    let has = |line: usize, o: O| seen[line].contains(&format!("{o:?}"));
    assert!(readings.contains(&Reading::FreeProcessors.describe()), "{readings:?}");
    assert!(readings.contains(&Reading::IdlePads.describe()), "{readings:?}");
    for (line, outcomes) in [(0, &[O::True, O::False][..]), (1, &[O::Ran]), (2, &[O::True, O::False]), (3, &[O::Ran, O::Done])] {
        for &o in outcomes {
            assert!(has(line, o), "line {line} never {o:?}: {seen:?}");
        }
    }
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
        smoke.contains(&format!("'{GOLDEN_CONFIG_HASH}'")),
        "scripts/wasm-smoke.mjs holds another config hash"
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
