//! Fixed play policies shared by the golden game, `hh-sim` and the wasm
//! parity check (which mirrors [`Policy::Greedy`] in JavaScript). Agents
//! proper belong in `hh-sim`.

use crate::record::Session;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    /// Never acts.
    Idle,
    /// Every `every` ticks, buys one of each building it can afford, in
    /// content order (the view's `affordable` flag, re-read after each buy).
    Greedy { every: u64 },
}

impl Policy {
    pub fn parse(s: &str) -> Result<Policy, String> {
        match s.split_once(':') {
            None if s == "idle" => Ok(Policy::Idle),
            None if s == "greedy" => Ok(Policy::Greedy { every: 1 }),
            Some(("greedy", rest)) => {
                let every = rest
                    .strip_prefix("every=")
                    .and_then(|n| n.parse().ok())
                    .filter(|&n| n > 0)
                    .ok_or_else(|| format!("bad policy {s:?}: expected greedy:every=N, N >= 1"))?;
                Ok(Policy::Greedy { every })
            }
            _ => Err(format!("unknown policy {s:?} (idle, greedy, greedy:every=N)")),
        }
    }
}

/// Plays `ticks` more ticks of `s` under `policy`.
pub fn play(s: &mut Session, ticks: u64, policy: Policy) {
    match policy {
        Policy::Idle => s.advance(ticks),
        Policy::Greedy { every } => {
            let end = s.state.tick + ticks;
            while s.state.tick < end {
                s.advance(every.min(end - s.state.tick));
                let count = s.rules.view(&s.state).buildings.len();
                for i in 0..count {
                    // Re-read: an earlier buy this round may have spent what this one needs.
                    let b = s.rules.view(&s.state).buildings.swap_remove(i);
                    if b.affordable {
                        s.apply(&b.buy).expect("the view said affordable");
                    }
                }
            }
        }
    }
}

/// The golden game: baseline config, seed `u64::MAX`, 2000 ticks of
/// `greedy:every=7` (README, *Rules changes*).
pub const GOLDEN_SEED: u64 = u64::MAX;
pub const GOLDEN_TICKS: u64 = 2000;
pub const GOLDEN_POLICY: Policy = Policy::Greedy { every: 7 };
