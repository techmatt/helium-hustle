"""Readies 077's draws (art/story_title_r2/raw/) for the mock, which injects them into the
running game page as data URLs, so each goes out as a WebP no wider than 2000 px into
art/story_title_r2/web/<id>.webp:

- wide and tall draws: letterbox bars trimmed (076's rule: top and bottom rows that are
  pure black, no pixel above 3 of 255, when they run to 5% of the height or more);
- spine draws: the middle 14% of the width, full height, a strip the mock repeats down the
  timeline's spine.

Round 1's picked `side` headers (art/story_title/header/act<N>-side.png) go along as
act<N>-side.webp, the row's carried-forward framing. Run from the repo root after gen.mjs.
"""

import json
import pathlib

from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
RAW = REPO / "art" / "story_title_r2" / "raw"
OUT = REPO / "art" / "story_title_r2" / "web"
OUT.mkdir(parents=True, exist_ok=True)
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))
MAXW = 2000


def trim_letterbox(im):
    grey = im.convert("L")
    w, h = im.size
    lit = [grey.crop((0, y, w, y + 1)).getextrema()[1] > 3 for y in range(h)]
    if not any(lit):
        raise SystemExit("process: an all-black draw")
    top = lit.index(True)
    bottom = h - lit[::-1].index(True)
    if top < h * 0.05:
        top = 0
    if h - bottom < h * 0.05:
        bottom = h
    return im.crop((0, top, w, bottom))


def save(im, name):
    if im.size[0] > MAXW:
        im = im.resize((MAXW, round(im.size[1] * MAXW / im.size[0])), Image.LANCZOS)
    im.save(OUT / f"{name}.webp", quality=86, method=6)
    print(f"process: {name}.webp {im.size}")


for act in range(1, 5):
    f = REPO / "art" / "story_title" / "header" / f"act{act}-side.png"
    if not f.exists():
        raise SystemExit(f"process: round 1's {f} missing")
    save(Image.open(f).convert("RGB"), f"act{act}-side")

for id_, (framing, _scene) in P["draws"].items():
    f = RAW / f"{id_}.png"
    if not f.exists():
        print(f"process: {id_} not drawn, skipped")
        continue
    im = Image.open(f).convert("RGB")
    if framing == "spine":
        w, h = im.size
        cut = im.crop((round(w * 0.43), 0, round(w * 0.57), h))
    else:
        cut = trim_letterbox(im)
    save(cut, id_)
