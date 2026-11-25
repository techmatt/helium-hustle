//! The test lanes behind `cargo test-all` and `cargo test-slow` (.cargo/config.toml;
//! README, *Build and test*):
//!
//!     cargo test-all [cargo test args] [-- test binary args]
//!
//! builds as `cargo test --no-run [--workspace] <cargo test args>` would (`--workspace`
//! unless a package is named), then runs every test binary at once, each with
//! `--include-ignored` (`--ignored` for `test-slow`) and the test binary args. `cargo test`
//! runs its binaries one after another, and the slow lane's longest tests sit in three
//! different binaries, so side by side the lane takes as long as its longest test, not
//! their sum. A test-name filter among the cargo args goes to every binary, as with
//! `cargo test`. Each binary's output prints whole when it finishes; then a line per
//! binary with its time, and the exit is 1 if any failed.

use std::path::{Path, PathBuf};
use std::process::{exit, Command, Stdio};
use std::time::Instant;

/// `cargo test` options that take a value as the next argument (`--opt=v` needs none).
const TAKES_VALUE: &[&str] = &[
    "-p", "--package", "--exclude", "-j", "--jobs", "--test", "--bin", "--example", "--bench",
    "-F", "--features", "--target", "--target-dir", "--profile", "--manifest-path",
    "--color", "--message-format", "--config", "-Z", "--lockfile-path",
];

fn main() {
    let mut args = std::env::args().skip(1);
    let lane = match args.next().as_deref() {
        Some("test-all") => "--include-ignored",
        Some("test-slow") => "--ignored",
        other => {
            eprintln!("xtask: expected test-all or test-slow, got {other:?}");
            exit(2);
        }
    };
    let rest: Vec<String> = args.collect();
    let split = rest.iter().position(|a| a == "--").unwrap_or(rest.len());
    let (given, after) = (&rest[..split], rest.get(split + 1..).unwrap_or(&[]));
    // Cargo's own flags stay with cargo; a bare word is a test-name filter for the binaries.
    let (mut cargo_args, mut filters) = (Vec::new(), Vec::new());
    let mut i = 0;
    while i < given.len() {
        let a = &given[i];
        if a.starts_with('-') {
            cargo_args.push(a.clone());
            if TAKES_VALUE.contains(&a.as_str()) {
                if let Some(v) = given.get(i + 1) {
                    cargo_args.push(v.clone());
                    i += 1;
                }
            }
        } else {
            filters.push(a.clone());
        }
        i += 1;
    }
    let names_package = cargo_args
        .iter()
        .any(|a| a.starts_with("-p") || a.starts_with("--package") || a == "--workspace" || a == "--all");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut build = Command::new(cargo);
    build.args(["test", "--no-run", "--message-format=json-render-diagnostics"]);
    if !names_package {
        build.arg("--workspace");
    }
    build.args(&cargo_args).stderr(Stdio::inherit());
    let built = build.output().unwrap_or_else(|e| {
        eprintln!("xtask: can't run cargo: {e}");
        exit(2);
    });
    if !built.status.success() {
        exit(built.status.code().unwrap_or(1));
    }
    let mut bins: Vec<(String, PathBuf, PathBuf)> = Vec::new();
    for line in String::from_utf8_lossy(&built.stdout).lines() {
        // A test harness (its profile's `test`), not a plain binary built beside them
        // (hh-sim's CLI for its integration tests, an example).
        let profile = line.split_once("\"profile\":{").map(|(_, p)| p.split('}').next().unwrap_or(""));
        if !line.contains("\"reason\":\"compiler-artifact\"") || !profile.is_some_and(|p| p.contains("\"test\":true")) {
            continue;
        }
        let (Some(exe), Some(manifest)) = (json_str(line, "executable"), json_str(line, "manifest_path")) else {
            continue;
        };
        let dir = Path::new(&manifest).parent().unwrap().to_path_buf();
        let pkg = dir.file_name().unwrap().to_string_lossy().into_owned();
        let target = line.split_once("\"target\":{").map_or("", |(_, t)| t);
        let kind = target.split_once("\"kind\":[\"").and_then(|(_, r)| r.split('"').next()).unwrap_or("");
        let name = format!("{pkg} {kind} {}", json_str(target, "name").unwrap_or_default());
        bins.push((name, PathBuf::from(exe), dir));
    }
    let mut test_args: Vec<String> = filters;
    if !after.iter().any(|a| a == lane) {
        test_args.push(lane.to_string());
    }
    test_args.extend(after.iter().cloned());
    let start = Instant::now();
    let (tx, rx) = std::sync::mpsc::channel();
    for (name, exe, dir) in bins.clone() {
        let (tx, test_args) = (tx.clone(), test_args.clone());
        std::thread::spawn(move || {
            let t = Instant::now();
            let out = Command::new(&exe)
                .args(&test_args)
                .current_dir(&dir)
                .env("CARGO_MANIFEST_DIR", &dir)
                .output();
            tx.send((name, out, t.elapsed().as_secs_f64())).unwrap();
        });
    }
    drop(tx);
    let mut times = Vec::new();
    let mut failed = false;
    for (name, out, secs) in rx {
        println!("==== {name} ({secs:.1} s)");
        let ok = match out {
            Ok(o) => {
                print!("{}", String::from_utf8_lossy(&o.stdout));
                eprint!("{}", String::from_utf8_lossy(&o.stderr));
                o.status.success()
            }
            Err(e) => {
                eprintln!("xtask: can't run {name}: {e}");
                false
            }
        };
        failed |= !ok;
        times.push(format!("{name}: {secs:.1} s{}", if ok { "" } else { ", FAILED" }));
    }
    println!("==== {} binaries side by side, {:.1} s:", bins.len(), start.elapsed().as_secs_f64());
    for t in times {
        println!("  {t}");
    }
    exit(i32::from(failed));
}

/// The string value of `"key":"..."` in one line of cargo's JSON (None for `null`).
fn json_str(line: &str, key: &str) -> Option<String> {
    let at = line.find(&format!("\"{key}\":\""))? + key.len() + 4;
    let mut s = String::new();
    let mut chars = line[at..].chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(s),
            '\\' => match chars.next()? {
                'n' => s.push('\n'),
                't' => s.push('\t'),
                c => s.push(c),
            },
            c => s.push(c),
        }
    }
    None
}
