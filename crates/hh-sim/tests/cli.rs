use hh_sim::{run, Fail};

// As main.rs: games play about twice as fast on mimalloc.
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn baseline() -> String {
    format!("{}/../../configs/baseline.json", env!("CARGO_MANIFEST_DIR"))
}

fn args(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| s.to_string()).collect()
}

#[test]
fn run_prints_the_state_and_checks_replay() {
    let c = baseline();
    let out = run(&args(&["run", "--config", &c, "--seed", "1", "--ticks", "365", "--policy", "greedy:every=7"])).unwrap();
    assert!(out.contains("day 365") && out.contains("Solar Panel") && out.contains("replay ok"), "{out}");
}

#[test]
fn a_saved_record_replays() {
    let c = baseline();
    let dir = format!("{}/../../out/hh-sim-tests", env!("CARGO_MANIFEST_DIR"));
    std::fs::create_dir_all(&dir).unwrap();
    let rec = format!("{dir}/scenario-record.json");
    run(&args(&["run", "--config", &c, "--scenario", "rich", "--ticks", "50", "--policy", "greedy", "--save", &rec])).unwrap();
    let out = run(&args(&["replay", &rec])).unwrap();
    assert!(out.contains("tick 50"), "{out}");
    // --summary: the end state as `run` prints it.
    let out = run(&args(&["replay", "--summary", &rec])).unwrap();
    for want in ["tick 50, run 1 day ", "  Energy ", "career log (tick, run, day):"] {
        assert!(out.contains(want), "{want}: {out}");
    }
}

#[test]
fn json_output_parses() {
    let c = baseline();
    let out = run(&args(&["run", "--config", &c, "--ticks", "10", "--json"])).unwrap();
    let v: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(v["replay_ok"], true);
    assert_eq!(v["state"]["tick"], 10);
}

#[test]
fn objectives_play_the_reference_scenario_to_a_forced_retirement() {
    let c = baseline();
    let sc = format!("{}/../../reference/scenarios/run1_fresh.json", env!("CARGO_MANIFEST_DIR"));
    let out = run(&args(&["objectives", &sc, "--config", &c, "--seed", "1"])).unwrap();
    for want in [
        "first_ship",
        "quest qpower_boot_sequence",
        "quest qfirst_shipment",
        "forced_retirement",
    ] {
        assert!(out.contains(want), "{want}: {out}");
    }
}

#[test]
fn bad_input_is_a_usage_error() {
    for a in [
        vec!["run", "--bogus"],
        vec!["run", "--seed", "-1"],
        vec!["run", "--policy", "smart"],
        vec!["run", "--scenario", "nope"],
        vec!["objectives"],
        // An option the command doesn't read is refused, not ignored.
        vec!["golden", "--config", "configs/baseline.json"],
        vec!["replay", "--json", "x.json"],
        vec!["objectives", "x.json", "--policy", "greedy"],
        vec!["frobnicate"],
        vec![],
    ] {
        assert!(matches!(run(&args(&a)), Err(Fail::Usage(_))), "{a:?}");
    }
}

fn root(p: &str) -> String {
    format!("{}/../../{p}", env!("CARGO_MANIFEST_DIR"))
}

/// A small grid: the given players, the mixed human model, 2 seeds, careers
/// of 2 retirements.
fn grid(cmd: &str, agents: &str, extra: &[&str]) -> Vec<String> {
    let (spec, humans, milestones, config) = (
        root("experiments/balance.json"),
        root("configs/sim/humans.json"),
        root("configs/sim/milestones.json"),
        baseline(),
    );
    let mut a = vec![
        cmd.to_string(),
        spec,
        "--human-models".into(),
        humans,
        "--milestones".into(),
        milestones,
        "--config".into(),
        config,
        "--agents".into(),
        agents.into(),
        "--humans".into(),
        "mixed".into(),
        "--seeds".into(),
        "2".into(),
        "--retirements".into(),
        "2".into(),
    ];
    a.extend(extra.iter().map(|s| s.to_string()));
    a
}

