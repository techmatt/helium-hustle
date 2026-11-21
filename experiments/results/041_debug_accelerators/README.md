# 041: debug accelerators

Debug only; no `RULES_REVISION` bump (rules 12, golden unchanged). The web debug menu's
cheats add 2k and 10k speeds, a **boost** (`Cheat::Boost`: factors on production, research
and boredom growth) and an **autopilot** (`hh_sim::autopilot`, `Cheat::Autopilot` as its
marker) that hands the game to a sim player toward the next milestone, the next main-quest
step, or everything. Docs: [docs/simulator.md](../../../docs/simulator.md#accelerators),
[docs/rules/README.md](../../../docs/rules/README.md#cheats).

## Done-when read: a fresh career, *Everything*, defaults

Boost `10,10,0.1` (production ×10, research ×10, boredom growth ×0.1); drivers
`author:stale_days=4000,dream_at=1,diversify=1 | author:stale_days=4000,dream_at=1,all_research=1`,
switching after 8,000 days with no step met; budget 200,000 days. Steps: the 14 milestones,
Q1-Q6 and Q6's four sub-objectives (24; Q7's condition is `never`, so it is no step).
Fresh seeds 3088-3093 (ledger).

| Seed | Days | Runs | Native s (6 at once) | Browser s (alone) | Browser replay check | Last step |
|---|---|---|---|---|---|---|
| 3088 | 60,805 | 24 | 17.3 | 10.9 | ok | M10 |
| 3089 | 172,594 | 48 | 37.3 | 27.5 | ok | M12 |
| 3090 | 30,087 | 14 | 10.5 | 8.8 | ok | M12 |
| 3091 | 37,316 | 11 | 11.0 | | | M10 |
| 3092 | 24,266 | 8 | 7.2 | | | M7 |
| 3093 | 91,219 | 22 | 22.1 | | | M11, M14, Q6 |

Every step met on all six. The browser's days equal the native ones on the seeds both
played (native = wasm). Browser: `timing.mjs` against `vite preview` (headless Edge);
the native column ran six careers at once, so it is slower than one alone.

The steps of the slowest career (3089) in career days: M1-M6 and Q1-Q5 by day 105; Q6's
rank-5, 100k-credit and 10-year sub-objectives by 3,650 (run 1); M4 at 4,177; M8 21,526;
M7 29,627; M10 131,237; M11, M14 and Q6 144,042; M12 172,594. The long tail is always one
of M7, M8, M10, M11, M12: steps the author reaches only in some runs.

## Tuning (dev seeds 0-5, then 3091)

| Driver set | Met all (of 6) | Missed |
|---|---|---|
| author `stale_days=4000` | 1/1 (seed 0), then 0/6 at 60k | M7 (never Dreams: boredom ×0.1 never reaches `dream_at` 60%) |
| + `dream_at=1` | 2/3 | M8 (3 goods in one run) |
| + `diversify=1` | 3/6 at 60k | M7, M10 (research skipped while a Long-Term project is open) |
| + `all_research=1` instead | 0/6 | M11 (projects starved) |
| diversify, then all_research, switch at 4,000 | 5/6 | M11 on one |
| the same, switch at 8,000 (frozen) | 6/6 dev, 6/6 fresh | |
| switch window doubling from 8,000 | 6/6, slower | |
| a third driver or `market` directive | 5/6 | M10 or M11 on one |

## Files

- `timing.mjs`: the browser read (`HH_URL`, `SEEDS`).
- `guards.mjs`: the review fixes in the browser. A speed is refused while the autopilot
  plays, Stop ends it, *Next milestone* twice moves on (the milestones met before stay
  met: `hh.pilotMet`), and the replay check passes. Seed 1: all ok.
