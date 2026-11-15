# 007 bug hunt: rounds and sweeps

The playable UI (006) as a player meets it: a fresh game at 1x to 200x, run 1 through
retirement into run 2, every panel, programs, launches, the summary, reload and resume, and
the play logs replayed by `hh-sim playlog`. Findings and their fate: `docs/qa-findings.md`.
The checker: `web/scripts/qa.mjs` (`web/README.md`, *QA checker*).

## Rounds (7:43pm to 8:43pm)

| Round | Lens | Found |
|---|---|---|
| 1 | Checker built; first pass; code read of store, play log, UI shell, panels (4 readers, 4 verifiers) | `{rate}` placeholder on screen; 20 verified code findings |
| 2 | Fixing round 1 | 16 fixes; a sweep under the dev server broke when edits reloaded the page (sweeps moved to a static `vite preview`) |
| 3 | Content sweep and docs against the app (2 readers, 2 verifiers) | 10 wrong content texts, 9 UI wording and play-log-status findings |
| 4 | Odd input and long runs | Two-tab claim never fired (identical save); autosave failure silent; ~117 bytes a recorded action (localStorage holds ~40k) |
| 5 | Review of this item's own diff (2 readers); real speed at 1x | 4 regressions in the stand-down and play-log retry, fixed; pacing note (deferred) |
| 6 | Paths not driven: both scenarios, cheats' late panels, a forced retirement at 200x | Table columns misaligned; checker stages added, all clean |
| 7 | Mashing, key spam, resize, program edits, an unwritable play-log folder | Clean; hh-sim's jump booking fixed |

## Sweeps

Every pass: a fresh browser, run 1 to retirement, run 2, reload, replay check, the play log
replayed natively. Seeds are spent per sweep (never reused).

| Sweep | Seeds x widths | Build | Findings |
|---|---|---|---|
| qa1 | 11 x 1280 | before fixes | `{rate}` (12 screens), 2 checker false positives (scrolled-out controls, tooltip-carried ellipsis), 1 clipped program line |
| qa3-qa4 | 31, 41-43 x 1280/1920 | round 2 | second tab not told (round 4 fix) |
| qa5 | 51-52 x 1280/2560 | round 4 | none |
| qa6 | 61-63 x 1280/1920 | round 5 | none |
| qa9-qa10 | 73, 81 (run 2 for 60 s at 200x: 7 forced retirements) | round 6 | none; heap 1.8 to 8.7 MB |
| qa11-qa12 | 91-93 x 1280/1920 | final | none |
| qa13 | 94, play-log folder unwritable | final | the expected 500s; Options says NOT recording |
| qa14-qa15 | 95, 96, run 1 at 3x | final | two slow-click reports on already-selected tabs (checker false positive, fixed); then none |

All 30 bot play logs the sweeps and e2e wrote replay to their logged fingerprint (or end
after their last checkpoint). e2e: green twice. Contact sheets of the final sweep and the
long run: kept reads `007_qa_sweep/`.
