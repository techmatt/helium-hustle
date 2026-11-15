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

#[test]
fn the_ladder_prints_its_tables_and_writes_results() {
    let dir = root("out/hh-sim-tests/ladder");
    let players = "random,greedy,author,casual=author:min_gap=120";
    let out = run(&grid("experiment", players, &["--out", &dir, "--threads", "3"])).unwrap();
    for want in [
        "## Pace in game time",
        "## Milestones",
        "## Real time",
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
    assert_eq!(games.lines().count(), 8);
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
}

#[test]
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
    let e = |c: &str, n| Entry { command: c.into(), count: n };
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
