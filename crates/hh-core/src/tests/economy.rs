//! The economy: buying and selling, the day's resolution in passes, upkeep, caps,
//! commands and programs running, and the resource breakdown.

use super::support::{buy, own, program};
use super::{baseline, f, rules, set};
use crate::driver::{play, Policy};
use crate::state::{Entry, Stall};
use crate::{scenario, Action, Fx, GameState, Scenario, Session};

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
    // 6 × 1.5² = 13.5 titanium, rounded to whole units, halves up (Matt, 2026-11-27: every
    // building's price, tagged or not; what's shown is what's charged).
    assert_eq!(panel.cost["titanium"], f("14"));
    // 11 titanium can't buy a 14-titanium panel, and a refused buy changes nothing.
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
    // Three Data Centers want 12 energy; with none to spare they run at 0 and grant no
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
    // Half paid (6 of 12): floor(3 × 0.5) = 1 works.
    set(&mut g, "energy", "6");
    r.advance(&mut g, 1);
    assert_eq!((g.run.buildings["data_center"].unpaid, programs_processors(&g)), (2, 1));
    // Switching one off takes an unpaid copy: the paid one keeps its processor.
    r.apply(&mut g, &Action::SetActive { building: "data_center".into(), active: 2 }).unwrap();
    assert_eq!((g.run.buildings["data_center"].unpaid, programs_processors(&g)), (1, 1));
    r.apply(&mut g, &Action::SetActive { building: "data_center".into(), active: 3 }).unwrap();
    set(&mut g, "energy", "12");
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
    r.advance(&mut g, 2);
    let energy_before = g.amount("energy");
    r.advance(&mut g, 1);
    let v = r.view(&g);
    // Load Launch Pads ran on the Solar Panels' power; a building with upkeep went short.
    assert_eq!(v.programs[0].outcomes[0], Some(crate::state::LineOutcome::Ran));
    // Buy Titanium on 10 credits fails each day (no partial purchase) and says why.
    assert_eq!(v.programs[0].outcomes[1], Some(crate::state::LineOutcome::Failed));
    let why = v.programs[0].shorts[1].as_ref().unwrap();
    assert_eq!((why.resource.as_str(), why.need, why.had, why.ran), ("credits", f("20"), f("10"), Fx::ZERO));
    assert!(g.run.pads.iter().any(|p| p.cargo > Fx::ZERO));
    let starved: Vec<_> = v.buildings.iter().filter_map(|b| b.starved.as_ref()).collect();
    assert!(!starved.is_empty() && starved.iter().all(|s| s.ran < Fx::ONE && s.resource == "energy"));
    let energy = v.breakdown.iter().find(|b| b.resource == "energy").unwrap();
    let row = |src: &str| energy.rows.iter().find(|x| x.source == src).unwrap();
    assert_eq!(row("building:solar_panel").produced, f("36"));
    assert_eq!(row("program:0").consumed, f("2"));
    assert_eq!(energy.shorts.len(), starved.len());
    // Every unit made is accounted for: sources less sinks is the change in stock (a short
    // day spends it all, a few 10^-6 left from yesterday too), but for the clamp lifting a
    // rounding dip below zero (a unit or two: each short consumer rounds its partial pay).
    let net: Fx = energy.rows.iter().map(|x| x.produced - x.consumed).sum();
    let off = net - (g.amount("energy") - energy_before);
    assert!(off.max(-off) <= f("0.000002"), "{net} against a change of {}", g.amount("energy") - energy_before);
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
                    // A stock drained to zero by several partly paid consumers can dip
                    // a few rounding units per consumer (seed 6's regolith: 10, rev 38).
                    off.max(-off) <= f("0.00002"),
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
    // Energy starts at its cap; the panel's 6 less the data center's 4 overflows.
    r.advance(&mut g, 1);
    assert_eq!(g.amount("energy"), f("100"));
    assert_eq!(g.run.today.overflow["energy"], f("2"));
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
fn commands_convert_in_full_or_not_at_all_and_never_past_a_cap() {
    let r = rules();
    let mut g = r.new_game(1);
    g.career.flags.insert(crate::modules::boredom::BURNOUT_FLAG.into());
    program(&r, &mut g, &[("cloud_compute", 1)]);
    r.advance(&mut g, 1);
    assert_eq!(g.amount("credits"), f("5"));
    assert_eq!(g.amount("boredom"), f("0.1") + f("0.1"), "phase growth plus the command's");
    // Buy Titanium with half its credits: no partial purchase. It fails, nothing paid or
    // made, and notes what it was short of.
    set(&mut g, "credits", "10");
    program(&r, &mut g, &[("buy_titanium", 1)]);
    r.advance(&mut g, 1);
    assert_eq!(super::outcomes(&g)[0], Some(crate::state::LineOutcome::Failed));
    assert_eq!((g.amount("titanium"), g.amount("credits")), (Fx::ZERO, f("10")));
    let why = r.view(&g).programs[0].shorts[0].clone().unwrap();
    assert_eq!((why.resource.as_str(), why.need, why.had, why.ran), ("credits", f("20"), f("10"), Fx::ZERO));
    // Its full price: one unit.
    set(&mut g, "credits", "20");
    r.advance(&mut g, 1);
    assert_eq!((g.amount("titanium"), g.amount("credits")), (Fx::ONE, Fx::ZERO));
    // A run that would pass the titanium cap (20) skips: nothing paid. At the cap less one
    // it runs, up to the cap exactly.
    let cap = crate::modules::economy::cap(&r, &g, "titanium").unwrap();
    assert_eq!(cap, f("20"));
    set(&mut g, "titanium", "19.5");
    set(&mut g, "credits", "100");
    r.advance(&mut g, 1);
    assert_eq!(super::outcomes(&g)[0], Some(crate::state::LineOutcome::Skipped));
    assert_eq!((g.amount("titanium"), g.amount("credits")), (f("19.5"), f("100")));
    set(&mut g, "titanium", "19");
    r.advance(&mut g, 1);
    assert_eq!((g.amount("titanium"), g.amount("credits")), (cap, f("80")));
    // A repeat count is per execution: 3x from 18 runs twice, then skips.
    set(&mut g, "titanium", "18");
    program(&r, &mut g, &[("buy_titanium", 3)]);
    r.advance(&mut g, 3);
    assert_eq!((g.amount("titanium"), g.amount("credits")), (cap, f("40")));
    assert_eq!(super::outcomes(&g)[0], Some(crate::state::LineOutcome::Skipped));
    // The pass records the skip (the gutter draws it gray): run 3 of 3.
    let rep = r.view(&g).programs[0].repeats[0].unwrap();
    assert_eq!((rep.done, rep.skipped, rep.first_skips, rep.failed), (3, 1, 0b100, 0));
}