/// The ladder's players' games on the small grid (seeds 1-2, careers of 2
/// retirements), pinned: an agent edit that should play the same moves
/// (a refactor) keeps these, as the core's `golden_game` does for the rules.
/// One that changes play updates them and says so; a rules change moves them
/// all. Copy them from the failing assertion (or the test's `games.jsonl`).
/// (The handsoff rows play only the author's runs: [`HANDSOFF_PIN`] pins its loadout.)
const PINNED: &[(&str, u64, &str)] = &[
    ("random", 1, "7a2fa27df2820e10"),
    ("random", 2, "1ae24b2d20817bc0"),
    ("greedy", 1, "e0fe1a489f42eeac"),
    ("greedy", 2, "603e13a782725fc5"),
    ("author", 1, "a958368ff4cab92e"),
    ("author", 2, "33132b80f8c4b022"),
    ("casual", 1, "d73f016d8fb6c470"),
    ("casual", 2, "40ba31f5c5e6317b"),
    ("handsoff", 1, "580f2ac9f0f3cdc4"),
    ("handsoff", 2, "77d3c25a8b2319d0"),
];

#[test]
fn the_ladder_prints_its_tables_and_writes_results() {
    let dir = root("out/hh-sim-tests/ladder");
    let players = "random,greedy,author,casual=author:min_gap=120,handsoff";
    let out = run(&grid("experiment", players, &["--out", &dir, "--threads", "3"])).unwrap();
    for want in [
        "## Pace in game time",
        "## Milestones",
        "## Real time",
        "## Hands-off",
        "| handsoff |",
        "## Never built",
        "| author |",
        "| greedy |",
        "| random |",
        "| casual |",
        "| author | mixed |",
        "M4",
    ] {
        assert!(out.contains(want), "{want}: {out}");
    }
    let games = std::fs::read_to_string(format!("{dir}/games.jsonl")).unwrap();
    assert_eq!(games.lines().count(), 10);
    let played: Vec<(String, u64, String)> = games
        .lines()
        .map(|l| {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            (v["cell"]["agent"].as_str().unwrap().into(), v["seed"].as_u64().unwrap(), v["fingerprint"].as_str().unwrap().into())
        })
        .collect();
    let pinned: Vec<(String, u64, String)> = PINNED.iter().map(|&(a, s, f)| (a.into(), s, f.into())).collect();
    assert_eq!(played, pinned, "the ladder's players played other games (PINNED's doc says what to do)");
    for line in games.lines() {
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(v["replay_ok"], true, "{line}");
        assert_eq!(v["retirements"], 2);
        // Players skip stretches that need no decision.
        let (ticks, looks, decisions) =
            (v["ticks"].as_u64().unwrap(), v["looks"].as_u64().unwrap(), v["decisions"].as_u64().unwrap());
        assert!(decisions > 0 && decisions <= looks && looks * 2 < ticks, "{ticks} {looks} {decisions}");
        let runs = v["runs"].as_array().unwrap();
        let per_run: u64 = runs.iter().map(|r| r["decisions"].as_u64().unwrap()).sum();
        assert!(per_run <= decisions, "{per_run} {decisions}");
        if v["cell"]["agent"] == "casual" {
            assert!(ticks / looks >= 60, "the casual author looks at most every 120 ticks: {ticks} {looks}");
        }
    }
    // `summarize` reads it back: each game's runs, milestones and career.
    let ms = root("configs/sim/milestones.json");
    let s = run(&args(&["summarize", &format!("{dir}/games.jsonl"), "--milestones", &ms])).unwrap();
    for want in ["## Runs (10 games", "| author | 1 | 1 |", "| casual | 2 | 2 |","## Milestones", "| M1 |", "## Career at the end"] {
        assert!(s.contains(want), "{want}: {s}");
    }
}

/// The hands-off player's own play, pinned: the small grid's careers end
/// before it loads its loadout (`if` opens in run 3 or 4 on seeds 1-2, so
/// PINNED's handsoff games are the author's), so this one plays 5 runs on
/// seed 1 and checks the run after Conditional Logic's, where the loadout
/// file plays, ships and buys. A new loadout file moves the pin: copy it from
/// the failing assertion.
const HANDSOFF_PIN: &str = "a82d4a3551a27008";

