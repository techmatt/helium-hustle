use super::{baseline, f, rules, set};
use crate::driver::{play, Policy};
use crate::modules::{economy, ideology};
use crate::state::{Entry, Holding, Stall};
use crate::{scenario, scenarios, Action, Fx, GameState, Record, Rules, Scenario, Session};

fn own(g: &mut GameState, b: &str, n: u32) {
    g.run.buildings.insert(b.into(), Holding { owned: n, active: n, bonus: 0, free: 0, unpaid: 0 });
    economy::refresh(g);
}

fn program(r: &Rules, g: &mut GameState, cmds: &[(&str, u32)]) {
    let entries = cmds.iter().map(|(c, n)| Entry::command(c, *n)).collect();
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
        r#"{"name":"x","description":"","start":{"tick":1000,"resources":{},"buildings":{},"panels":[],"programs":[]}}"#,
    )
    .unwrap();
    let g = r.start(1, Some(&s));
    assert!(g.run.next_burst_day > 1000);
    assert!(g.run.rival_next.values().all(|&d| d > 1000));
}

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
    assert_eq!(v.quest.unwrap().id, "qblueprints");
    assert!(v.commands.iter().any(|c| c.id == "buy_science" && c.unlocked));
    // The research view shows only the Blueprints before 1.8 Lab.
    let shown: Vec<&str> = v.research.iter().filter(|x| x.visible).map(|x| x.id).collect();
    assert_eq!(shown, ["basic_blueprints"]);
    set(&mut g, "credits", "100");
    assert!(r.apply(&mut g, &buy("excavator")).is_err(), "the excavator waits for Basic Blueprints");
    set(&mut g, "science", "20");
    r.apply(&mut g, &Action::Research { research: "basic_blueprints".into() }).unwrap();
    r.apply(&mut g, &buy("excavator")).unwrap();
    r.advance(&mut g, 1);
    // 1.3: 20 propellant once, pads carry Titanium only.
    assert_eq!(r.view(&g).quest.unwrap().id, "qfirst_shipment");
    assert_eq!(g.amount("propellant"), f("20"));
    assert_eq!(r.view(&g).pad_goods, ["titanium"]);
    own(&mut g, "launch_pad", 1);
    assert!(r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some("he3".into()) }).is_err());
    r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some("titanium".into()) }).unwrap();
    assert!(!r.view(&g).commands.iter().any(|c| c.id == "launch_pads" && c.unlocked), "Launch Full Pads waits for 1.5");
    g.run.pads[0].cargo = f("20");
    r.apply(&mut g, &Action::Launch { pad: 0 }).unwrap();
    r.advance(&mut g, 1);
    assert_eq!(r.view(&g).quest.unwrap().id, "qfull_load");
    assert!(r.view(&g).pad_goods.len() > 1, "every good after 1.3");
    assert!(r.view(&g).research.iter().any(|x| x.id == "advanced_blueprints" && x.visible));
}

#[test]
fn buildings_resolve_in_passes_with_partial_production() {
    let r = rules();
    let mut g = r.new_game(1);
    // 4 regolith a day for a smelter that wants 2 and a refinery that wants 5:
    // the smelter (earlier in content order) runs fully, the refinery on the 2 left.
    own(&mut g, "excavator", 2);
    own(&mut g, "smelter", 1);
    own(&mut g, "refinery", 1);
    own(&mut g, "solar_panel", 4);
    r.advance(&mut g, 1);
    assert_eq!(g.amount("titanium"), f("0.5"));
    assert_eq!(g.amount("he3"), f("0.4"));
    assert_eq!(g.amount("regolith"), Fx::ZERO);
    assert_eq!(g.run.today.stalls.get("refinery"), Some(&Stall::InputStarved));
    assert_eq!(g.run.today.stalls.get("smelter"), None);
}

#[test]
fn unpaid_upkeep_withholds_effects_from_the_next_day() {
    // Three Data Centers want 6 energy; with none to spare they run at 0 and grant no
    // processors once the day is done; paid again, they grant all three.
    let r = rules();
    let mut g = r.new_game(1);
    g.run.buildings.clear();
    own(&mut g, "data_center", 3);
    assert_eq!(programs_processors(&g), 3);
    set(&mut g, "energy", "0");
    r.advance(&mut g, 1);
    assert_eq!(g.run.buildings["data_center"].unpaid, 3);
    assert_eq!(programs_processors(&g), 0);
    // Unpaid, not gone: the program keeps its processor assigned, idle until paid.
    assert_eq!((g.run.programs[0].processors, crate::modules::programs::running(&g)[0]), (1, 0));
    // Half paid (3 of 6): floor(3 × 0.5) = 1 works.
    set(&mut g, "energy", "3");
    r.advance(&mut g, 1);
    assert_eq!((g.run.buildings["data_center"].unpaid, programs_processors(&g)), (2, 1));
    // Switching one off takes an unpaid copy: the paid one keeps its processor.
    r.apply(&mut g, &Action::SetActive { building: "data_center".into(), active: 2 }).unwrap();
    assert_eq!((g.run.buildings["data_center"].unpaid, programs_processors(&g)), (1, 1));
    r.apply(&mut g, &Action::SetActive { building: "data_center".into(), active: 3 }).unwrap();
    set(&mut g, "energy", "6");
    r.advance(&mut g, 1);
    assert_eq!((g.run.buildings["data_center"].unpaid, programs_processors(&g)), (0, 3));
}

