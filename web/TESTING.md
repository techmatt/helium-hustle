# web: browser checks

The client's checks: the e2e browser check, the unit checks and the QA checker. Building
and serving the page: [README.md](README.md). e2e and qa share the DevTools harness
`scripts/cdp.mjs`, whose `driver()` holds the page helpers both use (`panel`, `runOnly`,
`pickCommand`, the debug menu): a UI change a helper reads is fixed there once.

**When to run them** (light testing per item, CLAUDE.md): a `web/` change runs `npm test`,
the web fast lane (`npm run check` then `npm run unit`, ~1.5 s). e2e (~5 min) and qa
(~20 min) run only in a bug hunt or when the prompt asks; an item that changes a helper
they read still fixes it, unrun. One browser at a time on this machine: several exhaust
its memory.

## Browser check

`npm run e2e` (`scripts/e2e.mjs`) drives headless Edge or Chrome over the DevTools
protocol (no packages; the harness, shared with qa, is `scripts/cdp.mjs`) against a
served page: `HH_URL`, required (it exits naming `vite preview` when unset),
`HH_BROWSER` for a browser off the usual paths, `HH_SHOTS=<dir>` to save PNGs (an item's
results folder; kept shots are also copied to the shared screenshots folder, CLAUDE.md). About
5 min: Act 1 plays at 10x, its fastest, so run 1 to the boredom cap takes ~3.
`HH_ONLY=<n>[,<n>...]` runs only those sections (what a skipped one leaves behind is set up
directly). **What each section checks is `scripts/e2e.mjs`'s header**; in short, run in the
order 1, 2, 3, 6 to 11, 5, 4:

1. A fresh game through the normal UI only, the opening through 1.3 to the forced
   retirement, the run summary and run 2.
2. Every panel (`late_unlocks`), the cheats and the replay check.
3. Speeds, click-to-buy, a reload keeping the save, another revision's save discarded.
4. The theme: the toggles move no element's box; a reload keeps it.
5. Readability: capped rates, Resources' groups, affordable cards, a pad's states, Story.
6. Program instructions (`programLogic`): an `if` / `build` program through the editor.
7. Programs, Text mode: a pasted program with red and amber lines, round-tripped.
8. `else` (`programElse`): text and blocks agree, each pass runs its branch.
9. The All instructions panel (`instructionPanel`).
10. The Programs view's Blocks mode (`programsBlocks`).
11. Story and Achievements (`storyView`), played by the autopilot to 1.5 A Voice.

Any uncaught page error fails the run. Run it against `vite preview` (CLAUDE.md).
`panel()` closes the debug menu: call `debugMenu()` again before a `grant()` after one.
A story modal refuses every action and speed until read: `readStory()` reads one through
(the driver's), `autoStory()` clicks every later one away until the page is left.

- **Under load it flakes:** section 3's 10x pointer walk fails beside a cargo build, and
  so does section 6's "guarded line built" outcome check (once even with nothing beside it,
  then passing on a rerun). Section 1's double-click-on-Continue check has flaked once too.
- **The driver's `speed()` repeats its click** (up to a second) until that speed shows
  pressed: a popup opening as a pause lands refuses it, and `autoStory` reading it resumes
  the speed before it.
- **The late UI for screenshots:** `experiments/results/027_art_in_game/shots.mjs`
  (`pad_loading`, cheats, ~20 s); grants don't count toward `resource_cumulative`.
- **Speculators' first arrival:** scenario `speculators_due` (no other scenario has pads that
  load with the speculators unseen; He-3 caps at 20 and propellant at 30 without depots): pick
  He-3 on both pads, launch by hand, grant He-3 and propellant as they run low (116's `check.mjs`).
- **An item's check script** (`experiments/results/<item>/check.mjs`, 116's and 119's are
  short) uses `cdp.mjs`'s `launch` and `driver`. Start from a scenario's numbers: cheat
  grants refuse negative amounts, so nothing can be lowered. The Programs view is an
  overlay, not a `data-panel`: `.nav-tile[data-nav="programs"]`, wait for
  `[data-testid="mode-text"]`, Escape closes it. One-offs fire only under real play (the
  autopilot plays past popups): 112's `check_prop.mjs` takes two autopilot quest steps, then
  launches by hand with grants and 4-day jumps.
