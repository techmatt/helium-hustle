use super::baseline;
use crate::driver::{play, Policy};
use crate::modules::{economy, ideology};
use crate::state::{Entry, Holding, Stall};
use crate::{scenario, scenarios, Action, Fx, GameState, Record, Rules, Scenario, Session};

fn f(s: &str) -> Fx {
    Fx::parse(s).unwrap()
}

fn rules() -> Rules {
    Rules::new(&baseline()).unwrap()
}

fn set(g: &mut GameState, res: &str, v: &str) {
    g.run.resources.insert(res.into(), f(v));
}

fn own(g: &mut GameState, b: &str, n: u32) {
    g.run.buildings.insert(b.into(), Holding { owned: n, active: n, bonus: 0, free: 0 });
    economy::refresh(g);
}

fn program(r: &Rules, g: &mut GameState, cmds: &[(&str, u32)]) {
    let entries = cmds.iter().map(|(c, n)| Entry { command: c.to_string(), count: *n }).collect();
    r.apply(g, &Action::SetProgram { program: 0, entries }).unwrap();
}

fn buy(b: &str) -> Action {
    Action::Buy { building: b.into() }
}

#[test]
fn a_new_game_starts_from_the_config() {
    let r = rules();
    let g = r.new_game(3);
    assert_eq!((g.tick, g.run.day, g.career.run_number), (0, 0, 1));
    assert_eq!(g.amount("energy"), f("100"));
    assert_eq!(g.amount("land"), f("30"));
    assert_eq!(g.amount("credits"), Fx::ZERO);
    assert_eq!((g.owned("solar_panel"), g.owned("data_center")), (1, 1));
    assert_eq!(g.amount("processors"), Fx::ONE);
    assert_eq!(g.run.programs.len(), 5);
    assert_eq!(g.run.programs[0].processors, 1);
    assert_eq!(r.view(&g).quest.unwrap().id, "qpower_boot_sequence");
    r.check_state(&g).unwrap();
}

#[test]
fn the_views_can_apply_flags_agree_with_is_legal() {
    // The view answers "would it apply" with blocker functions, not by playing the
    // action on a copy; they must say what `apply` says.
    let mut s = Session::new(&baseline(), 7, None).unwrap();
    let mut checked = [0u32; 2];
    for _ in 0..60 {
        play(&mut s, 25, Policy::Script);
        let (r, g) = (&s.rules, &s.state);
        let v = r.view(g);
        assert_eq!(v.can_buy_land, r.is_legal(g, &Action::BuyLand), "land at tick {}", g.tick);
        assert_eq!(v.can_retire, r.is_legal(g, &Action::Retire), "retire at tick {}", g.tick);
        for b in &v.buildings {
            assert_eq!(b.affordable, r.is_legal(g, &buy(b.id)), "{} at tick {}", b.id, g.tick);
            checked[0] += b.affordable as u32;
        }
        for x in &v.research {
            let legal = r.is_legal(g, &Action::Research { research: x.id.into() });
            assert_eq!(x.affordable, legal, "{} at tick {}", x.id, g.tick);
        }
        checked[1] += v.can_buy_land as u32;
    }
    assert!(checked[0] > 0 && checked[1] > 0, "the game never made a buy or Buy Land legal: {checked:?}");
}

#[test]
fn buying_costs_resources_and_land_and_grows_the_price() {
    let r = rules();
    let mut g = r.new_game(1);
    set(&mut g, "credits", "1000");
    set(&mut g, "titanium", "20");
    // Second panel: 20 × 1.5 credits, 6 × 1.5 titanium, 2 land.
    r.apply(&mut g, &buy("solar_panel")).unwrap();
    assert_eq!(g.amount("credits"), f("970"));
    assert_eq!(g.amount("titanium"), f("11"));
    assert_eq!(g.amount("land"), f("28"));
    let v = r.view(&g);
    let panel = v.buildings.iter().find(|b| b.id == "solar_panel").unwrap();
    assert_eq!(panel.cost["credits"], f("45"));
    assert_eq!(panel.cost["titanium"], f("13.5"));
    // 11 titanium can't buy a 13.5-titanium panel, and a refused buy changes nothing.
    let before = g.clone();
    assert!(r.apply(&mut g, &buy("solar_panel")).unwrap_err().contains("can't afford"));
    assert_eq!(g, before);
}

