//! The text editor's form (`program_text`): reading, writing, the round trip,
//! line status, and the note, unparsed and locked lines it keeps.

use super::{logic_open, outcomes, rules};
use crate::modules::programs;
use crate::program_text::{parse, parse_op, to_text};
use crate::state::{Amount, Entry, GameState, Loadout, Op, Verb};
use crate::{Action, Rules};

/// A game with program logic and `if` open.
fn open(r: &Rules) -> GameState {
    logic_open(r)
}

/// 060's L5 written in the text form (display names, aliases mixed in).
const L5: &str = "\
# Program 1: build and seat processors
if free processors > 0
  assign processors program 2 1
  assign processors p3 1
  assign processors 1 all
if Energy < 60% and credits > 20
  build solar panel
if energy >= 20%
  10X Sell Cloud Compute
if energy < 21%
  5x idle
if titanium > 99% or regolith > 99%
  build storage_depot
if free land < 5 and credits > 200
  buy land
if circuit boards > 79% and credits > 150
  build data center up to 6
  build research lab up to 2
if ice < 30% and credits > 200
  10 x buy ice
if credits > 150 and energy > 70%
  build regolith excavator up to 2
  build smelter up to 1
  build refinery up to 1
  build launch pad up to 1
  build fabricator up to 1
if helium-3 > 10%
  5× load_pads
launch pads all
if propellant < 67% and energy > 30%
  5x buy propellant
if science > 49
  research process_telemetry
if days since a purchase > 299
  retire
if boredom > 50%
  5x dream
";

#[test]
fn text_reads_every_form_and_round_trips() {
    let r = rules();
    let g = open(&r);
    let p = parse(&r, &g, L5);
    let red: Vec<_> = p.lines.iter().filter(|l| l.status == "red").collect();
    assert!(red.is_empty(), "{red:?}");
    assert_eq!(p.lines.len(), L5.split('\n').count());
    assert_eq!(p.entries[0].op, Op::Note { text: "Program 1: build and seat processors".into() });
    assert_eq!(p.entries[3].op, Op::Act { verb: Verb::AssignProcessors, target: None, tab: Some(2), amount: Some(Amount::N(1)) });
    assert_eq!(p.entries[8].op, Op::Command { command: "cloud_compute".into(), count: 10 });
    assert!(p.entries[8].indented && !p.entries[9].indented);
    // Written and read again: the same lines, and the text is a fixed point.
    let text = to_text(&p.entries);
    let again = parse(&r, &g, &text);
    assert_eq!(again.entries, p.entries);
    assert_eq!(to_text(&again.entries), text);
    assert!(text.contains("  10x sell cloud compute\n"), "{text}");
    assert!(text.contains("if energy < 60% and credits > 20\n"), "{text}");
    // And `set_program` takes it whole.
    let mut g = g;
    r.apply(&mut g, &Action::SetProgram { program: 0, entries: p.entries }).unwrap();
}

#[test]
fn every_verb_round_trips() {
    let act = |verb, target: Option<&str>, tab, amount| Op::Act { verb, target: target.map(str::to_string), tab, amount };
    let ops = [
        act(Verb::LaunchPads, Some("he3"), None, None),
        act(Verb::LoadPads, None, None, None),
        act(Verb::SetPads, Some("titanium"), None, Some(Amount::N(2))),
        act(Verb::Pause, Some("smelter"), None, Some(Amount::All)),
        act(Verb::Resume, Some("solar_panel"), None, None),
        act(Verb::Research, Some("process_telemetry"), None, None),
        act(Verb::LoadProgram, Some("My Builder 2"), Some(1), None),
        act(Verb::AssignProcessors, None, Some(0), Some(Amount::N(0))),
        act(Verb::Retire, None, None, None),
        act(Verb::Fund, Some("foundation_grant"), None, Some(Amount::N(3))),
        Op::BuyLand {},
        Op::Build { building: "excavator".into(), up_to: Some(4) },
        Op::Command { command: "idle".into(), count: 1 },
    ];
    // A saved program's name is free text: signs, `#`, a verb's word, odd spacing.
    let names = ["x>y", "a <= b", "Plan #2", "Pads Morning", "Big  Plan", "go into orbit"];
    let loads = names.iter().map(|n| act(Verb::LoadProgram, Some(n), Some(2), None));
    for op in ops.into_iter().chain(loads) {
        let text = crate::program_text::op_text(&op);
        assert_eq!(parse_op(&text), Ok(op.clone()), "{text}");
    }
}

