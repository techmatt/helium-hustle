//! Program instructions beyond commands: `if` blocks, `wait until`, `build`
//! and `buy land` (docs/rules/programs.md).

use super::baseline;
use crate::modules::{economy, programs};
use crate::state::{Clause, Cmp, Condition, Entry, GameState, Holding, Join, Joined, LineOutcome as O, Op, Reading};
use crate::{Action, Fx, Rules};

fn f(s: &str) -> Fx {
    Fx::parse(s).unwrap()
}

fn rules() -> Rules {
    Rules::new(&baseline()).unwrap()
}

/// A new game with the instructions unlocked.
fn game(r: &Rules) -> GameState {
    let mut g = r.new_game(1);
    g.career.flags.insert(programs::LOGIC_FLAG.into());
    g
}

fn set(g: &mut GameState, res: &str, v: &str) {
    g.run.resources.insert(res.into(), f(v));
}

fn clause(reading: Reading, cmp: Cmp, v: &str) -> Clause {
    Clause { reading, cmp, value: f(v) }
}

fn amount(res: &str, cmp: Cmp, v: &str) -> Clause {
    clause(Reading::Amount { resource: res.into() }, cmp, v)
}

fn one(c: Clause) -> Condition {
    Condition { first: c, rest: None }
}

fn build(b: &str, up_to: Option<u32>) -> Entry {
    Entry::new(Op::Build { building: b.into(), up_to })
}

fn when(c: Condition) -> Entry {
    Entry::new(Op::If { when: c })
}

fn put(r: &Rules, g: &mut GameState, entries: Vec<Entry>) -> Result<(), String> {
    r.apply(g, &Action::SetProgram { program: 0, entries })
}

fn outcomes(g: &GameState) -> Vec<Option<O>> {
    crate::view::LinesRun::today(g).lines[0].clone()
}

#[test]
fn the_briefs_program_builds_a_base_unattended() {
    let r = rules();
    let mut g = game(&r);
    let rate = Reading::Rate { resource: "energy".into() };
    let entries = vec![
        build("solar_panel", Some(4)),
        when(one(clause(rate, Cmp::Lt, "0"))),
        build("solar_panel", None).indent(),
        when(Condition {
            first: amount("land", Cmp::Lt, "1"),
            rest: Some(Joined { join: Join::And, clause: amount("credits", Cmp::Gt, "500") }),
        }),
        Entry::new(Op::BuyLand {}).indent(),
        Entry::new(Op::WaitUntil { when: one(amount("credits", Cmp::Gt, "800")) }),
        build("data_center", Some(3)),
    ];
    put(&r, &mut g, entries).unwrap();
    // Materials arrive daily; credits only from day 20: the program waits for them.
    for day in 0..60 {
        // Past their caps (the day's clamp cuts them back): prices scale past them.
        set(&mut g, "titanium", "500");
        set(&mut g, "circuits", "500");
        if day < 20 {
            set(&mut g, "credits", "100");
        } else {
            set(&mut g, "credits", "2000");
        }
        r.advance(&mut g, 1);
        if day == 10 {
            assert_eq!(g.owned("solar_panel"), 2, "one bought a pass, and the pass is held at the wait");
            assert_eq!(g.owned("data_center"), 1, "waiting for credits");
            assert_eq!(outcomes(&g)[5], Some(O::Waiting));
            let v = r.view(&g);
            let c = v.programs[0].conditions[5].as_ref().expect("a wait until line");
            assert!(!c.holds);
            assert_eq!(c.clauses[0].value, Some(g.amount("credits")));
            assert!(v.programs[0].conditions[0].is_none());
        }
    }
    assert_eq!(g.owned("data_center"), 3, "{:?} {:?}", outcomes(&g), g.run.today.shorts);
    assert_eq!(g.owned("solar_panel"), 4, "energy never ran short, so the if never built");
    assert_eq!(g.run.land_bought, 0);
}

#[test]
fn a_false_if_jumps_its_body_for_free_and_a_true_one_runs_it_whole() {
    let r = rules();
    let mut g = game(&r);
    set(&mut g, "credits", "10");
    let entries = vec![
        when(one(amount("credits", Cmp::Ge, "50"))),
        Entry::command("idle", 1).indent(),
        Entry::command("idle", 1).indent(),
        Entry::command("cloud_compute", 1),
    ];
    put(&r, &mut g, entries).unwrap();
    // False: the one processor's step goes to line 3, the body jumped at no cost.
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[..3], [Some(O::False), Some(O::Jumped), Some(O::Jumped)]);
    assert!(outcomes(&g)[3].is_some());
    assert_eq!(g.run.programs[0].entry, 0);
    // True on entry: both body lines run, though the condition turns false midway.
    set(&mut g, "credits", "50");
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[..2], [Some(O::True), Some(O::Ran)]);
    assert_eq!(g.run.programs[0].entry, 2);
    set(&mut g, "credits", "0");
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[2], Some(O::Ran));
    assert_eq!(g.run.programs[0].entry, 3);
}

