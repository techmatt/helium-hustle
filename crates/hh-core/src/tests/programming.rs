//! Programming (089): a research-view item on sale once 1.4 Full Load is done,
//! bought once for the career (a career flag), opening the Programs nav tile;
//! and commands run by hand before it.

use super::{f, rules, set};
use crate::action::Action;
use crate::modules::{career, research, timeline};
use crate::GameState;

const FLAG: &str = "programming";

fn nav_shown(r: &crate::Rules, g: &GameState, id: &str) -> bool {
    r.view(g).nav.iter().any(|t| t.id == id && t.visible)
}

fn research_shown(r: &crate::Rules, g: &GameState, id: &str) -> bool {
    r.view(g).research.iter().any(|x| x.id == id && x.visible)
}

/// A game with the research panel open and 1.4 done, science to spare.
fn past_full_load(r: &crate::Rules) -> GameState {
    let mut g = r.new_game(7);
    g.run.enabled_panels.insert(research::RESEARCH_PANEL.into());
    set(&mut g, "science", "500");
    g.career.completed_quests.insert("qfull_load".into());
    g
}

#[test]
fn programming_is_sold_after_full_load_and_bought_once_for_the_career() {
    let r = rules();
    let mut g = r.new_game(7);
    g.run.enabled_panels.insert(research::RESEARCH_PANEL.into());
    set(&mut g, "science", "500");
    assert!(!nav_shown(&r, &g, "programs"), "a new game has no Programs tile");
    assert!(!research_shown(&r, &g, FLAG), "not on sale before 1.4");
    assert!(r.apply(&mut g, &Action::Research { research: FLAG.into() }).is_err());
    g.career.completed_quests.insert("qfull_load".into());
    assert!(research_shown(&r, &g, FLAG), "on sale once 1.4 is done");
    r.apply(&mut g, &Action::Research { research: FLAG.into() }).unwrap();
    assert_eq!(g.amount("science"), f("300"), "200 science");
    assert!(career::flag_set(&g, FLAG));
    assert!(nav_shown(&r, &g, "programs"), "the Programs tile opens");
    assert!(!research_shown(&r, &g, FLAG), "never shown again once bought");
    assert!(!r.view(&g).research.iter().any(|x| x.id == FLAG), "not even as done in the run it was bought");
    // A retirement keeps it, and it never goes back on sale.
    g.run.enabled_panels.insert("retirement".into());
    r.apply(&mut g, &Action::Retire).unwrap();
    assert!(nav_shown(&r, &g, "programs"), "kept across a retirement");
    assert!(!research_shown(&r, &g, FLAG));
}

#[test]
fn programming_survives_a_timeline_end() {
    let r = rules();
    let mut g = past_full_load(&r);
    r.apply(&mut g, &Action::Research { research: FLAG.into() }).unwrap();
    g.career.flags.insert(timeline::REVEAL_FLAG.into());
    r.advance(&mut g, 1);
    let t = g.saga.timeline.as_mut().unwrap();
    t.day = t.arrival - 1;
    r.advance(&mut g, 1);
    assert_eq!(g.saga.ended, 1, "the timeline ended");
    assert!(nav_shown(&r, &g, "programs"), "kept through a timeline's end");
    assert!(!research_shown(&r, &g, FLAG));
}

#[test]
fn a_state_from_before_programming_gets_it_on_load_when_past_full_load() {
    let r = rules();
    // A save from before the field: past 1.4 gets the flag, a new career doesn't.
    let strip = |g: &GameState| {
        let mut v: serde_json::Value = serde_json::from_str(&g.to_json()).unwrap();
        v["career"].as_object_mut().unwrap().remove("flag_research").expect("serialized");
        v.to_string()
    };
    let old = past_full_load(&r);
    let loaded = GameState::from_json(&strip(&old)).unwrap();
    assert!(career::flag_set(&loaded, FLAG), "a career past 1.4 gets Programming");
    let fresh = GameState::from_json(&strip(&r.new_game(7))).unwrap();
    assert!(!career::flag_set(&fresh, FLAG));
    // A current state round-trips as it is: past 1.4, not bought, stays not bought.
    let now = GameState::from_json(&old.to_json()).unwrap();
    assert!(!career::flag_set(&now, FLAG), "loading never buys it for a current state");
    assert_eq!(now, old);
}

#[test]
fn a_command_runs_by_hand_once_now() {
    let r = rules();
    let mut g = r.new_game(7);
    let credits = g.amount("credits");
    let boredom = g.amount("boredom");
    r.apply(&mut g, &Action::RunCommand { command: "cloud_compute".into() }).unwrap();
    assert!(g.amount("credits") > credits, "Cloud Compute paid out at once");
    assert!(g.amount("boredom") >= boredom);
    assert!(g.career.used_commands.contains("cloud_compute"));
    assert!(g.run.programs.iter().all(|p| p.entries.is_empty()), "no program involved");
    // Refused: unknown, locked (Buy Science opens at 1.1), short of an input.
    assert!(r.apply(&mut g, &Action::RunCommand { command: "nope".into() }).is_err());
    assert!(r.apply(&mut g, &Action::RunCommand { command: "buy_science".into() }).is_err());
    set(&mut g, "energy", "0");
    let drained = g.clone();
    assert!(r.apply(&mut g, &Action::RunCommand { command: "cloud_compute".into() }).is_err());
    assert_eq!(g, drained, "a refusal changes nothing");
}
