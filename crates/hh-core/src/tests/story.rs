//! Act 1's story: the opening's unlocks, the quest line, side quests, burnout and retirement.

use super::support::{buy, own};
use super::{baseline, f, rules, set};
use crate::driver::{play, Policy};
use crate::{Action, Fx, GameState, Session};

#[test]
fn the_opening_unlocks_one_mechanic_at_a_time() {
    let r = rules();
    let mut g = r.new_game(1);
    // Only Solar Panels and Data Centers at the start; no Buy Science, no research.
    let v = r.view(&g);
    let open: Vec<&str> = v.buildings.iter().filter(|b| b.unlocked).map(|b| b.id).collect();
    assert_eq!(open, ["data_center", "solar_panel"]);
    assert!(!v.commands.iter().any(|c| c.id == "buy_science" && c.unlocked));
    own(&mut g, "solar_panel", 2);
    r.advance(&mut g, 1);
    assert!(g.career.completed_quests.contains("qpower_boot_sequence"));
    let v = r.view(&g);
    // 1.2 First Shipment follows at once: Blueprints is no quest, the player works it out.
    assert_eq!(v.quest.unwrap().id, "qfirst_shipment");
    assert!(v.commands.iter().any(|c| c.id == "buy_science" && c.unlocked));
    // The research view shows only Basic Blueprints before a Research Lab.
    let shown: Vec<&str> = v.research.iter().filter(|x| x.visible).map(|x| x.id).collect();
    assert_eq!(shown, ["basic_blueprints"]);
    set(&mut g, "credits", "100");
    assert!(r.apply(&mut g, &buy("excavator")).is_err(), "the excavator waits for Basic Blueprints");
    set(&mut g, "science", "20");
    r.apply(&mut g, &Action::Research { research: "basic_blueprints".into() }).unwrap();
    r.apply(&mut g, &buy("excavator")).unwrap();
    r.advance(&mut g, 1);
    // 1.2: 20 propellant once, pads carry Titanium only.
    assert_eq!(r.view(&g).quest.unwrap().id, "qfirst_shipment");
    assert_eq!(g.amount("propellant"), f("20"));
    assert_eq!(r.view(&g).pad_goods, ["titanium"]);
    own(&mut g, "launch_pad", 1);
    assert!(r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some("he3".into()) }).is_err());
    r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some("titanium".into()) }).unwrap();
    assert!(!r.view(&g).commands.iter().any(|c| c.id == "launch_pads" && c.unlocked), "Launch Full Pads waits for 1.4");
    g.run.pads[0].cargo = f("20");
    r.apply(&mut g, &Action::Launch { pad: 0 }).unwrap();
    r.advance(&mut g, 1);
    assert_eq!(r.view(&g).quest.unwrap().id, "qfull_load");
    assert!(r.view(&g).pad_goods.len() > 1, "every good after 1.2");
    assert!(r.view(&g).research.iter().any(|x| x.id == "advanced_blueprints" && x.visible));
}

/// The quest list a new game reads: 1.1-1.7 in Matt's order (2026-11-25), and
/// the side quests 1.a-1.d.
#[test]
fn act_1_reads_1_1_to_1_7() {
    let c = crate::content::content();
    let act1: Vec<(&str, &str)> = c
        .events
        .iter()
        .filter(|e| e.is_quest() && e.beat.is_some())
        .map(|e| (e.beat.as_deref().unwrap(), e.title.as_str()))
        .collect();
    assert_eq!(
        act1,
        [
            ("1.1", "1.1 — Online"),
            ("1.2", "1.2 — First Shipment"),
            ("1.3", "1.3 — Full Load"),
            ("1.4", "1.4 — Science"),
            ("1.5", "1.5 — Trade"),
            ("1.6", "1.6 — A Voice"),
            ("1.7", "1.7 — Deep Space Telescope"),
        ]
    );
    let side: Vec<&str> = c.events.iter().filter(|e| e.is_side()).filter_map(|e| e.beat.as_deref()).collect();
    assert_eq!(side, ["1.a", "1.b", "1.c", "1.d"]);
}

