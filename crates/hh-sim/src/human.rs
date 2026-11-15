//! Human time models (`configs/sim/humans.json`): how a person's attention and
//! speed would map a sim player's ticks to real minutes. A projection only:
//! sim players play in game time ([`crate::agents::Agent::next_look`]), and
//! no model changes a game. A human's own play log has its real clock instead.
//!
//! A model reads a career's looks: each decision (a look that acted) takes
//! `decide_s` real seconds; the gap to the next look plays its first
//! `burst_after` ticks at `play` ticks a second (watching) and the rest at
//! `burst` (a brief burst through a stretch that needs nothing).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Human {
    /// Ticks a real second while deciding and watching.
    pub play: u64,
    /// Ticks a real second through a long stretch.
    pub burst: u64,
    /// Ticks of a gap played at `play` before the burst.
    pub burst_after: u64,
    /// Real seconds a decision takes.
    pub decide_s: f64,
    pub says: String,
}

impl Human {
    fn check(&self, name: &str) -> Result<(), String> {
        if !(1..=200).contains(&self.play) || !(1..=200).contains(&self.burst) {
            return Err(format!("human {name}: play and burst must be 1 to 200 ticks a second"));
        }
        if !(self.decide_s >= 0.0 && self.decide_s.is_finite()) {
            return Err(format!("human {name}: decide_s must be a number of seconds, 0 or more"));
        }
        Ok(())
    }

    /// Real seconds for a gap of `ticks`.
    pub fn gap_s(&self, ticks: u64) -> f64 {
        let watched = ticks.min(self.burst_after);
        watched as f64 / self.play as f64 + (ticks - watched) as f64 / self.burst as f64
    }

    /// The clock this model gives a career: `looks` are (tick, actions
    /// applied), in tick order, from `start` to `end`.
    pub fn clock(&self, start: u64, end: u64, looks: &[(u64, u32)]) -> Clock {
        let mut points = vec![(start, 0.0)];
        let mut s = 0.0;
        let mut at = start;
        // A gap: watched ticks, then the burst (a point at the knee, so a tick
        // inside the gap reads on the right slope).
        let gap = |points: &mut Vec<(u64, f64)>, s: &mut f64, from: u64, to: u64| {
            if to - from > self.burst_after && self.burst_after > 0 {
                points.push((from + self.burst_after, *s + self.gap_s(self.burst_after)));
            }
            *s += self.gap_s(to - from);
            points.push((to, *s));
        };
        for &(tick, actions) in looks {
            if tick > at {
                gap(&mut points, &mut s, at, tick);
                at = tick;
            }
            if actions > 0 {
                // Deciding happens at the look's tick, before play resumes.
                s += self.decide_s;
                points.push((at, s));
            }
        }
        if end > at {
            gap(&mut points, &mut s, at, end);
        }
        Clock { points }
    }
}

/// Real seconds at each career tick: points in tick order, read between them
/// in proportion. At a tick with several points, the last counts (what was
/// decided at a look happened before the tick's milestones were read).
#[derive(Debug, Clone, Default)]
pub struct Clock {
    pub points: Vec<(u64, f64)>,
}

impl Clock {
    pub fn at(&self, tick: u64) -> f64 {
        let p = &self.points;
        let Some(first) = p.first() else { return 0.0 };
        if tick <= first.0 {
            // The last point at the first tick.
            return p.iter().take_while(|x| x.0 == first.0).last().map_or(0.0, |x| x.1);
        }
        // The first point past `tick`.
        let i = p.partition_point(|x| x.0 <= tick);
        if i == p.len() {
            return p[i - 1].1;
        }
        let (a, b) = (p[i - 1], p[i]);
        if a.0 == tick {
            return a.1;
        }
        a.1 + (b.1 - a.1) * (tick - a.0) as f64 / (b.0 - a.0) as f64
    }

    /// Real seconds on reaching `tick`, before anything decided there: where
    /// a run that starts or ends at `tick` is cut (a decision at a run's
    /// first tick is that run's).
    pub fn before(&self, tick: u64) -> f64 {
        let i = self.points.partition_point(|x| x.0 < tick);
        match self.points.get(i) {
            Some(p) if p.0 == tick => p.1,
            _ => self.at(tick),
        }
    }

    /// Real seconds between reaching `from` and reaching `to`.
    pub fn span(&self, from: u64, to: u64) -> f64 {
        self.before(to) - self.before(from)
    }

    pub fn total(&self) -> f64 {
        self.points.last().map_or(0.0, |x| x.1)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Humans {
    #[serde(rename = "_doc")]
    pub doc: String,
    pub humans: BTreeMap<String, Human>,
}

impl Humans {
    pub fn from_json(text: &str) -> Result<Humans, String> {
        let h: Humans = serde_json::from_str(text).map_err(|e| format!("humans: {e}"))?;
        for (name, m) in &h.humans {
            m.check(name)?;
        }
        Ok(h)
    }

    pub fn get(&self, name: &str) -> Result<&Human, String> {
        self.humans.get(name).ok_or_else(|| {
            format!("unknown human model {name:?} (known: {})", self.humans.keys().cloned().collect::<Vec<_>>().join(", "))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_model_maps_looks_to_seconds() {
        let h = Human { play: 2, burst: 10, burst_after: 10, decide_s: 3.0, says: String::new() };
        // A decision at 0, a quiet look at 10, a decision at 40, the end at 50.
        let c = h.clock(0, 50, &[(0, 2), (10, 0), (40, 1)]);
        // 3 + 10/2 = 8 at tick 10; + 10/2 + 20/10 + 3 = 18 at 40; + 10/2 = 23 at 50.
        assert_eq!(c.at(0), 3.0);
        assert_eq!(c.at(10), 8.0);
        assert_eq!(c.at(40), 18.0);
        assert_eq!(c.total(), 23.0);
        assert!((c.at(45) - 20.5).abs() < 1e-9);
        assert_eq!(c.at(99), 23.0);
        // The decision at the first tick is the first run's.
        assert_eq!(c.before(0), 0.0);
        assert_eq!(c.span(0, 40), 15.0);
        // Inside a long gap: watched at `play`, then the burst.
        let c = h.clock(0, 200, &[(0, 0)]);
        assert_eq!(c.at(10), 5.0);
        assert_eq!(c.at(20), 6.0);
        assert_eq!(c.total(), 5.0 + 19.0);
    }
}