#[test]
#[ignore = "slow lane: a 5-run career (~7 s)"]
fn the_handsoff_loadout_ships_and_buys_after_if_opens() {
    let dir = root("out/hh-sim-tests/handsoff");
    let mut a = grid("experiment", "handsoff", &["--out", &dir]);
    for (k, v) in [("--seeds", "1"), ("--retirements", "5")] {
        let i = a.iter().position(|x| x == k).unwrap();
        a[i + 1] = v.into();
    }
    run(&a).unwrap();
    let games = std::fs::read_to_string(format!("{dir}/games.jsonl")).unwrap();
    let v: serde_json::Value = serde_json::from_str(games.lines().next().unwrap()).unwrap();
    let runs = v["runs"].as_array().unwrap();
    let opened = runs
        .iter()
        .position(|r| r["projects"].as_array().unwrap().iter().any(|p| p == "conditional_logic"))
        .expect("`if` opens within 5 runs");
    let loaded = runs.get(opened + 1).expect("a run after `if` opens finished");
    assert!(loaded["shipments"].as_u64().unwrap() > 0 && loaded["buildings_built"].as_u64().unwrap() > 0, "{loaded}");
    assert_eq!(v["replay_ok"], true);
    assert_eq!(v["fingerprint"], HANDSOFF_PIN, "the hands-off player played another game (HANDSOFF_PIN's doc)");
}

#[test]
#[ignore = "slow lane: the grid under two human models (~7 s)"]
fn real_time_is_a_projection_that_changes_no_game() {
    let fp = |humans: &str| {
        let dir = root(&format!("out/hh-sim-tests/humans-{}", humans.replace(',', "-")));
        let mut a = grid("experiment", "author", &["--out", &dir]);
        let i = a.iter().position(|x| x == "--humans").unwrap();
        a[i + 1] = humans.into();
        let out = run(&a).unwrap();
        let games = std::fs::read_to_string(format!("{dir}/games.jsonl")).unwrap();
        let fps: Vec<String> = games
            .lines()
            .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap()["fingerprint"].to_string())
            .collect();
        (out, fps)
    };
    let (slow, a) = fp("careful");
    let (fast, b) = fp("brisk");
    assert_eq!(a, b);
    let run1 = |out: &str, h: &str| -> f64 {
        let line = out.lines().find(|l| l.starts_with(&format!("| author | {h} |"))).unwrap();
        line.split('|').nth(3).unwrap().trim().parse().unwrap()
    };
    assert!(run1(&slow, "careful") > run1(&fast, "brisk"), "{slow}\n{fast}");
}

