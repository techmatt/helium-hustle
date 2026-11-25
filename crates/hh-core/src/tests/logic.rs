//! Program instructions beyond commands: `if` blocks, `build` and `buy land`
//! (docs/rules/programs.md).

use super::{f, logic_open, open_if, outcomes, rules, set};
use crate::modules::{economy, programs};
use crate::state::{Clause, Cmp, Condition, Entry, GameState, Holding, Join, Joined, LineOutcome as O, Op, Reading};
use crate::{Action, Fx, Rules};

/// A new game with the instructions unlocked (`if` too).
fn game(r: &Rules) -> GameState {
    logic_open(r)
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

#[test]
fn the_briefs_program_builds_a_base_unattended() {
    let r = rules();
    let mut g = game(&r);
    let energy = Reading::Percent { resource: "energy".into() };
    let entries = vec![
        build("solar_panel", Some(4)),
        when(one(clause(energy, Cmp::Lt, "1"))),
        build("solar_panel", None).indent(),
        when(Condition {
            first: clause(Reading::FreeLand, Cmp::Lt, "1"),
            rest: Some(Joined { join: Join::And, clause: amount("credits", Cmp::Gt, "500") }),
        }),
        Entry::new(Op::BuyLand {}).indent(),
        when(one(amount("credits", Cmp::Gt, "800"))),
        build("data_center", Some(3)).indent(),
    ];
    put(&r, &mut g, entries).unwrap();
    // Materials arrive daily; credits only from day 20: the Data Centers wait for them.
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
            assert_eq!(g.owned("data_center"), 1, "waiting for credits");
            let v = r.view(&g);
            let c = v.programs[0].conditions[5].as_ref().expect("an if line");
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
fn a_false_if_jumps_its_body_in_its_one_step_and_a_true_one_runs_it_whole() {
    let r = rules();
    let mut g = game(&r);
    set(&mut g, "credits", "10");
    let entries = vec![
        when(one(amount("credits", Cmp::Gt, "49"))),
        Entry::command("idle", 1).indent(),
        Entry::command("idle", 1).indent(),
        Entry::command("cloud_compute", 1),
    ];
    put(&r, &mut g, entries).unwrap();
    // False: the `if` is the one processor's step; the whole body is jumped with it.
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[..3], [Some(O::False), Some(O::Jumped), Some(O::Jumped)]);
    assert_eq!(g.run.programs[0].entry, 3);
    r.advance(&mut g, 1);
    assert!(outcomes(&g)[3].is_some());
    assert_eq!(g.run.programs[0].entry, 0);
    // True on entry: its step, then both body lines run, though the condition turns false midway.
    set(&mut g, "credits", "50");
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[0], Some(O::True));
    assert_eq!(g.run.programs[0].entry, 1);
    set(&mut g, "credits", "0");
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[1], Some(O::Ran));
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[2], Some(O::Ran));
    assert_eq!(g.run.programs[0].entry, 3);
}

#[test]
fn an_if_decides_its_else_in_its_one_step_and_the_else_line_is_free() {
    let r = rules();
    let mut g = game(&r);
    let entries = vec![
        when(one(amount("credits", Cmp::Gt, "49"))),
        Entry::command("idle", 1).indent(),
        Entry::new(Op::Else {}),
        Entry::command("idle", 2).indent(),
        Entry::command("idle", 1).indent(),
        Entry::command("idle", 1),
    ];
    put(&r, &mut g, entries).unwrap();
    // False: the `if`'s step jumps its body to the else's body.
    set(&mut g, "credits", "10");
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[..3], [Some(O::False), Some(O::Jumped), None]);
    assert_eq!(g.run.programs[0].entry, 3);
    for _ in 0..2 {
        r.advance(&mut g, 1);
        assert_eq!(outcomes(&g)[3], Some(O::Ran));
    }
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[4], Some(O::Ran));
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[5], Some(O::Ran));
    assert_eq!(g.run.programs[0].entry, 0);
    // True: the `if`'s step, then its body; the else line is passed free with its body.
    set(&mut g, "credits", "50");
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[0], Some(O::True));
    assert_eq!(g.run.programs[0].entry, 1);
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[1], Some(O::Ran));
    assert_eq!(g.run.programs[0].entry, 2);
    // The next step passes the else free and runs the last line.
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[2..], [None, Some(O::Jumped), Some(O::Jumped), Some(O::Ran)]);
    assert_eq!(g.run.programs[0].entry, 0);
}

