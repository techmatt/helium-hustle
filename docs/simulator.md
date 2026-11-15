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

The simulator measures balance and pacing: sim players ([agents.md](agents.md)) play whole
careers in game time, against the targets in [principles.md](principles.md); a human time model
projects real minutes on the side.

```sh
$S ladder                                     # the ladder: experiments/ladder.json (~11 s on 6 threads)
$S ladder --agents author --humans mixed --seeds 8 --retirements 5   # any slice of it
$S experiment experiments/balance.json --out out/b   # any spec
$S compare --config-b configs/other.json      # two configs, paired on the same seeds
$S compare --set boredom.base_cap=500         # a param (or module.variant=name) against the base
$S balance                                    # the judge's missed targets, as balance: lines (~5 s)
$S trace --agents author --retirements 2      # one career: actions, the agent's notes (#), milestones
$S trace --lines                              # ... and each day's program lines run, in queue order
$S agents                                     # the ladder's rungs
$S speed                                      # native engine speed
node scripts/wasm-speed.mjs                   # the same in wasm (needs crates/hh-wasm/pkg)
$S playlog C:/Code/gdrive-sync/helium-hustle/playlogs/<career>.jsonl   # a human's career (--json: one line)
$S vs-human C:/Code/gdrive-sync/helium-hustle/playlogs/<career>.jsonl  # the sim players beside it
```

