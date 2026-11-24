"""Joins mock.mjs's shots (out/077_story_title_art_r2/shots/<name>__<w>-<theme>.png) into the
picker's images, art/pickers/story_title_art_r2/<name>.png: a Story mock side by side, the
1280 px window's light shot left and the 1920 px window's dark shot right, top-aligned; a
title the four left-column clips in a row (1280 light, 1920 light, 1280 dark, 1920 dark).
Run from the repo root after mock.mjs.
"""

import collections
import pathlib

from PIL import Image

REPO = pathlib.Path(__file__).resolve().parents[3]
SHOTS = REPO / "out" / "077_story_title_art_r2" / "shots"
PICK = REPO / "art" / "pickers" / "story_title_art_r2"
PICK.mkdir(parents=True, exist_ok=True)
SHOTS_OUT = pathlib.Path(__file__).resolve().parent / "shots"
DRIVE = pathlib.Path(r"C:\Code\gdrive-sync\helium-hustle\screenshots")
GAP = 16
GROUND = (138, 133, 124)
STORY = ["1280-light", "1920-dark"]
TITLE = ["1280-light", "1920-light", "1280-dark", "1920-dark"]

shots = collections.defaultdict(dict)
for f in SHOTS.glob("*.png"):
    name, tag = f.stem.split("__")
    shots[name][tag] = f
for name, tags in sorted(shots.items()):
    order = TITLE if name.startswith("title_") else STORY
    missing = [t for t in order if t not in tags]
    if missing:
        raise SystemExit(f"compose: {name} has no {', '.join(missing)} shot: run mock.mjs")
    ims = [Image.open(tags[t]).convert("RGB") for t in order]
    w = sum(i.size[0] for i in ims) + GAP * (len(ims) + 1)
    h = max(i.size[1] for i in ims) + 2 * GAP
    sheet = Image.new("RGB", (w, h), GROUND)
    x = GAP
    for i in ims:
        sheet.paste(i, (x, GAP))
        x += i.size[0] + GAP
    sheet.save(PICK / f"{name}.png", optimize=True)
    print(f"compose: {name}.png {sheet.size}")
    # The whole-window pages are the item's screenshots (docs/rulings/ui.md, Screenshots): a
    # JPEG in the results folder (committed, kept small) and the PNG in the shared folder.
    if name.startswith("page_"):
        SHOTS_OUT.mkdir(exist_ok=True)
        sheet.save(SHOTS_OUT / f"{name}.jpg", quality=84)
        if DRIVE.exists():
            sheet.save(DRIVE / f"077_{name}.png", optimize=True)