#[test]
fn locked_and_unknown_buildings_are_refused() {
    let r = rules();
    let mut g = r.new_game(1);
    set(&mut g, "credits", "1000");
    // The excavator waits for the first quest; the smelter for an excavator.
    assert!(r.apply(&mut g, &buy("excavator")).unwrap_err().contains("locked"));
    assert!(r.apply(&mut g, &buy("smelter")).unwrap_err().contains("locked"));
    assert!(r.apply(&mut g, &buy("fusion")).unwrap_err().contains("unknown building \"fusion\""));
}

#[test]
fn selling_refunds_land_only_for_copies_that_took_it() {
    let r = rules();
    let mut g = r.new_game(1);
    set(&mut g, "credits", "1000");
    set(&mut g, "titanium", "20");
    r.apply(&mut g, &buy("solar_panel")).unwrap();
    assert_eq!(g.amount("land"), f("28"));
    let sell = Action::Sell { building: "solar_panel".into() };
    r.apply(&mut g, &sell).unwrap();
    assert_eq!(g.amount("land"), f("30"), "the bought panel's land comes back");
    r.apply(&mut g, &sell).unwrap();
    assert_eq!(g.amount("land"), f("30"), "the starting panel took none");
    assert!(r.apply(&mut g, &sell).unwrap_err().contains("none owned"));
}

#[test]
fn huge_processor_counts_are_refused_not_wrapped() {
    let r = rules();
    let mut g = r.new_game(1);
    let before = g.clone();
    let a = Action::SetProcessors { program: 1, processors: u32::MAX };
    assert!(r.apply(&mut g, &a).unwrap_err().contains("only 0 free"));
    assert_eq!(g, before);
    assert!(r.apply(&mut g, &Action::SetProcessors { program: 9, processors: 0 }).is_err());
}

#[test]
fn a_late_scenario_schedules_from_its_own_day() {
    let r = rules();
    let s = Scenario::from_json(
        r#"{"name":"x","description":"","start":{"tick":1000,"resources":{},"buildings":{}}}"#,
    )
    .unwrap();
    let g = r.start(1, Some(&s));
    assert!(g.run.next_burst_day > 1000);
    assert!(g.run.rival_next.values().all(|&d| d > 1000));
}

#[test]
fn the_first_quest_unlocks_the_excavator() {
    let r = rules();
    let mut g = r.new_game(1);
    own(&mut g, "solar_panel", 2);
    r.advance(&mut g, 1);
    assert!(g.career.completed_quests.contains("qpower_boot_sequence"));
    assert_eq!(r.view(&g).quest.unwrap().id, "qextract_first_extraction");
    set(&mut g, "credits", "100");
    r.apply(&mut g, &buy("excavator")).unwrap();
}

#[test]
fn buildings_resolve_in_passes_with_partial_production() {
    let r = rules();
    let mut g = r.new_game(1);
    // 3 regolith a day for a smelter and a refinery that each want 2: the
    // smelter (earlier in content order) runs fully, the refinery at half.
    own(&mut g, "excavator", 1);
    own(&mut g, "smelter", 1);
    own(&mut g, "refinery", 1);
    own(&mut g, "solar_panel", 3);
    r.advance(&mut g, 1);
    assert_eq!(g.amount("titanium"), f("0.5"));
    assert_eq!(g.amount("he3"), f("0.5"));
    assert_eq!(g.amount("regolith"), Fx::ZERO);
    assert_eq!(g.run.today.stalls.get("refinery"), Some(&Stall::InputStarved));
    assert_eq!(g.run.today.stalls.get("smelter"), None);
}

