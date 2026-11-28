# 130: ideology parity, committed players

Rules revision 45. `hh-sim axes` plays one committed player per axis at each skill
([docs/simulator.md](../../../docs/simulator.md), *Ideology parity*; the players:
[docs/oracle.md](../../../docs/oracle.md), *Committed*). Final read: fresh seeds 3153-3155,
the shipped baseline (`final_3153-3155.txt`). Earlier reads, each before a fix or a number
changed: `dev_1-3.txt` (the freeze on dev seeds), `read_3147-3149.txt`, `read_3150-3152.txt`.

## The six cases (fresh seeds 3153 / 3154 / 3155)

Rank 5 is 725.3 score. "Run 1 best" is the best this-run axis score in run 1 as a share of it.

| Skill | Axis | Rank 5 first reached (run, day) | Run 1 best | Run 1 projects | Projects done by rank 5 |
|---|---|---|---|---|---|
| skilled | Nationalist | r2 d1202 / r2 d1360 / r2 d1175 | 6% / 68% / 0% | – / BA / – | BA / BA LC / BA |
| skilled | Humanist | r2 d1426 / r2 d1581 / r2 d1305 | 57% / 0% / 37% | BA / – / BA | BA LC (all three) |
| skilled | Rationalist | r3 d1423 / none in 3 runs / r2 d1362 | 48% / 12% / 55% | – | BA LC |
| medium | Nationalist | r2 d1347 / r1 d985 / r1 d1171 | (95% at its run-1 end on 3153) | BA FG LC (start) | BA FG LC |
| medium | Humanist | r1 d1027 / r1 d984 / r1 d1111 | | BA FG LC (start) | BA FG LC |
| medium | Rationalist | r1 d1311 / r1 d1155 / r1 d1320 | | BA FG LC (start) | BA FG LC |

BA Blueprint Archive, LC Lunar Cartography, FG Foundation Grant (run-scope projects not listed).

## Against the bars

| Bar | Target | Final read | Met |
|---|---|---|---|
| Skilled run 1 short of rank 5 | best axis < ~80% | 0-68%, 9/9 | yes |
| Skilled run 1 finishes BA and LC | both in run 1 | BA 3/9, LC 0/9 | **no** |
| Skilled run 2 reaches rank 5 | every axis | N 3/3, H 3/3, R 1/3 (R: run 3 once, none once) | **no** (Rationalist) |
| Parity | each axis's run-2 day within ±15% of the 3-axis mean | means N 1246, H 1437, R 1362 (one seed); 3-axis mean 1348: N −7.6%, H +6.6%, R +1.0% | yes on the seeds that reached; R rests on one seed |
| Medium reaches rank 5 within one run | run 1 | 8/9 (Nationalist 3153: 95% at its forced end, run 2 day 1347) | nearly |

**Drivers of the misses.**
- *LC in run 1:* Blueprint Archive and Lunar Cartography open only at A Voice's start, which
  the skilled oracle reaches on run-1 days 1,000-1,440 of runs that end at 1,460-1,560; BA
  (67 days at the credits cap) then LC (50 days of regolith) rarely both fit. LC is done in
  run 2 on 8/9. Opening them earlier is a rules call (Matt).
- *Rationalist run 2:* its runs last longest (2,000+ days, Theorize holds boredom) but fund
  slowly (65% after ~1,570 days of Fund on 3154): Theorize's processors, sized by the same
  rule as Dream's, run ahead of Fund in program order, so late in a long run little is left
  for Fund. An agent fix (cap the tool's processors, or Fund before the tool), not a number.
- Foresight: rollouts keep seeing the real future (reseeding the rollouts' streams would not
  hide the run's scheduled bursts; not cheap), so the skilled bars read with slack.

## Final numbers

| Number | Was | Now | Kind |
|---|---|---|---|
| `ideology.rank_base_score` | 100 | **55** | the global knob (rank 5 at 1,318.75 → 725.3) |
| Theorize relief (`commands.json` `boredom`) | −2 | **−3** | tool strength (Rationalist lagged) |
| Recreation Dome `nudge` | 5 | **2.5** | tool nudge (Nationalist led) |
| Dream, Theorize `nudge` | 0.25 | **0.5** | tool nudge |
| Dome cap 100 / relief 0.5, Dream relief 2 | – | unchanged | tool strength |
| `fund_push` 1, `fund_cross_penalty` 0.5 | – | unchanged | Fund identical on every axis |
| `half_life_days` 365 | – | unchanged | skilled run 1 funds late, so carry-over barely mattered |

Outside the free set, all by Matt's rulings (not tuning): Electrochemistry research (1,000
science, career flag, shown at 8 Solar Panels in any run; MESSAGE #3890) unlocks Chemical
Energy; Rationalist major effect removed; AI Consciousness Cloud Compute boredom 0.15 (its
`command_boredom` values replace the command's own: 0.1 × 1.5); Universal Research Archive
(already only earlier-run research: description only); amendment #3897's project costs and
Foundation Grant reward (all resources fundable as they are: no substitution needed).

## How the players commit

`Directives::commit` (author and oracle, knob `commit=1|2|3`): funds its axis on 2
processors; its tool only (Dream / Theorize held from a third of the cap; Domes bought from
half the cap); Blueprints then Geopolitical Intelligence first, no science to projects
while those wait; tagged buildings ×3/2 in the build score; while A Voice is under way no
voluntary retirement (boredom ends the run). The skilled oracle's campaign is per axis,
pushes BA then LC while open, and replans at A Voice's start and when its pushed project
finishes.
