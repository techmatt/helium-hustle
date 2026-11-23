//! A program as text, the text editor's form and its copy/paste import and
//! export (docs/rules/programs.md, *Text form*): one line a program line,
//! the words the block editor shows, an indented line in the body of the `if`
//! or `else` above, `#` a note line. Reading is case-insensitive with flexible spacing,
//! and takes the aliases listed there; writing gives one canonical form, so
//! text read from `to_text` reads back to the same lines.

use serde::Serialize;

use crate::content::content;
use crate::fx::Fx;
use crate::modules::programs;
use crate::rules::Rules;
use crate::state::{Amount, Clause, Cmp, Condition, Entry, GameState, Join, Joined, Op, Reading, Verb};

/// A program's lines as text, one a line, ending with a newline when not empty.
pub fn to_text(entries: &[Entry]) -> String {
    entries.iter().map(|e| format!("{}{}\n", if e.indented { "  " } else { "" }, op_text(&e.op))).collect()
}

/// A content row's name as text: its display name, lowercase.
fn name_of(id: &str) -> String {
    let c = content();
    let name = c
        .commands
        .iter()
        .map(|x| (&x.id, &x.name))
        .chain(c.buildings.iter().map(|x| (&x.id, &x.name)))
        .chain(c.resources.iter().map(|x| (&x.id, &x.name)))
        .chain(c.research.iter().map(|x| (&x.id, &x.name)))
        .chain(c.projects.iter().map(|x| (&x.id, &x.name)))
        .find(|(i, _)| i.as_str() == id)
        .map_or(id, |(_, n)| n.as_str());
    name.to_lowercase()
}

/// A number as written: no trailing zeros ("20", "0.5", "-3").
fn number(v: Fx) -> String {
    let s = v.to_string();
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() || s == "-" { "0".to_string() } else { s.to_string() }
}

fn amount_text(a: Amount) -> String {
    match a {
        Amount::N(n) => n.to_string(),
        Amount::All => "all".to_string(),
    }
}

fn reading_text(r: &Reading) -> String {
    match r {
        Reading::Amount { resource } | Reading::Percent { resource } => name_of(resource),
        Reading::FreeProcessors => "free processors".to_string(),
        Reading::FreeLand => "free land".to_string(),
        Reading::IdlePads => "idle pads".to_string(),
        Reading::EnergyPerDay => "energy per day".to_string(),
        Reading::DaysSincePurchase => "days since a purchase".to_string(),
    }
}

fn clause_text(c: &Clause) -> String {
    let pct = if matches!(c.reading, Reading::Percent { .. }) { "%" } else { "" };
    format!("{} {} {}{pct}", reading_text(&c.reading), c.cmp.symbol(), number(c.value))
}

/// One line's text, without its indent.
pub fn op_text(op: &Op) -> String {
    match op {
        Op::Command { command, count: 1 } => name_of(command),
        Op::Command { command, count } => format!("{} {count}", name_of(command)),
        Op::Build { building, up_to: None } => format!("build {}", name_of(building)),
        Op::Build { building, up_to: Some(n) } => format!("build {} up to {n}", name_of(building)),
        Op::BuyLand {} => "buy land".to_string(),
        Op::Else {} => "else".to_string(),
        Op::If { when } => match &when.rest {
            None => format!("if {}", clause_text(&when.first)),
            Some(j) => {
                let join = if j.join == Join::And { "and" } else { "or" };
                format!("if {} {join} {}", clause_text(&when.first), clause_text(&j.clause))
            }
        },
        Op::Act { verb, target, tab, amount } => {
            let t = || target.as_deref().map_or("all".to_string(), name_of);
            let n = || amount.map(|a| format!(" {}", amount_text(a))).unwrap_or_default();
            let p = || tab.map_or(0, |t| t + 1);
            match verb {
                Verb::LaunchPads => format!("launch pads {}", t()),
                Verb::LoadPads => format!("load pads {}", t()),
                Verb::SetPads => format!("set pads {} up to{}", t(), n()),
                Verb::Pause => format!("pause {}{}", t(), n()),
                Verb::Resume => format!("resume {}{}", t(), n()),
                Verb::Research => format!("research {}", t()),
                // A saved program's name as it is (names keep their case).
                Verb::LoadProgram => format!("load {} into program {}", target.as_deref().unwrap_or(""), p()),
                Verb::AssignProcessors => format!("assign processors program {}{}", p(), n()),
                Verb::Retire => "retire".to_string(),
                Verb::Fund => format!("fund {}{}", t(), n()),
            }
        }
        Op::Note { text } => if text.is_empty() { "#".to_string() } else { format!("# {text}") },
        Op::Unparsed { text } => text.clone(),
    }
}

