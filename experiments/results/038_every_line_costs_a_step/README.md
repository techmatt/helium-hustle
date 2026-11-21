# 038: every non-`if` line costs a step

Rules revision 10. A line with nothing to do (`build X up to N` with N owned, a verb with
nothing to act on) is `done`: it takes its processor step and nothing else, and the pointer
moves on. A false `if` still skips its body free. New guard readings: `researched`, `pads`,
`running`, `processors`; `pads_ready` now counts only pads `launch pads` would launch
(unpaused), so it guards that verb.

## What it cost the sim players

Fresh seeds 3072-3079 (8 careers of 20 retirements a side, `ladder --agents
author,handsoff --seeds 8 --first-seed 3072`), the same seeds on all three sides. (3064-3071
read the same before the diff review's fixes: hands-off 93% / 2% / 99%.)

- **main**: rules 9, agents as on main.
- **unguarded**: rules 10, agents as on main (what the rule change alone costs them).
- **guarded**: rules 10, the author and hands-off player guard finished lines.

| Side | Author credits/run | Author peak cr/day | Hands-off credits/run (run 2+) | Share of author | Hands-off milestones first met from run 2 | Hands-off ships/run |
|---|---|---|---|---|---|---|
| main | 18710 | 26.0 | 17917 | 96% | 10 (author 15) | 18.0 |
| unguarded | 15049 | 20.6 | 279 | 2% | 0 (author 6) | 0.6 |
| guarded | 19725 | 30.1 | 19037 | 96% | 6 (author 15) | 18.4 |

Full tables: [ladder-main-3072.txt](ladder-main-3072.txt),
[ladder-ung-3072.txt](ladder-ung-3072.txt), [ladder-gua-3072.txt](ladder-gua-3072.txt).

- **Unguarded, the hands-off player collapses** (2% of the author's credits): its build order
  is a column of `build X up to N` lines, each a done step once built, and its lead holds
  assign/pause/resume lines that are done most days. The build order runs on one processor
  once a pad stands, so its pointer crawls through finished steps; it builds little, and its
  runs end early by its stale rule. The author, rewriting its programs at its looks, loses a
  fifth of its credits.
- **Guarded, both are back at or past main** (author +5% credits, hands-off +6%, the same
  96% share). The build order guards each step on `count < N`, research on `researched < 1`,
  the ship cycle's `set pads` on `pads on he3`, hands-off's processor lines on `processors`,
  its pause/resume on `running`.
- **Hands-off milestones from run 2 fell 10 → 6, but not from hands-off playing worse**: run 1
  (the author's, in both players) now meets M5 Positive Credit Flow inside run 1 (median day
  1069) where on main it fell in run 2 (author) or 3 (hands-off), so it no longer counts as first met from run 2.
  The measure counts only what run 1 left over.
- **Two things changed on the guarded side**, the guards and one guard's meaning: the
  author's "point a spare pad at a full good" line was `full and pads >= 2` and is now `full
  and pads on He-3 >= 2` (three clauses don't fit one `if`). So guarded vs main is not a clean
  read of the guards alone; guarded vs unguarded is a clean read of guarding (same rules).
- Left unguarded (a done step in rare states, sim cost only): `load pads all` when the He-3
  pad is full and cooling and another pad's good is out of stock; the full-good `set pads`
  line with three or more pads; hands-off's lead while no pad stands and a side program holds a
  processor (its guard reads the working processors, `assign ... all` gets only the free ones).