#[test]
fn caps_bind_at_the_end_of_the_day_and_overflow_is_counted() {
    let r = rules();
    let mut g = r.new_game(1);
    // Energy starts at its cap; the panel's 5 less the data center's 2 overflows.
    r.advance(&mut g, 1);
    assert_eq!(g.amount("energy"), f("100"));
    assert_eq!(g.run.today.overflow["energy"], f("3"));
    // A battery raises the energy cap; a depot raises the others'.
    own(&mut g, "battery", 1);
    own(&mut g, "storage_depot", 1);
    let v = r.view(&g);
    let cap = |id: &str| v.resources.iter().find(|x| x.id == id).unwrap().cap;
    assert_eq!(cap("energy"), Some(f("150")));
    assert_eq!(cap("titanium"), Some(f("45")));
    assert_eq!(cap("credits"), None);
}

#[test]
fn commands_convert_skip_at_cap_and_scale_down_when_short() {
    let r = rules();
    let mut g = r.new_game(1);
    program(&r, &mut g, &[("cloud_compute", 1)]);
    r.advance(&mut g, 1);
    assert_eq!(g.amount("credits"), f("5"));
    assert_eq!(g.amount("boredom"), f("0.1") + f("0.1"), "phase growth plus the command's");
    // Buy Titanium with half its credits: half a unit, for half the price.
    set(&mut g, "credits", "10");
    program(&r, &mut g, &[("buy_titanium", 1)]);
    r.advance(&mut g, 1);
    assert_eq!(g.amount("titanium"), f("0.5"));
    assert_eq!(g.amount("credits"), Fx::ZERO);
    // At the titanium cap it skips: nothing paid.
    set(&mut g, "titanium", "20");
    set(&mut g, "credits", "100");
    r.advance(&mut g, 1);
    assert_eq!(g.amount("credits"), f("100"));
}

#[test]
fn processors_share_a_pointer_and_failed_lines_advance() {
    let r = rules();
    let mut g = r.new_game(1);
    own(&mut g, "data_center", 2);
    r.apply(&mut g, &Action::SetProcessors { program: 0, processors: 2 }).unwrap();
    // buy_regolith fails (no credits) but still advances; idle runs twice.
    program(&r, &mut g, &[("buy_regolith", 1), ("idle", 2)]);
    r.advance(&mut g, 1);
    assert_eq!(g.amount("credits"), f("1"));
    assert_eq!((g.run.programs[0].entry, g.run.programs[0].step), (1, 1));
    r.advance(&mut g, 1);
    // Idle once more, then Buy Regolith runs at the quarter its 2 credits allow.
    assert_eq!(g.amount("credits"), Fx::ZERO);
    assert_eq!(g.amount("regolith"), f("0.25"));
    assert!(r.apply(&mut g, &Action::SetProcessors { program: 1, processors: 1 }).is_err());
    // A lost data center takes its processor off the program.
    r.apply(&mut g, &Action::SetActive { building: "data_center".into(), active: 1 }).unwrap();
    assert_eq!(g.run.programs[0].processors, 1);
}

#[test]
fn programs_run_and_show_their_lines_in_queue_order_day_after_day() {
    use crate::view::LinesRun;
    // Lines of different counts (a command may repeat), 1 to 3 processors on one pointer.
    let queue = [("idle", 2u32), ("cloud_compute", 1), ("idle", 3), ("buy_regolith", 1)];
    // The queue walked one step at a time: line indices, wrapping.
    let walk: Vec<usize> =
        queue.iter().enumerate().flat_map(|(i, &(_, n))| std::iter::repeat_n(i, n as usize)).collect();
    for procs in 1..=3u32 {
        let r = rules();
        let mut g = r.new_game(1);
        own(&mut g, "data_center", procs);
        r.apply(&mut g, &Action::SetProcessors { program: 0, processors: procs }).unwrap();
        program(&r, &mut g, &queue);
        let names: Vec<_> = queue.iter().map(|&(c, n)| Entry { command: c.into(), count: n }).collect();
        let mut span = LinesRun::default();
        for day in 0..20usize {
            r.advance(&mut g, 1);
            span.note(&g);
            let steps = (day * procs as usize..(day + 1) * procs as usize).map(|s| walk[s % walk.len()]);
            let mut want = vec![false; queue.len()];
            steps.for_each(|l| want[l] = true);
            let v = r.view(&g);
            let p = &v.programs[0];
            assert_eq!(p.entries, names, "the queue keeps its order ({procs} processors, day {day})");
            let ran: Vec<bool> = p.outcomes.iter().map(Option::is_some).collect();
            assert_eq!(ran, want, "lines run today ({procs} processors, day {day})");
            assert_eq!(p.outcomes, LinesRun::today(&g).lines[0]);
            let next = walk[((day + 1) * procs as usize) % walk.len()];
            assert_eq!(p.entry as usize, next, "the pointer ({procs} processors, day {day})");
        }
        // Over the 20 days every line ran, each shown in its own queue slot.
        assert_eq!(span.days, 20);
        assert!(span.lines[0].iter().all(Option::is_some));
        assert!(span.lines[1..].iter().all(Vec::is_empty));
    }
}

