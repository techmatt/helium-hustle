# 068: the 030 bug hunt's deferred rows

Six deferred rows from `docs/qa-findings.md` fixed (rows 88-93 there). Rules revision 21:
display-only state, no change to play.

- **Q1 teaches the first income.** The quest pin shows only a quest's summary, so the hint is
  in Q1's summary: "Build a second Solar Panel. Credits come from Sell Cloud Compute: add it
  to Program 1." Program 1 still starts empty.
- **The 20-day average is a true mean.** `avg += (day − avg) / k`, where `k` counts the days
  folded this run (`Run.flow_days`), up to `flow_average_days`. A new source enters at zero,
  so a one-off 200 on day 30 reads 10, not 200. Counting starts at the run's first fold that
  has a flow in it, so buys made before the first tick are kept and a scenario's starting
  sources are not reduced.
- **Story is ordered by day.** `Run.events` maps each event to the day it fired. The view
  sorts by that day, with content order breaking ties. Both Story lists show newest first.
- **Overclock cards** say `{days}`. The view fills it from `programs::overclock_days`, the
  same function the effect uses, so the card reads "4 days" at Rationalist rank −1.
- **Geopolitical Intelligence** says the panel opens "the next day".
- **The boredom `{rate}`** goes to the client as `StoryView.rate`. The UI fills it with
  `perDay(rate, 2)`, as Stats does: +0.2/day rather than a rounded-down 0.19.

Screenshots: `q1-hint-and-story-rate.png` (the Q1 banner, and Story's phase 3 at +0.5/day,
matching the Boredom rate) and `q1-quest-pin.png` (the summary in the quest pin at 1600 px).
