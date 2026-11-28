# 134: bug hunt (recent changes since 122)

Scope: the changes since the last hunt (122, `458c102`), items 123-133, with the planner's
focus: boredom and ideology (×m, the tools, nudges, escalation, refunds, dates, slots A-E),
Electrochemistry and the Chemical Energy gate, the new project costs and Foundation Grant's
free buildings, rank and price rounding, the playlog and `vs-human --skilled`, and
`prose.py`'s fingerprinted import. Findings rows 126-136 and the new deferred and won't-fix
rows are in [docs/qa-findings.md](../../../docs/qa-findings.md).

## Rounds

| Round | Lens | Found |
|---|---|---|
| 1 | Full `npm run qa` (seeds 11-13) and a read of 458c102..HEAD by five readers, each finding checked by a second | 9 user-facing (status-bar date cut, summary date, phase `{rate}`, timeline keys, Sell text, Foundation Grant text, hh-sim panic, `--skilled` landmarks, `axes`), 8 wording or docs |
| 2 | This folder's `check.mjs` in the running game; `playlog` and `vs-human --skilled` on Matt's logs and a current qa log | 1 user-facing (the summary read the new run's day, found by the check), 1 CLI (an old log's error under the usage text) |
| 3 | Confirming qa sweep; a second read (state across save and retirement, content text against the rules, UI against the view) | 3 user-facing (a Dome's rank lost at Retire, negative carried scores hidden, the Sell All confirm on free copies), 3 polish (axis marks and carried rows before Ideologies opens, "Rank -2", "−0") |
| 4 | Confirming qa sweep (clean); a read for odd input on 123-133's UI and long-run edges of the escalation and ×m math (none) | 2 user-facing (a double-click on the last Sell 1 buys it back, or the card that slid under; the library's stale save name) |
| 5 | A real double-click on a research card (sound); full e2e | 0 user-facing; e2e red since 129 on two stale steps (row 139) |
| 6 | Real speed: qa's `HH_PACE` first visit at 1x for 6 minutes | 0 (a day a second; Burnout's popup holds the clock) |

## Checks

- `check.mjs` (this folder): the status bar's date whole; the run summary's date equal to the
  status bar's behind it, after a Retire press and after a forced retirement; Foundation
  Grant's reward text; the Sell hovers; a real double-click on the last copy's Sell 1
  buying nothing; the library's save name following the tab; the Dome's relief to two places; the Research and
  Stats wording; Electrochemistry's card at 8 Solar Panels. Run from the repo root against
  a `vite preview`: `HH_URL=http://localhost:<port>/ node experiments/results/134_bug_hunt/check.mjs out/134_bug_hunt/shots`.
- qa: its overflow check now counts an ellipsis 1 px over (it let "Year 1, day 2…" pass);
  its forced retirement checks the summary's date against the status bar; its grammar probe
  writes program slots as letters.
- hh-core: `the_last_boredom_phase_rate_counts_the_yearly_escalation`,
  `a_rank_a_dome_reaches_between_days_counts_as_a_career_best_at_retire`.
- hh-sim `cli`: `vs-human --agents , --skilled` is a usage error; a log from another
  revision names its revision. `npm run unit`: timeline keys, `signed()`.

## Notes

- Matt's play logs are all from rules revision 45 or older: `playlog` now says which
  revision a log is and points at `scripts/baseline.py`; `vs-human --skilled` runs on none of
  them (deferred row). On qa's seed-12 log (rules 46) `vs-human --skilled` reads cleanly.
- qa's second sweep flagged "the autopilot's Everything took 60-62 s" on every pass, against
  its 60 s bar; that sweep ran beside a release build and a `vs-human` read (the first sweep,
  quieter, did not flag it). Read as load, not a finding.