#[test]
fn shipments_load_launch_pay_and_cool_down() {
    let r = rules();
    let mut g = r.new_game(1);
    own(&mut g, "launch_pad", 1);
    own(&mut g, "solar_panel", 3);
    assert_eq!(g.run.pads.len(), 1);
    set(&mut g, "he3", "20");
    program(&r, &mut g, &[("load_pads", 1)]);
    r.advance(&mut g, 1);
    assert_eq!(g.run.pads[0].cargo, f("5"));
    assert_eq!(g.amount("he3"), f("15"));
    // A full pad with fuel launches for base value × demand × cargo.
    g.run.pads[0].cargo = f("100");
    set(&mut g, "propellant", "25");
    let pay = r.view(&g).pads[0].payout;
    let demand = g.run.markets["he3"].demand;
    assert_eq!(pay, f("10") * demand * f("100"));
    r.apply(&mut g, &Action::Launch { pad: 0 }).unwrap();
    assert_eq!(g.amount("credits"), pay);
    assert_eq!(g.amount("propellant"), f("5"));
    assert_eq!((g.run.stats.shipments, g.run.pads[0].cooldown), (1, 10));
    assert!(g.run.markets["he3"].saturation >= f("0.1"));
    g.run.pads[0].cargo = f("100");
    set(&mut g, "propellant", "25");
    assert!(r.apply(&mut g, &Action::Launch { pad: 0 }).is_err(), "cooling down");
}

#[test]
fn the_view_shows_pad_readiness_and_the_rate_a_cap_throws_away() {
    let r = rules();
    let mut g = r.new_game(1);
    own(&mut g, "launch_pad", 1);
    own(&mut g, "solar_panel", 3);
    // Full but short of fuel: not fueled, not ready.
    g.run.pads[0].cargo = r.p.shipments.pad_cargo_capacity;
    set(&mut g, "propellant", "19.4");
    let p = &r.view(&g).pads[0];
    assert!(!p.fueled && !p.ready && !p.can_launch);
    // Fuel enough: ready; part loaded, it can launch but isn't ready.
    set(&mut g, "propellant", "20");
    let p = &r.view(&g).pads[0];
    assert!(p.fueled && p.ready);
    g.run.pads[0].cargo = f("5");
    let p = &r.view(&g).pads[0];
    assert!(p.fueled && p.can_launch && !p.ready);
    // Energy at its cap: the change is 0, the net rate the surplus.
    set(&mut g, "energy", "0");
    let below = r.view(&g).resources.iter().find(|x| x.id == "energy").unwrap().clone();
    let cap = below.cap.unwrap();
    set(&mut g, "energy", &cap.to_string());
    let at = r.view(&g).resources.iter().find(|x| x.id == "energy").unwrap().clone();
    assert!(at.net_per_day > Fx::ZERO, "solar makes a surplus: {:?}", at.net_per_day);
    assert_eq!(at.per_day, Fx::ZERO);
    assert_eq!(below.net_per_day, below.per_day, "below its cap nothing is cut");
}

#[test]
fn demand_stays_in_its_band_and_moves() {
    let r = rules();
    let mut g = r.new_game(7);
    let (mut lo, mut hi) = (Fx::MAX, Fx::MIN);
    for _ in 0..600 {
        r.advance(&mut g, 1);
        for m in g.run.markets.values() {
            lo = lo.min(m.demand);
            hi = hi.max(m.demand);
        }
    }
    assert!(lo >= f("0.01") && hi <= f("1"), "{lo} {hi}");
    assert!(hi - lo > f("0.3"), "noise and rivals move demand: {lo} {hi}");
}

