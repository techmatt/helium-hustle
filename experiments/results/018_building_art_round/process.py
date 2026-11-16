"""Cuts the raw draws in art/buildings/r1/raw/ into what the card mocks use.

A framed style (a 3:2 scene) gives a 3:1 banner (the full width, centered) and a square
thumb (side 80% of the height, centered). A cut-out style (alpha) is trimmed to its opaque
box plus a small margin. Output: art/buildings/r1/<style>/<building>_<banner|thumb|cut>.png.
Run from the repo root: python experiments/results/018_building_art_round/process.py
"""

import json
import pathlib

from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
R1 = REPO / "art" / "buildings" / "r1"
RAW = R1 / "raw"
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))


def center(im, shape, frac_h=1.0):
    """The largest box of `shape` (w/h) inside the centered `frac_h` of the height."""
    w, h = im.size
    bh = min(h * frac_h, w / shape)
    bw = bh * shape
    left, top = (w - bw) / 2, (h - bh) / 2
    return im.crop((round(left), round(top), round(left + bw), round(top + bh)))


def main():
    done = 0
    for s, st in P["styles"].items():
        out = R1 / s
        out.mkdir(parents=True, exist_ok=True)
        for b in P["buildings"]:
            src = RAW / f"{s}-{b}.png"
            if not src.exists():
                print(f"process: {s}-{b}: no draw yet")
                continue
            im = Image.open(src)
            if st["cut"]:
                im = im.convert("RGBA")
                alpha = im.getchannel("A").point(lambda v: 255 if v > 24 else 0)
                box = alpha.getbbox()
                if box is None or im.getchannel("A").getextrema()[0] > 0:
                    print(f"process: {s}-{b}: no transparency, kept whole")
                    box = (0, 0, *im.size)
                im = im.crop(box)
                m = round(max(im.size) * 0.04)
                pad = Image.new("RGBA", (im.width + 2 * m, im.height + 2 * m), (0, 0, 0, 0))
                pad.paste(im, (m, m))
                pad.thumbnail((600, 600), Image.LANCZOS)
                pad.save(out / f"{b}_cut.png", optimize=True)
            else:
                im = im.convert("RGB")
                center(im, 3.0).resize((900, 300), Image.LANCZOS).save(out / f"{b}_banner.png", optimize=True)
                center(im, 1.0, 0.8).resize((288, 288), Image.LANCZOS).save(out / f"{b}_thumb.png", optimize=True)
            done += 1
    print(f"process: {done} cut")


main()