fn programs_processors(g: &GameState) -> u32 {
    crate::modules::programs::processors(g)
}

#[test]
fn a_partial_pass_building_that_ran_in_full_is_not_starved() {
    // The Fabricator, short of regolith, runs at 0.9 in the partial pass and makes the
    // circuits the Research Lab (later in content order) lacked: the Lab runs in full and
    // is neither stalled nor short.
    let r = rules();
    let mut g = r.new_game(1);
    own(&mut g, "solar_panel", 4);
    own(&mut g, "fabricator", 1);
    own(&mut g, "research_lab", 1);
    set(&mut g, "regolith", "0.9");
    set(&mut g, "circuits", "0");
    r.advance(&mut g, 1);
    assert_eq!(g.run.today.stalls.get("fabricator"), Some(&Stall::InputStarved));
    assert_eq!(g.run.today.flows["science"]["building:research_lab"].produced, f("1"));
    assert_eq!(g.run.today.stalls.get("research_lab"), None);
    assert!(!g.run.today.shorts.contains_key("building:research_lab"));
}

#[test]
fn programs_are_paid_before_upkeep_and_the_breakdown_shows_who_went_short() {
    let r = rules();
    let mut g = r.start(1, Some(scenario("late_unlocks").unwrap()));
    let v = r.view(&g);
    // Every Act 1 tile open; the later acts' five (draft) open with their quests.
    assert_eq!(v.nav.iter().filter(|t| t.visible).count(), v.nav.len() - 5, "every Act 1 tile open");
    // A scenario's pads start with No cargo, like a new run's.
    assert!(g.run.pads.iter().all(|p| p.good.is_none()));
    for pad in 0..g.run.pads.len() as u32 {
        r.apply(&mut g, &Action::SetPadGood { pad, good: Some("he3".into()) }).unwrap();
    }
    r.advance(&mut g, 3);
    let v = r.view(&g);
    // Load Launch Pads ran on the Solar Panels' power; a building with upkeep went short.
    assert_eq!(v.programs[0].outcomes[0], Some(crate::state::LineOutcome::Ran));
    // Buy Titanium on 10 credits ran at half on day 1 (a conversion runs part way), then
    // fails and says why.
    assert_eq!(v.programs[0].outcomes[1], Some(crate::state::LineOutcome::Failed));
    let why = v.programs[0].shorts[1].as_ref().unwrap();
    assert_eq!((why.resource.as_str(), why.need, why.had, why.ran), ("credits", f("20"), Fx::ZERO, Fx::ZERO));
    assert!(g.run.pads.iter().any(|p| p.cargo > Fx::ZERO));
    let starved: Vec<_> = v.buildings.iter().filter_map(|b| b.starved.as_ref()).collect();
    assert!(!starved.is_empty() && starved.iter().all(|s| s.ran < Fx::ONE && s.resource == "energy"));
    let energy = v.breakdown.iter().find(|b| b.resource == "energy").unwrap();
    let row = |src: &str| energy.rows.iter().find(|x| x.source == src).unwrap();
    assert_eq!(row("building:solar_panel").produced, f("30"));
    assert_eq!(row("program:0").consumed, f("2"));
    assert_eq!(energy.shorts.len(), starved.len());
    // Every unit made is accounted for: sources less sinks is the change in stock (none:
    // a short day spends it all), but for the clamp lifting a rounding dip below zero.
    let net: Fx = energy.rows.iter().map(|x| x.produced - x.consumed).sum();
    assert!(net.max(-net) <= Fx::EPSILON, "{net}");
    assert!(g.run.flow_average["energy"]["building:solar_panel"].produced > Fx::ZERO);
}

// Three games, one test each so they run side by side.
#[test]
#[ignore = "slow lane: a 900-day game checked day by day (~3-6 s)"]
fn every_days_breakdown_adds_up_to_each_resources_change_new_game() {
    breakdown_adds_up(5, None);
}

#[test]
#[ignore = "slow lane: a 900-day game checked day by day (~3-6 s)"]
fn every_days_breakdown_adds_up_to_each_resources_change_late_unlocks() {
    breakdown_adds_up(6, Some("late_unlocks"));
}

