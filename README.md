# helium-hustle

**Helium Hustle**, a sci-fi browser idle game: you are an AI running helium-3 mining on the Moon,
balancing power, shipments and demand, with a longer arc of rival AIs and prestige resets. It is
being restarted from an earlier Godot build as a pure, deterministic **Rust simulation core** with
a **TypeScript UI** in the browser. A native simulator and a wasm build sit on top of the core,
and neither holds any rules.

- [CLAUDE.md](CLAUDE.md): the working contract for executors (engine rules, docs rules, reports).
- [reference/](reference/README.md): a read-only snapshot of the old Godot build, for the rules
  port. Not ground truth for the new game.

## Layout

How the engine fits together: [docs/architecture.md](docs/architecture.md). The rules as
implemented (the Godot build's Arc 1, ported): [docs/rules/](docs/rules/README.md).

| Path | What it is |
|---|---|
| `crates/hh-core` | The rules engine: every rule, fixed-point state, no I/O, no clock, no global RNG (CLAUDE.md, *Engine rules*). |
| `crates/hh-sim` | Native CLI ([docs/simulator.md](docs/simulator.md)): runs a seed, replays records; agents ([docs/agents.md](docs/agents.md)) play careers against the targets ([docs/principles.md](docs/principles.md)): the ladder, experiments, compare, balance. |
| `configs/sim/` | The simulator's data: player profiles (attention, speed) and the milestones with their targets. |
| `experiments/` | Experiment specs (`ladder.json`, `balance.json`); `results/` holds each item's write-up and working papers (`<item>/`). |
| `crates/hh-wasm` | wasm-bindgen bindings for `web/`, JSON in and out. |
| `web/` | The browser UI ([web/README.md](web/README.md)). Renders the engine's view, sends actions, holds no rules. |
| `configs/` | Game configs: modules, variants and every tunable number. No defaults in code. |
| `content/` | Game data (resources, buildings, commands, research, events, projects, achievements, rivals), embedded in `hh-core`. |
| `scenarios/` | Named starting states for the sim and the web debug menu, embedded in `hh-core`. |
| `scripts/` | `wasm-smoke.mjs` (native = wasm), `wasm-speed.mjs` (engine speed in wasm), `warning_checks.py` (CLAUDE.md, *Warning checks*), `set_golden.py` (re-pins the golden game). |
| `docs/` | The current game: `docs/rules/` (rules as implemented), `docs/rules-history.md`. |
| `reference/` | The Godot snapshot ([reference/README.md](reference/README.md)). Never edited. |
| `out/` | Scratch and run output, git-ignored. |

## Toolchain

Versions this was built with (2026-11-14): Rust 1.100.0 (stable), wasm-pack 0.15.0,
Node 24.19.0, npm 11.17.0.

- Rust stable, target `wasm32-unknown-unknown` (`rustup target add wasm32-unknown-unknown`).
- wasm-pack (`cargo install wasm-pack --locked`).
- Node (LTS) for `web/`.

## Build and test

Times are on this machine, otherwise idle.

```sh
cargo build --workspace                      # under a minute cold
cargo test --workspace                       # the test lane: ~1 s once built
cargo test-all                               # also the slow lane's #[ignore]d tests (none yet)
cargo clippy --workspace --all-targets       # must stay clean; ~7 s
cargo run -q -p hh-sim -- run --seed 1 --ticks 365 --policy script
cargo run -q --release -p hh-sim -- objectives reference/scenarios/run1_fresh.json   # ~0.2 s
cargo run -q --release -p hh-sim -- ladder   # the agent ladder, 135 careers: ~35 s on 6 threads (docs/simulator.md)
wasm-pack build crates/hh-wasm --target web  # -> crates/hh-wasm/pkg (git-ignored); ~22 s cold, ~5 s warm
node scripts/wasm-smoke.mjs                  # native = wasm: needs the pkg above (not web's copy); ~1 s
python scripts/warning_checks.py             # warning checks, incl. parity and balance (rebuilds stale builds); ~7 s, up to a minute cold
cd web && npm ci && npm run dev              # the browser UI at http://localhost:5173/ (web/README.md)
```

**Test lanes.** `cargo test --workspace` is the fast lane and the default: run it after
every change. `cargo test-all` (a `.cargo/config.toml` alias for `cargo test --workspace --
--include-ignored`) is the full run, before every commit: a test slower than about a second
is marked `#[ignore = "slow lane: ..."]`. Anything after an alias goes to the test binary,
with no second `--`, so `cargo test-all -j 6` fails (`Unrecognized option: 'j'`): limit
jobs with `CARGO_BUILD_JOBS=6 cargo test-all`. Tests play whole games, so the test profile
is optimized (`opt-level = 1`) and keeps overflow checks.

**Rules changes:** the `golden_game` test (`crates/hh-core/src/tests/mod.rs`) pins the final
state of one fixed game: baseline config, seed `u64::MAX`, 2000 ticks of the scripted
policy (`driver.rs`: it ships, retires at about tick 1100 and starts run 2). An intended play
change bumps `RULES_REVISION`
(`crates/hh-core/src/lib.rs`, and the revision line in `docs/rules/README.md`), adds its
`docs/rules-history.md` row, and updates `GOLDEN_REVISION` and `GOLDEN_FINGERPRINT` there and
the same three `GOLDEN_*` constants in `scripts/wasm-smoke.mjs`
(`wasm_smoke_expects_the_golden_constants` fails until they match; it also checks the smoke
plays the golden game through `play_golden`, the wasm build's own copy of `driver.rs`, with
the same seed and length). `GOLDEN_CONTENT_HASH` moves with any `content/` byte, play or
not. Never type them: `python scripts/set_golden.py` copies all six from `hh-sim golden`
(or copy each from the failing assertion's left side). A rename or refactor that plays the
same moves no golden constant; if it does, play changed.

## This machine

- Windows 10, Git Bash and PowerShell. 6 cores, 12 threads.
- LF line endings throughout (`.gitattributes`). Git Bash `grep` can't detect CRLF: use
  `git ls-files --eol`.
- Python writes cp1252 and CRLF by default: scripts open files with `encoding='utf-8',
  newline='\n'`.
- Windows can't replace a running exe: copy a binary to `out/` to run it while rebuilding.