#[test]
fn a_body_line_that_ran_today_keeps_its_outcome_when_its_if_turns_false() {
    let r = rules();
    let mut g = game(&r);
    g.run.buildings.insert("data_center".into(), Holding { owned: 2, active: 2, bonus: 0, free: 0, unpaid: 0 });
    economy::refresh(&mut g);
    r.apply(&mut g, &Action::SetProcessors { program: 0, processors: 2 }).unwrap();
    set(&mut g, "credits", "1000");
    set(&mut g, "titanium", "500");
    // One Solar Panel owned: the first step builds the second, the next finds the `if` false.
    let fewer = one(clause(Reading::Count { building: "solar_panel".into() }, Cmp::Lt, "2"));
    put(&r, &mut g, vec![when(fewer), build("solar_panel", None).indent()]).unwrap();
    r.advance(&mut g, 1);
    assert_eq!(g.owned("solar_panel"), 2);
    assert_eq!(outcomes(&g), [Some(O::False), Some(O::Ran)]);
}

#[test]
fn a_program_of_only_free_lines_idles_after_one_loop() {
    let r = rules();
    let mut g = game(&r);
    let never = one(amount("credits", Cmp::Lt, "0"));
    put(&r, &mut g, vec![when(never.clone()), Entry::command("idle", 1).indent(), build("solar_panel", Some(1))]).unwrap();
    let before = g.run.resources.clone();
    r.advance(&mut g, 3);
    assert_eq!(outcomes(&g), [Some(O::False), Some(O::Jumped), Some(O::Done)]);
    assert_eq!(g.amount("credits"), before["credits"]);
    assert_eq!(g.owned("solar_panel"), 1);
}

#[test]
fn build_fails_and_advances_when_it_cannot_pay_or_has_no_land() {
    let r = rules();
    let mut g = game(&r);
    put(&r, &mut g, vec![build("solar_panel", None), Entry::command("idle", 1)]).unwrap();
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[0], Some(O::Failed));
    assert_eq!(g.run.programs[0].entry, 1, "a failed build costs its step and advances");
    assert_eq!(g.run.today.shorts["program:0:0"].resource, "credits");
    r.advance(&mut g, 1);
    set(&mut g, "credits", "500");
    set(&mut g, "titanium", "20");
    set(&mut g, "land", "0");
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[0], Some(O::Failed));
    assert_eq!(g.run.today.shorts["program:0:0"].resource, "land");
    assert_eq!(g.owned("solar_panel"), 1);
    // Buy Land as an instruction, then the build goes through.
    put(&r, &mut g, vec![Entry::new(Op::BuyLand {}), build("solar_panel", None)]).unwrap();
    r.advance(&mut g, 2);
    assert_eq!(g.run.land_bought, 1);
    assert_eq!(g.owned("solar_panel"), 2);
    assert_eq!(g.run.stats.buildings_built, 1);
}

#[test]
fn readings_read_the_state() {
    let r = rules();
    let mut g = game(&r);
    let read = |g: &GameState, what: Reading| programs::reading(&r, g, &what);
    set(&mut g, "energy", "25");
    assert_eq!(read(&g, Reading::Percent { resource: "energy".into() }), Some(f("25")));
    assert_eq!(read(&g, Reading::Percent { resource: "credits".into() }), None, "no cap, nothing to read");
    assert_eq!(read(&g, Reading::Count { building: "solar_panel".into() }), Some(f("1")));
    assert_eq!(read(&g, Reading::Demand { good: "he3".into() }), None, "demand is not revealed yet");
    assert_eq!(read(&g, Reading::PadsReady), Some(Fx::ZERO));
    // The rate is the last full day's production less use: one panel against one Data Center.
    r.advance(&mut g, 2);
    assert_eq!(read(&g, Reading::Day), Some(f("2")));
    let (made, used) = (g.run.today.produced["energy"], g.run.today.consumed["energy"]);
    assert_eq!(read(&g, Reading::Rate { resource: "energy".into() }), Some(made - used));
    assert_eq!(read(&g, Reading::Rate { resource: "he3".into() }), Some(Fx::ZERO));
    // The view offers only what the player can see, by display name.
    let v = r.view(&g);
    let labels: Vec<&str> = v.readings.iter().map(|x| x.label.as_str()).collect();
    assert!(labels.contains(&"Energy rate") && labels.contains(&"Land free") && labels.contains(&"Solar Panel count"));
    assert!(!labels.iter().any(|l| l.ends_with("demand") || l.starts_with("Helium-3")), "{labels:?}");
}

