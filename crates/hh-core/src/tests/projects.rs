//! Projects, research and ideology.

use super::{f, rules, set};
use crate::modules::ideology;
use crate::{Action, Fx, GameState};

#[test]
fn ideology_ranks_are_geometric() {
    let r = rules();
    assert_eq!(ideology::score_for_rank(&r, 1), f("55"));
    // Two significant figures (Matt, 131): 55 / 140 / 260 / 450 / 730.
    let t: Vec<Fx> = (1..=5).map(|n| ideology::score_for_rank(&r, n)).collect();
    assert_eq!(t, ["55", "140", "260", "450", "730"].map(f));
    assert_eq!(f("-137.5").round_sig(2), f("-140"));
    assert_eq!(f("0.0123").round_sig(2), f("0.012"));
    assert_eq!(Fx::MAX.round_whole(), Fx::MAX, "clamped at the range's edge");
    assert_eq!(ideology::rank_for_score(&r, f("730")), 5);
    assert_eq!(ideology::rank_for_score(&r, f("729.999999")), 4);
    assert_eq!(ideology::rank_for_score(&r, f("-250")), -2);
    let mut g = r.new_game(1);
    ideology::fund(&r, &mut g, "humanist");
    assert_eq!(ideology::score(&g, "humanist"), Fx::ONE);
    assert_eq!(ideology::score(&g, "nationalist"), f("-0.5"));
}

#[test]
fn research_needs_visibility_and_science() {
    let r = rules();
    let mut g = r.new_game(1);
    let res = |id: &str| Action::Research { research: id.into() };
    // No research before 1.1 Online opens the research panel.
    assert!(r.apply(&mut g, &res("basic_blueprints")).unwrap_err().contains("not open"));
    g.run.enabled_panels.insert(crate::modules::research::RESEARCH_PANEL.into());
    assert!(r.apply(&mut g, &res("basic_blueprints")).unwrap_err().contains("science"));
    // Before speculators arrive this run Market Awareness doesn't show.
    assert!(r.apply(&mut g, &res("market_awareness")).unwrap_err().contains("not available"));
    g.run.events.insert("speculators_arrive".into(), 0);
    assert!(r.apply(&mut g, &res("market_awareness")).unwrap_err().contains("science"));
    assert!(r.apply(&mut g, &res("dream_protocols")).unwrap_err().contains("not available"));
    set(&mut g, "science", "250");
    r.apply(&mut g, &res("market_awareness")).unwrap();
    assert_eq!(g.amount("science"), f("50"));
    assert!(r.view(&g).markets[0].demand.is_some(), "market awareness shows demand");
}

