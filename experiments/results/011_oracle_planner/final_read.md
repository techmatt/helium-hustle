# 011 final read (fresh seeds 3013-3015, careers of 20 retirements)

Generated from the read's `games.jsonl` and `report.md` (kept in `out/keep/`).

## Pace in game time (3 seeds a row; medians for run 1, means over finished runs)

| Player | Ret | Run 1 ticks | Run ticks | Run 1 dec | Dec/run | Ticks/dec | Run credits | Peak cr/day | Ships/run | Forced | Cmd boredom | Idle | Piles at cap | Overflow lost | Ticks/s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| random | 20.0 | 1639 | 1664 | 328 | 333 | 5.0 | 1591 | 4.4 | 0.0 | 100% | 3% | 98% | energy 84%, titanium 74%, regolith 70% | he3 93%, titanium 90%, circuits 85% | 4822 |
| greedy | 20.0 | 1611 | 1636 | 122 | 120 | 13.6 | 7286 | 10.6 | 18.2 | 100% | 5% | 85% | energy 100%, titanium 78%, he3 34% | titanium 85%, he3 37%, energy 10% | 10529 |
| author | 20.0 | 1111 | 1551 | 115 | 154 | 10.1 | 14144 | 25.4 | 14.2 | 27% | 8% | 15% | regolith 46%, energy 28%, circuits 20% | propellant 56%, circuits 23%, he3 22% | 6842 |
| casual | 20.0 | 1560 | 1573 | 11 | 12 | 128.9 | 8739 | 15.5 | 16.8 | 80% | 9% | 11% | energy 93%, regolith 67%, titanium 57% | circuits 68%, titanium 68%, regolith 26% | 20757 |
| oracle | 20.0 | 1245 | 1656 | 130 | 162 | 10.2 | 28093 | 67.8 | 17.8 | 45% | 3% | 12% | regolith 67%, energy 30%, titanium 25% | regolith 54%, propellant 37%, titanium 27% | 36 |
| oracle-credits | 20.0 | 1853 | 1679 | 189 | 166 | 10.1 | 28864 | 62.3 | 19.1 | 38% | 7% | 14% | regolith 61%, energy 30%, titanium 25% | regolith 48%, propellant 36%, titanium 27% | 37 |
| oracle-ideology | 20.0 | 1756 | 1698 | 186 | 168 | 10.1 | 29350 | 71.7 | 20.5 | 47% | 6% | 14% | regolith 60%, energy 31%, titanium 30% | regolith 49%, titanium 34%, propellant 29% | 37 |
| oracle-builder | 20.0 | 1955 | 1727 | 199 | 176 | 9.8 | 30587 | 62.2 | 22.6 | 33% | -3% | 15% | regolith 66%, energy 30%, titanium 28% | regolith 55%, propellant 34%, titanium 29% | 35 |
| oracle-quests | 20.0 | 2253 | 1737 | 193 | 164 | 10.6 | 27457 | 68.9 | 18.0 | 45% | -5% | 15% | regolith 65%, energy 33%, titanium 31% | regolith 53%, titanium 35%, propellant 30% | 50 |

## Milestones (medians: `t` career ticks, `r` retirements before it (run r+1), `d` that run's day; `k/n` when only k of n seeds reached it)