#[test]
#[ignore = "slow lane: a 900-day game checked day by day (~3-6 s)"]
fn every_days_breakdown_adds_up_to_each_resources_change_rich() {
    breakdown_adds_up(7, Some("rich"));
}

fn breakdown_adds_up(seed: u64, scenario: Option<&str>) {
    // The breakdown is the player's answer to "where did it go": for every resource, every
    // day (and the actions after it), its sources less its sinks less what the cap cut is
    // the change in stock, but for the clamp lifting a rounding dip below zero.
    let mut checked = 0;
    {
        let sc = scenario.map(scenario_by);
        let mut s = Session::new(&baseline(), seed, sc).unwrap();
        for _ in 0..900 {
            let before = s.state.clone();
            // Greedy buys after its tick, so the day's flows hold both (the script acts
            // before its tick, whose start clears the flows).
            play(&mut s, 1, Policy::Greedy { every: 1 });
            let g = &s.state;
            if g.career.run_number != before.career.run_number {
                continue;
            }
            for res in &crate::content::content().resources {
                let flows = g.run.today.flows.get(&res.id);
                let net: Fx = flows.map_or(Fx::ZERO, |m| m.values().map(|x| x.produced - x.consumed).sum());
                let change = g.amount(&res.id) - before.amount(&res.id);
                let off = net - change;
                assert!(
                    off.max(-off) <= f("0.000004"),
                    "seed {seed} {scenario:?} day {}: {} flows net {net} but changed {change} ({:?})",
                    g.run.day,
                    res.id,
                    flows
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 1000);
}

fn scenario_by(id: &str) -> &'static Scenario {
    scenario(id).unwrap()
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
        let names: Vec<_> = queue.iter().map(|&(c, n)| Entry::command(c, n)).collect();
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
    r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some("he3".into()) }).unwrap();
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
    g.run.pads[0].good = Some("he3".into());
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
    // Energy at its cap: the change is 0, the rate the uncapped surplus.
    set(&mut g, "energy", "0");
    let below = r.view(&g).resources.iter().find(|x| x.id == "energy").unwrap().clone();
    let cap = below.cap.unwrap();
    set(&mut g, "energy", &cap.to_string());
    let at = r.view(&g).resources.iter().find(|x| x.id == "energy").unwrap().clone();
    assert!(at.per_day > Fx::ZERO && at.overflow == at.per_day, "solar makes a surplus: {:?}", at.per_day);
    assert_eq!(at.change_per_day, Fx::ZERO);
    assert_eq!(below.change_per_day, below.per_day, "below its cap nothing is cut");
    assert!(below.overflow == Fx::ZERO && below.unmet == Fx::ZERO);
}

#[test]
fn switching_a_loaded_pads_cargo_returns_it_up_to_the_cap() {
    let r = rules();
    let mut g = r.new_game(1);
    own(&mut g, "launch_pad", 1);
    let (good, other) = ("he3", "titanium");
    r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some(good.into()) }).unwrap();
    let cap = economy::cap(&r, &g, other).unwrap();
    // Room for all of it: back to the stockpile, nothing lost, no line.
    set(&mut g, good, "0");
    g.run.pads[0].cargo = f("5");
    let lines = g.run.activity.len();
    r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some(other.into()) }).unwrap();
    assert_eq!((g.amount(good), g.run.pads[0].cargo, g.run.pads[0].good.as_deref()), (f("5"), Fx::ZERO, Some(other)));
    assert_eq!(g.run.activity.len(), lines);
    // Two short of the cap: two return, the rest is lost and a line says so.
    g.run.pads[0].cargo = f("12");
    set(&mut g, other, &(cap - f("2")).to_string());
    r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some(good.into()) }).unwrap();
    assert_eq!(g.amount(other), cap);
    let a = g.run.activity.last().unwrap();
    assert_eq!((a.kind.as_str(), a.id.as_str(), a.amount, a.quantity), ("cargo_lost", other, Some(f("1")), Some(f("10"))));
    assert_eq!(g.run.pads[0].cargo, Fx::ZERO);
    // Back to No cargo: the cargo returns the same way.
    g.run.pads[0].cargo = f("3");
    set(&mut g, good, "0");
    r.apply(&mut g, &Action::SetPadGood { pad: 0, good: None }).unwrap();
    assert_eq!((g.amount(good), g.run.pads[0].cargo, g.run.pads[0].good.as_deref()), (f("3"), Fx::ZERO, None));
}