#[test]
fn a_locked_line_is_skipped_free_and_says_why() {
    let r = rules();
    // Before program logic: a `build` line is taken, amber, and skipped free when run.
    let mut g = r.new_game(1);
    super::set(&mut g, "credits", "100000");
    let solar = g.owned("solar_panel");
    let p = parse(&r, &g, "build solar panel\nlaunch pads all\n");
    r.apply(&mut g, &Action::SetProgram { program: 0, entries: p.entries }).unwrap();
    r.advance(&mut g, 1);
    if !programs::logic_unlocked(&g) {
        assert_eq!(g.owned("solar_panel"), solar, "nothing bought before program logic");
        assert_eq!(outcomes(&g)[0], None);
    }
    // With program logic open and no pad: the pad verb is amber, skipped when run.
    let g = open(&r);
    let p = parse(&r, &g, "launch pads all\n");
    assert_eq!(p.lines[0].status, "amber", "{:?}", p.lines[0]);
}

#[test]
fn aliases_read_as_the_canonical_form() {
    for (a, b) in [
        ("launch", "launch pads all"),
        ("IF   Credits <= 5", "if credits <= 5"),
        ("if energy≥50%", "if energy >= 50%"),
        ("if energy≤50%", "if energy <= 50%"),
        ("if propellant<20", "if propellant < 20"),
        ("3* buy_ice", "3x buy ice"),
        ("3 x buy ice", "3x buy ice"),
        ("3X Buy Ice", "3x buy ice"),
        ("3× buy ice", "3x buy ice"),
        ("set pads he3 2", "set pads helium-3 up to 2"),
        ("load builder into tab 2", "load builder into program 2"),
    ] {
        assert_eq!(parse_op(a), parse_op(b), "{a} / {b}");
        assert!(parse_op(a).is_ok(), "{a}: {:?}", parse_op(a));
    }
}

#[test]
fn a_repeat_count_leads_and_a_trailing_number_is_an_argument() {
    let op = parse_op("4x sell cloud compute").unwrap();
    assert_eq!(op, Op::Command { command: "cloud_compute".into(), count: 4 });
    assert_eq!(crate::program_text::op_text(&op), "4x sell cloud compute");
    // The old trailing form is no longer read; the reason shows the new one.
    for old in ["sell cloud compute 4", "sell cloud compute x4", "cloud_compute (x4)", "sell cloud compute x 4"] {
        let why = parse_op(old).unwrap_err();
        assert_eq!(why, "a repeat count goes first: 4x sell cloud compute", "{old}");
    }
    assert!(parse_op("0x idle").is_err());
    assert!(parse_op("4x castle").is_err());
    // Verbs keep their trailing argument.
    assert_eq!(parse_op("pause data center 1").unwrap(), Op::Act { verb: Verb::Pause, target: Some("data_center".into()), tab: None, amount: Some(Amount::N(1)) });
}

#[test]
fn red_lines_say_why_and_stay_as_unparsed_lines() {
    let r = rules();
    let g = open(&r);
    let p = parse(&r, &g, "if energy\nbuild castle\n  idle\nlaunch pads all # go\nif free land > 5%\nfrobnicate 3\n\n  # a note\n");
    let st: Vec<_> = p.lines.iter().map(|l| l.status).collect();
    assert_eq!(st, ["red", "red", "red", "red", "red", "red", "blank", "note", "blank"]);
    for l in &p.lines[..6] {
        assert!(l.reason.as_deref().is_some_and(|w| !w.is_empty()));
    }
    assert_eq!(p.lines[2].reason.as_deref(), Some("an indented line needs an if before it"));
    assert_eq!(p.entries[1].op, Op::Unparsed { text: "build castle".into() });
    assert!(p.entries.iter().all(|e| !e.indented), "no line above the indented ones is an if");
    // Applied, the program keeps them; they never run and cost no step.
    let mut g = g;
    r.apply(&mut g, &Action::SetProgram { program: 0, entries: p.entries.clone() }).unwrap();
    r.advance(&mut g, 3);
    assert_eq!(g.run.programs[0].entries, p.entries);
    assert!(outcomes(&g).iter().all(Option::is_none));
    let v = crate::view::build(&r, &g);
    assert!(v.programs[0].errors[1].as_deref().is_some_and(|w| w.contains("castle")));
    // Back to text and read again: nothing lost.
    assert_eq!(parse(&r, &g, &to_text(&p.entries)).entries, p.entries);
}

#[test]
fn over_long_text_and_too_many_lines_are_red() {
    let r = rules();
    let g = open(&r);
    let long = format!("# {}", "x".repeat(r.p.programs.line_text_max + 5));
    let p = parse(&r, &g, &long);
    assert_eq!(p.lines[0].status, "red");
    let many = "idle\n".repeat(r.p.programs.max_entries + 2);
    let p = parse(&r, &g, &many);
    assert_eq!(p.lines.iter().filter(|l| l.status == "red").count(), 2);
}

