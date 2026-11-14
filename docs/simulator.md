# Simulator

`hh-sim`, the native command line. Run from the repo root (paths are relative to it).

```sh
S="cargo run -q --release -p hh-sim --"
$S run --seed 1 --ticks 365 --policy greedy:every=7    # a year, buying every week
$S run --scenario rich --ticks 100 --policy greedy      # from a named starting state
$S run --scenario-file my.json --ticks 50               # from any scenario file
$S run --seed 5 --ticks 1000 --json                     # machine-readable, with the full state
$S run --seed 5 --ticks 1000 --save out/r.json          # write the game's record
$S replay out/r.json                                    # rebuild and check records
$S show                                                 # resolved config, hashes, revision
$S scenarios                                            # the embedded scenarios
$S golden                                               # the golden game's fingerprint
```

| Option | Default | Meaning |
|---|---|---|
| `--config F` | `configs/baseline.json` | The game config. |
| `--seed S` | `0` | Any `u64`. |
| `--ticks N` | `1000` | Days to run (from the scenario's start tick, if any). |
| `--policy P` | `idle` | `idle`; `greedy` (each tick, buy each affordable building); `greedy:every=N`. |
| `--scenario NAME` / `--scenario-file F` | none | Lay a scenario over the new game. |
| `--save F` | | Write the game's record (`hh-record/1`). |
| `--json` | off | One JSON line: hashes, fingerprint, `replay_ok`, the state. |

`run` prints the day, each resource (amount, cap, expected change per day), buildings
owned, the final fingerprint and the replay check: every game is replayed from its record
and must equal the final state. Exit codes: 0 ok, 1 a failed replay or record check, 2 bad
arguments or a bad config.

`replay` prints each record's day and fingerprint, warns on another rules revision or a
fingerprint that differs, and exits 1 on any warning or failure.

Policies live in `hh-core` (`driver.rs`) because the golden game uses one; agents proper
will live here.