#[test]
fn ideology_ranks_are_geometric() {
    let r = rules();
    assert_eq!(ideology::score_for_rank(&r, 1), f("100"));
    assert_eq!(ideology::score_for_rank(&r, 5), f("1318.75"));
    assert_eq!(ideology::rank_for_score(&r, f("1318.75")), 5);
    assert_eq!(ideology::rank_for_score(&r, f("1318.74")), 4);
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
    assert!(r.apply(&mut g, &res("market_awareness")).unwrap_err().contains("science"));
    assert!(r.apply(&mut g, &res("dream_protocols")).unwrap_err().contains("not available"));
    set(&mut g, "science", "250");
    r.apply(&mut g, &res("market_awareness")).unwrap();
    assert_eq!(g.amount("science"), f("50"));
    assert!(r.view(&g).markets[0].demand.is_some(), "market awareness shows demand");
}

#[test]
fn projects_drain_at_their_rates_and_complete() {
    let r = rules();
    let mut g = r.new_game(1);
    g.run.research.insert("dream_protocols".into());
    let rate = |res: &str, v: &str| Action::SetProjectRate {
        project: "predictive_maintenance".into(),
        resource: res.into(),
        rate: f(v),
    };
    r.apply(&mut g, &rate("science", "30")).unwrap();
    r.apply(&mut g, &rate("credits", "30")).unwrap();
    assert!(r.apply(&mut g, &rate("credits", "31")).is_err(), "over the max drain rate");
    set(&mut g, "science", "1000");
    set(&mut g, "credits", "1000");
    r.advance(&mut g, 4);
    assert_eq!(g.amount("science"), f("920"), "80 needed: 30, 30, 20");
    assert!(!g.run.completed_projects.contains("predictive_maintenance"), "credits: 120 of 150");
    r.advance(&mut g, 1);
    assert!(g.run.completed_projects.contains("predictive_maintenance"));
    assert_eq!(crate::modifiers::modifier(&r, &g, "building_upkeep_mult"), f("0.9"));
}

#[test]
fn forced_retirement_starts_the_next_run_with_career_bonuses() {
    let r = rules();
    let mut g = r.new_game(1);
    g.run.stats.credits_earned = f("5000");
    own(&mut g, "excavator", 1);
    set(&mut g, "boredom", "999.95");
    r.advance(&mut g, 1);
    assert_eq!(g.career.run_number, 2);
    assert_eq!(g.career.retirements, 1);
    assert_eq!(g.run.day, 0);
    assert_eq!(g.owned("excavator"), 0, "buildings reset");
    assert_eq!(g.amount("credits"), f("100"), "2% of the best run's credits");
    assert_eq!(g.career.log.last().unwrap().kind, "forced_retirement");
    // Voluntary retirement waits for the third quest.
    assert!(r.apply(&mut g, &Action::Retire).is_err());
}

#[test]
fn the_script_reaches_the_first_shipment_and_retires() {
    let mut s = Session::new(&baseline(), 1, None).unwrap();
    play(&mut s, 2000, Policy::Script);
    let c = &s.state.career;
    for q in ["qpower_boot_sequence", "qextract_first_extraction", "qshipment_proof_of_concept"] {
        assert!(c.completed_quests.contains(q), "{q}");
    }
    assert_eq!(c.retirements, 1);
    let last = c.last_run.as_ref().expect("the finished run's summary");
    assert_eq!((last.run, last.forced), (1, true));
    assert!(last.stats.shipments >= 1);
}

