# web: the Helium Hustle client

The playable browser game on `hh-wasm`: Vite + TypeScript + Preact. It renders the
engine's view and sends actions; every price, rate, cap, unlock and "affordable" comes from
the engine ([../docs/architecture.md](../docs/architecture.md), *The view*). Desktop, 1280
to 2560 px wide, in the Earthrise theme (light or dark, *Theme* below).

## Build and run

Needs the root README's *Toolchain*.

```sh
cd web
npm ci                                       # once per worktree (~10 s)
npm run dev                                  # builds the wasm (~45 s cold), then serves http://localhost:5173/
npm run game -- --port 5320 --strictPort     # conductor's Run game: npm install, wasm, vite, each skipped if unchanged (~1 s)
npm run wasm                                 # only rebuild web/src/wasm/ from crates/hh-wasm (~5 s warm, 0 if unchanged)
npm run check                                # typecheck (needs web/src/wasm/: npm run wasm first)
npm run build                                # wasm + typecheck + static site in web/dist/
npm run preview                              # serve web/dist/
```

- **Under a launcher that kills by pid** (conductor's `start`): `npm`/`npx` leave the server
  as a grandchild a tree kill can miss: start `node node_modules/vite/bin/vite.js
  [preview] --port <p> --strictPort`, or kill by port (`netstat -ano`, `taskkill //F //PID`).
- **The wasm build:** `scripts/build-wasm.mjs` runs `wasm-pack build crates/hh-wasm
  --target web --release` into `web/src/wasm/` (git-ignored); `HH_WASM_PROFILE=dev` builds
  unoptimized. It skips the build when a hash of its inputs (`crates/hh-core`,
  `crates/hh-wasm`, `content/`, `scenarios/`, `Cargo.*`, the script) matches `web/src/wasm/.inputs`; `HH_WASM_FORCE=1`
  builds anyway.
- **While the dev server runs**, editing `crates/hh-core`, `crates/hh-wasm`, `content/` or
  `scenarios/` rebuilds the wasm and reloads the page; a `configs/baseline.json` edit reloads it. A failed rebuild keeps the last good
  engine; the error is in the dev server's terminal.

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
2. **Every panel**: cheats reach Research, Projects and Ideologies; every nav tile opens; the
   cheated mark, a jump ahead and the replay check.
3. A program's frame at 10x and 50x, each speed's days per second, pause, click-to-buy, a reload keeping the save, a save from another
   revision discarded, the scenarios and the replay check; in `late_unlocks`, an `if` /
   `build` / `wait until` program written through the editor, which then builds.
4. **The theme:** the toggles switch it with every element's box unchanged (to 0.01 px);
   a reload keeps it.
5. **Readability** (before 4): capped rates, Resources' groups, affordable cards and Land,
   a pad short of fuel then Ready, Story.

Any uncaught page error fails the run. Run it against `vite preview` (CLAUDE.md).

- **Under load it flakes:** section 3's 10x pointer walk (`programOrder`) fails when a
  cargo build runs beside the browser (late timer wakes run two days a redraw). Rerun it
  alone.
- **The late UI for screenshots:** `experiments/results/027_art_in_game/shots.mjs` reaches
  it from scenario `pad_loading` with cheats (grants, buys, jumps; ~20 s). Cheat grants
  don't count toward `resource_cumulative` quests: build producers and jump instead.
- Edge needs an **absolute** `--user-data-dir`: given a relative one it hands the page to
  the Edge already running and exits (the script resolves it).
- On leaving a page the browser fires `pagehide`, then `visibilitychange`; the page saves
  on both, so a test that edits the save must do it after both.

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
Real mouse and key probes of click-to-buy and the program frame. The checks by kind and
the `HH_*` settings (`HH_PACE`, `HH_LONG` among them) are in the script's header. Exit 1
on any finding.

```sh
npm run build && npx vite preview --port 0          # a static build: dev-server reloads break a pass
HH_URL=http://localhost:<port>/ npm run qa           # required, as for e2e; ~1.3 min a pass, ~8 min in all
```

Output (`HH_QA_OUT`, default `out/qa`, wiped): a PNG per moment in
`<out>/<seed>-<width>-<theme>/`, a contact sheet per pass, and `report.json`.

## The screen

Three columns and a status bar:

- **Left:** the title, the nav grid (tiles from `content/nav.json`; a dot marks a tile new
  this run or whose panel holds something new; past nine tiles they go compact, icon beside
  name; a green dot on Launch Pads while a pad is ready), the speed buttons, Resources
  (Energy, Credits, Land, Boredom, Processors, then a second group of the rest; amount /
  cap, change per day, at its cap the surplus thrown away, muted, or green for Energy's
  spare power: "+9 CAP"; processors assigned / total; a click on a row opens its breakdown
  in place, one at a time: last day's sources and sinks, who went short), each ideology
  axis's rank once Ideologies is unlocked (folded at first in a window under 800 px high),
  and the lunar horizon with the habitat at the foot.
- **Middle:** the sky with the Earth, the main quest's text when it opens (the only story
  banner), then the active panel. Buildings: the Land bar (green and click-to-buy like a card
  when affordable), a collapsible section per category, cards (`Buildings.tsx`: picture,
  effects, a status row always kept, price with shortfalls in red, −/+ and Sell). A click on an affordable card (outlined in green, its name row tinted), off its
  controls, builds one (Shift: up to ten; ignored for 0.5 s after the summary or a story
  text closes); any other card says why it can't. Research cards work the same way. Pad
  cards tint their head: green Ready, yellow Paused, red full but short of fuel. Stats ends
  on every resource's breakdown, last day and rolling average.
- **Right:** the main quest in its own slot at the top, which nothing else takes; program
  tabs 1 to 5 with processors −/+ and Reset; a line per instruction, dragged to reorder
  (an `if` takes its body), tinted by today's outcome (a failed line says why: "needs ⚡2,
  had 0"), the next line to run in an accent frame. Once unlocked, Add: If, Wait until,
  Build, Buy Land (`ProgramLogic.tsx`): conditions as dropdowns, each clause with its live
  value and ✓/✗; body lines indented under the `if` (⇥/⇤), greyed with it when false.
  Above 10x a redraw covers several days: each line shows its outcome over them
  (`advance_run`'s `lines`), no frame. Below, Events: Story (side story and the latest
  boredom phase), completed quests, and Recent Activity (green good, red bad, blue news),
  each with an unread count, cleared once open and on screen for about a second.
- **Bottom:** day and run, the cheated mark, Boredom and Energy bars with their rates
  (capped as in Resources; "cap tomorrow" the day before a forced retirement).
- **Run summary:** when a run ends a summary covers the screen, the clock stopped until
  Continue.

Rates are per day; numbers past 10,000 use K/M/B/T. **"New" markers** are the UI's own, not
state: what is visible when a run starts (or the page loads) is the baseline; anything
revealed later in the run gets an accent edge (cards, rows) or dot (nav tiles) until
hovered or opened.

## Theme

Earthrise, light (the default) or dark, picked in Options and in the debug menu and
remembered in `localStorage` `hh.theme` (`index.html` applies it before the first paint).
The two share markup, layout and art; only the color tokens atop `src/style.css` differ,
so switching moves nothing (the e2e checks it). Chips (resources, ideology axes, rivals) keep their true
colors in both; an ideology axis drawn as text uses its `axis-<id>` class, which has a
readable variant per theme. Fonts are bundled (`@fontsource`, named in `src/style.css`).
Building pictures: `src/assets/buildings/<id>.webp` (one per building, both themes; 192 px
shown at 96 px; `src/ui/buildingArt.ts`; made by `experiments/results/020_building_card_art/`).
Chips, nav icons, horizon: `src/ui/icons.tsx` over `src/assets/{chips,nav,horizon}/`
(`experiments/results/027_art_in_game/`). The sky (`src/ui/art.tsx`, `.sky`) is the home
for a later live base view: replace what it draws, keep its box.

## Using it

- **Speed:** Pause, 1x, 3x, 10x, 50x, 200x; at 1x one day per second. Each timer wake
  (every 100 ms) runs the ticks owed since the last, so a throttled background tab
  catches up (stopping at a forced retirement); a closed page is paused. Keys: space
  pauses or resumes, 1 to 5 pick 1x to 200x; a card reached with Tab buys on Enter.
- **Actions** apply between ticks, stamped with the current tick; an illegal one shows the
  engine's reason in the notice line.
- **Autosave:** `localStorage` `hh.save`, at most once a second while ticking, on every
  action and speed change, and when the page is hidden or closed: the state, the game's
  record (seed, scenario, stamped actions) and the speed, tagged with the rules revision
  and config hash. A save from another revision or config, or one the engine can't read,
  is discarded with a notice and a new game starts.
- **One tab at a time:** a page claims the save when it loads; a tab already playing it
  sees the save change (`storage` event), stops its clock and says to reload it.
- **Debug menu** (Options → Debug): the theme, a scenario (`scenarios/`) or a new game from
  a seed; *Check replay* replays the record in wasm and compares fingerprints; a line with
  engine, revision, config hash, seed, action count and fingerprint. **Cheats** (off by
  default, remembered in `hh.cheats`): grant a resource, set boredom, jump ahead N days.
  They are engine actions, so replay stays exact, and any cheat marks the game CHEATED.

## Play logs

Every career is recorded as played, one JSON-lines file per career, so a human's play can
be replayed exactly and set against the sim players (`hh-sim playlog`, `hh-sim vs-human`,
[../docs/simulator.md](../docs/simulator.md)).

- **Where:** the dev server (and `vite preview`) takes the page's lines at `POST
  /__playlog` (`vite.config.ts`) and appends them to `<dir>/<career>.jsonl`: `dir` is
  `HH_PLAYLOG_DIR`, else `C:\Code\gdrive-sync\helium-hustle\playlogs\` when that folder's
  parent exists, else `out/playlogs/`. A browser under automation (`navigator.webdriver`)
  gets a `bot_` career, kept out of the synced folder. The page sends every 2 s, when
  hidden, and in a beacon when closed. A static build with no server records nothing
  (Options says so). A server that can't write answers 500: the page keeps the lines and
  retries, and Options says it is NOT recording; a malformed batch (400) is dropped.
- **One file per career:** `<local start time>_<random tag>_seed-<seed>.jsonl`. A new game
  or a discarded save closes the old file with a `career_end` line and starts a new one.
- **Lines** (`src/playlog.ts` has the format): a header (format `hh-playlog/1`, career,
  revision, config hash and config, seed, scenario, `prior_actions`, and `commit`, HEAD
  with `+dirty`, stamped by the server), then one line per happening, each with a unique
  `id` (a resent line is read once) and a `kind`: `action` (the game's record),
  `checkpoint` (the fingerprint, every 10 s while playing, at retirements, when hidden or
  closed), `session_start`/`_end`, `hidden`, `visible`, `speed`, `rejected`, `refused`,
  `panel`, `program_tab`, `retirement`, `continue`, `story_read`, `new_game`, `debug`,
  `jump_end`, `career_end` (`reason`; `offline` when written at a later visit).

`hh-sim playlog <file>` replays the header's config, seed and scenario with the logged
actions to the last tick and checks the fingerprint logged there.

## Source

`src/store.ts` holds the game in play (actions, ticking, saves, "new" markers, the run
summary), `src/playlog.ts` the play log's client half (the queue, sends to `/__playlog`,
career ids), `src/ui/` the panels; each file's header says the rest. The config is
`configs/baseline.json`, imported at build time.
