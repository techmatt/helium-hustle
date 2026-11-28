//! The oracle's and committed players' goals on top of the author's own
//! choices ([`Directives`]): one method per decision they change, asked by
//! `author.rs` (the author's play) and set by the oracle (`oracle/`) and the
//! `commit` knob. With none set each answers as the plain author plays.
//! A committed-player fix may also touch the author's `projects()` (the
//! side-by-side [`PARALLEL`] funding), `programs()` (the tool's and Fund's
//! seats) and `build()` (the bank's purchase gate).

use std::collections::BTreeMap;

use hh_core::{Fx, View};

use super::{DREAM, MAIN, MARKET, PROJECTS, RESEARCH};
use crate::agents::{amount, cap, command_cost, command_unlocked};

/// Goals a planner sets on top of the author's own choices (the oracle,
/// docs/agents.md). Unset, the author plays as itself.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Directives {
    /// The building to save for, in place of the best-scored one.
    pub target: Option<String>,
    /// Research to buy first, saving science for it.
    pub research: Option<String>,
    /// The project to fund, in place of the first open one in order; funded
    /// with everything to spare (credits past the target's price, all science).
    pub project: Option<String>,
    /// The axis to fund, and processors on it (0: the author's own rule).
    pub axis: Option<String>,
    pub fund_procs: u32,
    /// Hold boredom down with Dream (enough processors to outpace growth,
    /// from a third of the cap), and never retire for a stale run.
    pub sustain: bool,
    /// Meant to buy all visible research, not only [`RESEARCH`], but a no-op
    /// today: `next_research` and `science_wanted` look only at the directed
    /// id and [`RESEARCH`], so it plays as the plain author (kept for the
    /// oracle's strategy indices and the autopilot's second driver; whether
    /// to make it work is open, docs/agents.md).
    pub all_research: bool,
    /// Run the market program (Promote what ships, Disrupt) on one processor
    /// whenever two are free, not only three.
    pub market: bool,
    /// Extra pads each carry a different good with a maker running, dearest
    /// first, not only a good at its cap.
    pub diversify: bool,
    /// Buy no buildings and fund no project: let credits pile up.
    pub hoard: bool,
    /// The main program runs last (programs run in order), so Dream, Fund
    /// and the market get energy before Cloud Compute's filler takes it.
    pub main_last: bool,
    /// Committed to one ideology axis (130): its boredom tool is the only
    /// one used (Dream for Humanist, Theorize for Rationalist, held as
    /// `sustain` holds Dream; Recreation Domes for Nationalist, bought from
    /// half the boredom cap), its tool's research comes first, its tagged
    /// buildings score ×3/2, and while 1.5 A Voice is under way no run
    /// retires for being stale (boredom ends it: `retires_stale`).
    /// Funding itself is `axis` (the committed players set both).
    pub commit: Option<String>,
}

/// Quest 1.4 Trade, whose completion starts 1.5 A Voice (rank 5 on any
/// axis this run), content/events.json.
const TRADE_QUEST: &str = "qtrade";
const VOICE_QUEST: &str = "qa_voice";
/// Quest 1.3 Full Load: its completion starts 1.4 Trade.
const FULL_LOAD_QUEST: &str = "qfull_load";
/// The projects A Voice's start opens, banked for and funded side by side at
/// the cap by a committed player (131).
pub(crate) const PARALLEL: [&str; 2] = ["blueprint_archive", "lunar_cartography"];
/// Research Labs a committed player owns before the projects open (strategy).
pub(super) const BANK_LABS: u32 = 2;

