"""Round 2's cut-out sheets and base sketches, into art/pickers/building_art_r2/.

- `cutouts_<variant>.png`: the four cut-outs large (a 2 x 2 grid, 520 px cells) on the
  light card color, the way they sit on a card.
- `base_<variant>_<theme>.png`: a small base cluster, five buildings on a 2:1 isometric
  grid, back to front, over the theme's horizon colors (web/src/style.css --sky, --hz-far,
  --hz-near). Each draw is placed by where its patch actually landed (patch.json), scaled so
  the patch is one tile wide: the guided variants abut patch to patch; the control has no
  shared patch, so it is placed by its outer box, which is what the fixed camera fixes.

Run from the repo root after process.py:
python experiments/results/019_painterly_iso_round/sheets.py
"""

import json
import pathlib

from PIL import Image, ImageDraw

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
R2 = REPO / "art" / "buildings" / "r2"
PICK = REPO / "art" / "pickers" / "building_art_r2"
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))
IDS = list(P["buildings"])
CARD_LIGHT = "#ffffff"
# (sky, far horizon, near ground) per theme: style.css --sky, --hz-far, --hz-near.
THEMES = {"light": ("#e9e6df", "#dcd6cb", "#cbc3b5"), "dark": ("#111114", "#34343b", "#28282e")}
# A small early base: two panels, an excavator, a smelter, a data center; (column, row).
LAYOUT = [((0, 0), "solar_panel"), ((1, 0), "solar_panel"), ((2, 0), "data_center"), ((0, 1), "excavator"), ((1, 1), "smelter")]
TILE = 300
W, H = 1000, 640


def cutouts(v):
    cell, gap = 520, 16
    sheet = Image.new("RGBA", (2 * cell + 3 * gap, 2 * cell + 3 * gap), CARD_LIGHT)
    for i, b in enumerate(IDS):
        f = R2 / v / f"{b}_cut.png"
        if not f.exists():
            return None
        im = Image.open(f).convert("RGBA")
        im = im.resize((round(im.width * cell / max(im.size)), round(im.height * cell / max(im.size))), Image.LANCZOS)
        x = gap + (i % 2) * (cell + gap) + (cell - im.width) // 2
        y = gap + (i // 2) * (cell + gap) + (cell - im.height) // 2
        sheet.alpha_composite(im, (x, y))
    return sheet


def base(v, theme):
    st = P["variants"][v]
    guide = P["guides"].get(st["guide"]) if st["guide"] else None
    patches = json.loads((R2 / v / "patch.json").read_text(encoding="utf-8"))
    if any(b not in patches for _, b in LAYOUT):
        return None
    sky, far, near = THEMES[theme]
    im = Image.new("RGBA", (W, H), sky)
    d = ImageDraw.Draw(im)
    d.rectangle((0, int(H * 0.16), W, int(H * 0.24)), fill=far)
    d.rectangle((0, int(H * 0.24), W, H), fill=near)
    # Cell (c, r)'s diamond center on screen; the cluster centered.
    x0, y0 = W / 2 - TILE / 4, H * 0.47
    for (c, r), b in sorted(LAYOUT, key=lambda x: (x[0][0] + x[0][1], x[0][0])):
        pt = patches[b]
        full = Image.open(R2 / v / f"{b}_full.png").convert("RGBA").resize((1024, 1024), Image.LANCZOS)
        pw = pt["right"] - pt["left"]
        s = TILE / pw
        slab = guide["slab"] if guide else 0
        face_cy = pt["bottom"] - slab - pw / 4  # the patch's top-face center, frame px
        cx = x0 + (c - r) * TILE / 2
        cy = y0 + (c + r) * TILE / 4
        art = full.resize((round(1024 * s), round(1024 * s)), Image.LANCZOS)
        im.alpha_composite(art, (round(cx - (pt["left"] + pw / 2) * s), round(cy - face_cy * s)))
    return im


def main():
    PICK.mkdir(parents=True, exist_ok=True)
    for v in P["variants"]:
        sh = cutouts(v)
        if sh:
            sh.convert("RGB").save(PICK / f"cutouts_{v}.png", optimize=True)
            print(f"sheets: cutouts_{v}.png")
        for theme in THEMES:
            b = base(v, theme)
            if b:
                b.convert("RGB").save(PICK / f"base_{v}_{theme}.png", optimize=True)
                print(f"sheets: base_{v}_{theme}.png")


if __name__ == "__main__":
    main()