/// Lowercase, one space between words.
fn norm(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// The id a name means among `rows` (id, display name): the display name or
/// the id itself, case-insensitive.
fn lookup<'a>(rows: impl Iterator<Item = (&'a String, &'a String)>, words: &str) -> Option<String> {
    let w = norm(words);
    if w.is_empty() {
        return None;
    }
    let mut rows = rows;
    rows.find(|(id, name)| norm(name) == w || id.to_lowercase() == w).map(|(id, _)| id.clone())
}

fn command(words: &str) -> Option<String> {
    lookup(content().commands.iter().map(|x| (&x.id, &x.name)), words)
}

fn building(words: &str) -> Result<String, String> {
    lookup(content().buildings.iter().map(|x| (&x.id, &x.name)), words).ok_or_else(|| format!("no building {:?}", words.trim()))
}

fn resource(words: &str) -> Result<String, String> {
    lookup(content().resources.iter().map(|x| (&x.id, &x.name)), words).ok_or_else(|| format!("no reading or resource {:?}", words.trim()))
}

/// A good of the pad verbs (`all`: none named).
fn good(words: &str) -> Result<Option<String>, String> {
    if norm(words) == "all" || words.trim().is_empty() {
        return Ok(None);
    }
    lookup(content().resources.iter().map(|x| (&x.id, &x.name)), words).map(Some).ok_or_else(|| format!("no good {:?}", words.trim()))
}

fn count(s: &str) -> Result<u32, String> {
    s.parse::<u32>().map_err(|_| format!("{s:?} isn't a whole number"))
}

/// A command line's count token: `N`, `xN`, `(xN)`, `×N` or `*N`.
fn count_token(s: &str) -> Option<u32> {
    let s = s.trim_start_matches('(').trim_end_matches(')');
    let s = s.strip_prefix(['x', 'X', '×', '*']).unwrap_or(s);
    s.parse().ok()
}

/// Splits a trailing amount (`N` or `all`) off `words`.
fn tail_amount(words: &[&str]) -> (Vec<String>, Option<Amount>) {
    match words.split_last() {
        Some((last, rest)) if *last == "all" => (rest.iter().map(|s| s.to_string()).collect(), Some(Amount::All)),
        Some((last, rest)) if last.parse::<u32>().is_ok() => (rest.iter().map(|s| s.to_string()).collect(), Some(Amount::N(last.parse().unwrap_or(0)))),
        _ => (words.iter().map(|s| s.to_string()).collect(), None),
    }
}

/// A program tab as written: `program 2`, `tab 2`, `p2` or `2` (1-based).
fn tab_words(words: &[&str]) -> Result<(u32, usize), String> {
    let (n, used) = match words {
        ["program" | "tab", n, ..] => (*n, 2),
        [n, ..] => (n.strip_prefix('p').unwrap_or(n), 1),
        [] => return Err("which program? (program N)".to_string()),
    };
    match n.parse::<u32>() {
        Ok(t) if t >= 1 => Ok((t - 1, used)),
        _ => Err(format!("{n:?} isn't a program number")),
    }
}

const COUNTED: [(&str, Reading); 6] = [
    ("free processors", Reading::FreeProcessors),
    ("free land", Reading::FreeLand),
    ("idle pads", Reading::IdlePads),
    ("energy per day", Reading::EnergyPerDay),
    ("days since a purchase", Reading::DaysSincePurchase),
    ("days since purchase", Reading::DaysSincePurchase),
];