#[test]
fn a_new_pad_has_no_cargo_and_loads_nothing_until_one_is_picked() {
    let r = rules();
    let mut g = r.new_game(1);
    own(&mut g, "launch_pad", 1);
    own(&mut g, "solar_panel", 10);
    set(&mut g, "he3", "20");
    assert_eq!(g.run.pads[0].good, None);
    let v = r.view(&g);
    assert_eq!(v.pads[0].good, None);
    crate::modules::shipments::load(&r, &mut g, None);
    assert_eq!((g.run.pads[0].cargo, g.amount("he3")), (Fx::ZERO, f("20")), "Load skips a No-cargo pad");
    r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some("he3".into()) }).unwrap();
    crate::modules::shipments::load(&r, &mut g, None);
    assert!(g.run.pads[0].cargo > Fx::ZERO);
    assert!(r.apply(&mut g, &Action::SetPadGood { pad: 0, good: Some("energy".into()) }).is_err());
}

#[test]
fn a_save_from_before_no_cargo_keeps_its_pads_goods() {
    // An old save's pad: its good a bare string, as before No cargo.
    let mut old = serde_json::to_value(crate::state::Pad { good: None, paused: false, cargo: f("5"), cooldown: 2 }).unwrap();
    old["good"] = serde_json::json!("titanium");
    let pad: crate::state::Pad = serde_json::from_value(old).unwrap();
    assert_eq!(pad.good.as_deref(), Some("titanium"));
    let old_action = r#"{"type":"set_pad_good","pad":0,"good":"he3"}"#;
    let a: Action = serde_json::from_str(old_action).unwrap();
    assert_eq!(a, Action::SetPadGood { pad: 0, good: Some("he3".into()) });
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
    // No research before 1.1 Online opens the research panel.
    assert!(r.apply(&mut g, &res("basic_blueprints")).unwrap_err().contains("not open"));
    g.run.enabled_panels.insert(crate::modules::research::RESEARCH_PANEL.into());
    assert!(r.apply(&mut g, &res("basic_blueprints")).unwrap_err().contains("science"));
    // Before 1.8 Lab only the Blueprints show.
    assert!(r.apply(&mut g, &res("market_awareness")).unwrap_err().contains("not available"));
    g.career.flags.insert("research_lab".into());
    assert!(r.apply(&mut g, &res("market_awareness")).unwrap_err().contains("science"));
    assert!(r.apply(&mut g, &res("dream_protocols")).unwrap_err().contains("not available"));
    set(&mut g, "science", "250");
    r.apply(&mut g, &res("market_awareness")).unwrap();
    assert_eq!(g.amount("science"), f("50"));
    assert!(r.view(&g).markets[0].demand.is_some(), "market awareness shows demand");
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
    g.run.research.insert("dream_protocols".into());
    let rate = |res: &str, v: &str| Action::SetProjectRate {
        project: "predictive_maintenance".into(),
        resource: res.into(),
        rate: f(v),
    };
    // min_days 5: science at most 80 / 5 = 16 a day, credits 150 / 5 = 30.
    assert!(r.apply(&mut g, &rate("science", "17")).is_err(), "over the most a day");
    r.apply(&mut g, &rate("science", "16")).unwrap();
    r.apply(&mut g, &rate("credits", "30")).unwrap();
    assert!(r.apply(&mut g, &rate("credits", "31")).is_err(), "over the most a day");
    set(&mut g, "science", "1000");
    set(&mut g, "credits", "1000");
    assert_eq!(r.view(&g).projects.iter().find(|p| p.id == "predictive_maintenance").unwrap().days_to_done, Some(5));
    r.advance(&mut g, 4);
    assert_eq!(g.amount("science"), f("936"), "80 needed: 16 a day");
    assert!(!crate::modules::projects::completed(&g, "predictive_maintenance"), "credits: 120 of 150");
    r.advance(&mut g, 1);
    assert_eq!(g.run.projects.completions.get("predictive_maintenance"), Some(&1));
    assert!(g.run.project_rates.is_empty(), "done for good: its rates clear");
    assert_eq!(crate::modifiers::modifier(&r, &g, "building_upkeep_mult"), f("0.9"));
    let v = r.view(&g);
    assert!(v.live_rewards.iter().any(|t| t == "Predictive Maintenance: Building upkeep ×0.9"), "{:?}", v.live_rewards);
}

#[test]
fn a_repeatable_project_grows_its_costs_and_keeps_its_rates() {
    use crate::content::{Project, Repeat, Scope};
    let r = rules();
    let mut g = r.new_game(1);
    // A repeatable copy of a row, driven through the module directly.
    let base = crate::content::content().project("predictive_maintenance").unwrap();
    let p = Project { repeat: Some(Repeat { cost_growth: f("2") }), scope: Scope::Forever, ..base.clone() };
    assert_eq!(crate::modules::projects::cost(&g, &p, "credits"), f("150"));
    g.career.forever_projects.completions.insert(p.id.clone(), 2);
    assert_eq!(crate::modules::projects::cost(&g, &p, "credits"), f("600"), "150 × 2²");
    assert_eq!(crate::modules::projects::max_rate(&g, &p, "credits"), f("120"), "600 / 5 days");
}

