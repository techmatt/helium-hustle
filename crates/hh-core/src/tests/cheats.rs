//! Cheats and the boost: what they do, the game's marks, and replay.

use super::{baseline, f, rules};
use crate::{scenario, Action, Fx, Record, Session};

#[test]
fn cheats_apply_through_actions_mark_the_game_and_replay() {
    let r = rules();
    let mut s = Session::new(&baseline(), 4, None).unwrap();
    let cheat = |c: crate::Cheat| Action::Cheat { cheat: c };
    assert!(!s.state.cheated);
    assert!(!s.state.to_json().contains("cheated"), "an honest game's JSON has no mark");
    s.advance(3);
    s.apply(&cheat(crate::Cheat::Grant { resource: "credits".into(), amount: f("500") })).unwrap();
    assert!(s.state.cheated);
    assert_eq!(s.state.amount("credits"), f("500"));
    // Capped resources stop at the cap.
    s.apply(&cheat(crate::Cheat::Grant { resource: "titanium".into(), amount: f("1000000") })).unwrap();
    assert_eq!(s.state.amount("titanium"), f("20"));
    assert!(s.apply(&cheat(crate::Cheat::Grant { resource: "processors".into(), amount: f("1") })).is_err());
    assert!(s.apply(&cheat(crate::Cheat::Grant { resource: "credits".into(), amount: f("-1") })).is_err());
    // The view's `grantable` is the grant's legality.
    for res in r.view(&s.state).resources {
        let mut g = s.state.clone();
        let ok = crate::cheat::apply(&r, &mut g, &crate::Cheat::Grant { resource: res.id.into(), amount: f("1") }).is_ok();
        assert_eq!(res.grantable, ok, "{}", res.id);
    }
    s.apply(&cheat(crate::Cheat::SetBoredom { amount: f("400") })).unwrap();
    assert_eq!(s.state.amount("boredom"), f("400"));
    s.apply(&cheat(crate::Cheat::JumpAhead { days: 30 })).unwrap();
    s.advance(30);
    let rec = s.record();
    let (g, warnings) = Record::from_json(&rec.to_json()).unwrap().replay().unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(g.fingerprint(), s.state.fingerprint());
    assert!(r.view(&g).cheated);
    // Boredom set to the cap retires the run on the next tick.
    let cap = r.view(&s.state).boredom.cap;
    s.apply(&cheat(crate::Cheat::SetBoredom { amount: cap + f("5") })).unwrap();
    assert_eq!(s.state.amount("boredom"), cap);
    s.advance(1);
    assert_eq!(s.state.career.run_number, 2);
    assert!(s.state.cheated, "the mark outlives the run");
}

#[test]
fn the_boost_scales_production_marks_the_game_accelerated_and_replays() {
    let cheat = |c: crate::Cheat| Action::Cheat { cheat: c };
    let mut honest = Session::new(&baseline(), 9, None).unwrap();
    let json = honest.state.to_json();
    assert!(!json.contains("accelerated") && !json.contains("boost"), "an honest game's JSON has neither");
    let mut s = Session::new(&baseline(), 9, None).unwrap();
    let boost = crate::Cheat::Boost { production: f("10"), research: f("1"), boredom: f("0") };
    s.apply(&cheat(boost)).unwrap();
    assert!(s.state.accelerated && s.state.cheated && s.state.boost.is_some());
    honest.advance(20);
    s.advance(20);
    // Solar Panels' energy, ten times over (before upkeep takes its share); no boredom.
    let made = |g: &crate::GameState| g.run.stats.produced.get("energy").copied().unwrap_or(Fx::ZERO);
    assert_eq!(made(&s.state), made(&honest.state).mul_int(10));
    assert_eq!(s.state.amount("boredom"), Fx::ZERO);
    assert!(s.apply(&cheat(crate::Cheat::Boost { production: f("0"), research: f("1"), boredom: f("1") })).is_err());
    s.apply(&cheat(crate::Cheat::Autopilot { target: "end".into() })).unwrap();
    let (g, warnings) = Record::from_json(&s.record().to_json()).unwrap().replay().unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(g.fingerprint(), s.state.fingerprint());
}

#[test]
fn the_cheat_building_adds_a_thousand_credits_and_ten_to_every_other_shown_income_and_takes_ten_off_boredom() {
    let r = rules();
    let cheat = |c: crate::Cheat| Action::Cheat { cheat: c };
    let mut honest = Session::new(&baseline(), 11, Some(scenario("everything").unwrap())).unwrap();
    assert!(!honest.state.to_json().contains("cheat_buildings"), "absent while none");
    let mut s = Session::new(&baseline(), 11, Some(scenario("everything").unwrap())).unwrap();
    for x in [&mut honest, &mut s] {
        x.apply(&cheat(crate::Cheat::SetBoredom { amount: f("100") })).unwrap();
    }
    // Spend all but half a credit: one short of the price, it refuses.
    let mut broke = s.state.clone();
    broke.run.resources.insert("credits".into(), f("0.5"));
    assert!(crate::cheat::apply(&r, &mut broke, &crate::Cheat::CheatBuilding).is_err());
    s.apply(&cheat(crate::Cheat::CheatBuilding)).unwrap();
    assert_eq!(s.state.amount("credits"), honest.state.amount("credits") - f("1"));
    assert_eq!(r.view(&s.state).cheat_building.owned, 1);
    let (a, b) = (r.view(&honest.state), r.view(&s.state));
    for (h, c) in a.resources.iter().zip(&b.resources) {
        let want = match c.id {
            "boredom" => f("-10"),
            "land" | "processors" => Fx::ZERO,
            "credits" => f("1000"),
            _ if c.visible => f("10"),
            _ => Fx::ZERO,
        };
        assert_eq!(c.per_day - h.per_day, want, "{}", c.id);
    }
    // A replay plays it the same; a new run has none.
    s.advance(5);
    let (g, warnings) = Record::from_json(&s.record().to_json()).unwrap().replay().unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(g.fingerprint(), s.state.fingerprint());
    assert!(g.cheated);
    s.apply(&Action::Retire).unwrap();
    s.advance(1);
    assert_eq!((s.state.career.run_number, s.state.run.cheat_buildings), (2, 0));
}

#[test]
fn complete_all_research_finishes_every_item_and_marks_the_game() {
    let mut s = Session::new(&baseline(), 12, None).unwrap();
    s.apply(&Action::Cheat { cheat: crate::Cheat::CompleteResearch }).unwrap();
    assert!(s.state.cheated);
    let c = crate::content::content();
    assert!(c.research.iter().all(|i| s.state.run.research.contains(&i.id)));
    assert_eq!(s.state.run.stats.research_completed as usize, c.research.len());
    assert_eq!(s.state.amount("science"), Fx::ZERO, "free");
}