| Player | M1 | M2 | M3 | M4 | M5 | M6 | M7 | M8 | M9 | M10 | M11 | M12 | M13 | M14 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| random | t975 r0 d975 | - | - | t1639 r0 d1639 | - | - | - | - | - | - | - | - | - | - |
| greedy | t72 r0 d72 | t515 r0 d515 | - | t1611 r0 d1611 | - | - | - | - | - | - | - | - | - | - |
| author | t60 r0 d60 | t420 r0 d420 | t247 r0 d247 | t1111 r0 d1111 | t28168 r18 d1082 2/3 | - | t2325 r1 d1214 | t17862 r10 d1607 2/3 | - | - | - | - | t20372 r12 d1929 1/3 | - |
| casual | t240 r0 d240 | t682 r0 d682 | t2880 r1 d1320 | t1560 r0 d1560 | - | - | - | - | - | - | - | - | - | - |
| oracle | t60 r0 d60 | t420 r0 d420 | t247 r0 d247 | t1245 r0 d1245 | t1114 r0 d1076 | - | t2318 r1 d978 | t15914 r9 d1557 | - | t840 r0 d840 | - | t643 r0 d643 | t8937 r5 d1244 | - |
| oracle-credits | t60 r0 d60 | t527 r0 d527 | t247 r0 d247 | t1853 r0 d1853 | t1507 r0 d1078 | - | t4244 r2 d1527 | t14004 r7 d1582 2/3 | t23477 r13 d1898 1/3 | t1240 r0 d1240 | - | t2634 r1 d1135 1/3 | - | - |
| oracle-ideology | t60 r0 d60 | t643 r0 d643 | t247 r0 d247 | t1756 r0 d1756 | t1335 r0 d1335 | - | t10810 r5 d1807 | t12891 r6 d1776 2/3 | - | t1136 r0 d1136 | - | t1226 r0 d1226 1/3 | t17956 r10 d1773 2/3 | - |
| oracle-builder | t60 r0 d60 | t456 r0 d456 | t247 r0 d247 | t1955 r0 d1955 | t1019 r0 d1019 | - | t1682 r0 d1575 | t7279 r3 d1597 1/3 | t9864 r4 d1922 2/3 | t1186 r0 d1186 | - | t2212 r0 d1202 | t25769 r14 d1964 1/3 | - |
| oracle-quests | t60 r0 d60 | t420 r0 d420 | t247 r0 d247 | t2253 r0 d2253 | t1128 r0 d1128 | - | t1254 r0 d1079 | t24417 r13 d1168 2/3 | t21198 r10 d2328 1/3 | t840 r0 d840 | t10326 r4 d986 2/3 | t643 r0 d643 | t4138 r1 d1553 | - |

Targets (run windows): M1 r1 d0-350, M2 r1 d250-700, M3 r1-2, M4 r1, M5 r2-4, M6 r3-5, M7 r4-8, M8 r5-8, M9 r5-10, M10 r6-10, M11 r8-15, M12 r8-15, M13 r10-18, M14 r15-20

## Per player (3 seeds, careers of 20 retirements)

| Player | Runs | Credits a run | Best 100-day cr/day (median, max) | Retired by choice | Best ranks (max over seeds) | Long-Term projects | Open Horizons sub-objectives | Achievements | Wall min |
|---|---|---|---|---|---|---|---|---|---|
| random | 60 | 1,591 | 4.4, 5.0 | 0% | hum 0, nat 0, rat 0 | 0/0/0: - | 0/0/0: - | 0/0/0 | 0 |
| greedy | 60 | 7,286 | 10.6, 11.0 | 0% | hum 0, nat 0, rat 0 | 0/0/0: - | 0/0/0: - | 1/1/1 | 0 |
| author | 60 | 14,144 | 25.4, 26.9 | 73% | hum 3, nat 0, rat 0 | 0/0/0: - | 0/0/0: - | 2/1/1 | 0 |
| author:min_gap=120 | 60 | 8,739 | 15.5, 16.3 | 20% | hum 0, nat 0, rat 0 | 0/0/0: - | 0/0/0: - | 1/1/1 | 0 |
| oracle | 60 | 28,093 | 67.8, 74.8 | 55% | hum 4, nat 4, rat 3 | 0/0/0: - | 0/0/0: - | 5/4/5 | 15 |
| oracle:profile=1 | 60 | 28,864 | 62.3, 64.8 | 62% | hum 2, nat 0, rat 0 | 0/0/0: - | 0/0/0: - | 4/4/4 | 15 |
| oracle:profile=2 | 60 | 29,350 | 71.7, 72.4 | 53% | hum 3, nat 0, rat 3 | 0/0/0: - | 0/0/0: - | 4/5/4 | 15 |
| oracle:profile=3 | 60 | 30,587 | 62.2, 71.3 | 67% | hum 2, nat 0, rat 3 | 0/0/0: - | 0/0/0: - | 5/5/5 | 17 |
| oracle:profile=4 | 60 | 27,457 | 68.9, 75.6 | 55% | hum 5, nat 0, rat 0 | 1/1/0: ai_consciousness, foundation_grant | 3/2/1: ideology_rank_5, persistent_project, survive_10_years | 4/5/4 | 12 |

