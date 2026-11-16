"""A contact sheet of every raw draw (rows: styles, columns: buildings) for review.

Run from the repo root; writes out/018_building_art_round/raw_contact.png.
"""

import json
import pathlib

from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
RAW = REPO / "art" / "buildings" / "r1" / "raw"
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))
W, H = 360, 240
styles, buildings = list(P["styles"]), list(P["buildings"])
sheet = Image.new("RGB", (W * len(buildings), H * len(styles)), (138, 133, 124))
for i, s in enumerate(styles):
    for j, b in enumerate(buildings):
        f = RAW / f"{s}-{b}.png"
        if not f.exists():
            continue
        im = Image.open(f).convert("RGBA")
        im.thumbnail((W, H))
        sheet.paste(im, (j * W + (W - im.width) // 2, i * H + (H - im.height) // 2), im)
out = REPO / "out" / "018_building_art_round" / "raw_contact.png"
sheet.save(out)
print(out)
