//! `act` lines (`<verb> <target> [amount]`), the readings added with them and a
//! right side that is another reading times a number (docs/rules/programs.md,
//! *Verbs*, *Conditions*).

use super::{f, rules, set};
use crate::modules::{career, economy, programs};
use crate::state::{Amount, Clause, Cmp, Condition, Entry, GameState, Holding, LineOutcome as O, Op, Reading, Verb};
use crate::{Action, Fx, Rules};

fn game(r: &Rules) -> GameState {
    let mut g = r.new_game(1);
    g.career.flags.insert(programs::LOGIC_FLAG.into());
    g
}

fn act(verb: Verb, target: Option<&str>, tab: Option<u32>, amount: Option<Amount>) -> Entry {
    Entry::new(Op::Act { verb, target: target.map(str::to_string), tab, amount })
}

fn put(r: &Rules, g: &mut GameState, entries: Vec<Entry>) -> Result<(), String> {
    r.apply(g, &Action::SetProgram { program: 0, entries })
}

/// Runs line 0 of program 0 once (one step; nothing else in the day matters here).
fn run_one(r: &Rules, g: &mut GameState, e: Entry) -> O {
    put(r, g, vec![e]).unwrap();
    r.advance(g, 1);
    crate::view::LinesRun::today(g).lines[0][0].expect("ran")
}

fn own(r: &Rules, g: &mut GameState, b: &str, n: u32) {
    let _ = r;
    g.run.buildings.insert(b.into(), Holding::starting(n));
    economy::refresh(g);
}

#[test]
fn pause_and_resume_take_n_or_all_and_are_done_with_nothing_to_do() {
    let r = rules();
    let mut g = game(&r);
    own(&r, &mut g, "excavator", 3);
    assert_eq!(run_one(&r, &mut g, act(Verb::Pause, Some("excavator"), None, None)), O::Ran);
    assert_eq!(g.active("excavator"), 2, "no amount: one");
    assert_eq!(run_one(&r, &mut g, act(Verb::Pause, Some("excavator"), None, Some(Amount::N(5)))), O::Ran);
    assert_eq!(g.active("excavator"), 0, "N past what runs pauses what runs");
    assert_eq!(run_one(&r, &mut g, act(Verb::Pause, Some("excavator"), None, Some(Amount::All))), O::Done);
    assert_eq!(run_one(&r, &mut g, act(Verb::Resume, Some("excavator"), None, Some(Amount::All))), O::Ran);
    assert_eq!(g.active("excavator"), 3);
}

#[test]
fn research_buys_once_and_is_done_after() {
    let r = rules();
    let mut g = game(&r);
    let item = crate::content::content()
        .research
        .iter()
        .find(|it| it.requires.is_none() && crate::modules::research::visible(&g, it))
        .expect("an open item")
        .id
        .clone();
    let line = act(Verb::Research, Some(&item), None, None);
    assert_eq!(run_one(&r, &mut g, line.clone()), O::Failed, "no science");
    set(&mut g, "science", "100000");
    assert_eq!(run_one(&r, &mut g, line.clone()), O::Ran);
    assert!(g.run.research.contains(&item));
    assert_eq!(run_one(&r, &mut g, line), O::Done, "already researched: done");
    let read = programs::reading(&r, &g, &Reading::Researched { item: item.clone() });
    assert_eq!(read, Some(Fx::ONE));
    let other = crate::content::content().research.iter().find(|it| !g.run.research.contains(&it.id)).unwrap();
    assert_eq!(programs::reading(&r, &g, &Reading::Researched { item: other.id.clone() }), Some(Fx::ZERO));
}

#[test]
fn pad_verbs_with_nothing_to_do_cost_the_step_only() {
    let r = rules();
    let mut g = game(&r);
    let mut no_pad = g.clone();
    assert_eq!(run_one(&r, &mut no_pad, act(Verb::LaunchPads, None, None, None)), O::Failed, "locked: no Launch Pad");
    own(&r, &mut g, "launch_pad", 1);
    own(&r, &mut g, "solar_panel", 10);
    set(&mut g, "he3", "0");
    set(&mut g, "propellant", "100");
    // The same day with and without the line: only the line's outcome differs.
    let mut bare = g.clone();
    put(&r, &mut bare, vec![]).unwrap();
    r.advance(&mut bare, 1);
    for line in [act(Verb::LaunchPads, None, None, None), act(Verb::LoadPads, Some("he3"), None, None)] {
        let mut h = g.clone();
        assert_eq!(run_one(&r, &mut h, line), O::Done, "no pad ready, nothing to load");
        assert_eq!(h.run.resources, bare.run.resources, "no costs, no boredom");
    }
}