## Milestones by player: median career tick (run), k/n seeds

| Player | M1 | M2 | M3 | M4 | M5 | M6 | M7 | M8 | M9 | M10 | M11 | M12 | M13 | M14 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| random | t975 r1 | - | - | t1639 r1 | - | - | - | - | - | - | - | - | - | - |
| greedy | t72 r1 | t515 r1 | - | t1611 r1 | - | - | - | - | - | - | - | - | - | - |
| author | t60 r1 | t420 r1 | t247 r1 | t1111 r1 | t28168 r18.5 2/3 | - | t2325 r2 | t17862 r11.5 2/3 | - | - | - | - | t20372 r13 1/3 | - |
| author:min_gap=120 | t240 r1 | t682 r1 | t2880 r2 | t1560 r1 | - | - | - | - | - | - | - | - | - | - |
| oracle | t60 r1 | t420 r1 | t247 r1 | t1245 r1 | t1114 r1 | - | t2318 r2 | t15914 r10 | - | t840 r1 | - | t643 r1 | t8937 r6 | - |
| oracle:profile=1 | t60 r1 | t527 r1 | t247 r1 | t1853 r1 | t1507 r1 | - | t4244 r3 | t14004 r8 2/3 | t23477 r14 1/3 | t1240 r1 | - | t2634 r2 1/3 | - | - |
| oracle:profile=2 | t60 r1 | t643 r1 | t247 r1 | t1756 r1 | t1335 r1 | - | t10810 r6 | t12891 r7.5 2/3 | - | t1136 r1 | - | t1226 r1 1/3 | t17956 r10.5 2/3 | - |
| oracle:profile=3 | t60 r1 | t456 r1 | t247 r1 | t1955 r1 | t1019 r1 | - | t1682 r1 | t7279 r4 1/3 | t9864 r5.5 2/3 | t1186 r1 | - | t2212 r1 | t25769 r15 1/3 | - |
| oracle:profile=4 | t60 r1 | t420 r1 | t247 r1 | t2253 r1 | t1128 r1 | - | t1254 r1 | t24417 r14 2/3 | t21198 r11 1/3 | t840 r1 | t10326 r5.5 2/3 | t643 r1 | t4138 r2 | - |

## Each seed: milestone ticks (run)

