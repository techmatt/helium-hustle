"""078: writes the shipped Story art into the web client, from the picked painted files.

- Act bands (picker story_title_art_r2, Matt's final calls): Act 1 is round 2's `side3`
  draw (art/story_title_r2/raw/act1-side3.png); Acts 2-4 are round 1's picked `side`
  headers (art/story_title/header/act<N>-side.png). Each is cropped to the band's 3:1 at
  round 1's vertical focus (076's crops.json, 50% where it names none, as 077's mocks
  showed them) and saved as web/src/assets/story/act<N>.webp.
- Quest pictures: 1.1's picture is the left column's lunar-base scene, so
  web/src/assets/quests/lunar_base.webp is a byte copy of web/src/assets/horizon/bare.webp.

Run from the repo root (Python with Pillow). Every source is checked before anything is
written.
"""

import json
import pathlib
import shutil

from PIL import Image

REPO = pathlib.Path(__file__).resolve().parents[3]
ART = REPO / "art"
WEB = REPO / "web" / "src" / "assets"
CROPS = json.loads((REPO / "experiments" / "results" / "076_story_header_and_title_art_r1" / "crops.json").read_text(encoding="utf-8"))
BAND_W = 1536  # the draws' own width; the middle column is at most ~1060 css px wide
BANDS = {
    1: (ART / "story_title_r2" / "raw" / "act1-side3.png", 50),
    **{n: (ART / "story_title" / "header" / f"act{n}-side.png", CROPS.get(f"act{n}-side", 50)) for n in (2, 3, 4)},
}
BASE = WEB / "horizon" / "bare.webp"

for f, _ in BANDS.values():
    if not f.exists():
        raise SystemExit(f"process: {f} missing")
if not BASE.exists():
    raise SystemExit(f"process: {BASE} missing")

(WEB / "story").mkdir(exist_ok=True)
(WEB / "quests").mkdir(exist_ok=True)
for n, (f, focus) in BANDS.items():
    im = Image.open(f).convert("RGB")
    w, h = im.size
    bh = round(w / 3)
    top = round((h - bh) * focus / 100)
    band = im.crop((0, top, w, top + bh))
    if w > BAND_W:
        band = band.resize((BAND_W, round(bh * BAND_W / w)), Image.LANCZOS)
    out = WEB / "story" / f"act{n}.webp"
    band.save(out, quality=84, method=6)
    print(f"process: {out.relative_to(REPO)} {band.size} from {f.relative_to(REPO)} at {focus}%")
shutil.copyfile(BASE, WEB / "quests" / "lunar_base.webp")
print("process: web/src/assets/quests/lunar_base.webp (horizon/bare.webp)")
