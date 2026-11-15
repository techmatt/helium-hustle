# ladder

The agent ladder over the attention × speed grid: careers of 20 retirements from a new game, run 1 scored against reference/scenarios/run1_fresh.json's objectives.

Rules revision 2, content 38f4e713790969a6, 5 seeds from 1000, careers of 20 retirements (at most 60000 ticks).

## Pace (5 seeds a row; medians for run 1, means over every run)

| Agent, profile | Ret | Run 1 ticks | Run 1 min | Run ticks | Run min | Run credits | Peak cr/day | Ships/run | Forced | Cmd boredom | Idle | Piles at cap | Overflow lost | Ticks/s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| random active/1x | 20.0 | 1152 | 19.2 | 1158 | 19.3 | 1123 | 4.3 | 0.0 | 100% | 4% | 98% | energy 84%, regolith 55%, titanium 53% | he3 89%, titanium 86%, regolith 41% | 4016 |
| random active/mixed | 20.0 | 1152 | 19.2 | 1158 | 19.3 | 1123 | 4.3 | 0.0 | 100% | 4% | 98% | energy 84%, regolith 55%, titanium 53% | he3 89%, titanium 86%, regolith 41% | 4275 |
| random active/10x | 20.0 | 1148 | 1.9 | 1162 | 1.9 | 923 | 5.0 | 0.0 | 100% | 3% | 89% | energy 85%, regolith 23%, titanium 14% | he3 91%, circuits 86%, titanium 78% | 13087 |
| random active/ramp | 20.0 | 1152 | 19.2 | 1171 | 1.5 | 397 | 3.8 | 0.0 | 100% | 1% | 98% | energy 99%, regolith 13% | regolith 92%, he3 58%, energy 55% | 13067 |
| random active/200x | 20.0 | 1170 | 0.1 | 1172 | 0.1 | 281 | 5.0 | 0.0 | 100% | 1% | 98% | energy 100% | energy 58% | 14958 |
| random casual/1x | 20.0 | 1143 | 19.1 | 1163 | 19.4 | 867 | 5.0 | 0.0 | 100% | 3% | 81% | energy 90%, regolith 40%, titanium 9% | circuits 94%, regolith 85%, he3 73% | 12652 |
| random casual/mixed | 20.0 | 1143 | 19.1 | 1163 | 19.4 | 867 | 5.0 | 0.0 | 100% | 3% | 81% | energy 90%, regolith 40%, titanium 9% | circuits 94%, regolith 85%, he3 73% | 12466 |
| random casual/10x | 20.0 | 1170 | 1.9 | 1170 | 2.0 | 406 | 5.0 | 0.0 | 100% | 1% | 97% | energy 100% | energy 57% | 15719 |
| random casual/ramp | 20.0 | 1143 | 19.1 | 1176 | 1.7 | 110 | 2.9 | 0.0 | 100% | 0% | 97% | energy 100% | energy 59% | 15353 |
| random casual/200x | 20.0 | 1170 | 0.1 | 1177 | 0.1 | 22 | 0.0 | 0.0 | 100% | 0% | 99% | energy 100% | energy 60% | 15783 |
| random afk/1x | 20.0 | 1170 | 19.5 | 1173 | 19.6 | 220 | 2.9 | 0.0 | 100% | 1% | 99% | energy 100% | energy 59% | 15670 |
| random afk/mixed | 20.0 | 1170 | 19.5 | 1173 | 19.6 | 220 | 2.9 | 0.0 | 100% | 1% | 99% | energy 100% | energy 59% | 15962 |
| random afk/10x | 20.0 | 1170 | 1.9 | 1177 | 2.0 | 54 | 1.0 | 0.0 | 100% | 0% | 99% | energy 100% | energy 60% | 16268 |
| random afk/ramp | 20.0 | 1170 | 19.5 | 1176 | 3.1 | 64 | 1.0 | 0.0 | 100% | 0% | 95% | energy 100% | energy 60% | 14270 |
| random afk/200x | 20.0 | 1170 | 0.1 | 1177 | 0.1 | 22 | 0.0 | 0.0 | 100% | 0% | 99% | energy 100% | energy 60% | 15398 |
| greedy active/1x | 20.0 | 1113 | 18.6 | 1124 | 18.7 | 5339 | 13.1 | 9.5 | 100% | 10% | 81% | energy 100%, titanium 68%, he3 38% | titanium 77%, he3 42%, energy 12% | 4629 |
| greedy active/mixed | 20.0 | 1122 | 6.7 | 1133 | 8.2 | 3697 | 8.8 | 4.8 | 100% | 9% | 82% | energy 100%, titanium 65%, regolith 63% | titanium 78%, he3 42%, regolith 22% | 6178 |
| greedy active/10x | 20.0 | 1115 | 1.9 | 1129 | 1.9 | 3062 | 8.8 | 1.6 | 100% | 10% | 55% | energy 100%, regolith 70%, titanium 49% | titanium 72%, he3 57%, regolith 32% | 8826 |
| greedy active/ramp | 20.0 | 1113 | 18.6 | 1167 | 1.4 | 759 | 9.2 | 0.8 | 100% | 2% | 74% | energy 100%, titanium 10%, regolith 7% | titanium 74%, he3 51%, energy 38% | 12599 |
| greedy active/200x | 20.0 | 1070 | 0.1 | 1172 | 0.1 | 302 | 5.0 | 0.0 | 100% | 1% | 100% | energy 100% | energy 58% | 15119 |
| greedy casual/1x | 20.0 | 1122 | 18.7 | 1130 | 18.8 | 2340 | 5.0 | 0.0 | 100% | 9% | 28% | energy 100%, regolith 62%, he3 39% | he3 89%, regolith 64%, titanium 61% | 11568 |
| greedy casual/mixed | 20.0 | 1122 | 18.7 | 1130 | 18.8 | 2340 | 5.0 | 0.0 | 100% | 9% | 28% | energy 100%, regolith 62%, he3 39% | he3 89%, regolith 64%, titanium 61% | 11622 |
| greedy casual/10x | 20.0 | 1064 | 1.8 | 1172 | 2.0 | 266 | 5.0 | 0.0 | 100% | 1% | 100% | energy 100% | energy 58% | 15439 |
| greedy casual/ramp | 20.0 | 1122 | 18.7 | 1168 | 1.6 | 473 | 5.0 | 0.0 | 100% | 2% | 23% | energy 100% | titanium 71%, he3 55%, energy 54% | 16283 |
| greedy casual/200x | 20.0 | 1064 | 0.1 | 1172 | 0.1 | 266 | 5.0 | 0.0 | 100% | 1% | 100% | energy 100% | energy 58% | 14598 |
| greedy afk/1x | 20.0 | 1064 | 17.7 | 1172 | 19.5 | 266 | 5.0 | 0.0 | 100% | 1% | 100% | energy 100% | energy 58% | 15703 |
| greedy afk/mixed | 20.0 | 1064 | 17.7 | 1172 | 19.5 | 266 | 5.0 | 0.0 | 100% | 1% | 100% | energy 100% | energy 58% | 15799 |
| greedy afk/10x | 20.0 | 1064 | 1.8 | 1172 | 2.0 | 266 | 5.0 | 0.0 | 100% | 1% | 100% | energy 100% | energy 58% | 16133 |
| greedy afk/ramp | 20.0 | 1064 | 17.7 | 1172 | 3.1 | 266 | 5.0 | 0.0 | 100% | 1% | 100% | energy 100% | energy 58% | 16188 |
| greedy afk/200x | 20.0 | 1064 | 0.1 | 1172 | 0.1 | 266 | 5.0 | 0.0 | 100% | 1% | 100% | energy 100% | energy 58% | 16145 |
| author active/1x | 20.0 | 1024 | 17.1 | 1178 | 19.6 | 9921 | 20.6 | 8.0 | 94% | 10% | 20% | regolith 45%, energy 45%, circuits 26% | propellant 44%, circuits 31%, regolith 21% | 3801 |
| author active/mixed | 20.0 | 1106 | 10.4 | 1109 | 9.7 | 6140 | 19.6 | 8.0 | 93% | 15% | 45% | energy 94%, regolith 49%, titanium 32% | titanium 44%, he3 33%, regolith 24% | 5707 |
| author active/10x | 20.0 | 1056 | 1.8 | 1091 | 1.8 | 5183 | 15.2 | 2.6 | 100% | 17% | 20% | energy 99%, regolith 48%, titanium 44% | titanium 60%, he3 49%, regolith 23% | 8244 |
| author active/ramp | 20.0 | 1024 | 17.1 | 1024 | 1.3 | 3921 | 17.7 | 0.9 | 24% | 17% | 25% | energy 98%, titanium 79%, regolith 9% | titanium 45%, he3 36%, circuits 33% | 11853 |
| author active/200x | 20.0 | 1066 | 0.1 | 1133 | 0.1 | 2196 | 5.0 | 0.0 | 100% | 9% | 23% | energy 100%, titanium 43% | regolith 88%, energy 44% | 16019 |
| author casual/1x | 20.0 | 1104 | 18.4 | 1104 | 18.4 | 4864 | 12.0 | 3.6 | 100% | 14% | 7% | energy 100%, regolith 42%, titanium 28% | titanium 48%, regolith 35%, energy 29% | 10446 |
| author casual/mixed | 20.0 | 1104 | 18.4 | 1104 | 18.4 | 4864 | 12.0 | 3.6 | 100% | 14% | 7% | energy 100%, regolith 42%, titanium 28% | titanium 48%, regolith 35%, energy 29% | 10494 |
| author casual/10x | 20.0 | 1064 | 1.8 | 1143 | 1.9 | 1669 | 5.0 | 0.0 | 100% | 7% | 68% | energy 100%, titanium 29% | energy 48% | 15794 |
| author casual/ramp | 20.0 | 1104 | 18.4 | 1128 | 1.4 | 825 | 9.5 | 0.3 | 95% | 3% | 10% | energy 100%, titanium 15% | energy 51%, titanium 44%, circuits 30% | 17502 |
| author casual/200x | 20.0 | 1064 | 0.1 | 1172 | 0.1 | 266 | 5.0 | 0.0 | 100% | 1% | 100% | energy 100% | energy 58% | 15999 |
| author afk/1x | 20.0 | 1064 | 17.7 | 1151 | 19.2 | 1316 | 5.0 | 0.0 | 100% | 5% | 76% | energy 100%, titanium 23% | energy 51% | 13560 |
| author afk/mixed | 20.0 | 1064 | 17.7 | 1151 | 19.2 | 1316 | 5.0 | 0.0 | 100% | 5% | 76% | energy 100%, titanium 23% | energy 51% | 15997 |
| author afk/10x | 20.0 | 1064 | 1.8 | 1170 | 1.9 | 381 | 5.0 | 0.0 | 100% | 2% | 97% | energy 100% | energy 57% | 15459 |
| author afk/ramp | 20.0 | 1064 | 17.7 | 1168 | 3.1 | 477 | 5.0 | 0.0 | 100% | 2% | 97% | energy 100% | energy 57% | 15788 |
| author afk/200x | 20.0 | 1064 | 0.1 | 1172 | 0.1 | 266 | 5.0 | 0.0 | 100% | 1% | 100% | energy 100% | energy 58% | 15930 |