- random seed 3013: M1:t995r1 M4:t1640r1
- random seed 3014: M1:t640r1 M4:t1639r1
- random seed 3015: M1:t975r1 M4:t1638r1
- greedy seed 3013: M1:t72r1 M2:t515r1 M4:t1622r1
- greedy seed 3014: M1:t72r1 M2:t515r1 M4:t1611r1
- greedy seed 3015: M1:t72r1 M2:t515r1 M4:t1605r1
- author seed 3013: M1:t60r1 M2:t420r1 M3:t247r1 M4:t1640r1 M5:t29425r19 M7:t1132r1 M8:t26663r17 M13:t20372r13
- author seed 3014: M1:t60r1 M2:t420r1 M3:t247r1 M4:t1111r1 M5:t26912r18 M7:t2325r2
- author seed 3015: M1:t60r1 M2:t420r1 M3:t247r1 M4:t1038r1 M7:t3644r3 M8:t9062r6
- author:min_gap=120 seed 3013: M1:t240r1 M2:t682r1 M3:t1200r1 M4:t1560r1
- author:min_gap=120 seed 3014: M1:t240r1 M2:t682r1 M3:t3000r2 M4:t1440r1
- author:min_gap=120 seed 3015: M1:t240r1 M2:t682r1 M3:t2880r2 M4:t1560r1
- oracle seed 3013: M1:t60r1 M2:t420r1 M3:t247r1 M4:t1340r1 M5:t1076r1 M7:t2318r2 M8:t18716r10 M10:t965r1 M12:t994r1 M13:t14129r8
- oracle seed 3014: M1:t60r1 M2:t420r1 M3:t247r1 M4:t1054r1 M5:t2000r2 M7:t2356r2 M8:t3772r3 M10:t799r1 M12:t636r1 M13:t8462r6
- oracle seed 3015: M1:t60r1 M2:t420r1 M3:t247r1 M4:t1245r1 M5:t1114r1 M7:t2206r2 M8:t15914r11 M10:t840r1 M12:t643r1 M13:t8937r6
- oracle:profile=1 seed 3013: M1:t60r1 M2:t527r1 M3:t247r1 M4:t2069r1 M5:t1507r1 M7:t1689r1 M8:t24933r14 M10:t1018r1
- oracle:profile=1 seed 3014: M1:t60r1 M2:t456r1 M3:t247r1 M4:t1853r1 M5:t2931r2 M7:t4244r3 M9:t23477r14 M10:t9570r6
- oracle:profile=1 seed 3015: M1:t60r1 M2:t650r1 M3:t247r1 M4:t1499r1 M5:t736r1 M7:t12900r8 M8:t3075r2 M10:t1240r1 M12:t2634r2
- oracle:profile=2 seed 3013: M1:t60r1 M2:t649r1 M3:t247r1 M4:t1962r1 M5:t1119r1 M7:t1807r1 M10:t1074r1 M13:t25246r15
- oracle:profile=2 seed 3014: M1:t60r1 M2:t643r1 M3:t247r1 M4:t1754r1 M5:t1335r1 M7:t10810r6 M8:t6622r4 M10:t1136r1 M13:t10665r6
- oracle:profile=2 seed 3015: M1:t60r1 M2:t421r1 M3:t247r1 M4:t1756r1 M5:t1625r1 M7:t25869r15 M8:t19160r11 M10:t1515r1 M12:t1226r1
- oracle:profile=3 seed 3013: M1:t60r1 M2:t456r1 M3:t247r1 M4:t1955r1 M5:t1256r1 M7:t1575r1 M10:t1186r1 M12:t8217r5
- oracle:profile=3 seed 3014: M1:t60r1 M2:t456r1 M3:t247r1 M4:t1740r1 M5:t967r1 M7:t2896r2 M9:t12110r7 M10:t3710r2 M12:t1116r1
- oracle:profile=3 seed 3015: M1:t60r1 M2:t501r1 M3:t247r1 M4:t2286r1 M5:t1019r1 M7:t1682r1 M8:t7279r4 M9:t7619r4 M10:t924r1 M12:t2212r1 M13:t25769r15
- oracle:profile=4 seed 3013: M1:t60r1 M2:t420r1 M3:t247r1 M4:t1610r1 M5:t1076r1 M7:t2675r2 M8:t13914r8 M10:t965r1 M11:t5611r3 M12:t994r1 M13:t3163r2
- oracle:profile=4 seed 3014: M1:t60r1 M2:t420r1 M3:t247r1 M4:t2463r1 M5:t2188r1 M7:t1079r1 M8:t34920r20 M9:t21198r11 M10:t799r1 M11:t15040r8 M12:t636r1 M13:t4138r2
- oracle:profile=4 seed 3015: M1:t60r1 M2:t420r1 M3:t247r1 M4:t2253r1 M5:t1128r1 M7:t1254r1 M10:t840r1 M12:t643r1 M13:t31323r16
