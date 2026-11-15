# ladder

The agent ladder in game time: each player looks whenever a decision is useful and skips stretches that need none; careers of 20 retirements from a new game, run 1 scored against reference/scenarios/run1_fresh.json's objectives. Real time is projected by the human models in `humans`.

Rules revision 3, content 06d9cf540641e1af, 5 seeds from 3005, careers of 20 retirements (at most 60000 ticks).

## Pace in game time (5 seeds a row; medians for run 1, means over finished runs)

| Player | Ret | Run 1 ticks | Run ticks | Run 1 dec | Dec/run | Ticks/dec | Run credits | Peak cr/day | Ships/run | Forced | Cmd boredom | Idle | Piles at cap | Overflow lost | Ticks/s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| random | 20.0 | 1646 | 1664 | 330 | 333 | 5.0 | 1584 | 3.9 | 0.0 | 100% | 3% | 98% | energy 87%, titanium 71%, regolith 69% | he3 93%, titanium 90%, circuits 52% | 2123 |
| greedy | 20.0 | 1609 | 1636 | 124 | 128 | 12.8 | 7072 | 10.9 | 17.2 | 100% | 5% | 77% | energy 100%, titanium 78%, he3 32% | titanium 84%, he3 37%, energy 11% | 5543 |
| author | 20.0 | 1343 | 1492 | 136 | 146 | 10.2 | 12945 | 24.7 | 12.4 | 22% | 12% | 16% | regolith 45%, energy 28%, circuits 20% | propellant 56%, he3 23%, circuits 22% | 4285 |
| casual | 20.0 | 1560 | 1590 | 11 | 12 | 128.5 | 8997 | 16.2 | 17.3 | 90% | 9% | 9% | energy 92%, regolith 69%, titanium 57% | circuits 69%, titanium 67%, regolith 27% | 16200 |

## Milestones (medians: `t` career ticks, `r` retirements before it (run r+1), `d` that run's day; `k/n` when only k of n seeds reached it)

| Player | M1 | M2 | M3 | M4 | M5 | M6 | M7 | M8 | M9 | M10 | M11 | M12 | M13 | M14 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| random | t1205 r0 d1205 | - | - | t1646 r0 d1646 | - | - | - | - | - | - | - | - | - | - |
| greedy | t72 r0 d72 | t515 r0 d515 | - | t1609 r0 d1609 | - | - | - | - | - | - | - | - | - | - |
| author | t60 r0 d60 | t420 r0 d420 | t247 r0 d247 | t1343 r0 d1343 | t22875 r14 d1098 2/5 | - | t2277 r1 d1204 | t10281 r6 d1436 2/5 | - | - | - | - | t10505 r6 d1918 3/5 | - |
| casual | t240 r0 d240 | t682 r0 d682 | t1200 r0 d1200 | t1560 r0 d1560 | - | - | - | - | - | - | - | - | - | - |

Targets (run windows): M1 r1 d0-350, M2 r1 d250-700, M3 r1-2, M4 r1, M5 r2-4, M6 r3-5, M7 r4-8, M8 r5-8, M9 r5-10, M10 r6-10, M11 r8-15, M12 r8-15, M13 r10-18, M14 r15-20

## Real time (projected by a human model, configs/sim/humans.json: real minutes; medians for run 1 and milestones, means over finished runs)

| Player | Human | Run 1 min | Run min | Min/dec | M1 | M2 | M3 | M4 | M5 | M6 | M7 | M8 | M9 | M10 | M11 | M12 | M13 | M14 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| random | careful | 54.9 | 55.5 | 0.17 | 40m | - | - | 55m | - | - | - | - | - | - | - | - | - | - |
| random | mixed | 35.7 | 36.1 | 0.11 | 26m | - | - | 36m | - | - | - | - | - | - | - | - | - | - |
| random | brisk | 25.6 | 25.9 | 0.08 | 19m | - | - | 26m | - | - | - | - | - | - | - | - | - | - |
| greedy | careful | 37.1 | 38.0 | 0.30 | 2m | 12m | - | 37m | - | - | - | - | - | - | - | - | - | - |
| greedy | mixed | 21.7 | 22.2 | 0.17 | 1m | 7m | - | 22m | - | - | - | - | - | - | - | - | - | - |
| greedy | brisk | 15.1 | 15.5 | 0.12 | 1m | 5m | - | 15m | - | - | - | - | - | - | - | - | - | - |
| author | careful | 33.7 | 37.0 | 0.25 | 2m | 10m | 6m | 34m | 561m | - | 58m | 254m | - | - | - | - | 252m | - |
| author | mixed | 20.3 | 22.2 | 0.15 | 1m | 6m | 4m | 20m | 335m | - | 35m | 152m | - | - | - | - | 149m | - |
| author | brisk | 14.3 | 15.6 | 0.11 | 1m | 4m | 2m | 14m | 235m | - | 24m | 106m | - | - | - | - | 104m | - |
| casual | careful | 11.3 | 11.9 | 0.96 | 2m | 5m | 9m | 11m | - | - | - | - | - | - | - | - | - | - |
| casual | mixed | 5.1 | 5.3 | 0.43 | 1m | 2m | 4m | 5m | - | - | - | - | - | - | - | - | - | - |
| casual | brisk | 2.5 | 2.6 | 0.21 | 0m | 1m | 2m | 3m | - | - | - | - | - | - | - | - | - | - |

## Never built, never used (in every seed's career)

| Player | Buildings never built | Commands never used |
|---|---|---|
| random | ice_extractor, electrolysis, launch_pad, research_lab, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy | ice_extractor, fabricator, electrolysis, research_lab, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_ice, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author | battery, arbitrage_engine, fuel_cell, microwave_receiver | idle, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, fund_nationalist, fund_rationalist, buy_power |
| casual | ice_extractor, electrolysis, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |

## Run-1 objectives (reference/scenarios/run1_fresh.json; median day, against the Godot target window)

| Player | excavator [5, 20] | smelter [40, 100] | refinery [100, 200] | he3_10 [140, 220] | first_ship [260, 380] | data_center [380, 530] | research_lab [600, 800] | retirement [720, 1000] |
|---|---|---|---|---|---|---|---|---|
| random | 958 late 4/5 | 1088 late 4/5 | 1241 late 2/5 | 1250 late 2/5 | - | - | - | 525 early |
| greedy | 83 late | 103 late | 145 in | 154 in | 515 late | - | - | 416 early |
| author | 71 late | 88 in | 288 late | 297 late | 420 late | 248 early | 462 early | 374 early |
| casual | 361 late | 361 late | 481 late | 490 late | 682 late | 1201 late | - | 434 early |

engine speed, native: 2103 ticks/s scripted (golden game, a view a tick), 77115 ticks/s plain advance
engine speed, wasm: not measured (build crates/hh-wasm/pkg: README, Build and test)
