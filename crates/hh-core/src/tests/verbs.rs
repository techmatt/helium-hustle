//! `act` lines (`<verb> <target> [amount]`) and the readings that guard them
//! (docs/rules/programs.md, *Verbs*, *Conditions*).

use super::{f, logic_open, outcomes, rules, set};
use crate::modules::{career, economy, programs};
use crate::state::{Amount, Entry, GameState, Holding, LineOutcome as O, Op, Reading, Verb};
use crate::{Action, Fx, Rules};

/// A new game with every line open and research too (the Lab quest's panel).
fn game(r: &Rules) -> GameState {
    let mut g = logic_open(r);
    g.run.enabled_panels.insert(crate::modules::research::RESEARCH_PANEL.into());
    // Launch Full Pads opens with 1.5 Processors.
    g.career.completed_quests.insert("qautomate_automation".into());
    g
}

fn act(verb: Verb, target: Option<&str>, tab: Option<u32>, amount: Option<Amount>) -> Entry {
    Entry::new(Op::Act { verb, target: target.map(str::to_string), tab, amount })
}

fn put(r: &Rules, g: &mut GameState, entries: Vec<Entry>) -> Result<(), String> {
    r.apply(g, &Action::SetProgram { program: 0, entries })
}

/// Runs line 0 of program 0 once (one step; nothing else in the day matters
/// here): its outcome, `None` for a locked line (skipped free).
fn run_line(r: &Rules, g: &mut GameState, e: Entry) -> Option<O> {
    put(r, g, vec![e]).unwrap();
    r.advance(g, 1);
    outcomes(g)[0]
}

/// [`run_line`] for a line that runs.
fn run_one(r: &Rules, g: &mut GameState, e: Entry) -> O {
    run_line(r, g, e).expect("ran")
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
    // Excavators aren't unlocked in a new game: the line is locked, skipped free.
    own(&r, &mut g, "excavator", 3);
    assert_eq!(run_line(&r, &mut g, act(Verb::Pause, Some("excavator"), None, None)), None);
    assert_eq!(g.active("excavator"), 3);
    own(&r, &mut g, "solar_panel", 3);
    assert_eq!(run_one(&r, &mut g, act(Verb::Pause, Some("solar_panel"), None, None)), O::Ran);
    assert_eq!(g.active("solar_panel"), 2, "no amount: one");
    assert_eq!(run_one(&r, &mut g, act(Verb::Pause, Some("solar_panel"), None, Some(Amount::N(5)))), O::Ran);
    assert_eq!(g.active("solar_panel"), 0, "N past what runs pauses what runs");
    assert_eq!(run_one(&r, &mut g, act(Verb::Pause, Some("solar_panel"), None, Some(Amount::All))), O::Done);
    assert_eq!(run_one(&r, &mut g, act(Verb::Resume, Some("solar_panel"), None, Some(Amount::All))), O::Ran);
    assert_eq!(g.active("solar_panel"), 3);
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
}

