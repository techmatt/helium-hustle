# 112: right panel events and popups

Browser checks of the done-when rows, against `vite preview` (a static build), run from the
repo root: `HH_URL=http://localhost:<port>/ node experiments/results/112_right_panel_events_and_popups/check_prop.mjs`
(and `check.mjs`; `STEPS=0` skips its autopilot leg). Both drive `web/scripts/cdp.mjs`.

| Done-when row | Check | Result |
|---|---|---|
| Fresh career: Propellant discovery pops up, then one line under Events | `check_prop.mjs`: autopilot to 1.3, then four hand launches with cheats; every popup on the way read | Pass: popup with its Effect; Events line "Propellant Production Feasibility Lunar propellant synthesis may be possible." (no Effect); a click reopens it |
| A side quest's completion shows a reward popup and leaves the right panel | `check.mjs`: scenario `late_unlocks`, Complete all research | Pass: 1.b's popup "Reward: Every run starts with 50 Science, for good."; no `[data-side-quest="qfull_cluster"]` after |
| No "Completed" section | `check.mjs` | Pass (and no side-quest banner) |
| Checkbox on: Burnout doesn't pop up in run 2 | `check.mjs`: new career, jump 205 days (Burnout and Boredom Rising pop up), retire at the cap, jump 205 days in run 2 | Pass: no boredom popup in run 2 (Burnout fires once a career; Boredom Rising, seen in run 1, is skipped); Events lists Boredom Rising |

Also seen: an open side quest's click opens its popup (story, Reward, Goal); the checkbox
starts on; run 1's Events shows Burnout alone the day it fires with phase 2. No page errors.

Since 113 (rules 39) Boredom Rising is skipped in the run Burnout fires in, so it first pops up
in run 2; `check.mjs`'s run-1 and run-2 assertions were updated for that in 115 (not re-run).

Screenshots (`112_*.png`, also in the Drive screenshots folder): the open side quests under
the main quest, a side quest's popup and its completion popup, the Propellant popup and its
Events line, the Burnout popup, run 1's and run 2's Events.
