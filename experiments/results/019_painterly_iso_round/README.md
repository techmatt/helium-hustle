# 019: building art, round 2: painterly isometric

> Archived (034): `art/pickers/` now lives at `C:\Code\gdrive-sync\helium-hustle\archive\art\pickers\`.

Matt's r1 picks (painterly lunar concept art, a 96 px thumbnail under the title, cut out on
the card color, a "base grows" view worth pursuing) tried as one set of images for both the
cards and a base view: painterly rendering locked to one 2:1 isometric camera, one patch
shape and one sun. The same four buildings as 018, three variants and a control, each shown
as cut-outs, on cards in both themes and as a small base. Picker `building_art_r2`.

## Recipe

From the repo root (Node 24, system Chrome, Python with Pillow; `npm ci` in `web/` for the
fonts; `playwright-core` in a folder named by `PW_DIR`, by default
`out/019_painterly_iso_round/tools/`). 018's route, with a guide image:

1. `python experiments/results/019_painterly_iso_round/guide.py`: the patch guides
   (`art/buildings/r2/guides/std.png`, `large.png`), a 1024 px square, white ground, one
   2:1 diamond on a soil slab lit left and shaded right. No labels. Geometry in
   `prompts.json` `guides`.
2. `node …/gen.mjs [<variant>-<building> …] [--pool=4]`: 018's Codex draw, the variant's
   guide on `codex exec -i`. Into `art/buildings/r2/raw/`, a redraw keeping the old one as
   `.try<n>.png`.
3. `python …/process.py`: per variant into `art/buildings/r2/<variant>/`: `_cut` (trimmed),
   `_full` (the whole square), `_tight` (a square on the building, guided variants only),
   and `patch.json`, where each patch actually landed.
4. `python …/sheets.py` (cut-out sheets, base sketches), `node …/mock.mjs` (card strips,
   about 10 s), then `python …/picker.py` (manifest), all into `art/pickers/building_art_r2/`.

## Prompts

`prompts.json`: 018's preamble (white hulls, orange trim now in every subject line too), one
shared painterly line, one sun line (single hard sun low from the left; left-facing walls lit,
shadows fall right), each variant's `touch` and `shadow`, and for the guided variants one
camera line: the attached picture is a placement guide, not a style reference; paint the patch
exactly there; the white is transparent.

## Findings

- **The guide holds the camera and the patch.** Every guided draw came back 1254 px RGBA
  with real alpha, and its patch where the guide put it: the clean-edged variants within
  about 5 px (of 1024) on all four corners and the foot (`patch.json`). So the base sketch
  places the untouched draws on a grid and they abut patch to patch, no hand fitting.
- **The fixed camera does pull toward 018's sprite look**, as feared. The first `loose` draw
  (a softer "painter's sketch" line) came back a clean 3D sprite; it took an explicit
  "oil or gouache, palette-knife marks, color shifting within each wall, ragged patch edge,
  absolutely not a clean 3D model" to get a painted read. `crisp` ("legible") drifted all the
  way to a polished render: it is the sprite end of the axis.
- **The painterly variant costs guide accuracy**: `loose` keeps the angle and the patch's
  center but paints the patch about 20% larger than the guide (corners 85–100 px of 1024
  outside it on each side) with a ragged edge. Placing by the measured patch (as the sketch
  does) still tiles it; a real base view would either do that per draw or redraw to size.
- **One sun holds**: lit left walls, shadows to the right or back-right in every draw.
  Shadow length is the weak axis: the first `long` set came back with medium shadows despite
  "nearly on the horizon"; the redraw with "at least three times as long as the building is
  tall" gives real long shadows (the first set kept as `.try1`). It also draws the buildings
  smaller on the bigger patch, and its tiles' ground tones differ enough to show seams.
- **The control loses the base**: at 3/4 each draw has its own ground blob and scale, so
  the cluster overlaps and floats. On a card it is the largest, most detailed read.
- **At 96 px** the whole patch makes the building small (about half the box); the tight
  crop fills it and still reads as painted ground. That is the `thumb_crop` row.

## Draw costs

21 draws (16 kept, one `loose` and four `long` redrawn after a prompt change), none failed,
no usage limit hit: 49–97 s each (guided 55–97 s, the unguided control 49–71 s), 14 at 4
at once in about 3.5 minutes. Times in
`art/buildings/r2/raw/log.jsonl`.
