//! What one view costs (docs/architecture.md, *The view*), split into its state
//! copies, and a digest of every view's JSON over the golden game: a refactor
//! that only reshapes how the view is built keeps the digest.
//! `cargo run --release -p hh-sim --example view_bench` (~10 s).
use std::time::Instant;

use hh_core::{driver, modules::career, play, Config};

fn time<F: FnMut()>(n: u32, mut f: F) -> f64 {
    let t = Instant::now();
    for _ in 0..n {
        f();
    }
    t.elapsed().as_secs_f64() * 1e6 / n as f64
}

fn main() {
    let c = Config::from_json(&std::fs::read_to_string("configs/baseline.json").unwrap()).unwrap();
    // Digest: FNV over the view JSON at every tick of the golden game.
    let mut s = driver::golden_start(&c).unwrap();
    let mut h: u64 = 0xcbf29ce484222325;
    let mut bytes = 0usize;
    for _ in 0..driver::GOLDEN_TICKS {
        let j = serde_json::to_string(&s.rules.view(&s.state)).unwrap();
        bytes += j.len();
        for b in j.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        play(&mut s, 1, driver::GOLDEN_POLICY);
    }
    println!("view digest over the golden game: {h:016x} ({bytes} bytes)");
    for at in [100u64, 800, 1500] {
        let mut s = driver::golden_start(&c).unwrap();
        play(&mut s, at, driver::GOLDEN_POLICY);
        let (r, g) = (&s.rules, &s.state);
        let n = 3000;
        let view = time(n, || {
            std::hint::black_box(r.view(g));
        });
        let clone = time(n, || {
            std::hint::black_box(g.clone());
        });
        let config = time(n, || {
            std::hint::black_box(g.config.clone());
        });
        let peek = time(n, || {
            let mut h = g.clone();
            r.advance(&mut h, 1);
            std::hint::black_box(h);
        });
        let retire = time(n, || {
            let mut h = g.clone();
            career::retire(r, &mut h, false);
            std::hint::black_box(h);
        });
        println!(
            "tick {at:>5}: view {view:6.1} us | state clone {clone:5.1} (config {config:5.1}) | clone+tick {peek:5.1} | clone+retire {retire:5.1}"
        );
    }
}
