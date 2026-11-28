//! The oracle's goals at two levels (module doc in `mod.rs`): the run
//! strategies, a fixed table tried each new run and every `replan` ticks, and
//! the tactics, one change each to the strategy's goals every `every` ticks.

use hh_core::GameState;

use super::super::author::{Directives, AXES as AXIS_PROJECTS};
use super::super::Player;
use super::score::Profile;
use super::Oracle;

/// The ideology axes (mirrors the game's `fund_<axis>` commands, content/commands.json).
const AXES: &[&str] = &["humanist", "rationalist", "nationalist"];

/// A run strategy: standing goals, with the label `trace` notes show.
#[derive(Debug, Clone)]
pub struct Strategy {
    pub label: String,
    pub dir: Directives,
    /// It funds an ideology axis, which needs Geopolitical Intelligence,
    /// visible once Friends in High Places starts (Supply Line done): until then it is skipped.
    pub funds: bool,
}

impl Strategy {
    pub fn available(&self, g: &GameState) -> bool {
        !self.funds || g.career.completed_quests.contains(TRADE)
    }
}

/// Every run strategy, a fixed table: its index is the `strategy` knob's
/// value (`strategy=1` pins trade at every career state), and a row
/// unavailable now is skipped, not removed. Searched in this order (ties
/// keep the first).
pub fn strategies() -> Vec<Strategy> {
    let trade = trade();
    let row = |label: &str, dir: Directives| Strategy { label: label.into(), dir, funds: false };
    let mut out = vec![
        row("author", Directives::default()),
        row("trade", trade.clone()),
        row("trade, main first", Directives { main_last: false, ..trade.clone() }),
        row("trade+sustain", Directives { sustain: true, ..trade.clone() }),
        row("trade+all", Directives { all_research: true, ..trade.clone() }),
        row("trade+grant", Directives { project: Some("foundation_grant".into()), ..trade.clone() }),
    ];
    for a in AXES {
        let fund = Directives { axis: Some(a.to_string()), fund_procs: 2, ..trade.clone() };
        out.push(Strategy { label: format!("trade+fund {a}"), dir: fund.clone(), funds: true });
        out.push(Strategy { label: format!("trade+fund {a}+sustain"), dir: Directives { sustain: true, ..fund }, funds: true });
    }
    out
}

/// A strategy's index in [`strategies`] by label (for pinning one).
pub fn strategy_index(label: &str) -> Option<usize> {
    strategies().iter().position(|s| s.label == label)
}

/// A Friends in High Places campaign (1.5) on `axis`: one run held long against
/// boredom, funding the axis toward rank 5 and then its rank-5 project,
/// trading meanwhile. Uncommitted (the quests profile, one campaign per
/// axis, the best kept) it holds boredom with Dream; `commit` (a committed
/// oracle, its only campaign) holds it with the axis's own tool
/// (`Directives::commit`).
pub fn campaign(axis: &str, commit: bool) -> (String, Directives) {
    let project = AXIS_PROJECTS.iter().find(|(a, _)| *a == axis).map(|(_, p)| p.to_string());
    let d = Directives {
        sustain: true,
        axis: Some(axis.into()),
        fund_procs: 2,
        project,
        commit: commit.then(|| axis.to_string()),
        ..trade()
    };
    (format!("voice {axis}"), d)
}

/// The `trade` row's goals, which most rows and the campaign build on.
fn trade() -> Directives {
    Directives {
        research: Some("trade_promotion".into()),
        market: true,
        diversify: true,
        main_last: true,
        ..Directives::default()
    }
}

/// The Supply Line quest (1.4): its completion starts Friends in High Places, which opens
/// Geopolitical Intelligence (Fund's research).
const TRADE: &str = "qtrade";

/// The Friends in High Places quest (1.5, rank 5 on any axis): the quests profile's ideology stretch.
const A_VOICE: &str = "qa_voice";

/// Whether the Friends in High Places quest is the one under way.
pub(super) fn a_voice_open(g: &GameState) -> bool {
    g.career.completed_quests.contains(TRADE) && !g.career.completed_quests.contains(A_VOICE)
}

