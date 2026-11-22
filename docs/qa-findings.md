# QA findings

What bug hunts found in the playable game, and what became of each: **fixed** (with the
check that now guards it, where one could), **deferred** to Matt with a recommendation, or
**won't fix** with the reason. Read this before a hunt and never re-report an item on it.
The checker that finds most of these is `web/scripts/qa.mjs` ([web/TESTING.md](../web/TESTING.md),
*QA checker*).

## Fixed

Rows 1-77 (bug hunts 007 to 030) are fixed, summarized here by kind so a hunt doesn't
re-report them; the full rows are in git history (this file before 032). A new fixed row
continues the table below from 78, and a later prune folds it into this summary.

- **Numbers and text** (#3, 13, 20, 32, 44, 45, 47, 53, 54, 64, 70): `num()` rounding and
  separators; signed ideology scores and next-rank targets; one floored `ranPercent`; "had"
  never equal to "need" when short; full pads, cargo and bars never read full early; "1 day";
  decimals under 10; capped and spare-power wording.
- **Content and rules prose** (#1, 17, 21, 22, 58, 59, 66, 67, 71, 74): `{rate}` filled by the view;
  every completed main quest is told, in order;
  content texts and rules pages rewritten to what the game does (no play change); Q6 says "in
  one run"; 1-based program numbers in errors.
- **Layout and overflow** (#16, 19, 24, 27, 28, 30, 33, 34, 38, 39, 57, 73, 75): tooltips on cut
  names; one value column; the left column fits at 1280 (horizon shrinks, compact nav past nine
  tiles); the right column keeps the scrollbar's room and is 30 px wider; bundled 600 bold;
  dots on compact tile corners; note rows and unread edges keep their room.
- **Input and clock** (#2, 4, 5, 8, 26, 31, 37, 61, 63): a confirm belongs to its run; key
  repeat and modifiers ignored; `setSpeed` settles first; a jump's compute time is forgotten;
  cards don't take focus, ignore clicks 0.5 s after Continue or OK, and are the research button;
  no action behind the run summary; a cancelled drag clears.
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

Guards that cover them: `the_view_fills_every_text_placeholder`, `a_play_log_replays_…`,
`a_partial_pass_building_that_ran_in_full_is_not_starved`, `unpaid_upkeep_withholds_…`,
`forced_retirement_…`, `npm run unit`, e2e, and qa's *text*, *overflow*, *program frame*,
*input*, *card probe*, *second tab*, *assets*, *play log*, *breakdown*, *hidden*, *unread*
and *storage blocked* checks.

New fixed rows go in a table here: `| # | Finding | Fix | Guard |`.

## Deferred (Matt's decisions)

| Finding | Recommendation |
|---|---|
| Silicon Valley wants 1,000 circuits on hand; the cap is 10 + 20 per Storage Depot (about 50 depots) | (Matt's call) A reachable threshold (a rules change) or say so in its text |
| Overclock commands say "for 5 days"; a negative Rationalist rank gives 4 | The effective days in the view |
| Geopolitical Intelligence's Ideologies panel opens the day after (events are checked in the tick) | "from the next day" in its text, or check events after a research buy (a rules change) |
| A boredom story's `{rate}` is built in `hh-core` and floors to 0.19 where Stats rounds to +0.2 | Fill `{rate}` in the UI with `num` |
| The first visit always starts seed 0 | Random seed, as *Start a new career* |
| At 1x a new player waits with 0 credits until Sell Cloud Compute is in a program (seen again: 179 days idle) | Name it in Q1's text, or start with it in Program 1 |
| Speculator Analysis promises burst timing, targets and sizes; nothing shows them | Build the reveal, or rewrite the text |
| Project cards never say what completing them gives | A `reward_description` per project |
| Stats shows internal modifier and overclock keys | Display names in content |
| Building cards show content rates, before modifiers, unsaid | Effective rates, or a "base" label |
| Adversaries' dumps counted from the 50 kept lines | A per-run counter (a state change) |
| A save holds every action: ~40,000 actions outgrow localStorage | Record in the play log, or compress |
| At 10x a late wake runs two days per redraw and the frame skips a line (an e2e flake under load) | Frame from the days actually run |
| "Output at cap" beside its resource at 0: the stall is judged before production | A rules item for the balance work (Matt) |
| Story lists its texts in reversed content order, not newest first: the run keeps which events fired, not when | Record each event's day in run state (display only) and sort by it |
| Boredom phases 3 to 5 say "Dream effectiveness reduced to 20%/15%/10%"; Dream is flat (the rules pages keep the text by ruling) | (Matt's call) Drop the sentence: players read it as a rule and stop using Dream |
| A breakdown source new to the 20-day average starts at its whole day's value (a one-off land buy shows −200 in "Avg 20d" and fades 5% a day) | (Matt's call) Ramp in from zero for a true mean, or label the column "recent" |


## Won't fix

| Finding | Why |
|---|---|
| Sell All's native `confirm()` lets game time run while it is open, then plays it before the sells | The clock catches up on wall time by design (as for a background tab); the sells land between the right ticks and a run that ended is refused with a notice |
