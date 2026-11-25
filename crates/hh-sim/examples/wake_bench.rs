//! What one page wake costs the engine, natively, split into the phases the wasm
//! calls go through (`hh-wasm`'s `load`: state JSON in, rules rebuilt, state checked;
//! the work; JSON out), on an early state and a late one (094).
//! `cargo run --release -p hh-sim --example wake_bench` (~10 s).
use std::time::Instant;

use hh_core::{scenario, Config, GameState, Rules};

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn time<F: FnMut()>(n: u32, mut f: F) -> f64 {
    let t = Instant::now();
    for _ in 0..n {
        f();
    }
    t.elapsed().as_secs_f64() * 1e6 / n as f64
}

fn report(label: &str, r: &Rules, g: &GameState) {
    let json = g.to_json();
    let view = serde_json::to_string(&r.view(g)).unwrap();
    let n = 300;
    println!("{label}: state JSON {} KB, view JSON {} KB", json.len() / 1024, view.len() / 1024);
    let row = |what: &str, us: f64| println!("  {what:<34} {us:>9.1} us");
    row("GameState::from_json", time(n, || drop(GameState::from_json(&json).unwrap())));
    row("Rules::for_state", time(n, || drop(Rules::for_state(g).unwrap())));
    row("  Config::check", time(n, || g.config.check().unwrap()));
    row("  Config::hash", time(n, || drop(g.config.hash())));
    row("check_state", time(n, || r.check_state(g).unwrap()));
    row("advance 1 tick", time(n, || r.advance(&mut g.clone(), 1)));
    row("state clone", time(n, || drop(g.clone())));
    row("GameState::to_json", time(n, || drop(g.to_json())));
    row("view build", time(n, || drop(r.view(g))));
    row("view to JSON", {
        let v = r.view(g);
        time(n, || drop(serde_json::to_string(&v).unwrap()))
    });
    row("fingerprint", time(n, || drop(g.fingerprint())));
}

fn main() {
    let c = Config::from_json(&std::fs::read_to_string("configs/baseline.json").unwrap()).unwrap();
    let r = Rules::new(&c).unwrap();
    let mut early = r.start(1, None);
    r.advance(&mut early, 20);
    report("early (new game, day 20)", &r, &early);
    let mut late = r.start(1, Some(scenario("everything").unwrap()));
    r.advance(&mut late, 60);
    report("late (everything, +60 days)", &r, &late);
}