## Milestones (medians: `t` career ticks, `r` retirements before it (run r+1), `d` that run's day, `m` real minutes, from the career's start; `k/n` when only k of n seeds reached it)

| Agent, profile | M1 | M2 | M3 | M4 | M5 | M6 | M7 | M8 | M9 | M10 | M11 | M12 | M13 | M14 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| random active/1x | t2790 r2 d499 46m | - | - | t1152 r0 d1152 19m | - | - | - | - | - | - | - | - | - | - |
| random active/mixed | t2790 r2 d499 46m | - | - | t1152 r0 d1152 19m | - | - | - | - | - | - | - | - | - | - |
| random active/10x | t3450 r2 d1000 6m | - | - | t1148 r0 d1148 2m | - | - | - | - | - | - | - | - | - | - |
| random active/ramp | t1482 r0 d819 20m 4/5 | - | - | t1152 r0 d1152 19m | - | - | - | - | - | - | - | - | - | - |
| random active/200x | - | - | - | t1170 r0 d1170 0m | - | - | - | - | - | - | - | - | - | - |
| random casual/1x | t4920 r4 d806 82m 4/5 | - | - | t1143 r0 d1143 19m | - | - | - | - | - | - | - | - | - | - |
| random casual/mixed | t4920 r4 d806 82m 4/5 | - | - | t1143 r0 d1143 19m | - | - | - | - | - | - | - | - | - | - |
| random casual/10x | - | - | - | t1170 r0 d1170 2m | - | - | - | - | - | - | - | - | - | - |
| random casual/ramp | - | - | - | t1143 r0 d1143 19m | - | - | - | - | - | - | - | - | - | - |
| random casual/200x | - | - | - | t1170 r0 d1170 0m | - | - | - | - | - | - | - | - | - | - |
| random afk/1x | - | - | - | t1170 r0 d1170 20m | - | - | - | - | - | - | - | - | - | - |
| random afk/mixed | - | - | - | t1170 r0 d1170 20m | - | - | - | - | - | - | - | - | - | - |
| random afk/10x | - | - | - | t1170 r0 d1170 2m | - | - | - | - | - | - | - | - | - | - |
| random afk/ramp | - | - | - | t1170 r0 d1170 20m | - | - | - | - | - | - | - | - | - | - |
| random afk/200x | - | - | - | t1170 r0 d1170 0m | - | - | - | - | - | - | - | - | - | - |
| greedy active/1x | t70 r0 d70 1m | t536 r0 d536 9m | - | t1113 r0 d1113 19m | - | - | - | - | - | - | - | - | - | - |
| greedy active/mixed | t165 r0 d165 1m | t666 r0 d666 3m | - | t1122 r0 d1122 7m | - | - | - | - | - | - | - | - | - | - |
| greedy active/10x | t150 r0 d150 0m | t701 r0 d701 1m | - | t1115 r0 d1115 2m | - | - | - | - | - | - | - | - | - | - |
| greedy active/ramp | t70 r0 d70 1m | t536 r0 d536 9m | - | t1113 r0 d1113 19m | - | - | - | - | - | - | - | - | - | - |
| greedy active/200x | - | - | - | t1070 r0 d1070 0m | - | - | - | - | - | - | - | - | - | - |
| greedy casual/1x | t240 r0 d240 4m | - | - | t1122 r0 d1122 19m | - | - | - | - | - | - | - | - | - | - |
| greedy casual/mixed | t240 r0 d240 4m | - | - | t1122 r0 d1122 19m | - | - | - | - | - | - | - | - | - | - |
| greedy casual/10x | - | - | - | t1064 r0 d1064 2m | - | - | - | - | - | - | - | - | - | - |
| greedy casual/ramp | t240 r0 d240 4m | - | - | t1122 r0 d1122 19m | - | - | - | - | - | - | - | - | - | - |
| greedy casual/200x | - | - | - | t1064 r0 d1064 0m | - | - | - | - | - | - | - | - | - | - |
| greedy afk/1x | - | - | - | t1064 r0 d1064 18m | - | - | - | - | - | - | - | - | - | - |
| greedy afk/mixed | - | - | - | t1064 r0 d1064 18m | - | - | - | - | - | - | - | - | - | - |
| greedy afk/10x | - | - | - | t1064 r0 d1064 2m | - | - | - | - | - | - | - | - | - | - |
| greedy afk/ramp | - | - | - | t1064 r0 d1064 18m | - | - | - | - | - | - | - | - | - | - |
| greedy afk/200x | - | - | - | t1064 r0 d1064 0m | - | - | - | - | - | - | - | - | - | - |
| author active/1x | t55 r0 d55 1m | t471 r0 d471 8m | t315 r0 d315 5m | t1024 r0 d1024 17m | - | - | t6455 r5 d1225 108m | - | - | - | - | - | - | - |
| author active/mixed | t65 r0 d65 0m | t466 r0 d466 3m | t2995 r2 d698 27m | t1106 r0 d1106 10m | - | - | t18040 r15 d1297 151m 1/5 | - | - | - | - | - | - | - |
| author active/10x | t100 r0 d100 0m | t501 r0 d501 1m | t750 r0 d750 1m | t1056 r0 d1056 2m | - | - | - | - | - | - | - | - | - | - |
| author active/ramp | t55 r0 d55 1m | t471 r0 d471 8m | t315 r0 d315 5m | t1024 r0 d1024 17m | - | - | - | - | - | - | - | - | - | - |
| author active/200x | t9000 r7 d1104 1m | - | - | t1066 r0 d1066 0m | - | - | - | - | - | - | - | - | - | - |
| author casual/1x | t240 r0 d240 4m | t682 r0 d682 11m | - | t1104 r0 d1104 18m | - | - | - | - | - | - | - | - | - | - |
| author casual/mixed | t240 r0 d240 4m | t682 r0 d682 11m | - | t1104 r0 d1104 18m | - | - | - | - | - | - | - | - | - | - |
| author casual/10x | - | - | - | t1064 r0 d1064 2m | - | - | - | - | - | - | - | - | - | - |
| author casual/ramp | t240 r0 d240 4m | t682 r0 d682 11m | - | t1104 r0 d1104 18m | - | - | - | - | - | - | - | - | - | - |
| author casual/200x | - | - | - | t1064 r0 d1064 0m | - | - | - | - | - | - | - | - | - | - |
| author afk/1x | - | - | - | t1064 r0 d1064 18m | - | - | - | - | - | - | - | - | - | - |
| author afk/mixed | - | - | - | t1064 r0 d1064 18m | - | - | - | - | - | - | - | - | - | - |
| author afk/10x | - | - | - | t1064 r0 d1064 2m | - | - | - | - | - | - | - | - | - | - |
| author afk/ramp | - | - | - | t1064 r0 d1064 18m | - | - | - | - | - | - | - | - | - | - |
| author afk/200x | - | - | - | t1064 r0 d1064 0m | - | - | - | - | - | - | - | - | - | - |

