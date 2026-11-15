# web: the Helium Hustle client

The playable browser game on `hh-wasm`: Vite + TypeScript + Preact. It renders the
engine's view and sends actions; every price, rate, cap, unlock and "affordable" comes from
the engine ([../docs/architecture.md](../docs/architecture.md), *The view*). Light mode,
desktop 1280 to 2560 px wide.

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

- **Under a launcher that kills by pid** (conductor's `start`, a background job): `npm`
  and `npx` start the server as a grandchild, so killing the started pid's tree can leave
  it listening and the next start fails "port in use". Run `npm run wasm`, then start `node
  node_modules/vite/bin/vite.js --port <p> --strictPort` so the started pid is the server
  (what `npm run dev` runs after the wasm build), or kill by the port (`netstat -ano | grep :<p>`, then `taskkill //F //PID <pid>`).
- **The wasm build:** `scripts/build-wasm.mjs` runs `wasm-pack build crates/hh-wasm
  --target web --release` into `web/src/wasm/` (git-ignored). `HH_WASM_PROFILE=dev` builds
  unoptimized. It skips the build when a content hash of its inputs (`crates/`,
  `content/`, `scenarios/`, `Cargo.*`, the script) matches `web/src/wasm/.inputs`;
  `HH_WASM_FORCE=1` builds anyway. A no-change build still costs ~11 s (`wasm-opt`).
- **While the dev server runs**, editing anything under `crates/`, `content/` or
  `scenarios/` rebuilds the wasm and reloads the page; a config edit reloads it. A failed
  rebuild keeps the last good engine; the error is in the dev server's terminal.

## Browser check

`npm run e2e` (`scripts/e2e.mjs`) drives headless Edge or Chrome over the DevTools
protocol (no packages) against a served page: `HH_URL` (default
`http://localhost:5173/`), `HH_BROWSER` for a browser off the usual paths, `HH_SHOTS=<dir>`
to save a PNG at each step worth seeing. About 40 s. It plays, in order:

1. **A fresh game through the normal UI only** (no debug menu): it reads the screen,
   buys an opening build order, switches Program 1 between Sell Cloud Compute and the
   bridging buys, loads and launches the first pad, which completes Boot Sequence, First
   Extraction and Proof of Concept (around day 400 at 50x); then retires from the
   Retirement panel, checks the run summary holds the clock, and plays into run 2 (known
   tiles and resources still shown, the game not marked as cheated).
2. **Every panel**: with the debug menu's cheats it reaches Research, Projects and
   Ideologies, opens all 11 nav tiles, checks the cheated mark, a jump ahead and the
   replay check over the whole career.
3. Each speed's days per second, pause, a buy, a reload keeping the save, a save from
   another revision discarded with its notice, the debug-menu scenarios and the replay
   check. Any uncaught page error fails the run.

Against a snapshot rather than the dev server: `npm run build && npx vite preview
--port 61873 --strictPort`, then `HH_URL=http://localhost:61873/ npm run e2e`.

- Edge needs an **absolute** `--user-data-dir`: given a relative one it hands the page to
  the Edge already running and exits (the script resolves it).
- On leaving a page the browser fires `pagehide`, then `visibilitychange`; the page saves
  on both, so a test that edits the save must do it after both.

## QA checker

`npm run qa` (`scripts/qa.mjs`) plays the game the way a player would, over several seeds
and window widths, and checks the screen at each meaningful moment (a panel opened, a story
text, a new quest, a launch, the run summary, run 2, a reload, the replay check). Findings
list `docs/qa-findings.md` holds what hunts found. Per pass (a seed at a width), in one
fresh headless browser: a first visit, a new game on the seed from the debug menu, run 1 at
3x then 50x (buildings, programs, loading and launches) with every panel opened, retirement
and the summary (keys pressed during it must not start the clock), run 2 at 200x, a reload,
the replay check, the play log replayed by `hh-sim playlog` (needs `target/release/hh-sim`,
else it says so), and, on the first pass, a second tab (the first must stand down). The
checks: **overflow** (clipped text without a tooltip holding it, text past its button or
cell, past the window, a page wider than the window), **hidden** (a control's center
covered), **text** (`undefined`, `NaN`, `[object …]`, `{name}` placeholders), **assets**,
**console** (page errors), **input** (a click slower than 300 ms to show), **frame** (a gap
over 250 ms), **memory** (heap growth over 40 MB, a save over 2 MB). Exit 1 on any finding.

```sh
npm run build && npx vite preview --port 0          # a static build: dev-server reloads break a pass
HH_URL=http://localhost:<port>/ npm run qa           # ~2.5 min a pass; 3 seeds x 2 widths ~15 min
HH_PACE=120 HH_WIDTHS=1600 HH_URL=... npm run qa     # a first visit left at 1x for 2 min, a shot every 15 s
```

`HH_SEEDS` (default `11,12,13`), `HH_WIDTHS` (`1280,1920`), `HH_RUN2` (run 2 days, 150),
`HH_SPEED` (run 1's speed, 50; `3` plays it at a person's pace, ~2.5 min a pass more),
`HH_QA_OUT` (default `out/qa`, wiped): one PNG per moment in `<out>/<seed>-<width>/`, a
contact sheet per pass `<out>/contact-<seed>-<width>.png`, and `report.json`. Run it against
a static build or a dev server nobody is editing under: an edit reloads the page mid-pass.
`npm run unit` checks the number formatting (`src/format.ts`).

## The screen

The layout follows the last Godot build (`godot-buildings-day1427.png`, in
`C:\Code\gdrive-sync\helium-hustle\screenshots\`):

- **Left:** the nav grid (tiles from `content/nav.json`, shown as the engine says; a gold
  dot marks a tile that is new this run or whose panel holds something new), the speed
  buttons, Resources (amount / cap, change per day; processors show assigned / total) and,
  once Ideologies is unlocked, each axis's rank and score toward the next rank.
- **Middle:** the active panel. Buildings: the Land bar with Buy Land, a collapsible
  section per category, cards with description, per-day effects, ideology tag (once
  Ideologies is unlocked), stall, price with shortfalls in red, active/owned with −/+,
  Buy, Sell 1 and Sell All (confirmed). Commands, Launch Pads (markets, loading priority,
  pads with cargo, launch and pause), Research, Projects (a funding rate per resource),
  Ideologies, Adversaries (rivals, speculators), Stats (run, career, modifiers), Story
  (quests with their text, achievements, this run's events), Retirement (this run, the
  next run's bonuses, Retire Now with a confirm) and Options (new career, the debug menu).
- **Right:** program tabs 1 to 5, each with processors −/+ and Reset; a line per command
  with its progress, −/+/×, drag to reorder, and its color from today's outcome (ran,
  failed, skipped), and a dark green frame (thick on the left, drawn so no text moves) on
  the line the next processor runs. Above 10x one redraw covers several days (the clock
  wakes every 100 ms), so the last day's pointer would hop around the queue, even
  backward: there each line instead shows its last outcome over those days
  (`advance_run`'s `lines`), with no frame. Commands are added from the Commands panel
  to the selected program.
  Below, Events: the active quest with sub-goal checkmarks and progress, the completed
  quests, and Recent Activity (dated; green good, red bad, blue news).
- **Bottom:** day and run, the cheated mark, Boredom and Energy bars with their rates.
- **Run summary:** when a run ends (retired or forced) a summary covers the screen with
  the clock stopped until Continue.

Rates are per day; numbers past 10,000 use K/M/B/T. Keys: space pauses or resumes, 1 to 5
pick 1x to 200x.

**"New" markers** are the UI's own, not state: what is visible when a run starts (or the
page loads) is the baseline; anything revealed later in the run gets a gold accent (cards,
rows) or dot (nav tiles) until hovered or opened. Nothing is marked on load or at a run's
start, and markers don't survive a reload.

## Using it

- **Speed:** Pause, 1x, 3x, 10x, 50x, 200x. At 1x one tick (one in-game day) passes per
  second. On each timer wake (every 100 ms; browsers throttle background tabs to about once
  a second or less) the page runs the ticks owed for the wall time since the last wake at
  the current speed, so a background tab catches up (it stops at a forced retirement to
  show it). Closing the page pauses the game: there is no offline progress.
- **Actions** apply between ticks, stamped with the current tick; an illegal one shows the
  engine's reason in the notice line.
- **Autosave:** `localStorage` key `hh.save`, written at most once a second while
  ticking, on every action and speed change, and when the page is hidden or closed. It
  holds the state, the game's record (seed, scenario, stamped actions) and the speed,
  tagged with the rules revision and config hash. On load, a save from another revision or
  config, or one the engine can't read, is discarded with a notice and a new game starts
  (no migration code). Options → *Start a new career* begins again on a random seed.
- **One tab at a time:** a page claims the save when it loads; a tab already playing it
  sees the save change (`storage` event), stops its clock, saves and logs nothing more, and
  says to reload it to play there.
- **Debug menu** (Options → Debug): start a scenario (`scenarios/`) or a new game from a
  seed; *Check replay* replays the game's record in wasm and compares fingerprints. The
  line below shows engine, revision, config hash, seed, action count and fingerprint.
  **Cheats** (off by default; the checkbox is remembered in `localStorage` `hh.cheats`):
  grant a resource, set boredom, jump ahead N days. They are engine actions, so the
  replay check still passes, and any cheat marks the game (CHEATED in the status bar).

## Play logs

Every career is recorded as it is played, one JSON-lines file per career, so a human's
play can be replayed exactly and measured against the sim players (`hh-sim playlog`, and
`hh-sim vs-human` for the players on the same game side by side,
[../docs/simulator.md](../docs/simulator.md)).

- **Where:** the dev server (and `vite preview`) takes the page's lines at `POST
  /__playlog` (`vite.config.ts`) and appends them to `<dir>/<career>.jsonl`: `dir` is
  `HH_PLAYLOG_DIR`, else `C:\Code\gdrive-sync\helium-hustle\playlogs\` when that folder's
  parent exists, else `out/playlogs/`. A browser under automation (`navigator.webdriver`:
  the e2e) gets a `bot_` career, which goes to `out/playlogs/` unless `HH_PLAYLOG_DIR` is
  set, so the synced folder holds only people's play. The page sends every 2 s, when
  hidden, and in a beacon when closed (with any send still in flight). A static build with
  no server records nothing (Options says so). A server that can't write the folder answers
  500: the page keeps the lines and retries, and Options says it is NOT recording until a
  send lands; a malformed batch (400) is dropped.
- **One file per career:** `<local start time>_<random tag>_seed-<seed>.jsonl`. A new game
  (Options, a debug start) or a discarded save closes the old file with a `career_end`
  line and starts a new one; localStorage holds only the current career's id.
- **Header** (first line): `format` `hh-playlog/1`, `career`, `rules_revision`,
  `config_hash`, the resolved `config`, `seed`, `scenario`, `engine`, `prior_actions` (a
  career whose log began mid-way), and `commit` (the repo's HEAD, `+dirty` with
  uncommitted changes), stamped by the server.
- **Every other line:** `id` (unique: a line resent after a send the page saw fail is
  read once), `t` (wall-clock ms), `kind`, `tick`, `run`, `day`, `speed` (the speed from
  then on), and by kind: `session_start` (how: save, new, discarded; also after a career begun mid-visit),
  `session_end`, `hidden`, `visible`, `speed` (`from`), `action` (the action, applied at
  `tick`: the game's record), `rejected` (with the engine's `error`), `refused` (the run
  ended first), `panel`, `program_tab`, `retirement` (`ended`, `forced`, `days`),
  `continue`, `story_read`, `new_game`, `debug` (cheats toggled, replay checks),
  `checkpoint` (`fingerprint`, every 10 s while playing, at retirements, when hidden or
  closed), `jump_end` (a checkpoint after a jump ahead, pinning its days to no real time)
  and `career_end` (`reason`; `offline` when written at a later visit, for a discarded
  save, so it takes no place in the career's time). Cheats are actions, so they are in
  the record.

`hh-sim playlog <file>` replays the header's config, seed and scenario with the logged
actions to the last tick and checks the fingerprint logged there.

## Source

`src/engine.ts` (typed wasm wrapper and the view's types), `src/clock.ts` (wall clock and
speeds), `src/save.ts` (autosave), `src/store.ts` (the game in play: actions, ticking,
saves, "new" markers, the run summary), `src/format.ts` (number display), `src/ui/`
(`App.tsx` layout and status bar, `Left.tsx`, `Buildings.tsx`, `panels.tsx`,
`Options.tsx`, `Right.tsx`, `common.tsx`), `src/main.tsx` (entry). The config is
`configs/baseline.json`, imported at build time.
