# 059 Program simplify: reads

Two reads, both against main (72656ee, built with `scripts/baseline.py`) on the same seeds.
Dev seeds only (0-3 for the autopilot, the ladder's standing 1-3): no fresh seeds taken.

## Autopilot, boot to the ending (`hh-sim accelerate --seed N`, default drivers and boost)

| Seed | Branch: 1.9 Telescope | Branch: every step (Q18) | Main: every step |
|---|---|---|---|
| 0 | t15802 | t29992 | t36205 |
| 1 | t15784 | t29933 | t36180 |
| 2 | t15816 | t29966 | t35559 |
| 3 | t11612 | t25697 | t35965 |

Every step met on every seed ("left: none"). The branch finishes 6,000 to 10,000 ticks
sooner; that read changed two things at once (the program language and 1.b's new 50-Science
grant at each run start), so it doesn't say which one moved it.

## Ladder (`hh-sim ladder`, 3 seeds, 20 retirements)

| Player | Run ticks (from run 2) | Run credits | Ships a run | Main: run ticks | Main: credits | Main: ships |
|---|---|---|---|---|---|---|
| author | 1471 | 8895 | 6.8 | 1435 | 17740 | 15.9 |
| handsoff | 961 | 2781 | 1.2 | 1505 | 19183 | 20.1 |
| greedy, casual, random | unchanged | unchanged | unchanged | | | |

Hands-off share of the author's credits from run 2: 21% (main 109%); milestones from run 2:
3 of 3 (main 3 of 6). The balance judge (`hh-sim balance`) misses the same targets as main
but M12, which main met on 1 of 2 seeds at run 15 and the branch doesn't.

**Driver.** The program-driven players lost their exact guards. With no building count, a
build order's steps can't wait on the one before (energy users get bought ahead of Solar
Panels: brownouts leave pads unpaid, so fewer launches), and finished lines (a researched
item, a built step, `load pads` with nothing to load) cost a step each loop while their
guard holds, where a guard used to skip them free. The hands-off player also loads the
author's last saved ship text at every reset, whose fuel and load percents were written
against that run's grown caps, so at base caps it buys too little fuel to launch.

**What the item changed in the players** (a first branch read had the author at 2,644
credits a run): the ship cycle buys fuel only once a pad stands and while a pad holds cargo
(fuel while one stood empty deadlocked a single loaded pad), launches only once a pad stands,
and writes its thresholds as percents of the caps when written; the build order is one line
a building, each under a credits guard. Not tuned further: the item says balance moves
aren't tuned here.
