//! A loadout file: program tabs written as text, one line a program line, so a
//! sim player's programs are data, not strategy code (the `handsoff` player
//! loads `configs/sim/handsoff_loadout.txt`). The form (060's harness form):
//!
//! ```text
//! # a comment (anywhere after `#`)
//! seat: 1=all          # processors at a reset: tab=N or tab=all, in order
//! [1]                  # Program 1's lines follow
//! if energy < 60% and credits > 20
//!   build solar_panel  # indented: in the body of the `if` above
//! 10x cloud_compute
//! ```
//!
//! Each line is read by the game's text form (`hh_core::program_text`, the
//! text editor's: display names or ids, aliases, docs/rules/programs.md
//! *Text form*). Ids are the content's (buildings, commands, resources, research,
//! projects). A condition is `<reading> <op> <number>[%]` (`op`: `>`, `<`, `>=`, `<=`), with one
//! optional `and`/`or`: `free processors`, `free land` and `idle pads` read
//! their count, `energy per day` and `days since a purchase` their number
//! (both researched); any other reading is a resource, its fill percent with
//! `%`, its amount without (a capped one too). Verbs: `build X [up to N]`, `buy land`,
//! `launch pads G|all`, `load pads G|all`, `set pads G up to N`,
//! `pause|resume X [N|all]`, `research X`, `assign processors <tab> N|all`,
//! `fund P [N|all]`, `retire`; anything else is a command `[Nx] id` (its repeat count first).
//! A line it can't read is an error, never a silent skip.

#[cfg(test)]
use hh_core::state::{Clause, Cmp, Condition, Join, Joined, Op, Reading, Verb};
use hh_core::state::{Amount, Entry};
#[cfg(test)]
use hh_core::Fx;

/// A parsed loadout: the reset's processor seats and each tab's lines
/// (tabs 0-based, in file order).
#[derive(Debug, Clone, PartialEq)]
pub struct Loadout {
    pub seat: Vec<(usize, Amount)>,
    pub programs: Vec<(usize, Vec<Entry>)>,
}

fn amount(s: &str) -> Result<Amount, String> {
    if s == "all" {
        Ok(Amount::All)
    } else {
        s.parse::<u32>().map(Amount::N).map_err(|_| format!("bad amount {s:?}"))
    }
}

/// A 1-based tab number as a 0-based index.
fn tab(s: &str) -> Result<usize, String> {
    match s.parse::<usize>() {
        Ok(n) if n >= 1 => Ok(n - 1),
        _ => Err(format!("bad tab {s:?}")),
    }
}

#[cfg(test)]
fn clause(words: &[&str]) -> Result<Clause, String> {
    let at = words.iter().position(|w| *w == ">" || *w == "<").ok_or_else(|| format!("no > or < in {words:?}"))?;
    let cmp = if words[at] == ">" { Cmp::Gt } else { Cmp::Lt };
    let raw = *words.get(at + 1).ok_or("no number")?;
    if words.len() > at + 2 {
        return Err(format!("trailing words in {words:?}"));
    }
    let value = Fx::parse(raw.trim_end_matches('%'))?;
    let what = words[..at].join(" ");
    let pct = raw.ends_with('%');
    let counted = ["free processors", "free land", "idle pads", "energy per day", "days since a purchase"];
    if pct && counted.contains(&what.as_str()) {
        return Err(format!("{what} has no percent: {words:?}"));
    }
    let reading = match what.as_str() {
        "" => return Err(format!("no reading in {words:?}")),
        "free processors" => Reading::FreeProcessors,
        "free land" => Reading::FreeLand,
        "idle pads" => Reading::IdlePads,
        "energy per day" => Reading::EnergyPerDay,
        "days since a purchase" => Reading::DaysSincePurchase,
        r if pct => Reading::Percent { resource: r.to_string() },
        r => Reading::Amount { resource: r.to_string() },
    };
    Ok(Clause { reading, cmp, value })
}

/// One line, read by the game's text form (`hh_core::program_text`, the text
/// editor's), so a loadout file reads as the editor would.
fn line(text: &str) -> Result<Entry, String> {
    let op = hh_core::program_text::parse_op(text.trim())?;
    Ok(Entry { op, indented: text.starts_with([' ', '\t']) })
}

