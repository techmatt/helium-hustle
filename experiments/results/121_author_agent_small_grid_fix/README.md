# 121: the author reaches `if` on every small-grid seed

Rules revision 43, unchanged here: both fixes are in the agents (player code).

## The read before the fix (seeds 1-4, 5 retirements, author)

The seeds that failed had moved since 113 (rules 38). Under rules 43 seeds 1 and 2 opened `if`
in run 3, and seeds **3 and 4** did not. Those runs 2-5 were each 727 days long, with 0
shipments and 13 buildings, retired as stale. Run 1 on every seed ended at Boredom's cap
(about 1,600 days) with 8 to 17 buildings.

## Two bugs (found with `hh-sim trace --at`)

1. **A pad at 99.8/100 never fills** (`hand_plan`, `author/templates.rs`). The loads the plan
   writes were `floor(capacity - cargo)` rounded up to loads, so 0.2 free cargo counted as no
   room. He-3 comes in fractions a day, so the pad stopped at 99.8/100. It was never full, so
   it never launched, and it stayed that way for whole runs. Seed 3, run 2, day 687: pad 1 He-3
   99.8/100, `can_launch` true, and the program was `buy_regolith x11, cloud_compute x40`, with
   no load and no launch. The fuel short (`floor(fuel - propellant)`) and the stock count had
   the same flaw. All three now round up (`agents::ceil_int`).
2. **1.2's Titanium pad kept Titanium all run** (`agents::pick_cargo`). The look that
   hand-launched 1.2's shipment wrote a plan that reloaded the pad with Titanium the same day.
   That was before 1.2 completed, while `pad_goods` was still Titanium only. `pick_cargo` turned
   only an *empty* first pad back to He-3, so the pad shipped Titanium for the rest of the
   run. Meanwhile the author was buying Titanium for its Solar Panel target. Seed 1, run 1, day
   1000: pad on Titanium at 93/100, He-3 at its 50 cap, 8 buildings. A loaded first pad now
   switches too, when its cargo fits back in its stockpile (a switch returns it, per
   docs/rules/shipments.md). This is the hand that `pick_cargo` and docs/agents.md already
   described ("1.2's Titanium pad turns to He-3 after it").

Fix 2 alone (ablation, same seeds): fewer shipments, and seed 4 still never opened `if` in 5 runs.
Both stay.

## After (seeds 1-4, 5 retirements; `out/121_author_agent_small_grid_fix/try.py final 4 5`)

| Seed | `if` opens (author) | Author ships a run | Hands-off loadout runs: ships, buildings |
|---|---|---|---|
| 1 | run 3 | 8, 10, 10, 14, 13 | run 5: 5, 21 |
| 2 | run 3 | 8, 10, 15, 11, 11 | run 5: 5, 21 |
| 3 | run 3 | 6, 19, 12, 10, 12 | runs 4-5: 5, 5; 21, 21 |
| 4 | run 3 | 9, 5, 3, 10, 6 | runs 4-5: 5, 4; 21, 21 |

Run 1 now builds 34 to 42 buildings (it was 8 to 17). The hands-off player opens `if` a run
later than the author on seeds 1 and 2 (run 4). Its games differ from the author's from run 3
on, before the loadout loads, which is the case 113's README already described.

Before, per seed in the same order: `if` opened in run 3 or 3 / never / never, and the author's
ships a run were 6,14,4,12,4 / 7,10,7,3,9 / 9,0,0,0,0 / 9,0,0,0,0.

## Pins moved

`PINNED`'s author, casual and handsoff rows, `HANDSOFF_PIN` and the oracle's four
(`crates/hh-sim/tests/cli.rs`). Random and greedy kept their games. The hands-off test now
plays 5 runs on seed 1 and checks the run after the one that completes Conditional Logic,
instead of a fixed run 3. Run 3 had been the author's since rules 38 moved `if` to run 3.

Rerun: build `hh-sim` release, then
`python out/121_author_agent_small_grid_fix/try.py <tag> 4 5` (a copy is in this folder:
`try.py`, which finds the repo two levels up from `out/<item>/`).
