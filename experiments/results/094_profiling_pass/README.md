# 094: profiling pass

Where the time goes in the engine's day and in the page while the game runs, early (a new
game) and late (`everything`, or the golden game's tick 1500), at 1x and 200x. Then the
cheap fixes. None of them changes play: the golden fingerprint is unmoved, the view digest
over the golden game (`view_bench`) is unchanged at `0d0a3529776e6ff2`, and `RULES_REVISION` is not bumped.

## How a wake spends its time

The clock wakes every 100 ms. A wake that owes ticks calls `advance_run` (wasm: state JSON
in, the ticks, state JSON out), then `view` (state JSON in, a view built by playing a day on
a copy and previewing a retirement, view JSON out), parses the view, and Preact redraws.
At 1x only one wake a second owes a tick, so the page is nearly idle; 200x is 20 ticks a wake.

Main, before any fix (headless Edge, busy main-thread milliseconds per wall second; the
named functions come from an unminified build):

| Case | Busy | `num` (formatter) | engine calls (`wake`) | the rest (Preact, browser) |
|---|---|---|---|---|
| early 1x | 6 | 0.6 | 2.3 | ~3 |
| early 200x | 47 | 6-7 | 22.5 | ~18 |
| late 1x | 8-9 | 1-1.3 | 2.2 | ~5 |
| late 200x | 91 | 15.5-16 | 37.6 | ~38 |

Inside one wasm call (native split, `cargo run --release -p hh-sim --example wake_bench`,
late state): parsing the state JSON 89 µs, **rebuilding `Rules` from the state's config
97 µs** (`Config::check` 48, `Config::hash` 18), one tick 54 µs, writing the state 51 µs,
the view 283 µs, writing the view 176 µs. Wasm runs these about 4-5 times slower.

Inside the view (temporary timers, golden tick 1500, 377 µs): **the next-run preview
132 µs**, the day on a copy 70, buildings 32, the rest 28, projects 22, resources 18,
breakdown 17, commands 16, and everything else under 10 each. The preview was a whole
retirement on a copy, and almost all of its cost was the gap's Attention decay: 1,825 days
simulated one by one, each allocating a key string.

## Fixes, before and after

| Fix | Where | Before | After |
|---|---|---|---|
| `num` keeps one `Intl.NumberFormat` (`toLocaleString` built one per call) | `web/src/format.ts` | 15.5-16 ms/s late 200x, 6-7 early | under 1 |
| hh-wasm caches the rules of the last config and the last state it handed out (a wake's `view` reuses its `advance_run`'s state, and no call rebuilds `Rules`) | `crates/hh-wasm` | see the wake table | same |
| The gap's Attention decay at once: `max(0, a − days × decay)`, exact (`attention::decay_days`, with a test against day by day) | `modules/attention.rs`, `career.rs` | preview 132 µs (tick 1500) | 11 µs |
| The view previews the next run without the new run's setup (`career::next_run_preview`) | `career.rs`, `view.rs` | (in the row above) | |

The view, native (`view_bench`, mimalloc, quiet box): ticks 100 / 800 / 1500,
**202 / 311 / 396 µs → 168 / 238 / 266 µs**. Sim players are view-bound, so their careers
gain about as much. Real retirements gain the same ~110 µs each (the gap is shared).

A page wake's engine calls in wasm (`wake_wasm.mjs`, each wake from the last one's state,
medians of 2-4 runs): early 1 tick **1.72 → 1.19 ms**, early 20 ticks 2.3 → 1.55, late 1
tick 2.49 → 1.70, late 20 ticks 3.8 → 3.1.

The page (A/B, main's build and the branch's served in turn, two rounds that agree within 2 ms/s):

| Case | Main | Branch |
|---|---|---|
| early 1x | 6 | 5 |
| early 200x | 47 | 34 |
| late 1x | 8-9 | 6 |
| late 200x | 91 | 69-70 |

So 200x late uses 7% of one core instead of 9%. Most of that gain is `num`; the engine's
share of a late 200x second went 37.6 → 33 ms.

## Tried, no gain

- **LTO (fat, one codegen unit) for the wasm build**: within run-to-run noise on every read.
- **`opt-level = 3` for `hh-wasm`** (it is `"s"`, the scaffold's choice; the view's
  serialization is instantiated there): within noise, and the binary grows 100 KB.
- **`Fx`'s `Display` in `u64` instead of `i128`** (every number of every state and view is
  written through it): within noise. Reverted.

## Left as notes, with estimated payoff

- **A wasm-owned game handle** instead of state JSON crossing the boundary on every call:
  with the caches, the boundary's remaining cost is writing the state once per wake and
  JS's copy of it. Estimated 0.1-0.2 ms a wake (under 2 ms/s at 200x). Small: not worth
  the API change now.
- **The view without JSON**: writing the view (~176 µs native late, about half its build)
  and `JSON.parse` of it in the page (~0.35 ms) are a third of a wake's engine time. A
  `serde-wasm-bindgen` object or a smaller per-panel view could save ~0.5-0.8 ms a wake,
  5-8 ms/s at 200x. A restructure of `engine.ts` and every panel's reads.
- **The day on a copy** (~25-70 µs native, now the view's largest part) gives every
  per-day rate. A rate computed without playing the day would duplicate rules: no.
- **Preact's redraw** (~35 ms/s late 200x, now the page's largest share): every wake
  re-renders the whole tree from a new view object. Memoizing panels on the view fields
  they read could halve it. Medium work, and only 200x late is anywhere near noticeable.
- **Autosave** (a `JSON.stringify` of state and record once a second): its cost grows
  with the career's action count. It is not visible in these reads (a scenario has no
  actions). Worth a look if a long career's page stutters each second.

## Files

- `wake_wasm.mjs`: a wake's wasm calls (`node .../wake_wasm.mjs [<another web/src/wasm>]`).
- `page_profile.mjs`: busy time and top functions (self and inclusive) per case and speed,
  against a `vite preview` (`HH_URL`).
- `crates/hh-sim/examples/wake_bench.rs`: the native split of one call's phases.
- `crates/hh-sim/examples/view_bench.rs`: now also prints the next-run preview's cost.
