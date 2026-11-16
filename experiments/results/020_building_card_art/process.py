"""Cuts the building card set's draws into what the game and the picker use.

Each building's original is art/buildings/set/raw/<id>.png, or for the four reused from
019 the path in prompts.json `reused`. A draw with no transparency (the tool ignored the
alpha ask) has its near-white background keyed out from the border (019's process.py).
Each gives:

- `art/buildings/set/<id>_cut.png`: trimmed to its opaque box plus a small margin, at most
  600 px: the review sheet and the picker's originals.
- `web/src/assets/buildings/<id>.webp`: the shipped card picture, the opaque box (the
  building on its patch, no margin) centered in a 192 px square (2x the card's 96 px box),
  transparent around it, WebP.

Also `out/020_building_card_art/sheet.png`, every cut-out on one grey sheet at the same
height per row, for a consistency check by eye. Run from the repo root:
python experiments/results/020_building_card_art/process.py
"""

import json
import pathlib

from PIL import Image, ImageChops, ImageDraw

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
SET = REPO / "art" / "buildings" / "set"
RAW = SET / "raw"
WEB = REPO / "web" / "src" / "assets" / "buildings"
OUT = REPO / "out" / "020_building_card_art"
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))
SIDE = 192
PATCH_BAND = 0.30  # a column topping out this low in the cut-out is ground only
RIM = 0.04  # of the cut-out's height cut off the patch's lower rim
MARGIN = 0.06  # each side of the building's span
FADE = 0.10  # of the square's side faded at the left, right and bottom
CARDS = {"light": (255, 255, 255), "dark": (35, 35, 41)}  # style.css --card


def key_white(im):
    """Alpha for a draw on a flat light background: near-white connected to the border."""
    rgb = im.convert("RGB")
    probe = rgb.copy()
    w, h = probe.size
    for x, y in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1), (w // 2, 0)]:
        if min(probe.getpixel((x, y))) > 200:
            ImageDraw.floodfill(probe, (x, y), (255, 0, 255), thresh=40)
    diff = ImageChops.difference(probe, Image.new("RGB", (w, h), (255, 0, 255))).convert("L")
    mask = diff.point(lambda v: 0 if v < 8 else 255)
    out = rgb.convert("RGBA")
    out.putalpha(mask)
    return out


def tight(cut):
    """A square on the building: the patch's far side lobes and lower rim cut off, faded.

    A column whose topmost opaque pixel sits in the bottom PATCH_BAND of the cut-out holds
    only ground; the building is the span of the other columns. The square is centered on
    that span, as wide as the building plus a margin and at least as tall as the cut-out
    less its rim, bottom-aligned; where it cuts into the patch the alpha fades out over
    FADE of its side, so no hard straight edge shows on the card.
    """
    a = cut.getchannel("A")
    w, h = cut.size
    px = a.load()
    tops = []
    for x in range(w):
        top = next((y for y in range(h) if px[x, y] > 64), h)
        tops.append(top)
    tall = [x for x, t in enumerate(tops) if t < h * (1 - PATCH_BAND)]
    x0, x1 = (tall[0], tall[-1]) if tall else (0, w - 1)
    bottom = round(h * (1 - RIM))
    side = max(round((x1 - x0) * (1 + 2 * MARGIN)), bottom)
    cx = (x0 + x1) / 2
    left, top = round(cx - side / 2), bottom - side
    sq = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    sq.paste(cut, (-left, -top))
    # Fade where the square cuts opaque ground: the left, right and bottom edges.
    f = max(1, round(side * FADE))
    ramp = Image.linear_gradient("L").rotate(90, expand=True).resize((f, side))  # 0 at left
    mask = Image.new("L", (side, side), 255)
    mask.paste(ramp, (0, 0))
    mask.paste(ramp.transpose(Image.FLIP_LEFT_RIGHT), (side - f, 0))
    bot = Image.linear_gradient("L").resize((side, f)).transpose(Image.FLIP_TOP_BOTTOM)
    mask.paste(ImageChops.darker(mask.crop((0, side - f, side, side)), bot), (0, side - f))
    sq.putalpha(ImageChops.multiply(sq.getchannel("A"), mask))
    return sq


def source(b):
    return REPO / P["reused"][b] if b in P["reused"] else RAW / f"{b}.png"


def main():
    SET.mkdir(parents=True, exist_ok=True)
    WEB.mkdir(parents=True, exist_ok=True)
    OUT.mkdir(parents=True, exist_ok=True)
    cuts = {}
    thumbs = {}
    for b in P["buildings"]:
        src = source(b)
        if not src.exists():
            print(f"process: {b}: no draw yet")
            continue
        im = Image.open(src).convert("RGBA")
        if im.getchannel("A").getextrema()[0] > 0:
            print(f"process: {b}: no transparency, keyed the white out")
            im = key_white(im)
        box = im.getchannel("A").point(lambda x: 255 if x > 24 else 0).getbbox()
        cut = im.crop(box)
        sq = tight(cut).resize((SIDE, SIDE), Image.LANCZOS)
        sq.save(WEB / f"{b}.webp", "WEBP", quality=88, method=6)
        thumbs[b] = sq
        m = round(max(cut.size) * 0.04)
        pad = Image.new("RGBA", (cut.width + 2 * m, cut.height + 2 * m), (0, 0, 0, 0))
        pad.paste(cut, (m, m))
        pad.thumbnail((600, 600), Image.LANCZOS)
        pad.save(SET / f"{b}_cut.png", optimize=True)
        cuts[b] = pad
    # The review sheet: four per row, each fitted into a 300 px square.
    n = len(cuts)
    cols = 4
    rows = (n + cols - 1) // cols
    sheet = Image.new("RGB", (cols * 310, rows * 310), (128, 126, 122))
    for i, (b, im) in enumerate(cuts.items()):
        t = im.copy()
        t.thumbnail((300, 300), Image.LANCZOS)
        x, y = (i % cols) * 310 + (310 - t.width) // 2, (i // cols) * 310 + (310 - t.height) // 2
        sheet.paste(t, (x, y), t)
    sheet.save(OUT / "sheet.png")
    # The shipped pictures at 1x (96 px) on each theme's card color, eight per row.
    for theme, color in CARDS.items():
        rows = (len(thumbs) + 7) // 8
        th = Image.new("RGB", (8 * 104 + 8, rows * 104 + 8), color)
        for i, im in enumerate(thumbs.values()):
            t = im.resize((96, 96), Image.LANCZOS)
            th.paste(t, (8 + (i % 8) * 104, 8 + (i // 8) * 104), t)
        th.save(OUT / f"thumbs_{theme}.png")
    print(f"process: {n} cut")


if __name__ == "__main__":
    main()