Targets (run windows): M1 r1 d0-350, M2 r1 d250-700, M3 r1-2, M4 r1, M5 r2-4, M6 r3-5, M7 r4-8, M8 r5-8, M9 r5-10, M10 r6-10, M11 r8-15, M12 r8-15, M13 r10-18, M14 r15-20

## Never built, never used (in every seed's career)

| Agent, profile | Buildings never built | Commands never used |
|---|---|---|
| random active/1x | ice_extractor, electrolysis, launch_pad, research_lab, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| random active/mixed | ice_extractor, electrolysis, launch_pad, research_lab, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| random active/10x | ice_extractor, electrolysis, research_lab, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | load_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| random active/ramp | ice_extractor, fabricator, electrolysis, launch_pad, research_lab, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| random active/200x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| random casual/1x | ice_extractor, electrolysis, research_lab, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| random casual/mixed | ice_extractor, electrolysis, research_lab, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| random casual/10x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| random casual/ramp | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| random casual/200x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_ice, buy_titanium, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| random afk/1x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| random afk/mixed | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| random afk/10x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| random afk/ramp | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| random afk/200x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_ice, buy_titanium, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy active/1x | ice_extractor, fabricator, electrolysis, research_lab, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_ice, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy active/mixed | ice_extractor, fabricator, electrolysis, research_lab, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_ice, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy active/10x | ice_extractor, fabricator, electrolysis, research_lab, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_ice, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy active/ramp | ice_extractor, fabricator, electrolysis, research_lab, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_ice, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy active/200x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy casual/1x | ice_extractor, fabricator, electrolysis, research_lab, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_ice, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy casual/mixed | ice_extractor, fabricator, electrolysis, research_lab, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_ice, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy casual/10x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy casual/ramp | ice_extractor, fabricator, electrolysis, research_lab, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_ice, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy casual/200x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_titanium, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy afk/1x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy afk/mixed | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy afk/10x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy afk/ramp | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| greedy afk/200x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_titanium, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author active/1x | battery, arbitrage_engine, fuel_cell, microwave_receiver | idle, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, fund_nationalist, fund_rationalist, buy_power |
| author active/mixed | battery, arbitrage_engine, fuel_cell, microwave_receiver | idle, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author active/10x | ice_extractor, electrolysis, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author active/ramp | electrolysis, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author active/200x | ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author casual/1x | ice_extractor, electrolysis, research_lab, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author casual/mixed | ice_extractor, electrolysis, research_lab, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author casual/10x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author casual/ramp | ice_extractor, electrolysis, research_lab, battery, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author casual/200x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_titanium, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author afk/1x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author afk/mixed | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author afk/10x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author afk/ramp | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |
| author afk/200x | excavator, ice_extractor, smelter, refinery, fabricator, electrolysis, launch_pad, research_lab, battery, storage_depot, arbitrage_engine, fuel_cell, microwave_receiver, recreation_dome | idle, buy_regolith, buy_ice, buy_titanium, buy_propellant, load_pads, launch_pads, dream, overclock_mining, overclock_factories, promote_he3, promote_titanium, promote_circuits, promote_propellant, disrupt_speculators, fund_nationalist, fund_humanist, fund_rationalist, buy_power |