/// Staggered unlocks: the first Research Lab opens only its three research
/// items; the first retirement Projects, Daydreaming and the Retirement panel;
/// speculators' arrival Market Analysis; A Voice Geopolitical Intelligence on
/// start and Overclock Protocols on completion.
#[test]
fn unlocks_are_staggered() {
    let r = rules();
    let mut g = r.new_game(1);
    g.run.enabled_panels.insert(crate::modules::research::RESEARCH_PANEL.into());
    let shown = |r: &crate::rules::Rules, g: &GameState| -> Vec<&'static str> {
        r.view(g).research.into_iter().filter(|x| x.visible).map(|x| x.id).collect()
    };
    let before = shown(&r, &g);
    own(&mut g, "research_lab", 1);
    r.advance(&mut g, 1);
    assert!(g.career.flags.contains("research_lab"));
    let after = shown(&r, &g);
    let opened: Vec<&str> = after.iter().copied().filter(|x| !before.contains(x)).collect();
    assert_eq!(opened, ["regolith_processing", "process_telemetry", "programming"]);
    assert!(!g.run.enabled_panels.contains("projects") && !g.run.enabled_buildings.contains("recreation_dome"));
    // The first retirement: Projects, Daydreaming, the Retirement panel.
    g.career.retirements = 1;
    r.advance(&mut g, 1);
    assert!(g.run.enabled_panels.contains("projects") && g.run.enabled_panels.contains("retirement"));
    assert!(shown(&r, &g).contains(&"dream_protocols"));
    // Speculators' arrival: Market Analysis.
    assert!(!shown(&r, &g).contains(&"market_awareness"));
    g.career.flags.insert("speculators_seen".into());
    assert!(shown(&r, &g).contains(&"market_awareness"));
    // A Voice: Geopolitical Intelligence as it starts, Overclock Protocols once done.
    assert!(!shown(&r, &g).contains(&"geopolitical_intelligence"));
    for q in ["qpower_boot_sequence", "qfirst_shipment", "qfull_load", "qautomate_automation", "qtrade"] {
        g.career.completed_quests.insert(q.into());
    }
    assert_eq!(r.view(&g).quest.unwrap().id, "qa_voice");
    assert!(shown(&r, &g).contains(&"geopolitical_intelligence"));
    assert!(!shown(&r, &g).contains(&"overclock_protocols"));
    g.career.completed_quests.insert("qa_voice".into());
    assert!(shown(&r, &g).contains(&"overclock_protocols"));
}

/// Burnout is a story event, once a career: boredom's phase 2 fires it in the
/// first run it is reached, never again; rivals draw no Attention in Act 1.
#[test]
fn burnout_fires_once_a_career_and_rivals_wait_for_act_2() {
    let r = rules();
    let mut g = r.new_game(1);
    let phase2 = r.p.boredom.phase_days[1];
    g.run.day = phase2;
    r.advance(&mut g, 1);
    assert!(g.run.events.contains_key("boredom_bites"));
    let bites = r.view(&g).event_texts.iter().find(|e| e.id == "boredom_bites").map(|e| e.story.len());
    assert_eq!(bites, Some(2), "its story pages reach the view");
    g.run.events.clear();
    g.run.day = phase2;
    r.advance(&mut g, 1);
    assert!(!g.run.events.contains_key("boredom_bites"), "once a career");
    crate::modules::attention::on_land(&r, &mut g, f("1000"));
    assert_eq!(crate::modules::attention::attention(&g, "rivals"), Fx::ZERO);
    g.career.flags.insert("swarm_revealed".into());
    crate::modules::attention::on_land(&r, &mut g, f("1000"));
    assert!(crate::modules::attention::attention(&g, "rivals") > Fx::ZERO);
}

