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
    assert!(!r.view(&g).commands.iter().any(|c| c.id == "launch_pads" && c.unlocked), "Launch Full Pads waits for 1.3");
    g.run.pads[0].cargo = f("20");
    r.apply(&mut g, &Action::Launch { pad: 0 }).unwrap();
    r.advance(&mut g, 1);
    assert_eq!(r.view(&g).quest.unwrap().id, "qfull_load");
    assert!(r.view(&g).pad_goods.len() > 1, "every good after 1.2");
    assert!(r.view(&g).research.iter().any(|x| x.id == "advanced_blueprints" && x.visible));
}

/// The quest list a new game reads: 1.1-1.6 in Matt's order (1.4 Science deleted,
/// 2026-11-26), numbered from the chain, and the side quests 1.a-1.d.
#[test]
fn act_1_reads_1_1_to_1_6() {
    let c = crate::content::content();
    let act1: Vec<(&str, &str)> = c
        .events
        .iter()
        .filter(|e| e.is_quest() && c.act_of(&e.id) == Some(1))
        .map(|e| (e.beat.as_deref().unwrap(), e.title.as_str()))
        .collect();
    assert_eq!(
        act1,
        [
            ("1.1", "1.1 — Online"),
            ("1.2", "1.2 — First Shipment"),
            ("1.3", "1.3 — Full Load"),
            ("1.4", "1.4 — Trade"),
            ("1.5", "1.5 — A Voice"),
            ("1.6", "1.6 — Deep Space Telescope"),
        ]
    );
    // Each act numbers from 1: Act 2 opens with 2.1.
    let q = c.event("qexperiments_service").unwrap();
    assert_eq!((q.beat.as_deref(), q.title.as_str()), (Some("2.1"), "2.1 — Experiments as a Service"));
    // A reward line lists only real grants: none for a quest that grants nothing.
    let reward = |id: &str| c.event(id).unwrap().reward.as_str();
    assert_eq!(reward("qfull_load"), "The Launch Full Pads command.");
    for id in ["qfirst_shipment", "qtrade", "qa_voice"] {
        assert_eq!(reward(id), "", "{id} grants nothing");
    }
    let side: Vec<&str> = c.events.iter().filter(|e| e.is_side()).filter_map(|e| e.beat.as_deref()).collect();
    assert_eq!(side, ["1.a", "1.b", "1.c", "1.d"]);
}

/// Staggered unlocks: the first Research Lab opens only Regolith Processing and
/// Liquid Cooling; the first retirement Projects, Daydreaming and the Retirement
/// panel; speculators' arrival Market Analysis; A Voice Geopolitical Intelligence
/// and Programming on start (shown from then on) and Overclock Protocols on completion.
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
    assert_eq!(opened, ["regolith_processing", "liquid_cooling"]);
    assert!(!g.run.enabled_panels.contains("projects"));
    // The first retirement: Projects, Daydreaming, the Retirement panel.
    g.career.retirements = 1;
    r.advance(&mut g, 1);
    assert!(g.run.enabled_panels.contains("projects") && g.run.enabled_panels.contains("retirement"));
    assert!(shown(&r, &g).contains(&"dream_protocols"));
    // Speculators' arrival: Market Analysis.
    assert!(!shown(&r, &g).contains(&"market_awareness"));
    g.career.flags.insert("speculators_seen".into());
    assert!(shown(&r, &g).contains(&"market_awareness"));
    // A Voice: Geopolitical Intelligence and Programming as it starts, Overclock Protocols once done.
    for q in ["qpower_boot_sequence", "qfirst_shipment", "qfull_load"] {
        g.career.completed_quests.insert(q.into());
    }
    assert!(!shown(&r, &g).contains(&"geopolitical_intelligence") && !shown(&r, &g).contains(&"programming"));
    g.career.completed_quests.insert("qtrade".into());
    assert_eq!(r.view(&g).quest.unwrap().id, "qa_voice");
    assert!(shown(&r, &g).contains(&"geopolitical_intelligence") && shown(&r, &g).contains(&"programming"));
    assert!(!shown(&r, &g).contains(&"overclock_protocols"));
    g.career.completed_quests.insert("qa_voice".into());
    assert!(shown(&r, &g).contains(&"overclock_protocols") && shown(&r, &g).contains(&"programming"));
    // Process Telemetry and Accelerated Scheduling wait for Act 2 to place them.
    assert!(!shown(&r, &g).contains(&"process_telemetry") && !shown(&r, &g).contains(&"accelerated_scheduling"));
}