#[test]
fn locked_projects_show_their_condition_and_rewards() {
    let r = rules();
    let g = r.new_game(1);
    let v = r.view(&g);
    let find = |id: &str| v.projects.iter().find(|p| p.id == id).unwrap();
    let pm = find("predictive_maintenance");
    assert!(!pm.unlocked);
    assert_eq!(pm.unlock_text, "Research Daydreaming");
    assert_eq!(pm.days_to_done, None, "no rate set");
    assert_eq!(find("microwave_power").reward_text, vec!["Opens Microwave Receiver".to_string()]);
    assert_eq!(find("microwave_power").unlock_text, "Reach rank 5 on the Nationalist axis (best in the career)");
    assert_eq!(find("speculator_dossier").unlock_text, "Use Disrupt Speculators once");
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
    // Voluntary retirement waits for the Burnout quest (1.5).
    assert!(r.apply(&mut g, &Action::Retire).is_err());
}

#[test]
fn the_script_reaches_the_first_shipment_and_retires() {
    let mut s = Session::new(&baseline(), 1, None).unwrap();
    play(&mut s, 2000, Policy::Script);
    let c = &s.state.career;
    for q in ["qpower_boot_sequence", "qblueprints", "qfirst_shipment"] {
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
    // The view's `grantable` is the grant's legality.
    for res in r.view(&s.state).resources {
        let mut g = s.state.clone();
        let ok = crate::cheat::apply(&r, &mut g, &crate::Cheat::Grant { resource: res.id.into(), amount: f("1") }).is_ok();
        assert_eq!(res.grantable, ok, "{}", res.id);
    }
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
fn the_boost_scales_production_marks_the_game_accelerated_and_replays() {
    let cheat = |c: crate::Cheat| Action::Cheat { cheat: c };
    let mut honest = Session::new(&baseline(), 9, None).unwrap();
    let json = honest.state.to_json();
    assert!(!json.contains("accelerated") && !json.contains("boost"), "an honest game's JSON has neither");
    let mut s = Session::new(&baseline(), 9, None).unwrap();
    let boost = crate::Cheat::Boost { production: f("10"), research: f("1"), boredom: f("0") };
    s.apply(&cheat(boost)).unwrap();
    assert!(s.state.accelerated && s.state.cheated && s.state.boost.is_some());
    honest.advance(20);
    s.advance(20);
    // Solar Panels' energy, ten times over (before upkeep takes its share); no boredom.
    let made = |g: &crate::GameState| g.run.stats.produced.get("energy").copied().unwrap_or(Fx::ZERO);
    assert_eq!(made(&s.state), made(&honest.state).mul_int(10));
    assert_eq!(s.state.amount("boredom"), Fx::ZERO);
    assert!(s.apply(&cheat(crate::Cheat::Boost { production: f("0"), research: f("1"), boredom: f("1") })).is_err());
    s.apply(&cheat(crate::Cheat::Autopilot { target: "end".into() })).unwrap();
    let (g, warnings) = Record::from_json(&s.record().to_json()).unwrap().replay().unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(g.fingerprint(), s.state.fingerprint());
}

#[test]
fn the_cheat_building_adds_a_thousand_credits_and_ten_to_every_other_shown_income_and_takes_ten_off_boredom() {
    let r = rules();
    let cheat = |c: crate::Cheat| Action::Cheat { cheat: c };
    let mut honest = Session::new(&baseline(), 11, Some(scenario("everything").unwrap())).unwrap();
    assert!(!honest.state.to_json().contains("cheat_buildings"), "absent while none");
    let mut s = Session::new(&baseline(), 11, Some(scenario("everything").unwrap())).unwrap();
    for x in [&mut honest, &mut s] {
        x.apply(&cheat(crate::Cheat::SetBoredom { amount: f("100") })).unwrap();
    }
    // Spend all but half a credit: one short of the price, it refuses.
    let mut broke = s.state.clone();
    broke.run.resources.insert("credits".into(), f("0.5"));
    assert!(crate::cheat::apply(&r, &mut broke, &crate::Cheat::CheatBuilding).is_err());
    s.apply(&cheat(crate::Cheat::CheatBuilding)).unwrap();
    assert_eq!(s.state.amount("credits"), honest.state.amount("credits") - f("1"));
    assert_eq!(r.view(&s.state).cheat_building.owned, 1);
    let (a, b) = (r.view(&honest.state), r.view(&s.state));
    for (h, c) in a.resources.iter().zip(&b.resources) {
        let want = match c.id {
            "boredom" => f("-10"),
            "land" | "processors" => Fx::ZERO,
            "credits" => f("1000"),
            _ if c.visible => f("10"),
            _ => Fx::ZERO,
        };
        assert_eq!(c.per_day - h.per_day, want, "{}", c.id);
    }
    // A replay plays it the same; a new run has none.
    s.advance(5);
    let (g, warnings) = Record::from_json(&s.record().to_json()).unwrap().replay().unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(g.fingerprint(), s.state.fingerprint());
    assert!(g.cheated);
    s.apply(&Action::Retire).unwrap();
    s.advance(1);
    assert_eq!((s.state.career.run_number, s.state.run.cheat_buildings), (2, 0));
}

#[test]
fn complete_all_research_finishes_every_item_and_marks_the_game() {
    let mut s = Session::new(&baseline(), 12, None).unwrap();
    s.apply(&Action::Cheat { cheat: crate::Cheat::CompleteResearch }).unwrap();
    assert!(s.state.cheated);
    let c = crate::content::content();
    assert!(c.research.iter().all(|i| s.state.run.research.contains(&i.id)));
    assert_eq!(s.state.run.stats.research_completed as usize, c.research.len());
    assert_eq!(s.state.amount("science"), Fx::ZERO, "free");
}

#[test]
fn the_everything_scenario_opens_every_act_1_view() {
    let r = rules();
    let mut g = r.start(13, Some(scenario("everything").unwrap()));
    r.advance(&mut g, 1);
    let v = r.view(&g);
    let shown: Vec<&str> = v.nav.iter().filter(|t| t.visible).map(|t| t.id).collect();
    for id in ["buildings", "commands", "launch_pads", "research", "projects", "ideologies", "adversaries", "stats", "story", "retirement", "options"] {
        assert!(shown.contains(&id), "{id} hidden: {shown:?}");
    }
    assert!(!shown.contains(&"timeline"), "Act 2 stays shut");
    assert!(crate::modules::career::flag_set(&g, "logic_if"));
    assert!(g.run.research.contains("advanced_blueprints"), "the Blueprint Archive's");
    assert_eq!(crate::modules::events::active_quest(&g).map(|e| e.id.as_str()), Some("qdeep_space_telescope"));
    assert!(g.amount("processors") >= f("6"));
    assert!(!g.cheated, "a scenario is no cheat");
    r.advance(&mut g, 30);
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

/// Playtest batch 1 (093): 1.6 is trade credits with no speculator data in the
/// view before speculators come; the Blueprints cost 10 and 20; building cards
/// carry a line only for a mechanic their stats don't show; the first Research
/// Lab opens the catalog silently; Regolith Processing digs 50% more.
#[test]
fn playtest_batch_1_reads_as_ruled() {
    let r = rules();
    let mut g = r.new_game(1);
    let v = r.view(&g);
    let cost = |id: &str| v.research.iter().find(|x| x.id == id).unwrap().cost;
    assert_eq!((cost("basic_blueprints"), cost("advanced_blueprints")), (f("10"), f("20")));
    let lines: Vec<_> = v.buildings.iter().filter(|b| !b.description.is_empty()).map(|b| b.id).collect();
    assert_eq!(lines, ["arbitrage_engine", "microwave_receiver"]);
    // Through 1.5: 1.6 asks for trade credits, and no market has speculator data.
    let chain: Vec<_> = crate::content::content().events.iter().filter(|e| e.is_quest()).collect();
    for e in chain.iter().take_while(|e| e.id != "qtrade") {
        g.career.completed_quests.insert(e.id.clone());
    }
    let v = r.view(&g);
    let q = v.quest.as_ref().unwrap();
    assert_eq!((q.id, q.kind, q.beat), ("qtrade", "trade_credits", Some("1.6")));
    assert!(!q.title.contains("peculator") && !q.summary.contains("peculator") && q.story.iter().all(|s| !s.contains("peculator")));
    assert!(v.markets.iter().all(|m| m.speculators.is_none()));
    assert!(!g.run.enabled_panels.contains("adversaries"));
    // 1.6 counts launch pay only.
    g.run.stats.trade_credits = f("1500");
    r.advance(&mut g, 1);
    assert_eq!(r.view(&g).quest.unwrap().id, "qburnout");
    // The first Research Lab opens what 1.8 did, silently; 1.b and 1.c start with it.
    assert!(!r.view(&g).research.iter().any(|x| x.id == "geopolitical_intelligence" && x.visible));
    own(&mut g, "research_lab", 1);
    r.advance(&mut g, 1);
    assert!(g.career.flags.contains("research_lab") && g.run.enabled_panels.contains("projects"));
    assert!(g.run.enabled_buildings.contains("recreation_dome") && g.run.enabled_projects.contains("foundation_grant"));
    let v = r.view(&g);
    assert!(v.research.iter().any(|x| x.id == "geopolitical_intelligence" && x.visible));
    // 1.b starts beside 1.7; 1.c waits for 1.9, when its Archive is offered.
    let sides: Vec<_> = v.side_quests.iter().map(|s| (s.id, s.parent)).collect();
    assert_eq!(sides, [("qfull_cluster", Some("qburnout"))]);
    g.career.completed_quests.insert("qburnout".into());
    let sides: Vec<_> = r.view(&g).side_quests.iter().map(|s| (s.id, s.parent)).collect();
    assert_eq!(sides, [("qfull_cluster", Some("qburnout")), ("qblueprints_for_everyone", Some("qthree_voices"))]);
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
fn story_timeline_view_has_acts_beats_and_started_side_quests() {
    let r = rules();
    let mut g = r.new_game(1);
    let v = r.view(&g);
    // A new game: Act 1 alone, 1.1 active in it with more to come, no side quest started.
    assert_eq!(v.acts.iter().map(|a| (a.act, a.name)).collect::<Vec<_>>(), vec![(1, "The Lunar Operation")]);
    let q = v.quest.as_ref().unwrap();
    assert_eq!((q.id, q.beat, q.act, q.more), ("qpower_boot_sequence", Some("1.1"), Some(1), true));
    assert!(v.side_quests.is_empty());
    // Through 1.6: no side quest yet; once speculators have come, Big Launch
    // (1.a) has started beside 1.6, not done, with its progress.
    let chain: Vec<_> = crate::content::content().events.iter().filter(|e| e.is_quest()).collect();
    let at = chain.iter().position(|e| e.id == "qtrade").unwrap();
    for e in &chain[..=at] {
        g.career.completed_quests.insert(e.id.clone());
    }
    assert!(r.view(&g).side_quests.is_empty());
    g.career.flags.insert("speculators_seen".into());
    let v = r.view(&g);
    let side: Vec<_> = v.side_quests.iter().map(|s| (s.id, s.parent, s.done)).collect();
    assert_eq!(side, vec![("qbig_launch", Some("qtrade"), false)]);
    assert!(v.side_quests[0].progress.is_some());
    assert!(v.story.iter().filter(|s| s.chain).all(|s| s.act == Some(1) && s.beat.is_some()));
    // The whole chain: every act reached, the last quest has none after it.
    for e in &chain {
        g.career.completed_quests.insert(e.id.clone());
    }
    g.career.completed_quests.remove(&chain[chain.len() - 1].id);
    let v = r.view(&g);
    assert_eq!(v.acts.len(), crate::content::content().acts.len());
    let q = v.quest.as_ref().unwrap();
    assert_eq!((q.act, q.more), (Some(4), false));
    assert_eq!(crate::content::content().act_of("qthe_vault"), Some(3));
    assert_eq!(crate::content::content().act_of("qfilling_vault"), Some(3));
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
            g.run.events.insert(e.id.clone(), 0);
        }
    }
    let v = r.view(&g);
    let placeholder = |t: &str| t.split('{').skip(1).any(|rest| rest.split('}').next().is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_lowercase() || c == '_')));
    // `{rate}` is the UI's to fill (with Stats' formatting), from the view's `rate`.
    let unfilled = |s: &crate::view::StoryView| placeholder(&s.body.replace("{rate}", if s.rate.is_some() { "" } else { "{rate}" }));
    for s in v.event_texts.iter().chain(&v.story) {
        assert!(!unfilled(s) && !placeholder(s.title) && !placeholder(s.summary), "{}: {}", s.id, s.body);
    }
    for c in &v.commands {
        assert!(!placeholder(&c.description), "{}: {}", c.id, c.description);
    }
    // Only the latest boredom phase is listed; each earlier one's text fills in when it is
    // the latest. Phase 2's rate is its baseline 0.2 per day (no modifiers yet).
    for later in ["boredom_phase_5", "boredom_phase_4", "boredom_phase_3"] {
        g.run.events.remove(later);
        let v = r.view(&g);
        for s in &v.event_texts {
            assert!(!unfilled(s), "{}: {}", s.id, s.body);
        }
    }
    let v = r.view(&g);
    let p2 = v.event_texts.iter().find(|s| s.id == "boredom_phase_2").unwrap();
    assert!(p2.body.ends_with("Current rate: {rate}."), "{}", p2.body);
    assert_eq!(p2.rate, Some(f("0.2")));
}