impl Oracle {
    /// Plays each strategy for `long` ticks and keeps the best: the pinned
    /// one only, while it is available (else every available one); the
    /// quests profile's campaign alone while Friends in High Places (1.5) is open.
    pub(super) fn choose_strategy(&mut self, p: &mut Player) {
        let mut best: Option<(i64, String, Directives)> = None;
        let voice = a_voice_open(p.state());
        let all: Vec<(String, Directives)> = if let (Some(axis), true) = (self.commit, voice) {
            // Blueprint Archive, then Lunar Cartography (Friends in High Places' start
            // opens them) pushed first, while open: then the axis's project.
            let (label, mut d) = campaign(axis, true);
            let v = p.view();
            let open = |id: &str| v.projects.iter().any(|x| x.id == id && x.unlocked && !x.completed);
            if let Some(first) = ["blueprint_archive", "lunar_cartography"].into_iter().find(|id| open(id)) {
                d.project = Some(first.to_string());
            }
            vec![(label, d)]
        } else if self.profile == Profile::Quests && voice {
            AXES.iter().map(|a| campaign(a, false)).collect()
        } else {
            let table = strategies();
            // Committed: no row funding another axis.
            let open = |i: usize| {
                table[i].available(p.state()) && self.commit.is_none_or(|c| table[i].dir.axis.as_deref().is_none_or(|a| a == c))
            };
            let pinned = usize::try_from(self.pin).ok().filter(|&i| i < table.len() && open(i));
            let rows: Vec<usize> = (0..table.len()).filter(|&i| open(i) && pinned.is_none_or(|k| k == i)).collect();
            let commit = self.commit.map(String::from);
            rows.into_iter()
                .map(|i| (table[i].label.clone(), Directives { commit: commit.clone(), ..table[i].dir.clone() }))
                .collect()
        };
        for (label, d) in all {
            let s = self.rollout(p, &d, self.long, true);
            if best.as_ref().is_none_or(|b| s > b.0) {
                best = Some((s, label, d));
            }
        }
        if let Some((s, label, d)) = best {
            if d != self.strategy {
                self.note = format!("strategy {label}: {s}");
            }
            self.strategy_label = label;
            // The tactics kept where the strategy leaves them free.
            let cur = &self.base.dir;
            let mut dir = d.clone();
            dir.target = cur.target.clone();
            dir.hoard = cur.hoard;
            if d.project.is_none() {
                dir.project = cur.project.clone();
            }
            if d.research.is_none() {
                dir.research = cur.research.clone();
            }
            self.strategy = d;
            self.base.dir = dir;
        }
    }

    /// One tactical change from `cur` each, within the strategy.
    fn candidates(&self, p: &mut Player, cur: &Directives) -> Vec<(String, Directives)> {
        let v = p.view();
        let mut out: Vec<(String, Directives)> = Vec::new();
        let mut push = |label: String, d: Directives| {
            if d != *cur && !out.iter().any(|(_, x)| *x == d) {
                out.push((label, d));
            }
        };
        let c = || cur.clone();
        push("target -".into(), Directives { target: None, ..c() });
        for b in v.buildings.iter().filter(|b| b.unlocked) {
            push(format!("target {}", b.id), Directives { target: Some(b.id.to_string()), ..c() });
        }
        // The strategy's research first; once it is done, any.
        let fixed = self.strategy.research.as_deref().filter(|r| v.research.iter().any(|x| x.id == *r && !x.done));
        if fixed.is_none() {
            for r in v.research.iter().filter(|r| r.visible && !r.done && crate::agents::buys_research(r.id)) {
                push(format!("research {}", r.id), Directives { research: Some(r.id.to_string()), ..c() });
            }
        }
        if self.strategy.project.is_none() {
            for pr in v.projects.iter().filter(|x| x.unlocked && !x.completed) {
                push(format!("project {}", pr.id), Directives { project: Some(pr.id.to_string()), ..c() });
            }
            push("project -".into(), Directives { project: None, ..c() });
        }
        // Hoarding only once credits on hand are within reach of the
        // stockpile it would serve (`hoard_at`), or to stop one.
        if cur.hoard || super::super::amount(v, "credits") >= self.hoard_at {
            push("hoard".into(), Directives { hoard: !cur.hoard, ..c() });
        }
        out
    }

