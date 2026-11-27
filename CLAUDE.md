# CLAUDE.md

The standing contract for every executor working in this repo. The [README](README.md) has what
the game is, the layout, build and test commands, and this machine's traps.

## What this repo is

Helium Hustle: a Rust simulation core (`hh-core`) under a native CLI (`hh-sim`), wasm bindings
(`hh-wasm`) and a TypeScript UI (`web/`), restarted from an earlier Godot build.

- **Clients hold no rules.** The UI renders the engine's view and sends actions; it never
  computes a cost, a rate or an unlock itself.
- **`reference/` is the Godot build, read-only, for porting rules.** It is not ground truth:
  where it and `docs/rules/` disagree, `docs/rules/` wins. Never edit it.
- **Read `docs/rules/README.md`, then only the files your task touches.** Never describe code
  as doing more than it does.
- **Matt's standing rulings** (pillars, speed, saves, art, sim players, and executor defaults
  that stand until he overrides them) are indexed by area in [docs/rulings.md](docs/rulings.md).
  A new ruling, or a change to one, updates its line in the same commit (how a full part
  splits: that page).

## Engine rules

Locked by Matt: changing any of these is his decision, not an implementation detail.
`crates/hh-core/clippy.toml` enforces the clock, hash-map and platform-float bans.

- **`hh-core` holds every rule, with no I/O, no clock, no global RNG.** Time is a tick count
  passed in. Randomness comes from streams derived from the game seed, one per purpose, so a new
  draw in one system never shifts another's.
- **No hash-order leaks.** Nothing that affects play iterates a `HashMap`/`HashSet`; use
  `BTreeMap`, `Vec` or an index order.
- **State is serde-serializable**, all of it: a saved game reloads to the same future.
- **Exact replay, native = wasm.** The same config, seed and action list give bit-identical state
  natively and in wasm.
- **Fixed-point state.** Every quantity in game state is `Fx(i64)` at 10^-6 (1.0 = 1_000_000).
  Products and quotients go through `i128` intermediates and **one rounding rule**, defined once
  in `Fx`; no ad-hoc `as` casts or rounding at call sites.
- **Floats only for computing factors** (a cost-scaling multiplier, a curve), converted to `Fx`
  at once, and **only through `libm`** (wrapped in `crate::math`). Platform float
  transcendentals (`powf`, `exp`, `ln`, `sin`, ...) differ across platforms and break
  native = wasm; never `#[allow]` the lint that bans them.
- **`RULES_REVISION` and a pinned golden game.** Any change to how games play out bumps
  `RULES_REVISION` and updates the golden constants (how: README, *Rules changes*). Bump from
  the constant in `lib.rs`, not from a revision number in the prompt, and give the new
  revision in the rules-history row and the report. A rename or refactor leaves the
  fingerprint where it was (a content or config byte still moves its hash); if the
  fingerprint moves, play changed.
- **Every tunable number lives in a config** under `configs/`, with **no defaults in code**: a
  missing key is a load error, never a silent fallback. A test may build its own config. Sim
  agents' numbers are player strategy, not rules, and exempt (docs/rulings/sim.md).
- **Mechanics are modules, alternatives are variants, content is data.** A new system (shipments,
  boredom, research) is a module; another way of doing one is a variant chosen in the config;
  buildings, commands, research, events and the like are rows in `content/`, not code.
- **One home per rule:** its number in a config or content row, its effect in one place in
  `hh-core`.
- **Before the report**, `git grep` each renamed name and each changed number, and reread the
  `docs/rules/` file a changed mechanic touches: a diff doesn't show the prose it left stale.

## Docs rules

- **Docs state the current game.** An item updates the current text in place. No per-item
  history sections ("NNN's changes"), no archive files; history is git and the item's write-up
  under `experiments/results/<item>/`.
- **Each doc keeps under about 15 KB** (README, CLAUDE.md, web/README, web/TESTING and every
  page under `docs/`). Leave about 1 KB free: an item that fills a page trims it first,
  cutting what the code or a generated table already says, never moving text to an archive.
