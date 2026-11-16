# 020: card art for every building

Every building in `content/buildings.json` (16) drawn in 019's `cut34` style (painterly
lunar concept art, each building at its own natural 3/4 view from slightly above, on a
small irregular regolith patch, cut out; white hulls, orange trim, one hard sun from the
left) and shipped as the 96 px picture on its Buildings card, both themes. Picker
`building_art_set_r1`.

## Recipe

From the repo root (Node 24, system Chrome, Python with Pillow and WebP; `npm ci` in `web/`;
`playwright-core` in a folder named by `PW_DIR`, by default `out/020_building_card_art/tools/`):

1. `node experiments/results/020_building_card_art/gen.mjs [<building> …] [--pool=4]`:
   019's Codex draw with its `cut34` prompt, unguided, one subject line per building
   (`prompts.json`). With no ids, every building not yet drawn and not reused. Originals
   into the shared `art/buildings/set/raw/` (a redraw keeps the old one as `.try<n>.png`);
   the four 019 drew (Solar Panel, Excavator, Smelter, Data Center) are reused from
   `art/buildings/r2/raw/` (`prompts.json` `reused`).
2. `python …/process.py`: per building `art/buildings/set/<id>_cut.png` (trimmed, 600 px)
   and the shipped `web/src/assets/buildings/<id>.webp`; review sheets in `out/`.
3. `node …/mock.mjs` (the card in light and dark, about 10 s), then `python …/picker.py`,
   into `art/pickers/building_art_set_r1/`.

A new building is one subject line in `prompts.json` and steps 1–3 for its id; until it has
a file its card shows no picture.

## The shipped picture

192 px square WebP (2x the card's 96 px box), quality 88, 11–15 KB each, 236 KB for all 16,
bundled by Vite (hashed under `dist/assets/`). The crop is tight on the building: columns
whose opaque top sits in the bottom 30% of the cut-out are ground only; the square is
centered on the rest (the building's span plus 6% a side), at least as tall as the
cut-out less 4% of patch rim, bottom-aligned, and the patch fades out over the square's
outer 10% at the left, right and bottom, so no straight cut edge shows on the card color.
On the card it sits under the title, beside the description and flows (`.card-body`).

## Findings

- **The style holds across the set from the prompt alone.** All 12 new draws came back
  1254 px RGBA with clean alpha (no keying), white and orange, the sun from the left, at
  matching scale and patch size, beside 019's four. 12 at 4 at once in 2.5 minutes,
  44–75 s each, none failed, no usage limit (times in `art/buildings/set/raw/log.jsonl`).
- **Two redrawn before shipping** (first draws kept as `.try1.png`): Electrolysis Plant had
  a white vapor plume from its vent, which an airless Moon can't have (and the Smelter's
  line rules out); Microwave Receiver came back as a glassy flat panel that read as a
  second Solar Panel at 96 px. The redraw asks for see-through silver wire mesh with
  antenna stubs, "clearly not solar panels", and gets it.
- **Subject lines that carry function at 96 px** lean on one cue each: the ice hopper's
  pale blue chunks, the Fabricator's green boards, the Battery's green charge bars, the
  Arbitrage Engine's dish, the Launch Pad's rocket, the dome's trees. The Refinery and the
  Electrolysis Plant are the busiest silhouettes (pipes and tanks) and the likeliest to
  blur together small.
