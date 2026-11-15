# 011 first read (seeds 3010-3012), superseded by the sustain fix

Generated from the read's `games.jsonl` and `report.md` (kept in `out/keep/`).

## Pace in game time (3 seeds a row; medians for run 1, means over finished runs)

| Player | Ret | Run 1 ticks | Run ticks | Run 1 dec | Dec/run | Ticks/dec | Run credits | Peak cr/day | Ships/run | Forced | Cmd boredom | Idle | Piles at cap | Overflow lost | Ticks/s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| random | 20.0 | 1645 | 1663 | 329 | 333 | 5.0 | 1632 | 4.2 | 0.0 | 100% | 3% | 98% | energy 85%, titanium 71%, regolith 60% | he3 93%, titanium 90%, circuits 78% | 3495 |
| greedy | 20.0 | 1612 | 1635 | 121 | 125 | 13.1 | 7140 | 10.7 | 17.2 | 100% | 5% | 82% | energy 100%, titanium 78%, he3 33% | titanium 85%, he3 37%, energy 11% | 7905 |
| author | 20.0 | 1159 | 1423 | 117 | 142 | 10.0 | 12067 | 24.3 | 11.3 | 13% | 16% | 15% | regolith 49%, energy 31%, circuits 21% | propellant 52%, he3 26%, circuits 23% | 5508 |
| casual | 20.0 | 1583 | 1584 | 13 | 12 | 129.1 | 8867 | 15.3 | 17.3 | 88% | 9% | 11% | energy 94%, regolith 69%, titanium 57% | circuits 69%, titanium 67%, regolith 26% | 15204 |
| oracle | 20.0 | 1640 | 1755 | 164 | 175 | 10.0 | 29835 | 66.9 | 20.0 | 38% | -5% | 13% | regolith 67%, energy 34%, titanium 26% | regolith 56%, titanium 27%, propellant 16% | 27 |
| oracle-credits | 20.0 | 1549 | 1642 | 157 | 161 | 10.2 | 26830 | 64.7 | 16.4 | 38% | 4% | 13% | regolith 60%, energy 30%, titanium 26% | regolith 49%, propellant 35%, titanium 29% | 38 |
| oracle-ideology | 20.0 | 1335 | 1750 | 131 | 170 | 10.3 | 27708 | 62.3 | 18.4 | 55% | 4% | 14% | regolith 57%, energy 39%, titanium 28% | regolith 47%, titanium 33%, propellant 32% | 39 |
| oracle-builder | 20.0 | 2004 | 1751 | 195 | 171 | 10.3 | 28006 | 65.9 | 18.0 | 50% | 3% | 12% | regolith 59%, energy 36%, titanium 23% | regolith 49%, propellant 31%, titanium 27% | 37 |
| oracle-quests | 20.0 | 2346 | 1885 | 205 | 165 | 11.4 | 17782 | 49.8 | 10.1 | 87% | 8% | 17% | energy 92%, regolith 51%, titanium 12% | energy 42%, regolith 38%, ice 29% | 88 |

## Milestones (medians: `t` career ticks, `r` retirements before it (run r+1), `d` that run's day; `k/n` when only k of n seeds reached it)

| Player | M1 | M2 | M3 | M4 | M5 | M6 | M7 | M8 | M9 | M10 | M11 | M12 | M13 | M14 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| random | t870 r0 d870 | - | - | t1645 r0 d1645 | - | - | - | - | - | - | - | - | - | - |
| greedy | t72 r0 d72 | t515 r0 d515 | - | t1612 r0 d1612 | - | - | - | - | - | - | - | - | - | - |
| author | t60 r0 d60 | t420 r0 d420 | t247 r0 d247 | t1159 r0 d1159 | t26730 r18 d1772 1/3 | - | t5018 r3 d1138 | t23112 r16 d1395 1/3 | - | - | - | - | t27060 r18 d2102 1/3 | - |
| casual | t240 r0 d240 | t682 r0 d682 | t1560 r0 d1440 | t1583 r0 d1583 | - | - | - | - | - | - | - | - | - | - |
| oracle | t60 r0 d60 | t420 r0 d420 | t247 r0 d247 | t1640 r0 d1640 | t1043 r0 d1043 | - | t1225 r0 d1225 | t6316 r3 d1817 | - | t830 r0 d830 | t22538 r12 d1685 1/3 | t941 r0 d941 | t33187 r18 d1174 1/3 | - |
| oracle-credits | t60 r0 d60 | t502 r0 d502 | t247 r0 d247 | t1549 r0 d1549 | t977 r0 d977 | - | t3749 r2 d1486 | t9542 r6 d1309 2/3 | - | t1177 r0 d1175 | - | t13163 r7 d1272 1/3 | t23152 r13 d1684 1/3 | - |
| oracle-ideology | t60 r0 d60 | t498 r0 d498 | t247 r0 d247 | t1335 r0 d1335 | t2626 r1 d1101 | - | t6807 r3 d1468 | t23130 r12 d1648 2/3 | t28167 r15 d1749 1/3 | t2588 r1 d1206 | - | - | t3999 r2 d1284 | - |
| oracle-builder | t60 r0 d60 | t498 r0 d498 | t247 r0 d247 | t2004 r0 d2004 | t1242 r0 d1242 | - | t3249 r1 d1693 | t26398 r14 d1538 2/3 | - | t1068 r0 d1068 | - | t5343 r2 d734 | - | - |
| oracle-quests | t60 r0 d60 | t420 r0 d420 | t247 r0 d247 | t2346 r0 d2346 | t1115 r0 d1115 | - | t1204 r0 d1204 | - | - | t830 r0 d830 | t14954 r6 d3372 1/3 | t941 r0 d941 | t14647 r6 d3065 1/3 | - |