#[test]
fn guard_readings_read_pads_running_and_processors() {
    let r = rules();
    let mut g = game(&r);
    own(&r, &mut g, "launch_pad", 2);
    own(&r, &mut g, "excavator", 3);
    let read = |g: &GameState, what: Reading| programs::reading(&r, g, &what);
    let n = |v: i64| Some(Fx::from_int(v));
    assert_eq!(read(&g, Reading::Pads { good: None }), n(2));
    let on_he3 = g.run.pads.iter().filter(|p| p.good == "he3").count() as i64;
    assert_eq!(read(&g, Reading::Pads { good: Some("he3".into()) }), n(on_he3));
    r.apply(&mut g, &Action::SetActive { building: "excavator".into(), active: 1 }).unwrap();
    assert_eq!(read(&g, Reading::Running { building: "excavator".into() }), n(1));
    assert_eq!(read(&g, Reading::Count { building: "excavator".into() }), n(3));
    assert_eq!(read(&g, Reading::Processors { tab: 0 }), n(i64::from(g.run.programs[0].processors)));
    assert_eq!(read(&g, Reading::Processors { tab: 99 }), None, "no such program reads nothing");
}

/// `free processors` and `idle pads` fold a guard into one reading:
/// `free processors > 0` exactly when `assign processors <tab> all` would add
/// some, `idle pads (all) > idle pads X × 1` exactly when `set pads X` has a
/// pad to switch.
#[test]
fn folded_readings_guard_assign_and_set_pads_exactly() {
    let r = rules();
    let mut g = game(&r);
    own(&r, &mut g, "launch_pad", 2);
    own(&r, &mut g, "solar_panel", 10);
    own(&r, &mut g, "data_center", 3);
    let read = |g: &GameState, what: Reading| programs::reading(&r, g, &what);
    let n = |v: i64| Some(Fx::from_int(v));
    let free = |g: &GameState| programs::capacity(g) - programs::assigned(g);
    assert!(free(&g) > 0, "new Data Centers' processors start unassigned");
    assert_eq!(read(&g, Reading::FreeProcessors), n(i64::from(free(&g))));
    let all = act(Verb::AssignProcessors, None, Some(1), Some(Amount::All));
    assert_eq!(run_one(&r, &mut g, all.clone()), O::Ran);
    r.advance(&mut g, 1);
    assert_eq!(read(&g, Reading::FreeProcessors), n(0));
    assert_eq!(run_one(&r, &mut g, all), O::Done, "none free: nothing to add");
    // Both pads on He-3 and empty: both idle; cargo on one leaves the other.
    for p in &mut g.run.pads {
        p.good = "he3".into();
    }
    let idle = |g: &GameState, good: Option<&str>| read(g, Reading::IdlePads { good: good.map(str::to_string) });
    assert_eq!(idle(&g, None), n(2));
    g.run.pads[0].cargo = f("1");
    assert_eq!((idle(&g, None), idle(&g, Some("he3")), idle(&g, Some("titanium"))), (n(1), n(1), n(0)));
    // `set pads titanium` has a pad to switch exactly when idle pads (all) > idle pads titanium:
    // here the empty He-3 pad, never the loaded one.
    assert!(idle(&g, None) > idle(&g, Some("titanium")));
    let set_ti = act(Verb::SetPads, Some("titanium"), None, Some(Amount::N(2)));
    assert_eq!(run_one(&r, &mut g, set_ti.clone()), O::Ran);
    assert_eq!((g.run.pads[0].good.as_str(), g.run.pads[1].good.as_str()), ("he3", "titanium"), "the loaded pad stays");
    assert_eq!(idle(&g, None), idle(&g, Some("titanium")), "no idle pad on another good left");
    assert_eq!(run_one(&r, &mut g, set_ti), O::Failed, "only the loaded pad could switch");
    let v = r.view(&g);
    assert!(v.readings.iter().any(|x| x.reading == Reading::FreeProcessors));
    assert!(v.readings.iter().any(|x| x.reading == Reading::IdlePads { good: None }));
    assert!(v.readings.iter().any(|x| x.reading == Reading::IdlePads { good: Some("he3".into()) }));
}

