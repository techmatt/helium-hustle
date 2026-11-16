"""The fixed patch guides every isometric draw is made against (`codex exec -i`).

A 1024 px square: flat white background (the part the draw leaves transparent), one 2:1
isometric diamond (the patch's top face, mid grey) on a slab of soil (its two visible sides,
darker on the right, away from the sun). No labels: labels invite text. Two sizes, the
patch size being one of the round's axes. The geometry is also what process.py and
base_view.py use to place a draw: a draw that honors the guide lands its patch here.

Run from the repo root: python experiments/results/019_painterly_iso_round/guide.py
Writes art/buildings/r2/guides/<name>.png.
"""

import json
import pathlib

from PIL import Image, ImageDraw

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
OUT = REPO / "art" / "buildings" / "r2" / "guides"
GUIDES = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))["guides"]
SIZE = 1024


def diamond(g):
    """The patch's top-face corners (left, top, right, bottom) for a guide."""
    cx, cy, w = SIZE / 2, g["center_y"], g["width"]
    return [(cx - w / 2, cy), (cx, cy - w / 4), (cx + w / 2, cy), (cx, cy + w / 4)]


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    for name, g in GUIDES.items():
        im = Image.new("RGB", (SIZE, SIZE), "white")
        d = ImageDraw.Draw(im)
        left, top, right, bottom = diamond(g)
        t = g["slab"]
        # The two visible sides of the slab: lit (left, toward the sun) and shaded (right).
        d.polygon([left, bottom, (bottom[0], bottom[1] + t), (left[0], left[1] + t)], fill="#7a6a58")
        d.polygon([bottom, right, (right[0], right[1] + t), (bottom[0], bottom[1] + t)], fill="#4a3f34")
        d.polygon([left, top, right, bottom], fill="#a3a3a3")
        im.save(OUT / f"{name}.png", optimize=True)
        print(OUT / f"{name}.png")


if __name__ == "__main__":
    main()
