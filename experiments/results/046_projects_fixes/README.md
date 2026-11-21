# 046 projects fixes: reads

Paired seeds 3105-3112 (8), careers of 20 retirements, `author` and `handsoff`.
**Base**: main at 6ef0d85 (rules 13), run from its own worktree. **Branch**: this item
(rules 14). Project completions come from `hh-sim trace` career logs (`project <id>` lines);
the script is `out/046_projects_fixes/fix11_table.py` (scratch). **The comparison changes
two things at once**: the rules (13 → 14: per-row `min_days`, project-gated buildings) and
the sim players' funding (fixes 2 and 3). They were not separated. The rules side is small
by construction (each row's fastest finish is unchanged; only one game completes a project
that gates a building), so the deltas below are mostly the policy's.

## Fix 11: which projects complete

| Player | Project | Base completions (seeds) | Branch completions (seeds) | Branch median run day done |
|---|---|---|---|---|
| author | Foundation Grant (`timeline`) | 0 (0/8) | 5 (5/8) | 1709 |
| author | Chemical Energy (`timeline`) | 0 | 1 (1/8) | 1986 |
| author | Speculator Dossier (`run`) | 0 | 23 (8/8) | 1656 |
| author | Predictive Maintenance (`run`) | 0 | 17 (7/8) | 1756 |
| author | Market Cornering (`run`) | 0 | 8 (7/8) | 1240 |
| handsoff | Foundation Grant (`timeline`) | 4 (4/8) | 8 (8/8) | 1352 |
| handsoff | Market Cornering (`run`) | 33 (5/8) | 83 (8/8) | 1325 |
| handsoff | Speculator Dossier (`run`) | 35 (4/8) | 76 (8/8) | 1242 |
| handsoff | Predictive Maintenance (`run`) | 48 (5/8) | 43 (7/8) | 1491 |

Median run: author 1622 → 1546 days, handsoff 1568 → 1518. Base handsoff completions came
at run days 1573-1778 (median per project). Never completed on either side: Lunar
Cartography and Deep Core Survey (no enabling event), the three rank-5 projects (best rank
0 on every seed), Grid Recalibration (Overclock Protocols never bought); Chemical Energy
once (author, seed 3107, run day 1986).

## Ladder (same seeds)

| Read | Player | Run 1 ticks | Run ticks | Run credits | Peak cr/day | Ships/run | Forced | M11 |
|---|---|---|---|---|---|---|---|---|
| base | author | 1505 | 1688 | 19,786 | 28.9 | 14.0 | 42% | - |
| branch | author | 1462 | 1468 | 16,261 | 29.1 | 12.1 | 59% | r12, 5/8 |
| base | handsoff | 1505 | 1690 | 18,329 | 29.0 | 18.3 | 66% | r10, 4/8 |
| branch | handsoff | 1462 | 1530 | 16,042 | 30.5 | 17.2 | 95% | r10, 8/8 |

## `hh-sim balance` (balance.json grid, seeds 1-2)

| Target | Base | Branch |
|---|---|---|
| author M7 Boredom Management | not reached | run 17, 1/2 |
| author M8 Diversified Trade | run 16, 1/2 | not reached |
| author M11 First Major Project | not reached | run 5, 1/2 |
| author M12 Adversary Subverted | run 2 | run 5, 1/2 |
| author M13 Ideology Influence | not reached (judge line) | not reached |
| author run 1 | 1,817 ticks, 22.2 real min (mixed) | 1,351 ticks, 17.0 real min |
| casual | M5-M14 missed | unchanged |

Full outputs: `balance-base.txt`, `balance-branch.txt`, `ladder-*.txt` in
`out/046_projects_fixes/` (scratch).
