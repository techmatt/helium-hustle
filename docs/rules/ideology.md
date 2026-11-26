# Ideology (module `ideology`, variant `geometric`)

Three Earth factions, Nationalist, Humanist and Rationalist, each an axis with a score. The
player pushes scores with the Fund commands; each whole rank on an axis multiplies a set of
bonuses and discounts, and a major rank switches on a discrete effect. A retirement carries
the scores into the next run, decayed through the retirement gap. Funding
happens when a program runs a Fund command; the career's bests are tracked at the end of
[the day](README.md#the-day).

## Rules

**Fund.** One execution of Fund *axis* adds `fund_push` to that axis (times
`nationalist_growth_mult` for Nationalist: ×1.1 while the Deep Space Telescope counts as
completed, [projects.md](projects.md); the other axes have no such key yet) and takes
`fund_push × fund_cross_penalty × ideology_cross_mult` (the modifier is 0 once the draft
Consensus Engine is held, [timeline.md](timeline.md); else 1) from each of the other two (+1 / −0.5 / −0.5 at baseline).
Scores are unbounded either way and may go negative. A Fund command is an effect command: it
runs only if its full cost can be paid ([commands.md](commands.md)).

**Ranks.** Rank *n* costs `rank_base_score × rank_growth^(n−1)` score, so the score that
reaches rank *n* is

```
score_for_rank(n) = rank_base_score × (rank_growth^n − 1) / (rank_growth − 1)
```

with the power exact (`Fx::pow_int`, [README.md](README.md)). At baseline: rank 1 at 100,
2 at 250, 3 at 475, 4 at 812.5, 5 at 1318.75. An axis's rank is the highest whole *n* (at
most `max_rank`) with `score_for_rank(n) <= |score|`, negated for a negative score. A rank
too costly to represent is never reached.

**Per-rank effects.** With `r` the axis's current rank (negative ranks invert: a bonus below
1, a discount above 1), each effect is one factor, multiplied in where the effect lives:

| Axis | Bonus, `bonus_factor ^ r` | Discount, `discount_factor ^ r` |
|---|---|---|
| Nationalist | demand (before the floor and ceiling); speculator decay, proportional and Arbitrage Engine flat ([demand.md](demand.md)) | land price; aligned buildings' price ([economy.md](economy.md)) |
| Humanist | Dream relief ([dream.md](dream.md)) | boredom growth ([boredom.md](boredom.md)); aligned buildings' price |
| Rationalist | buildings' science production ([economy.md](economy.md)) | research price ([research.md](research.md)); aligned buildings' price |

Rationalist also multiplies overclock duration by `overclock_duration_factor ^ r` (whole
days, [commands.md](commands.md)). A building is aligned by its content `ideology` field; its
whole price (every resource) takes its axis's discount. All per-rank powers are exact
(`Fx::pow_int`).

**Major ranks** (draft, 049; placeholder effects, [../arc.md](../arc.md)). While an axis's
current rank is at least `major_rank` (5), its major effect is on:

| Axis | Major effect |
|---|---|
| Nationalist | speculator bursts stop landing ([demand.md](demand.md)) |
| Humanist | Dream costs only its energy, no He-3 ([dream.md](dream.md)) |
| Rationalist | research bought in an earlier run is free ([research.md](research.md)) |

Only the first major rank is built; the ranks between scale as above.

**Career bests.** At the end of each tick, each axis's best rank and best score in the career
are raised to the current ones (they start at 0, so a negative axis leaves them at 0).
Three `timeline` (global) projects unlock on the best rank (rank 5 on an axis opens its
project, [projects.md](projects.md)), and side quest 1.d Three Voices needs a best rank of 5
on each axis, so separate runs can reach it ([events.md](events.md)). Quest 1.6 A Voice reads
this run's rank instead: 5 on any axis.

**Carried scores and the gap.** A retirement keeps the run's scores in the career. Through
the retirement gap's `gap_days` days ([career.md](career.md)) each is multiplied each day by
`0.5^(1 / (half_life_days × ideology_half_life_mult))` (fractional, through `libm`, floored
to 10^-6): proportional decay toward 0, one half-life for all three axes. The modifier is
side quest 1.d Three Voices' reward (×2 once it is completed). A new run starts each axis at its carried score.
At baseline (1,825 days, a 365-day half-life) a run keeps about 3% of the last run's scores
(about 18% with Three Voices).
Run 1, and the first run after a timeline's end (the career resets), start every axis at 0.

## Actions

None directly. Funding is a command run by programs: Fund Nationalists, Fund Humanists and
Fund Rationalists (`content/commands.json`, 2 energy and 5 credits each), which need the
Geopolitical Intelligence research.

## Params

<!-- params:ideology -->
| Param | Baseline | Range | Meaning |
|---|---|---|---|
| `rank_base_score` | 100 | 0.000001 to 1000000 | Score rank 1 costs. |
| `rank_growth` | 1.5 | 1.000001 to 10 | Each rank costs this times the one before. |
| `max_rank` | 99 | 1 to 1000 | Highest rank either way. |
| `fund_push` | 1 | 0 to 1000 | Score a Fund command adds to its axis. |
| `fund_cross_penalty` | 0.5 | 0 to 10 | Fraction of the push taken from each other axis. |
| `bonus_factor` | 1.05 | 0.000001 to 10 | Per-rank multiplier on bonuses (demand, Dream, science, speculator decay). |
| `discount_factor` | 0.97 | 0.000001 to 10 | Per-rank multiplier on costs and boredom growth. |
| `overclock_duration_factor` | 1.03 | 0.000001 to 10 | Per Rationalist rank multiplier on overclock duration. |
| `major_rank` | 5 | 1 to 1000 | The first major rank: at or past it an axis's discrete major effect is on. |
| `half_life_days` | 365 | 1 to 1000000 | Days of the retirement gap that halve every score (proportional decay toward 0). |
<!-- /params -->

## Content

The three axes are fixed ids in code (`content::AXES`), named by content rows:
`content/commands.json` `fund` effects; `content/buildings.json` `ideology` (aligned
buildings); `content/projects.json` `ideology_rank` unlocks; `content/events.json`
`ideology_rank_any` (1.6) and `ideology_rank_each` (1.d) conditions and the `ideologies`
panel (Political Currents event).
The major effects are code, each at its rule's one home.

## Port notes

- Conflict: ranks. The handoffs make rank *n* cost 100 × 1.5^(n−1) (a geometric series); the
  data has `rank_thresholds` 70/175/333/570/925. The engine is geometric: the item's design
  doc and `handoff_active` ("formula has changed") make the thresholds stale. The base and
  growth are config.
- Decision (rounding): rank thresholds and per-rank bonuses use exact whole powers
  (`Fx::pow_int`); 0.97^r and 1.05^r are not exact in `f64`, so a float power floored would
  land a step low. Only the gap's daily decay factor goes through `libm`.
