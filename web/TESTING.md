# web: browser checks

The client's checks: the e2e browser check, the unit checks and the QA checker. Building
and serving the page: [README.md](README.md). e2e and qa share the DevTools harness
`scripts/cdp.mjs`, whose `driver()` holds the page helpers both use (`panel`, `runOnly`,
`pickCommand`, the debug menu): a UI change a helper reads is fixed there once.

## Browser check

`npm run e2e` (`scripts/e2e.mjs`) drives headless Edge or Chrome over the DevTools
protocol (no packages; the harness, shared with qa, is `scripts/cdp.mjs`) against a
served page: `HH_URL`, required (it exits naming `vite preview` when unset),
`HH_BROWSER` for a browser off the usual paths, `HH_SHOTS=<dir>` to save PNGs (an item's
results folder; kept shots are also copied to the shared screenshots folder, CLAUDE.md). About
100 s. `HH_ONLY=<n>[,<n>...]` runs only those sections (what a skipped one leaves behind is set
up directly). It plays, in order:

1. **A fresh game through the normal UI only** (no debug menu): only Solar Panel and Data
   Center offered, only 1.1's story modal (paused, its task on the last page; 1.1's picture
   above its paragraph, at the left column's foot and as Story's active-beat thumbnail, the
   split-glow title, no globe); then the
   opening at 50x, Program 1 switched between Sell Cloud Compute, Buy Science and the
   bridging buys: Basic Blueprints (1.2), the first pad offering Titanium only (1.3), 20+
   titanium loaded and launched by the card's Launch, Advanced Blueprints and Refineries,
   a full pad of Helium-3 (1.4). Each quest change must open a story modal chain that holds
   the clock and ends on the next task; Launch Full Pads never shows. No Retirement tile
   (it opens at 1.7 Burnout), so on at 200x (story modals now clicked through as they
   open, `autoStory`) to the forced retirement at the boredom cap; the run summary
   holding the clock, run 2 (known tiles and resources still shown, not cheated). Its double-click-on-Continue check failed once (058: a buy hint under
   it, nothing bought) and passed on a rerun: a timing flake, not yet fixed.
2. **Every panel**: scenario `late_unlocks` opens Research, Projects and Ideologies (1.8 Lab
   is past a second run); a locked project's condition, a funded one's days to done; every
   nav tile shown opens, and every Act 1 tile is shown (all of `content/nav.json` but panels
   only a DRAFT quest opens); no CHEATED tag in the bottom bar (the mark in the debug info), a jump ahead, the cheat building (+1000 credits a day, 1 credit,
   no card with Cheats off) and Complete all research, and the replay check.
3. A program's frame at 10x and 50x, each speed's days per second, pause, click-to-buy, a
   reload keeping the save, another revision's save discarded, the scenarios; in `late_unlocks`, an `if` /
   `build` program written through the editor (`>` / `<` / `>=` / `<=`, a fill's % / amt toggle, and
   Energy per day and Days since a purchase offered once Process Telemetry is researched;
   `programLogic`), whose guarded build waits for its `if`; saved, then loaded after a retirement, it runs in run 2;
   pause/resume verb lines under `if`s run.
4. **The theme:** the toggles switch it with every element's box unchanged (to 0.01 px);
   a reload keeps it.