#[test]
fn pad_verbs_with_nothing_to_do_cost_the_step_only() {
    let r = rules();
    let mut g = game(&r);
    let mut early = g.clone();
    early.career.completed_quests.remove("qautomate_automation");
    assert_eq!(run_line(&r, &mut early, act(Verb::LaunchPads, None, None, None)), None, "locked (before 1.5): skipped free");
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

/// `free processors` folds a guard into one reading: above 0 exactly when
/// `assign processors <tab> all` would add some; `idle pads` counts pads with
/// no cargo aboard, what `set pads` may switch.
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
    // New pads have No cargo: both idle.
    let idle = |g: &GameState| read(g, Reading::IdlePads);
    assert!(g.run.pads.iter().all(|p| p.good.is_none()));
    assert_eq!(idle(&g), n(2));
    // Both pads on He-3 and empty: both idle; cargo on one leaves the other.
    for p in &mut g.run.pads {
        p.good = Some("he3".into());
    }
    assert_eq!(idle(&g), n(2));
    g.run.pads[0].cargo = f("1");
    assert_eq!(idle(&g), n(1));
    // `set pads titanium` switches the empty He-3 pad, never the loaded one.
    let set_ti = act(Verb::SetPads, Some("titanium"), None, Some(Amount::N(2)));
    assert_eq!(run_one(&r, &mut g, set_ti.clone()), O::Ran);
    assert_eq!((g.run.pads[0].good.as_deref(), g.run.pads[1].good.as_deref()), (Some("he3"), Some("titanium")), "the loaded pad stays");
    assert_eq!(run_one(&r, &mut g, set_ti), O::Failed, "only the loaded pad could switch");
    let v = r.view(&g);
    assert!(v.readings.iter().any(|x| x.reading == Reading::FreeProcessors));
    assert!(v.readings.iter().any(|x| x.reading == Reading::IdlePads));
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
    assert_eq!(g.run.pads.iter().filter(|p| p.good.as_deref() == Some("titanium")).count(), 1);
    assert_eq!(g.run.pads[0].good.as_deref(), Some("titanium"), "a No-cargo pad switches, in pad order");
    assert_eq!(run_one(&r, &mut g, set_ti), O::Done, "one carries it already");
    set(&mut g, "titanium", "40");
    set(&mut g, "he3", "0");
    assert_eq!(run_one(&r, &mut g, act(Verb::LoadPads, Some("titanium"), None, None)), O::Ran);
    let per = crate::modules::shipments::load_amount(&r, &g);
    let cargo = |g: &GameState, good: &str| g.run.pads.iter().filter(|p| p.good.as_deref() == Some(good)).fold(Fx::ZERO, |s, p| s + p.cargo);
    assert_eq!((cargo(&g, "titanium"), cargo(&g, "he3")), (per, Fx::ZERO));
    // A full titanium pad, fuel aboard: `launch pads titanium` ships it alone.
    let ti = g.run.pads.iter().position(|p| p.good.as_deref() == Some("titanium")).unwrap();
    g.run.pads[ti].cargo = r.p.shipments.pad_cargo_capacity;
    set(&mut g, "propellant", "100");
    // Paused, it doesn't launch: done.
    g.run.pads[ti].paused = true;
    assert_eq!(run_one(&r, &mut g, act(Verb::LaunchPads, Some("titanium"), None, None)), O::Done);
    g.run.pads[ti].paused = false;
    assert_eq!(run_one(&r, &mut g, act(Verb::LaunchPads, Some("titanium"), None, None)), O::Ran);
    assert_eq!(g.run.stats.shipments, 1);
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
    // Before the logic unlock, a verb line is taken but locked, like any line but a command.
    g.career.flags.clear();
    put(&r, &mut g, vec![act(Verb::Retire, None, None, None)]).unwrap();
    assert!(r.view(&g).programs[0].locked[0].is_some());
}

#[test]
fn fund_sets_every_cost_rate_as_the_panel_does() {
    let r = rules();
    let mut g = game(&r);
    let fund = |a| act(Verb::Fund, Some("foundation_grant"), None, a);
    // Locked: skipped free, no rate.
    assert_eq!(run_line(&r, &mut g, fund(Some(Amount::N(3)))), None);
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
    // Completed: no longer open, so it is locked (skipped free).
    g.career.projects.completions.insert("foundation_grant".into(), 1);
    assert_eq!(run_line(&r, &mut g, fund(Some(Amount::All))), None);
    assert!(put(&r, &mut g, vec![act(Verb::Fund, Some("no_such"), None, None)]).is_err());
    assert!(put(&r, &mut g, vec![act(Verb::Fund, None, None, None)]).is_err());
}

#[test]
fn the_fund_verb_switches_off_in_config() {
    let json = super::BASELINE.replace("\"fund_verb\": 1", "\"fund_verb\": 0");
    let r = Rules::new(&crate::Config::from_json(&json).unwrap()).unwrap();
    let mut g = game(&r);
    let fund = act(Verb::Fund, Some("foundation_grant"), None, Some(Amount::All));
    put(&r, &mut g, vec![fund.clone()]).unwrap();
    assert_eq!(r.view(&g).programs[0].locked[0].as_deref(), Some("the fund verb is off"));
    let v = crate::view::build(&r, &g);
    assert!(!v.verbs.iter().any(|x| x.verb == Verb::Fund));
    // Held (as a loadout brings it), the fund line is skipped free when run.
    g.run.enabled_projects.insert("foundation_grant".into());
    g.run.programs[0].entries = vec![fund];
    r.advance(&mut g, 1);
    assert_eq!(outcomes(&g)[0], None);
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
