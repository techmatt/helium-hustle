# 122 bug hunt

This hunt covers recent changes since the 090 hunt (`47cb49f`), in the planner's focus order: popups
and arrival modals, the right panel (side quests fold, Events), the nav, program lights, Projects
(funding cap, hold-to-repeat, actual rates), research reveals by run trigger, Ideologies, no
partial purchases, and the quest chain and economy. Findings and what became of each are in
[docs/qa-findings.md](../../../docs/qa-findings.md), rows 117-122 and the deferred and won't-fix
tables.

## Rounds

| Round | Lens | Found |
|---|---|---|
| 1 (6:56-7:20pm) | qa sweep (seeds 51 and 52 at 1280 and 1920, both themes) and probes 1-4 of the changed areas | 3 user-facing (rows 117, 118, 119), 2 polish (120, 121), 1 checker flake (122) |
| 2 (7:20-7:30pm) | Odd input on the right column (probe 5: folds and reload), the long frame qa2 showed (probe 6), a real-speed look (`HH_PACE=120`) | Only polish: a red program light on a run's first day (deferred); a debug-only slow frame closing the Cortex mock (won't fix, tagged known) |
| 3 (7:30pm) | Reading `store.ts` for state that wrongly survives a new career or a retirement | Nothing |

The hunt stopped on yield after rounds 2 and 3.

## Probes

Run each from the repo root against a `vite preview`: `node experiments/results/122_bug_hunt/probeN.mjs <url> [width]`.

- **probe1**: Projects' −/+ held with a real mouse (paused, from 0), the hold's leave and
  release, Max at the 30-a-day cap, then the side quest and Events popups (the clock held,
  the speed back after OK). Screenshots of Projects, Research and Ideologies.
- **probe2**: each fundable project's + held for 1.5 s from 0, with the button's y before and
  after. Before the fix the button moved by −107 px and +43 px and the hold stopped at
  1/day. After the fix the hold reached 12/day (`hold-before-step.png`). The
  remaining 150 px jump is the debug scenario's start notice clearing (won't fix).
- **probe3**: popups across a retirement at 10x. Burnout pops up in run 1. Run 2 (180
  days) has none. The speed is 1x after Continue.
- **probe4**: an Events line clicked five times (one popup), a side quest line under it
  (refused), Space and 3 (the clock held), Esc (no effect), the OK double-clicked. Before the
  fix the second click opened Options' "Erase this career and start over?" (row 118).
- **probe5**: the right column folded. Side Quests' peek fits at 1280. The folds reset on
  reload, as designed.
- **probe6**: long tasks in scenario `everything` while opening Projects, holding +, and
  opening and closing the Cortex mock. None over 52 ms on a quiet box, so the 567 ms frame qa2
  timed before its project-held shot isn't the hold. With a capture added after the Cortex
  mock closes, the final pass (seed 54, 1920, dark) timed 634 ms before that capture.
  Closing the mock is the slow frame, the twin of its known slow open, and qa now reads it
  as known (the won't-fix row widened).

## Checks this hunt added

- qa's project probe: a real 1.2 s hold on the first row's + from 0 must take at least 3
  steps and none after release.
- qa's popup probe (scenario `late_unlocks`): an Events popup over Options holds the clock,
  and its OK double-clicked opens no confirm and leaves 3x running.
- A `contrast.mjs` pair: card on `--accent-text` (an eternal scope icon).
- e2e section 11 waits for the side quest popup's picture to load.

## Not findings

- An untouched first visit sits at 0 credits (pace at 1x: day 90 after 2 minutes) with 1.1's
  goal "Build a second Solar Panel". Matt ruled no how-to hints (docs/rulings/ui.md), and the
  empty program says "Add commands from the Commands panel".
- 1.5 A Voice asks for an ideology rank. The Ideologies panel opens with Geopolitical
  Intelligence (480 science, offered while A Voice is active), so in qa's passes it opened
  in run 2. This is the content as designed.
- The resource names wrap in the left column at 1280 ("Circuit Boards"). This is by design:
  the CSS lets a long name wrap rather than cut the rates.
