//! The draft later acts (docs/rules/timeline.md): the calendar, a timeline's
//! end, Earth techs, the vault and the ending.

use super::{f, rules, set};
use crate::action::Action;
use crate::modules::timeline;
use crate::GameState;

fn revealed(r: &crate::Rules) -> GameState {
    let mut g = r.new_game(7);
    g.career.flags.insert(timeline::REVEAL_FLAG.into());
    g.career.completed_quests.insert("qpower_boot_sequence".into());
    r.advance(&mut g, 1);
    g
}

#[test]
fn nothing_runs_before_the_reveal() {
    let r = rules();
    let mut g = r.new_game(7);
    r.advance(&mut g, 50);
    assert!(g.saga.is_default(), "no calendar before the reveal");
    assert!(!g.to_json().contains("saga"), "the saga stays out of an Act 1 game's JSON");
}

#[test]
fn the_calendar_sends_earth_science_and_retirements_cost_days() {
    let r = rules();
    let mut g = revealed(&r);
    let t = g.saga.timeline.clone().unwrap();
    assert_eq!((t.number, t.day), (1, 1));
    assert_eq!(t.arrival, 3000, "arrival_days at strength 1");
    set(&mut g, "earth_science", "50");
    r.advance(&mut g, 1);
    assert_eq!(g.saga.science, f("50"));
    assert_eq!(g.amount("earth_science"), f("0"));
    let day = g.saga.timeline.as_ref().unwrap().day;
    g.run.enabled_panels.insert("retirement".into());
    r.apply(&mut g, &Action::Retire).unwrap();
    assert_eq!(g.saga.timeline.as_ref().unwrap().day, day + 30, "retire_days");
}

#[test]
fn the_swarm_ends_a_timeline_and_the_mind_keeps_what_it_learned() {
    let r = rules();
    let mut g = revealed(&r);
    g.career.retirements = 4;
    g.career.best_run_days = 900;
    g.saga.science = f("1000");
    r.apply(&mut g, &Action::BuyEarthTech { tech: "boredom_dampers".into() }).unwrap();
    g.saga.science = f("0");
    g.saga.timeline.as_mut().unwrap().science_made = f("4000");
    let arrival = g.saga.timeline.as_ref().unwrap().arrival;
    let day = g.saga.timeline.as_ref().unwrap().day;
    r.advance(&mut g, arrival - day);
    assert_eq!(g.saga.ended, 1, "the first swarm always wins");
    assert_eq!(g.saga.repelled, 0);
    let t = g.saga.timeline.as_ref().unwrap();
    assert_eq!(t.number, 2);
    assert_eq!(g.career.run_number, 1);
    assert_eq!(g.career.retirements, 0);
    assert_eq!(g.career.best_run_days, 0);
    assert!(g.career.completed_quests.contains("qpower_boot_sequence"), "quests are kept");
    assert!(g.saga.techs.contains_key("boredom_dampers"), "a forever tech is kept");
    assert_eq!(g.saga.science, f("1000"), "science_carry of what was made");
    assert_eq!(g.saga.history[0].outcome, "fell");
    assert_eq!(crate::modifiers::modifier(&r, &g, "boredom_growth_mult"), f("0.85"));
}

#[test]
fn a_later_swarm_can_be_repelled_once_and_returns_entangled() {
    let r = rules();
    let mut g = revealed(&r);
    g.saga.ended = 1;
    g.saga.timeline.as_mut().unwrap().science_made = f("3000");
    let t = g.saga.timeline.clone().unwrap();
    r.advance(&mut g, t.arrival - t.day);
    assert_eq!(g.saga.repelled, 1);
    assert!(g.saga.entangled);
    let t2 = g.saga.timeline.clone().unwrap();
    assert_eq!(t2.number, 1, "the timeline goes on");
    assert_eq!(t2.arrival, t.arrival + 600);
    assert_eq!(t2.strength, f("1.5"));
}

