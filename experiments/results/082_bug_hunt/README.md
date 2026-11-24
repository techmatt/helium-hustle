# 082: bug hunt (recent changes since 066, b519661)

The findings and their fixes are rows 95-110 and the new deferred and won't-fix rows of
[docs/qa-findings.md](../../../docs/qa-findings.md); the checker's new probes are in
[web/TESTING.md](../../../web/TESTING.md), *QA checker* and section 10 of *Browser check*.

The qa checker had not finished a pass since 071 (the Programs tile became a takeover,
which its panel helper waited on), so 071-080 went unswept until this hunt: most of round
2's findings date from those items.

## Rounds

| Round | Lens | Found (user-facing / polish) |
|---|---|---|
| 1 | Story-modal chains under real input (probe scripts), plus a four-area code read of 067-081 with a skeptic per area | 7 / 0: pages skipped by a double-click or a held Enter, Esc closing the Programs view behind a modal, the modal's button without the keyboard, a double-click replaying a Story beat, a 200x wake playing past its transition, number-box Esc dropping the typed count, a retyped number restarting the program |
| 2 | The qa sweep, repaired, over every seed, width and theme | 3 / 2: the Blocks toggle at 1.66:1 in dark, "Achievements" past its 1920 nav tile, cut program-line names with no tooltip; polish: faked bold (four places), an achievement star at 3.82:1; plus six checker false findings |
| 3 | Contact sheets, short and narrow windows, a real-speed (3x) pass | 1 / 1: the Programs view's boredom figures cut to "285 /" at 1280, the boot "Loading…" line left under the app |
| 4 | Odd input around modals and the summary (resize, a frozen tab, reload with the summary up), the full sweep on the final build | 0 / 0 (the sweep caught this hunt's own strip change clipping Resume; fixed) |
| 5 | Odd input in the Programs view (typing at 200x, Enter, Tab, a program tab or Text clicked mid-typing) | 0 / 0 |

## Screenshots (also in the shared screenshots folder as `082_*.png`)

- `story_modal_after_double_click.png`: the new game's modal after qa's real double-click
  on Continue: page 2 (Begin), not the game running.
- `late_nav_and_achievements_1920.png`: a late career at 1920 (dark): the compact nav two
  tiles a row, "Achievements" inside its tile; the earned medals.
- `programs_strip_1280.png`: the Programs view's status strip at 1280 in a late career
  (qa's `late-programs`): the boredom bar holding its figures, Resume wrapped to a second
  line rather than over Close.
