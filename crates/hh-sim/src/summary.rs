//! `hh-sim summarize GAMES.jsonl` (docs/simulator.md): a read's per-game
//! tables from its `games.jsonl`, the lines a ladder or experiment writes
//! (streamed as games finish, so a read still running can be summarized).
//! Each game's runs, its milestones and its career's long goals at the end.

use std::fmt::Write as _;

use serde_json::Value;

use crate::milestones::Milestones;

/// The games' tables; milestone columns in `ms`'s order (any other a game
/// met after them).
pub fn tables(games: &[Value], ms: &Milestones) -> String {
    let mut t = String::new();
    let who = |g: &Value| {
        let agent = g["cell"]["agent"].as_str().or(g["agent"].as_str()).unwrap_or("?");
        match g["cell"]["variant"].as_str().filter(|v| *v != "base") {
            Some(v) => format!("{v} {agent}"),
            None => agent.to_string(),
        }
    };
    let num = |v: &Value| v.as_f64().unwrap_or(0.0);
    let _ = writeln!(t, "## Runs ({} games; `+` the run under way at the end)\n", games.len());
    let _ = writeln!(t, "| Player | Seed | Run | Ticks | Credits | Cr/day | Ships | Built | Research | Ended |");
    let _ = writeln!(t, "|---|---|---|---|---|---|---|---|---|---|");
    for g in games {
        let runs = g["runs"].as_array().cloned().unwrap_or_default();
        let current = g["current"].as_object().map(|_| g["current"].clone());
        for (r, under_way) in runs.iter().map(|r| (r, false)).chain(current.iter().map(|r| (r, true))) {
            // A run's length: `days` in games.jsonl written before 032.
            let ticks = num(if r["ticks"].is_null() { &r["days"] } else { &r["ticks"] });
            let credits = num(&r["credits_earned"]);
            let ended = if under_way {
                "-"
            } else if r["forced"].as_bool() == Some(true) {
                "forced"
            } else {
                "retired"
            };
            let _ = writeln!(
                t,
                "| {} | {} | {}{} | {ticks:.0} | {credits:.0} | {:.1} | {} | {} | {} | {ended} |",
                who(g),
                g["seed"],
                r["run"],
                if under_way { "+" } else { "" },
                if ticks > 0.0 { credits / ticks } else { 0.0 },
                r["shipments"],
                r["buildings_built"],
                r["research"],
            );
        }
    }
    // Milestone columns: the file's order, then any other met.
    let mut ids: Vec<String> = ms.milestones.iter().map(|m| m.id.clone()).collect();
    for g in games {
        for id in g["milestones"].as_object().into_iter().flat_map(|m| m.keys()) {
            if !ids.contains(id) {
                ids.push(id.clone());
            }
        }
    }
    let _ = writeln!(t, "\n## Milestones (career tick `t`, run `r`; `-` not met)\n");
    let _ = writeln!(t, "| Player | Seed | {} |", ids.join(" | "));
    let _ = writeln!(t, "|---|---|{}", "---|".repeat(ids.len()));
    for g in games {
        let cells: Vec<String> = ids
            .iter()
            .map(|id| {
                let h = &g["milestones"][id.as_str()];
                if h.is_object() {
                    format!("t{} r{}", h["tick"], h["run"])
                } else {
                    "-".into()
                }
            })
            .collect();
        let _ = writeln!(t, "| {} | {} | {} |", who(g), g["seed"], cells.join(" | "));
    }
    let _ = writeln!(t, "\n## Career at the end\n");
    let _ = writeln!(t, "| Player | Seed | Ret | Ticks | Best ranks | Projects | Quests | Sub-objectives | Achievements |");
    let _ = writeln!(t, "|---|---|---|---|---|---|---|---|---|");
    let list = |v: &Value| {
        let xs: Vec<&str> = v.as_array().into_iter().flatten().filter_map(Value::as_str).collect();
        crate::experiment::joined(&xs)
    };
    let count = |v: &Value| v.as_array().map_or(0, Vec::len);
    for g in games {
        let ranks: Vec<String> = g["best_ranks"]
            .as_object()
            .into_iter()
            .flatten()
            .map(|(axis, r)| format!("{axis} {r}"))
            .collect();
        let _ = writeln!(
            t,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            who(g),
            g["seed"],
            g["retirements"],
            g["ticks"],
            if ranks.is_empty() { "-".into() } else { ranks.join(", ") },
            list(&g["projects"]),
            count(&g["quests"]),
            count(&g["sub_objectives"]),
            list(&g["achievements"]),
        );
    }
    t
}

/// Parses `games.jsonl` text: one game a line (blank lines skipped).
pub fn parse(text: &str) -> Result<Vec<Value>, String> {
    text.lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
        .map(|(i, l)| serde_json::from_str(l).map_err(|e| format!("line {}: {e}", i + 1)))
        .collect()
}
