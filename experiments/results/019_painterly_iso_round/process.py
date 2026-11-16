"""Cuts round 2's raw draws (art/buildings/r2/raw/) into what the mocks use.

Every draw is a cut-out. A draw with no transparency (the tool ignored the alpha ask) has
its background keyed out: a flood fill of near-white from the image's border. Each gives:

- `<building>_cut.png`: trimmed to its opaque box plus a small margin, at most 600 px: the
  card thumbnail and the cut-out review sheet.
- `patch.json`: where each draw's patch actually landed, measured from the alpha: its left
  and right corners (the outermost opaque columns in the patch band) and its lowest opaque
  row, in raw pixels, beside the guide's own numbers. base_view.py places draws by it, and
  the spread across a variant's four is how well the guide held.

Output: art/buildings/r2/<variant>/. Run from the repo root:
python experiments/results/019_painterly_iso_round/process.py
"""

import json
import pathlib

from PIL import Image, ImageChops, ImageDraw

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
R2 = REPO / "art" / "buildings" / "r2"
RAW = R2 / "raw"
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))


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


def measure(im, guide):
    """The patch's corners and foot as drawn, scaled to the guide's 1024 px frame."""
    a = im.getchannel("A").point(lambda v: 255 if v > 64 else 0)
    w, h = im.size
    s = 1024 / w
    box = a.getbbox()
    bottom = box[3]
    # The patch band: the rows from the foot up to the guide's diamond height.
    band_h = round(guide["width"] / 2 / s) if guide else round((box[3] - box[1]) * 0.35)
    band = a.crop((0, max(0, bottom - band_h), w, bottom)).getbbox()
    return {"left": round(band[0] * s), "right": round(band[2] * s), "bottom": round(bottom * s), "top": round(box[1] * s)}


def main():
    done = 0
    for v, st in P["variants"].items():
        out = R2 / v
        out.mkdir(parents=True, exist_ok=True)
        guide = P["guides"].get(st["guide"]) if st["guide"] else None
        patches = {}
        if guide:
            cx, cy, gw, t = 512, guide["center_y"], guide["width"], guide["slab"]
            patches["_guide"] = {"left": round(cx - gw / 2), "right": round(cx + gw / 2), "bottom": round(cy + gw / 4 + t)}
        for b in P["buildings"]:
            src = RAW / f"{v}-{b}.png"
            if not src.exists():
                print(f"process: {v}-{b}: no draw yet")
                continue
            im = Image.open(src).convert("RGBA")
            keyed = im.getchannel("A").getextrema()[0] > 0
            if keyed:
                print(f"process: {v}-{b}: no transparency, keyed the white out")
                im = key_white(im)
            patches[b] = {**measure(im, guide), "size": im.size[0], "keyed": keyed}
            box = im.getchannel("A").point(lambda x: 255 if x > 24 else 0).getbbox()
            cut = im.crop(box)
            m = round(max(cut.size) * 0.04)
            pad = Image.new("RGBA", (cut.width + 2 * m, cut.height + 2 * m), (0, 0, 0, 0))
            pad.paste(cut, (m, m))
            pad.thumbnail((600, 600), Image.LANCZOS)
            pad.save(out / f"{b}_cut.png", optimize=True)
            im.save(out / f"{b}_full.png", optimize=True)
            if guide:
                # Tight: a square on the building, the patch's side corners and slab cut
                # off, for a thumbnail where the building should be larger.
                k = im.size[0] / 1024
                pt = patches[b]
                cx = (pt["left"] + pt["right"]) / 2 * k
                bottom = (pt["bottom"] - guide["slab"]) * k
                side = max(bottom - pt["top"] * k + 12 * k, 0.62 * (pt["right"] - pt["left"]) * k)
                tight = im.crop((round(cx - side / 2), round(bottom - side), round(cx + side / 2), round(bottom)))
                tight.thumbnail((600, 600), Image.LANCZOS)
                tight.save(out / f"{b}_tight.png", optimize=True)
            done += 1
        (out / "patch.json").write_text(json.dumps(patches, indent=1) + "\n", encoding="utf-8", newline="\n")
    print(f"process: {done} cut")


if __name__ == "__main__":
    main()
