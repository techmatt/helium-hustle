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
| 2 | A Retire Now left unconfirmed survived a forced retirement: after Continue, one click on "Yes, retire" ended the new run on day 0 | The confirm belongs to the run it was opened in; "Yes" is disabled when the engine says retiring isn't allowed | |
| 3 | Numbers rolled over to "1000K" / "1000M"; fractional amounts from 100 to 9,999 lost their thousands separators ("1235" beside "1,234") | `num()` rounds first and picks the suffix from the rounded mantissa | `npm run unit` |
| 4 | Space held down toggled pause at the key-repeat rate; Ctrl/Alt+digit changed the speed | Repeats and modified keys are ignored | |
| 5 | A speed change whose owed ticks ended the run left the summary up with the clock "running" at the new speed, and logged speed lines that never happened | `setSpeed` settles the owed ticks before checking for the summary | |
| 6 | Debug *New game* with a seed the engine refuses wrote `career_end` into the career that kept playing; a seed like `+5` gave a career id the log server refused, losing the log | The new state is built first; the seed is the engine's own reading of it | |
| 7 | Two tabs on one save overwrote each other's save and interleaved one play log | A tab stands down (clock stopped, no saves or log lines, a notice) when another tab saves | qa *second tab* |
| 8 | A jump ahead's own compute time was played as real time at the current speed right after `jump_end` | The clock forgets the time the jump took | |
| 9 | Play log: a resent first batch repeated the header and `hh-sim playlog` refused the whole career | The reader drops a repeated header | `a_play_log_replays_…` (cli.rs) |
| 10 | Play log: a failed first probe of the log server turned logging off for the page's life; a send in flight when the page closed was lost; queue overflow could drop the header | Probe failures stay pending and retry; the batch in flight is beaconed on leaving; overflow never drops a header | |
| 11 | A career begun mid-visit (Options new career, debug start) had no `session_start`, so `hh-sim playlog` counted 0 sessions | `newGame` logs one when the page's session is already open | |
| 12 | Research cards kept "Needs X first." after X (or the card itself) was researched | Shown only while the prerequisite is undone | |
| 13 | An ideology axis at rank 0 with a negative score showed "next rank at 10" and a green bar | Sign and tone follow the score | |
| 14 | Adversaries' "Dumps this run" counted only dumps still in the 50 kept activity lines | Relabeled "Recent dumps" with a tooltip saying so | |
| 15 | The building limit ("At its limit") was computed in the client | The view carries `at_limit` | |
| 16 | A program line's name is cut short at 1280 px with no way to read it | The line's tooltip carries the whole name | qa *overflow* |
| 17 | Ten content texts said what the game doesn't do: Overclock "cap +200%" (it is +50%, +100% with Overclock Boost), Arbitrage Engine "multiplies" decay, Bulk Shipper "cargo capacity", Research Lab "gated behind Fabricator", Storage Depot's Godot ids and missing propellant, Load Launch Pads "5 units", "fast cost scaling" on Data Center and Battery, the Fund command names, Dream Protocols' missing Helium-3 cost | Texts rewritten to the rules (`content/`; no play change, no revision bump) | |
| 18 | Options showed literal backticks; a log folder the server can't write showed "recording" while every batch was dropped | Plain text; a 5xx keeps the lines and retries with Options saying NOT recording; a static host's non-JSON answer reads as no log server | |
| 19 | Recent Activity's count (up to 50) didn't match the 40 lines shown | All kept lines shown | |
| 20 | Left column's ideology row dropped the minus of a negative score at rank 0 | Signed | |
| 21 | The Market Awareness note stayed after the research; the pad line read "One Load Launch Pads moves up to 5" | Note only while demand is hidden; "Each Load Launch Pads run loads up to 5 Helium-3" | |
| 22 | Stats showed a "Boredom growth" that differs from the status bar's with no reason, and an unexplained "phase 2" | "Base boredom growth (before commands and Dream)", "phase 2 of 5" | |
| 23 | An autosave that failed (storage full) was silent | A notice says progress is no longer kept | |
| 24 | Stats, Retirement and summary tables put their values in a different column per table | One value column | |
| 25 | `hh-sim playlog` booked a jump ahead that a forced retirement stopped as ticks played at speed | Ticks from the jump's action to its `jump_end` count as jumped | |
| 26 | A click on a building card left it focused, so the next space (the pause key) bought another and didn't pause | A mouse press neither focuses the card nor selects text; Tab still reaches it | e2e *space after a click*; qa *card probe* |
| 27 | At 1280 to 1366 px, once Launch Pads and Retirement tiles showed (or at 768 px high), the left column grew a scrollbar: tile names broke mid-word ("Command s", "Retiremen t") and the resource rates were cut off at the column edge | The horizon art shrinks before the column scrolls; tiles fit their longest name beside a scrollbar; a long resource name wraps between words before a rate is cut | qa *overflow* (cut off by a column, word broken) |
| 28 | Recent Activity outgrowing the window gave the right column a scrollbar mid-play, narrowing it: the program lines and their notes jumped | The right column always keeps the scrollbar's room | qa *program frame* (lines moved) |
| 29 | A program with lines but no processor framed its first line as "the line the next processor runs" though nothing runs it | No frame without a processor | qa *text* (frame without a processor) |
| 30 | Story banner titles, table headers, the Ideologies rank and CHEATED were drawn in IBM Plex Sans 700, which isn't bundled: the browser faked a smeared bold | Bold body text uses the bundled 600 | qa *assets* (font not bundled) |
| 31 | A double-click on the summary's Continue (or a story's OK) sent its second click to whatever slid under the pointer, usually a building card: a buy, or a "Short of Credits." | A building card ignores pointer clicks for 0.5 s after the player closes the summary or a story text | qa *input* (double-click on Continue) |
| 32 | With the late resources showing at 1280 px, a resource's amount and rate ran together ("43.6 / 50−0.2/day") | A fixed gap between the Resources table's cells | qa *overflow* (table cells touch) |
| 33 | Launch Pads' loading priority arrows sat wherever each good's name ended, a ragged column | The rows share one grid | |
| 34 | In a tight Resources table a one-word name ("Propellant", "Processors") wrapped below its color swatch, leaving the swatch alone on a line | The swatch stays with the name's first word | qa *overflow* (swatch alone on its line) |
| 35 | The qa checker never audited Projects or Ideologies (its late phase ran on a fresh scenario whose quests can't open them) | The late phase plays on the pass's own career | |
| 36 | Under automation (`bot_` careers) Options named the synced play-log folder while the lines went to `out/playlogs/`; the qa checker looked for the log there | The page asks the server for a bot's folder | qa *play log* |
| 37 | Research cards had the green "can buy" outline, but only their Research button researched (Matt's ruling) | The card is the button, as a building card is (`cardButton` in `common.tsx`), with a hint when it can't | e2e *research*; qa late phase |
| 38 | Late in a career at 1280 px the left column scrolled under 12 nav tiles (Matt's ruling) | Past nine tiles the nav turns compact: icon beside the name, columns of at least 120 px | |
| 39 | At 1280 px the right column cut program line names short (Matt's ruling) | `--right` is 30 px wider | qa *overflow* |
| 40 | "+100 Boredom storage" read as storing boredom (Matt's ruling) | Every card stores line reads "cap" ("+100 Boredom cap") | |

## Deferred (Matt's decisions)

| Finding | Recommendation |
|---|---|
| The first visit always starts seed 0, so every new player gets the same game | Random seed on a first visit, as Options' *Start a new career* does |
| At 1x a new player sees 0 credits, 0/day and "Build a second Solar Panel" with no income until Sell Cloud Compute is added to a program; days pass with nothing happening | Name Sell Cloud Compute in Q1's text, or start with it in Program 1 |
| Speculator Analysis (540 science) promises burst timing, targets and sizes on Launch Pads; nothing shows them | Build the reveal (view fields + pad panel), or rewrite the text to what it does |
| Project cards never say what completing them gives | A `reward_description` per project row, shown like achievements' |
| Stats' Active modifiers and overclocks show internal keys ("speculator burst interval mult", "extraction") | Display names in content |
| Building cards show content rates, before modifiers and overclocks, without saying so | Effective per-copy rates in the view, or a "base" label |
| Adversaries' dumps are counted from the 50 kept activity lines (now labeled "Recent dumps") | A per-run counter in run state (a state change: its own item) |
| A save holds every action: at roughly 40,000 actions (tens of hours) it outgrows localStorage; the failure is now shown | Keep the record in the play log and only state plus a count in the save, or compress |
| At 10x a late timer wake runs two days in one redraw and the frame skips a line (the e2e's "pointer walks the queue in order" failed once with a sweep running beside it) | Fine for a person; if it matters, frame from the days the redraw actually ran, not the nominal speed |
| A building's "Output at cap" can sit beside its resource reading 0 (seen: 8 Excavators, Regolith 0 / 50, seed 32 run 2 day 493 at 50x): the stall is judged at the start of the day's production (`economy.rs` `resolve`), the amount shown at the day's end | A rules item, left for the balance work (Matt): find what drew the stock from cap to 0 in a day, and whether the stall should be judged after consumption |
| The view's `{rate}` is display text built in `hh-core` (`view.rs`, `event_story`) | Fine as is; a general text-template step if more placeholders come |

## Won't fix

| Finding | Why |
|---|---|
| Sell All's native `confirm()` lets game time run while it is open, then plays it before the sells | The clock catches up on wall time by design (as for a background tab); the sells land between the right ticks and a run that ended is refused with a notice |