5. **Programs, Text mode** (before 6, after 3), from `everything`: 060's L4 Program 1
   pasted with a junk line (red, its reason) and a locked building (amber), Apply, green
   and grey dots and an `if`'s fired count; the block editor keeps every line (unparsed
   and locked rows); the text round-trips; a tab switch saves; 130 lines are red past 128,
   refused, and kept for the next open after a close; a loadout of it loads after
   a retirement with every line.
   Then `programElse`: an if/else typed there shows the same lines in the block view
   (an Else row, both bodies indented) and back, and at 1x each day's pass runs the
   branch its `if` picks. Then `instructionPanel`: on a new game the All instructions
   panel lists only usable entries (Commands and `#`, nothing locked); in `everything`
   clicks write an `if … > 0` and a body line under it (caret there), `else` is offered
   only below an `if` or its body, command and building hover cards show, it Applies with
   no red or amber, the block view shows `if` and its body, and the `if` runs at 1x. Then
   `programsBlocks`: in `everything` the Programs tile opens Programs in Blocks, an if/else and a `4x` line are built by
   panel clicks, ⇤ and the count field only, no help or hint line shows, a real click in the
   count box selects its text (typing replaces it), a drag inside it starts no row drag
   and Esc while typing in it keeps the number (`numberBox`), Text shows the same lines, and at 1x the
   `4x` line's four dots fill 1/4 to 4/4; set to 12, it shows a bar titled `k/12`. Sections open Text through the nav tile
   (`openText`); the mode is remembered.
   Then `storyView`: a new game played through 1.6 by the autopilot's *Next quest step*
   (the step from 1.5 passes 1.6 too, stopping at 1.7); Story shows the Act 1 band alone,
   painted, the finished beats lit (each with its picture's thumbnail; the active
   beat's thumbnail and the left column's foot on the active quest's picture), the active one glowing (in view) with its task, one
   "???" after it and no achievements; a lit beat's click replays its modal; the
   Achievements tile shows every badge by category, each with its reward; then scenario
   `late_unlocks` with *Complete all research*: 1.b's completion banner shows its picture.
6. **Readability** (before 4): capped rates, Resources' groups, affordable cards and Land,
   a pad starting with No cargo (status, red nav dot) given Helium-3, short of fuel then
   Ready, Story.

Any uncaught page error fails the run. Run it against `vite preview` (CLAUDE.md).
`panel()` closes the debug menu: call `debugMenu()` again before a `grant()` after one.
A story modal refuses every action and speed until read: `readStory()` reads one through
(the driver's), `autoStory()` clicks every later one away until the page is left.

- **Under load it flakes:** section 3's 10x pointer walk fails beside a cargo build, and
  so does its "guarded line built" outcome check (066: beside `cargo test-all`; 067: once with
  nothing beside it, then passed on a rerun). Its "if held" check reads only the if's check
  mark since 070 (qa-findings #94).
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
(a seed at a width), in a fresh headless browser: a first visit, a new game on the seed (its
story modal under real keys and mouse: the clock held, space and 3 ignored, its button
focused, Esc leaving the Programs view behind it open, a double-click on Continue turning
one page), run 1 at 3x then 50x with every panel and the Programs view (each chain's
modal at most a day past its "Quest complete" line; on the first, a held Enter turns
one page), retirement and the summary (keys must not start the
clock), run 2 at 200x, a reload, the replay check, the play log replayed by `hh-sim
playlog` (a release build, or `HH_SIM`: a fresh worktree has none, so build it first), a second tab and site data blocked (first pass only). At run 1's end and in every
debug scenario (each played, its pads given Helium-3 first, or Titanium while 1.3 allows only that, its panels audited): every Resources row's breakdown opened
and closed, Stats' breakdowns; program lines dragged; in `late_unlocks`, a loaded
pad switched to another good (the stockpile gains what its tooltip promised) and Recent
Activity's unread count (counts while folded, clears once read); in `everything`, the
takeovers (a program in every form the text grammar writes, pasted in Programs' Text mode:
each line reads, Blocks holds every line, the text reads back unchanged; the
Cortex mock by its toggle, closed with Esc) and every open project's Max then − steps,
each taken. A career short of 1.8 (Projects) is played on by the debug autopilot's
*Next quest step*, each step timed and checked to move the quest; after the scenarios its
*Everything* plays to the ending (~15 s) and every later-act panel is audited. Building cards with no
picture drawn yet (placeholder art) are not findings. The late panels again in
the other theme; every building's card (`?all-cards`, a display-only URL flag) in both.
Real mouse and key probes of click-to-buy and the program frame. Checks and `HH_*`
settings: the script's header. Exit 1 on any finding.

```sh
npm run build && npx vite preview --port 0          # a static build: dev-server reloads break a pass
HH_URL=http://localhost:<port>/ npm run qa           # required, as for e2e; ~1.3 min a pass, ~8 min in all
```

Output (`HH_QA_OUT`, default `out/qa`, wiped): a PNG per moment in
`<out>/<seed>-<width>-<theme>/`, a contact sheet per pass, and `report.json`.