Targets (run windows): M1 r1 d0-350, M2 r1 d250-700, M3 r1-2, M4 r1, M5 r2-4, M6 r3-5, M7 r4-8, M8 r5-8, M9 r5-10, M10 r6-10, M11 r8-15, M12 r8-15, M13 r10-18, M14 r15-20

## Per player (3 seeds, careers of 20 retirements)

| Player | Runs | Credits a run | Best 100-day cr/day (median, max) | Retired by choice | Best ranks (max over seeds) | Long-Term projects | Open Horizons sub-objectives | Achievements | Wall min |
|---|---|---|---|---|---|---|---|---|---|
| random | 60 | 1,632 | 4.2, 5.0 | 0% | hum 0, nat 0, rat 0 | 0/0/0: - | 0/0/0: - | 0/0/0 | 0 |
| greedy | 60 | 7,140 | 10.7, 12.3 | 0% | hum 0, nat 0, rat 0 | 0/0/0: - | 0/0/0: - | 1/1/1 | 0 |
| author | 60 | 12,067 | 24.3, 25.7 | 87% | hum 3, nat 0, rat 0 | 0/0/0: - | 0/0/0: - | 1/1/1 | 0 |
| author:min_gap=120 | 60 | 8,867 | 15.3, 15.5 | 12% | hum 0, nat 0, rat 0 | 0/0/0: - | 0/0/0: - | 1/1/1 | 0 |
| oracle | 60 | 29,835 | 66.9, 80.5 | 62% | hum 4, nat 0, rat 0 | 0/0/1: chemical_energy | 0/0/1: persistent_project | 5/5/5 | 22 |
| oracle:profile=1 | 60 | 26,830 | 64.7, 73.9 | 62% | hum 4, nat 0, rat 0 | 0/0/0: - | 0/0/0: - | 5/5/4 | 15 |
| oracle:profile=2 | 60 | 27,708 | 62.3, 76.2 | 45% | hum 5, nat 3, rat 1 | 0/0/0: - | 0/1/0: ideology_rank_5 | 5/4/4 | 15 |
| oracle:profile=3 | 60 | 28,006 | 65.9, 70.7 | 50% | hum 2, nat 0, rat 1 | 0/0/0: - | 0/0/0: - | 5/4/5 | 16 |
| oracle:profile=4 | 60 | 17,782 | 49.8, 55.1 | 13% | hum 4, nat 0, rat 0 | 1/0/0: foundation_grant | 2/0/0: persistent_project, survive_10_years | 3/3/3 | 6 |

## Milestones by player: median career tick (run), k/n seeds

| Player | M1 | M2 | M3 | M4 | M5 | M6 | M7 | M8 | M9 | M10 | M11 | M12 | M13 | M14 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| random | t870 r1 | - | - | t1645 r1 | - | - | - | - | - | - | - | - | - | - |
| greedy | t72 r1 | t515 r1 | - | t1612 r1 | - | - | - | - | - | - | - | - | - | - |
| author | t60 r1 | t420 r1 | t247 r1 | t1159 r1 | t26730 r19 1/3 | - | t5018 r4 | t23112 r17 1/3 | - | - | - | - | t27060 r19 1/3 | - |
| author:min_gap=120 | t240 r1 | t682 r1 | t1560 r1 | t1583 r1 | - | - | - | - | - | - | - | - | - | - |
| oracle | t60 r1 | t420 r1 | t247 r1 | t1640 r1 | t1043 r1 | - | t1225 r1 | t6316 r4 | - | t830 r1 | t22538 r13 1/3 | t941 r1 | t33187 r19 1/3 | - |
| oracle:profile=1 | t60 r1 | t502 r1 | t247 r1 | t1549 r1 | t977 r1 | - | t3749 r3 | t9542 r6.5 2/3 | - | t1177 r1 | - | t13163 r8 1/3 | t23152 r14 1/3 | - |
| oracle:profile=2 | t60 r1 | t498 r1 | t247 r1 | t1335 r1 | t2626 r2 | - | t6807 r4 | t23130 r13.5 2/3 | t28167 r16 1/3 | t2588 r2 | - | - | t3999 r3 | - |
| oracle:profile=3 | t60 r1 | t498 r1 | t247 r1 | t2004 r1 | t1242 r1 | - | t3249 r2 | t26398 r15.5 2/3 | - | t1068 r1 | - | t5343 r3 | - | - |
| oracle:profile=4 | t60 r1 | t420 r1 | t247 r1 | t2346 r1 | t1115 r1 | - | t1204 r1 | - | - | t830 r1 | t14954 r7 1/3 | t941 r1 | t14647 r7 1/3 | - |

