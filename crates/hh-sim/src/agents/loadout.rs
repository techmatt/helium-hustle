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

use hh_core::state::{Amount, Entry};

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

/// One line, read by the game's text form (`hh_core::program_text`, the text
/// editor's), so a loadout file reads as the editor would.
fn line(text: &str) -> Result<Entry, String> {
    let op = hh_core::program_text::parse_op(text.trim())?;
    Ok(Entry { op, indented: text.starts_with([' ', '\t']) })
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
    use hh_core::state::{Cmp, Join, Op, Reading};

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
    fn the_loadout_files_parse() {
        // The hands-off loadout and 060's write-up loadouts (README, *Rules
        // changes*: a grammar change converts them).
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
        let mut files = vec![format!("{root}/configs/sim/handsoff_loadout.txt")];
        for f in ["L1_compute_floor", "L2_short_chain", "L3_one_tab", "L4_two_tabs", "L5_dream_tab"] {
            files.push(format!("{root}/experiments/results/060_loadout_exploration/loadouts/{f}.txt"));
        }
        let mut n = 0;
        for f in files {
            let text = std::fs::read_to_string(&f).unwrap();
            let l = parse(&text).unwrap_or_else(|e| panic!("{f}: {e}"));
            n += l.programs.iter().map(|(_, e)| e.len()).sum::<usize>();
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
