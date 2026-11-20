# 025: nav tile icons and the lunar horizon, candidates

> Archived (034): `art/pickers/` now lives at `C:\Code\gdrive-sync\helium-hustle\archive\art\pickers\`.

Picker `nav_horizon_art_r1` (`art/pickers/nav_horizon_art_r1/manifest.json`, the shared,
uncommitted `art/` folder). Nothing goes into the UI here.

## Recipe

From the repo root (Node 24, system Chrome, Python with Pillow; `npm ci` in `web/`;
`playwright-core` in `PW_DIR`, by default `out/025_nav_horizon_art_r1/tools/`):

1. `node experiments/results/025_nav_horizon_art_r1/gen.mjs [<id> …] [--pool=N]`: 020's
   Codex draw (`../020_building_card_art/gen.mjs`) with this folder's `prompts.json`. Ids
   `icon-<tile>`, `horizon-<direction>`, and `progress-pad-cutout` then
   `progress-liftoff-cutout`, each an edit of the draw before it (attached with
   `codex exec -i`), so the base stays put; stages run last, one at a time, in order. The
   strips of all three stages are cut at the bare draw's band. Originals into `art/nav_horizon/raw/`.
2. `python …/process.py`: the painterly icons trimmed square, the card art crops (from
   `art/buildings/set/<id>_cut.png`) and the horizon strips, into `art/nav_horizon/`.
3. `node …/mock.mjs` (about 15 s) and `python …/picker.py`. `mock.mjs` reads the UI as it
   stood then (`NAV_COLORS` in `web/src/ui/common.tsx`, the inline `<Horizon>` SVG in
   `art.tsx`); 027 removed both, so run it from a checkout of commit `0828be0`.

`glyphs.mjs` holds the flat glyphs (24-unit SVG paths), drawn by hand.

## Candidates

Per nav tile (all 11 in `content/nav.json`, Ideologies included), each mocked on the real
nav tile CSS at the 1280 px column (82 px tile, 34 px icon, normal and selected; compact
18 px), light and dark:

| Key | What | Tiles |
|---|---|---|
| `crop` | the building card art cut-out, cropped to the building, patch faded | Buildings (Excavator), Commands (Data Center), Launch Pads, Research (Research Lab), Projects (Fabricator) |
| `paint` | a new painterly icon in the `cut34` style, one object, transparent | all |
| `glyph` | a solid flat silhouette in the tile's current color | all |
| `chip` | the current colored square with the glyph knocked out (white, or near-black under 2.5:1) | all |

icon_style_r1 (017) drew ink line, duotone, solid-ink and gray-tile icons in one geometry
and was rejected whole; nothing here is ink-on-panel: the glyphs carry the tile color.

Horizon: three painted strips (`cutout`: transparent sky, the panel shows above as today;
`earthrise`: painted edge to edge with a black sky and Earth; `earthlight`: night, blue-grey
ground, warm windows, transparent sky), cut at 2.5:1 and shown `object-fit: cover`,
bottom-aligned, in the 150 px strip at the 270 px and 346 px columns. Progress on
`cutout`: bare, plus pad and extractors, a rocket lifting off.

## Findings

- **The first horizon draws were close-ups** (the base filling the frame), useless in a
  270 x 138 strip; asking for "the base spans a third of the width, open regolith either
  side" fixed it. The close-ups are kept as `*.try1.png`.
- **Edit-by-attachment holds the scene**: the pad and liftoff stages came back with the
  same base, craters and rocks in place; only the added pieces differ. The liftoff
  rocket stands on the pad with its flame lit rather than clearly airborne.
- All 21 draws (5 of them the close-up round) 40-71 s each, up to 5 at once, none
  failed, no usage limit
  (`art/nav_horizon/raw/log.jsonl`).
- At 18 px the painterly icons and crops turn to a brown-grey blob except the strongest
  silhouettes (rocket, flask, flag, gear); the glyph and chip read at both sizes.