## Each seed: milestone ticks (run)

- random seed 3010: M1:t870r1 M4:t1644r1
- random seed 3011: M1:t845r1 M4:t1645r1
- random seed 3012: M1:t1575r1 M4:t1646r1
- greedy seed 3010: M1:t72r1 M2:t515r1 M4:t1612r1
- greedy seed 3011: M1:t72r1 M2:t515r1 M4:t1619r1
- greedy seed 3012: M1:t72r1 M2:t515r1 M4:t1611r1
- author seed 3010: M1:t60r1 M2:t420r1 M3:t247r1 M4:t1097r1 M7:t5746r5 M8:t23112r17
- author seed 3011: M1:t60r1 M2:t420r1 M3:t247r1 M4:t1159r1 M5:t26730r19 M7:t2362r2 M13:t27060r19
- author seed 3012: M1:t60r1 M2:t420r1 M3:t247r1 M4:t1197r1 M7:t5018r4
- author:min_gap=120 seed 3010: M1:t240r1 M2:t682r1 M3:t1440r1 M4:t1583r1
- author:min_gap=120 seed 3011: M1:t240r1 M2:t682r1 M3:t2880r2 M4:t1560r1
- author:min_gap=120 seed 3012: M1:t240r1 M2:t682r1 M3:t1560r1 M4:t1586r1
- oracle seed 3010: M1:t60r1 M2:t420r1 M3:t247r1 M4:t1255r1 M5:t1043r1 M7:t1225r1 M8:t6316r4 M10:t830r1 M12:t1046r1
- oracle seed 3011: M1:t60r1 M2:t420r1 M3:t247r1 M4:t2067r1 M5:t1183r1 M7:t1221r1 M8:t1817r1 M10:t830r1 M12:t629r1 M13:t33187r19
- oracle seed 3012: M1:t60r1 M2:t420r1 M3:t247r1 M4:t1640r1 M5:t987r1 M7:t1400r1 M8:t26544r15 M10:t902r1 M11:t22538r13 M12:t941r1
- oracle:profile=1 seed 3010: M1:t60r1 M2:t987r1 M3:t247r1 M4:t1375r1 M5:t1027r1 M7:t3392r2 M10:t1177r1 M12:t13163r8 M13:t23152r14
- oracle:profile=1 seed 3011: M1:t60r1 M2:t430r1 M3:t247r1 M4:t1820r1 M5:t872r1 M7:t7969r5 M8:t4536r3 M10:t1175r1
- oracle:profile=1 seed 3012: M1:t60r1 M2:t502r1 M3:t247r1 M4:t1549r1 M5:t977r1 M7:t3749r3 M8:t14549r10 M10:t28925r20
- oracle:profile=2 seed 3010: M1:t60r1 M2:t498r1 M3:t247r1 M4:t1525r1 M5:t2626r2 M7:t6807r4 M9:t28167r16 M10:t2588r2 M13:t1184r1
- oracle:profile=2 seed 3011: M1:t60r1 M2:t430r1 M3:t247r1 M4:t1251r1 M5:t8155r5 M7:t2426r2 M8:t20961r12 M10:t7991r5 M13:t3999r3
- oracle:profile=2 seed 3012: M1:t60r1 M2:t502r1 M3:t247r1 M4:t1335r1 M5:t985r1 M7:t9757r6 M8:t25300r15 M10:t2541r2 M13:t9966r6
- oracle:profile=3 seed 3010: M1:t60r1 M2:t498r1 M3:t247r1 M4:t2189r1 M5:t1418r1 M7:t1961r1 M8:t21158r12 M10:t1068r1 M12:t734r1
- oracle:profile=3 seed 3011: M1:t60r1 M2:t501r1 M3:t247r1 M4:t2004r1 M5:t1242r1 M7:t8278r5 M10:t1215r1 M12:t5343r3
- oracle:profile=3 seed 3012: M1:t60r1 M2:t420r1 M3:t247r1 M4:t1556r1 M5:t1008r1 M7:t3249r2 M8:t31638r19 M10:t862r1 M12:t9476r6
- oracle:profile=4 seed 3010: M1:t60r1 M2:t420r1 M3:t247r1 M4:t2346r1 M5:t1115r1 M7:t1204r1 M10:t830r1 M11:t14954r7 M12:t1174r1 M13:t14647r7
- oracle:profile=4 seed 3011: M1:t60r1 M2:t420r1 M3:t247r1 M4:t2359r1 M5:t1414r1 M7:t1204r1 M10:t830r1 M12:t629r1
- oracle:profile=4 seed 3012: M1:t60r1 M2:t420r1 M3:t247r1 M4:t2032r1 M5:t987r1 M7:t1368r1 M10:t902r1 M12:t941r1