/// Each decision the directives change, one method each: the author asks
/// these, and with none set each answers as the plain author plays.
impl Directives {
    /// The command holding boredom down: the committed axis's tool (none
    /// for Nationalist, whose tool is a building), else Dream with `sustain`.
    pub fn tool_command(&self) -> Option<&'static str> {
        match self.commit.as_deref() {
            Some("humanist") => Some("dream"),
            Some("rationalist") => Some("theorize"),
            Some(_) => None,
            None => self.sustain.then_some("dream"),
        }
    }

    /// The research that opens the committed axis's tool.
    pub(super) fn tool_research(&self) -> Option<&'static str> {
        match self.commit.as_deref() {
            Some("humanist") => Some("dream_protocols"),
            Some("rationalist") => Some("thought_experiments"),
            Some("nationalist") => Some("recreation_dome_blueprints"),
            _ => None,
        }
    }

    /// Committed, the research opening Fund (shown once A Voice starts),
    /// after only the Blueprints that open the base's science (before them
    /// it waited all run 2 on science it couldn't make).
    pub(super) fn fund_research(&self) -> &'static [&'static str] {
        if self.commit.is_some() {
            &["basic_blueprints", "advanced_blueprints", "geopolitical_intelligence"]
        } else {
            &[]
        }
    }

    /// The builder's ranking with the target, while unlocked, first; for a
    /// committed player its tagged buildings scored up, and a Recreation
    /// Dome first from half the boredom cap (Nationalist).
    pub(super) fn rank_target(&self, v: &View, ranked: &mut Vec<(String, i64)>) {
        if let Some(axis) = self.commit.as_deref() {
            let tagged = |id: &str| v.buildings.iter().any(|b| b.id == id && b.ideology == Some(axis));
            for (id, s) in ranked.iter_mut() {
                if tagged(id) {
                    *s = s.saturating_mul(3) / 2;
                }
            }
            ranked.sort_by_key(|x| std::cmp::Reverse(x.1));
            let dome = "recreation_dome";
            let half = v.boredom.amount.mul_int(2) >= v.boredom.cap;
            if axis == "nationalist" && half && v.buildings.iter().any(|b| b.id == dome && b.unlocked) {
                ranked.retain(|(id, _)| id != dome);
                ranked.insert(0, (dome.to_string(), i64::MAX));
            }
            // Banking for the projects: science made before they open (Fund's
            // research and their science; a 131 trace had no lab until run-1 day 1,162).
            let lab = "research_lab";
            let labs = v.buildings.iter().find(|b| b.id == lab && b.unlocked).map(|b| b.owned);
            if self.banking(v) && labs.is_some_and(|n| n < BANK_LABS) {
                ranked.retain(|(id, _)| id != lab);
                ranked.insert(0, (lab.to_string(), i64::MAX));
            }
        }
        if let Some(t) = &self.target {
            if v.buildings.iter().any(|b| b.id == t && b.unlocked) {
                ranked.retain(|(id, _)| id != t);
                ranked.insert(0, (t.clone(), i64::MAX));
            }
        }
    }

    /// Research worth its science whatever else is open: all of it, or the
    /// directed one.
    pub(super) fn wants_research(&self, id: &str) -> bool {
        self.all_research || self.research.as_deref() == Some(id) || self.tool_research() == Some(id) || self.fund_research().contains(&id)
    }

    /// Research in the order wanted: committed, Fund's; the directed one,
    /// the committed tool's, then [`RESEARCH`].
    pub(super) fn research_order(&self) -> impl Iterator<Item = &str> {
        self.fund_research()
            .iter()
            .copied()
            .chain(self.research.as_deref())
            .chain(self.tool_research())
            .chain(RESEARCH.iter().copied())
    }

    /// Projects in the order funded: the pushed one, then [`PROJECTS`].
    pub(super) fn project_order(&self) -> impl Iterator<Item = &str> {
        self.project.as_deref().into_iter().chain(PROJECTS.iter().copied())
    }

    /// Whether project `id` is pushed (funded with everything to spare).
    pub(super) fn pushes(&self, id: &str) -> bool {
        self.project.as_deref() == Some(id)
    }

    /// The program slots of the main program and Dream, of `len`: main last
    /// with `main_last`, Dream first while holding boredom, else in order.
    pub(super) fn program_slots(&self, len: usize) -> (usize, usize) {
        let last = len.saturating_sub(1);
        if self.main_last && last > MARKET {
            (last, DREAM)
        } else if self.tool_command().is_some() {
            (DREAM, MAIN)
        } else {
            (MAIN, DREAM)
        }
    }

    /// The axis to fund and its processors, when directed (else the
    /// author's own rule).
    pub(super) fn funded_axis(&self) -> Option<(&str, u32)> {
        self.axis.as_deref().filter(|_| self.fund_procs > 0).map(|a| (a, self.fund_procs))
    }

    /// Whether the market program runs with `left` processors free: the
    /// author's own rule (`own`, from 3 free), or directed from 2.
    pub(super) fn market_runs(&self, own: bool, left: u32) -> bool {
        (own && left > 2) || (self.market && left > 1)
    }

    /// The builder's energy reserve: holding boredom down, room for two
    /// Dreams a day once Dream is unlocked (before, the room starved the run).
    pub(super) fn energy_reserve(&self, own: Fx, v: &View) -> Fx {
        match self.tool_command().filter(|c| command_unlocked(v, c)) {
            Some(c) => own + command_cost(v, c, "energy").mul_int(2),
            None => own,
        }
    }

    /// Committed, what it banks while 1.4 Trade is under way (A Voice's start
    /// opens Blueprint Archive and Lunar Cartography): their credits and
    /// regolith not yet paid, so both fund at the cap from the day they open
    /// (131). From the view's project costs (both listed before they open).
    pub(super) fn banking(&self, v: &View) -> bool {
        let done = |q: &str| v.completed_quests.contains(&q);
        self.commit.is_some() && done(FULL_LOAD_QUEST) && !done(TRADE_QUEST)
    }

    pub(super) fn bank(&self, v: &View) -> Vec<(String, Fx)> {
        if !self.banking(v) {
            return Vec::new();
        }
        let mut bank: BTreeMap<String, Fx> = BTreeMap::new();
        for p in v.projects.iter().filter(|p| PARALLEL.contains(&p.id) && !p.completed) {
            for (res, &c) in p.costs.iter().filter(|(r, _)| *r == "credits" || *r == "regolith") {
                let paid = p.progress.get(res).copied().unwrap_or(Fx::ZERO);
                *bank.entry(res.clone()).or_default() += (c - paid).max(Fx::ZERO);
            }
        }
        bank.into_iter().collect()
    }

    /// Whether buying at `cost` keeps the bank ([`Directives::bank`]): every
    /// banked resource on hand past its bank after the purchase, so nothing is
    /// bought until the bank is full (a strict save; the bank holds at most a
    /// resource's cap, regolith's starting at 50).
    pub(super) fn keeps_bank(&self, v: &View, cost: &BTreeMap<String, Fx>) -> bool {
        self.bank(v).iter().all(|(res, b)| {
            let c = cost.get(res).copied().unwrap_or(Fx::ZERO);
            let b = cap(v, res).map_or(*b, |cap| (*b).min(cap));
            amount(v, res) - c >= b
        })
    }

    /// Whether a run that has bought nothing for `stale_days` retires: not
    /// while holding boredom down; committed, not while 1.5 A Voice (rank 5
    /// this run) is under way: each such run plays out until boredom ends it.
    pub fn retires_stale(&self, v: &View) -> bool {
        if self.commit.is_some() {
            let done = |q: &str| v.completed_quests.contains(&q);
            return !(done(TRADE_QUEST) && !done(VOICE_QUEST));
        }
        !self.sustain
    }
}
