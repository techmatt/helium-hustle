//! The oracle's rollouts and their score: profiles and their weights, a
//! rollout of the author under given goals on a copy of the game, and what
//! the score counts (module doc in `mod.rs`, *Score*).

use hh_core::modules::boredom;
use hh_core::{content, Fx, GameState, Session};

use super::super::author::Directives;
use super::super::{progress, Agent, Player};
use super::Oracle;
use crate::milestones::{Tracker, When};

/// What a profile values beyond credits, in credits per unit.
#[derive(Debug, Clone, Copy)]
pub struct Weights {
    /// Per point of an axis's career-best score.
    pub ideology: i64,
    /// Per Long-Term project's worth of progress (fractions paid, summed),
    /// and per completion.
    pub project: i64,
    pub project_done: i64,
    /// Per building type, command, quest or sub-objective, achievement first had.
    pub building: i64,
    pub command: i64,
    pub quest: i64,
    pub achievement: i64,
    /// Per research bought this run.
    pub research: i64,
    /// Per open milestone's progress squared at the end (0 to 1, [`progress`]).
    pub progress: i64,
    /// Per milestone met, plus `early` per tick it came before the rollout's end.
    pub milestone: i64,
    pub early: i64,
}

/// The profiles, by their `profile` knob: 0 milestones (the default), 1
/// credits, 2 ideology, 3 builder, 4 quests (the milestones' weights, and
/// while Open Horizons is open, its campaign as the only strategy:
/// [`super::strategy::campaign`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Milestones,
    Credits,
    Ideology,
    Builder,
    Quests,
}