/// One clause: `<reading> <op> <number>[%]`, `op` one of `>`, `<`, `>=`,
/// `<=` (`≥` and `≤` read as the last two).
fn clause(words: &[&str]) -> Result<Clause, String> {
    let cmp_of = |w: &str| match w {
        ">" => Some(Cmp::Gt),
        "<" => Some(Cmp::Lt),
        ">=" | "≥" => Some(Cmp::Ge),
        "<=" | "≤" => Some(Cmp::Le),
        _ => None,
    };
    let at = words.iter().position(|w| cmp_of(w).is_some()).ok_or("no comparison (>, <, >= or <=)")?;
    let cmp = cmp_of(words[at]).unwrap_or(Cmp::Gt);
    let what = words[..at].join(" ");
    let mut raw = words[at + 1..].join("");
    if raw.is_empty() {
        return Err(format!("no number after {}", words[at]));
    }
    let pct = raw.ends_with('%');
    if pct {
        raw.pop();
    }
    let value = Fx::parse(&raw).map_err(|_| format!("{raw:?} isn't a number"))?;
    let reading = match COUNTED.iter().find(|(w, _)| *w == what) {
        Some((w, _)) if pct => return Err(format!("{w} is a count, not a percent")),
        Some((_, r)) => r.clone(),
        None if what.is_empty() => return Err(format!("no reading before {}", words[at])),
        None if pct => Reading::Percent { resource: resource(&what)? },
        None => Reading::Amount { resource: resource(&what)? },
    };
    Ok(Clause { reading, cmp, value })
}

/// The words of a line, lowercase, with comparison signs spaced out.
fn words_of(s: &str) -> Vec<String> {
    let mut t = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '>' | '<' => {
                t.push(' ');
                t.push(ch);
                if chars.peek() == Some(&'=') {
                    t.push('=');
                    chars.next();
                }
                t.push(' ');
            }
            '≥' | '≤' => {
                t.push(' ');
                t.push(ch);
                t.push(' ');
            }
            _ => t.extend(ch.to_lowercase()),
        }
    }
    t.split_whitespace().map(str::to_string).collect()
}

/// `load <saved program> into program N`, the name taken exactly as written
/// (a saved program's name is free text: any case, spacing, `#` or sign);
/// `None` for a line of another form.
fn load_program(line: &str) -> Result<Option<Op>, String> {
    let body = line.trim();
    let Some((first, rest)) = body.split_once(char::is_whitespace) else {
        return Ok(None);
    };
    let rest = rest.trim_start();
    // ASCII lowercase keeps byte offsets.
    let Some(at) = (first.eq_ignore_ascii_case("load")).then(|| rest.to_ascii_lowercase().rfind(" into ")).flatten() else {
        return Ok(None);
    };
    let name = rest[..at].trim();
    let tail = words_of(&rest[at + " into ".len()..]);
    let tail: Vec<&str> = tail.iter().map(String::as_str).collect();
    let (t, used) = tab_words(&tail)?;
    if used != tail.len() {
        return Err("words after the program number".to_string());
    }
    if name.is_empty() {
        return Err("load which saved program?".to_string());
    }
    Ok(Some(Op::Act { verb: Verb::LoadProgram, target: Some(name.to_string()), tab: Some(t), amount: None }))
}