#[test]
fn the_cap_overflow_check_counts_a_scaled_output() {
    // Buy Power's output is scaled by the career's peak power: the skip judges the
    // scaled amount (held + output > cap), not the content's 20.
    let r = rules();
    let mut g = r.new_game(1);
    g.career.peak_power = f("100000");
    let c = crate::content::content().command("buy_power").unwrap();
    let (_, produces) = crate::modules::programs::flows(&r, &g, c);
    let out = produces["energy"];
    assert!(out > f("20"), "{out}");
    let cap = crate::modules::economy::cap(&r, &g, "energy").unwrap();
    set(&mut g, "energy", &(cap - out).to_string());
    assert!(!crate::modules::economy::would_overflow(&r, &g, &produces), "lands on the cap: runs");
    set(&mut g, "energy", &(cap - f("20")).to_string());
    assert!(crate::modules::economy::would_overflow(&r, &g, &produces), "room for 20, not the scaled output");
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
    // Idle once more, then Buy Regolith fails again: 2 credits are short of its 10.
    assert_eq!(g.amount("credits"), f("2"));
    assert_eq!(g.amount("regolith"), Fx::ZERO);
    assert!(r.apply(&mut g, &Action::SetProcessors { program: 1, processors: 1 }).is_err());
    // A lost data center takes its processor off the program.
    r.apply(&mut g, &Action::SetActive { building: "data_center".into(), active: 1 }).unwrap();
    assert_eq!(g.run.programs[0].processors, 1);
}