#[test]
fn else_reads_writes_and_says_why_where_it_cant_stand() {
    let r = rules();
    let g = open(&r);
    let text = "if energy < 20%\n  idle\nELSE\n  2x sell cloud compute\n  # a note\nidle\nif credits > 5\nelse\n";
    let p = parse(&r, &g, text);
    assert!(p.lines.iter().all(|l| l.status != "red"), "{:?}", p.lines);
    assert_eq!(p.entries[2].op, Op::Else {});
    assert!(!p.entries[2].indented && p.entries[3].indented && p.entries[4].indented && !p.entries[5].indented);
    assert_eq!(p.entries[7].op, Op::Else {});
    let written = to_text(&p.entries);
    assert_eq!(written, text.replace("ELSE", "else"));
    assert_eq!(parse(&r, &g, &written).entries, p.entries);
    let red = |t: &str| {
        let p = parse(&r, &g, t);
        p.lines.iter().find(|l| l.status == "red").and_then(|l| l.reason.clone())
    };
    assert_eq!(red("else\n").as_deref(), Some("an else needs an if before it"));
    assert_eq!(red("if free land > 0\nelse\nelse\n").as_deref(), Some("an if has one else"));
    assert_eq!(red("if free land > 0\n  idle\n  else\n").as_deref(), Some("an else can't sit inside an if"));
    assert_eq!(red("if free land > 0\nelse if credits > 1\n").as_deref(), Some("no else if: an else's body holds no if"));
    assert_eq!(red("if free land > 0\nelse\n  if credits > 1\n").as_deref(), Some("an if can't sit inside another if"));
    assert!(red("if free land > 0\nelse idle\n").is_some());
    // A red else's body lines are red too: nothing above them opens a block.
    let p = parse(&r, &g, "else\n  idle\n");
    assert_eq!(p.lines[1].reason.as_deref(), Some("an indented line needs an if before it"));
    // Held as unparsed lines, the view says why each one doesn't read.
    let mut g = g;
    r.apply(&mut g, &Action::SetProgram { program: 0, entries: p.entries }).unwrap();
    let v = r.view(&g);
    assert_eq!(v.programs[0].errors[0].as_deref(), Some("an else needs an if before it"));
}

#[test]
fn notes_in_a_body_ride_with_it_and_cost_nothing() {
    let r = rules();
    let mut g = open(&r);
    let p = parse(&r, &g, "if credits > 999999\n  # never\n  idle\nidle\n# end\n");
    assert!(p.entries[1].indented, "a note may sit in a body");
    r.apply(&mut g, &Action::SetProgram { program: 0, entries: p.entries }).unwrap();
    use crate::state::LineOutcome as O;
    // The false `if` takes the day's step; its body (note aside) is jumped.
    r.advance(&mut g, 1);
    let lines = outcomes(&g);
    assert_eq!(lines[..3], [Some(O::False), None, Some(O::Jumped)]);
    r.advance(&mut g, 1);
    let lines = outcomes(&g);
    assert_eq!(lines[3], Some(O::Ran));
    assert_eq!((lines[1], lines[4]), (None, None));
}

#[test]
fn locked_lines_are_amber_accepted_and_kept() {
    let r = rules();
    // A fresh game: no program logic, no `if`.
    let mut g = r.new_game(1);
    let text = "if energy > 50%\n  build launch pad\nfund foundation_grant all\nresearch dream_protocols\nidle\n";
    let p = parse(&r, &g, text);
    let st: Vec<_> = p.lines.iter().map(|l| l.status).collect();
    assert_eq!(st, ["amber", "amber", "amber", "amber", "ok", "blank"]);
    assert!(p.lines[0].reason.as_deref().is_some_and(|w| w.contains("program logic")));
    // `set_program` takes what this run hasn't unlocked; the view says why each waits.
    r.apply(&mut g, &Action::SetProgram { program: 0, entries: p.entries.clone() }).unwrap();
    let v = crate::view::build(&r, &g);
    assert_eq!(v.programs[0].locked.iter().filter(|l| l.is_some()).count(), 4);
    // Program logic open, `if` not yet: the `if` is still amber, the rest move on.
    g.career.flags.insert(programs::LOGIC_FLAG.into());
    let p = parse(&r, &g, text);
    assert_eq!(p.lines[0].status, "amber");
    assert!(p.lines[0].reason.as_deref().is_some_and(|w| w.contains("Conditional Logic")));
}