/// 060's own line reader, kept to check the text form reads its files alike.
#[cfg(test)]
fn line_060(text: &str) -> Result<Entry, String> {
    let indented = text.starts_with(' ');
    let w: Vec<&str> = text.split_whitespace().collect();
    let act = |verb, target: Option<&str>, tab: Option<u32>, amount: Option<Amount>| Op::Act { verb, target: target.map(str::to_string), tab, amount };
    let goods = |g: &str| if g == "all" { None } else { Some(g.to_string()) };
    let op = match w.as_slice() {
        ["if", rest @ ..] => {
            let (a, b) = match rest.iter().position(|x| *x == "and" || *x == "or") {
                Some(j) => (&rest[..j], Some((rest[j], &rest[j + 1..]))),
                None => (rest, None),
            };
            let first = clause(a)?;
            let rest = match b {
                Some((j, b)) => Some(Joined { join: if j == "and" { Join::And } else { Join::Or }, clause: clause(b)? }),
                None => None,
            };
            Op::If { when: Condition { first, rest } }
        }
        ["build", b] => Op::Build { building: b.to_string(), up_to: None },
        ["build", b, "up", "to", n] => Op::Build { building: b.to_string(), up_to: Some(n.parse().map_err(|_| format!("bad count {n:?}"))?) },
        ["buy", "land"] => Op::BuyLand {},
        ["launch", "pads", g] => Op::Act { verb: Verb::LaunchPads, target: goods(g), tab: None, amount: None },
        ["load", "pads", g] => Op::Act { verb: Verb::LoadPads, target: goods(g), tab: None, amount: None },
        ["set", "pads", g, "up", "to", n] => act(Verb::SetPads, Some(g), None, Some(amount(n)?)),
        ["pause", b] => act(Verb::Pause, Some(b), None, None),
        ["pause", b, n] => act(Verb::Pause, Some(b), None, Some(amount(n)?)),
        ["resume", b] => act(Verb::Resume, Some(b), None, None),
        ["resume", b, n] => act(Verb::Resume, Some(b), None, Some(amount(n)?)),
        ["research", r] => act(Verb::Research, Some(r), None, None),
        ["assign", "processors", t, n] => act(Verb::AssignProcessors, None, Some(tab(t)? as u32), Some(amount(n)?)),
        ["fund", p] => act(Verb::Fund, Some(p), None, None),
        ["fund", p, n] => act(Verb::Fund, Some(p), None, Some(amount(n)?)),
        ["retire"] => act(Verb::Retire, None, None, None),
        [c] => Op::Command { command: c.to_string(), count: 1 },
        [x, c] if x.ends_with('x') => Op::Command { command: c.to_string(), count: x[..x.len() - 1].parse().map_err(|_| format!("bad count {x:?}"))? },
        _ => return Err(format!("can't read {text:?}")),
    };
    Ok(Entry { op, indented })
}

/// Reads a loadout file; an error names the line.
pub fn parse(text: &str) -> Result<Loadout, String> {
    let mut l = Loadout { seat: Vec::new(), programs: Vec::new() };
    for (n, raw) in text.lines().enumerate() {
        let at = |e: String| format!("line {}: {e}", n + 1);
        let t = raw.split('#').next().unwrap_or("").trim_end();
        if t.trim().is_empty() {
            continue;
        }
        if let Some(s) = t.strip_prefix("seat:") {
            for kv in s.split_whitespace() {
                let (k, v) = kv.split_once('=').ok_or_else(|| at(format!("seat wants tab=N, got {kv:?}")))?;
                l.seat.push((tab(k).map_err(at)?, amount(v).map_err(at)?));
            }
        } else if t.starts_with('[') {
            let t = tab(t.trim_matches(|c| c == '[' || c == ']')).map_err(at)?;
            if l.programs.iter().any(|(x, _)| *x == t) {
                return Err(at(format!("tab {} twice", t + 1)));
            }
            l.programs.push((t, Vec::new()));
        } else {
            let p = l.programs.last_mut().ok_or_else(|| at("a line before any [tab]".to_string()))?;
            p.1.push(line(t).map_err(at)?);
        }
    }
    Ok(l)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_reads_every_form() {
        let l = parse(
            "seat: 1=all 2=1 # c\n[2]\nif he3 > 10% or credits < 20\n  load pads all\nlaunch pads he3\n\
             set pads he3 up to 1\nbuild smelter up to 2\nbuy land\nassign processors 3 all\nfund foundation_grant 3\n5x dream\nretire\n",
        )
        .unwrap();
        assert_eq!(l.seat, vec![(0, Amount::All), (1, Amount::N(1))]);
        let (t, e) = &l.programs[0];
        assert_eq!(*t, 1);
        assert_eq!(e.len(), 10);
        assert!(e[1].indented && !e[2].indented);
        match &e[0].op {
            Op::If { when } => {
                assert_eq!(when.first.reading, Reading::Percent { resource: "he3".into() });
                let r = when.rest.as_ref().unwrap();
                assert_eq!((r.join, r.clause.cmp), (Join::Or, Cmp::Lt));
                assert_eq!(r.clause.reading, Reading::Amount { resource: "credits".into() });
            }
            o => panic!("{o:?}"),
        }
        assert_eq!(e[8].op, Op::Command { command: "dream".into(), count: 5 });
    }

    #[test]
    fn the_text_form_reads_the_loadout_files_as_060_did() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
        let mut files = vec![format!("{root}/configs/sim/handsoff_loadout.txt")];
        for f in ["L1_compute_floor", "L2_short_chain", "L3_one_tab", "L4_two_tabs", "L5_dream_tab"] {
            files.push(format!("{root}/experiments/results/060_loadout_exploration/loadouts/{f}.txt"));
        }
        let mut n = 0;
        for f in files {
            let text = std::fs::read_to_string(&f).unwrap();
            for raw in text.lines() {
                let t = raw.split('#').next().unwrap_or("").trim_end();
                if t.trim().is_empty() || t.starts_with("seat:") || t.starts_with('[') {
                    continue;
                }
                assert_eq!(line(t), line_060(t), "{f}: {t}");
                n += 1;
            }
        }
        assert!(n > 100);
    }

    #[test]
    fn it_refuses_what_it_cannot_read() {
        for bad in ["build x", "[1]\nif energy 50%", "[1]\nbuild a up to many", "[0]", "[1]\n[1]", "seat: 1", "[1]\nif > 5", "[1]\nif energy per day < 0%", "[1]\nif energy above 5"] {
            assert!(parse(bad).is_err(), "{bad:?}");
        }
    }
}
