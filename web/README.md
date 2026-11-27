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
npm run dev                                  # builds the wasm, then serves http://localhost:5173/
npm run game -- --port 5320 --strictPort     # conductor's Run game: npm install, wasm, vite, each skipped if unchanged (~1 s)
npm run wasm                                 # only rebuild web/src/wasm/ (~80 s cold, ~5 s warm, <1 s unchanged)
npm run check                                # typecheck (needs web/src/wasm/: npm run wasm first)
npm run build                                # wasm + typecheck + static site in web/dist/
npm run preview                              # serve web/dist/
```

- **On the LAN:** the dev server and preview listen on every interface (`host: true` in
  `vite.config.ts`) and print a Network URL, so another PC on the local network can play.
- **Under a launcher that kills by pid** (conductor's `start`): `npm`/`npx` leave the server
  as a grandchild a tree kill can miss: start `node node_modules/vite/bin/vite.js
  [preview] --port <p> --strictPort`, or kill by port (`netstat -ano`, `taskkill //F //PID`).
- **The wasm build:** `scripts/build-wasm.mjs` runs `wasm-pack build crates/hh-wasm
  --target web --release` into `web/src/wasm/` (git-ignored); `HH_WASM_PROFILE=dev` builds
  unoptimized. It skips the build when its inputs' hash (list: the script) matches
  `web/src/wasm/.inputs`; `HH_WASM_FORCE=1` builds anyway. It reads a relative
  `CARGO_TARGET_DIR` from the repo root.
- **While the dev server runs**, editing `crates/hh-core`, `crates/hh-wasm`, `content/` or
  `scenarios/` rebuilds the wasm and reloads the page; a `configs/baseline.json` edit reloads
  it. A failed rebuild keeps the last good engine; the error is in the dev server's terminal.

## Checks

`npm run e2e` (the browser check) and `npm run qa` (the QA checker) against `vite preview`,
`npm run unit` in Node: [TESTING.md](TESTING.md).

## The screen

Three columns and a status bar:

- **Left** (`Left.tsx`): the title, the calendar strip once the swarm is revealed, the nav
  grid (tiles from `content/nav.json`, laid out by `navSlots.ts`: two columns of icon and
  name, open tiles packed, utility tiles last; a dot inside a tile's right edge marks it new
  this run or its panel holding something new; on Launch Pads a green dot while a pad is
  ready, a red one while a pad has No cargo), the speed buttons, Resources (Energy, Credits,
  Land, Boredom, Processors, then the rest; amount / cap and the engine's rate per day; at
  its cap the surplus thrown away, muted (Energy's green): "+9 CAP"; short of demand, red:
  "−7 SHORT"; a click on a row opens its breakdown in place: last day's sources and sinks,
  who went short), each ideology axis's rank once Ideologies is unlocked (folded at first in
  a window under 800 px), and at the foot the active main quest's picture, else the lunar
  horizon.
