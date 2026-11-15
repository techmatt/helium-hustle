# Architecture

How the engine is put together. The rules are in [rules/](rules/README.md); the CLI in
[simulator.md](simulator.md); the browser client in [web/README.md](../web/README.md).

## Crates

- **`hh-core`**: every rule; nothing else holds one.
- **`hh-sim`**: the native CLI: agents, careers, experiments ([simulator.md](simulator.md),
  [agents.md](agents.md)).
- **`hh-wasm`**: wasm-bindgen adapter, JSON strings in and out, no rules.
- **`web/`**: Vite + TypeScript, no UI framework. Renders the engine's view, sends
  actions, owns the wall clock.

## Core invariants

- **No I/O**: no files, network, env vars, stdout or logging. Content (`content/`) and
  scenarios (`scenarios/`) are embedded with `include_str!`; configs are handed in.
- **No clock**: time is a tick count passed in (`Rules::advance`). `std::time::Instant`
  and `SystemTime` are banned.
- **No global RNG**: randomness comes only from per-purpose streams in `GameState::rng`,
  each keyed by the seed and the FNV-1a hash of its purpose name, so a new draw in one
  system never shifts another's. Each variant declares its purposes (`VariantDef::rng`),
  never shared. Draws are `Stream::below(u32)` only: rand samples `usize` differently on
  native and wasm32.
- **No hash-order leaks**: `BTreeMap`/`Vec` only; `HashMap`/`HashSet` are banned.
- **Serializable state**: `GameState` round-trips through JSON and reloads to the same
  future. The seed and RNG positions are decimal strings (too big for JavaScript numbers);
  every `Fx` is a decimal string.
- **Exact replay**: config + seed (+ scenario) + tick-stamped actions rebuild the
  identical state (`Record::replay`); `hh-sim run` checks every game it plays.
- **Native = wasm**: `scripts/wasm-smoke.mjs` plays the golden game in wasm and must reach
  the fingerprint `golden_game` pins natively.

`crates/hh-core/clippy.toml` bans platform float functions, the clock and hash maps, and
`crates/hh-core/Cargo.toml` makes those lints `deny`: never `#[allow]` them.
`scripts/warning_checks.py` (CLAUDE.md, *Warning checks*) also looks past clippy.

## Numbers: `Fx`

Every quantity in game state is `Fx` (`fx.rs`): a private `i64` count of millionths.

- **One rounding rule: floor** (toward negative infinity), for `*`, `/`, `mul_ratio` and
  `from_f64`. Products and quotients are computed in `i128` and rounded once.
  Addition and subtraction are exact. Overflow panics in every build, so rule code that
  can meet huge values uses `checked_mul` (a price past range) or `saturating_add` (an
  uncapped running total) and decides what that means.
- **Floats** only compute factors, through `math.rs` (the `libm` crate), and become `Fx`
  at once via `Fx::try_from_f64`/`from_f64`, the one boundary (floor of the float's value,
  so a float a hair under an exact decimal lands one step low). Add a `math` function instead of calling
  an `f64` method.
- Whole powers (`Fx::pow_int`) need no float: exact to one final floor
  ([rules/README.md](rules/README.md#numbers)).
- Counts (buildings owned, the tick, days, ranks) are integers.
- The representation is private: changing its width or scale touches `fx.rs` only (and
  moves every golden constant).

## Time and actions

1 tick = 1 in-game day. The core exposes `Rules::advance(&mut g, n)` and
`Rules::apply(&mut g, &action)` between ticks; an illegal action returns `Err` and leaves
the state untouched. The UI owns the wall clock and speed ([web/README.md](../web/README.md)):
no offline progress.

## Config

A config is JSON (`configs/baseline.json`):
`{"name", "modules": {"<module>": {"variant", "params": {...}}}}`.

- **Every** parameter must be given: no defaults in code. Unknown, missing, wrong-kind
  and out-of-range keys are errors naming each one.
- `Fx` params are decimal strings (`"1.15"`, at most six places), integer params JSON
  integers; list params are JSON arrays of either, map params objects keyed by content id
  (a range bounds each element; ids are checked when `Rules::new` builds the params).
- **Config hash**: `Config::hash`, FNV-1a over the canonical resolved JSON, `name`
  excluded: `"100"` and `"100.000"` hash the same.
- Every module in `modules::REGISTRY` must be in the config (`Rules::new` names the
  missing ones); the tick plays them in `rules.rs`'s order, not config order.

## Modules and content

| Change | Kind |
|---|---|
| A number inside a mechanic | **parameter**: config only |
| Another way of doing it | **variant**: code, selected by config |
| A new mechanic | **module**: code, added to the config |
| A building, resource, command... | **content**: a row in `content/` |
| A new kind of player action | `Action` |

A module is `modules/<id>.rs` with `pub static DEF: ModuleDef` (variants, each with its
`ParamDef`s and RNG purposes) listed in `REGISTRY`, a typed `Params` built from its
config entry (a field of `modules::Params`), and the functions that play its rules. Arc
1's systems read each other every tick, so modules aren't isolated objects: `Rules`
holds every module's params and `Rules::tick` calls their functions in one order
([rules/README.md](rules/README.md#the-day)); `Rules::apply` dispatches each `Action` to
its module, on a copy of the state that replaces it only on success. Content is serde
structs in `content.rs` (unknown fields refused); `Content::check` validates ids (unique
across all content) and references. `content_hash()` is FNV-1a over the embedded files.
Shared derived values: `modifiers.rs` (the modifier framework).

The modules are Arc 1's ([rules/README.md](rules/README.md)).

## Scenarios

Named starting states (`scenarios/*.json`, listed in `scenario.rs` `FILES`), laid over a
new game: `{"name", "description", "start": {"tick", "resources", "buildings"}}`, every
key required; a listed resource or building (owned and active) replaces the new game's
value, the tick sets the run's day too, and processors gained go to the first program. Unknown ids are
errors naming them. `hh-sim run --scenario NAME` (or `--scenario-file F`) and the web
debug menu use them. A record carries its scenario in full, so a later edit to the file
can't change an old record.

## Records and saves

- **Record** (`record.rs`, format `hh-record/1`): config, seed, scenario, actions
  `[{tick, action}]`, the final tick, fingerprint and rules revision. `Session` plays a
  game and keeps its record.
- **Fingerprint**: FNV-1a of the state's JSON, 16 hex digits.
- **Web saves**: the state, tagged with `RULES_REVISION` and the config hash; a mismatch
  discards the save. No migration code, ever.

## The view

`Rules::view` (`view.rs`) gives everything a client shows and can do: resources (amount,
cap, change over the next day, visible), buildings (price, buyable, unlocked, visible,
stall, content rates per copy, the action that buys one), commands, programs, pads (fuel,
cargo per load, payout, can launch), the run's totals (credits earned, shipments, buildings, research), markets
(demand once Market Awareness is researched, else a tier), research, ideology, projects,
the active quest, the run's fired events, boredom (cap, rate, phase), unlocked panels, the
career's bests and next-run bonuses, and the career log. Clients
compute no rule; an agent reads the same view. The change per day comes from playing one
day on a copy. The view is never read by the rules and isn't in the state, fingerprint or
replay.

## State

`GameState` (`state.rs`): the seed, the config, the career-wide `tick`, the RNG streams,
the `career` (what persists across retirements) and the `run` (what a retirement resets,
including the run's own `day`). Everything that affects play is in it; derived values
(caps, modifiers, the active quest, processors' assignment limits) are recomputed from it.
