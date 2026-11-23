# 061 Hands-off redefined

**New definition (Matt).** The `handsoff` player loads the best known loadout at every reset;
its only hand move is a stale-run retirement. Its programs are data:
`configs/sim/handsoff_loadout.txt`, read by `crates/hh-sim/src/agents/loadout.rs` (060's
harness form) and built into the player. Runs until `if` opens (and that one) stay the
author's. No rules or content changed; the author plays the same games (its pins hold).

## L5 over L4

The ladder's measure is credits a finished run from run 2 on, and milestones first met from
run 2 on, against the author on the same seeds. On 060's paired final read (seeds 3119-3124,
10 retirements; its kept games.jsonl, `share.py` here):

| Player | Credits a run from run 2 | Share of the author | Milestones from run 2 | Ships a run from run 2 |
|---|---|---|---|---|
| author | 8,447 | - | 8 | 5.7 |
| old handsoff | 2,806 | 33% | 8 | 1.2 |
| L4 two tabs | 17,958 | 213% | 8 | 10.4 |
| **L5 Dream tab** | **23,738** | **281%** | 8 | 12.9 |

Milestones tie (no player passes M5 and M11 in 10 retirements), so credits decide: **L5**.
060's caveat stands: a few L5 runs stall near 8k (Dream's He-3 starves the pad); its runs end
by the stale-run retirement, not boredom.

The shipped player replays 060's L5 read exactly: on seeds 3119-3124 its credits are equal
run for run to 060's harness (fingerprints differ only by the library saves, `hands-off N`,
which the harness never made).

## Runs 1-2

Kept as the author's: every 060 loadout guards with `if`, which reads false until Conditional
Logic, so loading it earlier leaves the player mostly idle (060, *Phase*). No cleaner choice.

## The ladder (`hh-sim ladder`, seeds 1-3, 20 retirements)

| Build | Hands-off credits a run from run 2 | Author's | Share | Milestones | Ships a run (all runs) | Forced |
|---|---|---|---|---|---|---|
| main 59e302c | 1,745 | 8,180 | 21% | 3 of 3 | 1.2 | 7% |
| 061 | 22,708 | 8,180 | **278%** | 3 of 3 | 12.8 | 10% |

`if` opens in run 2 on all three seeds; every hands-off run from run 3 on ships (7-15 a run)
and builds on every seed.

## Fresh read (seeds 3125-3130, 10 retirements)

| Player | Credits a run from run 2 | Share | Milestones | Credits a run after `if` opens (SE) | Ships a run | Run days | Forced |
|---|---|---|---|---|---|---|---|
| author | 9,482 | - | 8 | 6,635 (418) | 4.2 | 1,596 | 80% |
| handsoff | 23,064 | **243%** | 8 | 23,179 (548) | 13.2 | 2,069 | 5% |

`if` opens in run 4 on two seeds (the author plays runs 1-4 there), run 2 on the rest. Every
hands-off run ships and builds; one run on each of two seeds stays under 10k (060's stall).

## Pins

`PINNED`'s handsoff rows (seeds 1-2, 2 retirements) don't move: those careers end before `if`
opens' next run, so they are the author's games, and never pinned the old player's loadouts
either. A new pin, `HANDSOFF_PIN` (`crates/hh-sim/tests/cli.rs`, slow lane), plays seed 1 to
run 3 and checks the loadout ships and builds there. A new loadout file moves it.

Reads kept outside the repo: conductor `kept/helium-hustle/061_handsoff_redefine/061_reads`.
