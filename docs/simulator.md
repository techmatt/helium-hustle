# Simulator

`hh-sim`, the native command line. Run from the repo root (paths are relative to it).

```sh
S="cargo run -q --release -p hh-sim --"
$S run --seed 1 --ticks 1200 --policy script            # the scripted opening, to a retirement
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
use one; agents proper will live here. They read only the view and act only through actions.

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
303, QAutomate 789, forced retirement 1098. The opening has no randomness, so other seeds
match it until the first shipment.