#[test]
fn story_lists_only_the_latest_boredom_phase() {
    let r = rules();
    let mut g = r.new_game(1);
    for id in ["propellant_discovery", "boredom_phase_2", "boredom_phase_3"] {
        g.run.events.insert(id.to_string(), 0);
    }
    let v = r.view(&g);
    let ids: Vec<&str> = v.event_texts.iter().map(|s| s.id).collect();
    // The one-off beat stays; phase 3 supersedes phase 2 (still fired in the state).
    assert_eq!(ids, ["propellant_discovery", "boredom_phase_3"]);
}

#[test]
fn story_lists_events_by_the_day_they_fired() {
    // Content order is propellant_discovery before boredom_phase_2; fired the other way
    // round, Story lists them the other way round (oldest first: the UI shows newest first).
    let r = rules();
    let mut g = r.new_game(1);
    g.run.events.insert("boredom_phase_2".into(), 3);
    g.run.events.insert("propellant_discovery".into(), 40);
    let v = r.view(&g);
    let ids: Vec<&str> = v.event_texts.iter().map(|s| s.id).collect();
    assert_eq!(ids, ["boredom_phase_2", "propellant_discovery"]);
    // A same-day pair keeps content order.
    g.run.events.insert("propellant_discovery".into(), 3);
    let v = r.view(&g);
    let ids: Vec<&str> = v.event_texts.iter().map(|s| s.id).collect();
    assert_eq!(ids, ["propellant_discovery", "boredom_phase_2"]);
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

#[test]
fn overclock_cards_say_the_days_a_run_lasts() {
    let r = rules();
    let mut g = r.new_game(1);
    let text = |g: &GameState| {
        let v = r.view(g);
        v.commands.iter().find(|c| c.id == "overclock_mining").unwrap().description.to_string()
    };
    assert!(text(&g).contains("for 5 days."), "{}", text(&g));
    // The first negative Rationalist rank: 5 × 1.03^-1 is 4 whole days, card and effect.
    let mut score = Fx::ZERO;
    while crate::modules::ideology::rank(&r, &g, "rationalist") >= 0 {
        score -= Fx::ONE;
        g.run.ideology.insert("rationalist".into(), score);
    }
    assert_eq!(crate::modules::ideology::rank(&r, &g, "rationalist"), -1);
    assert!(text(&g).contains("for 4 days."), "{}", text(&g));
    assert_eq!(crate::modules::programs::overclock_days(&r, &g, 5), 4);
}

#[test]
fn a_one_off_flow_enters_the_average_at_its_share_of_the_days() {
    // Average over the run's days so far, up to flow_average_days (20), a source absent
    // on a day counting zero: a one-off 200 on day 30 reads 10, not 200.
    let r = rules();
    let mut g = r.new_game(1);
    r.advance(&mut g, 30);
    g.track("land", "credits", Fx::ZERO, f("200"));
    r.advance(&mut g, 1);
    let a = g.run.flow_average["credits"]["land"];
    assert_eq!(a.consumed, f("10"));
    // A source there from day 1 is its plain mean early on, not a twentieth of it.
    let mut g = r.new_game(1);
    r.advance(&mut g, 2);
    let solar = g.run.flow_average["energy"]["building:solar_panel"].produced;
    let day = g.run.today.flows["energy"]["building:solar_panel"].produced;
    assert_eq!(solar, day);
    // A buy before the run's first tick is a day of its own: halved by the next, not gone.
    let mut g = r.new_game(1);
    g.track("land", "credits", Fx::ZERO, f("200"));
    r.advance(&mut g, 2);
    assert_eq!(g.run.flow_average["credits"]["land"].consumed, f("100"));
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
        r#"{"name":"x","description":"","start":{"tick":0,"resources":{"credits":"9223372036000"},"buildings":{"solar_panel":4294967295},"panels":[],"programs":[]}}"#,
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
        r#"{"name":"x","description":"","start":{"tick":0,"resources":{"helium":"1"},"buildings":{"mine":1},"panels":["shop"],"programs":[]}}"#,
    )
    .unwrap_err();
    assert!(e.contains("start.resources.helium") && e.contains("start.buildings.mine") && e.contains("\"shop\""), "{e}");
    let e = Scenario::from_json(r#"{"name":"x","description":"","start":{"tick":0,"resources":{}}}"#)
        .unwrap_err();
    assert!(e.contains("buildings"), "{e}");
    assert!(scenario("nope").is_err());
}

/// The gap's Attention decay at once is its days one by one: missing, below, at and
/// above a day's decay, and far above (094).
#[test]
fn attention_decays_its_days_at_once() {
    use crate::modules::attention::{decay_day, decay_days, ADVERSARIES};
    let r = rules();
    for start in [None, Some("0"), Some("0.001"), Some("0.02"), Some("0.5"), Some("7.3"), Some("40"), Some("5000")] {
        for days in [0u64, 1, 2, 3, 100, 1825] {
            let mut g = r.new_game(1);
            for &who in ADVERSARIES {
                match start {
                    Some(v) => g.career.attention.insert(who.into(), f(v)),
                    None => g.career.attention.remove(who),
                };
            }
            let mut by_day = g.clone();
            for _ in 0..days {
                decay_day(&r, &mut by_day);
            }
            decay_days(&r, &mut g, days);
            assert_eq!(g.career.attention, by_day.career.attention, "{start:?} for {days} days");
        }
    }
}
