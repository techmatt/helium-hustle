//! 060 scratch harness: plays a loadout file as a sim player.
//! Run 1 (every run until `if` opens, then that one) is the author's, exactly
//! as `handsoff` plays it, so the runs after are paired with handsoff's.
//! From the next run on, at each reset it writes the loadout's programs and
//! seats processors (the loadout's `seat` line), and its one other hand is
//! the stale-run retirement (`stale` days with nothing bought; 0 = none).
//!
//! A program it can't load (`set_program` refused) panics: never a silent empty tab.
//!
//! `agent=SPEC` in place of a file plays a stock sim player (`agent=author`,
//! `agent=handsoff`) through the same loop: the same games as `hh-sim ladder`
//! (fingerprints checked equal), with the same `marks`.
//! Each games.jsonl line is the ladder's `GameResult` plus `marks`: per run, the
//! day the first pad, first ship, 2nd and 3rd Data Center, 5th ship and first
//! research held (watched every tick). Run from the repo root (it reads
//! configs/baseline.json and configs/sim/milestones.json).
//! `TRACE=<run> [TRACE_SEED=<seed>] [TRACE_EVERY=<days>]` prints that run's
//! resources, buildings and processors to stderr at its looks.
//!
//! usage: loadout-harness LOADOUT.txt --seeds N --first-seed S --retirements R --out games.jsonl [--max-ticks T] [--print]
//! (`--print` shows the parsed lines as `set_program` JSON entries.)

use std::collections::BTreeMap;
use std::io::Write as _;

use hh_core::state::{Amount, Clause, Cmp, Condition, Entry, Join, Joined, Op, Reading, Verb};
use hh_core::{Action, Config, Fx, Session};
use hh_sim::agents::author::Author;
use hh_sim::agents::{Agent, Knobs, Player};
use hh_sim::career::Meter;
use hh_sim::milestones::Milestones;

#[derive(Clone)]
struct Loadout {
    seat: Vec<(usize, Amount)>,
    stale: u64,
    programs: Vec<(usize, Vec<Entry>)>,
}

fn num(s: &str) -> Result<Fx, String> {
    Fx::parse(s.trim_end_matches('%'))
}

fn amount(s: &str) -> Result<Amount, String> {
    if s == "all" {
        Ok(Amount::All)
    } else {
        s.parse::<u32>().map(Amount::N).map_err(|_| format!("bad amount {s:?}"))
    }
}

fn clause(words: &[&str]) -> Result<Clause, String> {
    let at = words.iter().position(|w| *w == "above" || *w == "below").ok_or_else(|| format!("no above/below in {words:?}"))?;
    let cmp = if words[at] == "above" { Cmp::Gt } else { Cmp::Lt };
    let value = num(words.get(at + 1).ok_or("no number")?)?;
    let what = words[..at].join(" ");
    let pct = words[at + 1].ends_with('%');
    let reading = match what.as_str() {
        "free processors" => Reading::FreeProcessors,
        "free land" => Reading::FreeLand,
        "idle pads" => Reading::IdlePads,
        r if pct => Reading::Percent { resource: r.to_string() },
        r => Reading::Amount { resource: r.to_string() },
    };
    Ok(Clause { reading, cmp, value })
}

