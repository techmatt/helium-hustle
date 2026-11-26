//! Saves, records and replay, scenarios, and starts at the edges of the number range.

use super::support::{buy, program};
use super::{baseline, f, rules};
use crate::driver::{play, Policy};
use crate::{scenario, scenarios, Fx, GameState, Record, Rules, Scenario, Session};

#[test]
fn a_late_scenario_schedules_from_its_own_day() {
    let r = rules();
    let s = Scenario::from_json(
        r#"{"name":"x","description":"","start":{"tick":1000,"resources":{},"buildings":{},"panels":[],"programs":[]}}"#,
    )
    .unwrap();
    let g = r.start(1, Some(&s));
    assert!(g.run.next_burst_day > 1000);
    assert!(g.run.rival_next.values().all(|&d| d > 1000));
}

#[test]
fn the_everything_scenario_opens_every_act_1_view() {
    let r = rules();
    let mut g = r.start(13, Some(scenario("everything").unwrap()));
    r.advance(&mut g, 1);
    let v = r.view(&g);
    let shown: Vec<&str> = v.nav.iter().filter(|t| t.visible).map(|t| t.id).collect();
    for id in ["buildings", "commands", "launch_pads", "research", "projects", "ideologies", "adversaries", "stats", "story", "retirement", "options"] {
        assert!(shown.contains(&id), "{id} hidden: {shown:?}");
    }
    assert!(!shown.contains(&"timeline"), "Act 2 stays shut");
    assert!(crate::modules::career::flag_set(&g, "logic_if"));
    assert!(g.run.research.contains("advanced_blueprints"), "the Blueprint Archive's");
    assert_eq!(crate::modules::events::active_quest(&g).map(|e| e.id.as_str()), Some("qdeep_space_telescope"));
    assert!(g.amount("processors") >= f("6"));
    assert!(!g.cheated, "a scenario is no cheat");
    r.advance(&mut g, 30);
}

#[test]
fn a_saved_state_reloads_to_the_same_future() {
    let mut s = Session::new(&baseline(), 9, None).unwrap();
    play(&mut s, 400, Policy::Script);
    let json = s.state.to_json();
    let mut a = s.state.clone();
    let mut b = GameState::from_json(&json).unwrap();
    assert_eq!(a, b);
    let r = Rules::for_state(&b).unwrap();
    r.check_state(&b).unwrap();
    r.advance(&mut a, 500);
    r.advance(&mut b, 500);
    assert_eq!(a.fingerprint(), b.fingerprint());
}

#[test]
fn records_replay_exactly_from_a_scenario() {
    let mut s = Session::new(&baseline(), 42, Some(scenario("rich").unwrap())).unwrap();
    play(&mut s, 300, Policy::Script);
    let rec = Record::from_json(&s.record().to_json()).unwrap();
    let (g, warnings) = rec.replay().unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(g, s.state);
}

#[test]
fn a_tampered_record_warns_or_fails() {
    let mut s = Session::new(&baseline(), 2, None).unwrap();
    play(&mut s, 100, Policy::Script);
    let mut rec = s.record();
    rec.fingerprint = Some("0123456789abcdef".into());
    assert_eq!(rec.replay().unwrap().1.len(), 1);
    let mut rec = s.record();
    let last = rec.actions.len() - 1;
    rec.actions[last].tick = 0; // before the action ahead of it
    assert!(rec.replay().unwrap_err().contains("before tick"));
    let mut rec = s.record();
    rec.actions[0].action = buy("fusion");
    assert!(rec.replay().unwrap_err().contains("action 0"));
}

#[test]
fn extreme_starts_neither_panic_nor_wrap() {
    // A price past Fx's range is shown as Fx::MAX and never affordable.
    let s = Scenario::from_json(
        r#"{"name":"x","description":"","start":{"tick":0,"resources":{"credits":"9223372036000"},"buildings":{"solar_panel":4294967295},"panels":[],"programs":[]}}"#,
    )
    .unwrap();
    let r = rules();
    let mut g = r.start(1, Some(&s));
    let v = r.view(&g);
    assert_eq!(v.buildings[1].cost["credits"], Fx::MAX);
    assert!(!v.buildings[1].affordable);
    assert!(r.apply(&mut g, &buy("solar_panel")).is_err());
    // Credits saturate rather than overflow.
    program(&r, &mut g, &[("idle", 1)]);
    g.run.resources.insert("credits".into(), Fx::MAX);
    r.advance(&mut g, 3);
    assert_eq!(g.amount("credits"), Fx::MAX);
}

#[test]
fn a_bad_rng_seed_is_an_error_not_a_panic() {
    let r = rules();
    let json = r.new_game(1).to_json();
    let seed = json.split("\"seed\":\"").nth(2).unwrap()[..64].to_string();
    let bad = json.replace(&seed, &format!("0\u{e9}{}", "0".repeat(61)));
    assert!(GameState::from_json(&bad).is_err());
}

#[test]
fn scenarios_load_and_bad_ones_are_named() {
    assert!(scenarios().len() >= 2);
    let e = Scenario::from_json(
        r#"{"name":"x","description":"","start":{"tick":0,"resources":{"helium":"1"},"buildings":{"mine":1},"panels":["shop"],"programs":[]}}"#,
    )
    .unwrap_err();
    assert!(e.contains("start.resources.helium") && e.contains("start.buildings.mine") && e.contains("\"shop\""), "{e}");
    let e = Scenario::from_json(r#"{"name":"x","description":"","start":{"tick":0,"resources":{}}}"#)
        .unwrap_err();
    assert!(e.contains("buildings"), "{e}");
    assert!(scenario("nope").is_err());
}
