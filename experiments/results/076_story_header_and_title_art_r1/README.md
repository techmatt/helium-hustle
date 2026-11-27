# 076: act chapter headers and the title, candidates

> Archived (127): `art/pickers/story_title_art_r1/` now lives at `C:\Code\gdrive-sync\helium-hustle\archive\art\pickers\`.

Picker `story_title_art_r1` (`art/pickers/story_title_art_r1/manifest.json`, the shared,
uncommitted `art/` folder). Nothing goes into the game here; a later item ships the picks.

## Recipe

From the repo root (Node 24, system Chrome, Python with Pillow; `npm ci` in `web/`;
`playwright-core` in `PW_DIR`, by default `out/076_story_header_and_title_art_r1/tools/`):

1. `node experiments/results/076_story_header_and_title_art_r1/gen.mjs [<id> …] [--pool=N]`:
   025's Codex draw with this folder's `prompts.json`. Ids `act<N>-<framing>` (framings
   `side`, `wide`, `vignette`) and `title-<style>` (`hull`, `stencil`, `amber`, `regolith`,
   `earthrise`). With no ids, every one not yet drawn. Originals into `art/story_title/raw/`, earlier draws kept as `*.try<n>.png`.
2. `python …/process.py`: titles trimmed to their alpha box (`art/story_title/title/`);
   act headers with letterbox bars trimmed (`art/story_title/header/`).
3. `node …/mock.mjs` (about 15 s), then `python …/picker.py`. `contact.py` makes a review
   sheet of every raw draw, the mock's band outlined, in `out/`.

`crops.json` holds the vertical focus (`background-position` y, percent) of a header whose
subject sits off center; the rest are centered.

## Candidates

- **Act headers**, four framings per act (after `none`, the act title with no art, as the
  row's current option), each mocked as a band at the top of an act in a timeline in the
  middle column: 595 px wide (a 1280 px window) on the light page, 1058 px (a 1920 px window)
  on the dark one, the band 3:1 at both, a proposal the shipping item may change. The
  painting is the same in both themes (a black space sky), so the act title is light text
  with a shadow in both. `side`: subject right, dark sky left for the title beside it.
  `wide`: panorama kept dark, title over the center with a soft scrim. `vignette`: one small
  central emblem, title under it on a bottom scrim. `fill`: the first vignette round, which
  came back filling the band (`act<N>-vignette.try1.png`), kept because some are the
  strongest paintings (the Vault door); the title sits on the art, so it reads less well.
- **Title**, at the h1's 29 px line in the 287 px and 363 px left columns (1280 and 1920 px
  windows), light and dark, the
  first nav tiles under it for scale: five painted wordmarks (transparent, each with its own
  dark outline or plate so one image serves both themes) and two CSS-lettered treatments on
  the theme tokens (`lettered-split`: HELIUM in ink, HUSTLE in amber, an amber rule;
  `lettered-orbit`: the current text with a small Earth-and-orbit mark).

## Findings

- **The image tool spells HELIUM HUSTLE right** in all five painted styles on the first
  try, when the prompt spells it out letter by letter and asks for one line, "seven times as
  wide as tall": they came back 6:1 to 7.6:1, with real alpha.
- **At 29 px** the plate styles (`stencil`, `amber`) read best on both themes: the plate is
  its own ground. The outlined letters (`hull`, `earthrise`) read on both; `regolith`'s grey
  letters are the weakest on the dark panel.
- **Wide bands from a 3:2 canvas.** Asking for the subject "inside the middle half of the
  height" works; an earlier "four times as wide" wording put Act 1's Earth at the very top
  (kept as `act1-side.try1.png`). Several draws came back letterboxed (a ~3:1 picture between
  pure black bars), which `process.py` trims.
- **"Small central emblem" is not honored by default**: the first vignettes filled the band
  (the Vault door, the sphere), leaving the title on bright art. The redraw states the
  emblem's size outright (a third of the width, half the height) and gets it; the first
  round is kept as the `fill` framing.
- 22 draws, 30-46 s each, up to 4 at once, none failed, no usage limit
  (`art/story_title/raw/log.jsonl`).
