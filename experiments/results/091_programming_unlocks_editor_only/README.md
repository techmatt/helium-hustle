# 091: Programming unlocks only the editor

Corrects 089/090 (Matt, 2026-11-25). Rules revision 31.

- **No Run anywhere.** `run_command`, `programs::run_by_hand`, `by_hand` (engine and view),
  the "Commands by hand" breakdown source and the Commands cards' Run button are gone (090's
  Idle refusal went with them). A Commands card has Add to Program again (Locked while its
  command is), keeping 090's guard against a modal double-click's second click.
- **Programs from day 1:** the right column's program, Add to Program and loadout Save/Load
  show on a new game, with the "Empty. Add commands..." and "days per redraw" hints back.
- **Programming** (200 science, on sale after 1.4, career flag) now gates only the Programs
  nav tile and its view (`Programs.tsx` `shown`, `nav.json`). Kept: the career flag, its
  never-shown-again research row, the pre-29 save migration, scenarios naming the flag.
- **Sims and the golden script** never buy it: `driver::RESEARCH` and the author's list drop
  it; `agents::NEVER_BUY` / `buys_research` keep random, greedy and the oracle off it, and
  the career meter's `something_to_buy` ignores it (or idle time would read low). The
  hands-off pin test is back to run 3 (it ships and buys there again).
- **Text:** 1.1's story gets a third page naming Add to Program on Sell Cloud Compute;
  1.5's body is back to its pre-089 words; the summary and Retirement list saved programs
  always, "Programming" once bought. The Processors row was never hidden; it stays.

## The right column alone covers 1.1-1.8

Checked in the code and by `npm run qa`, whose run 1 plays 1.1 to the forced retirement
through the right column only (`page.runOnly`: tab, Reset, Add to Program), then edits
Program 2 there (processor −/+, two adds, line −/+, ×, drag to reorder):

| Need | Right column |
|---|---|
| add | Commands cards' Add to Program (to the selected tab) |
| reorder | drag a line (`ProgramLines`) |
| delete | × per line; Reset clears the tab |
| numbers | − / + per line (repeat count); processors − / + |
| logic lines (after 1.8) | its own add row (`AddLogic`: if, else, build, buy land, verbs) |
| keep programs | Save / Load (library) |

Only typed counts, notes and the text form need the Programs view; none is needed for
1.1-1.8, so no control was added.

Shot: [day1-program.png](day1-program.png) (e2e, a new game's Commands panel and program).
