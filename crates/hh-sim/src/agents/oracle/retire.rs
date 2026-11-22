//! When the oracle retires (module doc in `mod.rs`, *Runs*): the prestige
//! rule, checked on a copy of the run played out.

use hh_core::{Action, Fx, GameState, Session};

use super::super::author::Directives;
use super::super::{Agent, Player};
use super::Oracle;

impl Oracle {
    /// Retires once the run's average credits a day is the best it will see
    /// (played out under the current goals), from day `min_run` and not
    /// while holding boredom for the ten years. Returns whether it retired.
    pub(super) fn maybe_retire(&mut self, p: &mut Player) -> bool {
        // Not before `min_run` days: a retirement at once re-rolls the
        // markets for free, which a peeking player could farm, burning runs.
        let can = p.view().can_retire;
        let g = p.state();
        let open = self.retire && can && g.run.day >= self.min_run;
        if !open {
            return false;
        }
        let dir = self.base.dir.clone();
        // The first retirement is a milestone: a profile that scores
        // milestones takes it once the run is within `first_margin`.
        let first = self.w.milestone != 0 && p.state().career.retirements == 0;
        let margin = if first { self.first_margin } else { self.retire_margin };
        let (now, best_later) = self.run_rates(p, &dir);
        if now > Fx::ZERO && best_later.mul_int(1000) <= now.mul_int(1000 + margin) {
            self.note = format!("retire: {now} a day so far, at most {best_later} later");
            let _ = p.apply(&Action::Retire);
            return true;
        }
        false
    }

    /// The run's average credits earned a day so far, and the best that
    /// average reaches later in the run, played out on a copy under goals
    /// `dir` (to the run's end, or `long` ticks).
    fn run_rates(&mut self, p: &Player, dir: &Directives) -> (Fx, Fx) {
        self.rollouts += 1;
        let avg = |g: &GameState| g.run.stats.credits_earned.mul_ratio(1, g.run.day.max(1) as i64);
        let now = avg(p.state());
        let mut s: Session = p.fork();
        let mut a = self.base.clone();
        a.dir = dir.clone();
        a.set_gaps(1, self.roll_gap);
        let run = s.state.career.run_number;
        let end = s.state.tick + self.long;
        let mut best = Fx::ZERO;
        'play: while s.state.tick < end {
            let gap = {
                let mut pl = Player::new(&mut s);
                a.act(&mut pl);
                a.next_look(&mut pl)
            };
            for _ in 0..gap.max(1) {
                if s.state.career.run_number != run || s.state.tick >= end {
                    break 'play;
                }
                s.advance(1);
                if s.state.career.run_number == run {
                    best = best.max(avg(&s.state));
                }
            }
        }
        (now, best)
    }
}