- Edge needs an **absolute** `--user-data-dir` (the script resolves it).
- Leaving a page fires `pagehide`, then `visibilitychange`; the page saves on both, so a
  test editing the save does it after both, or edits `localStorage` then sets
  `Storage.prototype.setItem = () => {}` before reloading (104's and 117's `check.mjs`).

`npm run unit` (Node, no page) checks `src/format.ts` (numbers, `delta`, `commandTip`,
`fillRate`: an event effect's `{rate}`), `programEdit.ts`'s line edits, `story.ts`'s popups
(told once a run, a seen one hidden, nothing seen under the autopilot or at a load), the
nav slots, the offered speeds, that every quest `picture` has a file, then `scripts/contrast.mjs`
the text-on-background pairs in its hand-kept `PAIRS` list, in both themes, at
4.5:1 (7:1 for `pos-text`, the Resources rates' green): a new token needs its pairs there.

## QA checker

`npm run qa` (`scripts/qa.mjs`) plays the game as a player would, over seeds, widths and
both themes, and audits the screen at each meaningful moment (a panel, a story text, a new
quest, a launch, the summary, a reload). Hunts' findings: `docs/qa-findings.md`. **Its
checks and `HH_*` settings are the script's header.** A pass is one seed's career, in a
fresh headless browser, played at one width; every moment is also audited and shot at the
other widths, in the theme on screen (a resize, then back: findings there say `at <width>`).
The played width and the theme cycle over seeds: 11 at 1280 light, 12 at 1920 light, 13 at
1920 dark, so the mouse and key probes meet both widths and each width meets both themes.
Per pass:

- **Run 1:** a first visit, a new game on the seed (its story modal probed under real keys
  and mouse), run 1 at 3x then 10x played from the right column's program with every panel,
  retirement and the summary. Programming is bought in run 1 with Buy Science if run 1
  reaches 1.5 A Voice, else in run 2 once the debug autopilot's *Next quest step* reaches it;
  then the Programs view and the right column's program edited.
- **Run 2** at 10x, a reload, the replay check, the play log replayed by `hh-sim playlog` (a
  release build, or `HH_SIM`: a fresh worktree has none, so build it first), and on the
  first pass only a second tab and site data blocked.
- **At run 1's end and in every debug scenario** (each played, its panels audited): every
  breakdown opened; scenario-specific probes (`late_unlocks`: a pad's good switched, Recent
  Activity's unread count, an Events popup; `everything`: the takeovers, a program in every
  text-grammar form, project funding steps under a held +). A career short of 1.5 is played
  on by *Next quest step*; after the scenarios *Everything* plays to the ending (~15 s) and
  every later-act panel is audited. The late panels again in the other theme; every
  building's card (`?all-cards`) in both. Cards with placeholder art are not findings.

Exit 1 on any finding, but known won't-fix slow frames (`docs/qa-findings.md`) print tagged
`[known: won't fix]`. A pass that never reaches the ending is an `ending` finding (NEVER
REACHED THE ENDING; `report.json` keeps `ended` per pass), and `scripts/warning_checks.py`
warns (kind `qa`) while the tree's last report has one.

```sh
npm run build && npx vite preview --port 0          # a static build: dev-server reloads break a pass
HH_URL=http://localhost:<port>/ npm run qa           # required, as for e2e; ~6.5 min a pass, ~20 min in all (run 1 at 10x)
HH_SEEDS=11 HH_WIDTHS=1280 npm run qa                # one pass at one width: ~6 min
```

Progress lines carry the pass's elapsed time (m:ss) at each quest and phase; its summary
line, the time spent in captures (at the played width and at the others). Where a pass's
time goes: run 1 ~2:55, run 2 and the late panels ~0:50, the debug scenarios ~1:30,
*Everything* with the second tab and blocked storage ~1:00, every card ~0:20; inside
those, captures take ~30 s at the played width and ~45 s at the other.
A theme switched by setting `<html>`'s `data-theme` alone is not the theme: the page sets
some colors from its theme listeners (`theme.ts`), so the contrast audit reads a mix.

Output (`HH_QA_OUT`, default `out/qa`, wiped): a PNG per moment in
`<out>/<seed>-<width>-<theme>/`, a contact sheet per seed and width, and `report.json`
(a pass's other widths under `also`, its `seconds`).
