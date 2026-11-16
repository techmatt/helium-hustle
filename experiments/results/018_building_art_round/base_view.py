"""A sketch of the 'base grows' view: the isometric cut-outs tiled on an isometric grid,
back to front, over each theme's sky and horizon colors (web/src/style.css). Shows whether
the card art can double as the base view; nothing is redrawn.

Run from the repo root; writes art/pickers/building_art_r1/base_view.png.
"""

import pathlib

from PIL import Image, ImageDraw

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
CUT = REPO / "art" / "buildings" / "r1" / "iso"
OUT = REPO / "art" / "pickers" / "building_art_r1" / "base_view.png"

# (sky, far horizon, near ground) per theme: style.css --sky, --hz-far, --hz-near.
THEMES = {"light": ("#e9e6df", "#dcd6cb", "#cbc3b5"), "dark": ("#111114", "#34343b", "#28282e")}
# Grid cells (column, row) -> building; a small early base: three panels, two
# excavators, a smelter, a data center.
LAYOUT = [
    ((0, 0), "solar_panel"), ((1, 0), "solar_panel"), ((2, 0), "solar_panel"),
    ((0, 1), "excavator"), ((1, 1), "smelter"), ((2, 1), "data_center"),
    ((0, 2), "excavator"),
]
TILE = 190  # a patch's width on screen
W, H = 660, 520


def panel(theme):
    sky, far, near = THEMES[theme]
    im = Image.new("RGBA", (W, H), sky)
    d = ImageDraw.Draw(im)
    d.rectangle((0, int(H * 0.30), W, int(H * 0.40)), fill=far)
    d.rectangle((0, int(H * 0.40), W, H), fill=near)
    # Back to front: larger column + row is nearer the viewer.
    for (c, r), b in sorted(LAYOUT, key=lambda x: x[0][0] + x[0][1]):
        cut = Image.open(CUT / f"{b}_cut.png").convert("RGBA")
        cut = cut.resize((TILE, round(cut.height * TILE / cut.width)), Image.LANCZOS)
        x = W // 2 + (c - r) * TILE // 2 - TILE // 2
        y = 130 + (c + r) * TILE // 4 - (cut.height - TILE * 0.6)
        im.alpha_composite(cut, (int(x), int(y)))
    return im


sheet = Image.new("RGBA", (W * 2 + 12, H), "#8a857c")
sheet.alpha_composite(panel("light"), (0, 0))
sheet.alpha_composite(panel("dark"), (W + 12, 0))
sheet.convert("RGB").save(OUT, optimize=True)
print(OUT)