#[test]
#[ignore = "slow lane: the grid at two thread counts (~10 s)"]
fn a_game_is_the_same_whatever_the_thread_count() {
    let fp = |threads: &str| {
        let dir = root(&format!("out/hh-sim-tests/threads{threads}"));
        run(&grid("experiment", "author,random", &["--out", &dir, "--threads", threads])).unwrap();
        let games = std::fs::read_to_string(format!("{dir}/games.jsonl")).unwrap();
        games
            .lines()
            .map(|l| {
                let v: serde_json::Value = serde_json::from_str(l).unwrap();
                format!("{} {} {}", v["agent"], v["seed"], v["fingerprint"])
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(fp("1"), fp("4"));
}

#[test]
fn compare_pairs_two_configs_on_the_same_seeds() {
    let out = run(&grid("compare", "author", &["--set", "boredom.base_cap=250"])).unwrap();
    assert!(out.contains("## Paired") && out.contains("run 1 ticks") && out.contains("# Side B"), "{out}");
    // A shorter boredom cap ends run 1 sooner.
    let line = out.lines().find(|l| l.contains("| run 1 ticks |")).unwrap();
    assert!(line.contains("| -"), "{line}");
}

#[test]
fn balance_prints_only_balance_lines() {
    let out = run(&grid("balance", "author", &[])).unwrap();
    assert!(!out.is_empty() && out.lines().all(|l| l.starts_with("balance: ")), "{out}");
}

/// A judge's spec with no human model says run 1's real time went unjudged
/// (balance.json's `humans` is the model's one home), never drops it silently.
#[test]
fn balance_without_a_human_model_says_so() {
    let dir = root("out/hh-sim-tests/no-humans");
    std::fs::create_dir_all(&dir).unwrap();
    let mut spec: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root("experiments/balance.json")).unwrap()).unwrap();
    spec.as_object_mut().unwrap().remove("humans");
    spec["configs"] = serde_json::json!([baseline()]);
    let path = format!("{dir}/spec.json");
    std::fs::write(&path, spec.to_string()).unwrap();
    let mut a = grid("balance", "author", &[]);
    a[1] = path;
    let at = a.iter().position(|x| x == "--humans").unwrap();
    a.drain(at..at + 2);
    let out = run(&a).unwrap();
    assert!(out.contains("balance: run 1's real time not judged: the spec names no human model, author"), "{out}");
}

#[test]
fn bad_grids_are_usage_errors() {
    for (agents, extra) in [
        ("smart", vec![]),
        ("planner", vec![]),
        ("author:bogus=1", vec![]),
        ("author", vec!["--humans", "warp"]),
        ("a=author,a=greedy", vec![]),
        ("author", vec!["--seeds", "0"]),
        ("author", vec!["--set", "boredom.nope=1"]),
    ] {
        let mut a = grid("experiment", agents, &[]);
        a.extend(extra.iter().map(|s| s.to_string()));
        if a.iter().any(|s| s == "--set") {
            a[0] = "compare".into();
        }
        assert!(matches!(run(&a), Err(Fail::Usage(_))), "{a:?}");
    }
}

#[test]
#[ignore = "slow lane: the grid with knobs and one program (~4 s)"]
fn agents_keep_their_knobs_and_cope_with_one_program() {
    let dir = root("out/hh-sim-tests/knobs");
    let out = run(&grid(
        "experiment",
        "author:hand_gap=0,loads=10,greedy",
        &["--out", &dir, "--set", "programs.program_count=1", "--set", "economy.upkeep_epsilon=0.000001"],
    ))
    .unwrap();
    assert!(out.contains("| author:hand_gap=0,loads=10 |") && out.contains("| greedy |"), "{out}");
}

/// The oracle plans on copies of the game: its own game's record replays, a
/// second play is the same game, a strategy pinned by its row in the table
/// (trade) and its knobs parse, and both play the pinned games (an oracle
/// refactor keeps them; see `PINNED`).
#[test]
#[ignore = "slow lane: an oracle career plays thousands of rollouts (~25 s)"]
fn the_oracle_plays_on_copies_and_its_game_replays() {
    let dir = root("out/hh-sim-tests/oracle");
    let player = "oracle:horizon=90,long=200,every=200,replan=1000,width=2";
    let trade = hh_sim::agents::oracle::strategy_index("trade").unwrap();
    let mut fingerprints = Vec::new();
    for threads in ["1", "2"] {
        let a = grid("experiment", &format!("{player},pinned=oracle:strategy={trade},horizon=60,long=100,every=400"), &[
            "--out", &dir, "--threads", threads,
        ]);
        let out = run(&a).unwrap();
        assert!(out.contains(&format!("| {player} |")) && out.contains("| pinned |"), "{out}");
        let games = std::fs::read_to_string(format!("{dir}/games.jsonl")).unwrap();
        for line in games.lines() {
            let g: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(g["replay_ok"], true, "{line}");
            fingerprints.push(g["fingerprint"].as_str().unwrap().to_string());
        }
    }
    let half = fingerprints.len() / 2;
    assert_eq!(fingerprints[..half], fingerprints[half..]);
    // Seeds 1 and 2 of the oracle, then of the pinned one.
    assert_eq!(fingerprints[..half], ["450f3a9032bb2db6", "308b8a2ad73bc5a0", "034cc48852f98d0b", "ba99f22b07fb64bb"]);
}

/// `--agents` splits players, not knobs: a knob's value may hold a `:` (a
/// drive path for `handsoff`'s `loadout`).
#[test]
fn the_agent_list_keeps_knobs_with_their_player() {
    let l = |s: &str| hh_sim::lab::agent_list(s);
    assert_eq!(l("author:a=1,b=2,greedy"), ["author:a=1,b=2", "greedy"]);
    assert_eq!(l("greedy,casual=author:min_gap=120"), ["greedy", "casual=author:min_gap=120"]);
    assert_eq!(l("h=handsoff:every=5,loadout=C:/x/a.txt,author"), ["h=handsoff:every=5,loadout=C:/x/a.txt", "author"]);
    assert_eq!(l("a=author:fill=4,b=author:fill=8"), ["a=author:fill=4", "b=author:fill=8"]);
}

/// `gates` lists each quest with what it opens.
#[test]
fn gates_lists_what_each_quest_opens() {
    let out = run(&["gates".to_string()]).unwrap();
    assert!(out.contains("on completion: "), "{out}");
    assert!(out.contains("research shown once done: "), "{out}");
    assert!(out.lines().any(|l| l.contains("research revealed this run: ") && l.contains("regolith_processing")), "{out}");
    assert!(matches!(run(&["gates".to_string(), "x".to_string()]), Err(Fail::Usage(_))));
}

#[test]
fn help_prints_the_usage() {
    for h in ["--help", "-h", "help"] {
        assert!(run(&[h.to_string()]).unwrap().starts_with("usage:"));
    }
}

/// `trace --at T` prints the state at career tick T: what walls a run in.
#[test]
fn trace_at_prints_the_state_at_a_tick() {
    let out = run(&grid("trace", "author", &["--at", "300"])).unwrap();
    for want in ["tick 300 run 1 day ","solar_panel", "  program 1 procs ", "      1. ", "  pad 1 good ", "  # target ", "career: retirements 0"] {
        assert!(out.contains(want), "{want}: {out}");
    }
    assert!(matches!(run(&grid("experiment", "author", &["--at", "300"])), Err(Fail::Usage(_))));
}

/// Every standing spec (`experiments/*.json`: ladder, ceiling, balance) loads
/// and builds its players; the ladder keeps the oracle out (ceiling.json's).
#[test]
fn every_standing_spec_loads() {
    let read = |p: &str| std::fs::read_to_string(root(p)).map_err(|e| format!("{p}: {e}"));
    let (humans, ms) = (read("configs/sim/humans.json").unwrap(), read("configs/sim/milestones.json").unwrap());
    let mut names = Vec::new();
    for entry in std::fs::read_dir(root("experiments")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|x| x == "json") {
            let spec = hh_sim::experiment::Spec::from_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
            let l = hh_sim::experiment::load(spec, &humans, &ms, &read).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            if l.spec.name == "ladder" {
                assert!(l.spec.agents.iter().all(|a| hh_sim::agents::kind(a) != "oracle"), "{:?}", l.spec.agents);
            }
            names.push(l.spec.name);
        }
    }
    names.sort();
    assert_eq!(names, ["balance", "ceiling", "ladder"]);
}

