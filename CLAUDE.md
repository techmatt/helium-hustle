# CLAUDE.md

The standing contract for every executor working in this repo. How to build and test, and this
machine's traps, are in the [README](README.md), which also maps the layout.

## What this repo is

**Helium Hustle**, a sci-fi browser idle game: you are an AI running helium-3 mining on the Moon.
It is being restarted from an earlier Godot build as a **Rust simulation core** with a
**TypeScript UI**.

- `hh-core` simulates every rule. `hh-sim` (native CLI: runs, sweeps, balance reads) and `hh-wasm`
  (bindings for `web/`) sit on top of it. **Clients hold no rules**: the UI renders state and sends
  actions; it never computes a cost, a rate or an unlock itself.
- `reference/` is a read-only snapshot of the Godot build, there to port rules from. It is **not
  ground truth** for the new game: where it and `docs/rules/` disagree, `docs/rules/` wins. Never
  edit it.
- Don't describe or document code as if it does more than it does. Read `docs/rules/README.md`,
  then only the files your task touches.

## Engine rules

Locked by Matt: a change to any of these is his decision, not an implementation detail.

- **`hh-core` holds every rule, with no I/O, no clock, no global RNG.** Time is a tick count passed
  in, never read from the system. Randomness comes from per-purpose streams derived from the game
  seed (one stream per purpose, so a new draw in one system never shifts another's).
- **No hash-order leaks.** Nothing that affects play iterates a `HashMap`/`HashSet`; use
  `BTreeMap`, `Vec` or an index order.
- **State is serde-serializable**, the whole of it: a saved game reloads to the same future.
- **Exact replay, native = wasm.** The same config, seed and action list give bit-identical state
  natively and in wasm.
- **Fixed-point state.** Every quantity in game state is `Fx(i64)` at 10^-6 (1.0 = 1_000_000).
  Products and quotients go through `i128` intermediates and **one rounding rule**, defined once
  in `Fx` and used everywhere; no ad-hoc `as` casts or rounding at call sites.
- **Floats only for computing factors** (a cost-scaling multiplier, a curve), converted to `Fx`
  at once, and **only through the `libm` crate**. Platform `f64` transcendentals (`powf`, `powi`,
  `exp`, `ln`, `log*`, `sin`, `cos`, `tan`, ...) are banned in `hh-core` by clippy
  `disallowed-methods` (`crates/hh-core/clippy.toml`): they differ across platforms and break
  native = wasm. Don't `#[allow]` the lint; use `libm`.
- **`RULES_REVISION` and a pinned golden game.** Any change to how games play out bumps
  `RULES_REVISION` and updates the golden constants (README, *Rules changes*). Bump from the
  constant, not from a revision number in the prompt, and say so in the rules-history row and the
  report. A rename or refactor that plays the same moves no golden constant: if it does, play
  changed.
- **Every tunable number lives in a config** under `configs/`, with **no defaults in code**: a
  missing key is a load error, never a silent fallback. A test may build its own config.
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
- **Each doc keeps under about 15 KB** (README, CLAUDE.md and every `docs/` page, `docs/rules/`
  included). Leave about 1 KB free: an item that fills a page trims it first, cutting what the
  code or a generated table already says, never moving text to an archive.
- **Every rules change** updates the `docs/rules/` files it touches and adds its row to
  `docs/rules-history.md` (revision, what changed, why) in the same commit.
- **Facts learned while working go in the page for their area**, never only in a report: build,
  test and this machine in `README.md`; a run's cost and recipe in the docs page for runs.

## Working rules

- **Commit to the branch you are on.** Leave nothing uncommitted. Never commit to or push `main`.
- **Test lanes** are in the README, *Build and test*. Run the fast lane after every change and
  the full run before every commit; a commit touching only Markdown skips the full run. Leave
  `cargo clippy --workspace --all-targets` clean.
- **Background any step over ~30 seconds** (builds, tests, sims, servers) and wait on the real
  process (the `cargo`, `node` or test binary), never on a launcher that returns before the work
  is done. A fresh worktree's first build takes minutes: give it its time.
- **One cargo at a time in a target dir.** Parallel helpers need their own `CARGO_TARGET_DIR`.
- **Write files with Write/Edit, not a shell heredoc or `sed`.** A file a script writes passes
  `encoding='utf-8', newline='\n'`.
- **Times are local, 12-hour clock** (2:45pm), in reports, docs and progress lines.
- **American English everywhere:** content, docs, UI, comments and names.
- **LF line endings.** `.gitattributes` sets `* text=auto eol=lf`; don't override it.
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

Warnings are surfaced, never gate: an item whose own change trips one finishes and reports it in
the header's `warnings:` line, each line led by its kind. Every item runs, before its report:

`python scripts/warning_checks.py` prints one `<kind>: <text>` line per tripped check, nothing
when clean (always exit 0; about 7 s, up to a minute if it must build). Its checks, on:

- **parity:** native = wasm: the golden game's fingerprint natively (`hh-sim golden`) and in
  wasm (`scripts/wasm-smoke.mjs`, rebuilding a stale `crates/hh-wasm/pkg`) equals the pins.
- **tunables:** a numeric literal in `hh-core` rule code (not tests) that belongs in a config
  or content. Exceptions, each with its reason: `scripts/warning_checks.json`.
- **platform-floats:** a platform float function reaching `hh-core` past clippy (a call by
  name not through `libm`, an `#[allow]` of the ban, the ban's `deny` or a name dropped).
- **balance:** a target the judge misses (`hh-sim balance`: milestones outside their run
  windows, run 1's length and real time; targets and judge in `docs/principles.md`).

And two by hand:

- **docs:** a doc over 15 KB (`wc -c README.md CLAUDE.md docs/*.md docs/rules/*.md web/README.md`).
- **docs:** a rules change without its `docs/rules-history.md` row, or a `RULES_REVISION` bump
  without golden constants moved (or the reverse).

Candidates the simulator now measures but no check reads yet (Matt picks, thresholds in a
config): resources sitting at cap; buildings never built or commands never used; idle stretches
with nothing worth buying; simulation speed regression against a baseline.
