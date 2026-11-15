# web: the Helium Hustle client

A minimal browser client on `hh-wasm`. Vite + TypeScript, no UI framework. It renders the
engine's view and sends actions; every price, rate, cap and "affordable" comes from the
engine ([../docs/architecture.md](../docs/architecture.md), *The view*).

## Build and run

Needs the root README's *Toolchain*.

```sh
cd web
npm ci                          # once per worktree (~10 s)
npm run dev                     # builds the wasm, then serves http://localhost:5173/
npm run dev -- --port 61870     # another port
npm run wasm                    # only rebuild web/src/wasm/ from crates/hh-wasm (~8 s warm)
npm run check                   # typecheck (needs web/src/wasm/: npm run wasm first)
npm run build                   # wasm + typecheck + static site in web/dist/
npm run preview                 # serve web/dist/
```

- **The wasm build:** `scripts/build-wasm.mjs` runs `wasm-pack build crates/hh-wasm
  --target web --release` into `web/src/wasm/` (git-ignored). `HH_WASM_PROFILE=dev` builds
  unoptimized.
- **While the dev server runs**, editing anything under `crates/`, `content/` or
  `scenarios/` rebuilds the wasm and reloads the page; a config edit reloads it. A failed
  rebuild keeps the last good engine; the error is in the dev server's terminal.

## Browser check

`npm run e2e` (`scripts/e2e.mjs`) drives headless Edge or Chrome over the DevTools
protocol (no packages) against a served page: `HH_URL` (default
`http://localhost:5173/`), `HH_BROWSER` for a browser off the usual paths. It checks each
speed's days per second, pause, a Solar Panel buy (in the `rich` scenario: a fresh game
can't afford one), a reload keeping the save, a save from another revision discarded with
its notice, debug-menu scenarios, a command set on Program 1 and the replay check. About
25 s. Against a snapshot rather than the dev server: `npm run build && npx vite preview
--port 61873 --strictPort`, then `HH_URL=http://localhost:61873/ npm run e2e`.

- Edge needs an **absolute** `--user-data-dir`: given a relative one it hands the page to
  the Edge already running and exits (the script resolves it).
- On leaving a page the browser fires `pagehide`, then `visibilitychange`; the page saves
  on both, so a test that edits the save must do it after both.

## Using it

- **Speed:** Pause, 1x, 3x, 10x, 50x, 200x. At 1x one tick (one in-game day) passes per
  second. On each timer wake (every 100 ms; browsers throttle background tabs to about once
  a second or less) the page runs the ticks owed for the wall time since the last wake at
  the current speed, so a background tab catches up. Closing the page pauses the game:
  there is no offline progress.
- **A minimal Arc 1 page** (the real UI is a later item): the day and run, the active
  quest, the resources and buildings the engine marks visible, and **Program 1**.
- **Buy:** each building's button shows its next price and is enabled when the engine says
  it can be bought now. An action applies between ticks, stamped with the current tick.
- **Program 1:** a button per visible command sets Program 1 to run it, one step per
  processor per day (a fresh game puts its one processor there). Sell Cloud Compute is the
  way to first credits. Other programs, research, pads, projects and retirement have no
  controls yet.
- **Autosave:** `localStorage` key `hh.save`, written at most once a second while
  ticking, on every action and speed change, and when the page is hidden or closed. It
  holds the state, the game's record (seed, scenario, stamped actions) and the speed,
  tagged with the rules revision and config hash. On load, a save from another revision or
  config, or one the engine can't read, is discarded with a notice and a new game starts
  (no migration code).
- **Debug menu** (bottom): start a scenario (`scenarios/`) or a new game from a seed;
  *Check replay* replays the game's record in wasm and compares fingerprints. The line
  below shows engine, revision, config hash, seed, action count and fingerprint.

## Source

`src/engine.ts` (typed wasm wrapper), `src/clock.ts` (wall clock and speeds),
`src/save.ts` (autosave), `src/main.ts` (wiring and rendering). The config is
`configs/baseline.json`, imported at build time.