/// A strategy pinned past the table's end is refused.
#[test]
fn the_oracle_refuses_a_strategy_past_its_table() {
    let rows = hh_sim::agents::oracle::strategies().len();
    assert!(matches!(run(&grid("experiment", &format!("oracle:strategy={rows}"), &[])), Err(Fail::Usage(_))));
}

#[test]
fn the_oracle_refuses_an_unknown_knob() {
    assert!(matches!(run(&grid("experiment", "oracle:peek=1", &[])), Err(Fail::Usage(_))));
}

#[test]
fn empty_lists_and_stray_options_are_usage_errors() {
    for extra in [
        vec!["--humans", "warp,mixed"],
        vec!["--agents", ""],
        vec!["--first-seed", "18446744073709551615"],
    ] {
        let mut a = grid("balance", "author", &[]);
        a.extend(extra.iter().map(|s| s.to_string()));
        assert!(matches!(run(&a), Err(Fail::Usage(_))), "{a:?}");
    }
    let mut a = grid("trace", "author", &[]);
    a.extend(["--out".to_string(), "x".to_string()]);
    assert!(matches!(run(&a), Err(Fail::Usage(_))), "{a:?}");
    let mut a = grid("compare", "author", &[]);
    a.push("--lines".to_string());
    assert!(matches!(run(&a), Err(Fail::Usage(_))), "{a:?}");
    for cmd in ["agents", "speed"] {
        let a = args(&[cmd, "--threads", "2"]);
        assert!(matches!(run(&a), Err(Fail::Usage(_))), "{a:?}");
    }
    let mut a = grid("balance", "author", &[]);
    a.push("--lines".into());
    assert!(matches!(run(&a), Err(Fail::Usage(_))), "{a:?}");
}