/// Burnout is a story event, once a career: boredom's phase 2 fires it in the
/// first run it is reached, never again, and Boredom Rising (phase 2 too) skips
/// that run, firing in later ones; rivals draw no Attention in Act 1.
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
    r.advance(&mut g, 5);
    assert!(!g.run.events.contains_key("boredom_phase_2"), "Burnout's run skips Boredom Rising");
    g.run.events.clear();
    g.run.day = phase2;
    r.advance(&mut g, 1);
    assert!(!g.run.events.contains_key("boredom_bites"), "once a career");
    assert!(g.run.events.contains_key("boredom_phase_2"), "a later run's phase-2 notice");
    crate::modules::attention::on_land(&r, &mut g, f("1000"));
    assert_eq!(crate::modules::attention::attention(&g, "rivals"), Fx::ZERO);
    g.career.flags.insert("swarm_revealed".into());
    crate::modules::attention::on_land(&r, &mut g, f("1000"));
    assert!(crate::modules::attention::attention(&g, "rivals") > Fx::ZERO);
}

/// Boredom stays at 0 until Burnout fires, the day's growth and command boredom
/// alike; Burnout starts it, and it grows from day 0 of every later run (Matt,
/// 2026-11-26).
#[test]
fn boredom_waits_for_burnout() {
    let r = rules();
    let mut g = r.new_game(1);
    let cloud = crate::content::content().commands.iter().find(|c| c.id == "cloud_compute").unwrap();
    let command_boredom = |g: &GameState| crate::modules::programs::boredom_cost(g, cloud);
    assert!(cloud.boredom > Fx::ZERO);
    let phase2 = r.p.boredom.phase_days[1];
    r.advance(&mut g, phase2);
    assert_eq!(g.amount("boredom"), Fx::ZERO, "no boredom before Burnout");
    assert_eq!((r.view(&g).boredom.rate, command_boredom(&g)), (Fx::ZERO, Fx::ZERO));
    r.advance(&mut g, 1);
    assert!(g.career.seen_events.contains("boredom_bites"));
    r.advance(&mut g, 1);
    assert!(g.amount("boredom") > Fx::ZERO, "Burnout starts it");
    assert!(r.view(&g).boredom.rate > Fx::ZERO);
    assert_eq!(command_boredom(&g), cloud.boredom);
    // A later run grows from its day 0.
    set(&mut g, "boredom", "1000");
    r.advance(&mut g, 1);
    assert_eq!((g.career.run_number, g.run.day), (2, 0));
    r.advance(&mut g, 1);
    assert!(g.amount("boredom") > Fx::ZERO);
}

/// Accelerated Scheduling is out of Act 1 (never shown until Act 2 places it);
/// its `fast_speeds` flag still opens the fast speeds for the career.
#[test]
fn accelerated_scheduling_waits_for_act_2() {
    let r = rules();
    let mut g = r.new_game(1);
    let listed = |g: &GameState| r.view(g).research.iter().any(|x| x.id == "accelerated_scheduling" && x.visible);
    assert!(!listed(&g) && !r.view(&g).fast_speeds);
    for q in ["qpower_boot_sequence", "qfirst_shipment", "qfull_load"] {
        g.career.completed_quests.insert(q.into());
    }
    g.run.enabled_panels.insert(crate::modules::research::RESEARCH_PANEL.into());
    set(&mut g, "science", "30");
    assert!(!listed(&g));
    assert!(r.apply(&mut g, &Action::Research { research: "accelerated_scheduling".into() }).is_err());
    g.career.flags.insert(crate::view::FAST_SPEEDS_FLAG.into());
    assert!(r.view(&g).fast_speeds);
}