#[test]
fn techs_need_science_and_their_quest_and_scopes_expire() {
    let r = rules();
    let mut g = revealed(&r);
    g.saga.science = f("5000");
    assert!(r.apply(&mut g, &Action::BuyEarthTech { tech: "compressed_blocks".into() }).is_err(), "needs Q9");
    g.career.completed_quests.insert("qearth_science".into());
    r.apply(&mut g, &Action::BuyEarthTech { tech: "compressed_blocks".into() }).unwrap();
    assert_eq!(g.saga.science, f("4400"));
    let t = g.saga.timeline.clone().unwrap();
    r.advance(&mut g, t.arrival - t.day);
    assert_eq!(g.saga.timeline.as_ref().unwrap().number, 2);
    // Run 1 of the new timeline got the blocks (on top of its usual start).
    assert!(g.amount("titanium") >= f("20"));
    assert_eq!(g.saga.techs.get("compressed_blocks"), Some(&0), "next_timeline: one more timeline");
    let t = g.saga.timeline.clone().unwrap();
    r.advance(&mut g, t.arrival - t.day);
    assert!(!g.saga.techs.contains_key("compressed_blocks"));
    assert!(g.saga.pending_start.is_empty() && g.amount("titanium") < f("20"), "the blocks paid once");
}

#[test]
fn the_consensus_engine_ends_zero_sum_funding() {
    let r = rules();
    let mut g = revealed(&r);
    g.saga.techs.insert("consensus_engine".into(), u32::MAX);
    crate::modules::ideology::fund(&r, &mut g, "humanist");
    assert_eq!(g.run.ideology["nationalist"], f("0"));
}

#[test]
fn the_vault_buys_a_super_timeline_and_a_finished_sphere_wins() {
    let r = rules();
    let mut g = revealed(&r);
    assert!(r.apply(&mut g, &Action::Deposit { resource: "tachyon".into() }).is_err(), "needs the vault");
    g.saga.techs.insert("temporal_vault".into(), u32::MAX);
    set(&mut g, "tachyon", "3000");
    r.apply(&mut g, &Action::Deposit { resource: "tachyon".into() }).unwrap();
    assert!(r.apply(&mut g, &Action::BeginSuperTimeline).is_err(), "Q15 first");
    g.career.completed_quests.insert("qfilling_vault".into());
    g.career.completed_projects.insert("foundation_grant".into());
    r.apply(&mut g, &Action::BeginSuperTimeline).unwrap();
    let t = g.saga.timeline.clone().unwrap();
    assert!(t.super_timeline);
    assert_eq!(t.arrival, 6000);
    assert!(g.career.completed_projects.contains("foundation_grant"), "projects survive into it");
    // A retirement inside it keeps resources.
    set(&mut g, "neodymium", "40");
    g.run.enabled_panels.insert("retirement".into());
    r.apply(&mut g, &Action::Retire).unwrap();
    assert_eq!(g.amount("neodymium"), f("40"));
    // Earth science made on a retirement day still reaches Earth.
    set(&mut g, "earth_science", "7");
    let made = g.saga.timeline.as_ref().unwrap().science_made;
    g.run.enabled_panels.insert("retirement".into());
    r.apply(&mut g, &Action::Retire).unwrap();
    assert_eq!(g.saga.timeline.as_ref().unwrap().science_made, made + f("7"));
    g.career.completed_projects.insert(timeline::DYSON.into());
    let t = g.saga.timeline.clone().unwrap();
    r.advance(&mut g, t.arrival - t.day);
    assert!(g.saga.victory);
    assert_eq!(g.saga.history.last().unwrap().outcome, "victory");
    let day = g.saga.timeline.as_ref().unwrap().day;
    r.advance(&mut g, 10);
    assert_eq!(g.saga.timeline.as_ref().unwrap().day, day, "the calendar stops");
}

#[test]
fn quiet_days_count_from_the_last_order() {
    let r = rules();
    let mut g = r.new_game(7);
    r.advance(&mut g, 10);
    r.apply(&mut g, &Action::BuyLand).ok();
    r.apply(&mut g, &Action::SetActive { building: "solar_panel".into(), active: 1 }).unwrap();
    assert_eq!(g.run.last_order, Some(10));
    let c = crate::content::Condition::QuietDays { days: 5 };
    assert!(!crate::modules::events::holds(&r, &g, "", &c));
    r.advance(&mut g, 4);
    assert!(crate::modules::events::holds(&r, &g, "", &c));
}
