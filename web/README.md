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
  unoptimized. It skips the build when its inputs' hash (list: the script) matches
  `web/src/wasm/.inputs`; `HH_WASM_FORCE=1` builds anyway. It reads a relative `CARGO_TARGET_DIR` from the repo root.
- **While the dev server runs**, editing `crates/hh-core`, `crates/hh-wasm`, `content/` or
  `scenarios/` rebuilds the wasm and reloads the page; a `configs/baseline.json` edit reloads it. A failed rebuild keeps the last good
  engine; the error is in the dev server's terminal.

## Checks

`npm run e2e` (the browser check), `npm run unit` and `npm run qa` (the QA checker), each
against `vite preview`: [TESTING.md](TESTING.md).

## The screen

Three columns and a status bar:

- **Left:** the title, the calendar strip once the swarm is revealed, the nav grid (tiles from `content/nav.json`; a dot marks a tile new
  this run or whose panel holds something new; past nine tiles they go compact, icon beside
  name; a green dot on Launch Pads while a pad is ready), the speed buttons, Resources
  (Energy, Credits, Land, Boredom, Processors, then a second group of the rest; amount /
  cap, the engine's rate per day; at its cap the surplus thrown away, muted (Energy's
  green): "+9 CAP"; short of demand, red: "−7 SHORT"; a click on a row opens
  its breakdown in place: last day's sources and sinks, who went short), each ideology
  axis's rank once Ideologies is unlocked (folded at first in a window under 800 px high),
  and at the foot the active main quest's picture, else the lunar horizon.
- **Middle:** a starry sky, a completed side quest's text (the story banner),
  then the active panel. Buildings: the Land bar (green and click-to-buy like a card
  when affordable), a collapsible section per category, cards (`Buildings.tsx`). A click on an affordable card (outlined in green, its name row tinted), off its
  controls, builds one (Shift: up to ten; ignored for 0.5 s after the summary or a story
  text closes); any other card says why it can't. Research cards work the same way.
  Commands: a card per verb (Buy, Promote, ...), a target dropdown. Pad
  cards tint their head: green Ready, yellow Paused, red full but short of fuel. Stats ends
  on every breakdown. Story (`Story.tsx`): a spine of main-story beats under painted act
  bands, each with its quest picture's thumbnail, finished ones lit (a click replays), the active one glowing, one
  "???", side quests off it; Achievements: badges by category. Acts 2 and 3 (`Saga.tsx`, DRAFT; `docs/arc.md`): Timeline,
  Earth Lab, Meta-Controller, Vault, Dyson; the ending covers the screen once.
