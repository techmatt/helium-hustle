# 137: profiling sim speed

Where the loops we run over and over spend their time, and three play-neutral fixes. Base
`d9ab590` (main when the item started), built by `scripts/baseline.py` into
`out/baseline/d9ab590/` and run from its own root. Every number below is an **upper bound**:
the box was not locked quiet, so the timings are paired runs taken back to back, base then
new. Compare the ratios, not the absolute times: the same base read took 105.6 s at 1:03am and
125.2 s at 1:38am.

**Play is unchanged.** Every read below gives the same tables on both sides, apart from wall
time and the speed columns. The golden fingerprint, every pin (`PINNED`, `HANDSOFF_PIN`, the
oracle's) and the view digest over the golden game (`view_bench`: `a8c77bd39e2c046d`, the same
on both sides) hold, and `RULES_REVISION` is unchanged.

## Results

| Target | Read | Base | New | Ratio |
|---|---|---|---|---|
| Skilled `axes` | `axes --skills skilled --seeds 3` (9 oracle careers, 6 threads), back to back | 125.2 s | 73.1 s | 0.58 |
| Skilled `axes`, 1 seed | `axes --skills skilled` (3 careers, 6 threads) | 69.7 s | 33.0 s | 0.47 |
| One oracle career | `axes --skills skilled --axes humanist --threads 1` | 51.2 s | 38.6 s | 0.75 |
| `cargo test-all` | warm build, run phase (xtask's "side by side") | 48.9 s | 37.5 s | 0.77 |
| `cargo test-all` | wall, warm (build check included) | 49.2 s | 37.8 s | 0.77 |
| Ladder | `ladder` (15 careers, 6 threads) | 26.7 s | 18.6 s | 0.70 |
| `balance` | 4 careers | 14.2 s | 8.8 s | 0.62 |
| Engine, native | the ladder's footer: golden game with a view each tick / bare `advance` | 2,052 / 44,774 ticks/s | 3,178 / 79,234 ticks/s | 1.55× / 1.77× |
| A view, golden tick 800 / 1500 | `view_bench`, two rounds | 303-333 / 358-375 µs | 252-272 / 267-283 µs | ~0.83 / ~0.75 |
| `playlog --today` | a 158-action log | 0.27 s | 0.28 s | (too small to matter) |

`cargo test-all`'s two longest binaries: hh-sim's lib (the autopilot's career) 48.9 → 37.5 s,
and its `cli` tests 33.3 → 26.2 s. Cold (a fresh worktree's first run) it was 2 min 49 s at
base, the build ~2 min of it; the fixes change only the run phase.

**From 094**, not re-measured: the page's split at 1x and 200x (Preact's redraw the largest
share, the engine ~33 ms/s late 200x), the boundary's costs (`Rules` cached, state JSON) and
its no-gain list (LTO, `opt-level = 3` for wasm, `Fx` display in `u64`). This item started
where 094 stopped: its view was 266 µs on the golden game's tick 1500 and sim players are
view-bound, so the sim players' loop (rollouts, views, the tick) is what this one profiled. The first skilled read, taken before the engine fixes with
only the thread lending in place and not back to back, was 105.6 → 81.1 s.

## The profile

No sampling profiler runs here: `samply` and the Windows Performance Toolkit need an elevated
shell. So the profile used temporary timers: thread-summed atomic counters in hh-sim, and a
scripted wrap of each field of `view::build` and each phase of `Rules::tick`. They are not
committed in the code. The recipe, here: `prof.rs.txt` (copy it to `crates/hh-sim/src/prof.rs`
and add `pub mod prof;` to `lib.rs`), `instr_view.py` and `instr_tick.py` (they wrap
`view::build`'s fields and `Rules::tick`'s phases), then `crate::prof::time(slot, || ...)`
around what to time and `eprint!("{}", crate::prof::report())` at the end. Revert it all with
`git checkout -- crates`.

**A skilled career** (humanist seed 1, one thread, ~50 s net of the timers):

- Rollouts take essentially all of it (56 s timed, including timer overhead).
- Inside the rollouts, the author's looks take 39 s, and nearly all of that is views: 93k
  views at 318 µs each. Ticks take 16 s (48 µs each). The milestone tracker (0.5 s) and the
  forks (0.01 s) are noise.
- Strategy choices took 8 s and tactics plans 48 s. Every one of them is rollouts.

Inside a view (318 µs):

| Part | µs |
|---|---|
| The next-run preview | 64 |
| The day on a copy | 62 |
| Buildings | 37 |
| Projects | 25 |
| Commands | 19 |
| Breakdown | 17 |
| Resources | 17 |
| Research | 10 |
| Readings | 10 |
| Verbs | 7 |
| Everything else | under 6 each |

The preview costs this much here, against 13 µs on the golden game, because a committed
player carries all three axes: the gap's ideology decay steps all 1,825 days for each nonzero
axis. Its floor never settles on a positive value at a 365-day half-life.

**After the fixes** (the same career, one thread, 37 s with the view timers on): a view is
241 µs (was 318), and views are still ~64% of the career. The day on a copy is 48.5 µs (was
62), buildings 33, the next-run preview 27 (was 64: the `i64` floor), projects 23, commands
17, breakdown 17 and resources 12.

**`axes` threads:** the 9 cases took 444 thread-seconds over 6 threads, so the ideal is 74 s
against the 105.6 s measured. The last wave left threads idle for ~30% of the wall time.

**The autopilot's career** (`a_fresh_career_on_the_autopilot_reaches_every_step`, test
profile, ~34 s alone):

- Views take 14 s and the tick 13 s (130 µs a tick at `opt-level = 2` with overflow checks).
- Inside the tick: `economy::resolve` 6.1 s, `programs::run` 3.4, `economy::clamp` 2.4,
  `produce_free` 1.2, `fold_flow_average` 1.1.
- Most of `clamp` and much of `resolve` was `modifiers::modifier`. It scanned every
  achievement, every quest unlock, every project's rewards and every research item, each time
  it was asked, and `cap()` asks it once per resource.

## Fixes

| Fix | Where | Effect |
|---|---|---|
| **`modifier()` reads a once-built index** of each key's sources (achievements, quest unlocks, project rewards, research), each list in content order, so the floor products are taken in the same order, bit for bit. Built from `content()` in a `OnceLock`, as `view::names()` is. | `hh-core/src/modifiers.rs` | Autopilot test 35.6 → 30.3 s alone; tick 137 → 106 µs (test profile); `clamp` 2.4 → 1.1 s; the native engine 1.55-1.77× |
| **Rollouts on lent threads**: a grid worker with no case left lends its thread to its grid's pool (`hh-sim::pool`, one pool a grid, found through a thread-local set by each worker), and a look's independent rollouts (`plan`'s screen, its long list, `choose_strategy`'s table) borrow what is free. Results come back in order, a grid never passes its `--threads`, and borrowed threads go back on unwind too (the diff review's two findings: a first process-wide counter could wrap when two grids ran in one process, as the `cli` tests do). | `hh-sim/src/pool.rs`, `oracle/score.rs`, `oracle/strategy.rs`, `axes.rs`, `experiment.rs` (`par_map`) | 9-career read 105.6 → 81.1 s before the engine fixes; 3-career read 0.47 |
| **`Fx` floor division in `i64`** when both operands fit (`i128` division is a software routine, natively and in wasm). The `i128` path is kept for the rest and for `i64::MIN / -1`. A test checks both paths against an `i128` reference over 400+ values of every sign. | `hh-core/src/fx.rs` | ~3-5% on a view alone (`view_bench`); part of the engine figures |
| `economy::stored` looks up a building's working count only when the building stores the resource. | `modules/economy.rs` | Small, inside `cap()` |

**Tried, no gain, reverted:**

- `Player::run()` in place of `p.view().run` in `career::drive` and the autopilot. Views went
  14.2 → 14.4 s: the post-action rebuild it was meant to skip rarely happens.
- `produce`/`consume`/`fold_flow_average` without per-call key `String` copies: 30.3 → 32.9 s,
  within noise. mimalloc makes those allocations cheap.

## Measured, not fixed (expected gain)

- **A view for sim players** that leaves out what no sim player reads. `hh-sim` reads none of
  `next_run`, `names`, `nav`, `story`, `event_texts`, `breakdown`, `activity`,
  `achievement_list`, `rivals`, `adversaries`, `modifiers` or `verbs`. That was ~100 µs of a
  ~320 µs view in a skilled career, and is ~60 µs of 241 µs after the fixes: ~15% of a
  skilled read now. Not built: it contradicts
  `docs/rulings/sim.md` *View* (009: "Sim players keep the full view"). Asked as question
  `helium-hustle__137_profiling_sim_speed__1`; the default taken is to keep the full view.
- **The next-run preview's gap** (27 µs of a committed player's view after the fixes, 64 before): 1,825 floor-multiply
  steps per nonzero axis. The steps are exact and have no closed form under floor rounding. A
  per-`Rules` memo keyed on (score, factor, days) would skip them only when scores repeat
  between views. Up to ~11% of a skilled view now; how often scores repeat is unmeasured.
- **The cheap state-reading rollout policy** (paged.md): it changes play, so it needs paired
  skilled bars and re-pins. Its ceiling is the views' share of rollout time, ~60%: about 2× on
  skilled reads. A design item of its own, as paged.md says.
- **The day on a copy** (62 µs of a view, a state clone plus a tick): it gives every per-day
  rate. Gains would come from a cheaper tick (below) or a cheaper clone. The state is mostly
  `BTreeMap<String, …>`, and interning its keys would be a broad state refactor.
- **The tick**: `economy::resolve` (formatted source keys, `BTreeMap` products a building)
  and `programs::run` are the largest left. A sampler would name the lines. Expected 10-20% of
  a tick from targeted work.
- **The web page at high speeds**: the engine fixes reach wasm too: the `modifier` index and
  the `i64` floor, and wasm's `i128` division is slower than native's. But this item did not
  rebuild the wasm or re-run 094's page profile (`page_profile.mjs`, `wake_wasm.mjs`). The
  late 200x engine share was ~33 ms/s after 094; expect it to fall by roughly the view's
  ~20-25%.
- **`vs-human`**: no play log on disk is on today's rules (revision 49). The newest is 45, and
  logs from before 135 carry no play record. So `vs-human` could not be timed. `playlog
  --today` takes 0.3 s.
- **The wasm build, `npm test`, e2e and qa**: not timed. This item touches no web code; their
  costs are build-bound (wasm ~80 s cold) or browser-bound.

## Files

- `reads.sh` (ladder, balance, vs-human, `--today`, 3-career axes), `axes9.sh`, `testall.sh`
  (each side warmed once, then timed base, new) and `bench.sh` (`view_bench`, `wake_bench`, one
  oracle career): the paired reads, base built by `python scripts/baseline.py <rev>`. Their
  paths name this item's worktree: edit `W` to rerun.
- `prof.rs.txt`, `instr_view.py`, `instr_tick.py`: the timer recipe above.
