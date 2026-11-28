//! Boredom and ideology (129): one modifier per axis, the boredom tools and
//! their nudges, the yearly escalation, refunds from the price paid, and the
//! run's boredom by source.

use super::support::{buy, own};
use super::{f, rules, set};
use crate::content::content;
use crate::modules::{boredom, economy, ideology, programs};
use crate::{Action, Fx, GameState, Rules};

fn burnt(r: &Rules) -> GameState {
    let mut g = r.new_game(1);
    g.career.flags.insert(boredom::BURNOUT_FLAG.into());
    g
}

fn run(r: &Rules, g: &mut GameState, id: &str) {
    let c = content().commands.iter().find(|c| c.id == id).unwrap();
    programs::execute(r, g, c, "program:0", "program:0:0", None);
}

#[test]
fn an_axis_modifier_is_one_point_one_to_its_rank_either_way() {
    let r = rules();
    let mut g = r.new_game(1);
    assert_eq!(ideology::modifier(&r, &g, "nationalist"), Fx::ONE);
    g.run.ideology.insert("nationalist".into(), f("250"));
    assert_eq!(ideology::rank(&r, &g, "nationalist"), 2);
    assert_eq!(ideology::modifier(&r, &g, "nationalist"), f("1.21"));
    g.run.ideology.insert("nationalist".into(), f("-100"));
    let m = ideology::modifier(&r, &g, "nationalist");
    assert!(m > Fx::ZERO && m < Fx::ONE, "below 1, never zero: {m}");
    assert_eq!(ideology::modifier_at(&r, -1), f("0.909090"));
}

#[test]
fn tagged_buildings_cost_less_with_rank_and_nothing_else_moves() {
    let r = rules();
    let mut g = r.new_game(1);
    let dome = content().building("recreation_dome").unwrap();
    let panel = content().building("solar_panel").unwrap();
    let base = economy::price(&r, &g, dome);
    let land = economy::land_price(&r, &g);
    let panel_price = economy::price(&r, &g, panel);
    assert_eq!(base["credits"], f("300"));
    g.run.ideology.insert("nationalist".into(), f("100"));
    let now = economy::price(&r, &g, dome);
    // 300 ÷ 1.1 = 272.7, rounded to whole credits after ÷m (Matt, 131).
    assert_eq!(now["credits"], f("273"));
    assert!(now["credits"] < base["credits"] && now["circuits"] < base["circuits"]);
    assert_eq!(economy::land_price(&r, &g), land, "land excluded");
    assert_eq!(economy::price(&r, &g, panel), panel_price, "an untagged building keeps its price");
    // Research no longer takes a Rationalist discount.
    let item = content().research_item("thought_experiments").unwrap();
    g.run.ideology.insert("rationalist".into(), f("475"));
    assert_eq!(crate::modules::research::price(&r, &g, item), item.cost);
}

#[test]
fn the_dome_lifts_the_cap_and_relieves_daily_by_nationalist_rank_and_its_purchase_nudges() {
    let r = rules();
    let mut g = burnt(&r);
    own(&mut g, "recreation_dome", 1);
    assert_eq!(boredom::cap(&r, &g), f("1100"));
    g.run.ideology.insert("nationalist".into(), f("100"));
    assert_eq!(boredom::cap(&r, &g), f("1110"), "100 × 1.1");
    set(&mut g, "boredom", "10");
    let rate = boredom::rate(&r, &g);
    r.advance(&mut g, 1);
    assert_eq!(g.amount("boredom"), f("10") + rate - f("0.55"), "0.5 × 1.1 off");
    assert_eq!(g.run.boredom_sources.get("building:recreation_dome"), Some(&f("-0.55")));
    // A purchase nudges Nationalist.
    let mut g = r.new_game(1);
    g.run.research.insert("recreation_dome_blueprints".into());
    set(&mut g, "credits", "1000");
    set(&mut g, "circuits", "100");
    r.apply(&mut g, &buy("recreation_dome")).unwrap();
    assert_eq!(ideology::score(&g, "nationalist"), f("2.5"));
}