**A career.** At each look the player reads the view and acts, then says how many ticks to
skip to its next look (`Agent::next_look`): it looks whenever a decision is useful and skips
stretches that need none ([agents.md](agents.md#when-it-looks)); a forced retirement wakes it
early. The game runs those ticks, measured tick by tick, and the career ends at the spec's
`retirements` (or `max_ticks`). Results are ticks and **decisions** (looks that applied an
action). Real time is not part of play: a **human model** (`configs/sim/humans.json`,
[principles.md](principles.md#players-and-human-models)) maps a career's looks to real minutes
afterwards, so the same game reads under any model.

**`playlog`** replays a web play log ([web/README.md](../web/README.md), *Play logs*) and
checks the fingerprint logged at its last tick (exit 1 on a mismatch). It measures the
career with the same `Meter` (`career.rs`) as a sim player's (a decision: a tick where the
human applied actions), so its tables are the ladder's for one human row; its real time is
the log's wall clock (each tick reached at its share of the time between logged points;
time paused before a tick counts toward it; time with the page closed doesn't count). Then
where the time went: sessions, minutes played, hidden and closed, pauses, real minutes and
ticks at each speed, actions applied and refused, panels opened. `--json` prints the
career's `GameResult` (the `games.jsonl` line a sim career writes) with the time use.

### Against a human

**`vs-human LOG`** replays the log, plays each sim player (the ladder's, or `--agents`) on the
log's config, seed and scenario for as many ticks as the human played (`--ticks N` for more),
and prints them beside the human: ticks, retirements, each run's length, credits and
decisions (`+` marks the run under way), decisions and ticks a decision, buildings and
commands used; the milestones in career ticks and retirements; real time (the human's from
the log's clock, each player's under the ladder's human models or `--humans`), with minutes
a decision. Then the human against the **best player** (most milestones, then most credits
earned): buildings and commands only one side used, and where the human first parted from it,
matching the two build orders purchase by purchase, run by run (a purchase in a run's last
tick counts in the next run): first in **when** (the same building more than 50 ticks or a
quarter of the player's tick apart) and first in **what** (a different building, or one side
bought nothing), with each side's next five purchases. `--json`: one line, every row's
`GameResult` and the divergence. Exit 1 if the log doesn't replay to its fingerprint.

### Grids and specs

Grid options (`ladder`, `experiment`, `compare`, `balance`, `trace`) override the spec:
`--agents a,b` (players; knobs stay with their agent: `author:a=1,b=2,greedy` is two;
`casual=author:min_gap=120` names one), `--humans a,b` (human models for the real-time
table), `--seeds N`, `--first-seed S`, `--retirements N`, `--max-ticks N`, `--config F`,
`--set module.key=value` (repeatable; added to every variant), `--threads T` (default 6),
`--human-models F`, `--milestones F`; `--out DIR` for `ladder`, `experiment` (default
`out/experiments/<name>/`) and `compare`. `compare` and `balance` default to
`experiments/balance.json`'s grid; `compare` drops the spec's variants (side A is the plain
config, side B `--config-b` and the `--set`s). `trace` plays the first cell's first seed.

**A spec** (`experiments/*.json`): `name`, `note`, `configs`, `variants` (optional, `[{"name",
"set": ["module.key=value", "module.variant=name"]}]`), `agents` (players, `[name=]agent[:knobs]`,
names unique), `humans` (optional, human models), `seeds`, `first_seed`, `retirements`,
`max_ticks`, `scenario` (optional, an embedded scenario laid over each new game), `objectives`
(optional, a Godot optimizer scenario file such as `reference/scenarios/run1_fresh.json`: its
objectives scored on run 1, as `objectives` scores them, in a *Run-1 objectives* table). The
grid is configs × variants × players, each cell `seeds` games; results are in game order
whatever the thread count.

**Output.** The tables on stdout and in `report.md`; `results.json` (`hh-experiment/1`: the
spec, revision, content hash and each cell's summary); `games.jsonl` (a line a game: every
measure, each run's ticks and decisions, every building bought, its cell, the fingerprint and
`replay_ok`). Every command exits 1 if a game's record doesn't replay to its state.
`compare` prints the paired table (B minus A on the same seeds, mean difference and its
standard error, per player; run 1's minutes under the spec's first human model) and both
sides' tables; `--out` writes `compare.md`.

### Measures

| Column | Meaning |
|---|---|
| Ret | Retirements reached (mean). |
| Run 1 ticks, Run ticks | Run 1's length (median), every finished run's (mean). |
| Run 1 dec, Dec/run, Ticks/dec | Decisions (looks that acted) in run 1 (median) and a finished run (mean); career ticks a decision. |
| Forced | Share of runs ended by boredom (the rest retired by choice). |
| Cmd boredom | Boredom's share from commands net of Dream, of all boredom gained (the rest is daily growth). |
| Idle | Share of ticks with nothing to buy: no building and no research affordable. |
| Piles at cap | Resources at their cap, as a share of ticks (top three, 5% or more). |
| Overflow lost | Production lost to caps, as a share of production (top three). |
| Ticks/s | Simulated ticks per wall second, agent and measurement included. |
| Milestones | Per milestone, medians: career ticks (`t`), retirements before it (`r`; it fell in run r+1) and that run's day (`d`); `k/n` when only k of n seeds reached it. |
| Run credits, Peak cr/day, Ships/run | Credits earned and shipments a finished run (means); each career's best 100-day credit rate (median over seeds). |
| Real time | With `humans`: per player and model, run 1's real minutes (median), a finished run's (mean), minutes a decision, and each milestone's minutes from the career's start (median). |
| Run-1 objectives | With a spec's `objectives`: each objective's median run-1 day against its Godot window (`in`, `early`, `late`). |
| Never built, never used | Buildings never owned and commands never run in any seed's career. |

After the tables, the engine's own speed natively and, when `crates/hh-wasm/pkg` is built and
`node` runs, in wasm (`scripts/wasm-speed.mjs`).

**Costs** (this machine, idle): the ladder (4 players × 3 seeds, careers of 20 retirements, 12 games) about 11 s on 6
threads, set by the author's careers; the same at 5 seeds about 18 s; `balance` (4 careers)
about 5 s; `vs-human` on a 1,600-tick log about a second. A career plays 4,000-20,000 ticks a
second, slower the more often its player looks (random and the author every 5-10 ticks,
the casual author every ~130).

### Seeds

The **seed ledger**: a read that must not be tuned on (a baseline, a confirming read) takes
fresh seeds: `python scripts/seeds.py next N` prints the next N; `--take` also moves *Next
free* past them (every seed below it counts as spent). The item records its range here and in
its write-up. **Dev seeds** 1-3 are for development, on/off checks and the standing specs
(`ladder.json`, `balance.json` start at 1).

Spent: 1-3 and 100-103 (005, development), 1000-1004 (005, the rev-2 baseline), 2000-2003
(005, a compare), 3000-3004 (008, a baseline read before review fixes, superseded),
3005-3009 (008, the rev-3 baseline).

Next free: 3010

## `run` and `replay`

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
