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

/// Jumps the countdown to the day before the swarm, then plays that day.
fn reach_arrival(r: &crate::Rules, g: &mut GameState) {
    let t = g.saga.timeline.as_mut().unwrap();
    t.day = t.arrival - 1;
    r.advance(g, 1);
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
fn the_calendar_sends_earth_science_and_retirements_cost_the_gap() {
    let r = rules();
    let mut g = revealed(&r);
    r.advance(&mut g, 1);
    let t = g.saga.timeline.clone().unwrap();
    assert_eq!((t.number, t.day), (1, 2), "the reveal day and the next");
    assert_eq!(t.arrival, 36_500, "arrival_days at strength 1");
    set(&mut g, "earth_science", "50");
    r.advance(&mut g, 1);
    assert_eq!(g.saga.science, f("50"));
    assert_eq!(g.amount("earth_science"), f("0"));
    let day = g.saga.timeline.as_ref().unwrap().day;
    g.run.enabled_panels.insert("retirement".into());
    r.apply(&mut g, &Action::Retire).unwrap();
    assert_eq!(g.saga.timeline.as_ref().unwrap().day, day + r.p.career.gap_days, "one gap, every act");
    assert_eq!(r.p.career.gap_days, 1825);
}

#[test]
fn nothing_counts_toward_the_swarm_before_the_first_discovery() {
    let r = rules();
    let mut g = r.new_game(7);
    g.career.completed_quests.insert("qpower_boot_sequence".into());
    r.advance(&mut g, 10);
    g.run.enabled_panels.insert("retirement".into());
    r.apply(&mut g, &Action::Retire).unwrap();
    r.advance(&mut g, 10);
    assert!(g.saga.is_default(), "neither days nor retirements count before 1.6");
    let v = r.view(&g);
    assert!(!v.saga.revealed && v.saga.days_left.is_none(), "no calendar shows");
    // The discovery starts the countdown that day, with no Telescope standing.
    g.career.flags.insert(timeline::REVEAL_FLAG.into());
    r.advance(&mut g, 1);
    assert!(!crate::modules::projects::completed(&g, "deep_space_telescope"));
    assert_eq!(r.view(&g).saga.days_left, Some(36_499));
}

/// Retires `n` times (each moves the countdown one gap).
fn retire(r: &crate::Rules, g: &mut GameState, n: u64) {
    for _ in 0..n {
        g.run.enabled_panels.insert("retirement".into());
        r.apply(g, &Action::Retire).unwrap();
    }
}

/// Matt's ruling (086): the discovery is career-long; every later timeline's
/// countdown runs from its first day, Telescope rebuilt or not.
#[test]
fn a_later_timelines_countdown_runs_from_its_first_day_without_the_telescope() {
    let r = rules();
    let mut g = revealed(&r);
    g.career.completed_quests.insert("qdeep_space_telescope".into());
    g.career.seen_events.insert("qdeep_space_telescope".into());
    finish(&r, &mut g, "deep_space_telescope");
    reach_arrival(&r, &mut g);
    assert_eq!(g.saga.ended, 1);
    assert!(!crate::modules::projects::completed(&g, "deep_space_telescope"), "the timeline's end took the Telescope");
    let t = g.saga.timeline.clone().unwrap();
    assert_eq!((t.number, t.day), (2, 0), "timeline 2 starts on its first day");
    let v = r.view(&g);
    assert_eq!(v.saga.days_left, Some(t.arrival), "the countdown shows at once");
    assert!(v.projects.iter().any(|p| p.id == "deep_space_telescope" && p.unlocked && !p.completed), "a rebuild is offered");
    // Days and retirements count, never the Telescope; the swarm comes on schedule.
    r.advance(&mut g, 3);
    assert_eq!(g.saga.timeline.as_ref().unwrap().day, 3);
    let gap = r.p.career.gap_days;
    let retirements = (t.arrival - 3 - 10) / gap;
    retire(&r, &mut g, retirements);
    let day = g.saga.timeline.as_ref().unwrap().day;
    assert_eq!(day, 3 + retirements * gap, "each retirement moves it one gap");
    // The rest day by day, boredom held off so no forced retirement moves it.
    let retired = g.career.retirements;
    while g.saga.timeline.as_ref().unwrap().day < t.arrival - 1 {
        set(&mut g, "boredom", "0");
        r.advance(&mut g, 1);
    }
    assert_eq!(g.career.retirements, retired);
    assert_eq!((g.saga.ended, g.saga.timeline.as_ref().unwrap().day), (1, t.arrival - 1), "not a day early");
    r.advance(&mut g, 1);
    assert_eq!(g.saga.ended, 2, "the swarm arrives on its day");
    assert_eq!(g.saga.history[1].days, t.arrival);
    assert_eq!(g.saga.timeline.as_ref().unwrap().number, 3);
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
    reach_arrival(&r, &mut g);
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
fn a_timeline_end_clears_timeline_projects_and_their_buildings_but_keeps_forever_ones() {
    let r = rules();
    let mut g = revealed(&r);
    let receiver = crate::content::content().building("microwave_receiver").unwrap();
    assert!(!crate::modules::economy::requirement_met(&g, &receiver.requires));
    g.career.projects.completions.insert("microwave_power".into(), 1);
    assert!(crate::modules::economy::requirement_met(&g, &receiver.requires), "its project opens it");
    // A forever book's progress and completions (no forever row yet: the book itself).
    g.career.forever_projects.completions.insert("some_forever".into(), 3);
    g.career.forever_projects.progress.insert("some_forever".into(), [("credits".to_string(), f("5"))].into());
    reach_arrival(&r, &mut g);
    assert_eq!(g.saga.ended, 1);
    assert!(g.career.projects.completions.is_empty(), "timeline projects reset");
    assert!(!crate::modules::economy::requirement_met(&g, &receiver.requires), "the building follows its project");
    assert_eq!(g.career.forever_projects.completions.get("some_forever"), Some(&3));
    assert_eq!(g.career.forever_projects.progress["some_forever"]["credits"], f("5"), "forever progress survives");
}

/// Completes a project honestly: its progress one short on each cost, the
/// stock and a rate of 1 to cover it, then a day.
fn finish(r: &crate::Rules, g: &mut GameState, id: &str) {
    let p = crate::content::content().project(id).unwrap();
    for (res, cost) in &p.costs {
        g.career.projects.progress.entry(id.into()).or_default().insert(res.clone(), *cost - f("1"));
        set(g, res, "10");
        r.apply(g, &Action::SetProjectRate { project: id.into(), resource: res.clone(), rate: f("1") }).unwrap();
    }
    r.advance(g, 1);
    assert!(crate::modules::projects::completed(g, id), "{id} completes");
}

#[test]
fn global_projects_effects_last_as_long_as_their_completion() {
    use crate::modules::{career, programs};
    let r = rules();
    let mut g = revealed(&r);
    g.career.flags.insert(programs::LOGIC_FLAG.into());
    for q in ["qa_voice", "qdeep_space_telescope"] {
        g.career.completed_quests.insert(q.into());
        g.career.seen_events.insert(q.into());
    }
    // The first global projects are offered from Friends in High Places' start (a day's events).
    r.advance(&mut g, 1);
    for id in ["conditional_logic", "blueprint_archive", "deep_space_telescope"] {
        finish(&r, &mut g, id);
    }
    assert!(programs::if_unlocked(&g));
    assert!(g.run.research.contains("basic_blueprints") && g.run.research.contains("advanced_blueprints"));
    assert!(!g.career.flags.contains(programs::IF_FLAG), "no career flag");
    let science = crate::modifiers::modifier(&r, &g, "science_output_mult");
    assert_eq!(science, f("1.1"));

    // The timeline ends: every effect goes with its completion.
    reach_arrival(&r, &mut g);
    assert_eq!(g.saga.ended, 1);
    let v = r.view(&g);
    assert!(!v.if_open && !programs::if_unlocked(&g), "if is locked");
    assert!(!g.run.research.contains("basic_blueprints") && !g.run.research.contains("advanced_blueprints"));
    assert!(g.career.completed_quests.contains("qa_voice"), "quests stay completed");
    assert_eq!(crate::modifiers::modifier(&r, &g, "science_output_mult"), f("1"));
    for id in ["conditional_logic", "blueprint_archive", "deep_space_telescope"] {
        let p = v.projects.iter().find(|p| p.id == id).unwrap();
        assert!(p.unlocked && !p.completed, "{id} is offered again");
    }
    assert!(career::flag_set(&g, timeline::REVEAL_FLAG), "what the Telescope revealed stays known");
    assert!(g.saga.timeline.is_some() && g.run.enabled_panels.contains("timeline"), "the calendar stays");
    assert_eq!(v.saga.days_left, Some(g.saga.timeline.as_ref().unwrap().arrival), "the countdown runs without it");

    // Re-funded, each gives its effect back; the Telescope replays no story.
    let quest_notes = |g: &GameState| g.career.log.iter().filter(|e| e.kind == "quest").count();
    let before = (quest_notes(&g), g.career.completed_quests.clone());
    finish(&r, &mut g, "conditional_logic");
    assert!(programs::if_unlocked(&g));
    finish(&r, &mut g, "blueprint_archive");
    assert!(g.run.research.contains("basic_blueprints") && g.run.research.contains("advanced_blueprints"));
    finish(&r, &mut g, "deep_space_telescope");
    assert_eq!((quest_notes(&g), g.career.completed_quests.clone()), before, "no quest completes again");
    let v = r.view(&g);
    for text in ["Science output ×1.1", "Nationalist growth ×1.1"] {
        assert!(v.live_rewards.iter().any(|x| x.contains(text)), "Stats shows {text}: {:?}", v.live_rewards);
    }
    // The Archive's Blueprints come with every later run of the timeline.
    g.run.enabled_panels.insert("retirement".into());
    r.apply(&mut g, &Action::Retire).unwrap();
    assert!(g.run.research.contains("basic_blueprints") && g.run.research.contains("advanced_blueprints"));
    assert!(programs::if_unlocked(&g));
}

#[test]
fn a_telescope_fund_grows_nationalist_ideology_a_tenth_faster() {
    let r = rules();
    let mut g = revealed(&r);
    let before = crate::modules::ideology::score(&g, "nationalist");
    crate::modules::ideology::fund(&r, &mut g, "nationalist");
    let plain = crate::modules::ideology::score(&g, "nationalist") - before;
    g.career.projects.completions.insert("deep_space_telescope".into(), 1);
    let before = crate::modules::ideology::score(&g, "nationalist");
    crate::modules::ideology::fund(&r, &mut g, "nationalist");
    assert_eq!(crate::modules::ideology::score(&g, "nationalist") - before, plain * f("1.1"));
}

#[test]
fn a_later_swarm_can_be_repelled_once_and_returns_entangled() {
    let r = rules();
    let mut g = revealed(&r);
    g.saga.ended = 1;
    g.saga.timeline.as_mut().unwrap().science_made = f("3000");
    let t = g.saga.timeline.clone().unwrap();
    reach_arrival(&r, &mut g);
    assert_eq!(g.saga.repelled, 1);
    assert!(g.saga.entangled);
    let t2 = g.saga.timeline.clone().unwrap();
    assert_eq!(t2.number, 1, "the timeline goes on");
    assert_eq!(t2.arrival, t.arrival + 600);
    assert_eq!(t2.strength, f("1.5"));
    // The entangled swarm comes sooner in every later timeline.
    reach_arrival(&r, &mut g);
    let t3 = g.saga.timeline.clone().unwrap();
    assert_eq!(t3.number, 2);
    // 3,000 made × 0.25 carried × 0.05 days each: 37 days later, then / 1.5.
    assert_eq!(t3.arrival, (36_500 + 37) * 2 / 3, "(arrival_days + carried) / entangle_mult");
}

#[test]
fn techs_need_science_and_their_quest_and_scopes_expire() {
    let r = rules();
    let mut g = revealed(&r);
    g.saga.science = f("5000");
    assert!(r.apply(&mut g, &Action::BuyEarthTech { tech: "compressed_blocks".into() }).is_err(), "needs 2.2 Earth Science");
    g.career.completed_quests.insert("qearth_science".into());
    r.apply(&mut g, &Action::BuyEarthTech { tech: "compressed_blocks".into() }).unwrap();
    assert_eq!(g.saga.science, f("4400"));
    reach_arrival(&r, &mut g);
    assert_eq!(g.saga.timeline.as_ref().unwrap().number, 2);
    // Run 1 of the new timeline got the blocks (on top of its usual start).
    assert!(g.amount("titanium") >= f("20"));
    assert_eq!(g.saga.techs.get("compressed_blocks"), Some(&0), "next_timeline: one more timeline");
    reach_arrival(&r, &mut g);
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
    assert!(r.apply(&mut g, &Action::BeginSuperTimeline).is_err(), "3.2 Filling the Vault first");
    g.career.completed_quests.insert("qfilling_vault".into());
    g.career.projects.completions.insert("foundation_grant".into(), 1);
    r.apply(&mut g, &Action::BeginSuperTimeline).unwrap();
    let t = g.saga.timeline.clone().unwrap();
    assert!(t.super_timeline);
    assert_eq!(t.arrival, 36_500);
    assert!(g.career.projects.completions.contains_key("foundation_grant"), "projects survive into it");
    r.advance(&mut g, 1);
    assert_eq!(g.saga.timeline.as_ref().unwrap().day, 1, "its countdown runs from its first day");
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
    g.career.projects.completions.insert(timeline::DYSON.into(), 1);
    reach_arrival(&r, &mut g);
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

#[test]
fn scope_icons_show_legacy_until_the_reveal_then_eternal() {
    let r = rules();
    let mut g = r.new_game(7);
    // 1.a Big Launch (a modifier reward) started; 1.c (no reward of its own) too.
    g.career.flags.insert("speculators_seen".into());
    g.career.flags.insert("a_voice".into());
    g.career.completed_quests.insert("qa_voice".into());
    g.career.completed_quests.insert("qtrade".into());
    g.career.completed_quests.insert("qfirst_shipment".into());
    let research = |v: &crate::View, id: &str| v.research.iter().find(|x| x.id == id).unwrap().keep;
    let project = |v: &crate::View, id: &str| v.projects.iter().find(|x| x.id == id).unwrap().keep;
    let side = |v: &crate::View, id: &str| v.side_quests.iter().find(|x| x.id == id).unwrap().keep;
    let v = r.view(&g);
    assert_eq!(research(&v, "programming"), Some("legacy"), "before the reveal, Legacy");
    assert_eq!(research(&v, "regolith_processing"), None, "run-only research shows nothing");
    assert_eq!(project(&v, "conditional_logic"), Some("legacy"), "a global project");
    assert!(v.projects.iter().filter(|p| p.scope == "run").all(|p| p.keep.is_none()), "Strategic projects");
    assert_eq!(side(&v, "qbig_launch"), Some("legacy"));
    g.career.flags.insert(timeline::REVEAL_FLAG.into());
    let v = r.view(&g);
    assert_eq!(research(&v, "programming"), Some("eternal"), "after the reveal, Eternal");
    assert_eq!(research(&v, "regolith_processing"), None);
    assert_eq!(project(&v, "conditional_logic"), Some("legacy"), "a timeline's end clears it");
    assert_eq!(side(&v, "qbig_launch"), Some("eternal"));
}

#[test]
fn a_rank_on_one_axis_shows_its_progress() {
    // A career-best rank (Friends Everywhere, 1.d) counts up, as `holds` reads them.
    let r = rules();
    let mut g = r.new_game(7);
    let c = crate::content::Condition::IdeologyRank { axis: "humanist".into(), rank: 5 };
    g.career.max_ideology_ranks.insert("humanist".into(), 3);
    assert_eq!(crate::modules::events::progress(&r, &g, &c), Some((f("3"), f("5"))));
    assert!(!crate::modules::events::holds(&r, &g, "", &c));
}
