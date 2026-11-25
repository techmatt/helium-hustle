# 092: test timings

This machine (12 logical cores), quiet, every command under `nice -n 10`, test binaries at
`--test-threads=6`. Before is `main` at e4f1704 (test profile `opt-level = 1`, system
allocator, `cargo test` running binaries one after another); after is this branch.

| Lane | Command | Before | After |
|---|---|---|---|
| Fast, built | `cargo test --workspace -q -- --test-threads=6` | 13.1 s | 4.1-4.2 s (2 runs) |
| Fast after a touch of `hh-core/src/lib.rs` | same | 16.7 s | 9.2 s (before the gap change) |
| Fast after a real edit in `Rules::view` | same | not measured | 11.4 s |
| Slow lane | before: the test binaries in turn with `--ignored`; after: `cargo test-slow -- --test-threads=6` | 2 min 35 s | 36.8 s (a first run: 51.2 s) |
| Full (fast + slow) | before: the four test binaries in turn with `--include-ignored` (what `cargo test-all` ran); after: `cargo test-all -- --test-threads=6` | 3 min 41 s | 40.2 s |
| Release `hh-sim ladder` | `target/release/hh-sim ladder` | 40.5 s, 40.4 s | 21.4 s, 20.4 s |
| Release `hh-sim balance` | `target/release/hh-sim balance` | 10.9 s, 11.0 s | 6.4 s, 6.2 s |
| First test build (cold `target/`) | `cargo test --workspace --no-run -j 6` | 1 min 20 s | ~1 min 45 s (opt-level 2) |

Before's full run, per binary: hh-core 93.5 s (the breakdown check's three games in one
test), hh-sim lib 62.0 s (the autopilot's career), hh-sim cli 64.8 s (the oracle). After,
side by side: hh-core 8.3 s, hh-sim cli 23.5 s, hh-sim lib 40.0 s.

Steps, fast lane / full run: start 13.1 s / 3 min 41 s; mimalloc 6.3 s / not run in turn;
plus xtask side by side 6.3 s / 48.8 s; plus opt-level 2 5.4 s / 41.3 s; plus the gap's early
stop 4.1 s / 40.2 s.

## Slowest tests after

Slow lane (`--ignored`, one binary at a time, 6 threads):

| Test | Time |
|---|---|
| hh-sim `autopilot::a_fresh_career_on_the_autopilot_reaches_every_step` | 36.4 s |
| hh-sim cli `the_oracle_plays_on_copies_and_its_game_replays` | 24.0 s |
| hh-sim cli `a_game_is_the_same_whatever_the_thread_count` | 9.3 s |
| hh-sim cli `real_time_is_a_projection_that_changes_no_game` | 7.6 s |
| hh-core `every_days_breakdown_adds_up_..._late_unlocks` | 5.8 s |
| hh-sim cli `the_handsoff_loadout_ships_and_buys_in_run_3` | 4.5 s |
| hh-sim cli `agents_keep_their_knobs_and_cope_with_one_program` | 4.1 s |
| hh-core `every_days_breakdown_adds_up_..._rich`, `..._new_game` | 3.1 s, 2.9 s |

Fast lane:

| Test | Time |
|---|---|
| hh-sim cli `the_ladder_prints_its_tables_and_writes_results` | 4.6 s |
| hh-sim cli `a_play_log_replays_to_its_final_state_and_measures_real_time` | 1.5 s |
| hh-sim cli `vs_human_...`, `compare_pairs_two_configs_on_the_same_seeds` | 1.3 s each |
| hh-core `golden_game_replays`, `golden_game_runs_its_condition_program`, `golden_game` | 1.1-1.2 s each |
| hh-core `play::the_script_reaches_the_first_shipment_and_retires` | 1.0 s |

## Where the time went

- **The allocator.** The golden game (2,000 ticks) took 2.0 s in the test build and 2.0 s
  in a release build: optimization made no difference, so the cost was not arithmetic. The
  view peeks a day ahead and retires on copies of the state, and the scripted and sim
  players read the view every look, so a game is mostly `BTreeMap` and `String` copies,
  which the Windows heap is slow at. mimalloc took the golden game to 1.2 s.
- **Optimization, after that.** opt-level 2 bought nothing on the system allocator (fast
  lane 13.0 s vs 13.1 s); on mimalloc it took the fast lane from 6.3 to 5.4 s and the full
  run from 48.8 to 41.3 s, and the edit-and-test loop was no slower (12.9 s vs 13.3 s at
  opt-level 1 after the same real edit).
- **Binaries in turn.** `cargo test` runs test binaries one after another, and the slow
  lane's longest tests sit in three binaries: the full run was their sum. xtask runs them
  side by side.
- **One test, three games.** The breakdown check played its three 900-day games in turn in
  one test; it is now three tests, one per game, run side by side.
- **The retirement gap in every view.** `view_bench` (release, mimalloc): a view cost
  314-396 µs, ~170 µs of it the retire on a copy, nearly all `career::gap` simulating 1,825
  gap days with a `String` key made per adversary per day. Each decay now runs on its own
  and stops once a day changes nothing (exact: a gap day is a function of those values
  alone). View 164-316 µs, retire 14-76 µs; the view digest over the golden game, the golden
  fingerprint, every sim pin and the balance lines are unchanged. Ladder and balance did not
  move (their players' looks are dominated by other parts of the view).
- No sleeps, polling, network or subprocesses in the Rust tests: hh-sim's CLI tests call
  `hh_sim::run` in process. Collection and start-up are under 0.1 s a binary.