#[test]
fn the_dome_needs_its_blueprints_which_the_archive_discounts_once_bought() {
    use crate::modules::{economy, projects, research};
    let r = rules();
    let mut g = r.new_game(1);
    let c = crate::content::content();
    let dome = || c.building("recreation_dome").unwrap();
    let item = c.research_item("recreation_dome_blueprints").unwrap();
    let res = Action::Research { research: item.id.clone() };
    g.run.enabled_panels.insert(research::RESEARCH_PANEL.into());
    assert!(r.apply(&mut g, &res).unwrap_err().contains("not available"), "hidden before the first retirement");
    // Run 2: the blueprint shows at 300 science; the Dome waits for it.
    g.run.enabled_panels.insert("retirement".into());
    r.apply(&mut g, &Action::Retire).unwrap();
    r.advance(&mut g, 1);
    assert_eq!(g.career.run_number, 2);
    g.run.enabled_panels.insert(research::RESEARCH_PANEL.into());
    assert!(!economy::unlocked(&g, dome()), "no Dome before its blueprint");
    let shown = |g: &GameState| r.view(g).research.into_iter().find(|x| x.id == item.id).unwrap();
    // It shows once the run's Research Lab is built.
    assert!(!shown(&g).visible);
    super::support::own(&mut g, "research_lab", 1);
    r.advance(&mut g, 1);
    assert!(shown(&g).visible);
    assert_eq!(shown(&g).cost, f("300"));
    set(&mut g, "science", "300");
    r.apply(&mut g, &res).unwrap();
    assert!(economy::unlocked(&g, dome()));
    // Run 3 with the Archive done: its opening Blueprints, and this one at 240.
    r.apply(&mut g, &Action::Retire).unwrap();
    r.advance(&mut g, 1);
    assert!(!economy::unlocked(&g, dome()), "a new run needs it again");
    assert_eq!(shown(&g).cost, f("300"), "full price without the Archive");
    let p = c.project("blueprint_archive").unwrap();
    for (rs, cost) in &p.costs {
        g.career.projects.progress.entry(p.id.clone()).or_default().insert(rs.clone(), *cost - f("1"));
        set(&mut g, rs, "10");
        r.apply(&mut g, &Action::SetProjectRate { project: p.id.clone(), resource: rs.clone(), rate: f("1") })
            .unwrap();
    }
    r.advance(&mut g, 1);
    assert!(projects::completed(&g, "blueprint_archive"));
    assert!(g.run.research.contains("basic_blueprints") && g.run.research.contains("advanced_blueprints"));
    assert_eq!(research::price(&r, &g, item), f("240"));
    assert_eq!(shown(&g).cost, f("240"));
    // Only Blueprints: other research bought before keeps its price.
    let other = c.research_item("regolith_processing").unwrap();
    g.career.researched.insert(other.id.clone());
    assert_eq!(research::price(&r, &g, other), other.cost);
    // A later run still opens with both Blueprints.
    r.apply(&mut g, &Action::Retire).unwrap();
    assert!(g.run.research.contains("basic_blueprints") && g.run.research.contains("advanced_blueprints"));
}

#[test]
fn a_repeatable_projects_multiplier_stacks_by_adding() {
    use crate::modules::projects::stacked;
    assert_eq!(stacked(f("1.05"), 1), f("1.05"));
    assert_eq!(stacked(f("1.05"), 10), f("1.5"), "+5% ×10 is +50%");
    assert_eq!(stacked(f("0.9"), 3), f("0.7"));
    assert_eq!(stacked(f("0.5"), 3), Fx::ZERO, "floored at 0");
}

#[test]
fn projects_drain_at_their_rates_and_complete() {
    let r = rules();
    let mut g = r.new_game(1);
    g.career.flags.insert("used_disrupt_speculators".into());
    let id = "speculator_dossier";
    let rate = |res: &str, v: &str| Action::SetProjectRate { project: id.into(), resource: res.into(), rate: f(v) };
    // The funding cap: science at most 10 a day, credits 30.
    assert!(r.apply(&mut g, &rate("science", "11")).is_err(), "over the most a day");
    r.apply(&mut g, &rate("science", "10")).unwrap();
    r.apply(&mut g, &rate("credits", "30")).unwrap();
    assert!(r.apply(&mut g, &rate("credits", "31")).is_err(), "over the most a day");
    r.apply(&mut g, &rate("credits", "20")).unwrap();
    set(&mut g, "science", "1000");
    set(&mut g, "credits", "1000");
    assert_eq!(r.view(&g).projects.iter().find(|p| p.id == id).unwrap().days_to_done, Some(15));
    r.advance(&mut g, 5);
    assert!(crate::modules::projects::progress(&g, crate::content::content().project(id).unwrap())
        .is_some_and(|m| m.get("credits") == Some(&f("100")) && m.get("science") == Some(&f("50"))));
    // Science runs dry: the actual rate is what the stock covered.
    set(&mut g, "science", "0");
    r.advance(&mut g, 1);
    let pv = |g: &crate::GameState| r.view(g).projects.into_iter().find(|p| p.id == id).unwrap();
    assert_eq!(pv(&g).actual_rates.get("science"), Some(&f("0")), "nothing in stock: nothing taken");
    assert_eq!(pv(&g).days_to_done, None, "a cost owed that gets nothing");
    set(&mut g, "science", "1000");
    r.advance(&mut g, 9);
    assert!(!crate::modules::projects::completed(&g, id), "science: 140 of 150");
    r.advance(&mut g, 1);
    assert_eq!(g.run.projects.completions.get(id), Some(&1));
    assert!(g.run.project_rates.is_empty(), "done for good: its rates clear");
    assert_eq!(crate::modifiers::modifier(&r, &g, "speculator_burst_interval_mult"), f("1.33"));
    let v = r.view(&g);
    assert!(v.live_rewards.iter().any(|t| t.starts_with("Speculator Dossier: ")), "{:?}", v.live_rewards);
    // Its popup's line: this run's completion, what it gave and what it cost.
    let done = &v.project_done;
    assert_eq!(done.len(), 1);
    assert_eq!((done[0].id, done[0].completion, done[0].day), (id, 1, g.run.day - 1), "the day it played");
    assert_eq!((done[0].costs["science"], done[0].costs["credits"]), (f("150"), f("100")));
    assert_eq!(done[0].reward_text.len(), 1);
    // A timeline's end starts the next run 1 afresh: nothing from before it lists.
    g.note("timeline_end", "fell");
    assert!(r.view(&g).project_done.is_empty());
}