/// `trace --lines` shows a program's lines run in the order queued, day after
/// day: counts 2, 1, 3 on one processor (more processors: hh-core's
/// `programs_run_and_show_their_lines_in_queue_order_day_after_day`).
#[test]
fn trace_lines_show_the_queue_in_order() {
    use hh_core::{state::Entry, Action, Config, Session};
    let c = Config::from_json(&std::fs::read_to_string(baseline()).unwrap()).unwrap();
    let mut s = Session::new(&c, 3, None).unwrap();
    let e = Entry::command;
    let entries = vec![e("idle", 2), e("cloud_compute", 1), e("idle", 3)];
    s.apply(&Action::SetProgram { program: 0, entries }).unwrap();
    s.advance(7);
    let days = hh_sim::lab::day_lines(&s.record()).unwrap();
    let got: Vec<String> = days.iter().map(|(t, d)| format!("{t} {d}")).collect();
    let want = [
        "1 lines p0: 0 idle ran",
        "2 lines p0: 0 idle ran",
        "3 lines p0: 1 cloud_compute ran",
        "4 lines p0: 2 idle ran",
        "5 lines p0: 2 idle ran",
        "6 lines p0: 2 idle ran",
        "7 lines p0: 0 idle ran",
    ];
    assert_eq!(got, want);
}

/// Agents decide in integers and `Fx` with ordered maps only, so a decision
/// can't differ between native and wasm builds (docs/agents.md).
#[test]
fn agents_use_no_floats_or_hash_maps() {
    // Every file under `agents/`, subfolders (`oracle/`) included, and
    // `milestones.rs`, whose `Tracker` the oracle's decisions run through.
    let src = format!("{}/src", env!("CARGO_MANIFEST_DIR"));
    let mut dirs = vec![std::path::PathBuf::from(format!("{src}/agents"))];
    let tracker = std::fs::read_to_string(format!("{src}/milestones.rs")).unwrap();
    for banned in ["f64", "f32", "HashMap", "HashSet", "to_f64", "from_f64"] {
        assert!(!tracker.contains(banned), "milestones.rs uses {banned}");
    }
    let mut files = 0;
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            files += 1;
            let text = std::fs::read_to_string(&path).unwrap();
            for banned in ["f64", "f32", "HashMap", "HashSet", "to_f64", "from_f64"] {
                assert!(!text.contains(banned), "{} uses {banned}", path.display());
            }
        }
    }
    assert!(files >= 9, "only {files} agent files read");
}

#[test]
fn a_stockpile_milestone_on_a_capped_resource_is_refused() {
    let m = std::fs::read_to_string(root("configs/sim/milestones.json")).unwrap();
    let bad = m.replace("\"resource\": \"credits\"", "\"resource\": \"he3\"");
    assert!(hh_sim::milestones::Milestones::from_json(&bad).unwrap_err().contains("capped"));
    assert!(hh_sim::milestones::Milestones::from_json(&m).is_ok());
}

/// A play log as the web client writes one: a scripted career at 10x with a
/// minute's pause, its actions, and a closing checkpoint.
fn write_playlog(path: &str, tamper: bool) {
    use hh_core::{driver, Config, Session};
    let c = Config::from_json(&std::fs::read_to_string(baseline()).unwrap()).unwrap();
    let mut s = Session::new(&c, 7, None).unwrap();
    driver::play(&mut s, 600, driver::Policy::Script);
    write_log_of(path, &c, &s, tamper);
}