#[test]
fn an_else_after_an_empty_if_body_and_a_locked_if_skips_both_bodies() {
    let r = rules();
    let mut g = game(&r);
    set(&mut g, "credits", "10");
    let entries = vec![
        when(one(amount("credits", Cmp::Gt, "49"))),
        Entry::new(Op::Else {}),
        Entry::command("idle", 1).indent(),
        Entry::command("cloud_compute", 1),
    ];
    put(&r, &mut g, entries.clone()).unwrap();
    r.advance(&mut g, 1);
    assert_eq!(g.run.programs[0].entry, 2, "false: into the else's body");
    // A locked `if` passes over its body and its else's, free.
    let mut g = r.new_game(1);
    g.career.flags.insert(programs::LOGIC_FLAG.into());
    set(&mut g, "credits", "10");
    put(&r, &mut g, entries).unwrap();
    assert!(r.view(&g).programs[0].locked[1].is_some(), "an else is open when its if is");
    r.advance(&mut g, 1);
    let lines = outcomes(&g);
    assert_eq!(lines[..3], [None, None, Some(O::Jumped)]);
    assert!(lines[3].is_some(), "the line after both bodies took the step");
}

#[test]
fn an_else_stands_only_after_an_if_or_its_body_once() {
    let r = rules();
    let mut g = game(&r);
    let yes = || when(one(amount("credits", Cmp::Gt, "49")));
    let idle = || Entry::command("idle", 1);
    let els = || Entry::new(Op::Else {});
    let bad = [
        vec![els()],
        vec![idle(), els()],
        vec![yes(), idle(), els()],
        vec![yes(), els(), els()],
        vec![yes(), els(), idle().indent(), els()],
        vec![yes(), els().indent()],
        vec![yes(), els(), yes().indent()],
    ];
    for entries in bad {
        assert!(put(&r, &mut g, entries.clone()).is_err(), "{entries:?}");
    }
    for entries in [vec![yes(), els()], vec![yes(), idle().indent(), els(), idle().indent(), idle(), yes(), els()]] {
        put(&r, &mut g, entries).unwrap();
    }
    let json = serde_json::to_string(&els()).unwrap();
    assert_eq!(json, r#"{"op":"else"}"#);
}

#[test]
fn a_body_line_that_ran_today_keeps_its_outcome_when_its_if_turns_false() {
    let r = rules();
    let mut g = game(&r);
    g.run.buildings.insert("data_center".into(), Holding { owned: 3, active: 3, bonus: 0, free: 0, unpaid: 0 });
    economy::refresh(&mut g);
    r.apply(&mut g, &Action::SetProcessors { program: 0, processors: 3 }).unwrap();
    set(&mut g, "credits", "1000");
    set(&mut g, "titanium", "500");
    // The `if` holds, the next step builds a Solar Panel with credits, the third finds the `if` false.
    let rich = one(amount("credits", Cmp::Gt, "999"));
    put(&r, &mut g, vec![when(rich), build("solar_panel", None).indent()]).unwrap();
    r.advance(&mut g, 1);
    assert_eq!(g.owned("solar_panel"), 2);
    assert_eq!(outcomes(&g), [Some(O::False), Some(O::Ran)]);
}

#[test]
fn a_repeated_line_records_each_run_of_its_pass_for_the_gutter() {
    use crate::state::Repeat;
    let r = rules();
    let mut g = game(&r);
    let rep = |g: &GameState| crate::view::build(&r, g).programs[0].repeats.clone();
    r.apply(&mut g, &Action::SetProcessors { program: 0, processors: 1 }).unwrap();
    put(&r, &mut g, vec![Entry::command("idle", 4)]).unwrap();
    r.advance(&mut g, 1);
    assert_eq!(rep(&g), [Some(Repeat { done: 1, failed: 0, first_fails: 0 })]);
    r.advance(&mut g, 3);
    assert_eq!(rep(&g), [Some(Repeat { done: 4, failed: 0, first_fails: 0 })], "the last pass stays whole");
    r.advance(&mut g, 1);
    assert_eq!(rep(&g)[0].map(|x| x.done), Some(1), "a new pass starts afresh");
    // Failed runs: red, by run; a new program clears the record; a single run keeps none.
    set(&mut g, "credits", "0");
    put(&r, &mut g, vec![Entry::command("buy_ice", 3), Entry::command("idle", 1)]).unwrap();
    assert_eq!(rep(&g), [None, None]);
    r.advance(&mut g, 2);
    assert_eq!(rep(&g), [Some(Repeat { done: 2, failed: 2, first_fails: 0b11 }), None]);
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
fn a_done_line_costs_its_step_and_a_guarded_one_is_jumped() {
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
    // Guarded: the false `if` takes the step and jumps its body; Idle runs the next day.
    let never = one(amount("credits", Cmp::Lt, "0"));
    put(&r, &mut g, vec![when(never), build("solar_panel", Some(1)).indent(), Entry::command("idle", 1)]).unwrap();
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g), [Some(O::False), Some(O::Jumped), None]);
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g), [None, None, Some(O::Ran)]);
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
    set(&mut g, "land", "3");
    assert_eq!(read(&g, Reading::Percent { resource: "energy".into() }), Some(f("25")));
    assert_eq!(read(&g, Reading::Percent { resource: "credits".into() }), None, "no cap, nothing to read");
    assert_eq!(read(&g, Reading::Amount { resource: "credits".into() }), Some(g.amount("credits")));
    assert_eq!(read(&g, Reading::FreeLand), Some(f("3")));
    assert_eq!(read(&g, Reading::FreeProcessors), Some(Fx::ZERO), "the one processor is on Program 1");
    assert_eq!(read(&g, Reading::IdlePads), Some(Fx::ZERO));
    // The view offers one reading per thing the player can see, by display name.
    let v = r.view(&g);
    let labels: Vec<&str> = v.readings.iter().map(|x| x.label.as_str()).collect();
    for want in ["Energy", "Regolith", "Titanium", "Boredom", "Credits", "Free processors", "Free land"] {
        assert!(labels.contains(&want), "{want} in {labels:?}");
    }
    assert!(!labels.iter().any(|l| l.starts_with("Helium-3") || *l == "Idle pads" || *l == "Land"), "{labels:?}");
    assert!(labels.len() <= 13, "{labels:?}");
}

