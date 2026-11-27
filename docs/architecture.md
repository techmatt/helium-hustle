# Architecture

How the engine is put together. The rules are in [rules/](rules/README.md); the CLI in
[simulator.md](simulator.md); the browser client in [web/README.md](../web/README.md).

## Crates

- **`hh-core`**: every rule; nothing else holds one.
- **`hh-sim`**: the native CLI: agents, careers, experiments ([simulator.md](simulator.md),
  [agents.md](agents.md)).
- **`hh-wasm`**: wasm-bindgen adapter, JSON strings in and out, no rules. It keeps two
  caches that change no result (094): the rules of the last config seen, and the last state
  it handed out with its JSON, so a call on that same string skips the parse (a page
  wake's `view` after its `advance_run`). It also links
  `hh-sim`'s library for the debug autopilot alone (a sim player driving the page's game:
  [simulator.md](simulator.md#accelerators)); `hh-sim` code it reaches must not read the
  clock (`Instant` panics in wasm) or the disk.
- **`web/`**: Vite + TypeScript + Preact. Renders the engine's view, sends actions, owns
  the wall clock ([web/README.md](../web/README.md)).

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
  missing ones). The tick plays them in the hand-written order of `Rules::tick`
  (`rules.rs`), not config or `REGISTRY` order; `REGISTRY`'s order only orders config
  checks and their errors.

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
new game: `{"name", "description", "start": {"tick", "resources", "buildings", "panels",
"programs"}}`, every key required, and optional `flags` (career flags set), `projects`
(counted completed once in their scope, rewards live: `conditional_logic` opens `if`) and `events`
(taken as fired: unlocks applied, seen, a quest among them completed); a listed resource or
building (owned and active) replaces
the new game's value, panels add to the run's enabled panels, programs replace queues from the
first (command lines only), the tick sets the run's day too and shifts the run's burst and rival schedules by
as much (they start from that day, not day 0), and processors gained go to the first program. Unknown ids are
errors naming them. `hh-sim run --scenario NAME` (or `--scenario-file F`) and the web
debug menu use them. A record carries its scenario in full, so a later edit to the file
can't change an old record. `everything` (first, the menu's default) is Matt's test-all
start: Act 1's beats 1.1-1.5 and 1.a-1.d done, every Act 1 view open, a rich working base;
the rest serve tests and e2e (`golden_start` the golden; `story_acts`, every chain quest
before Act 4's done, shows Story's four act bands). Their contents are asserted on: hh-core's
`tests/economy.rs` reads `late_unlocks`' one-energy shortage and its programs,
`tests/replay.rs` `everything`'s nav tiles, flags, active quest and processors, and replays
`rich` (as hh-sim's `tests/cli.rs` does); e2e plays `late_unlocks` (sections 2, 6, 11), `everything` (7-10), `rich` and `at_cap`
(3, 5) and `pad_loading` (5); `speculators_due` is for hand checks (speculators' first
arrival popup: 116's `check.mjs`). An edit to a scenario, or to the content it leans on, reruns both.

## Records and saves

- **Record** (`record.rs`, format `hh-record/1`): config, seed, scenario, actions
  `[{tick, action}]`, the final tick, fingerprint and rules revision. `Session` plays a
  game and keeps its record; `Session::fork` copies the game with an empty record (the
  oracle's rollouts, [oracle.md](oracle.md)).
- **Fingerprint**: FNV-1a of the state's JSON less its config, 16 hex digits: it pins play;
  the config is pinned beside it by its hash (`Config::hash`).
- **Web saves**: the state, tagged with `RULES_REVISION` and the config hash; a mismatch
  discards the save. No migration code now; an easy one is allowed when it helps development
  ([rulings/engine.md](rulings/engine.md#saves)).

## The view

`Rules::view` (`view.rs` `build`, each panel's part in a `view/` file) gives everything a
client shows and can do: resources (amount, cap, the next day's rate (before the cap's cut,
unmet demand counted), the stock's actual change, whether
the cap cuts or demand goes short that day, visible, grantable by the debug grant), the
loadout library, buildings (price, what's short, buyable, unlocked,
visible, stall, description, ideology tag, content rates per copy, the action that buys
one), land (price, per purchase, buyable), commands, programs (with each line's outcome
today and each condition's live value; the readings a condition may use), pads (fuel, whether the shared Propellant covers it, cargo per load, payout, can launch,
ready: full and can launch) and the goods a pad may be set to now (`pad_goods`), the run's totals, markets (demand
once Market Awareness is researched, else a tier), research, ideology (with the scores of
this rank and the next), projects, the active quest (with countable progress and its story), quest and
event texts (of the boredom phases, only the latest; a chain quest's with its story and payoff
for the UI's story modals), achievements, rivals, adversaries (Attention, threshold, present, seen), whether `if` is open, live overclocks and modifiers, boredom (cap, rate,
phase), the nav tiles (`content/nav.json`), the career's bests, the next run's bonuses if
the run retired now (`career::retire` on a copy), the last finished run's summary, the
run's activity feed, each resource's flow breakdown, display names by id and the cheat
mark. Clients
compute no rule; an agent reads the same view. The change per day comes from playing one
day on a copy. The view is never read by the rules and isn't in the state, fingerprint or
replay.

**Its cost.** Sim players build a view at every look (about half a ladder's CPU), so it
borrows content text (`&'static` ids, names, descriptions, content rates; `names` built
once) instead of copying it, and answers "can it apply" with blocker functions
(`economy::buy_blocker`, `land_blocker`, `research::blocker`), never `Rules::is_legal`'s
state copy; the state's config is shared (`Arc`), so a state copy doesn't copy it. What's
left is 130-270 µs natively on mimalloc (it grows with the run; about twice that on the
Windows allocator, four to five times it in wasm): the day on a copy ~25-70 µs, the
largest part, and the next run's numbers from `career::next_run_preview` (a retirement
without the new run's setup) ~5-10 µs, both exact by construction (the gap's ideology
decay stops once a day changes nothing, its Attention decay takes its days at once, 094).
`cargo run --release -p hh-sim --example view_bench` prints a view's cost split into those
parts and a digest of every view's JSON over the golden game: a refactor that only reshapes
the view keeps the digest. Where a page wake's time goes: [094's write-up](../experiments/results/094_profiling_pass/README.md).

## State

`GameState` (`state.rs`): the seed, the config, the career-wide `tick`, the RNG streams,
the `career` (what persists across retirements) and the `run` (what a retirement resets,
including the run's own `day`). Everything that affects play is in it; derived values
(caps, modifiers, the active quest, processors' assignment limits) are recomputed from it.
Granted resources (processors) are held in `run.resources` but are rewritten from the working (active, upkeep-paid)
buildings by `economy::refresh` whenever buildings change, so set the buildings, not the
amount.