#[test]
fn pad_verbs_and_readings_address_pads_by_good() {
    let r = rules();
    let mut g = game(&r);
    own(&r, &mut g, "launch_pad", 2);
    own(&r, &mut g, "solar_panel", 10);
    assert_eq!(g.run.pads.len(), 2);
    let set_ti = act(Verb::SetPads, Some("titanium"), None, Some(Amount::N(1)));
    assert_eq!(run_one(&r, &mut g, set_ti.clone()), O::Ran);
    assert_eq!(g.run.pads.iter().filter(|p| p.good == "titanium").count(), 1);
    assert_eq!(run_one(&r, &mut g, set_ti), O::Done, "one carries it already");
    set(&mut g, "titanium", "40");
    set(&mut g, "he3", "0");
    assert_eq!(run_one(&r, &mut g, act(Verb::LoadPads, Some("titanium"), None, None)), O::Ran);
    let read = |g: &GameState, what: Reading| programs::reading(&r, g, &what);
    let per = crate::modules::shipments::load_amount(&r, &g);
    assert_eq!(read(&g, Reading::Cargo { good: Some("titanium".into()) }), Some(per));
    assert_eq!(read(&g, Reading::Cargo { good: Some("he3".into()) }), Some(Fx::ZERO));
    assert_eq!(read(&g, Reading::Cargo { good: None }), Some(per));
    // A full titanium pad, fuel aboard: ready, and `launch pads titanium` ships it alone.
    let ti = g.run.pads.iter().position(|p| p.good == "titanium").unwrap();
    g.run.pads[ti].cargo = r.p.shipments.pad_cargo_capacity;
    set(&mut g, "propellant", "100");
    assert_eq!(read(&g, Reading::PadsReady { good: Some("titanium".into()) }), Some(Fx::ONE));
    assert_eq!(read(&g, Reading::PadsReady { good: Some("he3".into()) }), Some(Fx::ZERO));
    assert_eq!(read(&g, Reading::DaysSinceLaunch), Some(Fx::from_int(g.run.day as i64)), "none yet: the run's day");
    // Paused, it doesn't launch, so it doesn't read ready: the guard matches the verb.
    g.run.pads[ti].paused = true;
    assert_eq!(read(&g, Reading::PadsReady { good: Some("titanium".into()) }), Some(Fx::ZERO));
    assert_eq!(run_one(&r, &mut g, act(Verb::LaunchPads, Some("titanium"), None, None)), O::Done);
    g.run.pads[ti].paused = false;
    assert_eq!(run_one(&r, &mut g, act(Verb::LaunchPads, Some("titanium"), None, None)), O::Ran);
    assert_eq!(g.run.stats.shipments, 1);
    assert_eq!(read(&g, Reading::DaysSinceLaunch), Some(Fx::ONE), "launched yesterday");
}