#[test]
fn a_loadout_with_locked_lines_loads_at_a_fresh_reset_and_keeps_them() {
    let r = rules();
    let mut g = r.new_game(1);
    let entries = parse(&r, &open(&r), L5).entries;
    g.career.library.push(Loadout { name: "L5".into(), entries: entries.clone() });
    // A fresh run: nothing of it open but plain commands.
    assert!(!programs::logic_unlocked(&g));
    r.apply(&mut g, &Action::LoadLoadout { loadout: 0, program: 0 }).unwrap();
    assert_eq!(g.run.programs[0].entries, entries);
    r.advance(&mut g, 5);
    assert_eq!(g.run.programs[0].entries, entries, "kept while they wait");
    let v = crate::view::build(&r, &g);
    assert!(v.programs[0].locked.iter().filter(|l| l.is_some()).count() > 20);
    // Saved again from the program, it round-trips through the library.
    r.apply(&mut g, &Action::SaveLoadout { program: 0, slot: None, name: "again".into() }).unwrap();
    assert_eq!(g.career.library[1].entries, entries);
}

#[test]
fn the_written_up_loadouts_read_without_a_red_line() {
    let r = rules();
    let g = open(&r);
    for text in [
        include_str!("../../../../experiments/results/063_program_text_editor/L4_program1.txt"),
        include_str!("../../../../experiments/results/063_program_text_editor/L4_program2.txt"),
        include_str!("../../../../experiments/results/063_program_text_editor/L5_program3.txt"),
    ] {
        let p = parse(&r, &g, text);
        assert!(p.lines.iter().all(|l| l.status != "red"), "{:?}", p.lines);
        // Already canonical: written back, the same text.
        assert_eq!(to_text(&p.entries), text);
    }
}

#[test]
fn hand_entries_match_the_text_form() {
    // The text of a held program is what the editor shows; an empty program is empty text.
    assert_eq!(to_text(&[]), "");
    let e = vec![Entry::command("idle", 3), Entry::new(Op::Note { text: String::new() })];
    assert_eq!(to_text(&e), "3x idle\n#\n");
}

#[test]
fn the_instruction_panel_lists_what_is_revealed_and_each_line_reads() {
    use crate::program_text::{else_fits, instructions};
    let r = rules();
    let g = open(&r);
    let groups = instructions(&r, &g, "", 0);
    let names: Vec<_> = groups.iter().map(|x| x.name).collect();
    assert_eq!(names.first(), Some(&"Commands"));
    assert_eq!(names.last(), Some(&"Control flow"));
    // Every entry reads back as a line, none red at the top level.
    for item in groups.iter().flat_map(|x| &x.items) {
        let p = parse(&r, &g, &item.text);
        assert_ne!(p.lines[0].status, "red", "{}: {:?}", item.text, p.lines[0]);
    }
    let flow = &groups.last().unwrap().items;
    assert!(flow.iter().any(|i| i.text == "if energy > 0%"), "{flow:?}");
    assert!(flow.iter().any(|i| i.text == "if credits > 0"), "{flow:?}");
    assert!(flow.iter().all(|i| i.text != "else"), "no else at the top of an empty program");
    let build = groups.iter().find(|x| x.name == "Build").unwrap();
    let owned = g.owned("solar_panel");
    assert!(build.items.iter().any(|i| i.text == format!("build solar panel up to {}", owned + 1)));
    // `else` fits below an `if` or its body, never below a plain line or a second time.
    let text = "if credits > 0\n  idle\nidle\nif energy > 0%\nelse\n  idle";
    // On the `if` line it would take the body: only at the body's end.
    assert!(!else_fits(&r, &g, text, 0) && else_fits(&r, &g, text, 1));
    assert!(!else_fits(&r, &g, text, 2) && !else_fits(&r, &g, text, 3) && !else_fits(&r, &g, text, 4));
    assert!(else_fits(&r, &g, "if credits > 0", 0) && else_fits(&r, &g, "if credits > 0\n  idle\n\nidle", 1));
    // Only what can be used now: before program logic, nothing locked (amber) is listed.
    let fresh = r.new_game(1);
    let groups = instructions(&r, &fresh, "", 0);
    assert!(groups.iter().all(|x| x.name != "Build"), "{groups:?}");
    for item in groups.iter().flat_map(|x| &x.items) {
        let p = parse(&r, &fresh, &item.text);
        assert!(["ok", "note"].contains(&p.lines[0].status), "{}: {:?}", item.text, p.lines[0]);
    }
    assert!(!groups.is_empty(), "the commands are usable on a fresh career");
}
