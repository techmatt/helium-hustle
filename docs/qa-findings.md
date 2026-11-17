# QA findings

What bug hunts found in the playable game, and what became of each: **fixed** (with the
check that now guards it, where one could), **deferred** to Matt with a recommendation, or
**won't fix** with the reason. Read this before a hunt and never re-report an item on it.
The checker that finds most of these is `web/scripts/qa.mjs` ([web/README.md](../web/README.md),
*QA checker*).

## Fixed

| # | Finding | Fix | Guard |
|---|---|---|---|
| 1 | Boredom phase event texts showed a raw `Current rate: {rate}/day.` | The view fills `{rate}` with that phase's boredom per day under the run's multiplier | `the_view_fills_every_text_placeholder`; qa *text* |
| 2 | An unconfirmed Retire Now survived a forced retirement: one click on "Yes" ended the new run on day 0 | The confirm belongs to its run; "Yes" follows the engine's `can_retire` | |
| 3 | Numbers rolled over to "1000K"; 100 to 9,999 lost their separators | `num()` rounds first, suffix from the rounded mantissa | `npm run unit` |
| 4 | Held space toggled pause at key-repeat; Ctrl/Alt+digit changed speed | Ignored | |
| 5 | A speed change whose owed ticks ended the run left the clock "running" under the summary | `setSpeed` settles first | |
| 6 | Debug *New game* with a refused seed ended the live career; `+5` lost the log | State built first; the engine's seed | |
| 7 | Two tabs on one save overwrote each other | A tab stands down when another saves | qa *second tab* |
| 8 | A jump ahead's compute time was played as real time | The clock forgets it | |
| 9 | Play log: a resent header made `hh-sim playlog` refuse the career | Repeated header dropped | `a_play_log_replays_…` |
| 10 | Play log: a failed probe stopped logging; the batch in flight at close was lost | Retries; beaconed | |
| 11 | A career begun mid-visit had no `session_start` | `newGame` logs one | |
| 12 | Research kept "Needs X first." after X was done | Only while undone | |
| 13 | A negative rank-0 ideology read "next rank at 10", green | Sign and tone follow the score | |
| 14 | "Dumps this run" counted only the 50 kept lines | "Recent dumps" | |
| 15 | "At its limit" computed in the client | The view's `at_limit` | |
| 16 | A program line's name cut short at 1280 px | Its tooltip has it | qa *overflow* |
| 17 | Ten content texts said what the game doesn't do (Overclock's cap, Arbitrage Engine, Bulk Shipper, Research Lab, Storage Depot, Load Launch Pads, Data Center, Battery, Fund names, Dream Protocols' cost) | Rewritten to the rules (no play change) | |
| 18 | Options showed literal backticks; an unwritable log folder showed "recording" | Plain text; a 5xx keeps the lines, retries, and Options says NOT recording | |
| 19 | Recent Activity counted 50, showed 40 | All shown | |
| 20 | Left ideology row dropped a negative score's minus | Signed | |
| 21 | The Market Awareness note outlived the research; "One Load Launch Pads moves up to 5" | Note while hidden; "Each … run loads up to 5" | |
| 22 | Stats' "Boredom growth" differed from the status bar's unexplained | "Base boredom growth (before …)", "phase 2 of 5" | |
| 23 | A failed autosave was silent | A notice | |
| 24 | Stats, Retirement, summary values in different columns | One column | |
| 25 | `hh-sim playlog` booked a stopped jump as ticks at speed | Counted as jumped | |
| 26 | A clicked card kept focus: the next space bought instead of pausing | A press doesn't focus a card | e2e; qa *card probe* |
| 27 | At 1280 to 1366 px the left column grew a scrollbar: tile names broke mid-word, rates were cut off | The horizon shrinks first; tiles fit beside a scrollbar; names wrap between words | qa *overflow* |
| 28 | Recent Activity outgrowing the window gave the right column a scrollbar mid-play: program lines jumped | The right column always keeps the scrollbar's room | qa *program frame* |
| 29 | A program with no processor framed a line as the one it runs next | No frame without a processor | qa *text* |
| 30 | Bold text in IBM Plex Sans 700, not bundled: a faked bold | Bold uses the bundled 600 | qa *assets* (font not bundled) |
| 31 | A double-click on Continue (or a story's OK) sent its second click to the card under it: a buy | Cards ignore pointer clicks 0.5 s after either closes | qa *input* |
| 32 | Amount and rate ran together ("43.6 / 50−0.2/day") | A fixed cell gap | qa *overflow* |
| 33 | Loading priority arrows in a ragged column | One grid | |
| 34 | A one-word resource name wrapped below its swatch | Swatch stays with the word | qa *overflow* |
| 35 | qa never audited Projects or Ideologies | Late phase on the pass's career | |
| 36 | Automation's play log went to `out/playlogs/`, Options named the synced folder | The page asks the server | qa *play log* |
| 37 | Research cards outlined green but only their button researched (Matt's ruling) | The card is the button (`cardButton`) | e2e; qa |
| 38 | Late at 1280 px the left column scrolled under 12 tiles (Matt's ruling) | Compact nav past nine tiles | |
| 39 | Right column cut program line names at 1280 px (Matt's ruling) | `--right` 30 px wider | qa *overflow* |
| 40 | "+100 Boredom storage" read as storing boredom (Matt's ruling) | "+100 Boredom cap" | |
| 41 | The qa checker's scenario step read the scenario list with the Debug fold shut: it audited no scenario | Opens the fold first; an empty list is a finding | qa *input* |
| 42 | A breakdown kept a stopped source's row for hundreds of days, its cells blank | Rows with nothing to show are left out | qa *text* (row with no numbers) |
| 43 | Stats' breakdown colored the average cell by the last day's sign (a red "+4"); Boredom's sources read green for growth | Each cell its own color; Boredom's the other way round, as its rate | qa *text* (colored against its sign) |
| 44 | A run share floored on the card, rounded in its tooltip and the breakdown ("ran at 100%" when short) | One `ranPercent` (floored) everywhere | qa *text* (status vs tooltip) |
| 45 | "needs 150, had 150" when 149.6 was held | `had` never reads equal to `need` | |
| 46 | A conversion line that ran part way showed a full bar and no reason | The line says "ran at N% · needs …" | |
| 47 | A full pad cooling down read "loading"; "1 days"; cargo 99.96 read "100 / 100" | "Full: ready in N days"; "1 day"; never full early | qa *text* (full pad reads loading) |
| 48 | Pause loading didn't say a paused pad also stops Launch Full Pads | Its tooltip says so | |
| 49 | Pad numbers 0-based in `set_pad_good` / `set_pad_paused` errors | 1-based | |
| 50 | A partial-pass building that ran in full was still marked Short of inputs | Marked only below a full run | `a_partial_pass_building_that_ran_in_full_is_not_starved` |
| 51 | Processors' row opened "Nothing moved it the last day" (processors are counted, not made) | The row opens nothing; its tooltip says why | qa *breakdown* |
| 52 | An open breakdown row reopened by itself when its resource showed again | It closes when its resource hides | |
| 53 | The left column's ideology row dropped the minus of a negative score's next-rank target ("−5 / 10") | "−5 / −10" | |
| 54 | Capped rate tooltip "+6/day is lost each day", and Energy's spare power "lost" | "6 a day is lost"; Energy: "spare power goes unused" | |
| 55 | A Research card hidden again before it was hovered left Research's nav dot on all run | A marker goes when its item hides | |
| 56 | Recent Activity counted unread lines but marked none | Unread lines carry the accent edge, as Story's | qa *text* (badge vs items) |
| 57 | Compact nav: the Launch Pads ready dot sat over the name | Compact tiles wear their dots on the corners | qa *hidden* (dot over the tile name) |
| 69 | Revision 6 review: switching off or selling a short building took a paid copy; a card read "running at <1%" at 0; an unpaid pad read "inactive" | Unpaid copies go first; "not running"; "idle: upkeep unpaid" | `unpaid_upkeep_withholds_…` |
| 70 | Speculator pools under 0.5 read "0" though Disrupt Speculators still targets them; "N / 20K" switched number forms | A decimal under 10; "under 20K" | |
| 71 | Rules pages cited stale day steps (achievements, projects, events) and "active" Arbitrage Engines | Current steps; "working" | |
| 72 | The day before a forced retirement every rate read 0/day (the view's peek lands in the next run) | The view's `retires_next`: "cap tomorrow", rates "—" | `forced_retirement_…` |
| 73 | At 3x a line's note row came and went day to day, moving the lines below (seen once, seed 55, 1440 px) | A line keeps the row once shown, until its program changes; a first note still grows its line once (a row reserved on every line: Matt's call) | qa *program frame* |
| 74 | Q6 "Earn 100,000 credits in one lifetime" counts one run (0 / 100K after each retirement) | "in one run" (content hash only) | |
| 75 | An Events line's unread edge took 6 px of room: a line read shifted and could rewrap, moving the lines below (#56 exposed it in Recent Activity) | Every line keeps the edge's room; read changes its color | qa *unread* (lines moved as read) |
| 77 | Revision 6 unassigned a short Data Center's processor; paid again, it came back free and the program stayed stopped (seed 85) (Matt's ruling) | Assignments stay; a short day runs `min(assigned, working)`, "2 assigned, 1 running" | `unpaid_upkeep_withholds_…` |
| 76 | The e2e's "same layout in both themes" failed now and then: a scroll left by earlier steps, not the theme | It compares from the top, and names the first element that differs | e2e |
| 58 | Rules pages kept the pre-revision-5 day order, "per-source tracking not built", "from the next building pass", "a pad's good changes only while empty" | Rewritten to the code | |
| 59 | A main quest that opened and completed within one redraw (Q4 with two Data Centers owned) never showed its text | Every untold completed quest is told, in order | |
| 60 | After a failed autosave, the next action cleared its notice and later failures were silent | Said again while saves keep failing | |
| 61 | Enter/space on a card still focused behind the run summary bought into the new run unseen | No action while the summary is up | |
| 62 | With site data blocked the page hung on "Loading…" (a storage read threw at module load) | Storage reads and writes are guarded; the game plays, saying it isn't kept | qa *storage blocked* |
| 63 | A cancelled program-line drag left its index set: a later drop of anything on a line moved it | Drag end clears it | qa *input* (drag probe) |
| 64 | "Short of Science." beside "200 Science" for 199.96; a project bar "2,000 / 2,000" short of done | The hint says how much more; the bar never reads full early | |
| 65 | Stats kept an expired overclock a day ("until day 12" on day 12); career bests without "(finished runs)" | Live ones only, "through day 11"; labeled | |
| 66 | Long-Term projects "carry across retirements" without saying funding rates reset | Projects and Retirement say so | |
| 67 | `set_program` / `set_processors` errors said "program 0"; content texts: "persistent project", Electrolysis "only propellant source", Solar Panel and Ice Extractor designer notes | 1-based; texts rewritten (content hash only) | |
| 68 | Data Center, Launch Pad, Arbitrage Engine short of upkeep read "running at 0%" yet worked in full (Matt's ruling) | Rules revision 6: effects come from paid copies, `floor(active × fraction)`, from the next day; their cards say "1 of 4 working" | `unpaid_upkeep_withholds_…` |

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
