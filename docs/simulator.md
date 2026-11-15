# Simulator

`hh-sim`, the native command line. Run from the repo root (paths are relative to it).

```sh
S="cargo run -q --release -p hh-sim --"
$S run --seed 1 --ticks 1700 --policy script            # the scripted opening, to a retirement
$S run --scenario rich --ticks 100 --policy greedy      # from a named starting state
$S run --scenario-file my.json --ticks 50               # from any scenario file
$S run --seed 5 --ticks 1000 --json                     # machine-readable, with the full state
$S run --seed 5 --ticks 1000 --save out/r.json          # write the game's record
$S replay out/r.json                                    # rebuild and check records
$S objectives reference/scenarios/run1_fresh.json      # a Godot optimizer scenario, scored
$S show                                                 # resolved config, hashes, revision
$S scenarios                                            # the embedded scenarios
$S golden                                               # the golden game's fingerprint
```

## Measuring play

The simulator measures balance and pacing: agents ([agents.md](agents.md)) play whole careers
under player profiles, against the targets in [principles.md](principles.md).

```sh
$S ladder                                     # the ladder: experiments/ladder.json (~35 s on 6 threads)
$S ladder --agents author --speeds 1x,mixed --seeds 8 --retirements 5   # any slice of it
$S experiment experiments/balance.json --out out/b   # any spec
$S compare --config-b configs/other.json      # two configs, paired on the same seeds
$S compare --set boredom.base_cap=500         # a param (or module.variant=name) against the base
$S balance                                    # the judge's missed targets, as balance: lines (~3 s)
$S trace --agents author --attention active --speeds 1x --retirements 2   # one career: actions, the agent's notes (#), milestones
$S agents                                     # the ladder's rungs
$S speed                                      # native engine speed
node scripts/wasm-speed.mjs                   # the same in wasm (needs crates/hh-wasm/pkg)
$S playlog C:/Code/gdrive-sync/helium-hustle/playlogs/<career>.jsonl   # a human's career (--json: one line)
```

**`playlog`** replays a web play log ([web/README.md](../web/README.md), *Play logs*) and
checks the fingerprint logged at its last tick (exit 1 on a mismatch). It measures the
career with the same `Meter` (`career.rs`) as an agent's, so its tables are the ladder's
for one human row; real time comes from the log's wall clock (each tick reached at its
share of the time between logged points; time paused before a tick counts toward it;
time with the page closed doesn't count). Then where the time went: sessions, minutes
played, hidden and closed, pauses, real minutes and ticks at each speed, actions applied
and refused, panels opened. `--json` prints the career's `GameResult` (the
`games.jsonl` line an agent's career writes) with the time use, for comparing against
sim players on the same seed.