fn line(text: &str) -> Result<Entry, String> {
    let indented = text.starts_with(' ');
    let w: Vec<&str> = text.split_whitespace().collect();
    let act = |verb, target: Option<&str>, tab: Option<u32>, amount: Option<Amount>| Op::Act { verb, target: target.map(str::to_string), tab, amount };
    let op = match w.as_slice() {
        ["if", rest @ ..] => {
            let j = rest.iter().position(|x| *x == "and" || *x == "or");
            let (a, b) = match j {
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
        ["build", b, "up", "to", n] => Op::Build { building: b.to_string(), up_to: Some(n.parse().map_err(|_| "bad n")?) },
        ["buy", "land"] => Op::BuyLand {},
        ["launch", "pads", g] => act(Verb::LaunchPads, if *g == "all" { None } else { Some(g) }, None, None),
        ["load", "pads", g] => act(Verb::LoadPads, if *g == "all" { None } else { Some(g) }, None, None),
        ["set", "pads", g, "up", "to", n] => act(Verb::SetPads, Some(g), None, Some(amount(n)?)),
        ["pause", b] => act(Verb::Pause, Some(b), None, None),
        ["pause", b, n] => act(Verb::Pause, Some(b), None, Some(amount(n)?)),
        ["resume", b] => act(Verb::Resume, Some(b), None, None),
        ["resume", b, n] => act(Verb::Resume, Some(b), None, Some(amount(n)?)),
        ["research", r] => act(Verb::Research, Some(r), None, None),
        ["assign", "processors", t, n] => act(Verb::AssignProcessors, None, Some(t.parse::<u32>().map_err(|_| "bad tab")? - 1), Some(amount(n)?)),
        ["fund", p] => act(Verb::Fund, Some(p), None, None),
        ["fund", p, n] => act(Verb::Fund, Some(p), None, Some(amount(n)?)),
        ["retire"] => act(Verb::Retire, None, None, None),
        [c] => Op::Command { command: c.to_string(), count: 1 },
        [c, x] if x.starts_with('x') => Op::Command { command: c.to_string(), count: x[1..].parse().map_err(|_| "bad count")? },
        _ => return Err(format!("can't read {text:?}")),
    };
    Ok(Entry { op, indented })
}

fn parse(text: &str) -> Result<Loadout, String> {
    let mut l = Loadout { seat: Vec::new(), stale: 300, programs: Vec::new() };
    for raw in text.lines() {
        let t = raw.split('#').next().unwrap_or("").trim_end();
        if t.trim().is_empty() {
            continue;
        }
        if let Some(s) = t.strip_prefix("seat:") {
            for kv in s.split_whitespace() {
                let (k, v) = kv.split_once('=').ok_or("seat k=v")?;
                l.seat.push((k.parse::<usize>().map_err(|_| "seat tab")? - 1, amount(v)?));
            }
        } else if let Some(s) = t.strip_prefix("stale:") {
            l.stale = s.trim().parse().map_err(|_| "stale")?;
        } else if t.starts_with('[') {
            let n: usize = t.trim_matches(|c| c == '[' || c == ']').parse().map_err(|_| "tab")?;
            l.programs.push((n - 1, Vec::new()));
        } else {
            let p = l.programs.last_mut().ok_or("line before a [tab]")?;
            p.1.push(line(t)?);
        }
    }
    Ok(l)
}

struct LoadoutPlayer {
    author: Author,
    l: Loadout,
    first_run: Option<u32>,
    loaded_run: u32,
    bought: ((u32, Fx), u64),
}

impl LoadoutPlayer {
    fn hands_off(&self, run: u32) -> bool {
        self.first_run.is_some_and(|r| run != r)
    }
}

impl Agent for LoadoutPlayer {
    fn act(&mut self, p: &mut Player) {
        let v = p.view();
        let run = v.run;
        if v.if_open {
            self.first_run.get_or_insert(run);
        }
        if !self.hands_off(run) {
            self.author.act(p);
            return;
        }
        if self.loaded_run == run {
            if self.l.stale > 0 {
                let owned = v.buildings.iter().map(|b| b.owned).sum::<u32>() + v.research.iter().filter(|r| r.done).count() as u32;
                let have = (owned, v.land_price);
                if have != self.bought.0 {
                    self.bought = (have, v.day);
                } else if v.day.saturating_sub(self.bought.1) >= self.l.stale {
                    let _ = p.apply(&Action::Retire);
                }
            }
            return;
        }
        self.bought = ((u32::MAX, Fx::ZERO), v.day);
        self.loaded_run = run;
        let n = v.programs.len();
        for t in 0..n {
            let entries = self.l.programs.iter().find(|(x, _)| *x == t).map(|x| x.1.clone()).unwrap_or_default();
            // A refused program would leave the tab empty and the read meaningless.
            if let Err(e) = p.apply(&Action::SetProgram { program: t as u32, entries }) {
                panic!("run {run}: Program {} refused: {e}", t + 1);
            }
        }
        let v = p.view();
        let mut want = vec![0u32; n];
        let mut left = v.processors;
        for &(t, a) in &self.l.seat {
            let k = match a {
                Amount::All => left,
                Amount::N(k) => k.min(left),
            };
            want[t] += k;
            left -= k;
        }
        let cur: Vec<u32> = v.programs.iter().map(|p| p.processors).collect();
        for i in (0..n).filter(|&i| want[i] < cur[i]) {
            let _ = p.apply(&Action::SetProcessors { program: i as u32, processors: want[i] });
        }
        for i in (0..n).filter(|&i| want[i] > cur[i]) {
            let _ = p.apply(&Action::SetProcessors { program: i as u32, processors: want[i] });
        }
    }

    fn next_look(&mut self, p: &mut Player) -> u64 {
        let run = p.view().run;
        if self.hands_off(run) {
            10
        } else {
            self.author.next_look(p)
        }
    }
}

fn play(cfg: &Config, ms: &Milestones, l: &Loadout, name: &str, seed: u64, retirements: u32, max_ticks: u64) -> Result<String, String> {
    let start = std::time::Instant::now();
    let mut s = Session::new(cfg, seed, None)?;
    let mut author = Author::new(&Knobs(BTreeMap::new()));
    author.stop_at_retire = true;
    let mut agent: Box<dyn Agent> = match name.strip_prefix("agent=") {
        Some(spec) => hh_sim::agents::build(spec, seed, ms)?,
        None => Box::new(LoadoutPlayer { author, l: l.clone(), first_run: None, loaded_run: 0, bought: ((0, Fx::ZERO), 0) }),
    };
    // Per run: the day each key thing first held (watched every tick).
    let mut marks: BTreeMap<u32, BTreeMap<&'static str, u64>> = BTreeMap::new();
    let mut meter = Meter::new(&s, ms, &[]);
    let max_tick = s.state.tick.saturating_add(max_ticks);
    'career: while s.state.career.retirements < retirements && s.state.tick < max_tick {
        let (applied, gap) = {
            let mut p = Player::new(&mut s);
            agent.act(&mut p);
            let gap = agent.next_look(&mut p).max(1);
            (p.applied, gap)
        };
        meter.look(&s, applied);
        if let Ok(tr) = std::env::var("TRACE") {
            let v = s.rules.view(&s.state);
            let want: u32 = tr.parse().unwrap_or(0);
            let ts: u64 = std::env::var("TRACE_SEED").ok().and_then(|x| x.parse().ok()).unwrap_or(1);
            let every: u64 = std::env::var("TRACE_EVERY").ok().and_then(|x| x.parse().ok()).unwrap_or(50);
            if v.run == want && seed == ts && v.day % every < 10 {
                let res: Vec<String> = v.resources.iter().filter(|r| r.amount != Fx::ZERO).map(|r| format!("{}={}", r.id, r.amount.floor_int())).collect();
                let b: Vec<String> = v.buildings.iter().filter(|b| b.owned > 0).map(|b| format!("{}{}/{}", &b.id[..5.min(b.id.len())], b.active, b.owned)).collect();
                let pr: Vec<String> = v.programs.iter().map(|p| p.processors.to_string()).collect();
                eprintln!("d{} {} | {} | procs {}", v.day, res.join(" "), b.join(" "), pr.join(","));
            }
        }
        if s.state.career.retirements >= retirements {
            break;
        }
        let run = s.state.career.run_number;
        for _ in 0..gap {
            meter.tick(&mut s)?;
            {
                let g = &s.state;
                let m = marks.entry(g.career.run_number).or_default();
                let owned = |id: &str| g.run.buildings.get(id).map_or(0, |h| h.owned);
                let day = g.run.day;
                let checks: [(&'static str, bool); 6] = [
                    ("pad", owned("launch_pad") >= 1),
                    ("ship", g.run.stats.shipments >= 1),
                    ("dc2", owned("data_center") >= 2),
                    ("dc3", owned("data_center") >= 3),
                    ("ship5", g.run.stats.shipments >= 5),
                    ("research", g.run.stats.research_completed >= 1),
                ];
                for (k, ok) in checks {
                    if ok {
                        m.entry(k).or_insert(day);
                    }
                }
            }
            if s.state.career.retirements >= retirements || s.state.tick >= max_tick {
                break 'career;
            }
            if s.state.career.run_number != run {
                break;
            }
        }
    }
    let r = meter.finish(&s, seed, name, Vec::new(), start.elapsed().as_secs_f64());
    let mut j = serde_json::to_value(&r).map_err(|e| e.to_string())?;
    j["marks"] = serde_json::to_value(&marks).map_err(|e| e.to_string())?;
    Ok(j.to_string())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = &args[0];
    let get = |k: &str, d: u64| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).map_or(d, |v| v.parse().expect(k));
    let out = args.iter().position(|a| a == "--out").map(|i| args[i + 1].clone()).expect("--out");
    let (seeds, first, ret, max) = (get("--seeds", 3), get("--first-seed", 1), get("--retirements", 10) as u32, get("--max-ticks", 60000));
    let text = if path.starts_with("agent=") { String::new() } else { std::fs::read_to_string(path).expect("loadout") };
    let l = parse(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
    if args.iter().any(|a| a == "--print") {
        for (t, e) in &l.programs {
            println!("[{}] {}", t + 1, serde_json::to_string(e).unwrap());
        }
        return;
    }
    let name = if path.starts_with("agent=") { path.clone() } else { std::path::Path::new(path).file_stem().unwrap().to_string_lossy().to_string() };
    let cfg = Config::from_json(&std::fs::read_to_string("configs/baseline.json").unwrap()).unwrap();
    let ms = Milestones::from_json(&std::fs::read_to_string("configs/sim/milestones.json").unwrap()).unwrap();
    let results: Vec<_> = std::thread::scope(|sc| {
        let hs: Vec<_> = (first..first + seeds).map(|seed| {
            let (cfg, ms, l, name) = (&cfg, &ms, &l, &name);
            sc.spawn(move || play(cfg, ms, l, name, seed, ret, max))
        }).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut f = std::fs::File::create(&out).unwrap();
    for r in results {
        writeln!(f, "{}", r.expect("game")).unwrap();
    }
}
