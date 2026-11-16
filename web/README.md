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
npm run wasm                                 # only rebuild web/src/wasm/ from crates/hh-wasm (~13 s warm, 0 if unchanged)
npm run check                                # typecheck (needs web/src/wasm/: npm run wasm first)
npm run build                                # wasm + typecheck + static site in web/dist/
npm run preview                              # serve web/dist/
```

- **Under a launcher that kills by pid** (conductor's `start`): `npm` and `npx` start the
  server as a grandchild, so killing the started pid's tree can leave it listening. Run
  `npm run wasm`, then start `node node_modules/vite/bin/vite.js --port <p> --strictPort`,
  or kill by the port (`netstat -ano | grep :<p>`, then `taskkill //F //PID <pid>`).
- **The wasm build:** `scripts/build-wasm.mjs` runs `wasm-pack build crates/hh-wasm
  --target web --release` into `web/src/wasm/` (git-ignored); `HH_WASM_PROFILE=dev` builds
  unoptimized. It skips the build when a hash of its inputs (`crates/`, `content/`,
  `scenarios/`, `Cargo.*`, the script) matches `web/src/wasm/.inputs`; `HH_WASM_FORCE=1`
  builds anyway.
- **While the dev server runs**, editing `crates/`, `content/` or `scenarios/` rebuilds the
  wasm and reloads the page; a config edit reloads it. A failed rebuild keeps the last good
  engine; the error is in the dev server's terminal.

## Browser check

`npm run e2e` (`scripts/e2e.mjs`) drives headless Edge or Chrome over the DevTools
protocol (no packages) against a served page: `HH_URL` (default
`http://localhost:5173/`), `HH_BROWSER` for a browser off the usual paths, `HH_SHOTS=<dir>`
to save a PNG at each step worth seeing. About 45 s. It plays, in order:

1. **A fresh game through the normal UI only** (no debug menu): an opening build order,
   Program 1 switched between Sell Cloud Compute and the bridging buys, the first pad
   loaded and launched (Boot Sequence, First Extraction and Proof of Concept, around day
   400 at 50x); retirement from its panel, the run summary holding the clock, run 2
   (known tiles and resources still shown, not marked as cheated).
2. **Every panel**: the debug menu's cheats reach Research, Projects and Ideologies; all 11
   nav tiles open; the cheated mark, a jump ahead and the replay check over the career.
3. Each speed's days per second, pause, click-to-buy (a green card's text buys, its −/+ and
   Sell 1 don't, a grey card buys nothing and says why), a reload keeping the save, a save
   from another revision discarded with its notice, the debug-menu scenarios and the
   replay check.
4. **The theme:** the game opens light; the Options and debug-menu toggles switch it with
   every element's box unchanged (to 0.01 px); Buildings and Commands lay out the same in
   both; a reload keeps it. Shots `theme-<light|dark>-<buildings|commands>.png`.

Any uncaught page error fails the run. Against a snapshot rather than the dev server:
`npm run build && npx vite preview --port 61873 --strictPort`, then
`HH_URL=http://localhost:61873/ npm run e2e`.

- Edge needs an **absolute** `--user-data-dir`: given a relative one it hands the page to
  the Edge already running and exits (the script resolves it).
- On leaving a page the browser fires `pagehide`, then `visibilitychange`; the page saves
  on both, so a test that edits the save must do it after both.

`npm run unit` checks the number formatting (`src/format.ts`). `node scripts/contrast.mjs`
checks every text-on-background pair the theme tokens make, in both themes, at 4.5:1.

## QA checker

