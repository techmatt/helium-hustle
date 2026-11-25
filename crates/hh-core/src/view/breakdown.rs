//! The view's breakdown part: each resource's flows by source for the last day
//! and the rolling average, and who went short of it, with display labels.

use serde::Serialize;
use std::collections::BTreeMap;

use super::{names, Text};
use crate::content::content;
use crate::fx::Fx;
use crate::state::{Flow, GameState, Op, Short};

/// One source's line in a resource's breakdown: the last day's flows and the
/// rolling average's.
#[derive(Debug, Clone, Serialize)]
pub struct FlowRow {
    /// The state's source key (`building:<id>`, `program:<n>`, `cap`, ...).
    pub source: String,
    pub label: String,
    pub produced: Fx,
    pub consumed: Fx,
    pub avg_produced: Fx,
    pub avg_consumed: Fx,
}

/// Who ran short the last day, and of what (`state::Short`, labeled).
#[derive(Debug, Clone, Serialize)]
pub struct ShortView {
    pub source: String,
    pub label: String,
    pub resource: String,
    pub need: Fx,
    pub had: Fx,
    /// The fraction it ran at (0: not at all).
    pub ran: Fx,
}

/// A resource's breakdown: its sources and sinks (sources first, each side
/// largest first; `cap` is what the clamp cut), and who went short of it.
#[derive(Debug, Clone, Serialize)]
pub struct BreakdownView {
    pub resource: Text,
    pub rows: Vec<FlowRow>,
    pub shorts: Vec<ShortView>,
}

/// A breakdown source's display label.
fn source_label(g: &GameState, source: &str) -> String {
    let names = names();
    let mut parts = source.split(':');
    match (parts.next(), parts.next(), parts.next()) {
        (Some("building"), Some(id), _) => names.get(id).copied().unwrap_or(id).to_string(),
        (Some("program"), Some(n), line) => {
            let n: usize = n.parse().unwrap_or(0);
            match line.and_then(|l| l.parse::<usize>().ok()) {
                Some(l) => {
                    let what = match g.run.programs.get(n).and_then(|p| p.entries.get(l)).map(|e| &e.op) {
                        Some(Op::Command { command, .. }) => names.get(command.as_str()).copied().unwrap_or(command).to_string(),
                        Some(Op::Build { building, .. }) => {
                            format!("Build {}", names.get(building.as_str()).copied().unwrap_or(building))
                        }
                        Some(Op::BuyLand {}) => "Buy Land".to_string(),
                        _ => String::new(),
                    };
                    format!("Program {}, line {}: {}", n + 1, l + 1, what)
                }
                None => format!("Program {}", n + 1),
            }
        }
        (Some(k), _, _) => match k {
            "pads" => "Pad loading",
            "launch" => "Launches",
            "buy" => "Buildings bought",
            "sell" => "Buildings sold",
            "land" => "Land bought",
            "research" => "Research",
            "projects" => "Projects",
            "growth" => "Daily growth",
            "dream" => "Dream",
            "cap" => "Lost to cap",
            crate::cheat::CHEAT_SOURCE => "Cheat Fountain",
            other => other,
        }
        .to_string(),
        _ => source.to_string(),
    }
}

pub(super) fn short_view(g: &GameState, source: &str, s: &Short) -> ShortView {
    ShortView {
        source: source.to_string(),
        label: source_label(g, source),
        resource: s.resource.clone(),
        need: s.need,
        had: s.had,
        ran: s.ran,
    }
}

/// Every resource with a flow (the last day's or the average's) or a shortfall.
pub(super) fn breakdown(g: &GameState) -> Vec<BreakdownView> {
    let (today, avg) = (&g.run.today.flows, &g.run.flow_average);
    content()
        .resources
        .iter()
        .filter_map(|res| {
            let mut sources: Vec<&String> =
                today.get(&res.id).into_iter().chain(avg.get(&res.id)).flat_map(|m| m.keys()).collect();
            sources.sort();
            sources.dedup();
            let get = |m: &BTreeMap<String, BTreeMap<String, Flow>>, s: &String| {
                m.get(&res.id).and_then(|x| x.get(s)).copied().unwrap_or_default()
            };
            let mut rows: Vec<FlowRow> = sources
                .into_iter()
                .map(|s| {
                    let (d, a) = (get(today, s), get(avg, s));
                    FlowRow {
                        source: s.clone(),
                        label: source_label(g, s),
                        produced: d.produced,
                        consumed: d.consumed,
                        avg_produced: a.produced,
                        avg_consumed: a.consumed,
                    }
                })
                .collect();
            // Sources (net in) first, then sinks, each largest first.
            rows.sort_by_key(|r| {
                let net = (r.produced - r.consumed, r.avg_produced - r.avg_consumed);
                (net.0 <= Fx::ZERO && net.1 <= Fx::ZERO, std::cmp::Reverse((net.0.max(-net.0), net.1.max(-net.1))))
            });
            let shorts: Vec<ShortView> = g
                .run
                .today
                .shorts
                .iter()
                .filter(|(_, s)| s.resource == res.id)
                .map(|(k, s)| short_view(g, k, s))
                .collect();
            (!rows.is_empty() || !shorts.is_empty()).then_some(BreakdownView { resource: &res.id, rows, shorts })
        })
        .collect()
}
