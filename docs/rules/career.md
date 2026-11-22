# Career (module `career`, variant `arc1`)

A game is a career of runs. A run ends in retirement, forced when boredom reaches its cap
([boredom.md](boredom.md)) or chosen by the player; the next run starts at once with a fresh
operation, and a few career bonuses earned from personal bests. Forced retirement is the last
step of [the day](README.md#the-day), and so is a voluntary one a program's `retire` line
asked for that day (programs.md); the Retire action retires between days.

## Rules

**Persists and resets.** Game state is a `Career` (kept across retirements) and a `Run`
(replaced whole at each retirement):

- *Run (resets):* the run's day, every resource (boredom included), buildings, land bought,
  programs, launch pads, loading priority, markets and speculators, bursts and rival
  schedules, overclocks, research bought, every project's funding rates (every
  scope's), `run` project progress and completions, events fired and the buildings, panels and projects they enabled, the
  boredom phase, run stats. The run's ideology scores are replaced by the carried ones.
- *Career (persists):* run number, retirements, lifetime totals (credits earned, shipments,
  days, buildings built, research completed), personal bests (longest run in days, most
  credits earned in a run, most shipments in a run), each axis's best ideology rank and best
  score, the ideology scores carried into the next run, each adversary's Attention
  ([attention.md](attention.md)), the days spent in retirement gaps, peak power, career
  flags (the program instructions' unlocks among them), the program loadout library
  ([programs.md](programs.md)), ids ever researched, owned and used, events seen, quests
  (side quests too) and sub-objectives completed, `timeline` (global) and `forever` project
  progress and completions, achievements, the log, and the last finished run's summary
  (display: its number, length, whether forced, stats and final ranks, for the UI's run
  summary).

**Retirement.** `retire` logs `retired` or `forced_retirement`, records the run's summary,
folds the run into the
career (adds the run's stats and day count to the lifetime totals, raises each personal best
it beats, counts a retirement, raises the run number), keeps the run's ideology scores in
the career, runs the retirement gap (below), charges the calendar its `retire_days` once the
swarm is revealed (draft, [timeline.md](timeline.md)), then starts a new run. The run's day
count at retirement is its length (a forced retirement counts the tick it happened on).

**The retirement gap** (draft, 049). Between runs `gap_days` days pass, simulated one by
one, in which only the slow systems run: each carried ideology score is multiplied by
`0.5^(1 / half_life_days)` each day ([ideology.md](ideology.md); with the baseline's 1,825
days and 365-day half-life, about 3% of each score is left), and each adversary's Attention
loses its daily decay ([attention.md](attention.md)). The career counts the gap's days. The
gap is unseen in Act 1 (no calendar); it is not the calendar's `retire_days`.

**A new run** (a new game is run 1 of a fresh career), in this order:

1. A fresh run: every content resource at 0, the config's `starting_buildings`
   (economy) owned and active, the tradeable goods in content order as loading priority.
2. The config's `starting_resources` (economy: 100 energy, 30 land). Starting buildings take
   no land.
3. Ideology: each axis's score is the career's carried score, decayed through the gap (0 in
   run 1 and after a timeline's end, [ideology.md](ideology.md)).
4. Starting credits added: `floor(best_run_credits × credits_bonus_fraction)`, whole credits.
   They are not counted as credits earned.
5. Bonus buildings from completed projects (`starting_buildings` rewards) and earned
   achievements (`bonus_buildings` rewards): owned and active, outside the cost curve
   ([economy.md](economy.md)), taking no land.
6. Every building now held is recorded as owned in the career.
7. Each seen event's unlocks are reapplied (panels, buildings, projects; [events.md](events.md)).
8. Derived resources and pads refreshed (processors from Data Centers, a pad per Launch Pad).
9. Empty programs, every processor on Program 1 ([programs.md](programs.md)).
10. Fresh markets: new noise, empty speculator pools, the first burst and rival dumps
    scheduled ([demand.md](demand.md)).
11. Boredom phase 1.
12. The later acts' run start (draft, [timeline.md](timeline.md)): compressed blocks in a
    timeline's first run, a super timeline's carried resources, the reset plan's loads.

A timeline's end (draft) hard-resets the career itself to run 1: [timeline.md](timeline.md).

**Career bonuses.** Derived from the career on demand, never stored:

- *Starting credits*, above (applied once, at run start).
- *Boredom resilience*: `boredom_resilience_base ^ (best_run_days / boredom_resilience_period)`
  multiplies boredom growth ([boredom.md](boredom.md)). A fractional power, through `libm`.
  At 800 days, 0.990025.
- *Buy Power scaling*: `1 + max(0, peak_power − buy_power_peak_threshold) × buy_power_per_energy`,
  folded into the `buy_power_mult` modifier, which multiplies both Buy Power's cost and its
  energy ([programs.md](programs.md)). `peak_power` is the most energy buildings made (gross,
  before upkeep and caps) in one day of any finished run, folded in at retirement, so the
  bonus is fixed for the whole run (the handoffs' "re-derived on run start").

## Actions

- **Retire** (`{"type": "retire"}`): voluntary retirement, exactly as a forced one but logged
  `retired`. Illegal until the `retirement` panel is enabled this run, which quest 1.5
  Burnout (the first retirement, forced by boredom) does; seen events reapply it at every
  later run start.

## Params

<!-- params:career -->
| Param | Baseline | Range | Meaning |
|---|---|---|---|
| `credits_bonus_fraction` | 0.02 | 0 to 10 | Starting credits: this fraction of the best run's credits earned (whole credits). |
| `boredom_resilience_base` | 0.995 | 0.000001 to 1 | Boredom growth multiplier per resilience period of the longest run. |
| `boredom_resilience_period` | 400 | 1 to 1000000 | Days of the longest run per resilience step. |
| `gap_days` | 1825 | 0 to 100000 | Days of the retirement gap, simulated day by day (slow systems only). |
| `buy_power_peak_threshold` | 100 | 0 to 1000000 | Peak energy production per day above which Buy Power scales. |
| `buy_power_per_energy` | 0.01 | 0 to 10 | Buy Power scaling per unit of peak production over the threshold. |
<!-- /params -->

The starting resources and buildings are economy params (`configs/baseline.json`).

## Content

`content/events.json` `enable_nav_panel: retirement` (1.5 Burnout);
`content/projects.json` `starting_buildings` rewards; `content/achievements.json`
`bonus_buildings` rewards; `content/commands.json` `scale_modifier: buy_power_mult` (Buy Power).
The panel id `retirement` is fixed in code (`career::RETIREMENT_PANEL`: retiring needs it
enabled), and so is the modifier key `buy_power_mult` (`modifiers.rs` adds the career's part).

## Port notes

- Conflict: starting credits. The handoffs say best run credits / 100; the data's
  `career_credits_bonus_fraction` is 0.02 (/50). The engine takes the data.
- Conflict: starting land. The data starts a run with 30 land, a Solar Panel and a Data
  Center; the 30 is free land, and the starting buildings take none.
- Conflict: milestones. The data has `game_config.milestones`; the handoffs removed them. Not
  ported.
- Candidate: peak power is the gross energy buildings made in a day.
- Candidate: a new run puts every processor on Program 1.
- Candidate: the day. Forced retirement is checked after the day count rises, and the next
  run starts at once, in the same tick.
- Candidate: quests ignore an event's `game_start` run filter, so the quest chain picks up
  where the career left off ([events.md](events.md)).
