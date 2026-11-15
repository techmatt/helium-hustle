# Ideology (module `ideology`, variant `geometric`)

Three Earth factions, Nationalist, Humanist and Rationalist, each an axis with a score. The
player pushes scores with the Fund commands; each whole rank on an axis multiplies a set of
bonuses and discounts. Scores reset each run to a head start from the career's best. Funding
happens when a program runs a Fund command; the career's bests are tracked at the end of
[the day](README.md#the-day).

## Rules

**Fund.** One execution of Fund *axis* adds `fund_push` to that axis and takes
`fund_push × fund_cross_penalty` from each of the other two (+1 / −0.5 / −0.5 at baseline).
Scores are unbounded either way and may go negative. A Fund command is an effect command: it
runs only if its full cost can be paid ([programs.md](programs.md)).

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
days, [programs.md](programs.md)). A building is aligned by its content `ideology` field; its
whole price (every resource) takes its axis's discount. All per-rank powers are exact
(`Fx::pow_int`).

**Career bests.** At the end of each tick, each axis's best rank and best score in the career
are raised to the current ones (they start at 0, so a negative axis leaves them at 0).
Persistent projects unlock on the best rank (rank 5 on an axis opens its project,
[projects.md](projects.md)); the Open Horizons sub-objective reads the current rank of any
axis ([events.md](events.md)).

**Head start.** A new run sets each axis's score to

```
score_for_continuous_rank(continuous_rank(best_score) × ideology_headstart_fraction)
```

where `continuous_rank(s) = log_growth(s × (rank_growth − 1) / rank_base_score + 1)` (0 for
a score at or below 0) and `score_for_continuous_rank` is its inverse,
`rank_base_score × (rank_growth^rank − 1) / (rank_growth − 1)`. Both are fractional, so they
go through `libm` and are floored to 10^-6. The fraction is a career param
([career.md](career.md)). At baseline a best score of 1318.75 (rank 5.0) starts the next run at
rank 1.0 (score about 100); run 1 starts every axis at 0.

## Actions

None directly. Funding is a command run by programs: Fund Nationalists, Fund Humanists and
Fund Rationalists (`content/commands.json`, 2 energy and 5 credits each), which need the
Geopolitical Intelligence research.

## Params

| Param | Baseline | Meaning |
|---|---|---|
| `rank_base_score` | 100 | Score rank 1 costs. |
| `rank_growth` | 1.5 | Each rank costs this times the one before. |
| `max_rank` | 99 | Highest rank either way. |
| `fund_push` | 1 | Score a Fund command adds to its axis. |
| `fund_cross_penalty` | 0.5 | Fraction of the push taken from each other axis. |
| `bonus_factor` | 1.05 | Per-rank multiplier on bonuses (demand, speculator decay, Dream, science). |
| `discount_factor` | 0.97 | Per-rank multiplier on costs and boredom growth. |
| `overclock_duration_factor` | 1.03 | Per Rationalist rank multiplier on overclock duration. |

## Content

The three axes are fixed ids in code (`content::AXES`), named by content rows:
`content/commands.json` `fund` effects; `content/buildings.json` `ideology` (aligned
buildings); `content/projects.json` `ideology_rank` unlocks; `content/events.json`
`ideology_rank_any` conditions and the `ideologies` panel (Political Currents event).

## Port notes

- Conflict: ranks. The handoffs make rank *n* cost 100 × 1.5^(n−1) (a geometric series); the
  data has `rank_thresholds` 70/175/333/570/925. The engine is geometric: the item's design
  doc and `handoff_active` ("formula has changed") make the thresholds stale. The base and
  growth are config.
- Decision (rounding): rank thresholds and per-rank bonuses use exact whole powers
  (`Fx::pow_int`); 0.97^r and 1.05^r are not exact in `f64`, so a float power floored would
  land a step low. Only the head start's continuous rank goes through `libm`.