/// One line (its text without indent, not blank, not a `#` note) as an
/// instruction; `Err` says why it doesn't read.
pub fn parse_op(line: &str) -> Result<Op, String> {
    if let Some(op) = load_program(line)? {
        return Ok(op);
    }
    let owned = words_of(line);
    let w: Vec<&str> = owned.iter().map(String::as_str).collect();
    if w.iter().any(|x| x.starts_with('#')) {
        return Err("a comment goes on its own line".to_string());
    }
    if w.first() == Some(&"else") {
        return match w.len() {
            1 => Ok(Op::Else {}),
            _ if w[1] == "if" => Err("no else if: an else's body holds no if".to_string()),
            _ => Err("else stands alone: its lines go indented under it".to_string()),
        };
    }
    if w.first() == Some(&"if") {
        let rest = &w[1..];
        let joins: Vec<usize> = rest.iter().enumerate().filter(|(_, x)| **x == "and" || **x == "or").map(|(i, _)| i).collect();
        let when = match joins.as_slice() {
            [] => Condition { first: clause(rest)?, rest: None },
            [j] => Condition {
                first: clause(&rest[..*j])?,
                rest: Some(Joined { join: if rest[*j] == "and" { Join::And } else { Join::Or }, clause: clause(&rest[j + 1..])? }),
            },
            _ => return Err("at most one and/or".to_string()),
        };
        return Ok(Op::If { when });
    }
    // A command's name (or id), then with a trailing count.
    if let Some(c) = command(&w.join(" ")) {
        return Ok(Op::Command { command: c, count: 1 });
    }
    if let Some((n, rest)) = w.split_last().and_then(|(last, rest)| count_token(last).map(|n| (n, rest))) {
        let mut rest = rest.to_vec();
        // `cloud compute x 10`
        if matches!(rest.last(), Some(&("x" | "×" | "*"))) {
            rest.pop();
        }
        if let Some(c) = command(&rest.join(" ")) {
            if n == 0 {
                return Err("a command runs at least once".to_string());
            }
            return Ok(Op::Command { command: c, count: n });
        }
    }
    let act = |verb, target: Option<String>, tab: Option<u32>, amount: Option<Amount>| Op::Act { verb, target, tab, amount };
    let up_to = |rest: &[&str]| -> Result<(String, Option<u32>), String> {
        match rest {
            [b @ .., "up", "to", n] => Ok((b.join(" "), Some(count(n)?))),
            b => Ok((b.join(" "), None)),
        }
    };
    let op = match w.as_slice() {
        ["build", rest @ ..] => {
            let (b, n) = up_to(rest)?;
            Op::Build { building: building(&b)?, up_to: n }
        }
        ["buy", "land"] => Op::BuyLand {},
        ["launch"] => act(Verb::LaunchPads, None, None, None),
        ["launch", "pads", rest @ ..] => act(Verb::LaunchPads, good(&rest.join(" "))?, None, None),
        ["load", "pads", rest @ ..] => act(Verb::LoadPads, good(&rest.join(" "))?, None, None),
        ["set", "pads", rest @ ..] => {
            let (g, n) = up_to(rest)?;
            let (g, n) = match n {
                Some(n) => (g, n),
                None => {
                    let (g, a) = tail_amount(rest);
                    match a {
                        Some(Amount::N(n)) => (g.join(" "), n),
                        _ => return Err("set pads <good> up to N".to_string()),
                    }
                }
            };
            act(Verb::SetPads, Some(good(&g)?.ok_or("set pads names one good")?), None, Some(Amount::N(n)))
        }
        [v @ ("pause" | "resume"), rest @ ..] => {
            let (b, a) = tail_amount(rest);
            let verb = if *v == "pause" { Verb::Pause } else { Verb::Resume };
            act(verb, Some(building(&b.join(" "))?), None, a)
        }
        ["research", rest @ ..] => {
            let item = lookup(content().research.iter().map(|x| (&x.id, &x.name)), &rest.join(" "));
            act(Verb::Research, Some(item.ok_or_else(|| format!("no research {:?}", rest.join(" ")))?), None, None)
        }
        ["load", ..] => return Err("load <saved program> into program N (or load pads)".to_string()),
        ["assign", "processors", rest @ ..] => {
            let (t, used) = tab_words(rest)?;
            let (left, a) = tail_amount(&rest[used..]);
            if !left.is_empty() || a.is_none() {
                return Err("assign processors program N <count | all>".to_string());
            }
            act(Verb::AssignProcessors, None, Some(t), a)
        }
        ["retire"] => act(Verb::Retire, None, None, None),
        ["fund", rest @ ..] => {
            let (p, a) = tail_amount(rest);
            let p = lookup(content().projects.iter().map(|x| (&x.id, &x.name)), &p.join(" "));
            act(Verb::Fund, Some(p.ok_or_else(|| format!("no project {:?}", rest.join(" ")))?), None, a)
        }
        [first, ..] => return Err(format!("no command or verb {first:?}")),
        [] => return Err("an empty line".to_string()),
    };
    Ok(op)
}

/// How a text line stands: `ok`, `amber` (reads, but not unlocked yet this
/// run: kept, skipped free when run), `red` (doesn't read: kept as
/// an `unparsed` line), `note` or `blank` (dropped).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextLine {
    /// Its line in `entries` (none for a blank line).
    pub entry: Option<usize>,
    pub status: &'static str,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Parsed {
    /// The program the text makes: always one `set_program` takes on form
    /// (red lines are `unparsed`), unless it has too many lines.
    pub entries: Vec<Entry>,
    /// One per line of the text.
    pub lines: Vec<TextLine>,
}

