use hh_sim::{run, Fail};

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
    assert!(out.contains("day 50"), "{out}");
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
        "quest qextract_first_extraction",
        "quest qshipment_proof_of_concept",
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
        vec!["frobnicate"],
        vec![],
    ] {
        assert!(matches!(run(&args(&a)), Err(Fail::Usage(_))), "{a:?}");
    }
}

fn root(p: &str) -> String {
    format!("{}/../../{p}", env!("CARGO_MANIFEST_DIR"))
}

/// A small grid: the given agents, one profile, 2 seeds, careers of 2 retirements.
fn grid(cmd: &str, agents: &str, extra: &[&str]) -> Vec<String> {
    let (spec, profiles, milestones, config) = (
        root("experiments/balance.json"),
        root("configs/sim/profiles.json"),
        root("configs/sim/milestones.json"),
        baseline(),
    );
    let mut a = vec![
        cmd.to_string(),
        spec,
        "--profiles".into(),
        profiles,
        "--milestones".into(),
        milestones,
        "--config".into(),
        config,
        "--agents".into(),
        agents.into(),
        "--attention".into(),
        "active".into(),
        "--speeds".into(),
        "10x".into(),
        "--seeds".into(),
        "2".into(),
        "--retirements".into(),
        "2".into(),
    ];
    a.extend(extra.iter().map(|s| s.to_string()));
    a
}

#[test]
fn the_ladder_prints_its_tables_and_writes_results() {
    let dir = root("out/hh-sim-tests/ladder");
    let out = run(&grid("experiment", "random,greedy,author", &["--out", &dir, "--threads", "3"])).unwrap();
    for want in ["## Pace", "## Milestones", "## Never built", "author active/10x", "greedy active/10x", "random active/10x", "M4"] {
        assert!(out.contains(want), "{want}: {out}");
    }
    let games = std::fs::read_to_string(format!("{dir}/games.jsonl")).unwrap();
    assert_eq!(games.lines().count(), 6);
    for line in games.lines() {
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(v["replay_ok"], true, "{line}");
        assert_eq!(v["retirements"], 2);
    }
}

#[test]
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

#[test]
fn bad_grids_are_usage_errors() {
    for (agents, extra) in [
        ("smart", vec![]),
        ("planner", vec![]),
        ("author:bogus=1", vec![]),
        ("author", vec!["--speeds", "warp"]),
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
fn agents_keep_their_knobs_and_cope_with_one_program() {
    let dir = root("out/hh-sim-tests/knobs");
    let out = run(&grid(
        "experiment",
        "author:hand_gap=0,loads=10,greedy",
        &["--out", &dir, "--set", "programs.program_count=1", "--set", "economy.upkeep_epsilon=0.000001"],
    ))
    .unwrap();
    assert!(out.contains("| author:hand_gap=0,loads=10 active/10x |") && out.contains("| greedy active/10x |"), "{out}");
}

#[test]
fn empty_lists_and_stray_options_are_usage_errors() {
    for extra in [
        vec!["--speeds", ","],
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
}

/// Agents decide in integers and `Fx` with ordered maps only, so a decision
/// can't differ between native and wasm builds (docs/agents.md).
#[test]
fn agents_use_no_floats_or_hash_maps() {
    let dir = format!("{}/src/agents", env!("CARGO_MANIFEST_DIR"));
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        for banned in ["f64", "f32", "HashMap", "HashSet", "to_f64", "from_f64"] {
            assert!(!text.contains(banned), "{} uses {banned}", path.display());
        }
    }
}

#[test]
fn a_stockpile_milestone_on_a_capped_resource_is_refused() {
    let m = std::fs::read_to_string(root("configs/sim/milestones.json")).unwrap();
    let bad = m.replace("\"resource\": \"credits\"", "\"resource\": \"he3\"");
    assert!(hh_sim::milestones::Milestones::from_json(&bad).unwrap_err().contains("capped"));
    assert!(hh_sim::milestones::Milestones::from_json(&m).is_ok());
}