/// The play log of a played session (seed 7, 600 ticks).
fn write_log_of(path: &str, c: &hh_core::Config, s: &hh_core::Session, tamper: bool) {
    use hh_core::RULES_REVISION;
    let rec = s.record();
    assert_eq!(s.state.tick, 600);
    // 10 ticks a second, with a 60 s pause at tick 300.
    let t = |tick: u64| tick as f64 * 100.0 + if tick >= 300 { 60_000.0 } else { 0.0 };
    let mut lines = vec![serde_json::json!({
        "kind": "header", "format": "hh-playlog/1", "career": "test", "rules_revision": RULES_REVISION,
        "config_hash": c.hash(), "engine": "test", "config": c, "seed": "7", "scenario": null,
        "prior_actions": [], "t": 0.0, "commit": "test",
    })];
    let at = |kind: &str, tick: u64, ms: f64, speed: u32| {
        serde_json::json!({"t": ms, "kind": kind, "tick": tick, "run": 1, "day": tick, "speed": speed})
    };
    lines.push(at("session_start", 0, 0.0, 10));
    let mut paused = false;
    for a in &rec.actions {
        if a.tick >= 300 && !paused {
            lines.push(at("speed", 300, t(300) - 60_000.0, 0));
            lines.push(at("speed", 300, t(300), 10));
            paused = true;
        }
        let mut l = at("action", a.tick, t(a.tick), 10);
        l["action"] = serde_json::to_value(&a.action).unwrap();
        lines.push(l);
    }
    let mut end = at("checkpoint", 600, t(600), 10);
    let fp = s.state.fingerprint();
    end["fingerprint"] = serde_json::json!(if tamper { "0000000000000000".to_string() } else { fp });
    lines.push(end);
    let text: String = lines.iter().map(|l| format!("{l}\n")).collect();
    std::fs::write(path, text).unwrap();
}

#[test]
fn a_play_log_replays_to_its_final_state_and_measures_real_time() {
    let dir = root("out/hh-sim-tests");
    std::fs::create_dir_all(&dir).unwrap();
    let path = format!("{dir}/play.jsonl");
    write_playlog(&path, false);
    let (ms, pf) = (root("configs/sim/milestones.json"), root("configs/sim/humans.json"));
    let out = run(&args(&["playlog", &path, "--milestones", &ms, "--human-models", &pf])).unwrap();
    assert!(out.contains("final state reproduced") && out.contains("## Milestones") && out.contains("| paused |"), "{out}");
    let j = run(&args(&["playlog", &path, "--json", "--milestones", &ms, "--human-models", &pf])).unwrap();
    let v: serde_json::Value = serde_json::from_str(j.trim()).unwrap();
    assert_eq!(v["reproduced"], true);
    assert_eq!(v["game"]["ticks"], 600);
    let paused = v["time"]["at_speed_s"]["0"].as_f64().unwrap();
    assert!((paused - 60.0).abs() < 1e-6, "{paused}");
    // 600 ticks at 10x plus the pause, all played.
    let real = v["game"]["real_s"].as_f64().unwrap();
    assert!((real - 120.0).abs() < 1e-6, "{real}");
    // The first batch sent twice (the page saw the send fail): its header repeats.
    let resent = format!("{dir}/play-resent.jsonl");
    write_playlog(&resent, false);
    let text = std::fs::read_to_string(&resent).unwrap();
    let mut head: serde_json::Value = serde_json::from_str(text.lines().next().unwrap()).unwrap();
    head["id"] = serde_json::json!("p:0");
    let rest: Vec<&str> = text.lines().skip(1).collect();
    std::fs::write(&resent, format!("{head}\n{}\n{head}\n", rest.join("\n"))).unwrap();
    let out = run(&args(&["playlog", &resent, "--milestones", &ms, "--human-models", &pf])).unwrap();
    assert!(out.contains("final state reproduced"), "{out}");
    let bad = format!("{dir}/play-bad.jsonl");
    write_playlog(&bad, true);
    assert!(matches!(run(&args(&["playlog", &bad, "--milestones", &ms, "--human-models", &pf])), Err(Fail::Check(_))));
    // An action after the last checkpoint, at its tick: the fingerprint no longer
    // stands for the end, so nothing is checked (not a mismatch); a line written at a
    // later visit (`offline`) takes no time.
    let late = format!("{dir}/play-late.jsonl");
    write_playlog(&late, false);
    let mut text = std::fs::read_to_string(&late).unwrap();
    // The same line twice (a resend): read once.
    let dup = serde_json::json!({"id": "p:1", "t": 999_998.0, "kind": "action", "tick": 600, "run": 1, "day": 600,
        "speed": 10, "action": {"type": "cheat", "cheat": {"kind": "grant", "resource": "credits", "amount": "1"}}});
    text.push_str(&format!("{dup}\n{dup}\n"));
    text.push_str(&format!(
        "{}\n{}\n",
        serde_json::json!({"t": 999_999.0, "kind": "action", "tick": 600, "run": 1, "day": 600, "speed": 10,
            "action": {"type": "set_program", "program": 4, "entries": []}}),
        serde_json::json!({"t": 9.9e12, "kind": "career_end", "tick": 0, "run": 0, "day": 0, "speed": 0,
            "reason": "save discarded", "offline": true})
    ));
    std::fs::write(&late, text).unwrap();
    let actions = v["game"]["actions"].as_u64().unwrap();
    let j = run(&args(&["playlog", &late, "--json", "--milestones", &ms, "--human-models", &pf])).unwrap();
    let v: serde_json::Value = serde_json::from_str(j.trim()).unwrap();
    assert_eq!(v["reproduced"], serde_json::Value::Null);
    assert_eq!(v["game"]["actions"].as_u64().unwrap(), actions + 2, "the repeated line is read once");
    // Time at the last tick (the actions there, 880 s on) is played; the
    // offline line adds none.
    let real = v["game"]["real_s"].as_f64().unwrap();
    let active = v["time"]["active_s"].as_f64().unwrap();
    assert!((real - 999.999).abs() < 1e-6 && (real - active).abs() < 1e-6, "{real} {active}");
}