fn clip(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// Reads a program's text, each line checked as `set_program` would and
/// for what this run has unlocked.
pub fn parse(r: &Rules, g: &GameState, text: &str) -> Parsed {
    let max = r.p.programs.line_text_max;
    let mut entries: Vec<Entry> = Vec::new();
    let mut lines = Vec::new();
    // The block the last line leaves open.
    let mut block = programs::Block::None;
    for raw in text.split('\n') {
        let body = raw.trim();
        if body.is_empty() {
            lines.push(TextLine { entry: None, status: "blank", reason: None });
            continue;
        }
        let indented = raw.starts_with([' ', '\t']) && block != programs::Block::None;
        let (op, status, reason) = if let Some(note) = body.strip_prefix('#') {
            (Op::Note { text: clip(note.trim(), max) }, "note", None)
        } else {
            let read = parse_op(body).and_then(|op| {
                let e = Entry { op, indented: raw.starts_with([' ', '\t']) };
                programs::check_line(r, g, &e, block).map(|_| e.op)
            });
            match read {
                Ok(op) => {
                    let why = programs::locked(r, g, &Entry { op: op.clone(), indented });
                    let status = if why.is_some() { "amber" } else { "ok" };
                    (op, status, why)
                }
                // An indent it can't hold stays in the text, so the line
                // reads back as it was written.
                Err(why) => {
                    let lead = if raw.starts_with([' ', '\t']) && !indented { "  " } else { "" };
                    (Op::Unparsed { text: clip(&format!("{lead}{body}"), max) }, "red", Some(why))
                }
            }
        };
        // Text past the most a line holds is cut: said, in red.
        let held = match &op {
            Op::Note { text } | Op::Unparsed { text } => text.chars().count(),
            _ => 0,
        };
        let written = body.strip_prefix('#').map_or(body.chars().count(), |n| n.trim().chars().count());
        let (status, reason) = if held == max && written > max {
            ("red", Some(reason.map_or(String::new(), |r| format!("{r}; ")) + &format!("cut to {max} characters")))
        } else {
            (status, reason)
        };
        lines.push(TextLine { entry: Some(entries.len()), status, reason });
        let e = Entry { op, indented };
        block = block.after(&e);
        entries.push(e);
    }
    // Lines past the most a program holds: red (set_program refuses the whole).
    let most = r.p.programs.max_entries;
    for l in lines.iter_mut().filter(|l| l.entry.is_some_and(|e| e >= most)) {
        l.status = "red";
        l.reason = Some(format!("at most {most} lines in a program"));
    }
    Parsed { entries, lines }
}

/// One entry of the text editor's instruction panel: a line as text, why it
/// won't run yet this run (amber, `programs::locked`), and the card a hover
/// shows (`command:<id>`, `building:<id>`, `research:<id>`, `project:<id>`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Instruction {
    pub text: String,
    pub locked: Option<String>,
    pub card: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InstructionGroup {
    pub name: &'static str,
    pub items: Vec<Instruction>,
}

/// The instruction panel's groups, only what the game has revealed (the
/// editor's lists: visible commands, buildings and research, open projects,
/// the library, the readings on offer). Number slots hold 1; `up to N` the
/// count owned + 1. `else` is listed only when an `else` line inserted
/// unindented below text line `cursor` of `text` reads where it stands.
pub fn instructions(r: &Rules, g: &GameState, text: &str, cursor: usize) -> Vec<InstructionGroup> {
    let v = r.view(g);
    let item = |op: Op, card: Option<String>| {
        let locked = programs::locked(r, g, &Entry::new(op.clone()));
        Instruction { text: op_text(&op), locked, card }
    };
    let act = |verb, target: Option<&str>, tab: Option<u32>, amount: Option<Amount>| Op::Act { verb, target: target.map(str::to_string), tab, amount };
    let one = Some(Amount::N(1));
    let targets = |verb: Verb| -> Vec<String> {
        v.verbs.iter().find(|x| x.verb == verb).and_then(|x| x.targets.as_ref()).map_or(Vec::new(), |t| t.iter().map(|c| c.value.clone()).collect())
    };
    let buildings: Vec<_> = v.buildings.iter().filter(|b| b.visible).collect();
    let mut groups = Vec::new();
    let mut group = |name: &'static str, items: Vec<Instruction>| {
        if !items.is_empty() {
            groups.push(InstructionGroup { name, items });
        }
    };

    group(
        "Commands",
        v.commands.iter().filter(|c| c.visible).map(|c| item(Op::Command { command: c.id.to_string(), count: 1 }, Some(format!("command:{}", c.id)))).collect(),
    );

    let mut build = Vec::new();
    for b in &buildings {
        let card = Some(format!("building:{}", b.id));
        build.push(item(Op::Build { building: b.id.to_string(), up_to: None }, card.clone()));
        build.push(item(Op::Build { building: b.id.to_string(), up_to: Some(b.owned + 1) }, card));
    }
    build.push(item(Op::BuyLand {}, None));
    group("Build", build);

    // The pad verbs once their commands are shown.
    let pad_card = |verb: Verb| {
        let id = v.commands.iter().find(|c| c.visible && programs::pad_command(verb).is_some_and(|p| p.id == c.id))?.id;
        Some(format!("command:{id}"))
    };
    let mut pads = Vec::new();
    for verb in [Verb::LoadPads, Verb::LaunchPads] {
        if let Some(card) = pad_card(verb) {
            pads.push(item(act(verb, None, None, None), Some(card)));
        }
    }
    if !pads.is_empty() {
        // Traded goods the player has seen.
        let seen = |good: &String| v.resources.iter().any(|x| x.visible && x.id == good.as_str());
        for good in targets(Verb::SetPads).into_iter().filter(seen) {
            let have = v.pads.iter().filter(|p| p.good.as_deref() == Some(good.as_str())).count() as u32;
            pads.push(item(act(Verb::SetPads, Some(&good), None, Some(Amount::N(have + 1))), None));
        }
    }
    group("Pads", pads);

    let mut onoff = Vec::new();
    for verb in [Verb::Pause, Verb::Resume] {
        for b in &buildings {
            onoff.push(item(act(verb, Some(b.id), None, one), Some(format!("building:{}", b.id))));
        }
    }
    group("Buildings on/off", onoff);

    let mut rp: Vec<Instruction> = targets(Verb::Research).iter().map(|t| item(act(Verb::Research, Some(t), None, None), Some(format!("research:{t}")))).collect();
    if v.verbs.iter().any(|x| x.verb == Verb::Fund) {
        rp.extend(targets(Verb::Fund).iter().map(|t| item(act(Verb::Fund, Some(t), None, one), Some(format!("project:{t}")))));
    }
    group("Research and projects", rp);

    let mut progs: Vec<Instruction> = targets(Verb::LoadProgram).iter().map(|t| item(act(Verb::LoadProgram, Some(t), Some(0), None), None)).collect();
    progs.push(item(act(Verb::AssignProcessors, None, Some(0), one), None));
    progs.push(item(act(Verb::Retire, None, None, None), None));
    group("Programs", progs);

    let zero = Fx::ZERO;
    let mut flow: Vec<Instruction> = v
        .readings
        .iter()
        .map(|o| item(Op::If { when: Condition { first: Clause { reading: o.reading.clone(), cmp: Cmp::Gt, value: zero }, rest: None } }, None))
        .collect();
    if else_fits(r, g, text, cursor) {
        flow.push(item(Op::Else {}, None));
    }
    flow.push(item(Op::Note { text: String::new() }, None));
    group("Control flow", flow);
    groups
}