## Run-1 objectives (reference/scenarios/run1_fresh.json; median day, against the Godot target window)

| Agent, profile | excavator [5, 20] | smelter [40, 100] | refinery [100, 200] | he3_10 [140, 220] | first_ship [260, 380] | data_center [380, 530] | research_lab [600, 800] | retirement [720, 1000] |
|---|---|---|---|---|---|---|---|---|
| random active/1x | 924 late 2/5 | 1076 late 1/5 | - | - | - | - | - | 524 early |
| random active/mixed | 924 late 2/5 | 1076 late 1/5 | - | - | - | - | - | 524 early |
| random active/10x | - | - | - | - | - | - | - | 517 early |
| random active/ramp | 924 late 2/5 | 1076 late 1/5 | - | - | - | - | - | 524 early |
| random active/200x | - | - | - | - | - | - | - | 540 early |
| random casual/1x | - | - | - | - | - | - | - | 530 early |
| random casual/mixed | - | - | - | - | - | - | - | 530 early |
| random casual/10x | - | - | - | - | - | - | - | 540 early |
| random casual/ramp | - | - | - | - | - | - | - | 530 early |
| random casual/200x | - | - | - | - | - | - | - | 540 early |
| random afk/1x | - | - | - | - | - | - | - | 540 early |
| random afk/mixed | - | - | - | - | - | - | - | 540 early |
| random afk/10x | - | - | - | - | - | - | - | 540 early |
| random afk/ramp | - | - | - | - | - | - | - | 540 early |
| random afk/200x | - | - | - | - | - | - | - | 540 early |
| greedy active/1x | 76 late | 96 in | 156 in | 165 in | 536 late | - | - | 414 early |
| greedy active/mixed | 171 late | 296 late | 406 late | 415 late | 666 late | - | - | 442 early |
| greedy active/10x | 251 late | 251 late | 401 late | 410 late | 701 late | - | - | 450 early |
| greedy active/ramp | 76 late | 96 in | 156 in | 165 in | 536 late | - | - | 414 early |
| greedy active/200x | - | - | - | - | - | - | - | 400 early |
| greedy casual/1x | 481 late | 481 late | 721 late | 730 late | - | - | - | 480 early |
| greedy casual/mixed | 481 late | 481 late | 721 late | 730 late | - | - | - | 480 early |
| greedy casual/10x | - | - | - | - | - | - | - | 400 early |
| greedy casual/ramp | 481 late | 481 late | 721 late | 730 late | - | - | - | 480 early |
| greedy casual/200x | - | - | - | - | - | - | - | 400 early |
| greedy afk/1x | - | - | - | - | - | - | - | 400 early |
| greedy afk/mixed | - | - | - | - | - | - | - | 400 early |
| greedy afk/10x | - | - | - | - | - | - | - | 400 early |
| greedy afk/ramp | - | - | - | - | - | - | - | 400 early |
| greedy afk/200x | - | - | - | - | - | - | - | 400 early |
| author active/1x | 151 late | 161 late | 331 late | 340 late | 471 late | 316 early | 576 early | 397 early |
| author active/mixed | 131 late | 141 late | 196 in | 205 in | 466 late | - | - | 426 early |
| author active/10x | 151 late | 151 late | 251 late | 260 late | 501 late | 751 late | 801 late | 423 early |
| author active/ramp | 151 late | 161 late | 331 late | 340 late | 471 late | 316 early | 576 early | 397 early |
| author active/200x | - | - | - | - | - | - | - | 400 early |
| author casual/1x | 361 late | 361 late | 481 late | 490 late | 682 late | - | - | 434 early |
| author casual/mixed | 361 late | 361 late | 481 late | 490 late | 682 late | - | - | 434 early |
| author casual/10x | - | - | - | - | - | - | - | 400 early |
| author casual/ramp | 361 late | 361 late | 481 late | 490 late | 682 late | - | - | 434 early |
| author casual/200x | - | - | - | - | - | - | - | 400 early |
| author afk/1x | - | - | - | - | - | - | - | 400 early |
| author afk/mixed | - | - | - | - | - | - | - | 400 early |
| author afk/10x | - | - | - | - | - | - | - | 400 early |
| author afk/ramp | - | - | - | - | - | - | - | 400 early |
| author afk/200x | - | - | - | - | - | - | - | 400 early |

engine speed, native: 3704 ticks/s scripted (golden game, a view a tick), 59074 ticks/s plain advance
engine speed, wasm: 3936 ticks/s scripted (golden game, a view a tick), 50718 ticks/s plain advance
