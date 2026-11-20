//! The two judges over an experiment's results: `compare`'s paired table of
//! side B minus side A, and `balance`'s lines for the judge's cells against
//! the targets. Re-exported from `experiment`.

use std::fmt::Write as _;

use super::{mean, Results, Summary};
use crate::career::GameResult;
use crate::human::Human;
use crate::milestones::Milestones;

// ---- Compare ----

/// One game's value of a paired measure, if it has one.
type Measure<'m> = Box<dyn Fn(&GameResult) -> Option<f64> + 'm>;

/// Paired differences, side B minus side A, per player, over the seeds both
/// sides played; real minutes under `model` too, when given.
pub fn compare_table(a: &Results, b: &Results, ms: &Milestones, names: (&str, &str), model: Option<(&str, &Human)>) -> String {
    let mut t = String::new();
    let _ = writeln!(
        t,
        "## Paired: {} minus {} (same seeds; mean difference ± its standard error)\n",
        names.1, names.0
    );
    let _ = writeln!(t, "| Player | Measure | {} | {} | Diff | SE |", names.0, names.1);
    let _ = writeln!(t, "|---|---|---|---|---|---|");
    for ((ca, ga), (_, gb)) in a.cells.iter().zip(&b.cells) {
        let label = ca.agent.clone();
        let mut measures: Vec<(String, Measure)> = vec![
            ("run 1 ticks".into(), Box::new(|g: &GameResult| g.runs.first().map(|r| r.ticks as f64))),
            (
                "mean run ticks".into(),
                Box::new(|g: &GameResult| {
                    (!g.runs.is_empty()).then(|| g.runs.iter().map(|r| r.ticks as f64).sum::<f64>() / g.runs.len() as f64)
                }),
            ),
            ("run 1 decisions".into(), Box::new(|g: &GameResult| g.runs.first().map(|r| r.decisions as f64))),
            ("idle share".into(), Box::new(|g: &GameResult| (g.ticks > 0).then(|| g.idle_ticks as f64 / g.ticks as f64))),
            ("milestones reached".into(), Box::new(|g: &GameResult| Some(g.milestones.len() as f64))),
        ];
        if let Some((name, h)) = model {
            measures.push((
                format!("run 1 minutes ({name})"),
                Box::new(move |g: &GameResult| {
                    let r = g.runs.first()?;
                    let c = g.clock(Some(h))?;
                    Some(c.span(r.start_tick, r.end_tick) / 60.0)
                }),
            ));
        }
        for m in &ms.milestones {
            let id = m.id.clone();
            measures.push((
                format!("{} run", m.id),
                Box::new(move |g: &GameResult| g.milestones.get(&id).map(|h| h.run as f64)),
            ));
            let id = m.id.clone();
            measures.push((
                format!("{} tick", m.id),
                Box::new(move |g: &GameResult| g.milestones.get(&id).map(|h| h.tick as f64)),
            ));
        }
        for (name, f) in &measures {
            let pairs: Vec<(f64, f64)> = ga
                .iter()
                .zip(gb)
                .filter_map(|(x, y)| Some((f(x)?, f(y)?)))
                .collect();
            if pairs.is_empty() {
                continue;
            }
            let ma = mean(&pairs.iter().map(|p| p.0).collect::<Vec<_>>());
            let mb = mean(&pairs.iter().map(|p| p.1).collect::<Vec<_>>());
            let d: Vec<f64> = pairs.iter().map(|p| p.1 - p.0).collect();
            let md = mean(&d);
            let se = if d.len() > 1 {
                let var = d.iter().map(|x| (x - md) * (x - md)).sum::<f64>() / (d.len() - 1) as f64;
                format!("{:.2}", (var / d.len() as f64).sqrt())
            } else {
                "-".into()
            };
            let note = if pairs.len() < ga.len() { format!(" ({} pairs)", pairs.len()) } else { String::new() };
            let _ = writeln!(t, "| {label} | {name}{note} | {ma:.2} | {mb:.2} | {md:+.2} | {se} |");
        }
    }
    t
}

// ---- Balance ----

/// `balance:` lines for the judge's cells against the targets: each
/// milestone reached outside its run window (median), or not reached by
/// careers long enough to reach it (the cell's shortest career, in finished
/// runs); run 1's length against its player's band, and its real time under
/// the spec's first human model (a line when the spec names none).
pub fn balance(sums: &[Summary], ms: &Milestones) -> Vec<String> {
    let mut out = Vec::new();
    for s in sums {
        let who = s.cell.agent.clone();
        let n = s.games;
        let retirements = s.min_retirements;
        for m in &ms.milestones {
            let [lo, hi] = m.target.runs;
            match s.milestones.get(&m.id) {
                None if lo <= retirements => {
                    out.push(format!(
                        "balance: {} {} not reached in {} runs (target runs {lo}-{hi}), {who}, {n} seeds",
                        m.id,
                        m.name,
                        retirements
                    ));
                }
                None => {}
                Some(h) => {
                    let run = h.run.round() as u32;
                    let late = h.reached as u64 * 100 <= n as u64 * ms.judge.late_percent && lo <= retirements;
                    if run < lo || run > hi || late {
                        let part = if h.reached < n { format!(", reached by {}/{n} seeds", h.reached) } else { String::new() };
                        out.push(format!(
                            "balance: {} {} in run {:.0} (target runs {lo}-{hi}){part}, {who}, {n} seeds",
                            m.id, m.name, h.run
                        ));
                    } else if let (Some([dlo, dhi]), 1) = (m.target.ticks, run) {
                        if h.day < dlo as f64 || h.day > dhi as f64 {
                            out.push(format!(
                                "balance: {} {} on run-1 day {:.0} (target {dlo}-{dhi}), {who}, {n} seeds",
                                m.id, m.name, h.day
                            ));
                        }
                    }
                }
            }
        }
        if let (Some(ticks), Some([lo, hi])) = (s.run1_ticks, ms.run1.ticks.get(&s.cell.agent)) {
            if ticks < *lo as f64 || ticks > *hi as f64 {
                out.push(format!("balance: run 1 lasts {ticks:.0} ticks (target {lo}-{hi}), {who}, {n} seeds"));
            }
        }
        // Real time under the spec's first human model (balance.json's
        // `humans`); a judged player with none is said, not skipped.
        if ms.run1.minutes.players.contains(&s.cell.agent) {
            match s.real.first() {
                None => out.push(format!(
                    "balance: run 1's real time not judged: the spec names no human model, {who}, {n} seeds"
                )),
                Some(r) => {
                    let human = &r.human;
                    if let Some(min) = r.run1_min {
                        let [lo, hi] = ms.run1.minutes.band;
                        if min < lo as f64 || min > hi as f64 {
                            out.push(format!(
                                "balance: run 1 takes {min:.1} real minutes as {human} (target {lo}-{hi}), {who}, {n} seeds"
                            ));
                        }
                    }
                }
            }
        }
    }
    out
}