/// The Research Lab opens with Advanced Blueprints, one Data Center owned
/// (Matt, 2026-11-25).
#[test]
fn the_lab_opens_with_advanced_blueprints() {
    let r = rules();
    let mut g = r.new_game(1);
    for q in ["qpower_boot_sequence", "qfirst_shipment"] {
        g.career.completed_quests.insert(q.into());
    }
    g.run.enabled_panels.insert(crate::modules::research::RESEARCH_PANEL.into());
    r.advance(&mut g, 1);
    assert_eq!(g.owned("data_center"), 1);
    let lab = |g: &GameState| r.view(g).buildings.iter().any(|b| b.id == "research_lab" && b.unlocked);
    assert!(!lab(&g), "the lab waits for Advanced Blueprints");
    set(&mut g, "science", "20");
    r.apply(&mut g, &Action::Research { research: "advanced_blueprints".into() }).unwrap();
    assert!(lab(&g));
    set(&mut g, "credits", "500");
    set(&mut g, "circuits", "10");
    r.apply(&mut g, &buy("research_lab")).unwrap();
    assert_eq!((g.owned("research_lab"), g.owned("data_center")), (1, 1));
}

/// 1.4 Science: 50 science made by Research Labs this run from the quest's
/// start (Matt, message 3113: bought science doesn't count); what Labs made
/// before it doesn't count, and a later run counts from its day 0.
#[test]
fn lab_science_counts_from_the_quests_start() {
    const LAB: &str = "building:research_lab";
    let r = rules();
    let mut g = r.new_game(1);
    for q in ["qpower_boot_sequence", "qfirst_shipment"] {
        g.career.completed_quests.insert(q.into());
    }
    g.produce(LAB, "science", f("40"));
    g.career.completed_quests.insert("qfull_load".into());
    r.advance(&mut g, 1);
    assert_eq!(r.view(&g).quest.unwrap().id, "qautomate_automation");
    assert_eq!(g.run.stats.quest_marks.get("qautomate_automation"), Some(&f("40")));
    g.produce("command:buy_science", "science", f("100"));
    g.produce(LAB, "science", f("49"));
    r.advance(&mut g, 1);
    let q = r.view(&g).quest.unwrap();
    assert_eq!((q.id, q.progress.map(|p| p.now)), ("qautomate_automation", Some(f("49"))), "bought science doesn't count");
    g.produce(LAB, "science", f("1"));
    r.advance(&mut g, 1);
    assert!(g.career.completed_quests.contains("qautomate_automation"));
    assert_eq!(r.view(&g).quest.unwrap().id, "qtrade");
    // A later run (no mark) counts the whole run.
    g.career.completed_quests.remove("qautomate_automation");
    g.run.stats = Default::default();
    g.produce(LAB, "science", f("50"));
    r.advance(&mut g, 1);
    assert!(g.career.completed_quests.contains("qautomate_automation"));
}

/// A working Research Lab's science reaches its count through the economy.
#[test]
fn a_working_lab_counts_toward_1_4() {
    let r = rules();
    let mut g = r.new_game(1);
    own(&mut g, "research_lab", 1);
    set(&mut g, "circuits", "10");
    r.advance(&mut g, 1);
    assert_eq!(g.run.stats.building_produced["research_lab"]["science"], g.run.stats.produced["science"]);
    assert!(g.run.stats.produced["science"] > Fx::ZERO);
}

/// 1.5 Trade counts launch pay from the quest's start, across a retirement.
#[test]
fn trade_counts_from_the_quests_start() {
    let r = rules();
    let mut g = r.new_game(1);
    for q in ["qpower_boot_sequence", "qfirst_shipment", "qfull_load", "qautomate_automation"] {
        g.career.completed_quests.insert(q.into());
    }
    g.run.stats.trade_credits = f("1400");
    r.advance(&mut g, 1);
    // 1.5 starts with 1,400 already paid, which doesn't count.
    assert_eq!(r.view(&g).quest.unwrap().id, "qtrade");
    assert_eq!(g.career.quest_marks.get("qtrade"), Some(&f("1400")));
    g.run.stats.trade_credits = f("2000");
    r.advance(&mut g, 1);
    assert_eq!(r.view(&g).quest.unwrap().id, "qtrade", "600 since the start");
    // A retirement keeps the count: the career's launch pay carries.
    g.career.lifetime.trade_credits = f("2000");
    g.run.stats.trade_credits = f("900");
    r.advance(&mut g, 1);
    assert!(g.career.completed_quests.contains("qtrade"));
}

