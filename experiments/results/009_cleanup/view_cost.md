# 009_cleanup: the view's cost, before and after

004 noted that `Rules::view` plays a day on a copy of the state for `per_day` on every view.
Measured first, on this machine (6 cores, nothing beside), release builds.

## Where a view's time went (before)

`cargo run --release -p hh-sim --example view_bench` at three points of the golden game:

| Golden tick | View | State copy (of it: config) | Copy + one day (`per_day`) | Copy + retire (`next_run`) |
|---|---|---|---|---|
| 100 | 344 µs | 22 µs (14) | 41 µs | 32 µs |
| 800 | 435 µs | 23 µs (8) | 50 µs | 41 µs |
| 1500 | 390 µs | 27 µs (9) | 59 µs | 46 µs |

Instrumenting the ladder (12 careers): 104,101 views took 64 thread-seconds, of which the
`per_day` peek was 9.2 (14%). So the day on a copy was not the cost; copying content text
into every view was (names, descriptions, content rates, a `names` map rebuilt each time),
plus two more state copies for `can_buy_land` (`Rules::is_legal` copies, then `apply` copies
again). The UI pays little: it builds
a view per clock wake, not per tick or per frame.

## What changed (no play change)

- The view borrows content (`&'static str`, `&'static Amounts`, `&'static Requirement`;
  `StoryView.body` a `Cow` for the `{rate}` fill; `names` built once). Same JSON.
- `can_buy_land` from `economy::land_blocker` (which `buy_land` now uses), not `is_legal`.
- `GameState.config` is an `Arc<Config>` (serde `rc`: same JSON), so a state copy no longer
  deep-copies the config: cheaper peeks, retire copies and every `Rules::apply`.
- `per_day` and `next_run` still play the day and the retirement on a copy: exact by
  construction. Replacing them is a design call (report, *Deferred*).

## After

| Golden tick | View | State copy | Copy + one day | Copy + retire |
|---|---|---|---|---|
| 100 | 123 µs | 7 µs | 24 µs | 20 µs |
| 800 | 184 µs | 15 µs | 42 µs | 31 µs |
| 1500 | 200 µs | 18 µs | 49 µs | 33 µs |

- **Same output:** the digest of every view's JSON over the 2000-tick golden game is
  `3900436dab1b8345` before and after (69,194,560 bytes of JSON); the ladder's 12 games are
  field-for-field identical except `wall_s`; golden constants unmoved.
- **Ladder:** 20.2 s → 13.3 s wall with the view refactor → 11.4 s with the shared config
  (5 seeds: ~35 s → 18.2 s; `balance` ~7 s → 4.8 s). Career speeds now 4,400 (random) to
  19,600 (casual) ticks/s.
- **`hh-sim speed`, scripted golden game (a view a tick):** ~2,400 → ~3,800 (view refactor)
  → ~4,500 ticks/s; plain advance unchanged (~85,000). Wasm after: 4,585 scripted, 77,868
  plain (`scripts/wasm-speed.mjs`; no before read in wasm).
