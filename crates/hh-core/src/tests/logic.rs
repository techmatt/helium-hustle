//! Program instructions beyond commands: `if` blocks, `wait until`, `build`
//! and `buy land` (docs/rules/programs.md).

use super::{f, rules, set};
use crate::modules::{economy, programs};
use crate::state::{Clause, Cmp, Condition, Entry, GameState, Holding, Join, Joined, LineOutcome as O, Op, Reading};
use crate::{Action, Fx, Rules};

/// A new game with the instructions unlocked.
fn game(r: &Rules) -> GameState {
    let mut g = r.new_game(1);
    g.career.flags.insert(programs::LOGIC_FLAG.into());
    g
}

fn clause(reading: Reading, cmp: Cmp, v: &str) -> Clause {
    Clause { reading, cmp, value: f(v), against: None }
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
fn a_program_of_only_false_ifs_idles_after_one_loop() {
    let r = rules();
    let mut g = game(&r);
    let never = one(amount("credits", Cmp::Lt, "0"));
    put(&r, &mut g, vec![when(never.clone()), Entry::command("idle", 1).indent()]).unwrap();
    let before = g.run.resources.clone();
    r.advance(&mut g, 3);
    assert_eq!(outcomes(&g), [Some(O::False), Some(O::Jumped)]);
    assert_eq!(g.amount("credits"), before["credits"]);
}

#[test]
fn a_done_line_costs_its_step_and_a_guarded_one_is_skipped_free() {
    let r = rules();
    let mut g = game(&r);
    assert_eq!(g.run.programs[0].processors, 1, "one step a day");
    assert_eq!(g.owned("solar_panel"), 1);
    // `build up to 1` with one owned: done, the day's step, Idle not reached.
    put(&r, &mut g, vec![build("solar_panel", Some(1)), Entry::command("idle", 1)]).unwrap();
    let credits = g.amount("credits");
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g), [Some(O::Done), None]);
    assert_eq!(g.amount("credits"), credits, "a done line pays nothing");
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g), [None, Some(O::Ran)], "the next step is the next line");
    // Guarded: the false `if` skips its body for free and Idle runs the same day.
    let fewer = one(clause(Reading::Count { building: "solar_panel".into() }, Cmp::Lt, "1"));
    put(&r, &mut g, vec![when(fewer), build("solar_panel", Some(1)).indent(), Entry::command("idle", 1)]).unwrap();
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g), [Some(O::False), Some(O::Jumped), Some(O::Ran)]);
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
    assert_eq!(read(&g, Reading::PadsReady { good: None }), Some(Fx::ZERO));
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
fn rate_reads_demand_so_a_line_under_rate_below_zero_runs_every_day_of_a_brownout() {
    // Production less demand (docs/rules/programs.md): unmet upkeep counts, so
    // Energy's rate stays negative after the store has drained.
    let r = rules();
    let mut g = game(&r);
    g.run.buildings.insert("data_center".into(), Holding { owned: 6, active: 6, bonus: 0, free: 0, unpaid: 0 });
    economy::refresh(&mut g);
    let below = one(clause(Reading::Rate { resource: "energy".into() }, Cmp::Lt, "0"));
    put(&r, &mut g, vec![when(below), Entry::command("idle", 1).indent()]).unwrap();
    let rate = |g: &GameState| programs::reading(&r, g, &Reading::Rate { resource: "energy".into() }).unwrap();
    r.advance(&mut g, 1);
    assert_eq!(rate(&g), f("-7"), "one panel's 5 against six Data Centers' 12");
    let mut brownout_days = 0;
    for _ in 0..60 {
        r.advance(&mut g, 1);
        assert_eq!(rate(&g), f("-7"), "the demand stays, met or not (day {})", g.run.day);
        assert_eq!(outcomes(&g)[1], Some(O::Ran), "the body ran (day {})", g.run.day);
        if g.amount("energy") < f("0.01") {
            brownout_days += 1;
        }
    }
    assert!(brownout_days > 30, "a steady brownout: {brownout_days} days at 0");
    // The panel shows the same quantity for the next day, with the stock's change beside it.
    let e = r.view(&g).resources.into_iter().find(|x| x.id == "energy").unwrap();
    assert_eq!(e.per_day, f("-7"));
    assert!(e.unmet > f("6.99") && e.change_per_day < f("0.01") && e.change_per_day > f("-0.01"), "{e:?}");
}

#[test]
fn unmet_demand_counts_only_on_the_input_that_ran_short() {
    // A Smelter starved of energy wanted its regolith too, but regolith was on
    // hand: its rate reads what was used, not a shortage.
    let r = rules();
    let mut g = game(&r);
    g.run.buildings.insert("smelter".into(), Holding { owned: 4, active: 4, bonus: 0, free: 0, unpaid: 0 });
    economy::refresh(&mut g);
    set(&mut g, "energy", "0");
    set(&mut g, "regolith", "40");
    r.advance(&mut g, 1);
    let read = |res: &str| programs::reading(&r, &g, &Reading::Rate { resource: res.into() }).unwrap();
    assert_eq!(read("regolith"), g.amount("regolith") - f("40"), "regolith: actual use");
    assert!(!g.run.today.unmet.contains_key("regolith"));
    assert!(read("energy") < Fx::ZERO && g.run.today.unmet["energy"] > Fx::ZERO);
}

