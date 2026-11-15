//! Plays one career of an agent to a tick and prints the state's economy
//! (resources against caps, buildings with their next price, programs,
//! markets) and the career's long goals (quests and sub-objectives, ideology
//! bests, projects, achievements): a look at what walls a run in
//! (`cargo run --release -p hh-sim --example probe -- AGENT SEED TICKS`).

use hh_core::{Config, Session};
use hh_sim::agents::{self, Player};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let agent = args.get(1).map_or("author", |s| s.as_str());
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let ticks: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1200);
    let text = std::fs::read_to_string("configs/baseline.json").expect("baseline config");
    let config = Config::from_json(&text).expect("config loads");
    let mut s = Session::new(&config, seed, None).expect("session");
    let mut a = agents::build(agent, seed).expect("agent");
    let mut run = 1;
    while s.state.tick < ticks {
        let gap = {
            let mut p = Player::new(&mut s);
            if p.view().run != run {
                run = p.view().run;
                a.new_run(&mut p);
            }
            a.act(&mut p);
            a.next_look(&mut p).max(1)
        };
        for _ in 0..gap {
            s.advance(1);
            if s.state.tick >= ticks {
                break;
            }
        }
    }
    let v = s.rules.view(&s.state);
    println!("tick {} run {} day {} processors {} land {}", v.tick, v.run, v.day, v.processors, agents::amount(&v, "land"));
    for r in &v.resources {
        println!("  {:12} {:>10} cap {:>10} per_day {:>8}", r.id, r.amount.to_string(), r.cap.map_or("-".into(), |c| c.to_string()), r.per_day.to_string());
    }
    for b in v.buildings.iter().filter(|b| b.unlocked || b.owned > 0) {
        let cost: Vec<String> = b.cost.iter().map(|(k, x)| format!("{k} {}", x.floor_int())).collect();
        println!("  {:20} owned {:>2} active {:>2} land {} next: {} short {:?}", b.id, b.owned, b.active, b.land, cost.join(", "), b.short);
    }
    for p in &v.programs {
        if !p.entries.is_empty() {
            let lines: Vec<String> = p.entries.iter().map(|e| format!("{} x{}", e.command, e.count)).collect();
            println!("  program procs {}: {}", p.processors, lines.join(", "));
        }
    }
    for m in &v.markets {
        println!("  market {:10} demand {:?} tier {}", m.good, m.demand.map(|d| d.to_string()), m.tier);
    }
    println!("  {}", a.note());
    let c = &s.state.career;
    println!("career: retirements {} best run days {} best run credits {}", c.retirements, c.best_run_days, c.best_run_credits);
    println!("  quests: {:?}", c.completed_quests);
    println!("  sub-objectives: {:?}", c.completed_sub_objectives);
    println!("  best ranks: {:?} best scores: {:?}", c.max_ideology_ranks, c.max_ideology_scores);
    println!("  projects done: {:?} progress: {:?}", c.completed_projects, c.project_progress);
    println!("  achievements: {:?}", c.achievements);
}