#[test]
fn rate_is_the_actual_flow_negative_while_the_stock_drains_and_zero_in_a_steady_brownout() {
    // The reading's definition (docs/rules/programs.md); counting unmet upkeep as
    // demand instead would keep it negative through the brownout.
    let r = rules();
    let mut g = game(&r);
    g.run.buildings.insert("data_center".into(), Holding { owned: 6, active: 6, bonus: 0, free: 0, unpaid: 0 });
    economy::refresh(&mut g);
    let rate = |g: &GameState| programs::reading(&r, g, &Reading::Rate { resource: "energy".into() }).unwrap();
    r.advance(&mut g, 1);
    assert_eq!(rate(&g), f("-7"), "one panel's 5 against six Data Centers' 12");
    r.advance(&mut g, 40);
    assert!(g.amount("energy") < f("0.01"), "the store has drained: {}", g.amount("energy"));
    assert!(rate(&g) >= Fx::ZERO && rate(&g) < f("0.01"), "use equals production: {}", rate(&g));
}

#[test]
fn set_program_refuses_what_the_language_does_not_allow() {
    let r = rules();
    let mut locked = r.new_game(1);
    let cond = one(amount("credits", Cmp::Gt, "1"));
    let e = put(&r, &mut locked, vec![when(cond.clone())]).unwrap_err();
    assert!(e.contains("only commands"), "{e}");
    assert!(put(&r, &mut locked, vec![Entry::command("idle", 1)]).is_ok());

    let mut g = game(&r);
    let bad = [
        vec![Entry::command("idle", 1).indent()],
        vec![when(cond.clone()), when(cond.clone()).indent()],
        vec![when(cond.clone()), Entry::command("idle", 1), Entry::command("idle", 1).indent()],
        vec![build("nope", None)],
        vec![build("solar_panel", Some(0))],
        vec![when(one(amount("nope", Cmp::Gt, "1")))],
        vec![when(one(clause(Reading::Demand { good: "he3".into() }, Cmp::Gt, "1")))],
    ];
    for entries in bad {
        assert!(put(&r, &mut g, entries.clone()).is_err(), "{entries:?}");
    }
    let wait = Entry::new(Op::WaitUntil { when: cond.clone() });
    put(&r, &mut g, vec![when(cond), wait.indent(), Entry::new(Op::BuyLand {}).indent()]).unwrap();
}

#[test]
fn the_unlock_is_a_career_first_kept_across_retirement() {
    let r = rules();
    let mut g = r.new_game(1);
    r.advance(&mut g, 3);
    assert!(!programs::logic_unlocked(&g), "one processor");
    assert!(!r.view(&g).program_logic);
    g.run.buildings.insert("data_center".into(), Holding { owned: 2, active: 2, bonus: 0, free: 0, unpaid: 0 });
    economy::refresh(&mut g);
    r.advance(&mut g, 1);
    assert!(programs::logic_unlocked(&g));
    crate::modules::career::retire(&r, &mut g, false);
    assert_eq!(g.owned("data_center"), 1);
    assert!(r.view(&g).program_logic, "kept in run 2");
    assert!(g.run.programs.iter().all(|p| p.entries.is_empty()), "programs still empty on retirement");
}

#[test]
fn a_line_is_json_with_an_op_tag() {
    let line = build("solar_panel", Some(4)).indent();
    let json = serde_json::to_string(&line).unwrap();
    assert_eq!(json, r#"{"op":"build","building":"solar_panel","up_to":4,"indented":true}"#);
    assert_eq!(serde_json::from_str::<Entry>(&json).unwrap(), line);
    let cmd: Entry = serde_json::from_str(r#"{"op":"command","command":"idle","count":2}"#).unwrap();
    assert_eq!(cmd, Entry::command("idle", 2));
    assert!(serde_json::from_str::<Entry>(r#"{"command":"idle","count":2}"#).is_err(), "the op tag is required");
    // A misspelt key is refused, not dropped (an unlimited build, a top-level line).
    for bad in [
        r#"{"op":"build","building":"solar_panel","upto":3}"#,
        r#"{"op":"buy_land","indent":true}"#,
        r#"{"op":"command","command":"idle","count":1,"up_to":2}"#,
    ] {
        assert!(serde_json::from_str::<Entry>(bad).is_err(), "{bad}");
    }
    assert_eq!(serde_json::from_str::<Entry>(r#"{"op":"build","building":"solar_panel"}"#).unwrap(), build("solar_panel", None));
    let a: Action = serde_json::from_str(
        r#"{"type":"set_program","program":0,"entries":[{"op":"if","when":{"first":{"reading":{"kind":"amount","resource":"credits"},"cmp":"gt","value":"5"},"rest":null}},{"op":"buy_land","indented":true}]}"#,
    )
    .unwrap();
    let Action::SetProgram { entries, .. } = a else { panic!() };
    assert_eq!(entries[1], Entry::new(Op::BuyLand {}).indent());
}
