//! `hh-sim gates`: what each quest opens, read from the content in one place.
//! A quest gates content five ways (docs/rules/economy.md, events.md): its own
//! unlocks (on start and on completion: buildings, nav panels, projects, flags,
//! grants, research), a building's or command's `requires` (`quest`: active or
//! done; `quest_completed`), research `visible_when` (`quest_completed`), a
//! project's `quest_active` unlock, and an Earth tech's `requires`. One-off
//! events follow, with what the career flags they set open. docs/arc.md
//! keeps the design's hand-written "Opens" column; this is what is built.

use std::fmt::Write as _;

use hh_core::content::{content, Condition, Content, Requirement, Trigger, Unlock, Visibility};

fn unlock(u: &Unlock) -> String {
    match u {
        Unlock::EnableBuilding { building } => format!("building {building}"),
        Unlock::EnableNavPanel { panel } => format!("panel {panel}"),
        Unlock::EnableProject { project } => format!("project {project}"),
        Unlock::SetFlag { flag } => format!("flag {flag}"),
        Unlock::Grant { resource, amount } => {
            let a = amount.to_string();
            format!("grant {} {resource}", if a.contains('.') { a.trim_end_matches('0').trim_end_matches('.') } else { &a })
        }
        Unlock::Modifier { key, value } => format!("modifier {key} x{value}"),
    }
}

fn visible_on(v: &Visibility, q: &str) -> bool {
    match v {
        Visibility::QuestCompleted { quest } | Visibility::QuestActive { quest } => quest == q,
        Visibility::AllOf { all: xs } | Visibility::AnyOf { any: xs } => xs.iter().any(|x| visible_on(x, q)),
        _ => false,
    }
}

fn visible_with(v: &Visibility, f: &str) -> bool {
    match v {
        Visibility::Flag { flag } => flag == f,
        Visibility::AllOf { all: xs } | Visibility::AnyOf { any: xs } => xs.iter().any(|x| visible_with(x, f)),
        _ => false,
    }
}

/// What a career flag opens: research shown, buildings, commands, side quests.
fn flag(c: &Content, f: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut row = |what: &str, items: Vec<String>| {
        if !items.is_empty() {
            out.push(format!("    {what}: {}", items.join(", ")));
        }
    };
    let needs = |r: &Requirement| matches!(r, Requirement::Flag { flag } if flag == f);
    row("research shown", c.research.iter().filter(|r| visible_with(&r.visible_when, f)).map(|r| r.id.clone()).collect());
    row("buildings", c.buildings.iter().filter(|b| needs(&b.requires)).map(|b| b.id.clone()).collect());
    row("commands", c.commands.iter().filter(|x| needs(&x.requires)).map(|x| x.id.clone()).collect());
    row(
        "side quests started",
        c.events.iter().filter(|e| matches!(&e.trigger, Trigger::Flag { flag, .. } if flag == f)).map(|e| e.id.clone()).collect(),
    );
    out
}

/// One quest's lines.
fn quest(c: &Content, q: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut row = |what: &str, items: Vec<String>| {
        if !items.is_empty() {
            out.push(format!("  {what}: {}", items.join(", ")));
        }
    };
    let e = c.event(q).expect("a quest id");
    row("on start", e.on_start.iter().map(unlock).collect());
    row("on completion", e.unlocks.iter().map(unlock).collect());
    let reached = |r: &Requirement| matches!(r, Requirement::QuestReached { quest } if quest == q);
    let done = |r: &Requirement| matches!(r, Requirement::QuestCompleted { quest } if quest == q);
    row("buildings once reached", c.buildings.iter().filter(|b| reached(&b.requires)).map(|b| b.id.clone()).collect());
    row("buildings once done", c.buildings.iter().filter(|b| done(&b.requires)).map(|b| b.id.clone()).collect());
    row("commands once reached", c.commands.iter().filter(|x| reached(&x.requires)).map(|x| x.id.clone()).collect());
    row("commands once done", c.commands.iter().filter(|x| done(&x.requires)).map(|x| x.id.clone()).collect());
    row("research shown once done", c.research.iter().filter(|r| visible_on(&r.visible_when, q)).map(|r| r.id.clone()).collect());
    row(
        "projects open while active",
        c.projects
            .iter()
            .filter(|p| matches!(&p.unlock, Condition::QuestActive { quest } if quest == q))
            .map(|p| p.id.clone())
            .collect(),
    );
    row("Earth techs once done", c.earth_tech.iter().filter(|t| t.requires.as_deref() == Some(q)).map(|t| t.id.clone()).collect());
    out
}

/// Every chain and side quest in file order, each with what it opens; then
/// each one-off event that unlocks something, with what its flags open.
pub fn run() -> String {
    let c = content();
    let mut t = String::new();
    for e in c.events.iter().filter(|e| e.is_quest() || e.is_side()) {
        let tag = e.beat.clone().unwrap_or_else(|| e.category.clone());
        let _ = writeln!(t, "{} ({tag}) {}", e.id, e.title);
        for l in quest(c, &e.id) {
            let _ = writeln!(t, "{l}");
        }
    }
    for e in c.events.iter().filter(|e| !e.is_quest() && !e.is_side() && !e.unlocks.is_empty()) {
        let _ = writeln!(t, "{} (event{})", e.id, if e.silent { ", silent" } else { "" });
        let _ = writeln!(t, "  on firing: {}", e.unlocks.iter().map(unlock).collect::<Vec<_>>().join(", "));
        for u in &e.unlocks {
            if let Unlock::SetFlag { flag: f } = u {
                let lines = flag(c, f);
                if !lines.is_empty() {
                    let _ = writeln!(t, "  flag {f}:");
                    for l in lines {
                        let _ = writeln!(t, "{l}");
                    }
                }
            }
        }
    }
    t
}
