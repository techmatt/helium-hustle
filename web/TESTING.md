# web: browser checks

The client's checks: the e2e browser check, the unit checks and the QA checker. Building
and serving the page: [README.md](README.md). e2e and qa share the DevTools harness
`scripts/cdp.mjs`, whose `driver()` holds the page helpers both use (`panel`, `runOnly`,
`pickCommand`, the debug menu): a UI change a helper reads is fixed there once.

## Browser check

`npm run e2e` (`scripts/e2e.mjs`) drives headless Edge or Chrome over the DevTools
protocol (no packages; the harness, shared with qa, is `scripts/cdp.mjs`) against a
served page: `HH_URL`, required (it exits naming `vite preview` when unset),
`HH_BROWSER` for a browser off the usual paths, `HH_SHOTS=<dir>` to save PNGs. About
45 s. It plays, in order:

1. **A fresh game through the normal UI only** (no debug menu): a build order, Program 1
   switched between Sell Cloud Compute and the bridging buys, the first pad loaded and
   launched (Q1 to Q3, around day 400 at 50x); retirement, the run summary holding the
   clock, run 2 (known tiles and resources still shown, not cheated).
2. **Every panel**: cheats reach Research, Projects and Ideologies; every nav tile shown opens, and every Act 1 tile is
   shown (all of `content/nav.json` but panels only a DRAFT quest opens); the
   cheated mark, a jump ahead and the replay check.
3. A program's frame at 10x and 50x, each speed's days per second, pause, click-to-buy, a
   reload keeping the save, another revision's save discarded, the scenarios; in `late_unlocks`, an `if` /
   `build` / `wait until` program written through the editor, which then builds; saved,
   then loaded after a retirement, it runs in run 2; pause/resume verb lines under a
   `reading ×` clause and `every N days` run.
4. **The theme:** the toggles switch it with every element's box unchanged (to 0.01 px);
   a reload keeps it.
5. **Readability** (before 4): capped rates, Resources' groups, affordable cards and Land,
   a pad short of fuel then Ready, Story.

Any uncaught page error fails the run. Run it against `vite preview` (CLAUDE.md).

- **Under load it flakes:** section 3's 10x pointer walk fails beside a cargo build.
- **The late UI for screenshots:** `experiments/results/027_art_in_game/shots.mjs`
  (`pad_loading`, cheats, ~20 s); grants don't count toward `resource_cumulative`.
- Edge needs an **absolute** `--user-data-dir` (the script resolves it).
- Leaving a page fires `pagehide`, then `visibilitychange`; the page saves on both, so a
  test editing the save does it after both.

`npm run unit` checks the number formatting (`src/format.ts`), then `scripts/contrast.mjs`
the text-on-background pairs in its hand-kept `PAIRS` list, in both themes, at
4.5:1 (7:1 for `pos-text`, the Resources rates' green): a new token needs its pairs there.

## QA checker

`npm run qa` (`scripts/qa.mjs`) plays the game as a player would, over seeds, widths and
both themes, and audits the screen at each meaningful moment (a panel, a story text, a new
quest, a launch, the summary, a reload). Hunts' findings: `docs/qa-findings.md`. Per pass
(a seed at a width), in a fresh headless browser: a first visit, a new game on the seed,
run 1 at 3x then 50x with every panel, retirement and the summary (keys must not start the
clock), run 2 at 200x, a reload, the replay check, the play log replayed by `hh-sim
playlog` (a release build, or `HH_SIM`: a fresh worktree has none, so build it first), a second tab and site data blocked (first pass only). At run 1's end and in every
debug scenario (each played, its panels audited): every Resources row's breakdown opened
and closed, Stats' breakdowns; program lines dragged; in `late_unlocks`, a loaded
pad switched to another good (the stockpile gains what its tooltip promised) and Recent
Activity's unread count (counts while folded, clears once read). The late panels again in
the other theme; every building's card (`?all-cards`, a display-only URL flag) in both.
Real mouse and key probes of click-to-buy and the program frame. Checks and `HH_*`
settings: the script's header. Exit 1 on any finding.

```sh
npm run build && npx vite preview --port 0          # a static build: dev-server reloads break a pass
HH_URL=http://localhost:<port>/ npm run qa           # required, as for e2e; ~1.3 min a pass, ~8 min in all
```

Output (`HH_QA_OUT`, default `out/qa`, wiped): a PNG per moment in
`<out>/<seed>-<width>-<theme>/`, a contact sheet per pass, and `report.json`.
