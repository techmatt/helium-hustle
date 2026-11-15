# 005 simulator v0: the baseline for rules revision 2

- [ladder_rev2.md](ladder_rev2.md): the full ladder, `hh-sim ladder --seeds 5 --first-seed 1000`
  (3 agents × 3 attention × 5 speeds, 5 seeds a cell, careers of 20 retirements, 225 games,
  about 100 s on 6 threads). Raw games and summaries: `out/keep/005_ladder_rev2/`
  (`games.jsonl`, `results.json`), kept by conductor.
- [compare_base_cap_1000.md](compare_base_cap_1000.md): an example `compare`,
  `--set boredom.base_cap=1000` against the baseline (the judge's grid, 4 seeds from 2000).
  A read, not a tuning: nothing here changes a number.

Seeds spent: 1-3 and 100-103 (development), 2000-2003 (the compare), 1000-1004 (the baseline,
read once on the final agents). The repo has no seed ledger yet.

## Baseline (rules revision 2, content `38f4e713790969a6`)

The author is the judge; greedy is the builder without programs. Medians over 5 seeds for run 1;
means over every run. `t`/`r`/`d`/`m`: career ticks, retirements before it, its run's day,
real minutes.

| Agent, profile | Run 1 ticks | Run 1 min | Run credits | Peak cr/day | Ships/run | Idle | M1 | M2 | M3 | M4 | M7 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| author active/1x | 1024 | 17.1 | 9,921 | 20.6 | 8.0 | 20% | d55 | d471 | d315 | d1024 | r5 (run 6) 108m |
| author active/mixed | 1106 | 10.4 | 6,140 | 19.6 | 8.0 | 45% | d65 | d466 | r2 (run 3) | d1106 | 1/5 seeds, run 16 |
| author active/10x | 1056 | 1.8 | 5,183 | 15.2 | 2.6 | 20% | d100 | d501 | d750 | d1056 | - |
| author active/200x | 1066 | 0.1 | 2,196 | 5.0 | 0 | 23% | run 8 | - | - | d1066 | - |
| author casual/1x | 1104 | 18.4 | 4,864 | 12.0 | 3.6 | 7% | d240 | d682 | - | d1104 | - |
| author afk/1x | 1064 | 17.7 | 1,316 | 5.0 | 0 | 76% | - | - | - | d1064 | - |
| greedy active/1x | 1113 | 18.6 | 5,339 | 13.1 | 9.5 | 81% | d70 | d536 | - | d1113 | - |
| random active/1x | 1152 | 19.2 | 1,123 | 4.3 | 0 | 98% | r2 (run 3) d499 | - | - | d1152 | - |

M5, M6 and M8 to M14 were reached by no agent under any profile in 20 runs. Targets: M1 run 1
day 0-350, M2 run 1 day 250-700, M3 runs 1-2, M4 end of run 1, M7 runs 4-8.

## Main misses

1. **Progress doesn't compound across runs.** Every run replays run 1: the author's credits a
   run are flat from run 2 to 20 (about 10,000), and nothing persistent is reachable (Foundation
   Grant needs 2,000 science; a lab makes 1 a day and comes about day 580). M5 to M14 (but M7)
   are never met; the design wants M5-M7 by runs 2-8 and M14 by run 20.
2. **Income plateaus near 20 credits a day.** The author's best 100-day credit rate is 20.6
   (M5 asks 25, M6 100): 1.5× cost scaling on Data Centers and panels, and caps, stop the build
   at about 3-4 processors, while energy sits at cap 45-94% of the time unspent.
3. **Run 1 is 10-18 real minutes, not 30.** At 1x a run of 1,024-1,104 ticks is 17-18 minutes;
   `mixed` (10x while waiting) 10 minutes. No profile at 1x or faster reaches 30 minutes inside
   800-1,100 ticks: the target needs a slower clock or pauses.
4. **A casual run 1 lasts 1,104 ticks, not about 1,500**: the boredom cap (500) forces every run
   out by day 1,064-1,170 unless Dream is running, and Dream needs 300 science, which a casual
   player never has. With `base_cap` 1,000 the casual run 1 is 1,584 and the active one 1,346
   ([compare](compare_base_cap_1000.md)).
5. **Speed only hurts.** Faster speeds mean fewer looks a run: at 200x the author meets M1 in
   run 8 and never ships; at 10x it earns half of 1x. M6 ("speed becomes useful") has no
   mechanism yet: nothing pays off a player for running faster.
6. **"Leave it running" can't play.** Buildings need a click, so an `afk` player (a look every
   30 minutes, one or two a run) builds nothing and meets only M4, every attention-speed pair.
7. **Never used:** Battery, Arbitrage Engine, Fuel Cell and Microwave Receiver are never built by
   the author; Overclock, Promote, Buy Power and two of the Fund commands are never run (the
   research or rank they need never comes).

The Godot run-1 objectives (run1_fresh.json) land late on the build (first excavator day 151 vs
5-20, first shipment 471 vs 260-380) and early on boredom (boredom 100 on day ~400 vs 720-1,000):
the port's boredom phases climb faster early than the Godot targets assumed.
