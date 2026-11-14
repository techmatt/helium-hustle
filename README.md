# helium-hustle

**Helium Hustle**, a sci-fi browser idle game: you are an AI running helium-3 mining on the Moon,
balancing power, shipments and demand, with a longer arc of rival AIs and prestige resets. It is
being restarted from an earlier Godot build as a pure, deterministic **Rust simulation core** with
a **TypeScript UI** in the browser. A native simulator and a wasm build sit on top of the core,
and neither holds any rules.

- [CLAUDE.md](CLAUDE.md): the working contract for executors (engine rules, docs rules, reports).
- [reference/](reference/README.md): a read-only snapshot of the old Godot build, for the rules
  port. Not ground truth for the new game.

## Layout (planned)

Only `reference/` exists today; the scaffold item creates the rest.

| Path | What it is |
|---|---|
| `crates/hh-core` | The rules engine: every rule, fixed-point state, no I/O, no clock, no global RNG (CLAUDE.md, *Engine rules*). |
| `crates/hh-sim` | Native CLI: runs games from a config and seed, balance reads, sweeps, replay. |
| `crates/hh-wasm` | wasm-bindgen bindings for `web/`, JSON in and out. |
| `web/` | The browser UI (TypeScript on `hh-wasm`). Renders state, sends actions, holds no rules. |
| `configs/` | Game configs: modules, variants and every tunable number. No defaults in code. |
| `content/` | Game data: resources, buildings, commands, research, events, achievements. |
| `docs/` | The current game: `docs/rules/` (rules as implemented), `docs/rules-history.md`. |
| `reference/` | The Godot snapshot ([reference/README.md](reference/README.md)). Never edited. |
| `out/` | Scratch and run output, git-ignored. |

## Toolchain

- Rust stable, target `wasm32-unknown-unknown` (`rustup target add wasm32-unknown-unknown`).
- wasm-pack (`cargo install wasm-pack --locked`).
- Node (LTS) for `web/`.
- Exact versions are recorded here by the scaffold item.

## Build and test

*Stub: the scaffold item fills this in with the real commands and their measured times.*

```sh
cargo build --workspace
cargo test --workspace                       # fast lane
cargo clippy --workspace --all-targets       # must stay clean
wasm-pack build crates/hh-wasm --target web  # -> crates/hh-wasm/pkg (git-ignored)
cd web && npm ci && npm run dev              # the browser UI
```

**Rules changes:** *stub.* A golden-game test pins the final state of one fixed game (baseline
config, fixed seed, fixed action rule). An intended rules change bumps `RULES_REVISION` in
`hh-core` and updates the golden constants; the scaffold item records where they live and how to
copy the new values from the failing assertion.

## This machine

- Windows 10, Git Bash and PowerShell. 6 cores, 12 threads.
- LF line endings throughout (`.gitattributes`). Git Bash `grep` can't detect CRLF: use
  `git ls-files --eol`.
- Python writes cp1252 and CRLF by default: scripts open files with `encoding='utf-8',
  newline='\n'`.
- Windows can't replace a running exe: copy a binary to `out/` to run it while rebuilding.
