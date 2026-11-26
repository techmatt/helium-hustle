//! Programming (089, 091): a research-view item on sale once 1.3 Full Load is
//! done, bought once for the career (a career flag), opening the Programs nav
//! tile (the editors) and nothing else: programs run from day 1.

use super::{f, rules, set};
use crate::action::Action;
use crate::modules::{career, research, timeline};
use crate::state::Entry;
use crate::GameState;

const FLAG: &str = "programming";

fn nav_shown(r: &crate::Rules, g: &GameState, id: &str) -> bool {
    r.view(g).nav.iter().any(|t| t.id == id && t.visible)
}

fn research_shown(r: &crate::Rules, g: &GameState, id: &str) -> bool {
    r.view(g).research.iter().any(|x| x.id == id && x.visible)
}

/// A game with the research panel open and 1.3 done, science to spare.
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
    assert!(!research_shown(&r, &g, FLAG), "not on sale before 1.3");
    assert!(r.apply(&mut g, &Action::Research { research: FLAG.into() }).is_err());
    g.career.completed_quests.insert("qfull_load".into());
    assert!(research_shown(&r, &g, FLAG), "on sale once 1.3 is done");
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
    // A save from before the field: past 1.3 gets the flag, a new career doesn't.
    let strip = |g: &GameState| {
        let mut v: serde_json::Value = serde_json::from_str(&g.to_json()).unwrap();
        v["career"].as_object_mut().unwrap().remove("flag_research").expect("serialized");
        v.to_string()
    };
    let old = past_full_load(&r);
    let loaded = GameState::from_json(&strip(&old)).unwrap();
    assert!(career::flag_set(&loaded, FLAG), "a career past 1.3 gets Programming");
    let fresh = GameState::from_json(&strip(&r.new_game(7))).unwrap();
    assert!(!career::flag_set(&fresh, FLAG));
    // A current state round-trips as it is: past 1.3, not bought, stays not bought.
    let now = GameState::from_json(&old.to_json()).unwrap();
    assert!(!career::flag_set(&now, FLAG), "loading never buys it for a current state");
    assert_eq!(now, old);
}

/// 091: programs work from day 1 without Programming (it gates only the Programs
/// view): a new game's program earns credits, and its loadout saves and loads.
#[test]
fn a_new_game_earns_through_its_program_without_programming() {
    let r = rules();
    let mut g = r.new_game(7);
    assert!(!career::flag_set(&g, FLAG));
    let entries = vec![Entry::command("cloud_compute", 1)];
    r.apply(&mut g, &Action::SetProgram { program: 0, entries }).unwrap();
    r.apply(&mut g, &Action::SetProcessors { program: 0, processors: 1 }).unwrap();
    let credits = g.amount("credits");
    r.advance(&mut g, 50);
    assert!(g.amount("credits") > credits, "the program sold compute");
    r.apply(&mut g, &Action::SaveLoadout { program: 0, slot: None, name: "sell".into() }).unwrap();
    r.apply(&mut g, &Action::LoadLoadout { loadout: 0, program: 1 }).unwrap();
    assert_eq!(g.run.programs[1].entries, g.run.programs[0].entries);
    assert!(!career::flag_set(&g, FLAG), "nothing here bought Programming");
}
