"""A review sheet of every raw draw (art/story_title_r2/raw/, not earlier tries), each scaled to
a 300 px tall thumbnail with its id, into out/077_story_title_art_r2/contact.png. Run from the
repo root.
"""

import pathlib

from PIL import Image, ImageDraw

REPO = pathlib.Path(__file__).resolve().parents[3]
RAW = REPO / "art" / "story_title_r2" / "raw"
OUT = REPO / "out" / "077_story_title_art_r2" / "contact.png"
H = 300
files = [f for f in sorted(RAW.glob("*.png")) if ".try" not in f.name]
thumbs = []
for f in files:
    im = Image.open(f).convert("RGB")
    t = im.resize((round(im.size[0] * H / im.size[1]), H))
    ImageDraw.Draw(t).text((6, 6), f.stem, fill=(255, 210, 120))
    thumbs.append(t)
rows, row, width = [], [], 0
for t in thumbs:
    if width + t.size[0] > 1900 and row:
        rows.append(row)
        row, width = [], 0
    row.append(t)
    width += t.size[0] + 6
rows.append(row)
sheet = Image.new("RGB", (1900, len(rows) * (H + 6)), (60, 60, 60))
for y, r in enumerate(rows):
    x = 0
    for t in r:
        sheet.paste(t, (x, y * (H + 6)))
        x += t.size[0] + 6
sheet.save(OUT)
print(f"contact: {len(thumbs)} draws -> {OUT}")
