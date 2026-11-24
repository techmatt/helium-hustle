"""Trims each painted title draw (art/story_title/raw/title-*.png, real alpha from the tool) to
its opaque box, a 1% alpha threshold, into art/story_title/title/<style>.png, and trims
letterbox bars off the act headers (below), which mock.mjs then shows `cover` at the focus in
crops.json. Run from the repo root after gen.mjs.
"""

import pathlib

from PIL import Image

REPO = pathlib.Path(__file__).resolve().parents[3]
RAW = REPO / "art" / "story_title" / "raw"
OUT = REPO / "art" / "story_title" / "title"
OUT.mkdir(parents=True, exist_ok=True)

for f in sorted(RAW.glob("title-*.png")):
    if ".try" in f.name:
        continue
    im = Image.open(f).convert("RGBA")
    box = im.getchannel("A").point(lambda a: 255 if a > 3 else 0).getbbox()
    if box is None:
        raise SystemExit(f"process: {f.name} is fully transparent")
    cut = im.crop(box)
    cut.save(OUT / f"{f.stem[len('title-'):]}.png")
    print(f"process: {f.name} {im.size} -> {cut.size} ({cut.size[0] / cut.size[1]:.1f}:1)")

# Act headers: some draws came back letterboxed (a ~3:1 picture between pure black bars in the
# 3:2 canvas), which `cover` would show as bars. Trim top and bottom rows that are pure black
# (no pixel above 3 of 255; painted sky always carries brush noise) when they run to 5% of
# the height or more, into art/story_title/header/<id>.png; others are copied as drawn.
HEAD = REPO / "art" / "story_title" / "header"
HEAD.mkdir(parents=True, exist_ok=True)
# The first vignette round filled the band instead of staying a small emblem; kept as a
# fourth framing, `fill` (act<N>-vignette.try1.png -> header/act<N>-fill.png).
for f in sorted(RAW.glob("act*.png")):
    name = f.name
    if name.endswith("-vignette.try1.png"):
        name = name.replace("-vignette.try1.png", "-fill.png")
    elif ".try" in name:
        continue
    im = Image.open(f).convert("RGB")
    grey = im.convert("L")
    w, h = im.size
    lit = [grey.crop((0, y, w, y + 1)).getextrema()[1] > 3 for y in range(h)]
    if not any(lit):
        raise SystemExit(f"process: {f.name} is all black")
    top = lit.index(True)
    bottom = h - lit[::-1].index(True)
    if top < h * 0.05:
        top = 0
    if h - bottom < h * 0.05:
        bottom = h
    cut = im.crop((0, top, w, bottom))
    cut.save(HEAD / name)
    print(f"process: {f.name} -> {name} {im.size} -> {cut.size}" + (" (letterbox trimmed)" if cut.size != im.size else ""))
