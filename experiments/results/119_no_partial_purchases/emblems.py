"""119 (amendment #3518): ships Matt's ideology emblem picks (picker ideology_emblems_r1, item 118)
into the Ideologies view's emblem slot (item 117: 160 px wide, the axis card's full height, at
least 190 px).

Each pick's square original art/ideology_emblems_r1/raw/<prefix>-<key>.png is center-cropped to
4:5 (the panel 118's picker previewed) and resized to 320x400, 2x the slot's width, then saved as
web/src/assets/ideology/<axis>.webp, WebP quality 88 as the building and project pictures. The
view fills the slot with it (object-fit: cover), so a taller card trims its sides a little.

Run from the repo root (Python with Pillow and WebP). Every pick and source is checked before
anything is written.
"""

import json
import pathlib

from PIL import Image

REPO = pathlib.Path(__file__).resolve().parents[3]
ART = REPO / "art"
SRC = ART / "ideology_emblems_r1" / "raw"
MANIFEST = ART / "pickers" / "ideology_emblems_r1" / "manifest.json"
WEB = REPO / "web" / "src" / "assets" / "ideology"

# axis -> picked option key (Matt, amendment #3518).
PICKS = {"nationalist": "marker", "humanist": "garden", "rationalist": "archive"}
SIZE = (320, 400)


def main():
    rows = {r["id"]: r for r in json.loads(MANIFEST.read_text(encoding="utf-8"))["rows"]}
    plan = []
    for axis, key in PICKS.items():
        row = rows.get(axis)
        if row is None:
            raise SystemExit(f"emblems: no row {axis} in {MANIFEST}")
        opt = next((o for o in row["options"] if o["key"] == key), None)
        if opt is None:
            raise SystemExit(f"emblems: no option {key} in row {axis}")
        src = SRC / opt["image"]
        if not src.exists():
            raise SystemExit(f"emblems: {src} missing")
        im = Image.open(src).convert("RGB")
        w, h = im.size
        if w != h:
            raise SystemExit(f"emblems: {src} is {im.size}, not square")
        cw = w * 4 // 5
        left = (w - cw) // 2
        plan.append((axis, im.crop((left, 0, left + cw, h)).resize(SIZE, Image.LANCZOS)))
    WEB.mkdir(parents=True, exist_ok=True)
    total = 0
    for axis, im in plan:
        out = WEB / f"{axis}.webp"
        im.save(out, "WEBP", quality=88, method=6)
        total += out.stat().st_size
    print(f"emblems: {len(plan)} pictures, {total // 1024} KB")


if __name__ == "__main__":
    main()