#[test]
fn a_failed_command_and_a_short_project_count_as_demand() {
    let r = rules();
    let mut g = game(&r);
    // Cloud Compute wants 2 energy a run; with none on hand it fails, still wanting it.
    g.run.buildings.insert("solar_panel".into(), Holding::default());
    economy::refresh(&mut g);
    set(&mut g, "energy", "0");
    put(&r, &mut g, vec![Entry::command("cloud_compute", 1)]).unwrap();
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[0], Some(O::Failed));
    assert_eq!(g.run.today.unmet["energy"], f("4"), "Cloud Compute's 2 and the Data Center's 2");
}

#[test]
fn a_loadout_saved_in_one_run_loads_and_runs_in_the_next() {
    let r = rules();
    let mut g = game(&r);
    let lines = vec![when(one(amount("credits", Cmp::Ge, "0"))), Entry::command("idle", 1).indent()];
    put(&r, &mut g, lines.clone()).unwrap();
    let save = |g: &mut GameState, slot: Option<u32>, name: &str| {
        r.apply(g, &Action::SaveLoadout { program: 0, slot, name: name.into() })
    };
    save(&mut g, None, "  Opening  ").unwrap();
    assert_eq!(g.career.library[0].name, "Opening", "trimmed");
    crate::modules::career::retire(&r, &mut g, false);
    assert!(g.run.programs[0].entries.is_empty(), "programs still empty on retirement");
    assert_eq!(r.view(&g).library.len(), 1, "the library is career state");
    r.apply(&mut g, &Action::LoadLoadout { loadout: 0, program: 0 }).unwrap();
    assert_eq!(g.run.programs[0].entries, lines);
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g), vec![Some(O::True), Some(O::Ran)]);
    // Overwrite, rename, delete; the refusals change nothing.
    put(&r, &mut g, vec![Entry::command("idle", 3)]).unwrap();
    save(&mut g, Some(0), "Idle").unwrap();
    assert_eq!(g.career.library[0].entries, vec![Entry::command("idle", 3)]);
    r.apply(&mut g, &Action::RenameLoadout { loadout: 0, name: "Idler".into() }).unwrap();
    assert_eq!(g.career.library[0].name, "Idler");
    let before = g.clone();
    assert!(save(&mut g, None, "   ").is_err(), "a name is needed");
    assert!(save(&mut g, None, &"x".repeat(r.p.programs.loadout_name_max + 1)).is_err());
    assert!(save(&mut g, Some(5), "nope").is_err(), "no slot 6");
    assert!(r.apply(&mut g, &Action::SaveLoadout { program: 1, slot: None, name: "empty".into() }).is_err());
    assert!(r.apply(&mut g, &Action::LoadLoadout { loadout: 3, program: 0 }).is_err());
    assert!(r.apply(&mut g, &Action::LoadLoadout { loadout: 0, program: 99 }).is_err());
    assert_eq!(g, before);
    while g.career.library.len() < r.p.programs.library_slots {
        save(&mut g, None, "more").unwrap();
    }
    assert!(save(&mut g, None, "one too many").is_err(), "the library is full");
    r.apply(&mut g, &Action::DeleteLoadout { loadout: 0 }).unwrap();
    assert_eq!(g.career.library.len(), r.p.programs.library_slots - 1);
    assert_eq!(g.career.library[0].name, "more", "later slots move up");
}

#[test]
fn a_loaded_line_this_run_has_not_unlocked_fails_or_reads_nothing() {
    let r = rules();
    let mut g = game(&r);
    // Demand revealed in an earlier run: a saved line reads it.
    let demand = one(clause(Reading::Demand { good: "he3".into() }, Cmp::Gt, "0"));
    g.career.library.push(crate::state::Loadout {
        name: "Later".into(),
        entries: vec![when(demand.clone()), Entry::command("idle", 1).indent(), Entry::command("dream", 1)],
    });
    assert!(put(&r, &mut g, vec![when(demand)]).is_err(), "set_program refuses it this run");
    r.apply(&mut g, &Action::LoadLoadout { loadout: 0, program: 0 }).unwrap();
    r.advance(&mut g, 1);
    let o = outcomes(&g);
    assert_eq!(o[0], Some(O::False), "demand not revealed: reads nothing");
    assert_eq!(o[2], Some(O::Failed), "Dream not researched: fails and advances");
    // The loaded program stays editable: an edit keeping that line goes through, a
    // new such line is still refused.
    let mut lines = g.run.programs[0].entries.clone();
    lines.push(Entry::command("idle", 2));
    put(&r, &mut g, lines.clone()).unwrap();
    lines.push(when(one(clause(Reading::Demand { good: "titanium".into() }, Cmp::Gt, "0"))));
    assert!(put(&r, &mut g, lines).is_err());
}

#[test]
fn a_project_funded_past_the_stock_counts_its_rate_as_demand() {
    let r = rules();
    let mut g = game(&r);
    let p = crate::content::content()
        .projects
        .iter()
        .find(|p| p.costs.contains_key("credits") && matches!(p.unlock, crate::content::Condition::ProjectEnabled { .. }))
        .expect("an event-unlocked project costs credits");
    g.run.enabled_projects.insert(p.id.clone());
    set(&mut g, "credits", "0");
    g.run.project_rates.insert(p.id.clone(), [("credits".to_string(), f("3"))].into());
    crate::modules::projects::drain(&r, &mut g);
    assert_eq!(g.run.today.unmet["credits"], f("3"), "it wanted 3 and got none");
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