#[test]
fn a_repeatable_project_grows_its_costs_and_keeps_its_rates() {
    use crate::content::{Project, Repeat, Scope};
    let r = rules();
    let mut g = r.new_game(1);
    // A repeatable copy of a row, driven through the module directly.
    let base = crate::content::content().project("speculator_dossier").unwrap();
    let p = Project { repeat: Some(Repeat { cost_growth: f("2") }), scope: Scope::Forever, ..base.clone() };
    assert_eq!(crate::modules::projects::cost(&g, &p, "credits"), f("100"));
    g.career.forever_projects.completions.insert(p.id.clone(), 2);
    assert_eq!(crate::modules::projects::cost(&g, &p, "credits"), f("400"), "100 × 2²");
    assert_eq!(crate::modules::projects::max_rate(&r, &g, &p, "credits"), f("30"), "the cap, whatever the cost");
    // A deliberate minimum (the telescope's 3,000 days) lowers the cap further.
    let t = crate::content::content().project("deep_space_telescope").unwrap();
    assert_eq!(crate::modules::projects::max_rate(&r, &g, t, "credits"), f("10"), "30,000 / 3,000");
    assert_eq!(crate::modules::projects::max_rate(&r, &g, t, "science"), f("1.666667"), "5,000 / 3,000 rounded up");
}

#[test]
fn locked_projects_show_their_condition_and_rewards() {
    let r = rules();
    let g = r.new_game(1);
    let v = r.view(&g);
    let find = |id: &str| v.projects.iter().find(|p| p.id == id).unwrap();
    let sd = find("speculator_dossier");
    assert!(!sd.unlocked);
    assert_eq!(sd.days_to_done, None, "no rate set");
    assert_eq!(find("microwave_power").reward_text, vec!["Opens Microwave Receiver".to_string()]);
    assert_eq!(find("microwave_power").unlock_text, "Reach rank 5 on the Nationalist axis (best in the career)");
    assert_eq!(find("speculator_dossier").unlock_text, "Use Disrupt Speculators once");
}

#[test]
fn overclock_cards_say_the_days_a_run_lasts() {
    let r = rules();
    let mut g = r.new_game(1);
    let text = |g: &GameState| {
        let v = r.view(g);
        v.commands.iter().find(|c| c.id == "overclock_mining").unwrap().description.to_string()
    };
    assert!(text(&g).contains("for 5 days."), "{}", text(&g));
    // Rank no longer touches it (ruling 3, 129): a negative Rationalist rank keeps 5 days.
    let mut score = Fx::ZERO;
    while crate::modules::ideology::rank(&r, &g, "rationalist") >= 0 {
        score -= Fx::ONE;
        g.run.ideology.insert("rationalist".into(), score);
    }
    assert_eq!(crate::modules::ideology::rank(&r, &g, "rationalist"), -1);
    assert!(text(&g).contains("for 5 days."), "{}", text(&g));
    assert_eq!(crate::modules::programs::overclock_days(&r, &g, 5), 5);
}
