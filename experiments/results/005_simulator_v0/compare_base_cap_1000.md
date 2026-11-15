## Paired: configs/baseline.json boredom.base_cap=1000 minus configs/baseline.json (same seeds; mean difference ± its standard error)

| Agent, profile | Measure | configs/baseline.json | configs/baseline.json boredom.base_cap=1000 | Diff | SE |
|---|---|---|---|---|---|
| author active/mixed | run 1 ticks | 1075.75 | 1345.75 | +270.00 | 123.69 |
| author active/mixed | run 1 minutes | 9.89 | 13.62 | +3.73 | 1.65 |
| author active/mixed | mean run ticks | 1093.46 | 1418.12 | +324.66 | 36.04 |
| author active/mixed | idle share | 0.46 | 0.46 | +0.01 | 0.01 |
| author active/mixed | milestones reached | 4.00 | 4.75 | +0.75 | 0.25 |
| author active/mixed | M1 run | 1.00 | 1.00 | +0.00 | 0.00 |
| author active/mixed | M1 minutes | 0.33 | 0.33 | +0.00 | 0.00 |
| author active/mixed | M2 run | 1.00 | 1.00 | +0.00 | 0.00 |
| author active/mixed | M2 minutes | 2.52 | 2.52 | +0.00 | 0.00 |
| author active/mixed | M3 run | 4.50 | 6.50 | +2.00 | 1.41 |
| author active/mixed | M3 minutes | 41.25 | 85.17 | +43.92 | 27.33 |
| author active/mixed | M4 run | 1.00 | 1.00 | +0.00 | 0.00 |
| author active/mixed | M4 minutes | 9.89 | 13.62 | +3.73 | 1.65 |
| author casual/mixed | run 1 ticks | 1104.00 | 1583.75 | +479.75 | 2.84 |
| author casual/mixed | run 1 minutes | 18.40 | 23.88 | +5.48 | 0.94 |
| author casual/mixed | mean run ticks | 1103.50 | 1619.59 | +516.09 | 0.36 |
| author casual/mixed | idle share | 0.07 | 0.06 | -0.01 | 0.01 |
| author casual/mixed | milestones reached | 3.00 | 4.00 | +1.00 | 0.00 |
| author casual/mixed | M1 run | 1.00 | 1.00 | +0.00 | 0.00 |
| author casual/mixed | M1 minutes | 4.00 | 4.00 | +0.00 | 0.00 |
| author casual/mixed | M2 run | 1.00 | 1.00 | +0.00 | 0.00 |
| author casual/mixed | M2 minutes | 11.37 | 11.37 | +0.00 | 0.00 |
| author casual/mixed | M4 run | 1.00 | 1.00 | +0.00 | 0.00 |
| author casual/mixed | M4 minutes | 18.40 | 23.88 | +5.48 | 0.94 |

# Side A

## Pace (4 seeds a row; medians for run 1, means over every run)

| Agent, profile | Ret | Run 1 ticks | Run 1 min | Run ticks | Run min | Run credits | Peak cr/day | Ships/run | Forced | Cmd boredom | Idle | Piles at cap | Overflow lost | Ticks/s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| author active/mixed | 20.0 | 1104 | 9.5 | 1093 | 9.6 | 5971 | 16.9 | 8.3 | 86% | 15% | 46% | energy 95%, regolith 51%, titanium 32% | titanium 47%, he3 34%, regolith 24% | 5316 |
| author casual/mixed | 20.0 | 1104 | 18.4 | 1104 | 18.4 | 4814 | 11.1 | 3.6 | 100% | 14% | 7% | energy 100%, regolith 42%, titanium 28% | titanium 48%, regolith 35%, energy 29% | 10654 |

## Milestones (medians: `t` career ticks, `r` retirements before it (run r+1), `d` that run's day, `m` real minutes, from the career's start; `k/n` when only k of n seeds reached it)

| Agent, profile | M1 | M2 | M3 | M4 | M5 | M6 | M7 | M8 | M9 | M10 | M11 | M12 | M13 | M14 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| author active/mixed | t65 r0 d65 0m | t466 r0 d466 3m | t4545 r4 d586 43m | t1104 r0 d1104 10m | - | - | - | - | - | - | - | - | - | - |
| author casual/mixed | t240 r0 d240 4m | t682 r0 d682 11m | - | t1104 r0 d1104 18m | - | - | - | - | - | - | - | - | - | - |

Targets (run windows): M1 r1 d0-350, M2 r1 d250-700, M3 r1-2, M4 r1 d800-1100, M5 r2-4, M6 r3-5, M7 r4-8, M8 r5-8, M9 r5-10, M10 r6-10, M11 r8-15, M12 r8-15, M13 r10-18, M14 r15-20

## Never built, never used (in every seed's career)

| Agent, profile | Buildings never built | Commands never used |
|---|---|---|
| author active/mixed | electrolysis, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author casual/mixed | ice_extractor, electrolysis, research_lab, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |

# Side B

## Pace (4 seeds a row; medians for run 1, means over every run)

| Agent, profile | Ret | Run 1 ticks | Run 1 min | Run ticks | Run min | Run credits | Peak cr/day | Ships/run | Forced | Cmd boredom | Idle | Piles at cap | Overflow lost | Ticks/s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| author active/mixed | 20.0 | 1407 | 14.5 | 1418 | 13.8 | 7891 | 20.0 | 12.6 | 49% | 11% | 46% | energy 89%, titanium 45%, regolith 45% | titanium 59%, he3 39%, ice 23% | 4826 |
| author casual/mixed | 20.0 | 1586 | 23.3 | 1620 | 18.4 | 5971 | 14.4 | 9.4 | 100% | 7% | 6% | energy 96%, regolith 44%, titanium 34% | circuits 69%, titanium 64%, regolith 29% | 10586 |

## Milestones (medians: `t` career ticks, `r` retirements before it (run r+1), `d` that run's day, `m` real minutes, from the career's start; `k/n` when only k of n seeds reached it)

| Agent, profile | M1 | M2 | M3 | M4 | M5 | M6 | M7 | M8 | M9 | M10 | M11 | M12 | M13 | M14 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| author active/mixed | t65 r0 d65 0m | t466 r0 d466 3m | t6472 r4 d778 61m | t1407 r0 d1407 14m | - | - | t14460 r9 d1410 134m 3/4 | - | - | - | - | - | - | - |
| author casual/mixed | t240 r0 d240 4m | t682 r0 d682 11m | t3000 r1 d1328 41m | t1586 r0 d1586 23m | - | - | - | - | - | - | - | - | - | - |

Targets (run windows): M1 r1 d0-350, M2 r1 d250-700, M3 r1-2, M4 r1 d800-1100, M5 r2-4, M6 r3-5, M7 r4-8, M8 r5-8, M9 r5-10, M10 r6-10, M11 r8-15, M12 r8-15, M13 r10-18, M14 r15-20

## Never built, never used (in every seed's career)

| Agent, profile | Buildings never built | Commands never used |
|---|---|---|
| author active/mixed | battery, arbitrage_engine, fuel_cell, microwave_receiver | idle, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, fund_nationalist, fund_rationalist, buy_power |
| author casual/mixed | ice_extractor, electrolysis, research_lab, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
