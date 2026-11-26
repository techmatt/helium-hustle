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
| `crates/hh-sim` | Native CLI ([docs/simulator.md](docs/simulator.md)): runs a seed, replays records; agents ([docs/agents.md](docs/agents.md)) play careers in game time against the targets ([docs/principles.md](docs/principles.md)): the ladder, experiments, compare, balance, a human's play log beside them (`vs-human`). |
| `crates/hh-wasm` | wasm-bindgen bindings for `web/`, JSON in and out (with `hh-sim`'s agents, for the debug autopilot). |
| `web/` | The browser UI ([web/README.md](web/README.md)). Renders the engine's view, sends actions, holds no rules. `web/src/engine.ts` mirrors hh-core's `View` by hand, and hh-sim reads view fields the page doesn't: grep both before dropping or renaming one. |
| `configs/` | Game configs: modules, variants and every tunable number. No defaults in code. |
| `configs/sim/` | The simulator's data: human time models, the milestones with their targets, the hands-off player's loadout. |
| `content/` | Game data (resources, buildings, commands, research, events, acts, projects, achievements, earth_tech, rivals, nav, messages), embedded in `hh-core`; `prose.md` is `scripts/prose.py`'s export. |
| `scenarios/` | Named starting states for the sim and the web debug menu, embedded in `hh-core`. |
| `experiments/` | Experiment specs (`ladder.json`, `ceiling.json` the oracle's, `balance.json` the judge's); `results/<item>/` holds an item's write-up and working papers when it left any (bug hunts record into `docs/qa-findings.md`). A read's `games.jsonl`: `hh-sim summarize`. |
| `scripts/` | `wasm-smoke.mjs` (native = wasm), `wasm-speed.mjs` (engine speed in wasm), `warning_checks.py` (CLAUDE.md, *Warning checks*), `set_golden.py` (re-pins the golden game), `set_pins.py` (re-pins the sim players' games, *Rules changes* below), `seeds.py` (fresh seeds from the ledger, [docs/simulator.md](docs/simulator.md) *Seeds*), `baseline.py` (a built copy of another revision for reads against it, [docs/experiments.md](docs/experiments.md)), `prose.py` (every player-facing string out to `content/prose.md` and back by id: `export`, `import FILE`, `check`; re-export after a text change). |
| `docs/` | `rules/` (the rules as implemented) and `rules-history.md`; `rulings.md` and `rulings/` (Matt's standing rulings, by area); `architecture.md`; `principles.md` (targets and the judge); `extending.md` (what a new module or program piece touches); `simulator.md`, `experiments.md` (grid reads, measures, costs), `agents.md` and `oracle.md` (hh-sim); `qa-findings.md` (bug hunts). |
| `reference/` | The Godot snapshot ([reference/README.md](reference/README.md)). Never edited. |
| `art/` | Generated art and picker images: a link to the main checkout's `art/`, shared by every worktree and never committed. The scripts that draw it are committed with their item's write-up in `experiments/results/<item>/`. Painted art is drawn by Codex CLI's image tool on Matt's ChatGPT login (no API key), 37–97 s a draw, 3–4 at once: recipe in [018's write-up](experiments/results/018_building_art_round/README.md); drawing against a fixed guide image (`codex exec -i`, isometric patches) in [019's](experiments/results/019_painterly_iso_round/README.md); the shipped building card pictures (draw, cut to WebP, picker) in [020's](experiments/results/020_building_card_art/README.md). **What ships** is in `web/src/assets/`: painted art cut to WebP (020's `process.py`, 027's `images.py`), and chip and compact-nav glyphs as SVG written by 027's `export.mjs` from [026's `glyphs.mjs`](experiments/results/026_icon_art_r1/) (which reads `icons.tsx`'s color maps as text, and holds the nav colors). Finished pickers (`art/pickers/`, mock-ups only) and superseded rounds (`art/buildings/r1/`) move to the Drive archive, `C:\Code\gdrive-sync\helium-hustle\archive\` at the same relative path; a new picker writes `art/pickers/` afresh. |
| `out/` | Scratch and run output, git-ignored. |

**One wasm build**, `web/src/wasm/` (git-ignored), by `node web/scripts/build-wasm.mjs`
(needs only node and wasm-pack, not `web/node_modules`), which skips the build when a hash of
its inputs is unchanged ([web/README.md](web/README.md)). The page, `scripts/wasm-smoke.mjs`
(the parity check) and `scripts/wasm-speed.mjs` all load it; `warning_checks.py` runs the build
script first.

## Toolchain

Versions this was built with (2026-11-14): Rust 1.100.0 (stable), wasm-pack 0.15.0,
Node 24.19.0, npm 11.17.0.

- Rust stable, target `wasm32-unknown-unknown` (`rustup target add wasm32-unknown-unknown`).
- wasm-pack (`cargo install wasm-pack --locked`).
- Node (LTS) for `web/`.

## Build and test

Times are on this machine, otherwise idle.

```sh
cargo build --workspace                      # ~30 s cold on 3 jobs; target/debug ~680 MB, safe to delete
cargo test --workspace                       # the fast lane: ~4 s once built, ~11 s after an hh-core edit; first build ~1.5 min
cargo test-all                               # the full run, slow lane too: ~40 s (the autopilot's career)
cargo clippy --workspace --all-targets       # must stay clean; ~7 s
cargo run -q -p hh-sim -- run --seed 1 --ticks 365 --policy script
cargo run -q --release -p hh-sim -- objectives reference/scenarios/run1_fresh.json   # ~0.2 s
cargo run -q --release -p hh-sim -- ladder   # the agent ladder, 15 careers: ~20 s on 6 threads (the oracle's ceiling.json: ~20 min)
node web/scripts/build-wasm.mjs              # -> web/src/wasm (git-ignored); ~80 s cold, <1 s when inputs are unchanged
node scripts/wasm-smoke.mjs                  # native = wasm on that build; ~1 s
python scripts/warning_checks.py             # warning checks; parity and balance only if the branch touched what they read: ~1 s, else up to 2 min cold
python scripts/warning_checks.py --all       # parity and balance whatever was touched (rebuilds stale builds)
python scripts/warning_checks.py --no-parity --no-balance   # static checks only (tunables, floats, docs, qa's last report), no builds; ~1 s
cd web && npm ci && npm run dev              # the browser UI at http://localhost:5173/ (web/README.md)
cd web && npm run wasm && npm run check      # type-check alone: a fresh worktree has no web/src/wasm until `npm run wasm` (~80 s)
```

**Test lanes.** `cargo test --workspace` is the fast lane and the default: run it after
every change. `cargo test-all` is the full run, before a commit that touches Rust, content
or configs (when to run what: CLAUDE.md, *Working rules*): a test over ~3 s is marked
`#[ignore = "slow lane: ..."]` (but hh-sim's ladder pin, ~4 s, the guard on every agent
edit), and `cargo test-slow` runs only those. Both aliases (`.cargo/config.toml`) run
`crates/xtask`: it builds as `cargo test` does (cargo's flags before a `--`, `--workspace`
unless a package is named), then runs every test binary at once (the slow lane's longest
tests sit in three), each printed whole as it ends, then their times. A bare word filters
tests, as for `cargo test`: `cargo test-all -j 3 -p hh-sim golden -- --test-threads=3`.
The test profile is `opt-level = 2` with overflow checks, and test and sim binaries
(release `hh-sim` too, never hh-core or wasm) use mimalloc: a game's time goes to the
view's state copies, twice as slow on the Windows allocator. To time tests one by one, run the test binary itself (`<exe> --list`,
`<exe> --exact <name>`; per-test times: `RUSTC_BOOTSTRAP=1 <exe> -Z unstable-options
--report-time`): Rust 1.100 puts them under `target/debug/build/<crate>/<hash>/out/`.

**A second target dir** (a parallel helper, a copy under `out/`) is set absolute:
`CARGO_TARGET_DIR="$(git rev-parse --show-toplevel)/out/target-<name>"`. Cargo reads a
relative one from the directory it runs in, and wasm-pack runs it in `crates/hh-wasm`, so
`out/x` lands in `crates/hh-wasm/out/x`. `build-wasm.mjs` and `warning_checks.py` pin a
relative one to the repo root; bare `cargo` can't (the variable outranks `.cargo/config.toml`).
`/crates/**/out/` is git-ignored as a backstop.

**Rules changes:** the `golden_game` test (`crates/hh-core/src/tests/mod.rs`) pins the final
state of one fixed game: baseline config, seed `u64::MAX`, the `golden_start` scenario (`if`
open from tick 0, `driver::golden_start`), 2000 ticks of the scripted policy
(`driver.rs`: it ships, runs a condition program once program logic opens, is retired by
boredom and starts run 2). An intended play
change bumps `RULES_REVISION`
(`crates/hh-core/src/lib.rs`, and the revision line in `docs/rules/README.md`), adds its
`docs/rules-history.md` row, and updates the four golden constants, `GOLDEN_REVISION`,
`GOLDEN_FINGERPRINT`, `GOLDEN_CONTENT_HASH` and `GOLDEN_CONFIG_HASH`, both in
`crates/hh-core/src/tests/mod.rs` and in `scripts/wasm-smoke.mjs`
(`wasm_smoke_expects_the_golden_constants` fails until they match; it also checks the smoke
plays the golden game through `play_golden`, which runs `hh_core::driver` itself in wasm, with
the same seed and length). The content hash moves with any `content/` byte and the config hash
with any `configs/baseline.json` value, play or not; the fingerprint (`GameState::fingerprint`,
the state less its config) only with play. **Step:** `python scripts/set_golden.py` (a release build first: background it) sets all
eight from `hh-sim golden`; never type them. The sim players' pinned games are separate:
`PINNED`, `HANDSOFF_PIN` and the oracle's in `crates/hh-sim/tests/cli.rs` (an agent refactor
keeps them; a rules change moves those whose play it changes): run `cargo test-all -p hh-sim`
(the oracle's pin is a slow-lane test; failing pins still write their games to
`out/hh-sim-tests/`), then `python scripts/set_pins.py` copies them into `cli.rs`. A param
added, removed or re-ranged regenerates the `docs/rules/` param tables:
`cargo run -q --release -p hh-sim -- params --write` (a test fails while they are stale).
Content ids are also named outside `content/` (`configs/sim/milestones.json`, the sim agents,
tests, `web/scripts/e2e.mjs`): `git grep` an id before removing it. The program text form
is read from write-ups too (`experiments/results/063_program_text_editor/*.txt` by hh-core's
tests, `060_loadout_exploration/loadouts/` by hh-sim's): a grammar change converts them.

**Adding to the engine** (a module, an event condition, a program reading, line kind or
comparison): the files each one touches are listed in [docs/extending.md](docs/extending.md).

**Scenarios are asserted on**: hh-core's `tests/economy.rs` (`late_unlocks`' one-energy
shortage and programs, `rich`), `tests/replay.rs` (`everything`'s views and quest, `rich`),
hh-sim's `tests/cli.rs` (`rich`) and e2e sections (docs/architecture.md, *Scenarios*). A content change (an upkeep, a
quest's unlocks) re-checks them.

## This machine

- Windows 10, Git Bash and PowerShell. 6 cores, 12 threads. LF line endings (`.gitattributes`);
  Git Bash `grep` can't detect CRLF: use `git ls-files --eol`.
- Python writes cp1252 and CRLF by default: scripts open files with `encoding='utf-8',
  newline='\n'`.
- Write a multi-line edit script with the Write tool and run the file: a Git Bash heredoc
  loses a level of backslash escaping (`'\\n'` arrives as `'\n'`), and conductor's hooks refuse
  heredocs and `sed -i` that write files.
- An agent's Bash `cd` (into `web/` for `npm ci`, say) persists into its later calls and
  into conductor's `start`: use absolute paths or `cd` inside one `&&` chain.
- Windows can't replace a running exe: copy a binary to `out/` to run it while rebuilding.
  `warning_checks.py` rebuilds `target/release/hh-sim.exe`, so its parity and balance checks
  fail ("Access is denied") while that binary runs.
