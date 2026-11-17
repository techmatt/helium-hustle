# 027: the picked art in the game

Matt's picks from `nav_horizon_art_r1` (025) and `small_icons_r1` (026) are the game's art.

## What is where

- **`web/src/ui/icons.tsx`** holds it all: `Chip` (the color square with the glyph knocked
  out, its name as an SVG `<title>` tooltip), `NavIcon` (one map from nav id to its files)
  and `Horizon`. The color tokens (resources, ideology axes and the rivals' 026 colors:
  ARIA-7 `#ec407a`, CRUCIBLE `#ff6f00`, NODAL `#006064`, FRINGE-9 `#9e9d24`) live there;
  the old `Square`, `NAV_COLORS` and the inline SVG horizon are gone.
- **`web/src/assets/chips/<id>.svg`**: the bare picked glyph per resource, axis and rival
  (`export.mjs`). The square and ink (white, or near-black under 2.5:1) are drawn in code from
  the token, so a recolor touches one line.
- **`web/src/assets/nav/<id>.webp`** (025 `paint`, 96 px) and **`<id>-compact.svg`** (18 px:
  025's glyph chip, or 026's subject glyph for Projects, Adversaries, Retirement, Options),
  from `images.py` and `export.mjs`. Another icon style only swaps these files.
- **`web/src/assets/horizon/{bare,pad,climb}.webp`**: 026's earthlight strips, full size.
  Bare until `view.pads` is non-empty, then pad; a new shipment line in the view shows
  climb for 4 s of real time. Climb's nose sits about 8 px under the strip's top; the strip
  keeps 12 px of padding above it, so nothing is clipped and no extra height was needed.

Both scripts read the shared `art/` folder and never write it. Run from the repo root:
`node experiments/results/027_art_in_game/export.mjs`, `python experiments/results/027_art_in_game/images.py`.

## Where chips appear

Resources, card flows and costs, the Land row, the Research and Projects rows, Markets,
Loading priority, pad cargo and fuel, program lines (the chips of what the command
produces), Recent Activity (shipments, speculator surges, rival dumps with the rival's
chip, cargo lost), the Adversaries tables (rival chips, market chips), the Ideology summary
and Ideologies headings. A building's ideology is a chip in its card header with the axis
name as its tooltip; the two "Nationalist-aligned." description tails are gone from
`content/buildings.json`.

## Shots (`shots/`, `shots.mjs`, 1280 x 800, light and dark)

`<theme>-fresh-1280` and `-late-1280` (whole page), `-nav-normal`, `-nav-compact`,
`-resources`, `-card`, `-markets`, `-pad-card`, `-program-lines`, `-event-shipment`,
`-events-late` (rival chips), `-adversaries`, `-ideologies`, `-ideology-summary`,
`-ideology-badge-tooltip`, `-horizon-bare`, `-horizon-pad`, `-horizon-climb`,
`-left-column-climb`. The tooltip box in `-ideology-badge-tooltip` is a stand-in drawn by
the script with the chip's `<title>` text: native tooltips never paint in a headless
screenshot.

The late game is reached from scenario `pad_loading` with cheats (grants, buys, jumps)
through the quest chain to Ideologies (all 11 tiles: compact nav).

QA: `npm run qa` at 1280, seeds 27 (light) and 28 (dark): 0 findings (`qa/report.json`;
contact sheets, 6 MB each, kept outside the repo).
