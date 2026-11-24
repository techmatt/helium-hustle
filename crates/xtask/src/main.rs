//! The test lanes behind `cargo test-all` and `cargo test-slow` (.cargo/config.toml;
//! README, *Build and test*). A cargo alias only appends its arguments, so an alias
//! ending in `-- --include-ignored` sent every later flag (`-j 3`, `-p hh-sim`) to the
//! test binary. This splits them instead:
//!
//!     cargo test-all [cargo test args] [-- test binary args]
//!
//! runs `cargo test [--workspace] <cargo test args> -- --include-ignored <test binary args>`
//! (`--ignored` for `test-slow`). `--workspace` is added unless a package is named.

use std::process::{exit, Command};

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
    let (cargo_args, test_args) = (&rest[..split], rest.get(split + 1..).unwrap_or(&[]));
    let names_package = cargo_args
        .iter()
        .any(|a| a.starts_with("-p") || a.starts_with("--package") || a == "--workspace" || a == "--all");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo);
    cmd.arg("test");
    if !names_package {
        cmd.arg("--workspace");
    }
    cmd.args(cargo_args).arg("--");
    if !test_args.iter().any(|a| a == lane) {
        cmd.arg(lane);
    }
    cmd.args(test_args);
    let status = cmd.status().unwrap_or_else(|e| {
        eprintln!("xtask: can't run cargo: {e}");
        exit(2);
    });
    exit(status.code().unwrap_or(1));
}