- **Right:** the main quest in its own slot at the top, which nothing else takes; program
  tabs 1 to 5 with processors −/+, Reset, Save / Load (`Library.tsx`: the loadout library;
  save new or over a slot, load, rename, delete); a line per instruction, dragged to reorder
  (an `if` takes its body and `else`), tinted by today's outcome (a failed line says why),
  the next line in an accent frame. Once unlocked, Add: If, Else, Build, Buy Land,
  Verb (`ProgramLogic.tsx`): a condition is
  a reading (the view's list), `>`, `<`, `>=` or `<=`, and a number (with `%` for a fill), and
  `and`/`or` one more; each clause with its live value and ✓/✗; body lines indented under the `if` or `else` (⇥/⇤), greyed with it when jumped; an `else`
  row has no outcome (free: its `if` decides).
  Above 10x a redraw covers several days: each line shows its outcome over them
  (`advance_run`'s `lines`), no frame. A note line (`#`) is muted text, an unparsed one a
  red-tinted disabled row with its reason, a locked one dimmed with "locked this run" (the
  view's `errors` and `locked`). Below, Events: Story (side story and the latest
  boredom phase), completed quests, and Recent Activity (green good, red bad, blue news),
  each with an unread count.
- **Bottom:** day and run, Boredom and Energy bars with their rates
  (capped as in Resources; "cap tomorrow" the day before a forced retirement).
- **Takeover views** (`Takeover.tsx`): a large view that covers the middle and right
  columns while the left stays, opened and closed by its button under the nav grid, or
  closed with Esc or a nav tile. A slim strip on top shows the day, Boredom, Energy,
  Credits, He-3 and Science, and pause. The game runs on behind it: the covered columns
  stay mounted, only hidden. A view is one `Takeover` entry in `App.tsx`'s `TAKEOVERS`;
  `bleed` drops the body's padding and scrolling for a view that fits itself to it.
  `noButton` keeps a view out of the left column. Users: the Cortex mock (below) and
  **Programs** (`Programs.tsx`), opened by its nav tile (a takeover, not a panel), with a
  **Text | Blocks** toggle on the same program; the last mode is kept in localStorage
  (`hh-programs-mode`), Blocks for a new player. **Blocks** is the right column's list
  (`ProgramLines.tsx`) with a run gutter first in each row, a row selected by a
  click, a repeat-count number field first on command rows and a note's text field; edits
  apply at once. A number field (`NumberField`, a text input: a number input's selection
  can't be set) selects its text on a click or Tab, and its row is undraggable while the
  button is down in it, so a drag there selects text. **Text** (`TextEditor.tsx`): program tabs, Apply, and a textarea between
  the gutter and a notes column (reasons, "fired k/10" on `if`s, from the store's history
  of `advance_run`'s `if_true` bits). The engine parses (`parse_program_text`) and writes
  (`program_text`) the text (docs/rules/programs.md, *Text form*); a changed draft is
  applied on Apply, on a tab switch, a mode switch and on closing. **The gutter**
  (`Gutter.tsx`): a status dot (green ran, grey skipped, red unparsed, amber locked, hollow
  failed); a repeated line's runs as dots up to 8 (green done, red failed, hollow pending;
  the view's `repeats`), a bar past 8. Right of either mode, **All instructions**
  (`Instructions.tsx`): the engine's list (`program_instructions`: by group, only lines
  revealed and usable now, nothing locked, `else` only where it fits at the caret or selected row); a
  click inserts the line below the caret's line or selected row with its indent (`if` /
  `else` at the top level with an empty body line, an empty caret line filled in place;
  `programEdit.ts` `insertLine`) and moves the caret or selection there; a hover shows the
  live card of its command, building, research or project.
- **Modals**, each holding the clock: the run summary when a run ends; the story modal
  (`store.ts` `tell`) at each main-quest change, a page per paragraph of the view's
  `story`/`payoff`, the new task last (a new game: the first quest's; a load, the autopilot:
  none; after a summary, once it closes).

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
Building pictures: `src/assets/buildings/<id>.webp` (`src/ui/buildingArt.ts`,
`experiments/results/020_building_card_art/`).
Chips, nav icons, horizon: `src/ui/icons.tsx` over `src/assets/{chips,nav,horizon}/`
(`experiments/results/027_art_in_game/`); a nav `<id>.svg` stands in for an unpainted
`<id>.webp`. `src/ui/art.tsx`: act bands and quest pictures (`src/assets/{story,quests}/`,
by `experiments/results/078_*/process.py`), and the sky (`.sky`), home for a later live
base view: replace what it draws, keep its box.

## Using it

- **Speed:** Pause, 1x to 200x; at 1x a day a second. Each timer wake (100 ms) runs the
  ticks owed since the last, so a throttled tab catches up (stopping at a forced
  retirement); a closed page is paused. Keys: space
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
  column*, which shows the button of a UI-only radial mock of the talent tree, a takeover
  view (`src/ui/CortexMock.tsx`, placeholders, no effect, in memory; hovering a locked node
  outlines its prerequisites; left-click adds a point or switches a socket's part,
  right-click takes one back). **Cheats** (off by
  default, remembered in `hh.cheats`): grant a resource, set boredom, complete all
  research, jump ahead N days, the **cheat building** (a hazard-striped *Cheat Fountain*
  card at the foot of the Buildings panel, only while Cheats are on: 1 credit, +1000 credits
  a day, +10 to every other income, −10 boredom); the **accelerators**: 2k and 10k speeds and the
  autopilot with its default player and boost
  ([simulator.md](../docs/simulator.md#accelerators)). Engine actions, so replay stays
  exact; any cheat marks the game cheated in its save (shown in the debug info, not the bottom bar),
  the autopilot ACCELERATED too.

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
actions to the last tick and checks the fingerprint logged there. An autopilot's actions
are logged as any others, its days closed by `jump_end` as a jump's.

## Source

`src/store.ts` holds the game in play (actions, ticking, saves, "new" markers, the run
summary), `src/playlog.ts` the play log's client half (the queue, sends to `/__playlog`,
career ids), `src/ui/` the panels; each file's header says the rest. The config is
`configs/baseline.json`, imported at build time.