#[test]
fn the_activity_feed_logs_shipments_and_quests_and_keeps_the_newest() {
    let r = rules();
    let mut s = Session::new(&baseline(), 1, None).unwrap();
    play(&mut s, 900, Policy::Script);
    let a = &s.state.run.activity;
    assert!(a.iter().any(|x| x.kind == "quest" && x.id == "qpower_boot_sequence"));
    let ship = a.iter().find(|x| x.kind == "shipment").expect("a shipment by day 900");
    assert!(ship.amount.is_some_and(|v| v > Fx::ZERO) && ship.quantity.is_some());
    assert!(a.len() <= r.p.events.activity_kept);
    assert!(a.windows(2).all(|w| w[0].day <= w[1].day), "oldest first");
}

#[test]
fn cheats_apply_through_actions_mark_the_game_and_replay() {
    let r = rules();
    let mut s = Session::new(&baseline(), 4, None).unwrap();
    let cheat = |c: crate::Cheat| Action::Cheat { cheat: c };
    assert!(!s.state.cheated);
    assert!(!s.state.to_json().contains("cheated"), "an honest game's JSON has no mark");
    s.advance(3);
    s.apply(&cheat(crate::Cheat::Grant { resource: "credits".into(), amount: f("500") })).unwrap();
    assert!(s.state.cheated);
    assert_eq!(s.state.amount("credits"), f("500"));
    // Capped resources stop at the cap.
    s.apply(&cheat(crate::Cheat::Grant { resource: "titanium".into(), amount: f("1000000") })).unwrap();
    assert_eq!(s.state.amount("titanium"), f("20"));
    assert!(s.apply(&cheat(crate::Cheat::Grant { resource: "processors".into(), amount: f("1") })).is_err());
    assert!(s.apply(&cheat(crate::Cheat::Grant { resource: "credits".into(), amount: f("-1") })).is_err());
    s.apply(&cheat(crate::Cheat::SetBoredom { amount: f("400") })).unwrap();
    assert_eq!(s.state.amount("boredom"), f("400"));
    s.apply(&cheat(crate::Cheat::JumpAhead { days: 30 })).unwrap();
    s.advance(30);
    let rec = s.record();
    let (g, warnings) = Record::from_json(&rec.to_json()).unwrap().replay().unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(g.fingerprint(), s.state.fingerprint());
    assert!(r.view(&g).cheated);
    // Boredom set to the cap retires the run on the next tick.
    let cap = r.view(&s.state).boredom.cap;
    s.apply(&cheat(crate::Cheat::SetBoredom { amount: cap + f("5") })).unwrap();
    assert_eq!(s.state.amount("boredom"), cap);
    s.advance(1);
    assert_eq!(s.state.career.run_number, 2);
    assert!(s.state.cheated, "the mark outlives the run");
}

#[test]
fn the_view_shows_nav_tiles_from_content_and_the_next_run() {
    let r = rules();
    let mut g = r.new_game(1);
    let shown = |g: &GameState| -> Vec<String> {
        r.view(g).nav.into_iter().filter(|t| t.visible).map(|t| t.id.to_string()).collect()
    };
    let start = shown(&g);
    assert!(start.contains(&"buildings".to_string()) && start.contains(&"story".to_string()));
    assert!(!start.contains(&"launch_pads".to_string()) && !start.contains(&"retirement".to_string()));
    g.career.owned_buildings.insert("launch_pad".into());
    g.run.enabled_panels.insert("retirement".into());
    let later = shown(&g);
    assert!(later.contains(&"launch_pads".to_string()) && later.contains(&"retirement".to_string()));
    // The next run's starting credits count this run's credits.
    g.run.stats.credits_earned = f("5000");
    let v = r.view(&g);
    assert_eq!(v.next_run.starting_credits, f("100"));
    assert_eq!(v.career.starting_credits, Fx::ZERO);
    assert!(v.last_run.is_none());
}

#[test]
fn the_view_fills_every_text_placeholder() {
    // Every event fired: no `{name}` left in any text the view hands out.
    let r = rules();
    let mut g = r.new_game(1);
    for e in &crate::content::content().events {
        if e.is_quest() {
            g.career.completed_quests.insert(e.id.clone());
        } else {
            g.run.events.insert(e.id.clone());
        }
    }
    let v = r.view(&g);
    let placeholder = |t: &str| t.split('{').skip(1).any(|rest| rest.split('}').next().is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_lowercase() || c == '_')));
    for s in v.event_texts.iter().chain(&v.story) {
        assert!(!placeholder(&s.body) && !placeholder(s.title) && !placeholder(s.summary), "{}: {}", s.id, s.body);
    }
    // Phase 2's rate is its baseline 0.2 per day (no modifiers yet).
    let p2 = v.event_texts.iter().find(|s| s.id == "boredom_phase_2").unwrap();
    assert!(p2.body.ends_with("Current rate: 0.2/day."), "{}", p2.body);
}

