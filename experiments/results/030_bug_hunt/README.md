# 030: bug hunt (the 023-029 UI, the three-stage day, the breakdown)

Findings and what became of each: [docs/qa-findings.md](../../../docs/qa-findings.md), rows 41-74
and the Deferred rows added beside them. The checker: `web/scripts/qa.mjs`
([web/README.md](../../../web/README.md), *QA checker*).

## Rounds

| Round (lens) | Found |
|---|---|
| 1, recent changes (023-029): a sweep (seeds 31-33) and a five-area code read with a verify pass | Breakdown blank rows, colors and run shares; pad states and labels; unread marks; nav dots over names; stale rules pages; the checker audited no scenario |
| 2, content and client state; a 12-minute run at 200x | Chained quests never told; autosave notice lost; keys buying behind the summary; blocked storage hanging the page; drag state; labels and content texts |
| 3, paths and odd input: 2560x1440, 1280x720, 1366, 1600 px; blocked storage; drag; a run at 3x | A note row moving program lines at 3x (later fixed); revision 6 (Matt's ruling) |
| 4, a review of this hunt's own diff; an engine code read; 1x for 3 minutes | Revision 6 edge cases (switching off a short building); unpaid pads; speculator pools; stale day steps; five deferred |
| 5, deferred items reopened; a 10-minute run on revision 6 | "cap tomorrow"; note rows keep their space |
| 6, content looked at; a 600-day run 2 | Q6's "one lifetime" label; revision 6 stripping processor assignments (Matt ruled: keep them) |
| 7, a paced pass, final sweeps, e2e x3, a 14-minute run | Events lines shifting when read; an e2e flake (a leftover scroll) |

## Sweeps (all against `vite preview`)

Seeds 31-33, 41, 42-44, 51-52, 53 (2560), 54 (1280x720), 55 (3x), 61-62, 63-64, 65-66,
81-83, 84-85 (run 2 600 days): every finding is in the findings list or was load (a 3 s
frame gap with a test run and the e2e beside it; clean alone). Two runs at 200x (12 and 10
minutes, 88 runs): heap flat at about 2 MB, DOM flat, save 25 KB, no frame over 117 ms.

## Revision 6 and the balance judge

`hh-sim balance` on revision 5 (this branch before the change) and revision 6, same cells:
21 misses each. Revision 6 moves four: the author's M13 no longer reached (was 1 of 2 seeds
in run 17), M7 no longer early (was run 2), M5 now at run 4, run 1 lasts 1615 ticks (was
1520) and 24.3 real minutes (was 22.8). The judge's misses are otherwise as on main. Matt's
second ruling (assignments kept, a short day runs fewer) left every pinned game and the
judge's read unchanged: no scripted or sim player leaves a Data Center unpaid.
