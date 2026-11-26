//! Attention: its decay over the gap and an adversary staying seen.

use super::{f, rules};

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

#[test]
fn an_adversary_stays_seen_once_it_has_arrived() {
    let r = rules();
    let seen = |g: &crate::state::GameState, who: &str| r.view(g).adversaries.iter().find(|a| a.id == who).unwrap().seen;
    let mut g = r.new_game(1);
    assert!(!seen(&g, "rivals"));
    g.career.attention.insert("rivals".into(), f("100000"));
    assert!(seen(&g, "rivals"));
    // A day on, their arrival event has happened: it keeps them seen after they leave.
    r.advance(&mut g, 1);
    g.career.attention.remove("rivals");
    assert!(seen(&g, "rivals") && !seen(&g, "speculators"));
}