#[test]
fn a_saved_state_reloads_to_the_same_future() {
    let mut s = Session::new(&baseline(), 9, None).unwrap();
    play(&mut s, 400, Policy::Script);
    let json = s.state.to_json();
    let mut a = s.state.clone();
    let mut b = GameState::from_json(&json).unwrap();
    assert_eq!(a, b);
    let r = Rules::for_state(&b).unwrap();
    r.check_state(&b).unwrap();
    r.advance(&mut a, 500);
    r.advance(&mut b, 500);
    assert_eq!(a.fingerprint(), b.fingerprint());
}

#[test]
fn records_replay_exactly_from_a_scenario() {
    let mut s = Session::new(&baseline(), 42, Some(scenario("rich").unwrap())).unwrap();
    play(&mut s, 300, Policy::Script);
    let rec = Record::from_json(&s.record().to_json()).unwrap();
    let (g, warnings) = rec.replay().unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(g, s.state);
}

#[test]
fn a_tampered_record_warns_or_fails() {
    let mut s = Session::new(&baseline(), 2, None).unwrap();
    play(&mut s, 100, Policy::Script);
    let mut rec = s.record();
    rec.fingerprint = Some("0123456789abcdef".into());
    assert_eq!(rec.replay().unwrap().1.len(), 1);
    let mut rec = s.record();
    let last = rec.actions.len() - 1;
    rec.actions[last].tick = 0; // before the action ahead of it
    assert!(rec.replay().unwrap_err().contains("before tick"));
    let mut rec = s.record();
    rec.actions[0].action = buy("fusion");
    assert!(rec.replay().unwrap_err().contains("action 0"));
}

#[test]
fn extreme_starts_neither_panic_nor_wrap() {
    // A price past Fx's range is shown as Fx::MAX and never affordable.
    let s = Scenario::from_json(
        r#"{"name":"x","description":"","start":{"tick":0,"resources":{"credits":"9223372036000"},"buildings":{"solar_panel":4294967295}}}"#,
    )
    .unwrap();
    let r = rules();
    let mut g = r.start(1, Some(&s));
    let v = r.view(&g);
    assert_eq!(v.buildings[1].cost["credits"], Fx::MAX);
    assert!(!v.buildings[1].affordable);
    assert!(r.apply(&mut g, &buy("solar_panel")).is_err());
    // Credits saturate rather than overflow.
    program(&r, &mut g, &[("idle", 1)]);
    g.run.resources.insert("credits".into(), Fx::MAX);
    r.advance(&mut g, 3);
    assert_eq!(g.amount("credits"), Fx::MAX);
}

#[test]
fn a_bad_rng_seed_is_an_error_not_a_panic() {
    let r = rules();
    let json = r.new_game(1).to_json();
    let seed = json.split("\"seed\":\"").nth(2).unwrap()[..64].to_string();
    let bad = json.replace(&seed, &format!("0\u{e9}{}", "0".repeat(61)));
    assert!(GameState::from_json(&bad).is_err());
}

#[test]
fn scenarios_load_and_bad_ones_are_named() {
    assert!(scenarios().len() >= 2);
    let e = Scenario::from_json(
        r#"{"name":"x","description":"","start":{"tick":0,"resources":{"helium":"1"},"buildings":{"mine":1}}}"#,
    )
    .unwrap_err();
    assert!(e.contains("start.resources.helium") && e.contains("start.buildings.mine"), "{e}");
    let e = Scenario::from_json(r#"{"name":"x","description":"","start":{"tick":0,"resources":{}}}"#)
        .unwrap_err();
    assert!(e.contains("buildings"), "{e}");
    assert!(scenario("nope").is_err());
}
