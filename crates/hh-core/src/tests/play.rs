use super::baseline;
use crate::driver::{play, Policy};
use crate::{scenario, scenarios, Action, Fx, GameState, Record, Rules, Scenario, Session};

fn buy() -> Action {
    Action::Buy {
        building: "solar_panel".into(),
    }
}

#[test]
fn a_new_game_starts_from_the_config() {
    let r = Rules::new(&baseline()).unwrap();
    let g = r.new_game(3);
    assert_eq!(g.tick, 0);
    assert_eq!(g.amount("credits"), Fx::from_int(20));
    assert_eq!(g.amount("energy"), Fx::ZERO);
    r.check_state(&g).unwrap();
}

#[test]
fn ticks_add_income_and_the_cap_holds() {
    let mut s = Session::new(&baseline(), 1, Some(scenario("at_cap").unwrap())).unwrap();
    let credits = s.state.amount("credits");
    s.advance(10);
    assert_eq!(s.state.tick, 110);
    assert_eq!(s.state.amount("credits"), credits + Fx::from_int(10));
    assert_eq!(s.state.amount("energy"), Fx::from_int(100));
}

#[test]
fn buying_costs_credits_and_grows_the_price() {
    let r = Rules::new(&baseline()).unwrap();
    let mut g = r.new_game(1);
    r.apply(&mut g, &buy()).unwrap();
    assert_eq!(g.owned("solar_panel"), 1);
    assert_eq!(g.amount("credits"), Fx::from_int(10));
    let price = r.view(&g).buildings[0].cost["credits"];
    assert_eq!(price, Fx::parse("11.5").unwrap());
    // 10 credits can't buy an 11.5-credit panel, and a refused buy changes nothing.
    let before = g.clone();
    assert!(r.apply(&mut g, &buy()).is_err());
    assert_eq!(g, before);
    assert!(!r.view(&g).buildings[0].affordable);
}

#[test]
fn an_unknown_building_is_refused() {
    let r = Rules::new(&baseline()).unwrap();
    let mut g = r.new_game(1);
    let e = r
        .apply(&mut g, &Action::Buy { building: "fusion".into() })
        .unwrap_err();
    assert!(e.contains("unknown building \"fusion\""), "{e}");
}

#[test]
fn panels_make_energy_within_the_sunlight_spread() {
    let r = Rules::new(&baseline()).unwrap();
    let mut g = r.new_game(5);
    r.apply(&mut g, &buy()).unwrap();
    let mut lo = Fx::from_int(10);
    let mut hi = Fx::ZERO;
    for _ in 0..50 {
        let e = g.amount("energy");
        r.advance(&mut g, 1);
        let made = g.amount("energy") - e;
        lo = lo.min(made);
        hi = hi.max(made);
    }
    assert!(lo >= Fx::parse("0.8").unwrap() && hi <= Fx::parse("1.2").unwrap());
    assert!(lo < hi, "sunlight varies");
}

#[test]
fn a_saved_state_reloads_to_the_same_future() {
    let mut s = Session::new(&baseline(), 9, None).unwrap();
    play(&mut s, 300, Policy::Greedy { every: 3 });
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
    play(&mut s, 400, Policy::Greedy { every: 1 });
    assert!(s.state.owned("solar_panel") >= 20);
    let rec = Record::from_json(&s.record().to_json()).unwrap();
    let (g, warnings) = rec.replay().unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(g, s.state);
}

#[test]
fn a_tampered_record_warns_or_fails() {
    let mut s = Session::new(&baseline(), 2, None).unwrap();
    play(&mut s, 100, Policy::Greedy { every: 5 });
    let mut rec = s.record();
    rec.fingerprint = Some("0123456789abcdef".into());
    assert_eq!(rec.replay().unwrap().1.len(), 1);
    let mut rec = s.record();
    rec.actions[1].tick = 0; // before the action ahead of it
    assert!(rec.replay().unwrap_err().contains("before tick"));
    let mut rec = s.record();
    rec.actions[0].action = Action::Buy { building: "fusion".into() };
    assert!(rec.replay().unwrap_err().contains("action 0"));
}

#[test]
fn extreme_starts_neither_panic_nor_wrap() {
    // A price past Fx's range is shown as Fx::MAX and never affordable.
    let s = Scenario::from_json(
        r#"{"name":"x","description":"","start":{"tick":0,"resources":{"credits":"9223372036000"},"buildings":{"solar_panel":4294967295}}}"#,
    )
    .unwrap();
    let r = Rules::new(&baseline()).unwrap();
    let mut g = r.start(1, Some(&s));
    let v = r.view(&g);
    assert_eq!(v.buildings[0].cost["credits"], Fx::MAX);
    assert!(!v.buildings[0].affordable);
    assert!(r.apply(&mut g, &buy()).is_err());
    // Credits saturate rather than overflow.
    r.advance(&mut g, 1_000_000);
    assert_eq!(g.amount("credits"), Fx::MAX);
}

#[test]
fn a_bad_rng_seed_is_an_error_not_a_panic() {
    let r = Rules::new(&baseline()).unwrap();
    let json = r.new_game(1).to_json();
    let seed = json.split("\"seed\":\"").nth(2).unwrap()[..64].to_string();
    let bad = json.replace(&seed, &format!("0\u{e9}{}", "0".repeat(61)));
    assert!(GameState::from_json(&bad).is_err());
}

#[test]
fn scenarios_load_and_bad_ones_are_named() {
    assert!(scenarios().len() >= 2);
    let e = Scenario::from_json(
        r#"{"name":"x","description":"","start":{"tick":0,"resources":{"helium":"1"},"buildings":{"mine":1}}}"#,
    )
    .unwrap_err();
    assert!(e.contains("start.resources.helium") && e.contains("start.buildings.mine"), "{e}");
    let e = Scenario::from_json(r#"{"name":"x","description":"","start":{"tick":0,"resources":{}}}"#)
        .unwrap_err();
    assert!(e.contains("buildings"), "{e}");
    assert!(scenario("nope").is_err());
}
