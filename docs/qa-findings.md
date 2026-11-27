# QA findings

What bug hunts found in the playable game, and what became of each: **fixed** (with the
check that now guards it, where one could), **deferred** to Matt with a recommendation, or
**won't fix** with the reason. Read this before a hunt and never re-report an item on it.
The checker that finds most of these is `web/scripts/qa.mjs` ([web/TESTING.md](../web/TESTING.md),
*QA checker*).

## Fixed

Rows 1-116 (bug hunts 007 to 090) are fixed, summarized here by kind so a hunt doesn't
re-report them; the full rows are in git history (rows 1-77: this file before 032; 78-93:
before 090; 94-116: before 122). A new fixed row continues the table below, and a later prune folds it here.

- **Numbers and text** (#3, 13, 20, 32, 44, 45, 47, 53, 54, 64, 70): `num()` rounding and
  separators; signed ideology scores and next-rank targets; one floored `ranPercent`; "had"
  never equal to "need" when short; full pads, cargo and bars never read full early; "1 day";
  decimals under 10; capped and spare-power wording.
- **Content and rules prose** (#1, 17, 21, 22, 58, 59, 66, 67, 71, 74): `{rate}` filled by the view;
  every completed main quest is told, in order;
  content texts and rules pages rewritten to what the game does (no play change); Open Horizons says "in
  one run"; 1-based program numbers in errors.
- **Layout and overflow** (#16, 19, 24, 27, 28, 30, 33, 34, 38, 39, 57, 73, 75): tooltips on cut
  names; one value column; the left column fits at 1280 (horizon shrinks, a two-column icon-and-name
  nav, 109); the right column keeps the scrollbar's room and is 30 px wider; bundled 600 bold;
  dots on nav tile corners; note rows and unread edges keep their room.
- **Input and clock** (#2, 4, 5, 8, 26, 31, 37, 61, 63): a confirm belongs to its run; key
  repeat and modifiers ignored; `setSpeed` settles first; a jump's compute time is forgotten;
  cards don't take focus, ignore clicks 0.5 s after Continue or OK, and are the research button;
  no action behind the run summary; a canceled drag clears.
- **Saves, tabs and the play log** (#6, 7, 9, 10, 11, 18, 23, 25, 36, 60, 62): a second tab
  stands down; failed and blocked saves say so, repeatedly; play log headers, retries, beacons,
  `session_start`, jumps and its folder.
- **Stats, breakdowns, markers** (#12, 14, 15, 29, 35, 41, 42, 43, 46, 51, 52, 55, 56, 65, 72):
  "Recent dumps"; the view's `at_limit` and `retires_next`; no frame without a processor; qa
  audits Projects, Ideologies and scenarios; breakdown rows and cell colors; partial-run notes;
  markers that go when their item hides; live overclocks only.
- **Upkeep and building state** (#40, 48, 49, 50, 68, 69, 77; revision 6 and Matt's rulings):
  effects from paid copies, `floor(active × fraction)`, unpaid copies go first, processor
  assignments stay and a short day runs `min(assigned, working)`; "Boredom cap"; pad pause and
  1-based pad errors; Short of inputs only below a full run.
- **Checker flakes** (#76): the e2e theme-layout check compares from the top.
- **Rows 78-93:** a notice over a takeover; Projects' Max then − sends six places; Load and
  Fund verbs not pickable with nothing to name; Three Voices' rank progress; flag rewards' `text`;
  faked bold (h4, Plex Mono 600); eight speed buttons at 1280; the left column's scrollbar
  room; Saga bars' own width; Esc behind the summary; 1.1 Online's pin names Sell Cloud Compute; the
  20-day average a true mean; Story by day; overclock `{days}`; "the next day"; `{rate}` as
  Stats rounds it.
- **Rows 94-116:** e2e's "if held" reads the check mark only; Esc and Enter under a story
  modal (no view closed, one page a click or key, the button focused on every page); a
  beat or Add to Program ignores clicks 0.5 s after a modal; a modal on the day its quest
  changed at any speed; Esc in a Programs number box keeps the number; a same-number edit
  sends nothing; Text / Blocks toggle and medal star contrast; faked bold on act bands, a
  Ready pad and "complete"; compact nav tiles at 1920; a cut program line's title; the boot
  line removed; the Programs status strip's boredom bar; qa's false findings and run 1's
  Full Load; Run by hand's rows (moot since 091).

Guards that cover them: `the_view_fills_every_text_placeholder`, `a_play_log_replays_…`,
`a_partial_pass_building_that_ran_in_full_is_not_starved`, `unpaid_upkeep_withholds_…`,
`forced_retirement_…`, `npm run unit`, e2e, and qa's *text*, *overflow*, *program frame*,
*input*, *card probe*, *story modal*, *contrast*, *second tab*, *assets*, *play log*,
*breakdown*, *hidden*, *unread* and *storage blocked* checks.

New fixed rows go in a table here: `| # | Finding | Fix | Guard |`.

| # | Finding | Fix | Guard |
|---|---|---|---|
| 117 | A held project −/+ from 0 stepped once: the first step added the card's status line (and reflowed its row of cards), the button slid from under a still pointer and its leave ended the hold | The press captures the pointer (release ends it); a fundable card always has its status line ("Not funded") | qa project probe (a real 1.2 s hold from 0) |
| 118 | A double-click on a popup's OK: its second click reached what lay under it (Options' "Start a new career…" opened its confirm; a third click could erase the career). Only cards, beats and Add to Program had a guard | No pointer click outside a modal counts for 0.5 s after one moves, page-wide; a −/+ press too | qa popup probe |
| 119 | Scope icons (◆, ∞; 099) read 3.99:1 in the light theme | On `--accent-text` (a `contrast.mjs` pair) | qa *contrast*, `npm run unit` |
| 120 | Faked fonts: an idle program tab at Rajdhani 400 (114), the ideology rank figure at Plex Sans 700 (118) | 600 | qa *assets* |
| 121 | A Story card's first sentence cut again by its box (108), with no hover | The line is its own title | qa *overflow* |
| 122 | e2e section 11 read the side quest popup's picture before it loaded (failed once beside qa) | Waits up to 3 s for it | e2e |

## Deferred (Matt's decisions)

| Finding | Recommendation |
|---|---|
| The first visit always starts seed 0 | Random seed, as *Start a new career* |
| Speculator Analysis promises burst timing, targets and sizes; nothing shows them | Build the reveal, or rewrite the text |
| Stats shows internal modifier and overclock keys | Display names in content |
| Building cards show content rates, before modifiers, unsaid | Effective rates, or a "base" label |
| Adversaries' dumps counted from the 50 kept lines | A per-run counter (a state change) |
| A save holds every action: ~40,000 actions outgrow localStorage | Record in the play log, or compress |
| At 10x a late wake runs two days per redraw and the frame skips a line (an e2e flake under load) | Frame from the days actually run |
| "Output at cap" beside its resource at 0: the stall is judged before production | A rules item for the balance work (Matt) |
| After a timeline ends, Blueprint Archive (and Conditional Logic, the Telescope) is offered again; funding it gives nothing new | Hide a project whose effects are all spent, or say so on its card |
| A Blocks number box being typed in keeps its draft on its row index: an instruction inserted above it (keyboard focus only) moves the draft to another line | Key the rows by a stable id |
| qa once (090, seed 45 at 1280): the debug autopilot's Everything reached super timeline 4 and the page answered nothing for 30 s, so the pass never reached the ending; the same seed passed on a rerun | Debug only; if it recurs, have the autopilot yield between chunks in late timelines |
| A refusal notice stays up until the next accepted action or its ×, across days and a retirement (seen 160 days on, under 089's since-removed Run) | Clear it at a run's end, or fade it after ~5 s |
| A program's tab light is red on a run's first day (and a scenario's start) before any day has run: "no steps ran", nothing wrong yet | No light (or a gray one) until the run's first day has played |


## Won't fix

| Finding | Why |
|---|---|
| Sell All's native `confirm()` lets game time run while it is open, then plays it before the sells | The clock catches up on wall time by design (as for a background tab); the sells land between the right ticks and a run that ended is refused with a notice |
| Opening the Cortex mock takes one 300-400 ms frame (the whole wheel drawn at once), and closing it one of 570-630 ms under qa's load (122; 52 ms or less on a quiet box) | A UI-only mock: the real Cortex's build decides how it draws |
| Starting a debug scenario takes a 0.3-1.3 s frame | Debug menu only: a new game built and its first view drawn |
| Jump ahead plays no days when the wake settling it opens a story modal | Debug only: the modal is shown, the jump marked in the play log |
| The autopilot's Everything from scenario story_acts never reaches the ending | Debug only: a display scenario whose quests are marked done with no game behind them |
| The first panel opened after Story takes a 100-160 ms frame (qa: "before panel-achievements"; later switches 20-30 ms): likely the act bands, CSS backgrounds, decoding on the main thread | Once a visit, under the 250 ms bar on a quiet box (300-520 ms under qa's load); `<img decoding="async">` bands if it grows |
| The debug autopilot's *Everything* ends on a 1.3-1.4 s frame (~28,000 days in ~12 s) | Debug only: its last chunk and the ending drawn at once |
| Silicon Valley wants 1,000 circuits on hand; the cap is 10 + 20 per Storage Depot (about 50 depots) | Placeholder content, Matt's call in 068 |
| Boredom phases 3 to 5 say "Dream effectiveness reduced to 20%/15%/10%"; Dream is flat | Placeholder content, Matt's call in 068 |
| A first note ("needs …") on a program line grows that line once | Matt's call in 068: the row is kept on purpose, so a note coming and going day to day moves no line below it |
| A held project −/+ keeps stepping when the pointer drifts off it (122) | Captured on purpose, so a reflow can't end the hold; release ends it, and it stops at 0 or the cap |
| A debug scenario's start notice, cleared by the first action, moves the panel up ~150 px under the pointer | Debug only; a real refusal notice is the deferred row above |
