"""Review sheets of 079's draws, one row per quest, into out/079_quest_pictures_act1_r1/contact/.
`python contact.py [<beat-prefix> …]`: with none, every beat; e.g. `1.1 1.2` for two rows.
Each draw at 360x240 with its id, then at 72x48 beside it (the timeline's size). Review only.
"""

import json
import pathlib
import sys

from PIL import Image, ImageDraw, ImageFont

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
RAW = REPO / "art" / "quest_act1_r1" / "raw"
OUT = REPO / "out" / "079_quest_pictures_act1_r1" / "contact"
OUT.mkdir(parents=True, exist_ok=True)
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))
F = ImageFont.truetype("C:/Windows/Fonts/segoeui.ttf", 14)

beats = {}
for id_, (beat, _e, _s) in P["draws"].items():
    beats.setdefault(beat, []).append(id_)
want = sys.argv[1:] or list(beats)
for beat in want:
    ids = [i for i in beats[beat] if (RAW / f"{i}.png").exists()]
    if not ids:
        continue
    sheet = Image.new("RGB", (len(ids) * 450, 270), "#16161a")
    d = ImageDraw.Draw(sheet)
    for n, id_ in enumerate(ids):
        im = Image.open(RAW / f"{id_}.png").convert("RGB")
        sheet.paste(im.resize((360, 240), Image.LANCZOS), (n * 450 + 5, 25))
        sheet.paste(im.resize((72, 48), Image.LANCZOS), (n * 450 + 370, 25))
        d.text((n * 450 + 5, 4), f"{id_} {im.size[0]}x{im.size[1]}", font=F, fill="#eeeae4")
    sheet.save(OUT / f"{beat}.png")
    print(f"contact: {beat}.png")
