# 077: Story header compositions and the title, round 2

> Archived (127): `art/pickers/story_title_art_r2/` now lives at `C:\Code\gdrive-sync\helium-hustle\archive\art\pickers\`.

Picker `story_title_art_r2` (`art/pickers/story_title_art_r2/manifest.json`, the shared,
uncommitted `art/` folder): 7 rows, 57 images. Nothing goes into the game here; a later item
ships the picks. The whole-window pages are also in `shots/` (JPEG) and the shared
screenshots folder as `077_page_*.png`.

## Recipe

From the repo root (Node 24, Python with Pillow, system Edge or Chrome; `npm ci` and
`npm run build` in `web/`, then `npx vite preview --port <port>` there):

1. `node …/gen.mjs [<id> …] [--pool=N]`: 076's Codex draw with this folder's `prompts.json`
   (`draws`: id → [framing, scene]; framings `side`, `tall`, `spine`, `bridge`, `takeover`).
   Originals into `art/story_title_r2/raw/`, one log line each. `contact.py` makes a review sheet.
2. `python …/process.py`: WebPs into `art/story_title_r2/web/` (letterbox trimmed; a spine
   draw cut to its middle 14% strip), plus round 1's picked `side` headers.
3. `node …/mock.mjs <url>` (about 40 s; `MOCK_ONLY=<regex>` for a subset), then
   `python …/compose.py` and `python …/picker.py`.

## How the mocks are made

In the running game, not a stand-alone page: `mock.mjs` plays a new game through 1.6 with the
debug autopilot (075's route; the step stops at 1.7 Burnout), then lays each candidate into the
real Story panel or left column with injected CSS and DOM. Story mocks are the middle column at
1280x800 light (595 px of content) and 1920x1080 dark (1058 px); page mocks the whole window at
both; titles the left column's top at 1280 and 1920, each theme, shot at 2x pixels.

Acts 2-4 can't be reached in a screenshot run yet, so a later act's mock finishes Act 1 and
appends the later acts' beats in the panel's own markup, from `content/events.json`'s main
quests split by `content/acts.json` (every beat lit with its summary, the act's last taking the
real active beat's markup, then the ???).

## Candidates

- **Act rows**: `current` (075's plain band), round 1's `side` for comparison, three new `side`
  draws per act (different subjects in the same framing), then compositions: `tall` (a 2:3
  backdrop behind the act, the spine and beats over it, fading down the act), `bleed` (the
  picked side painting shown whole, 3:2 uncropped, running on behind the first beats), `sky`
  (Act 1 only: the base painted with no Earth, reaching up into the column's sky so the page's
  own Earth hangs over it), `bridge` (Acts 2-4: a 5:1 transition strip from the act before).
- **Spine row**: the spine as a painted strip repeated down the timeline (conduit, launch
  trail, sun beam; a luminance mask drops the strip's black ground so it stands on the light
  page) and a CSS-only dashed orbit line; done dots become amber lamps.
- **Page row**: whole windows: current, side band, bleed + conduit, sky, tall, Act 2 reached
  (bridge + trail spine), and a full-width Story takeover (like the Cortex) at 1.7, unreached
  acts fogged, and the same late in the game, unfogged.
- **Title row**: round 1's `lettered-split` and ten variants on it, all CSS on the theme tokens:
  heavier, stacked, outlined HELIUM, HUSTLE on an amber plate, a helium-3 superscript, a glow;
  and four tying into the page: the rule as a tiny Story spine, the page's own Earth SVG between
  the words, the left column's horizon art behind it, the act bands' night-sky plate.

## Findings

- **Act 1 art fights the page's Earth.** Every Act 1 draw paints its own Earth, a second one
  under the column's Earth at the top of the Story page. Asking for "no Earth, no planet, ...
  the page hangs its own Earth there" works on the first try (`act1-skyless`), and the `sky`
  composition lets the page's Earth hang over the painted base.
- **A painted spine needs its ground dropped**: the strips come back on pure black, a black
  bar on the light page; a luminance mask of the strip itself keeps the painting and loses the
  ground in both themes.
- **The takeover panorama doesn't line up with four equal columns**: the tool spreads the four
  regions unevenly (the vault sits across Acts 2 and 3). A shipping takeover would place the
  act columns over the painting's regions, or draw one panel per act.
- **Spoilers**: the art for later acts shows only once the act is reached (the takeover mock
  fogs unreached columns); the panorama would otherwise show the vault and sphere at 1.1.
- 24 draws, 27-64 s each, 4 at once, none failed, no usage limit
  (`art/story_title_r2/raw/log.jsonl`). Several side draws came back wider than 3:2 (2000x667,
  1536x450) rather than letterboxed.