#[test]
fn the_days_rate_reads_demand_all_through_a_brownout() {
    // Production less demand (docs/rules/programs.md): unmet upkeep counts, so
    // Energy's rate (the Resources panel's per-day number) stays negative
    // after the store has drained.
    let r = rules();
    let mut g = game(&r);
    g.run.buildings.insert("data_center".into(), Holding { owned: 6, active: 6, bonus: 0, free: 0, unpaid: 0 });
    economy::refresh(&mut g);
    let rate = |g: &GameState| g.run.day_net.get("energy").copied().unwrap_or(Fx::ZERO);
    r.advance(&mut g, 1);
    assert_eq!(rate(&g), f("-7"), "one panel's 5 against six Data Centers' 12");
    let mut brownout_days = 0;
    for _ in 0..60 {
        r.advance(&mut g, 1);
        assert_eq!(rate(&g), f("-7"), "the demand stays, met or not (day {})", g.run.day);
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
    let read = |res: &str| g.run.day_net.get(res).copied().unwrap_or(Fx::ZERO);
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
    let lines = vec![when(one(amount("credits", Cmp::Gt, "-1"))), Entry::command("idle", 1).indent()];
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
    r.advance(&mut g, 2);
    assert_eq!(outcomes(&g), vec![None, Some(O::Ran)], "day 1 the `if`, day 2 its body");
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
fn a_loaded_line_this_career_has_not_unlocked_is_skipped_free() {
    let r = rules();
    let mut g = r.new_game(1);
    g.career.flags.insert(programs::LOGIC_FLAG.into());
    // `if` not open (a loadout from elsewhere, say): a saved line holds one.
    let any = one(amount("credits", Cmp::Gt, "-1"));
    g.career.library.push(crate::state::Loadout {
        name: "Later".into(),
        entries: vec![when(any.clone()), Entry::command("idle", 1).indent(), Entry::command("dream", 1)],
    });
    // set_program takes it too (063): kept, locked this run.
    put(&r, &mut g, vec![when(any.clone())]).unwrap();
    assert!(r.view(&g).programs[0].locked[0].as_deref().is_some_and(|w| w.contains("Conditional Logic")));
    r.apply(&mut g, &Action::LoadLoadout { loadout: 0, program: 0 }).unwrap();
    let credits = g.amount("credits");
    r.advance(&mut g, 1);
    // `if` not open: skipped free with its body; Dream not researched: skipped free.
    assert_eq!(outcomes(&g), vec![None, Some(O::Jumped), None]);
    assert!(g.amount("credits") >= credits, "nothing ran");
    // The loaded program stays editable, new locked lines and all.
    let mut lines = g.run.programs[0].entries.clone();
    lines.push(Entry::command("idle", 2));
    lines.push(when(one(amount("credits", Cmp::Lt, "5"))));
    put(&r, &mut g, lines).unwrap();
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
    // Before the unlock a line is taken on its form (063), locked this run.
    put(&r, &mut locked, vec![when(cond.clone())]).unwrap();
    assert!(r.view(&locked).programs[0].locked[0].as_deref().is_some_and(|w| w.contains("program logic")));
    assert!(put(&r, &mut locked, vec![Entry::command("idle", 1)]).is_ok());

    let mut g = game(&r);
    let bad = [
        vec![Entry::command("idle", 1).indent()],
        vec![when(cond.clone()), when(cond.clone()).indent()],
        vec![when(cond.clone()), Entry::command("idle", 1), Entry::command("idle", 1).indent()],
        vec![build("nope", None)],
        vec![build("solar_panel", Some(0))],
        vec![when(one(amount("nope", Cmp::Gt, "1")))],
        // An uncapped stock has no percent; land and processors read their
        // free counts.
        vec![when(one(amount("land", Cmp::Gt, "1")))],
        vec![when(one(amount("processors", Cmp::Gt, "1")))],
        vec![when(one(clause(Reading::Percent { resource: "credits".into() }, Cmp::Gt, "1")))],
    ];
    for entries in bad {
        assert!(put(&r, &mut g, entries.clone()).is_err(), "{entries:?}");
    }
    let boredom = one(clause(Reading::Percent { resource: "boredom".into() }, Cmp::Gt, "1"));
    put(&r, &mut g, vec![when(cond), Entry::new(Op::BuyLand {}).indent(), when(boredom)]).unwrap();
    // The unit toggle: a capped stock, boredom too, may read its amount.
    put(&r, &mut g, vec![when(one(amount("energy", Cmp::Gt, "1"))), when(one(amount("boredom", Cmp::Gt, "1")))]).unwrap();
}

#[test]
fn a_capped_amount_reads_the_stock_up_to_its_cap() {
    let r = rules();
    let mut g = game(&r);
    let read = |g: &GameState, res: &str| programs::reading(&r, g, &Reading::Amount { resource: res.into() });
    set(&mut g, "energy", "25");
    assert_eq!(read(&g, "energy"), Some(f("25")));
    let cap = economy::cap(&r, &g, "energy").unwrap();
    set(&mut g, "energy", "100000");
    assert_eq!(read(&g, "energy"), Some(cap), "as the panel shows it, before the day's clamp");
    // A clause on it: "Energy < 30" against "Energy < 30%".
    set(&mut g, "energy", "40");
    assert!(programs::clause_holds(&r, &g, &amount("energy", Cmp::Gt, "30")));
    // The four comparisons, at the edge: `>=` and `<=` hold on equal, `>` and `<` don't.
    for (cmp, holds) in [(Cmp::Gt, false), (Cmp::Lt, false), (Cmp::Ge, true), (Cmp::Le, true)] {
        assert_eq!(programs::clause_holds(&r, &g, &amount("energy", cmp, "40")), holds, "{cmp:?}");
    }
    assert_eq!(programs::reading(&r, &g, &Reading::Percent { resource: "energy".into() }), Some(f("40") / cap * f("100")));
}

#[test]
fn the_researched_readings_open_with_process_telemetry() {
    let r = rules();
    let mut g = game(&r);
    let read = |g: &GameState, what: Reading| programs::reading(&r, g, &what);
    let offered = |g: &GameState| r.view(g).readings.iter().map(|x| x.label.clone()).collect::<Vec<_>>();
    let rate = one(clause(Reading::EnergyPerDay, Cmp::Lt, "0"));
    let stale = one(clause(Reading::DaysSincePurchase, Cmp::Gt, "5"));
    // Before the research: not offered, a clause taken but locked, reading nothing.
    assert!(!offered(&g).iter().any(|l| l == "Energy per day" || l == "Days since a purchase"));
    put(&r, &mut g, vec![when(rate.clone())]).unwrap();
    assert!(r.view(&g).programs[0].locked[0].as_deref().is_some_and(|w| w.contains("research")));
    assert_eq!(read(&g, Reading::EnergyPerDay), None);
    assert_eq!(read(&g, Reading::DaysSincePurchase), None);
    // Research it (the panel open, past 1.8 Lab, science enough): offered last, and readable.
    g.run.enabled_panels.insert(crate::modules::research::RESEARCH_PANEL.into());
    g.career.flags.insert("research_lab".into());
    set(&mut g, "science", "50");
    r.advance(&mut g, 3);
    r.apply(&mut g, &Action::Research { research: "process_telemetry".into() }).unwrap();
    assert_eq!(g.run.last_purchase, Some(g.run.day), "research is a purchase");
    let labels = offered(&g);
    assert_eq!(&labels[labels.len() - 2..], ["Energy per day", "Days since a purchase"]);
    put(&r, &mut g, vec![when(rate), when(stale)]).unwrap();
    let bought = g.run.day;
    r.advance(&mut g, 7);
    assert_eq!(read(&g, Reading::DaysSincePurchase), Some(Fx::from_int((g.run.day - bought) as i64)));
    assert_eq!(read(&g, Reading::EnergyPerDay), Some(g.run.day_net.get("energy").copied().unwrap_or(Fx::ZERO)));
    // A building bought by hand resets it; so does land.
    set(&mut g, "credits", "100000");
    set(&mut g, "land", "10");
    set(&mut g, "titanium", "50");
    r.apply(&mut g, &Action::Buy { building: "solar_panel".into() }).unwrap();
    assert_eq!(read(&g, Reading::DaysSincePurchase), Some(Fx::ZERO));
    r.advance(&mut g, 2);
    r.apply(&mut g, &Action::BuyLand).unwrap();
    assert_eq!(read(&g, Reading::DaysSincePurchase), Some(Fx::ZERO));
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
fn if_opens_on_its_own_flag() {
    let r = rules();
    let mut g = r.new_game(1);
    g.career.flags.insert(programs::LOGIC_FLAG.into());
    let cond = one(amount("credits", Cmp::Gt, "-1"));
    let iff = vec![when(cond), Entry::command("idle", 1).indent()];
    put(&r, &mut g, iff.clone()).unwrap();
    assert!(r.view(&g).programs[0].locked[0].is_some(), "if needs the Conditional Logic flag");
    assert!(put(&r, &mut g, vec![build("solar_panel", Some(9))]).is_ok(), "build needs only program logic");
    open_if(&mut g);
    assert!(put(&r, &mut g, iff).is_ok());
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
    // Gone: `wait until`, a reading on the right side, comparators but the four.
    let clause = |rest: &str| format!(r#"{{"op":"if","when":{{"first":{{"reading":{{"kind":"amount","resource":"credits"}},{rest}}},"rest":null}}}}"#);
    assert!(serde_json::from_str::<Entry>(&clause(r#""cmp":"gt","value":"5""#)).is_ok());
    for bad in [
        clause(r#""cmp":"eq","value":"5""#),
        clause(r#""cmp":"above","value":"5""#),
        clause(r#""cmp":"gt","value":"5","against":{"kind":"land_price"}"#),
        r#"{"op":"wait_until","when":{"first":{"reading":{"kind":"free_land"},"cmp":"lt","value":"1"},"rest":null}}"#.to_string(),
    ] {
        assert!(serde_json::from_str::<Entry>(&bad).is_err(), "{bad}");
    }
}
