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

| # | Finding | Fix | Guard |
|---|---|---|---|
| 78 | A notice or story banner raised while a takeover (text editor, Cortex mock) is open is hidden with the middle column until it closes | Shown at the top of the takeover | qa *hidden* (the notice was flagged covered) |
| 79 | Projects: Max then − sends a float like `0.6666669999999999`; the engine refuses it and the rate stays at max (Deep Space Telescope's science, Conditional Logic, Foundation Grant, Dyson Sphere) | The step sends six places | qa project probe |
| 80 | A verb line switched to Load (empty library) or Fund (no open project) is refused with an error | Such a verb is listed but not pickable, "(nothing to name yet)" | `npm run check` |
| 81 | Quest 1.7's "Rank 5 …" rows never showed how far along they are | `events::progress` counts `ideology_rank` | `a_rank_on_one_axis_shows_its_progress` |
| 82 | Conditional Logic's card says "Reward: Sets logic_if" (a flag id; Dyson Sphere too) | `set_flag` rewards carry their `text` (required) | content load |
| 83 | Stats breakdown and Project rewards headings (h4), the DRAFT tag and Cortex plaque names drawn at weights the bundled fonts lack (browser-faked bold) | h4 at 600, Plex Mono 600 bundled | qa *assets* |
| 84 | With Cheats on, eight speed buttons: "200x" wider than its button at 1280 | Smaller text and gap at eight | qa *overflow* |
| 85 | A late career's left column (16 tiles, the calendar) scrolls at 1280×800, and its scrollbar clipped the Resources rates ("fre", "−8/da", the CAP chips) | The column keeps its scrollbar's room (17 px wider) | qa's autopilot-to-the-end audit |
| 86 | The Saga's progress bars (Timeline's calendar, Vault's super timeline price) were only as wide as their label | A track of its own width | screenshots (qa `saga-*`) |
| 87 | Esc with the run summary up closed a takeover behind it, trying to save the text editor's draft to the ended run | Esc does nothing to a takeover while the summary is up | manual |
| 88 | At 1x a new player waits with 0 credits until Sell Cloud Compute is in a program (179 days idle seen) | Q1's summary (the quest pin) names it and Program 1; never pre-filled | content |
| 89 | A breakdown source new to the 20-day average starts at its whole day's value (a one-off land buy shows −200 in "Avg 20d") | A true mean over the run's days so far, up to 20: a new source enters at zero (rules 21, display only) | `a_one_off_flow_enters_the_average_at_its_share_of_the_days` |
| 90 | Story lists its texts in reversed content order, not newest first | `Run.events` records each event's day; the view sorts by it (rules 21, display only) | `story_lists_events_by_the_day_they_fired` |
| 91 | Overclock commands say "for 5 days"; a negative Rationalist rank gives 4 | `{days}` in the text, filled by the view from the effect's own `overclock_days` | `overclock_cards_say_the_days_a_run_lasts` |
| 92 | Geopolitical Intelligence's Ideologies panel opens the day after | Its text says "the next day" | content |
| 93 | A boredom story's `{rate}` floors to 0.19 where Stats rounds to +0.2 | The view sends `rate`; the UI fills `{rate}` with Stats' `perDay(rate, 2)` | `npm run unit`, `the_view_fills_every_text_placeholder` |
| 94 | e2e's "the if held once Credits > 800" read that day's line outcomes after the program moved on (the new Battery drops Energy, the top `if` runs instead): failed 5 of 7 runs in 068 | Checks the depot built and the `if`'s ✓ only (070) | e2e |

## Deferred (Matt's decisions)

| Finding | Recommendation |
|---|---|
| The first visit always starts seed 0 | Random seed, as *Start a new career* |
| Speculator Analysis promises burst timing, targets and sizes; nothing shows them | Build the reveal, or rewrite the text |
| Project cards never say what completing them gives | A `reward_description` per project |
| Stats shows internal modifier and overclock keys | Display names in content |
| Building cards show content rates, before modifiers, unsaid | Effective rates, or a "base" label |
| Adversaries' dumps counted from the 50 kept lines | A per-run counter (a state change) |
| A save holds every action: ~40,000 actions outgrow localStorage | Record in the play log, or compress |
| At 10x a late wake runs two days per redraw and the frame skips a line (an e2e flake under load) | Frame from the days actually run |
| "Output at cap" beside its resource at 0: the stall is judged before production | A rules item for the balance work (Matt) |


## Won't fix

| Finding | Why |
|---|---|
| Sell All's native `confirm()` lets game time run while it is open, then plays it before the sells | The clock catches up on wall time by design (as for a background tab); the sells land between the right ticks and a run that ended is refused with a notice |
| Opening the Cortex mock takes one 300-400 ms frame (the whole wheel drawn at once) | A UI-only mock: the real Cortex's build decides how it draws |
| Starting a debug scenario takes a 0.3-1.3 s frame | Debug menu only: a new game built and its first view drawn |
| The debug autopilot's *Everything* ends on a 1.3-1.4 s frame (~28,000 days in ~12 s) | Debug only: its last chunk and the ending drawn at once |
| Silicon Valley wants 1,000 circuits on hand; the cap is 10 + 20 per Storage Depot (about 50 depots) | Placeholder content, Matt's call in 068 |
| Boredom phases 3 to 5 say "Dream effectiveness reduced to 20%/15%/10%"; Dream is flat | Placeholder content, Matt's call in 068 |
| A first note ("needs …") on a program line grows that line once | Matt's call in 068: the row is kept on purpose, so a note coming and going day to day moves no line below it |