/// The human and the sim players on the same game: a log of the author's own
/// career parts from the author nowhere; the scripted opening parts from it.
#[test]
fn vs_human_sets_the_players_beside_the_human_and_finds_where_they_part() {
    use hh_core::Config;
    let dir = root("out/hh-sim-tests");
    std::fs::create_dir_all(&dir).unwrap();
    let c = Config::from_json(&std::fs::read_to_string(baseline()).unwrap()).unwrap();
    let ms_text = std::fs::read_to_string(root("configs/sim/milestones.json")).unwrap();
    let ms = hh_sim::milestones::Milestones::from_json(&ms_text).unwrap();
    let game = hh_sim::career::Game {
        config: &c,
        seed: 7,
        scenario: None,
        agent: "author",
        retirements: u32::MAX,
        max_ticks: 600,
        milestones: &ms,
        objectives: &[],
        trace: false,
    };
    let (_, s) = hh_sim::career::play_session(&game).unwrap();
    let mine = format!("{dir}/vs-author.jsonl");
    write_log_of(&mine, &c, &s, false);
    let (msf, hf) = (root("configs/sim/milestones.json"), root("configs/sim/humans.json"));
    let opts = ["--agents", "greedy,author", "--humans", "mixed", "--milestones", &msf, "--human-models", &hf];
    let mut a = vec!["vs-human", mine.as_str()];
    a.extend(opts);
    let out = run(&args(&a)).unwrap();
    for want in ["## Side by side", "| human | 600 |", "| author | 600 |", "| greedy | 600 |", "## Real time", "| human | log |", "| author | mixed |"] {
        assert!(out.contains(want), "{want}: {out}");
    }
    assert!(out.contains("best player, author") && out.contains("First apart in what: never"), "{out}");
    // The scripted opening builds in another order.
    let script = format!("{dir}/vs-script.jsonl");
    write_playlog(&script, false);
    let mut a = vec!["vs-human", script.as_str()];
    a.extend(opts);
    let out = run(&args(&a)).unwrap();
    assert!(out.contains("First apart in what: run 1, purchase"), "{out}");
    let mut a = vec!["vs-human", script.as_str(), "--json"];
    a.extend(opts);
    let v: serde_json::Value = serde_json::from_str(run(&args(&a)).unwrap().trim()).unwrap();
    assert_eq!(v["rows"].as_array().unwrap().len(), 3);
    assert_eq!(v["rows"][0]["name"], "human");
    assert!(v["divergence"]["what"]["nth"].as_u64().unwrap() >= 1);
    // A tampered log is a failed check.
    let bad = format!("{dir}/vs-bad.jsonl");
    write_playlog(&bad, true);
    let mut a = vec!["vs-human", bad.as_str()];
    a.extend(opts);
    assert!(matches!(run(&args(&a)), Err(Fail::Check(_))));
}