#[test]
fn load_program_and_assign_processors_reach_other_tabs() {
    let r = rules();
    let mut g = game(&r);
    own(&r, &mut g, "data_center", 3);
    own(&r, &mut g, "solar_panel", 10);
    put(&r, &mut g, vec![Entry::command("idle", 2)]).unwrap();
    r.apply(&mut g, &Action::SaveLoadout { program: 0, slot: None, name: "Idle".into() }).unwrap();
    assert_eq!(run_one(&r, &mut g, act(Verb::LoadProgram, Some("Idle"), Some(2), None)), O::Ran);
    assert_eq!(g.run.programs[2].entries, vec![Entry::command("idle", 2)]);
    assert_eq!(run_one(&r, &mut g, act(Verb::LoadProgram, Some("missing"), Some(2), None)), O::Failed);
    let free = programs::capacity(&g) - programs::assigned(&g);
    assert!(free > 0);
    assert_eq!(run_one(&r, &mut g, act(Verb::AssignProcessors, None, Some(2), Some(Amount::All))), O::Ran);
    assert_eq!(g.run.programs[2].processors, free, "all: every processor not on another program");
    assert_eq!(run_one(&r, &mut g, act(Verb::AssignProcessors, None, Some(2), Some(Amount::All))), O::Done);
    assert_eq!(run_one(&r, &mut g, act(Verb::AssignProcessors, None, Some(2), Some(Amount::N(99)))), O::Failed);
    // Loading into its own tab: the new lines start at the top.
    put(&r, &mut g, vec![act(Verb::LoadProgram, Some("Idle"), Some(0), None)]).unwrap();
    r.advance(&mut g, 1);
    assert_eq!(g.run.programs[0].entries, vec![Entry::command("idle", 2)]);
}

#[test]
fn retire_fails_until_retiring_is_allowed_then_retires_at_the_days_end() {
    let r = rules();
    let mut g = game(&r);
    assert_eq!(run_one(&r, &mut g, act(Verb::Retire, None, None, None)), O::Failed);
    g.run.enabled_panels.insert(career::RETIREMENT_PANEL.into());
    put(&r, &mut g, vec![act(Verb::Retire, None, None, None)]).unwrap();
    let run = g.career.run_number;
    r.advance(&mut g, 1);
    assert_eq!(g.career.run_number, run + 1);
    let last = g.career.last_run.as_ref().unwrap();
    assert!(!last.forced, "a voluntary retirement, summary and all");
    assert_eq!(last.days, 2, "the day it ran, played out");
}

#[test]
fn a_right_side_reading_times_a_number_and_the_new_readings() {
    let r = rules();
    let mut g = game(&r);
    own(&r, &mut g, "solar_panel", 3);
    own(&r, &mut g, "excavator", 2);
    let count = |b: &str| Reading::Count { building: b.into() };
    let times = |cmp, n: &str| Clause { reading: count("solar_panel"), cmp, value: f(n), against: Some(count("excavator")) };
    assert!(programs::clause_holds(&r, &g, &times(Cmp::Lt, "2")), "3 < 2 × 2");
    assert!(!programs::clause_holds(&r, &g, &times(Cmp::Lt, "1.5")), "3 < 2 × 1.5 is false");
    let nothing = Clause { against: Some(Reading::Percent { resource: "credits".into() }), ..times(Cmp::Lt, "2") };
    assert!(!programs::clause_holds(&r, &g, &nothing), "a right side reading nothing is false");
    let read = |g: &GameState, what: Reading| programs::reading(&r, g, &what);
    let b = crate::content::content().building("solar_panel").unwrap();
    assert_eq!(read(&g, Reading::NextCost { building: "solar_panel".into() }), economy::price(&r, &g, b).get("credits").copied());
    assert_eq!(read(&g, Reading::LandPrice), Some(economy::land_price(&r, &g)));
    let every = Reading::Every { days: 3 };
    assert_eq!(read(&g, every.clone()), Some(Fx::ONE), "day 0");
    g.run.day = 4;
    assert_eq!(read(&g, every.clone()), Some(Fx::ZERO));
    g.run.day = 6;
    assert_eq!(read(&g, every), Some(Fx::ONE));
    // Days to cap: what is left over the rate; nothing at a rate of 0 or less.
    let cap = economy::cap(&r, &g, "regolith").unwrap();
    set(&mut g, "regolith", "0");
    g.run.day_net.insert("regolith".into(), cap / f("4"));
    assert_eq!(read(&g, Reading::DaysToCap { resource: "regolith".into() }), Some(f("4")));
    g.run.day_net.insert("regolith".into(), Fx::ZERO);
    assert_eq!(read(&g, Reading::DaysToCap { resource: "regolith".into() }), None);
    g.run.resources.insert("regolith".into(), cap);
    assert_eq!(read(&g, Reading::DaysToCap { resource: "regolith".into() }), Some(Fx::ZERO));
}