- **Middle:** a starry sky, then the active panel. Buildings (`Buildings.tsx`): the Land bar
  (green and click-to-buy like a card when affordable), a collapsible section per category
  of cards (the picture beside flows and status, one row of cost, land and Sell). A click on
  an affordable card (outlined in green, its name row tinted), off its controls, builds one
  (Shift: up to ten; ignored for 0.5 s after the summary or a story text closes); any other
  card says why it can't. Research cards work the same way (affordable: name row tinted, no
  outline). Commands: a card per verb (Buy, Promote, ...), a target dropdown. Pad cards tint
  their head: green Ready, yellow Paused, red full but short of fuel. Stats: a Boredom
  section (this run's boredom by source, totalling it; then per day), then every other
  resource's breakdown. Story (`Story.tsx`): the quest archive, a spine of main-story beats under
  painted act bands, side quests off it (docs/rulings/story-ui.md, *Story cards*);
  Achievements: badges by category. Acts 2 and 3 (`Saga.tsx`, DRAFT; `docs/arc.md`):
  Timeline, Earth Lab, Meta-Controller, Vault, Dyson; the ending covers the screen once.
- **Right** (`Right.tsx`): the main quest in its own slot at the top, which nothing else
  takes; program tabs A to E (each the engine's status light, and its ⚙ count when it has
  processors: "B ⚙1") with
  processors −/+, Reset, Save / Load (`Library.tsx`, the loadout library); a line per
  instruction (`ProgramLines.tsx`, each row's editors `ProgramLogic.tsx`'s), dragged to
  reorder (an `if` takes its body and `else`), tinted by today's outcome (a failed line says
  why), the next line in an accent frame. Once unlocked, Add: If, Else, Build, Buy Land, Verb;
  the line forms are docs/rules/programs.md's. Body lines indent under their `if` or `else`
  (⇥/⇤) and gray with it when jumped; a note (`#`) is muted text, an unparsed line a
  red-tinted disabled row with its reason, a locked one dimmed, "locked this run" (the
  view's `errors` and `locked`). Above 10x a redraw covers several days: each
  line shows its outcome over them (`advance_run`'s `lines`), no frame. Below the program:
  Events (each one-off and project completion this run, `store.ts` `eventLines`, a click
  reopening its popup), Side Quests and Recent Activity, each fold with an unread count or
  progress (docs/rulings/story-ui.md, *Right panel*).
- **Bottom:** day and run, Boredom and Energy bars with their rates
  (capped as in Resources; "cap tomorrow" the day before a forced retirement).
- **Takeover views** (`Takeover.tsx`, listed in `ui/App.tsx`): a large view over the middle
  and right columns, the left staying, under a slim strip with the day, key resources and
  pause; the game runs on behind it, the covered columns mounted but hidden. Esc or a nav
  tile closes one. Users: the Cortex mock (below) and **Programs** (`Programs.tsx`, opened by
  its nav tile; docs/rulings/programs.md, *Programs view*): **Text** (`TextEditor.tsx`) and
  **Blocks** on the same program, the last mode kept in `localStorage` `hh-programs-mode`,
  both with the run gutter (`Gutter.tsx`) and the **All instructions** panel
  (`Instructions.tsx`, inserting with `programEdit.ts` `insertLine`). Those files' headers
  say how each behaves.
- **Modals**, each holding the clock: the run summary (the ended run's last state behind
  it, saved with it); the story modal (`story.ts` `tellPages`, unit-checked) at a main-quest
  change, a side quest's or project's completion and each one-off event, the new task last
  (a new game: the first quest's; the autopilot: none; a load: a saved chain's unread pages;
  after a summary, once closed).

Rates are per day; dates read "Year 4, day 112" (`format.ts` `dateText`, the view's
`days_per_year`; durations stay in days); resource lists on cards follow the Resources
panel's order (`ui/resourceOrder.ts`); numbers past 10,000 use K/M/B/T. **"New" markers** are the UI's own, not
state: what is visible when a run starts (or the page loads) is the baseline; anything
revealed later in the run gets an accent edge (cards, rows) or dot (nav tiles) until
hovered or opened.

## Theme

Earthrise, light (the default) or dark, picked in Options or the debug menu,
remembered in `localStorage` `hh.theme` (`index.html` applies it before the first paint).
The two share markup, layout and art; only the color tokens atop `src/style.css` differ,
so switching moves nothing (e2e-checked). Chips (resources, axes, rivals) keep their true
colors in both; an ideology axis drawn as text uses its `axis-<id>` class, which has a
readable variant per theme. Fonts are bundled (`@fontsource`, named in `src/style.css`).

**Art** (each loader's header names the files and the write-up that made them): card
pictures for buildings, projects and ideology, `src/ui/cardArt.ts` over
`src/assets/{buildings,projects,ideology}/`; chips (research ones too: a `"BG"` fill takes
the chip's category color), nav icons and the horizon, `src/ui/icons.tsx` (a nav `<id>.svg`
stands in for an unpainted `<id>.webp`); act bands, quest pictures and the sky,
`src/ui/art.tsx`. The sky (`.sky`) is home for a later live base view: replace what it draws,
keep its box.

## Using it

- **Speed:** Pause, 1x to 10x, and 50x and 200x once Accelerated Scheduling is bought
  (`speeds.ts`; a faster pick or save drops to 10x until then); at 1x a day a
  second. Each timer wake (100 ms) runs the ticks owed since the last, so a throttled tab
  catches up (stopping at a forced retirement); a closed page is paused. Keys: space
  pauses or resumes, 1 to 5 pick 1x to 200x; a card reached with Tab buys on Enter.
- **Actions** apply between ticks, stamped with the current tick; an illegal one shows the
  engine's reason in the notice line.
- **Autosave:** `localStorage` `hh.save`, at most once a second while ticking, on every
  action and speed change, and when the page is hidden or closed: the state, the record
  and the speed (a debug speed reloads as 1x), tagged with the rules revision and config
  hash. A save from another revision or config, or one the engine can't read, is
  discarded with a notice.
- **One tab at a time:** a page claims the save when it loads; a tab already playing it
  sees the change (`storage` event), stops and says to reload.
- **Debug menu** (Options → Debug): the theme, a scenario (`scenarios/`) or a new game from
  a seed; *Check replay* replays the record in wasm and compares fingerprints; a line with
  engine, revision, config hash, seed, action count and fingerprint; *Cortex mock in the left
  column* (a UI-only talent-tree mock, `src/ui/CortexMock.tsx`). **Cheats** (off by
  default, remembered in `hh.cheats`): grant a resource, set boredom, complete all
  research, jump ahead N days, the **cheat building** (a hazard-striped *Cheat Fountain*
  card at the foot of the Buildings panel while Cheats are on; its numbers:
  `crates/hh-core/src/cheat.rs`); the **accelerators**: the autopilot with its default
  player and boost ([simulator.md](../docs/simulator.md#accelerators)). Engine actions, so
  replay stays exact; any cheat marks the game cheated in its save (shown in the debug
  info, not the bottom bar), the autopilot ACCELERATED too.

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
- **Lines:** a header, then one line per happening, each with a unique `id` (a resent line
  is read once) and a `kind`; `src/playlog.ts`'s header has the format, `grep "log('"` the
  kinds. A `checkpoint` (the fingerprint) is written every 10 s while playing, at
  retirements, and when the page is hidden or closed.

`hh-sim playlog <file>` replays the header's config, seed and scenario with the logged
actions to the last tick and checks the fingerprint logged there. An autopilot's actions
are logged as any others, its days closed by `jump_end` as a jump's.

## Source

`src/store.ts` holds the game in play (actions, ticking, saves, "new" markers, the run
summary; its header indexes its sections), `src/story.ts` what the story modal tells
(pure), `src/playlog.ts` the play log's client half (the queue, sends to `/__playlog`,
career ids), `src/ui/` the panels; each file's header says the rest. The config is
`configs/baseline.json`, imported at build time.
