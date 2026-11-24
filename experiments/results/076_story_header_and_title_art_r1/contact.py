"""A review sheet into out/076_story_header_and_title_art_r1/contact.png: every act header as
the mock uses it (process.py's art/story_title/header/, letterbox trimmed, `fill` included)
with the 3:1 band the mock shows outlined at its crops.json focus, then every raw title draw
(earlier tries included). Run from the repo root after process.py.
"""

import json
import pathlib

from PIL import Image, ImageDraw

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
ART = REPO / "art" / "story_title"
OUT = REPO / "out" / "076_story_header_and_title_art_r1"
CROPS = json.loads((HERE / "crops.json").read_text(encoding="utf-8"))
W, H = 384, 256
BAND = 3.0

files = sorted((ART / "header").glob("act*.png")) + sorted((ART / "raw").glob("title-*.png"))
cols = 4
sheet = Image.new("RGB", (cols * (W + 8), ((len(files) + cols - 1) // cols) * (H + 24)), (110, 106, 100))
d = ImageDraw.Draw(sheet)
for n, f in enumerate(files):
    im = Image.open(f).convert("RGBA")
    im.thumbnail((W, H))
    x, y = (n % cols) * (W + 8), (n // cols) * (H + 24)
    sheet.paste(im, (x, y), im)
    if f.stem.startswith("act"):
        # `cover` into a 3:1 box: the image is scaled to fill, then the overflow axis is
        # positioned at the focus (y percent; x always 50%).
        if im.width / im.height < BAND:
            bh = im.width / BAND
            top = (im.height - bh) * CROPS.get(f.stem, 50) / 100
            d.rectangle([x, y + top, x + im.width - 1, y + top + bh], outline=(255, 200, 0))
        else:
            bw = im.height * BAND
            left = (im.width - bw) / 2
            d.rectangle([x + left, y, x + left + bw, y + im.height - 1], outline=(255, 200, 0))
    d.text((x + 2, y + H + 4), f.parent.name + "/" + f.stem, fill=(255, 255, 255))
OUT.mkdir(parents=True, exist_ok=True)
sheet.save(OUT / "contact.png")
print(f"contact: {len(files)} images")