#[test]
fn set_program_refuses_act_lines_with_the_wrong_slots() {
    let r = rules();
    let mut g = game(&r);
    let bad = [
        act(Verb::Pause, Some("no_such"), None, None),
        act(Verb::Pause, Some("excavator"), None, Some(Amount::N(0))),
        act(Verb::Research, None, None, None),
        act(Verb::SetPads, Some("he3"), None, None),
        act(Verb::SetPads, Some("he3"), None, Some(Amount::All)),
        act(Verb::LaunchPads, Some("regolith"), None, None),
        act(Verb::AssignProcessors, None, Some(99), Some(Amount::N(1))),
        act(Verb::Retire, Some("x"), None, None),
        act(Verb::LoadProgram, Some("x"), None, None),
    ];
    for e in bad {
        assert!(put(&r, &mut g, vec![e.clone()]).is_err(), "{e:?}");
    }
    let every0 = Entry::new(Op::If {
        when: Condition { first: Clause { reading: Reading::Every { days: 0 }, cmp: Cmp::Eq, value: Fx::ONE, against: None }, rest: None },
    });
    assert!(put(&r, &mut g, vec![every0]).is_err());
    // Before the logic unlock, a verb line is refused like any line but a command.
    g.career.flags.clear();
    assert!(put(&r, &mut g, vec![act(Verb::Retire, None, None, None)]).is_err());
}

fn when(reading: Reading, cmp: Cmp, v: i64) -> Entry {
    Entry::new(Op::If { when: Condition { first: Clause { reading, cmp, value: Fx::from_int(v), against: None }, rest: None } })
}

#[test]
fn days_since_purchase_counts_buildings_land_and_research() {
    let r = rules();
    let mut g = game(&r);
    let read = |g: &GameState| programs::reading(&r, g, &Reading::DaysSincePurchase);
    r.advance(&mut g, 3);
    assert_eq!(read(&g), Some(Fx::from_int(g.run.day as i64)), "none yet: the run's day");
    set(&mut g, "credits", "100000");
    set(&mut g, "titanium", "1000");
    r.apply(&mut g, &Action::Buy { building: "solar_panel".into() }).unwrap();
    assert_eq!(read(&g), Some(Fx::ZERO), "a building today");
    r.advance(&mut g, 2);
    assert_eq!(read(&g), Some(Fx::from_int(2)));
    r.apply(&mut g, &Action::BuyLand).unwrap();
    assert_eq!(read(&g), Some(Fx::ZERO), "land");
    r.advance(&mut g, 2);
    let item = crate::content::content()
        .research
        .iter()
        .find(|it| it.requires.is_none() && crate::modules::research::visible(&g, it))
        .expect("an open item")
        .id
        .clone();
    set(&mut g, "science", "100000");
    r.apply(&mut g, &Action::Research { research: item }).unwrap();
    assert_eq!(read(&g), Some(Fx::ZERO), "research");
    // A command that buys a resource is not a purchase.
    r.advance(&mut g, 1);
    assert_eq!(run_one(&r, &mut g, Entry::new(Op::Command { command: "idle".into(), count: 1 })), O::Ran);
    assert_eq!(read(&g), Some(Fx::from_int(2)));
}