`npm run qa` (`scripts/qa.mjs`) plays the game the way a player would, over several seeds
and window widths, and checks the screen at each meaningful moment (a panel opened, a story
text, a new quest, a launch, the run summary, run 2, a reload, the replay check). Findings
list `docs/qa-findings.md` holds what hunts found. Per pass (a seed at a width), in one
fresh headless browser: a first visit, a new game on the seed from the debug menu, run 1 at
3x then 50x with every panel opened, retirement and the summary (keys pressed during it
must not start the clock), run 2 at 200x, a reload, the replay check, the play log replayed
by `hh-sim playlog` (needs `target/release/hh-sim`), and, on the first pass, a second tab
(the first must stand down). The checks: **overflow** (clipped text without a tooltip
holding it, text past its box or the window, a page wider than the window), **hidden** (a
control's center covered), **text** (`undefined`, `NaN`, `[object …]`, `{name}`),
**assets**, **console** (page errors), **input** (a click slower than 300 ms to show),
**frame** (a gap over 250 ms), **memory** (heap growth over 40 MB, a save over 2 MB). Exit 1
on any finding.

```sh
npm run build && npx vite preview --port 0          # a static build: dev-server reloads break a pass
HH_URL=http://localhost:<port>/ npm run qa           # ~2.5 min a pass; 3 seeds x 2 widths ~15 min
HH_PACE=120 HH_WIDTHS=1600 HH_URL=... npm run qa     # a first visit left at 1x for 2 min, a shot every 15 s
```

`HH_SEEDS` (default `11,12,13`), `HH_WIDTHS` (`1280,1920`), `HH_RUN2` (run 2 days, 150),
`HH_SPEED` (run 1's speed, 50; `3` for a person's pace), `HH_QA_OUT` (default `out/qa`,
wiped): a PNG per moment in `<out>/<seed>-<width>/`, a contact sheet per pass, and
`report.json`. Run it against a static build: an edit under a dev server reloads the page.

## The screen

Three columns and a status bar, as in the last Godot build:

- **Left:** the title, the nav grid (tiles from `content/nav.json`; a dot marks a tile new
  this run or whose panel holds something new), the speed buttons, Resources (amount / cap,
  change per day; processors assigned / total), each ideology axis's rank once Ideologies
  is unlocked, and the lunar horizon with the habitat at the foot.
- **Middle:** the sky with the Earth on top, then the active panel. Buildings: the Land bar,
  a collapsible section per category, cards (a 96 px painted picture beside the
  description and per-day effects, ideology tag,
  stall, price with shortfalls in red, active/owned with −/+, Sell 1, Sell All). A click on
  an affordable card (outlined in green), off its controls, builds one (Shift: up to ten);
  any other card buys nothing and briefly says why. Then Commands, Launch Pads, Research,
  Projects, Ideologies, Adversaries, Stats, Story, Retirement and Options.
- **Right:** program tabs 1 to 5 with processors −/+ and Reset; a line per command with
  its progress, −/+/×, drag to reorder, its tint from today's outcome (ran, failed,
  skipped), and an accent frame (thick on the left, drawn so no text moves) on the line
  the next processor runs. Above 10x one redraw covers several days, so each line shows its
  last outcome over them (`advance_run`'s `lines`) with no frame. Below, Events: the
  active quest, completed quests, and Recent Activity (green good, red bad, blue news).
- **Bottom:** day and run, the cheated mark, Boredom and Energy bars with their rates.
- **Run summary:** when a run ends a summary covers the screen, the clock stopped until
  Continue.

Rates are per day; numbers past 10,000 use K/M/B/T. **"New" markers** are the UI's own, not
state: what is visible when a run starts (or the page loads) is the baseline; anything
revealed later in the run gets an accent edge (cards, rows) or dot (nav tiles) until
hovered or opened.

## Theme

Earthrise, light (the default) or dark, picked in Options and in the debug menu and
remembered in `localStorage` `hh.theme` (`index.html` applies it before the first paint).
The two are 1:1: the same markup, layout and art; only the color tokens at the top of
`src/style.css` differ (surfaces, lines, text, muted text, accent, positive/negative,
selection, bar fills, horizon fills). A token block holds colors only, so switching moves
nothing (the e2e checks it). Resource and nav swatches keep their true colors in both; an
ideology axis drawn as text uses its `axis-<id>` class, which has a readable variant per
theme. Fonts are bundled (`@fontsource`): Rajdhani for headings, IBM Plex Sans for body
text, IBM Plex Mono for numbers. The building pictures are painted cut-outs (one per
building, the same file in both themes): `src/assets/buildings/<id>.webp`, 192 px
squares shown at 96 px, looked up by `src/ui/buildingArt.ts` (a building with no file
shows none), drawn and cut by `experiments/results/020_building_card_art/`. The rest of
the art is inline SVG in `src/ui/art.tsx`; the sky over
the center column (`.sky`) is the home for a later live view of the base: replace what it
draws, keep its box.

## Using it

- **Speed:** Pause, 1x, 3x, 10x, 50x, 200x; at 1x one day per second. Each timer wake
  (every 100 ms; browsers throttle background tabs) runs the ticks owed for the wall time
  since the last, so a background tab catches up (stopping at a forced retirement).
  Closing the page pauses the game: no offline progress. Keys: space pauses or resumes, 1
  to 5 pick 1x to 200x.
- **Actions** apply between ticks, stamped with the current tick; an illegal one shows the
  engine's reason in the notice line.
- **Autosave:** `localStorage` `hh.save`, written at most once a second while ticking, on
  every action and speed change, and when the page is hidden or closed: the state, the
  game's record (seed, scenario, stamped actions) and the speed, tagged with the rules
  revision and config hash. A save from another revision or config, or one the engine
  can't read, is discarded with a notice and a new game starts (no migration code).
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
- **Header** (first line): `format` `hh-playlog/1`, `career`, `rules_revision`,
  `config_hash`, the resolved `config`, `seed`, `scenario`, `engine`, `prior_actions` (a
  log begun mid-career), and `commit` (HEAD, `+dirty` if uncommitted), stamped by the server.
- **Every other line:** `id` (unique; a resent line is read once), `t` (wall-clock ms),
  `kind`, `tick`, `run`, `day`, `speed`, and by kind: `session_start` (how: save, new,
  discarded), `session_end`, `hidden`, `visible`, `speed` (`from`), `action` (applied at
  `tick`: the game's record), `rejected` (the engine's `error`), `refused` (the run ended
  first), `panel`, `program_tab`, `retirement` (`ended`, `forced`, `days`), `continue`,
  `story_read`, `new_game`, `debug` (cheats, replay checks), `checkpoint` (`fingerprint`,
  every 10 s while playing, at retirements, when hidden or closed), `jump_end` (after a
  jump ahead) and `career_end` (`reason`; `offline` when written at a later visit).

`hh-sim playlog <file>` replays the header's config, seed and scenario with the logged
actions to the last tick and checks the fingerprint logged there.

## Source

`src/engine.ts` (typed wasm wrapper, the view's types), `src/clock.ts` (wall clock,
speeds), `src/save.ts` (autosave), `src/store.ts` (the game in play: actions, ticking,
saves, "new" markers, the run summary), `src/format.ts` (number display), `src/theme.ts`
(the theme), `src/ui/` (`App.tsx` layout and status bar, `Left.tsx`, `Buildings.tsx`,
`panels.tsx`, `Options.tsx`, `Right.tsx`, `common.tsx`, `art.tsx`, `buildingArt.ts`),
`src/assets/` (images), `src/main.tsx` (entry).
The config is `configs/baseline.json`, imported at build time.
