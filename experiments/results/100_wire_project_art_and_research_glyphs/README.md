# 100: project card art and research chips in the game

> Archived (127): `art/pickers/project_art_r1/` and `art/pickers/research_glyphs_r1/` now live at `C:\Code\gdrive-sync\helium-hustle\archive\art\pickers\`.

Ships Matt's picks from pickers `project_art_r1` (one picture per project, all 15) and
`research_glyphs_r1` (category chip colors, one glyph per research item, all 17).

## Recipe (repo root)

- `python experiments/results/100_wire_project_art_and_research_glyphs/projects.py` (Pillow
  with WebP): `PICKS` names each project's option; 097's already-cut
  `art/projects_r1/<project>-<option>_192.png` is saved as
  `web/src/assets/projects/<project>.webp` (quality 88 as 020's building art; 198 KB for 15).
  `src/ui/cardArt.ts` (was `buildingArt.ts`) serves both buildings and projects.
- `node experiments/results/100_wire_project_art_and_research_glyphs/research.mjs`: `PICKS`
  names each item's glyph in `098_research_glyphs_r1/glyphs.mjs`; writes
  `web/src/assets/research/<id>.svg` (the 24-unit glyph, `"BG"` kept as a token).
  `ResearchChip` in `src/ui/icons.tsx` draws the chip as `Chip` does, its square in
  `CATEGORY_COLORS[category]` (098's proposed hues, picked as the `category` scheme), and
  swaps `"BG"` for that color.
- `shots.mjs` (header has the preview commands): Research and Projects in scenario
  `everything`, light and dark; checks every shown research card leads its title with its
  own 22 px chip in a category color and every shown project card has its 192 px picture
  in the 96 px box. `100_*.png` here.

A new project or research item: one line in `PICKS` and a rerun; without a file its card
shows no picture or chip.

## The art rounds, rerunnable

Copied from conductor's kept folders (`out/keep/` of items 097 and 098):

- `097_project_art_r1/`: `gen.mjs` (Codex draws per `prompts.json` into
  `art/projects_r1/raw/`), `picker.py` (cuts, the `_192.png` card pictures, and the picker
  `art/pickers/project_art_r1/`). Their repo root now resolves four folders up (it was two
  under `out/`); scratch still goes to `out/097_project_art_r1/`.
- `098_research_glyphs_r1/`: `glyphs.mjs` (every candidate glyph, the category hues, `chip()`),
  `mock.mjs` (the picker `art/pickers/research_glyphs_r1/`), `sheet.mjs` (a contact sheet).
  Run from the repo root as `node experiments/results/100_wire_project_art_and_research_glyphs/098_research_glyphs_r1/mock.mjs`;
  identical to the shared `art/research_glyphs/` copy.

## Decisions

- **The research chip sits before the title** (098's mock, which Matt picked from); the
  099 scope badge and "Done" stay at the right. Its tooltip is the category name.
- **A project's picture sits beside its text** (description, reward, completion and days
  lines), as on building cards; the funding rows keep the card's full width below it.
