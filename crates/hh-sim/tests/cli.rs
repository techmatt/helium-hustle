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

/// A play log as the web client writes one: a scripted career at 10x with a
/// minute's pause, its actions, and a closing checkpoint.
fn write_playlog(path: &str, tamper: bool) {
    use hh_core::{driver, Config, Session, RULES_REVISION};
    let c = Config::from_json(&std::fs::read_to_string(baseline()).unwrap()).unwrap();
    let mut s = Session::new(&c, 7, None).unwrap();
    driver::play(&mut s, 600, driver::Policy::Script);
    let rec = s.record();
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
    let (ms, pf) = (root("configs/sim/milestones.json"), root("configs/sim/profiles.json"));
    let out = run(&args(&["playlog", &path, "--milestones", &ms, "--profiles", &pf])).unwrap();
    assert!(out.contains("final state reproduced") && out.contains("## Milestones") && out.contains("| paused |"), "{out}");
    let j = run(&args(&["playlog", &path, "--json", "--milestones", &ms, "--profiles", &pf])).unwrap();
    let v: serde_json::Value = serde_json::from_str(j.trim()).unwrap();
    assert_eq!(v["reproduced"], true);
    assert_eq!(v["game"]["ticks"], 600);
    let paused = v["time"]["at_speed_s"]["0"].as_f64().unwrap();
    assert!((paused - 60.0).abs() < 1e-6, "{paused}");
    // 600 ticks at 10x plus the pause, all played.
    let real = v["game"]["real_s"].as_f64().unwrap();
    assert!((real - 120.0).abs() < 1e-6, "{real}");
    let bad = format!("{dir}/play-bad.jsonl");
    write_playlog(&bad, true);
    assert!(matches!(run(&args(&["playlog", &bad, "--milestones", &ms, "--profiles", &pf])), Err(Fail::Check(_))));
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
    let j = run(&args(&["playlog", &late, "--json", "--milestones", &ms, "--profiles", &pf])).unwrap();
    let v: serde_json::Value = serde_json::from_str(j.trim()).unwrap();
    assert_eq!(v["reproduced"], serde_json::Value::Null);
    assert_eq!(v["game"]["actions"].as_u64().unwrap(), actions + 2, "the repeated line is read once");
    let real = v["game"]["real_s"].as_f64().unwrap();
    assert!((real - 120.0).abs() < 1e-6, "the late line adds no time: {real}");
}