/// Whether an `else` line inserted (unindented) below text line `cursor`
/// reads where it stands and leaves every other line as it read: at the end
/// of an `if`'s body (or below an `if` with none), never taking body lines
/// below the cursor into its own.
pub fn else_fits(r: &Rules, g: &GameState, text: &str, cursor: usize) -> bool {
    let mut lines: Vec<&str> = text.split('\n').collect();
    if cursor >= lines.len() {
        return false;
    }
    let next = lines[cursor + 1..].iter().find(|l| !l.trim().is_empty());
    if next.is_some_and(|l| l.starts_with([' ', '\t'])) {
        return false;
    }
    let red = |p: &Parsed| p.lines.iter().filter(|l| l.status == "red").count();
    let before = red(&parse(r, g, text));
    lines.insert(cursor + 1, "else");
    let after = parse(r, g, &lines.join("\n"));
    after.lines[cursor + 1].status != "red" && red(&after) == before
}

/// Why an `unparsed` line doesn't read now: `indented` as it is held, `block`
/// the one the line above leaves open.
pub fn unparsed_reason(r: &Rules, g: &GameState, text: &str, indented: bool, block: programs::Block) -> String {
    // A kept indent (one the line couldn't hold) is the reason.
    let indented = indented || text.starts_with([' ', '\t']);
    match parse_op(text.trim()).and_then(|op| programs::check_line(r, g, &Entry { op, indented }, block)) {
        Err(why) => why,
        Ok(()) => "reads now, where it stands: edit it to keep it".to_string(),
    }
}