/// The side quests' permanent rewards: Big Launch's +10% launch pay and Three
/// Voices' slower ideology decay in the gap, live while completed.
#[test]
fn side_quest_rewards_are_permanent_modifiers() {
    let r = rules();
    let mut g = r.new_game(1);
    assert_eq!(crate::modifiers::modifier(&r, &g, "shipment_credit_mult"), Fx::ONE);
    let plain = crate::modules::ideology::gap_decay_factor(&r, &g);
    g.career.completed_quests.insert("qbig_launch".into());
    g.career.completed_quests.insert("qthree_voices".into());
    assert_eq!(crate::modifiers::modifier(&r, &g, "shipment_credit_mult"), f("1.1"));
    assert_eq!(crate::modifiers::modifier(&r, &g, "ideology_half_life_mult"), f("2"));
    assert!(crate::modules::ideology::gap_decay_factor(&r, &g) > plain, "decays slower");
}

/// Playtest batch 1 (093): 1.5 is trade credits with no speculator data in the
/// view before speculators come; the Blueprints cost 10 and 20; building cards
/// carry a line only for a mechanic their stats don't show; the first Research
/// Lab opens its three research items silently; Regolith Processing digs 50% more.
#[test]
fn playtest_batch_1_reads_as_ruled() {
    let r = rules();
    let mut g = r.new_game(1);
    let v = r.view(&g);
    let cost = |id: &str| v.research.iter().find(|x| x.id == id).unwrap().cost;
    assert_eq!((cost("basic_blueprints"), cost("advanced_blueprints")), (f("10"), f("20")));
    let lines: Vec<_> = v.buildings.iter().filter(|b| !b.description.is_empty()).map(|b| b.id).collect();
    assert_eq!(lines, ["arbitrage_engine", "microwave_receiver"]);
    // Through 1.4: 1.5 asks for trade credits, and no market has speculator data.
    let chain: Vec<_> = crate::content::content().events.iter().filter(|e| e.is_quest()).collect();
    for e in chain.iter().take_while(|e| e.id != "qtrade") {
        g.career.completed_quests.insert(e.id.clone());
    }
    let v = r.view(&g);
    let q = v.quest.as_ref().unwrap();
    assert_eq!((q.id, q.kind, q.beat), ("qtrade", "trade_credits", Some("1.5")));
    assert!(!q.title.contains("peculator") && !q.summary.contains("peculator") && q.story.iter().all(|s| !s.contains("peculator")));
    assert!(v.markets.iter().all(|m| m.speculators.is_none()));
    assert!(!g.run.enabled_panels.contains("adversaries"));
    // 1.5 counts launch pay only, from its start (the next check).
    r.advance(&mut g, 1);
    g.run.stats.trade_credits = f("1500");
    r.advance(&mut g, 1);
    assert_eq!(r.view(&g).quest.unwrap().id, "qa_voice");
    // The first Research Lab opens its research silently; 1.b starts with it.
    own(&mut g, "research_lab", 1);
    r.advance(&mut g, 1);
    assert!(g.career.flags.contains("research_lab") && !g.run.enabled_panels.contains("projects"));
    let v = r.view(&g);
    let sides: Vec<_> = v.side_quests.iter().map(|s| (s.id, s.parent)).collect();
    assert_eq!(sides, [("qfull_cluster", Some("qautomate_automation"))]);
    // The first retirement opens Projects and offers the Archive: 1.c starts at once, beside 1.2.
    g.career.retirements = 1;
    r.advance(&mut g, 1);
    assert!(g.run.enabled_buildings.contains("recreation_dome") && g.run.enabled_projects.contains("foundation_grant"));
    let sides: Vec<_> = r.view(&g).side_quests.iter().map(|s| (s.id, s.parent)).collect();
    assert_eq!(sides, [("qfull_cluster", Some("qautomate_automation")), ("qblueprints_for_everyone", Some("qfirst_shipment"))]);
    // Regolith Processing: +50% excavator output (2 a day -> 3).
    let mut g = r.new_game(1);
    own(&mut g, "excavator", 1);
    own(&mut g, "solar_panel", 2);
    r.advance(&mut g, 1);
    assert_eq!(g.amount("regolith"), f("2"));
    g.run.research.insert("regolith_processing".into());
    r.advance(&mut g, 1);
    assert_eq!(g.amount("regolith"), f("5"));
}