- **Every rules change** updates the `docs/rules/` files it touches and adds its row to
  `docs/rules-history.md` (revision, what changed, why) in the same commit.
- **Facts learned while working go in the page for their area**, never only in a report: build,
  test and this machine in `README.md`; a read's cost and recipe in `docs/experiments.md`.

## Working rules

- **Commit to the branch you are on.** Leave nothing uncommitted. Never commit to or push `main`.
- **Testing is light by default** (Matt, docs/rulings/engine.md). After every change: the fast
  lane, `cargo test --workspace`, plus the checks for what you touched (`npm test` in `web/`:
  its fast lane, `check` then `unit`). Before a commit that touches Rust, `content/` or `configs/`: the
  full run, `cargo test-all`; Markdown or `web/` alone skips it. Leave `cargo clippy
  --workspace --all-targets` clean. **Heavy reads only on request** (a bug hunt, or the prompt
  asks), never as routine: a full `npm run qa`, e2e, balance reads (`hh-sim balance`, ladders,
  before/after reads) and other sweeps. A balance warning is reported, not chased. Lanes,
  commands and their times: README, *Build and test*.
- **Background any step over ~30 seconds** (builds, tests, sims, servers) and wait on the real
  process (the `cargo`, `node` or test binary), never on a launcher that returns before the work
  is done. A fresh worktree's first build takes minutes: give it its time. Its first web check
  needs `npm ci` and `npm run wasm` in `web/` first.
- **A before/after read** (main, another commit, new rules with old agents) builds its other
  side with `python scripts/baseline.py` (docs/experiments.md, *Reads against another revision*).
- **Browser checks, sweeps and e2e run against `vite preview`** (a static build), never the dev
  server: its watcher rebuilds the wasm and reloads the page when engine sources change.
- **One cargo at a time in a target dir.** A parallel helper gets its own `CARGO_TARGET_DIR`,
  absolute, under the root `out/` (why and how: README, *Build and test*).
- **Write-up scripts under `experiments/results/` may read `web/src` source as text.** When
  you remove or move a name from `web/src`, grep `experiments/results/` for it.
- **Write files with Write/Edit, not a shell heredoc or `sed`.** Scripts that write files:
  README, *This machine*.
- **LF line endings.** `.gitattributes` sets `* text=auto eol=lf`; don't override it.
- **Times are local, 12-hour clock** (2:45pm), in reports, docs and progress lines.
- **American English everywhere:** content, docs, UI, comments and names.
- Scratch and run output go in `out/` (git-ignored), never `%TEMP%`.
- **Screenshots** an item makes go in its results folder and are also copied to
  `C:\Code\gdrive-sync\helium-hustle\screenshots\` as `NNN_*.png` (NNN: the item's number):
  a standing exception to writing only inside the worktree (docs/rulings/ui.md).

## Report

Write the report to the path conductor's preamble gives, starting with the header it specifies.

- Soft target about 60 lines: findings, decisions and surprises only, no process narration.
  Long tables go in the item's results folder, linked.
- Mention tests only for a red you couldn't fix.
- Answer every amendment and message by number.
- A rules item's report ends with a short new-player summary of its rule changes.
- End your final message with `Report written to <path>`.

## Warning checks

Warnings are surfaced, never gate: an item whose own change trips one still finishes, and lists
it in the header's `warnings:` line, led by its kind. Before its report, every item runs
`python scripts/warning_checks.py`: it always exits 0 and prints one `<kind>: <text>` line per
tripped check. Its kinds (each defined in the script's docstring, with its flags) are
**parity**, **tunables** (exceptions, each with its reason, in `scripts/warning_checks.json`),
**platform-floats**, **balance** (the judge: `docs/principles.md`, *The judge*), **docs**,
**qa** and **checks**.

And one by hand, also kind **docs:** a rules change without its `docs/rules-history.md` row, or
a `RULES_REVISION` bump without the golden constants moved (or the reverse).

Not checked yet, Matt's to pick (thresholds would go in a config): resources at cap, buildings
or commands never used, idle stretches, simulation speed regression.
