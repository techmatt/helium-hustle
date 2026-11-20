# construct

031: the author and the casual player with their construction program (Program 5: if / build / buy land) against the same players without it (construct=0, which plays as revision 6's did), paired on seeds 101-108, careers of 5 retirements.

Rules revision 7, content 1bb305eca42caf60, 8 seeds from 101, careers of 5 retirements (at most 60000 ticks).

## Pace in game time (8 seeds a row; medians for run 1, means over finished runs)

| Player | Ret | Run 1 ticks | Run ticks | Run 1 dec | Dec/run | Ticks/dec | Run credits | Peak cr/day | Ships/run | Forced | Cmd boredom | Idle | Piles at cap | Overflow lost | Ticks/s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| author | 5.0 | 1448 | 1528 | 130 | 140 | 10.9 | 18935 | 27.3 | 10.3 | 38% | 23% | 9% | regolith 53%, energy 50%, propellant 32% | propellant 75%, regolith 44%, circuits 25% | 2298 |
| author0 | 5.0 | 1458 | 1520 | 144 | 149 | 10.2 | 18601 | 27.4 | 10.8 | 18% | 15% | 19% | regolith 53%, energy 47%, propellant 28% | propellant 71%, regolith 41%, circuits 25% | 1846 |
| casual | 5.0 | 1580 | 1570 | 14 | 13 | 119.4 | 8914 | 15.2 | 15.6 | 82% | 10% | 11% | energy 95%, regolith 63%, titanium 54% | circuits 63%, titanium 62%, regolith 25% | 6489 |
| casual0 | 5.0 | 1581 | 1583 | 13 | 12 | 127.4 | 8780 | 14.0 | 16.8 | 82% | 9% | 12% | energy 95%, regolith 66%, titanium 56% | circuits 68%, titanium 66%, regolith 26% | 4791 |

## Milestones (medians: `t` career ticks, `r` retirements before it (run r+1), `d` that run's day; `k/n` when only k of n seeds reached it)

| Player | M1 | M2 | M3 | M4 | M5 | M6 | M7 | M8 | M9 | M10 | M11 | M12 | M13 | M14 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| author | t60 r0 d60 | t420 r0 d420 | t247 r0 d247 | t1448 r0 d1448 | t1074 r0 d1022 7/8 | - | t3888 r2 d1120 | - | - | - | - | t970 r0 d970 1/8 | - | - |
| author0 | t60 r0 d60 | t420 r0 d420 | t247 r0 d247 | t1458 r0 d1458 | t1758 r0 d1016 | - | t2816 r1 d1126 | - | - | - | - | - | t3320 r1 d1832 1/8 | - |
| casual | t240 r0 d240 | t682 r0 d682 | t1440 r0 d1440 | t1580 r0 d1580 | - | - | - | - | - | - | - | - | - | - |
| casual0 | t240 r0 d240 | t682 r0 d682 | t1440 r0 d1440 | t1581 r0 d1581 | - | - | - | - | - | - | - | - | - | - |

Targets (run windows): M1 r1 d0-350, M2 r1 d250-700, M3 r1-2, M4 r1, M5 r2-4, M6 r3-5, M7 r4-8, M8 r5-8, M9 r5-10, M10 r6-10, M11 r8-15, M12 r8-15, M13 r10-18, M14 r15-20

## Real time (projected by a human model, configs/sim/humans.json: real minutes; medians for run 1 and milestones, means over finished runs)

| Player | Human | Run 1 min | Run min | Min/dec | M1 | M2 | M3 | M4 | M5 | M6 | M7 | M8 | M9 | M10 | M11 | M12 | M13 | M14 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| author | mixed | 20.7 | 22.1 | 0.16 | 1m | 6m | 4m | 21m | 16m | - | 57m | - | - | - | - | 14m | - | - |
| author0 | mixed | 21.6 | 22.6 | 0.15 | 1m | 6m | 4m | 22m | 26m | - | 42m | - | - | - | - | - | 50m | - |
| casual | mixed | 5.4 | 5.3 | 0.40 | 1m | 2m | 5m | 5m | - | - | - | - | - | - | - | - | - | - |
| casual0 | mixed | 5.4 | 5.3 | 0.43 | 1m | 2m | 5m | 5m | - | - | - | - | - | - | - | - | - | - |

## Never built, never used (in every seed's career)

| Player | Buildings never built | Commands never used |
|---|---|---|
| author | battery, fuel_cell, microwave_receiver | idle, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, fund_nationalist, fund_rationalist, buy_power |
| author0 | battery, arbitrage_engine, fuel_cell, microwave_receiver | idle, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, fund_nationalist, fund_rationalist, buy_power |
| casual | ice_extractor, electrolysis, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| casual0 | ice_extractor, electrolysis, research_lab, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |

## Run-1 objectives (reference/scenarios/run1_fresh.json; median day, against the Godot target window)

| Player | excavator [5, 20] | smelter [40, 100] | refinery [100, 200] | he3_10 [140, 220] | first_ship [260, 380] | data_center [380, 530] | research_lab [600, 800] | retirement [720, 1000] |
|---|---|---|---|---|---|---|---|---|
| author | 71 late | 88 in | 288 late | 297 late | 420 late | 248 early | 461 early | 373 early |
| author0 | 71 late | 88 in | 288 late | 297 late | 420 late | 248 early | 462 early | 374 early |
| casual | 361 late | 361 late | 481 late | 490 late | 682 late | 1441 late 7/8 | 1441 late 1/8 | 434 early |
| casual0 | 361 late | 361 late | 481 late | 490 late | 682 late | 1441 late 7/8 | - | 434 early |

engine speed, native: 1802 ticks/s scripted (golden game, a view a tick), 33198 ticks/s plain advance
engine speed, wasm: not measured (build crates/hh-wasm/pkg: README, Build and test)