/// A program tab's light (the view's `status`), judged on the day just
/// played: none without processors; red when nothing useful ran; yellow when
/// some steps were short or failed; green when every step ran in full.
#[test]
fn program_tabs_light_by_how_the_last_day_went() {
    use crate::view::Light;
    let r = rules();
    let mut g = r.new_game(1);
    own(&mut g, "data_center", 2);
    own(&mut g, "solar_panel", 2);
    let status = |g: &GameState| r.view(g).programs[0].status.clone();
    assert!(r.view(&g).programs[1].status.is_none(), "no processors, no light");
    r.apply(&mut g, &Action::SetProcessors { program: 0, processors: 2 }).unwrap();
    program(&r, &mut g, &[]);
    let s = status(&g).unwrap();
    assert_eq!((s.light, s.tip.as_str()), (Light::Red, "2 processors · the program is empty"));
    // Idle twice: both steps in full.
    program(&r, &mut g, &[("idle", 1)]);
    r.advance(&mut g, 1);
    let s = status(&g).unwrap();
    assert_eq!((s.light, s.steps, s.full), (Light::Green, 2, 2), "{}", s.tip);
    assert_eq!(s.tip, "2 processors · 2 of 2 steps ran in full");
    // Buy Regolith with no credits fails; idle runs: some short.
    set(&mut g, "credits", "0");
    program(&r, &mut g, &[("buy_regolith", 1), ("idle", 1)]);
    r.advance(&mut g, 1);
    let s = status(&g).unwrap();
    assert_eq!((s.light, s.steps, s.full), (Light::Yellow, 2, 1), "{}", s.tip);
    assert!(s.tip.starts_with("2 processors · 1 of 2 steps ran in full · Buy Regolith: short on credits"), "{}", s.tip);
    // Every step fails: red.
    set(&mut g, "credits", "0");
    program(&r, &mut g, &[("buy_regolith", 1)]);
    r.advance(&mut g, 1);
    let s = status(&g).unwrap();
    assert_eq!((s.light, s.steps, s.full), (Light::Red, 2, 0), "{}", s.tip);
    // A locked line is skipped free: every step ran in full, but yellow, named.
    let load = crate::state::Entry::new(crate::state::Op::Act { verb: crate::state::Verb::LoadPads, target: None, tab: None, amount: None });
    r.apply(&mut g, &Action::SetProgram { program: 0, entries: vec![load, Entry::command("idle", 1)] }).unwrap();
    r.advance(&mut g, 1);
    let s = status(&g).unwrap();
    assert_eq!((s.light, s.steps, s.full), (Light::Yellow, 2, 2), "{}", s.tip);
    assert!(s.tip.ends_with("Load pads all: locked (Load Launch Pads isn't unlocked this run)"), "{}", s.tip);
    // Unlocked, with no pad ready (a new pad carries No cargo): a skip, yellow, named.
    own(&mut g, "launch_pad", 1);
    r.advance(&mut g, 1);
    let s = status(&g).unwrap();
    assert_eq!((s.light, s.steps, s.full), (Light::Yellow, 2, 1), "{}", s.tip);
    assert!(s.tip.ends_with("Load pads all: no pad ready"), "{}", s.tip);
    // A paused Data Center takes its processor off the tab: one processor, green again.
    program(&r, &mut g, &[("idle", 1)]);
    r.apply(&mut g, &Action::SetActive { building: "data_center".into(), active: 1 }).unwrap();
    r.advance(&mut g, 1);
    let s = status(&g).unwrap();
    assert_eq!((s.light, s.tip.as_str()), (Light::Green, "1 processor · 1 of 1 steps ran in full"));
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
        // Power for the Data Centers' upkeep over the 20 days (no brownout idles a line).
        own(&mut g, "solar_panel", 2);
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
