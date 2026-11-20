# 031: programs gain `if` blocks, `build`, `buy land` and `wait until`

Rules revision 7. The rules are in [docs/rules/programs.md](../../../docs/rules/programs.md);
this folder holds the sim read.

## What the author's construction program changed

`construct.json` plays the author and the casual player (the author looking every 120 days)
with the construction program (Program 5, `construct` 1, the default now) beside the same
players without it (`construct=0`, which plays as revision 6's players did: the rules change
moves no game that queues only commands). Paired on seeds 101-108, careers of 5 retirements,
32 games. Full tables: [construct_report.md](construct_report.md).

```sh
cargo run -q --release -p hh-sim -- experiment experiments/results/031_program_conditions_and_build/construct.json --out out/construct   # ~25 s
```

| Player | Run credits | Peak cr/day | Run 1 ticks | Decisions/run | Forced | Command boredom | Idle |
|---|---|---|---|---|---|---|---|
| author, construction program | 18,935 | 27.3 | 1448 | 140 | 38% | 23% | 9% |
| author without | 18,601 | 27.4 | 1458 | 149 | 18% | 15% | 19% |
| casual, construction program | 8,914 | 15.2 | 1580 | 13 | 82% | 10% | 11% |
| casual without | 8,780 | 14.0 | 1581 | 12 | 82% | 9% | 12% |

- **Credits barely move** (+2% a run for both players, inside the seed-to-seed spread at 8
  seeds). The author already looks the day its target comes within reach (`plan_gap`), so
  buying between looks gains it little; it needs 6% fewer decisions a run.
- **More runs end forced.** Command boredom rises from 15% to 23% of the run's boredom and
  forced retirements from 18% to 38% of runs; ticks with nothing affordable fall from 19% to
  9% (credits are spent the day they suffice). The driver isn't isolated: Program 5's
  processor runs the filler (Sell Cloud Compute) every day it doesn't build, and a run that
  keeps buying never goes stale, so it isn't retired by choice.
- **Milestones:** M5 falls from career tick 1758 to 1074 (7 of 8 seeds), M7 moves the other
  way (2816 to 3888). One seed reached M12 with it; one reached M13 without. Not separable
  from noise at 8 seeds.
- **The casual player** builds a Research Lab in 1 of 8 careers (never without) and its peak
  income rises 14.0 to 15.2 credits a day: with looks 120 days apart, the program is the only
  thing buying in between.

The construction program, as the author writes it for a target `T`:

```
if Land free < T's land and Credits >= the land price      (only while short of land)
    buy land
if Credits >= T's price and <T's scarcest other input> >= its price
    build T up to owned + 1
<filler> x1
```

## In the browser

`web/scripts/e2e.mjs` section 6 writes a program of the brief's shape through the editor
(scenario `late_unlocks`) and checks it: the unlock, display names only, live values on each
condition, a Battery built unattended, the wait holding until Credits pass 800, then the
Storage Depot. The brief's own program (Solar Panels and Data Centers) is the core test
`tests::logic::the_briefs_program_builds_a_base_unattended`.
