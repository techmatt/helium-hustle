"""Cuts 025's draws and the card art into what mock.mjs shows.

Into the shared, uncommitted art/nav_horizon/:

- `icons/<tile>_paint.png`: the painterly icon draw (raw/icon-<tile>.png), trimmed to its
  opaque box, centered in a square with a small margin, 256 px.
- `icons/<tile>_crop.png`: for a tile with a natural building subject (CROPS), a tight
  crop of that building's card art cut-out (art/buildings/set/<id>_cut.png, 020): the
  building's span (020's rule: columns topping out above the patch band) and its height
  down to the patch, the patch faded out at the bottom, 256 px.
- `horizon/<id>.png`: a horizon draw cut to a strip ASPECT wide, at the band given in
  BANDS (the band's top as a fraction of the draw's height), 2x the widest strip it fills.

A draw with no transparency where one was asked (icons) has its light background keyed out
(020's key_white). Run from the repo root:
python experiments/results/025_nav_horizon_art_r1/process.py
"""

import importlib.util
import json
import pathlib
import sys

from PIL import Image, ImageChops

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
ART = REPO / "art" / "nav_horizon"
RAW = ART / "raw"
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))
SIDE = 256
MARGIN = 0.04  # of the icon's longer side, each side
# The left column renders 270-440 px wide (style.css --left), the strip 16 px wider
# (its -8 px margins), 150 px tall less 12 px top padding: 456 x 138 at the widest.
# The strip is cut at 2.5:1, between the 270 px column's 1.96:1 and the 440 px one's 3.3:1;
# mock.mjs shows it object-fit: cover, bottom-aligned, so a narrow column loses a little
# of each side and a wide one a little of the (empty) top.
STRIP = (690, 276)
ASPECT = STRIP[0] / STRIP[1]
# A nav tile and the building whose card art it crops; tiles not here have no crop.
CROPS = {
    "buildings": "excavator",
    "commands": "data_center",
    "launch_pads": "launch_pad",
    "research": "research_lab",
    "projects": "fabricator",
}
# Where the strip's band starts in a draw with a painted sky, as a fraction of its height
# (just above the base's mast). A draw with a transparent sky finds it: TOP_GAP above the
# first row with at least TOP_ROW of its pixels opaque.
BANDS = {"horizon-earthrise": 0.30}
TOP_ROW = 0.004
TOP_GAP = 0.03
PATCH_BAND = 0.30
FADE = 0.16

sys.dont_write_bytecode = True  # no __pycache__ left in 020's folder
spec = importlib.util.spec_from_file_location("p020", HERE.parent / "020_building_card_art" / "process.py")
p020 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(p020)


def square(im, margin=MARGIN):
    box = im.getchannel("A").point(lambda v: 255 if v > 24 else 0).getbbox()
    cut = im.crop(box)
    side = round(max(cut.size) * (1 + 2 * margin))
    sq = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    sq.paste(cut, ((side - cut.width) // 2, (side - cut.height) // 2))
    return sq.resize((SIDE, SIDE), Image.LANCZOS)


def crop(cut):
    """The building alone: its column span, from its top down to where the patch starts."""
    a = cut.getchannel("A")
    w, h = cut.size
    px = a.load()
    tops = [next((y for y in range(h) if px[x, y] > 64), h) for x in range(w)]
    tall = [x for x, t in enumerate(tops) if t < h * (1 - PATCH_BAND)]
    x0, x1 = tall[0], tall[-1]
    y0 = min(tops)
    y1 = round(h * 0.86)  # the building's foot; below it is the patch's front rim
    side = max(x1 - x0, y1 - y0)
    cx = (x0 + x1) / 2
    left, top = round(cx - side / 2), y1 - side
    sq = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    sq.paste(cut, (-left, -top))
    f = max(1, round(side * FADE))
    bot = Image.linear_gradient("L").resize((side, f)).transpose(Image.FLIP_TOP_BOTTOM)
    mask = Image.new("L", (side, side), 255)
    mask.paste(bot, (0, side - f))
    sq.putalpha(ImageChops.multiply(sq.getchannel("A"), mask))
    return sq.resize((SIDE, SIDE), Image.LANCZOS)


def band_top(im, base):
    if base in BANDS:
        return round(im.height * BANDS[base])
    a = im.getchannel("A").point(lambda v: 1 if v > 200 else 0)
    w, h = im.size
    row = next(y for y in range(h) if a.crop((0, y, w, y + 1)).histogram()[1] >= w * TOP_ROW)
    return max(0, row - round(h * TOP_GAP))


def strip(im, top):
    w, h = im.size
    bh = round(w / ASPECT)
    y0 = min(top, h - bh)
    return im.crop((0, y0, w, y0 + bh)).resize(STRIP, Image.LANCZOS)


def main():
    (ART / "icons").mkdir(parents=True, exist_ok=True)
    (ART / "horizon").mkdir(parents=True, exist_ok=True)
    for tile in P["icons"]:
        src = RAW / f"icon-{tile}.png"
        if not src.exists():
            print(f"process: {tile}: no painterly draw yet")
            continue
        im = Image.open(src).convert("RGBA")
        if im.getchannel("A").getextrema()[0] > 0:
            print(f"process: {tile}: no transparency, keyed the white out")
            im = p020.key_white(im)
        square(im).save(ART / "icons" / f"{tile}_paint.png", optimize=True)
    for tile, b in CROPS.items():
        cut = Image.open(REPO / "art" / "buildings" / "set" / f"{b}_cut.png").convert("RGBA")
        crop(cut).save(ART / "icons" / f"{tile}_crop.png", optimize=True)
    for src in sorted(RAW.glob("*.png")):
        if not src.stem.startswith(("horizon-", "progress-")) or ".try" in src.stem:
            continue
        # A progress stage is cut at its base draw's band, so the base stays put.
        base = src.stem if src.stem.startswith("horizon-") else "horizon-" + src.stem.split("-")[-1]
        im = Image.open(src).convert("RGBA")
        top = band_top(Image.open(RAW / f"{base}.png").convert("RGBA"), base)
        strip(im, top).save(ART / "horizon" / f"{src.stem}.png", optimize=True)
    print("process: done")


if __name__ == "__main__":
    main()
