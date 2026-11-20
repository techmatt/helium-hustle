# 018: building art, style round 1

> Archived (034): `art/buildings/r1/` and `art/pickers/` now live under
> `C:\Code\gdrive-sync\helium-hustle\archive\art\` at the same relative paths.

Seven styles drawing the same four buildings (Solar Panel, Regolith Excavator, Smelter, Data
Center), mocked onto the Buildings panel's cards in Earthrise light and dark, and the picker
`building_art_r1`. Adapted from AIComplete's `tools/card-art/` scene route (copied and cut
down, nothing shared).

## Recipe

Run from the repo root (Node 24, system Chrome, Python with Pillow; `npm ci` in `web/` for the
fonts; `playwright-core` in a folder named by `PW_DIR`, by default
`out/018_building_art_round/tools/`):

1. `node experiments/results/018_building_art_round/gen.mjs [<style>-<building> …] [--pool=3]`
   draws with Codex CLI's built-in image tool (`codex exec --json -s read-only`, the prompt
   on stdin, the PNG collected from `~/.codex/generated_images/<thread>/`), on Matt's
   ChatGPT login. With no ids, every style × building in `prompts.json`. Originals go to
   `art/buildings/r1/raw/<style>-<building>.png`; a redraw keeps the earlier one as
   `.try<n>.png`. Stops at the first usage-limit message.
2. `python …/process.py` cuts them: a framed style gives a 3:1 banner (full width, centered)
   and a square thumb (80% of the height); a cut-out style (real alpha from the tool) is
   trimmed to its opaque box. Into `art/buildings/r1/<style>/`.
3. `node …/mock.mjs [sheet …]` renders the sheets into `art/pickers/building_art_r1/`: the
   cards are Buildings.tsx's markup under the real `web/src/style.css` and fonts, the two
   theme token blocks scoped to a class so light and dark sit side by side; the art box is the
   only added markup. About 10 s for all 13.
4. `python …/base_view.py` (the "base grows" sketch), then `python …/picker.py` (the
   manifest). `contact.py` makes a review sheet of every raw draw in `out/`.

## Prompts

`prompts.json` holds one shared preamble (near-future lunar engineering, white and grey hulls,
orange accents, no text or people), one closing line (one building, reads as a thumbnail),
each style's `style`, `emphasis` (what it must never be: photograph, 3D render) and `framing`
lines, and one subject line per building, the same in every style. A new building is one
subject line; a new style is one block.

- `flat` is the poster prompt's first try, kept as a style: it came back as flat-painted
  true-color art, not a poster. The rewritten `poster` (strictly five inks, big crescent
  Earth, "never a painting", "the building takes the poster's palette") gives a real
  screenprint.
- Asking for a "fully transparent background (alpha)" works: every iso and low-poly draw came
  back RGBA with a clean alpha edge, no keying needed.

## Draw costs

28 draws, none failed, no usage limit hit: 37–69 s each (framed scenes 37–53 s, the
transparent cut-outs 52–69 s), about 4 minutes per 20 at 3 at once. Times per draw are
in `art/buildings/r1/raw/log.jsonl`.

## Findings

- **Consistency across buildings is good in every style** from one shared preamble: the white
  hull and orange accent carry through, so four buildings read as one base. The flip side:
  the preamble dominates, so painterly, flat and cutaway sit close together.
- **The 3:1 banner crop is the fragile one.** A 3:2 draw cropped to 3:1 keeps only the middle
  half of the height; the poster draws ran taller than asked and lose their tops. A banner
  style should be drawn for the banner (subject inside the middle band, checked per draw) or
  get a per-style crop offset.
- **At 96 px**, the cut-outs (iso, low-poly) and the poster read best: one silhouette on
  plain ground. Painterly, pixel and cutaway carry ground and sky that shrink to texture.
- **The isometric patches vary** in angle, patch shape and scale; tiled into a base view
  (`base_view.png`) they read as a base but don't line up. A base view needs each draw made
  against a fixed patch guide image (`codex exec -i`, AIComplete's frame route).