Grid options (`ladder`, `experiment`, `compare`, `balance`, `trace`) override the spec:
`--agents a,b` (an agent's knobs stay with it: `author:a=1,b=2,greedy` is two), `--attention
a,b`, `--speeds a,b`, `--seeds N`, `--first-seed S`, `--retirements N`, `--max-ticks N`,
`--config F`, `--set module.key=value` (repeatable; added to every variant), `--threads T`
(default 6), `--profiles F`, `--milestones F`; `--out DIR` for `ladder`, `experiment` (default
`out/experiments/<name>/`) and `compare`. `compare` and `balance` default to
`experiments/balance.json`'s grid; `compare` drops the spec's variants (side A is the plain
config, side B `--config-b` and the `--set`s). `trace` plays the first cell's first seed.

**A career.** At each look the agent reads the view and acts; the speed policy picks the speed
from whether it acted and the retirements so far; the game then runs `attention × speed` ticks
to the next look, measured tick by tick. It ends at the spec's `retirements` (or `max_ticks`).
Real time is ticks over speed. Profiles: [principles.md](principles.md#profiles).

**A spec** (`experiments/*.json`): `name`, `note`, `configs`, `variants` (optional, `[{"name",
"set": ["module.key=value", "module.variant=name"]}]`), `agents`, `attention`, `speeds`,
`seeds`, `first_seed`, `retirements`, `max_ticks`, `scenario` (optional, an embedded scenario
laid over each new game), `objectives` (optional, a Godot optimizer scenario file such as
`reference/scenarios/run1_fresh.json`: its objectives scored on run 1, as `objectives` scores
them, in a *Run-1 objectives* table). The grid is configs × variants × agents × attention ×
speeds, each cell `seeds` games; results are in game order whatever the thread count.

**Output.** The tables on stdout and in `report.md`; `results.json` (`hh-experiment/1`: the
spec, revision, content hash and each cell's summary); `games.jsonl` (a line a game: every
measure, its cell, the fingerprint and `replay_ok`). Every command exits 1 if a game's record
doesn't replay to its state.
`compare` prints the paired table (B minus A on the same seeds, mean difference and its
standard error, per agent and profile) and both sides' tables; `--out` writes `compare.md`.

### Measures

| Column | Meaning |
|---|---|
| Ret | Retirements reached (mean). |
| Run 1 ticks, Run 1 min | Run 1's length in ticks and in real minutes (median). |
| Run ticks, Run min | Every finished run's length (mean). |
| Forced | Share of runs ended by boredom (the rest retired by choice). |
| Cmd boredom | Boredom's share from commands net of Dream, of all boredom gained (the rest is daily growth). |
| Idle | Share of real time with nothing to buy: no building and no research affordable. |
| Piles at cap | Resources at their cap, as a share of ticks (top three, 5% or more). |
| Overflow lost | Production lost to caps, as a share of production (top three). |
| Ticks/s | Simulated ticks per wall second, agent and measurement included. |
| Milestones | Per milestone, medians: career ticks (`t`), retirements before it (`r`; it fell in run r+1), that run's day (`d`) and real minutes from the career's start (`m`); `k/n` when only k of n seeds reached it. |
| Run credits, Peak cr/day, Ships/run | Credits earned and shipments a finished run (means); each career's best 100-day credit rate (median over seeds). |
| Run-1 objectives | With a spec's `objectives`: each objective's median run-1 day against its Godot window (`in`, `early`, `late`). |
| Never built, never used | Buildings never owned and commands never run in any seed's career. |

After the tables, the engine's own speed natively and, when `crates/hh-wasm/pkg` is built and
`node` runs, in wasm (`scripts/wasm-speed.mjs`).

**Costs** (this machine, idle): the ladder (3 agents × 15 profiles × 3 seeds, careers of 20
retirements, 135 games) about 35 s on 6 threads; `balance` (4 careers) about 3 s; a career
plays 5,000-25,000 ticks a second, slower the more often its agent looks.

| Option | Default | Meaning |
|---|---|---|
| `--config F` | `configs/baseline.json` | The game config. |
| `--seed S` | `0` | Any `u64`. |
| `--ticks N` | `1000` (`objectives`: `5000`) | Days to run (from the scenario's start tick, if any); for `objectives`, the most to play. |
| `--policy P` | `idle` | `idle`; `greedy` (each tick, buy each affordable building); `greedy:every=N`; `script`. |
| `--scenario NAME` / `--scenario-file F` | none | Lay a scenario over the new game. |
| `--save F` | | Write the game's record (`hh-record/1`). |
| `--json` | off | One JSON line: hashes, fingerprint, `replay_ok`, the state. |

`run` prints the tick, the run and its day, each resource (amount, cap, change over the
next day), the buildings owned, the career's milestones (quests, achievements, projects,
research, retirements, each with its tick, run and day), the final fingerprint and the
replay check: every game is replayed from its record and must equal the final state. Exit
codes: 0 ok, 1 a failed replay or record check, 2 bad arguments or a bad config.

`replay` prints each record's day and fingerprint, warns on another rules revision or a
fingerprint that differs, and exits 1 on any warning or failure.

## Policies

Policies live in `hh-core` (`driver.rs`) because the golden game and the wasm parity check
use one; the agents that measure play live here ([agents.md](agents.md)). Both read only the
view and act only through actions.

- **`greedy`** buys every affordable building in content order. Under Arc 1 it earns
  nothing (no program runs), so from a new game it buys nothing.
- **`script`** is a fixed Arc 1 opening, one decision a day: put free processors on Program
  1; buy the next building in its build order (Solar Panels, Excavator, Smelter, Refinery,
  Storage Depot, Launch Pad, more extraction, Fabricator, a second Data Center, Research
  Lab...), switching to a Solar Panel when the next building's energy upkeep would leave
  less than a Cloud Compute spare, to a Storage Depot when a price exceeds a cap, and buying
  land when short; then set Program 1 to one command: launch a full fuelled pad, buy fuel
  for a full pad, load He-3, bridge the next building's missing titanium, regolith or ice
  with a Buy command, else Sell Cloud Compute. It never researches, funds projects or
  retires voluntarily.

## `objectives`

Plays one of the Godot optimizer's scenario files (`reference/scenarios/*.json`) with the
script, limited to the scenario's available buildings and commands (Godot ids mapped to
the port's), from a new game until the first run's forced retirement. It prints each
objective's tick against its target window (`in`, `early`, `late`, `not hit`), when the
scenario's end condition was met, and the milestones. Objective types: `building` (with
`count`), `resource` (stockpile `threshold`), `event` `shipment_complete` and
`boredom_N`. A run takes about 0.2 s.

Seed 1 (`run1_fresh.json`): QPower at tick 52, QExtract 157, the first shipment (QShipment)
303, QAutomate 789, forced retirement 1572. The opening has no randomness, so other seeds
match it until the first shipment.