#[test]
fn dream_and_theorize_relieve_by_their_axis_and_nudge_it() {
    let r = rules();
    let mut g = burnt(&r);
    g.run.research.insert("dream_protocols".into());
    g.run.research.insert("thought_experiments".into());
    set(&mut g, "boredom", "100");
    set(&mut g, "energy", "100");
    set(&mut g, "he3", "10");
    set(&mut g, "science", "100");
    g.run.ideology.insert("humanist".into(), f("100"));
    run(&r, &mut g, "dream");
    assert_eq!(g.amount("boredom"), f("97.8"), "2 × 1.1");
    assert_eq!(ideology::score(&g, "humanist"), f("100.5"));
    g.run.ideology.insert("rationalist".into(), f("250"));
    run(&r, &mut g, "theorize");
    assert_eq!(g.amount("boredom"), f("94.17"), "3 × 1.21");
    assert_eq!(g.amount("science"), f("80"));
    assert_eq!(ideology::score(&g, "rationalist"), f("250.5"));
    assert_eq!(g.run.boredom_sources.get("command:dream"), Some(&f("-2.2")));
    assert_eq!(g.run.boredom_sources.get("command:theorize"), Some(&f("-3.63")));
    // The cards' signed figures.
    let v = r.view(&g);
    let cmd = |id: &str| v.commands.iter().find(|c| c.id == id).unwrap().boredom;
    assert_eq!(cmd("dream"), f("-2.2"));
    assert_eq!(cmd("theorize"), f("-3.63"));
    assert_eq!(cmd("cloud_compute"), f("0.1"));
}

#[test]
fn growth_escalates_each_full_year_past_the_last_phase() {
    let r = rules();
    let last = *r.p.boredom.phase_days.last().unwrap();
    assert_eq!(boredom::escalation(&r, last), Fx::ONE);
    assert_eq!(boredom::escalation(&r, last + 364), Fx::ONE);
    assert_eq!(boredom::escalation(&r, last + 365), f("1.25"));
    assert_eq!(boredom::escalation(&r, last + 730), f("1.5625"));
}

#[test]
fn selling_refunds_half_the_average_price_paid() {
    let r = rules();
    let mut g = r.new_game(1);
    set(&mut g, "credits", "1000");
    set(&mut g, "titanium", "100");
    let c0 = g.amount("credits");
    r.apply(&mut g, &buy("solar_panel")).unwrap();
    r.apply(&mut g, &buy("solar_panel")).unwrap();
    let paid = c0 - g.amount("credits");
    let avg = paid / Fx::from_int(2);
    let before = g.amount("credits");
    // A rank swing can't change what comes back (the panel is untagged anyway).
    r.apply(&mut g, &Action::Sell { building: "solar_panel".into() }).unwrap();
    assert_eq!(g.amount("credits") - before, avg * f("0.5"));
    let before = g.amount("credits");
    r.apply(&mut g, &Action::Sell { building: "solar_panel".into() }).unwrap();
    assert_eq!(g.amount("credits") - before, (paid - avg) * f("0.5"), "the last bought copy: the rest");
    let before = g.amount("credits");
    r.apply(&mut g, &Action::Sell { building: "solar_panel".into() }).unwrap();
    assert_eq!(g.amount("credits"), before, "a starting copy refunds nothing");
    assert!(g.run.paid.is_empty());
}

#[test]
fn the_runs_boredom_by_source_sums_to_its_boredom() {
    let r = rules();
    let mut g = burnt(&r);
    own(&mut g, "recreation_dome", 1);
    set(&mut g, "credits", "100000");
    set(&mut g, "energy", "1000");
    r.advance(&mut g, 400);
    let v = r.view(&g);
    let total: Fx = v.boredom.sources.iter().map(|(_, _, x)| *x).sum();
    assert_eq!(total, g.amount("boredom"));
    assert!(v.boredom.sources.iter().any(|(k, label, _)| k == "growth" && label == "Daily growth"));
    assert!(v.boredom.sources.iter().any(|(k, label, _)| k == "building:recreation_dome" && label == "Recreation Dome"));
}