#[test]
fn fund_sets_every_cost_rate_as_the_panel_does() {
    let r = rules();
    let mut g = game(&r);
    let fund = |a| act(Verb::Fund, Some("foundation_grant"), None, a);
    // Locked: fails, no rate.
    assert_eq!(run_one(&r, &mut g, fund(Some(Amount::N(3)))), O::Failed);
    assert!(g.run.project_rates.is_empty());
    g.run.enabled_projects.insert("foundation_grant".into());
    set(&mut g, "credits", "0");
    set(&mut g, "science", "0");
    let rates = |g: &GameState| g.run.project_rates.get("foundation_grant").cloned().unwrap_or_default();
    assert_eq!(run_one(&r, &mut g, fund(Some(Amount::N(3)))), O::Ran);
    assert_eq!(rates(&g).get("credits"), Some(&Fx::from_int(3)));
    assert_eq!(rates(&g).get("science"), Some(&Fx::from_int(3)));
    assert_eq!(run_one(&r, &mut g, fund(Some(Amount::N(3)))), O::Done, "already at that rate: the step only");
    // The most a day is each cost over min_days (167): 5000 and 2000.
    let fg = crate::content::content().project("foundation_grant").unwrap();
    let most = |res: &str| crate::modules::projects::max_rate(&g, fg, res);
    let (credits_max, science_max) = (most("credits"), most("science"));
    assert_eq!(credits_max, f("29.94012"), "5000 / 167 rounded up a step: done in exactly 167 days");
    assert_eq!(run_one(&r, &mut g, fund(Some(Amount::N(1_000_000)))), O::Ran);
    assert_eq!(rates(&g).get("credits"), Some(&credits_max), "capped at the most a day");
    assert_eq!(run_one(&r, &mut g, fund(Some(Amount::N(20)))), O::Ran);
    assert_eq!(rates(&g).get("credits"), Some(&Fx::from_int(20)));
    assert_eq!(rates(&g).get("science"), Some(&science_max), "each cost capped on its own");
    assert_eq!(run_one(&r, &mut g, fund(Some(Amount::N(0)))), O::Ran);
    assert!(rates(&g).is_empty(), "0 stops it");
    assert_eq!(run_one(&r, &mut g, fund(None)), O::Ran);
    assert_eq!(rates(&g).get("science"), Some(&science_max), "no amount: the most, as all");
    // The `completed` reading guards it.
    let done = Reading::Completed { project: "foundation_grant".into() };
    assert_eq!(programs::reading(&r, &g, &done), Some(Fx::ZERO));
    // Completed: no longer open, so it fails.
    g.career.projects.completions.insert("foundation_grant".into(), 1);
    assert_eq!(programs::reading(&r, &g, &done), Some(Fx::ONE));
    assert_eq!(run_one(&r, &mut g, fund(Some(Amount::All))), O::Failed);
    assert!(put(&r, &mut g, vec![act(Verb::Fund, Some("no_such"), None, None)]).is_err());
    assert!(put(&r, &mut g, vec![act(Verb::Fund, None, None, None)]).is_err());
}

#[test]
fn the_fund_verb_and_purchase_reading_switch_off_in_config() {
    let json = super::BASELINE.replace("\"fund_verb\": 1", "\"fund_verb\": 0").replace(
        "\"days_since_purchase_reading\": 1",
        "\"days_since_purchase_reading\": 0",
    );
    let r = Rules::new(&crate::Config::from_json(&json).unwrap()).unwrap();
    let mut g = game(&r);
    let fund = act(Verb::Fund, Some("foundation_grant"), None, Some(Amount::All));
    let stale = when(Reading::DaysSincePurchase, Cmp::Ge, 1);
    assert!(put(&r, &mut g, vec![fund.clone()]).is_err(), "a new fund line is refused");
    assert!(put(&r, &mut g, vec![stale.clone(), act(Verb::Retire, None, None, None).indent()]).is_err());
    assert_eq!(programs::reading(&r, &g, &Reading::DaysSincePurchase), None);
    let v = crate::view::build(&r, &g);
    assert!(!v.verbs.iter().any(|x| x.verb == Verb::Fund));
    assert!(!v.readings.iter().any(|x| x.reading == Reading::DaysSincePurchase));
    // Held (as a loadout brings it), the fund line fails when run.
    g.run.enabled_projects.insert("foundation_grant".into());
    g.run.programs[0].entries = vec![fund];
    r.advance(&mut g, 1);
    assert_eq!(crate::view::LinesRun::today(&g).lines[0][0], Some(O::Failed));
    assert!(g.run.project_rates.is_empty());
}

#[test]
fn an_act_line_is_flat_json() {
    let e = act(Verb::SetPads, Some("he3"), None, Some(Amount::N(2)));
    let json = serde_json::to_string(&e).unwrap();
    assert_eq!(json, r#"{"op":"act","verb":"set_pads","target":"he3","amount":{"n":2}}"#);
    let back: Entry = serde_json::from_str(r#"{"op":"act","verb":"assign_processors","tab":1,"amount":"all"}"#).unwrap();
    assert_eq!(back, act(Verb::AssignProcessors, None, Some(1), Some(Amount::All)));
}
