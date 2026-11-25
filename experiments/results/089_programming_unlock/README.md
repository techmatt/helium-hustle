# 089: Programming unlock

A new game shows Commands only: each Commands card runs by hand (Run), and there is no
Programs tile, right-column program, Add to Program or loadout buttons
([commands-only.png](commands-only.png)). **Programming** (research, 200 science, on sale
once 1.4 Full Load completes) sets the career flag `programming` for good. It opens all of
those and is never sold again.

## Run-1 timing

[timing.md](timing.md), from [timing.py](timing.py): main against 089 on paired seeds 31-40,
careers of 2 retirements, agents author, greedy and random.

- **Run 1 barely moves.** The author's median run 1 runs 1,432 → 1,446 days; greedy and
  random are unchanged. Every milestone inside run 1 (M1-M4) reaches the same tick. 1.4
  completes late in run 1, so the 200 science is mostly spent in run 2.
- **Run 2 pays for it.** The author's M5 (positive credit flow, runs 2-4) falls from 8/10 to
  4/10 careers inside two retirements. Buy Science at 10 credits a unit makes Programming
  about 2,000 credits of early-game income.
- **Hands-off, seed 1.** Conditional Logic (`if`) used to complete in run 2; it now
  completes in run 4, so the loadout first plays in run 5. hh-sim's
  `the_handsoff_loadout_ships_and_buys_in_run_5` (was run 3) checks that run. On this seed
  the author ships nothing in run 3.

The sims program from day 1 because the engine doesn't gate programs (as ruled: UI only).
So these numbers count only the 200 science. A human also loses programs before Programming,
which no sim player models.