impl Profile {
    /// The profile for knob value `n`; a value outside 0-4 is a spec error.
    pub fn from_knob(n: i64) -> Result<Profile, String> {
        Ok(match n {
            0 => Profile::Milestones,
            1 => Profile::Credits,
            2 => Profile::Ideology,
            3 => Profile::Builder,
            4 => Profile::Quests,
            _ => {
                return Err(format!(
                    "agent oracle: profile={n} is not 0-4 (0 milestones, 1 credits, 2 ideology, 3 builder, 4 quests)"
                ))
            }
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Profile::Milestones => "milestones",
            Profile::Credits => "credits",
            Profile::Ideology => "ideology",
            Profile::Builder => "builder",
            Profile::Quests => "quests",
        }
    }

    pub fn weights(self) -> Weights {
        let w = |ideology, project, project_done, building, command, quest, achievement, research, progress, milestone, early| {
            Weights { ideology, project, project_done, building, command, quest, achievement, research, progress, milestone, early }
        };
        match self {
            Profile::Credits => w(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            Profile::Ideology => w(30, 3000, 3000, 0, 0, 1000, 0, 100, 0, 0, 0),
            Profile::Builder => w(2, 3000, 3000, 3000, 100, 1000, 300, 100, 0, 0, 0),
            // `early` 100 against 20 (dev seeds 1-3, run 1): the same M2 and M3,
            // M7 in run 1 on 2 seeds of 3 against 1.
            Profile::Quests | Profile::Milestones => w(3, 2000, 2000, 300, 200, 2000, 300, 300, 4000, 4000, 100),
        }
    }
}

impl Oracle {
    /// Plays the author with goals `dir` on a copy of the game for `horizon`
    /// ticks and scores it; `through` plays on past a retirement (with the
    /// same goals, the target dropped), else the rollout stops there.
    pub(super) fn rollout(&mut self, p: &Player, dir: &Directives, horizon: u64, through: bool) -> i64 {
        self.rollouts += 1;
        let ms = &self.ms;
        let mut s: Session = p.fork();
        let mut a = self.base.clone();
        a.dir = dir.clone();
        a.set_gaps(1, self.roll_gap);
        let start = s.state.clone();
        let mut run = s.state.career.run_number;
        let t0 = s.state.tick;
        let end = t0 + horizon;
        let half = t0 + horizon / 2;
        let scored = self.w.milestone != 0 || self.w.progress != 0;
        let mut tracker = Tracker::new(ms);
        tracker.after(&s);
        // Each open milestone's first tick met in the rollout.
        let mut hits: Vec<(usize, u64)> = Vec::new();
        let open: Vec<usize> = (0..ms.milestones.len()).filter(|&i| scored && !self.done.contains(&ms.milestones[i].id)).collect();
        let mut sample: Option<(u64, Fx, Fx, u32)> = None;
        'play: while s.state.tick < end {
            let gap = {
                let mut pl = Player::new(&mut s);
                a.act(&mut pl);
                a.next_look(&mut pl)
            };
            for _ in 0..gap.max(1) {
                if s.state.career.run_number != run {
                    if !through {
                        break 'play;
                    }
                    run = s.state.career.run_number;
                    a.dir.target = None;
                    break;
                }
                if s.state.tick >= end {
                    break 'play;
                }
                // The closing income is sampled from the later of the
                // second half and the start of the run under way.
                if s.state.tick >= half && sample.is_none_or(|x| x.3 != run) {
                    sample = Some((s.state.tick, earned(&s.state), s.state.amount("boredom"), run));
                }
                if scored {
                    tracker.before(&s);
                }
                s.advance(1);
                if scored {
                    tracker.after(&s);
                    for &i in &open {
                        if !hits.iter().any(|h| h.0 == i) && tracker.met(&ms.milestones[i].when, &s) {
                            hits.push((i, s.state.tick));
                        }
                    }
                }
            }
        }
        let sample = sample.filter(|x| x.3 == s.state.career.run_number).map(|x| (x.0, x.1, x.2));
        let mut total = self.score(&start, &s, sample);
        for &(_, t) in &hits {
            let early = end.saturating_sub(t) as i64;
            total += self.w.milestone + self.w.early * early;
        }
        if self.w.progress != 0 {
            // Each open milestone not met here, by how far it got (squared).
            let rate = sample.map_or(Fx::ZERO, |(t, e, _)| (earned(&s.state) - e).mul_ratio(1, (s.state.tick - t).max(1) as i64));
            let met = |w: &When| tracker.met(w, &s);
            let seen = progress::Seen { rate, shipped: tracker.shipped(), tracker_met: &met };
            for &i in open.iter().filter(|i| !hits.iter().any(|h| h.0 == **i)) {
                let f = progress::toward(&s.rules, &s.state, &ms.milestones[i].when, &seen);
                total += (f * f).mul_int(self.w.progress).floor_int();
            }
        }
        total
    }

    fn score(&self, a: &GameState, s: &Session, sample: Option<(u64, Fx, Fx)>) -> i64 {
        let w = &self.w;
        let b = &s.state;
        let mut total = (earned(b) - earned(a)).floor_int();
        // The closing income for the days the run under way has left (a
        // sample from before a retirement reads the new run's start).
        if let Some((t, e, bored)) = sample {
            let dt = (b.tick - t).max(1) as i64;
            let rate = (earned(b) - e).mul_ratio(1, dt);
            let growth = (b.amount("boredom") - bored).mul_ratio(1, dt);
            let room = boredom::cap(&s.rules, b) - b.amount("boredom");
            let left = if growth > Fx::ZERO { (room / growth).floor_int().clamp(0, self.tail) } else { self.tail };
            total += rate.mul_int(left).floor_int();
        }
        let ideo = |g: &GameState| g.career.max_ideology_scores.values().fold(Fx::ZERO, |s, x| s + *x);
        total += (ideo(b) - ideo(a)).mul_int(w.ideology).floor_int();
        total += (projects(b) - projects(a)).mul_int(w.project).floor_int();
        let n = |x: usize, y: usize| y.saturating_sub(x) as i64;
        total += w.project_done * n(a.career.completed_projects.len(), b.career.completed_projects.len());
        total += w.building * n(a.career.owned_buildings.len(), b.career.owned_buildings.len());
        total += w.command * n(a.career.used_commands.len(), b.career.used_commands.len());
        let quests = |g: &GameState| g.career.completed_quests.len() + g.career.completed_sub_objectives.len();
        total += w.quest * n(quests(a), quests(b));
        total += w.achievement * n(a.career.achievements.len(), b.career.achievements.len());
        let research = |g: &GameState| g.career.lifetime.research_completed + g.run.stats.research_completed;
        total += w.research * i64::from(research(b).saturating_sub(research(a)));
        total
    }
}

/// Long-Term projects' fractions paid, summed (completed ones count 1).
fn projects(g: &GameState) -> Fx {
    content()
        .projects
        .iter()
        .filter(|p| p.persistent())
        .fold(Fx::ZERO, |s, p| {
            if g.career.completed_projects.contains(&p.id) {
                return s + Fx::ONE;
            }
            let (paid, total) = crate::agents::progress::paid(g, p);
            s + if total > Fx::ZERO { paid / total } else { Fx::ZERO }
        })
}

/// Credits earned in the career so far, the run under way included.
pub(super) fn earned(g: &GameState) -> Fx {
    g.career.lifetime.credits_earned + g.run.stats.credits_earned
}