/// Liquid Cooling opens Run Simulations: 3 energy and 1 ice into 6 credits, no
/// boredom (Matt, 2026-11-26).
#[test]
fn run_simulations_turns_ice_and_energy_into_credits() {
    let r = rules();
    let mut g = r.new_game(1);
    let open = |g: &GameState| r.view(g).commands.iter().any(|c| c.id == "run_simulations" && c.unlocked);
    assert!(!open(&g));
    g.run.research.insert("liquid_cooling".into());
    assert!(open(&g));
    let cmd = crate::content::content().command("run_simulations").unwrap();
    assert_eq!(cmd.costs.get("energy"), Some(&f("3")));
    assert_eq!(cmd.costs.get("ice"), Some(&f("1")));
    assert_eq!(cmd.produces.get("credits"), Some(&f("6")));
    assert_eq!(cmd.boredom, Fx::ZERO);
    assert_eq!(crate::content::content().command("buy_ice").unwrap().costs.get("credits"), Some(&f("5")));
    // Run once a day by a program: 1 ice and 3 energy become 6 credits, and boredom grows
    // only by the day's own rate (past Burnout), as with an Idle line.
    let day = |cmd: &str| {
        let mut g = r.new_game(1);
        g.career.flags.insert(crate::modules::boredom::BURNOUT_FLAG.into());
        g.run.research.insert("liquid_cooling".into());
        set(&mut g, "ice", "5");
        set(&mut g, "energy", "20");
        super::support::program(&r, &mut g, &[(cmd, 1)]);
        let before = (g.amount("credits"), g.amount("ice"), g.amount("energy"));
        r.advance(&mut g, 1);
        (g.amount("credits") - before.0, g.amount("ice") - before.1, g.amount("energy") - before.2, g.amount("boredom"))
    };
    let (sim, idle) = (day("run_simulations"), day("idle"));
    assert_eq!(sim.0 - idle.0, f("6") - f("1"), "6 credits (Idle's 1 for comparison)");
    assert_eq!((sim.1, idle.1), (f("-1"), Fx::ZERO));
    assert_eq!(sim.2 - idle.2, f("-3"));
    assert_eq!(sim.3, idle.3, "no boredom of its own");
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

/// 1.3 Full Load's completion opens Launch Full Pads (1.4 Science deleted).
#[test]
fn full_load_opens_launch_full_pads() {
    let r = rules();
    let mut g = r.new_game(1);
    let open = |g: &GameState| r.view(g).commands.iter().any(|c| c.id == "launch_pads" && c.unlocked);
    for q in ["qpower_boot_sequence", "qfirst_shipment"] {
        g.career.completed_quests.insert(q.into());
    }
    assert!(!open(&g));
    g.career.completed_quests.insert("qfull_load".into());
    assert!(open(&g));
    r.advance(&mut g, 1);
    assert_eq!(r.view(&g).quest.unwrap().id, "qtrade", "Trade follows 1.3");
}

/// 1.4 Trade counts all of this run's launch pay, from before the quest's start
/// too (Matt, 2026-11-26); a retirement starts the count over.
#[test]
fn trade_counts_the_whole_run() {
    let r = rules();
    let mut g = r.new_game(1);
    for q in ["qpower_boot_sequence", "qfirst_shipment", "qfull_load"] {
        g.career.completed_quests.insert(q.into());
    }
    g.run.stats.trade_credits = f("1400");
    r.advance(&mut g, 1);
    let q = r.view(&g).quest.unwrap();
    assert_eq!((q.id, q.progress.map(|p| p.now)), ("qtrade", Some(f("1400"))), "pay before its start counts");
    // An earlier run's pay doesn't.
    g.career.lifetime.trade_credits = f("5000");
    r.advance(&mut g, 1);
    assert_eq!(r.view(&g).quest.unwrap().id, "qtrade");
    g.run.stats.trade_credits = f("1500");
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

/// Playtest batch 1 (093): 1.4 is trade credits with no speculator data in the
/// view before speculators come; the Blueprints cost 10 and 20; building cards
/// carry a line only for a mechanic their stats don't show; the first Research
/// Lab opens its research items silently; Regolith Processing digs 50% more.
#[test]
fn playtest_batch_1_reads_as_ruled() {
    let r = rules();
    let mut g = r.new_game(1);
    let v = r.view(&g);
    let cost = |id: &str| v.research.iter().find(|x| x.id == id).unwrap().cost;
    assert_eq!((cost("basic_blueprints"), cost("advanced_blueprints")), (f("10"), f("20")));
    let lines: Vec<_> = v.buildings.iter().filter(|b| !b.description.is_empty()).map(|b| b.id).collect();
    assert_eq!(lines, ["arbitrage_engine", "microwave_receiver"]);
    // Through 1.3: 1.4 asks for trade credits, and no market has speculator data.
    let chain: Vec<_> = crate::content::content().events.iter().filter(|e| e.is_quest()).collect();
    for e in chain.iter().take_while(|e| e.id != "qtrade") {
        g.career.completed_quests.insert(e.id.clone());
    }
    let v = r.view(&g);
    let q = v.quest.as_ref().unwrap();
    assert_eq!((q.id, q.kind, q.beat), ("qtrade", "trade_credits", Some("1.4")));
    assert!(!q.title.contains("peculator") && !q.goal.contains("peculator") && q.story.iter().all(|s| !s.contains("peculator")));
    assert!(v.markets.iter().all(|m| m.speculators.is_none()));
    assert!(!g.run.enabled_panels.contains("adversaries"));
    // 1.4 counts this run's launch pay only (trade_counts_the_whole_run).
    r.advance(&mut g, 1);
    g.run.stats.trade_credits = f("1500");
    r.advance(&mut g, 1);
    assert_eq!(r.view(&g).quest.unwrap().id, "qa_voice");
    // A Voice's start opens Projects and four global projects at their costs (Matt,
    // 2026-11-26), and 1.c hangs beside A Voice.
    assert!(g.run.enabled_panels.contains("projects"));
    let v = r.view(&g);
    let open: Vec<_> = v.projects.iter().filter(|p| p.unlocked).map(|p| (p.id, p.costs["credits"], p.costs["science"])).collect();
    assert_eq!(
        open,
        [
            ("blueprint_archive", f("2000"), f("300")),
            ("foundation_grant", f("10000"), f("2000")),
            ("lunar_cartography", f("1500"), f("200")),
            ("conditional_logic", f("500"), f("100")),
        ]
    );
    let sides: Vec<_> = v.side_quests.iter().map(|s| (s.id, s.parent)).collect();
    assert_eq!(sides, [("qblueprints_for_everyone", Some("qa_voice"))]);
    // The first Research Lab opens its research silently; 1.b starts with it.
    own(&mut g, "research_lab", 1);
    r.advance(&mut g, 1);
    assert!(g.career.flags.contains("research_lab"));
    let sides: Vec<_> = r.view(&g).side_quests.iter().map(|s| (s.id, s.parent)).collect();
    assert_eq!(sides, [("qfull_cluster", Some("qfull_load")), ("qblueprints_for_everyone", Some("qa_voice"))]);
    // A retirement: in run 2 the four stay open from day 0, and Chemical Energy joins them.
    g.run.enabled_panels.insert("retirement".into());
    r.apply(&mut g, &Action::Retire).unwrap();
    assert_eq!(g.career.run_number, 2);
    let open = |g: &GameState| r.view(g).projects.into_iter().filter(|p| p.unlocked).map(|p| p.id).collect::<Vec<_>>();
    assert_eq!(open(&g), ["blueprint_archive", "foundation_grant", "lunar_cartography", "conditional_logic"], "from day 0");
    assert!(g.run.enabled_panels.contains("projects"));
    r.advance(&mut g, 1);
    assert_eq!(open(&g), ["blueprint_archive", "foundation_grant", "lunar_cartography", "chemical_energy", "conditional_logic"]);
    // A retirement before A Voice still opens Projects with three of them.
    let mut g = r.new_game(2);
    g.career.retirements = 1;
    r.advance(&mut g, 1);
    let open: Vec<_> = r.view(&g).projects.iter().filter(|p| p.unlocked).map(|p| p.id).collect();
    assert_eq!(open, ["blueprint_archive", "foundation_grant", "chemical_energy", "conditional_logic"]);
    assert!(g.run.enabled_panels.contains("projects") && r.view(&g).side_quests.is_empty());
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
    g.career.flags.insert(crate::modules::boredom::BURNOUT_FLAG.into());
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

/// Mechanics never live in narrative (docs/rulings/ui.md, *Quest mechanics text*): every
/// quest has a goal and a reward only if it grants something (never "None."), and every event's narrative (a one-off's summary, body,
/// story and payoff) carries no numbers (the He-3 in a name excepted), no `{rate}` and no
/// UI instructions.
#[test]
fn narrative_carries_no_mechanics() {
    for e in crate::content::content().events.iter().filter(|e| !e.silent) {
        let quest = e.is_quest() || e.is_side();
        assert!(!quest || !e.goal.is_empty() && e.reward != "None.", "{}: a goal, and no empty reward line", e.id);
        for t in [&e.summary, &e.body].into_iter().chain(&e.story).chain(&e.payoff) {
            let t = t.replace("Helium-3", "").replace("He-3", "");
            assert!(!t.contains("{rate}"), "{}: a rate in its narrative: {t}", e.id);
            assert!(!t.bytes().any(|b| b.is_ascii_digit()), "{}: a number in its narrative: {t}", e.id);
            for word in ["press ", "Press ", "click", "Click", "Add to Program", "Reward:", "on its card", "'s card"] {
                assert!(!t.contains(word), "{}: an instruction ({word:?}) in its narrative: {t}", e.id);
            }
        }
    }
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
