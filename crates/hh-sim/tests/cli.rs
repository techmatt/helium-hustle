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
fn bad_input_is_a_usage_error() {
    for a in [
        vec!["run", "--bogus"],
        vec!["run", "--seed", "-1"],
        vec!["run", "--policy", "smart"],
        vec!["run", "--scenario", "nope"],
        vec!["frobnicate"],
        vec![],
    ] {
        assert!(matches!(run(&args(&a)), Err(Fail::Usage(_))), "{a:?}");
    }
}
