# 026: small chip icons, compact nav fallbacks, horizon progress on earthlight

Picker `small_icons_r1` (`art/pickers/small_icons_r1/manifest.json`, the shared, uncommitted
`art/` folder): 32 `many` rows, 116 options. Nothing goes into the UI here. 025's
`nav_horizon_art_r1` files are untouched (its earthlight draw is read, never written).

## Recipe

From the repo root (Node 24, system Chrome, Python with Pillow; `npm ci` in `web/`;
`playwright-core` in `PW_DIR`, by default `out/026_icon_art_r1/tools/`):

1. `node experiments/results/026_icon_art_r1/gen.mjs [pad|liftoff|climb|hop]`: Codex draws
   (025's route), each stage an edit of the one before: `pad` edits 025's bare
   `art/nav_horizon/raw/horizon-earthlight.png`, `liftoff`, `climb` and `hop` edit `pad`. Into
   `art/small_icons/raw/`, one line each in `log.jsonl`. Prompts: `prompts.json`.
2. `python …/process.py`: strips cut at the bare draw's band with 025's rule, faint alpha
   (under 40) zeroed, into `art/small_icons/horizon/`.
3. `node …/mock.mjs` (about 30 s) and `python …/picker.py`.

`glyphs.mjs` holds the glyphs (24-unit SVG, drawn by hand) and the chip (025's look; it also passes its ink to stroked parts).
`sheet.mjs` renders a working contact sheet of every glyph into `out/026_icon_art_r1/`.

## Candidates

- **Resources** (all 12 in `content/resources.json`), **ideology axes** (3) and **rivals**
  (all 4 in `content/rivals.json`): three glyphs each, plus `current` (the plain square).
  Each image: the chip at 96 px, then light and dark inline at 14 px (Resources or
  Adversaries row, card cost line or the card's "aligned" line) and 16 px (event line).
- **Colors.** Resources keep `RESOURCE_COLORS`; ideologies already have `AXIS_COLORS`
  (nationalist `#c62828`, humanist `#2e7d32`, rationalist `#1565c0`), kept. Rivals had none;
  proposed ARIA-7 `#ec407a` (rose), CRUCIBLE `#ff6f00` (forge orange), NODAL `#006064`
  (deep teal), FRINGE-9 `#9e9d24` (olive). Each sits at least 32 CIE76 ΔE from every
  resource and axis color (the closest existing pair, Helium-3 and Science, is 16).
- **Compact nav** (all 11 tiles, 18 px only): `paint` (the pick, shrunk: current), `chip`
  (025's glyph in the square), and either `subj` (a new glyph matched to the painted
  subject: console, crane, drone, deck chair, wrench) where 025's glyph showed something
  else, or `glyph` (025's glyph bare in the tile color).
- **Horizon progress on earthlight:** bare (025's strip), pad (plus landing pad and two
  extractors), and three takes of the airborne rocket: liftoff, climb, hop.
- Two family sheets (every candidate at 16 and 32 px, light and dark) for judging the set.

## Findings

- **Edit-by-attachment held on earthlight too**: pad, liftoff and climb keep the base,
  rocks and lamp in place; the night lighting carried over (the pad and flame light the
  ground warm).
- **The first liftoff draw still sat almost on the pad** (a short flame gap). A second
  prompt, `climb` ("no taller than the small dome", "a gap at least its own height"), gave
  a clearly airborne rocket, but it sits higher, its nose level with the lamp top at the
  strip's top edge (about 8 px of 276 to spare): neither take is both clearly airborne and
  low. A third take, `hop` (a rocket half the small dome's height, a gap about its own
  height), is low and airborne but small: at the 270 px column the rocket is about 8 px
  wide and the gap reads mostly as flame. Climb's
  first take (`progress-climb-earthlight.try1.png`) came back with a painted grey haze and
  a fake checkerboard in the "transparent" sky (alpha up to 80); redrawn, the retake is
  clean. `process.py` zeroes alpha under 40 as a guard.
- 5 draws, 46-60 s each, none failed, no usage limit.
- At 14 px the chip carries a glyph of about 10 px: bold single silhouettes (bolt, drop,
  flame, heart, pin, star) read; three-part ones (snowflake, atom, hub, lattice, CPU pins)
  turn to texture but keep the square's color, which is still the identity.
