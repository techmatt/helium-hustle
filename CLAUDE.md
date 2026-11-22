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
  that stand until he overrides them) are in [docs/rulings.md](docs/rulings.md). A new ruling,
  or a change to one, updates its line there in the same commit.

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
  the constant, not from a revision number in the prompt, and say so in the rules-history row
  and the report. A rename or refactor that plays the same moves no golden constant: if it
  does, play changed.
- **Every tunable number lives in a config** under `configs/`, with **no defaults in code**: a
  missing key is a load error, never a silent fallback. A test may build its own config. Sim
  agents' numbers are player strategy, not rules, and exempt (docs/rulings.md).
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
- **Each doc keeps under about 15 KB** (README, CLAUDE.md, web/README, web/TESTING and every `docs/` page,
  `docs/rules/` included). Leave about 1 KB free: an item that fills a page trims it first,
  cutting what the code or a generated table already says, never moving text to an archive.
- **Every rules change** updates the `docs/rules/` files it touches and adds its row to
  `docs/rules-history.md` (revision, what changed, why) in the same commit.
- **Facts learned while working go in the page for their area**, never only in a report: build,
  test and this machine in `README.md`; a read's cost and recipe in `docs/experiments.md`.

## Working rules

- **Commit to the branch you are on.** Leave nothing uncommitted. Never commit to or push `main`.
- **Test lanes** are in the README, *Build and test*. Run the fast lane after every change and
  the full run before every commit; a commit touching only Markdown skips the full run. Leave
  `cargo clippy --workspace --all-targets` clean.
- **Background any step over ~30 seconds** (builds, tests, sims, servers) and wait on the real
  process (the `cargo`, `node` or test binary), never on a launcher that returns before the work
  is done. A fresh worktree's first build takes minutes: give it its time. It also has no
  `web/node_modules`: run `npm ci` in `web/` before the first web build or check.
- **Browser checks, sweeps and e2e run against `vite preview`** (a static build), never the dev
  server: its watcher rebuilds the wasm and reloads the page when engine sources change.
- **One cargo at a time in a target dir.** Parallel helpers need their own `CARGO_TARGET_DIR`,
  absolute and under the root `out/` (README, *Build and test*): a relative one lands in `crates/`.
- **Write-up scripts under `experiments/results/` may read `web/src` source as text.** When
  you remove or move a name from `web/src`, grep `experiments/results/` for it.
- **Write files with Write/Edit, not a shell heredoc or `sed`.** Scripts that write files:
  README, *This machine*.
- **LF line endings.** `.gitattributes` sets `* text=auto eol=lf`; don't override it.
- **Times are local, 12-hour clock** (2:45pm), in reports, docs and progress lines.
- **American English everywhere:** content, docs, UI, comments and names.
- Scratch and run output go in `out/` (git-ignored), never `%TEMP%`.

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
`python scripts/warning_checks.py` (always exits 0; prints one `<kind>: <text>` line per tripped
check, nothing when clean). Its docstring defines each check:

- **parity:** the golden game's fingerprint, native and wasm, against the pins.
- **tunables:** a numeric literal in `hh-core` rule code that belongs in a config or content
  (exceptions, each with its reason: `scripts/warning_checks.json`).
- **platform-floats:** a platform float function reaching `hh-core` past clippy.
- **balance:** a target the judge misses (`hh-sim balance`; the judge and its targets are
  described in `docs/principles.md`, *The judge*).
- **docs:** a doc over 15 KB, or inside the 1 KB margin below it.

And one by hand, also kind **docs:** a rules change without its `docs/rules-history.md` row, or
a `RULES_REVISION` bump without the golden constants moved (or the reverse).

Not checked yet, Matt's to pick (thresholds would go in a config): resources at cap, buildings
or commands never used, idle stretches, simulation speed regression.
