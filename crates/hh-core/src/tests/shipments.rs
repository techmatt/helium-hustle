//! Shipments: pads, their cargo, launches, demand and the activity feed's shipments.

use super::support::{own, program};
use super::{baseline, f, rules, set};
use crate::driver::{play, Policy};
use crate::modules::economy;
use crate::{Action, Fx, Session};

#[test]
fn shipments_load_launch_pay_and_cool_down() {
    let r = rules();
    let mut g = r.new_game(1);
    own(&mut g, "launch_pad", 1);
    own(&mut g, "solar_panel", 3);
    assert_eq!(g.run.pads.len(), 1);
    set(&mut g, "he3", "20");
    r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some("he3".into()) }).unwrap();
    program(&r, &mut g, &[("load_pads", 1)]);
    r.advance(&mut g, 1);
    assert_eq!(g.run.pads[0].cargo, f("5"));
    assert_eq!(g.amount("he3"), f("15"));
    // A full pad with fuel launches for base value × demand × cargo.
    g.run.pads[0].cargo = f("100");
    set(&mut g, "propellant", "25");
    let pay = r.view(&g).pads[0].payout;
    let demand = g.run.markets["he3"].demand;
    assert_eq!(pay, f("10") * demand * f("100"));
    r.apply(&mut g, &Action::Launch { pad: 0 }).unwrap();
    assert_eq!(g.amount("credits"), pay);
    assert_eq!(g.amount("propellant"), f("5"));
    assert_eq!((g.run.stats.shipments, g.run.pads[0].cooldown), (1, 10));
    assert!(g.run.markets["he3"].saturation >= f("0.1"));
    g.run.pads[0].cargo = f("100");
    set(&mut g, "propellant", "25");
    assert!(r.apply(&mut g, &Action::Launch { pad: 0 }).is_err(), "cooling down");
}

#[test]
fn switching_a_loaded_pads_cargo_returns_it_up_to_the_cap() {
    let r = rules();
    let mut g = r.new_game(1);
    own(&mut g, "launch_pad", 1);
    let (good, other) = ("he3", "titanium");
    r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some(good.into()) }).unwrap();
    let cap = economy::cap(&r, &g, other).unwrap();
    // Room for all of it: back to the stockpile, nothing lost, no line.
    set(&mut g, good, "0");
    g.run.pads[0].cargo = f("5");
    let lines = g.run.activity.len();
    r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some(other.into()) }).unwrap();
    assert_eq!((g.amount(good), g.run.pads[0].cargo, g.run.pads[0].good.as_deref()), (f("5"), Fx::ZERO, Some(other)));
    assert_eq!(g.run.activity.len(), lines);
    // Two short of the cap: two return, the rest is lost and a line says so.
    g.run.pads[0].cargo = f("12");
    set(&mut g, other, &(cap - f("2")).to_string());
    r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some(good.into()) }).unwrap();
    assert_eq!(g.amount(other), cap);
    let a = g.run.activity.last().unwrap();
    assert_eq!((a.kind.as_str(), a.id.as_str(), a.amount, a.quantity), ("cargo_lost", other, Some(f("1")), Some(f("10"))));
    assert_eq!(g.run.pads[0].cargo, Fx::ZERO);
    // Back to No cargo: the cargo returns the same way.
    g.run.pads[0].cargo = f("3");
    set(&mut g, good, "0");
    r.apply(&mut g, &Action::SetPadGood { pad: 0, good: None }).unwrap();
    assert_eq!((g.amount(good), g.run.pads[0].cargo, g.run.pads[0].good.as_deref()), (f("3"), Fx::ZERO, None));
}

#[test]
fn a_new_pad_has_no_cargo_and_loads_nothing_until_one_is_picked() {
    let r = rules();
    let mut g = r.new_game(1);
    own(&mut g, "launch_pad", 1);
    own(&mut g, "solar_panel", 10);
    set(&mut g, "he3", "20");
    assert_eq!(g.run.pads[0].good, None);
    let v = r.view(&g);
    assert_eq!(v.pads[0].good, None);
    crate::modules::shipments::load(&r, &mut g, None);
    assert_eq!((g.run.pads[0].cargo, g.amount("he3")), (Fx::ZERO, f("20")), "Load skips a No-cargo pad");
    r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some("he3".into()) }).unwrap();
    crate::modules::shipments::load(&r, &mut g, None);
    assert!(g.run.pads[0].cargo > Fx::ZERO);
    assert!(r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some("energy".into()) }).is_err());
}

#[test]
fn a_save_from_before_no_cargo_keeps_its_pads_goods() {
    // An old save's pad: its good a bare string, as before No cargo.
    let mut old = serde_json::to_value(crate::state::Pad { good: None, paused: false, cargo: f("5"), cooldown: 2 }).unwrap();
    old["good"] = serde_json::json!("titanium");
    let pad: crate::state::Pad = serde_json::from_value(old).unwrap();
    assert_eq!(pad.good.as_deref(), Some("titanium"));
    let old_action = r#"{"type":"set_pad_good","pad":0,"good":"he3"}"#;
    let a: Action = serde_json::from_str(old_action).unwrap();
    assert_eq!(a, Action::SetPadGood { pad: 0, good: Some("he3".into()) });
}

#[test]
fn demand_stays_in_its_band_and_moves() {
    let r = rules();
    let mut g = r.new_game(7);
    let (mut lo, mut hi) = (Fx::MAX, Fx::MIN);
    for _ in 0..600 {
        r.advance(&mut g, 1);
        for m in g.run.markets.values() {
            lo = lo.min(m.demand);
            hi = hi.max(m.demand);
        }
    }
    assert!(lo >= f("0.01") && hi <= f("1"), "{lo} {hi}");
    assert!(hi - lo > f("0.3"), "noise and rivals move demand: {lo} {hi}");
}

#[test]
fn the_activity_feed_logs_shipments_and_quests_and_keeps_the_newest() {
    let r = rules();
    let mut s = Session::new(&baseline(), 1, None).unwrap();
    play(&mut s, 900, Policy::Script);
    let a = &s.state.run.activity;
    assert!(a.iter().any(|x| x.kind == "quest" && x.id == "qpower_boot_sequence"));
    let ship = a.iter().find(|x| x.kind == "shipment").expect("a shipment by day 900");
    assert!(ship.amount.is_some_and(|v| v > Fx::ZERO) && ship.quantity.is_some());
    assert!(a.len() <= r.p.events.activity_kept);
    assert!(a.windows(2).all(|w| w[0].day <= w[1].day), "oldest first");
}