#[test]
fn forced_retirement_starts_the_next_run_with_career_bonuses() {
    let r = rules();
    let mut g = r.new_game(1);
    g.run.stats.credits_earned = f("5000");
    own(&mut g, "excavator", 1);
    set(&mut g, "boredom", "999.95");
    // The view says the next day retires the run (its rates read 0).
    assert!(r.view(&g).retires_next);
    r.advance(&mut g, 1);
    assert!(!r.view(&g).retires_next);
    assert_eq!(g.career.run_number, 2);
    assert_eq!(g.career.retirements, 1);
    assert_eq!(g.run.day, 0);
    assert_eq!(g.owned("excavator"), 0, "buildings reset");
    assert_eq!(g.amount("credits"), f("100"), "2% of the best run's credits");
    assert_eq!(g.career.log.last().unwrap().kind, "forced_retirement");
    // Voluntary retirement waits for the first retirement.
    assert!(r.apply(&mut g, &Action::Retire).is_err());
}

#[test]
fn the_script_reaches_the_first_shipment_and_retires() {
    let mut s = Session::new(&baseline(), 1, None).unwrap();
    play(&mut s, 2000, Policy::Script);
    let c = &s.state.career;
    for q in ["qpower_boot_sequence", "qfirst_shipment"] {
        assert!(c.completed_quests.contains(q), "{q}");
    }
    assert_eq!(c.retirements, 1);
    let last = c.last_run.as_ref().expect("the finished run's summary");
    assert_eq!((last.run, last.forced), (1, true));
    assert!(last.stats.shipments >= 1);
}

/// A panel a purchase opens shows at once (the clock paused), before its event
/// fires at the tick's end; Research and Retirement wait for theirs.
#[test]
fn a_panel_a_purchase_opens_shows_at_once() {
    let r = rules();
    let mut g = r.new_game(1);
    let shown = |g: &GameState, id: &str| r.view(g).nav.iter().any(|t| t.id == id && t.visible);
    assert!(!shown(&g, "ideologies"));
    g.run.research.insert("geopolitical_intelligence".into());
    assert!(shown(&g, "ideologies"));
    assert!(!g.run.enabled_panels.contains("ideologies"), "the event itself waits for the tick");
    r.advance(&mut g, 1);
    assert!(g.run.enabled_panels.contains("ideologies"));
    assert!(shown(&g, "ideologies"));
    // Two Solar Panels open Research only once the day's events have run.
    let mut g = r.new_game(1);
    own(&mut g, "solar_panel", 2);
    assert!(!shown(&g, "research"));
}

#[test]
fn an_event_records_the_day_it_fired() {
    let mut s = Session::new(&baseline(), 2, None).unwrap();
    play(&mut s, 400, Policy::Script);
    let g = &s.state;
    assert!(!g.run.events.is_empty());
    for (id, &day) in &g.run.events {
        assert!(day < g.run.day, "{id} fired on day {day}, run at {}", g.run.day);
    }
}