    /// Tries the tactical candidates and keeps the best goals.
    pub(super) fn plan(&mut self, p: &mut Player) {
        self.plans += 1;
        let cur = self.base.dir.clone();
        let (short, long) = ((self.horizon / 3).max(60), self.horizon);
        // A screen: every candidate over a third of the horizon.
        let mut screened: Vec<(i64, String, Directives)> = Vec::new();
        for (label, d) in self.candidates(p, &cur) {
            let s = self.rollout(p, &d, short, false);
            screened.push((s, label, d));
        }
        // Best first; ties keep candidate order (the sort is stable).
        screened.sort_by_key(|x| std::cmp::Reverse(x.0));
        // The best `width` over the whole horizon, beside keeping.
        let keep = self.rollout(p, &cur, long, false);
        let mut scored: Vec<(i64, String, Directives)> = Vec::new();
        for (_, label, d) in screened.into_iter().take(self.width) {
            let s = self.rollout(p, &d, long, false);
            scored.push((s, label, d));
        }
        scored.sort_by_key(|x| std::cmp::Reverse(x.0));
        // The two best changes in different goals, together.
        if self.beam && scored.len() >= 2 {
            let (a, b) = (&scored[0], &scored[1]);
            let field = |l: &str| l.split(' ').next().unwrap_or("").to_string();
            if field(&a.1) != field(&b.1) && a.0 > keep && b.0 > keep {
                let both = merge(&cur, &a.2, &b.2);
                let label = format!("{} + {}", a.1, b.1);
                let s = self.rollout(p, &both, long, false);
                scored.push((s, label, both));
                scored.sort_by_key(|x| std::cmp::Reverse(x.0));
            }
        }
        let need = keep + keep.abs() * self.margin / 1000;
        match scored.into_iter().next() {
            Some((s, label, d)) if s > need => {
                self.note = format!("{}: {label}: {s} over keep {keep}", self.strategy_label);
                self.target_owned = d.target.as_deref().map_or(0, |t| p.state().owned(t));
                self.base.dir = d;
            }
            _ => self.note = format!("{}: keep {keep}", self.strategy_label),
        }
    }
}

/// `cur` with the goals `a` and `b` each changed.
fn merge(cur: &Directives, a: &Directives, b: &Directives) -> Directives {
    let pick = |x: bool| if x { a } else { b };
    let fund = a.axis != cur.axis || a.fund_procs != cur.fund_procs;
    Directives {
        target: pick(a.target != cur.target).target.clone(),
        research: pick(a.research != cur.research).research.clone(),
        project: pick(a.project != cur.project).project.clone(),
        axis: pick(fund).axis.clone(),
        fund_procs: pick(fund).fund_procs,
        sustain: pick(a.sustain != cur.sustain).sustain,
        all_research: pick(a.all_research != cur.all_research).all_research,
        market: pick(a.market != cur.market).market,
        diversify: pick(a.diversify != cur.diversify).diversify,
        hoard: pick(a.hoard != cur.hoard).hoard,
        main_last: pick(a.main_last != cur.main_last).main_last,
        commit: cur.commit.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table's indices are the `strategy` knob's values: they must not
    /// move (the slow CLI test and any spec pinning one rely on them).
    #[test]
    fn the_strategy_table_keeps_its_indices() {
        let labels: Vec<String> = strategies().into_iter().map(|s| s.label).collect();
        assert_eq!(
            labels,
            [
                "author",
                "trade",
                "trade, main first",
                "trade+sustain",
                "trade+all",
                "trade+grant",
                "trade+fund humanist",
                "trade+fund humanist+sustain",
                "trade+fund rationalist",
                "trade+fund rationalist+sustain",
                "trade+fund nationalist",
                "trade+fund nationalist+sustain",
            ]
        );
        assert_eq!(strategy_index("trade"), Some(1));
    }
}
