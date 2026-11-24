"""080: ships Matt's Act 1 quest-picture picks (picker quest_pictures_act1_r1) into the game.

For each pick, the painted original 079 drew (art/quest_act1_r1/raw/<row>-<key>.png,
1536x1024, the file behind the picker's composed sheet of the same name) is scaled to
768x512 (2x the modal's 360x240 slot, past the left column's 287 css px at 2x) and saved as
web/src/assets/quests/<stem>.webp, and the quest's row in content/events.json gets
"picture": "<stem>". 1.1's pick replaces 078's `lunar_base` stand-in, whose file goes.

Run from the repo root (Python with Pillow). Every source and quest row is checked before
anything is written. events.json is edited as text (one line per quest), keeping its layout.
"""

import json
import pathlib
import re

from PIL import Image

REPO = pathlib.Path(__file__).resolve().parents[3]
RAW = REPO / "art" / "quest_act1_r1" / "raw"
MANIFEST = REPO / "art" / "pickers" / "quest_pictures_act1_r1" / "manifest.json"
QUESTS = REPO / "web" / "src" / "assets" / "quests"
EVENTS = REPO / "content" / "events.json"
SIZE = (768, 512)

# beat -> (picker row, picked key); the stem in the game is "q<beat>_<key>" (1.10 -> q1_10_facing).
PICKS = {
    "1.1": ("q1_1", "earthrise"),
    "1.2": ("q1_2", "print"),
    "1.3": ("q1_3", "flight"),
    "1.4": ("q1_4", "refinery"),
    "1.5": ("q1_5", "tworacks"),
    "1.6": ("q1_6", "trails"),
    "1.7": ("q1_7", "ruts"),
    "1.8": ("q1_8", "inside"),
    "1.9": ("q1_9", "scales"),
    "1.10": ("q1_10", "facing"),
    "1.11": ("q1_11", "build"),
    "1.a": ("q1_a", "heavy"),
    "1.b": ("q1_b", "bench"),
    "1.c": ("q1_c", "wireframes"),
}

manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
rows = {r["id"]: {o["key"]: o["image"] for o in r["options"]} for r in manifest["rows"]}
events = json.loads(EVENTS.read_text(encoding="utf-8"))
by_beat = {e["beat"]: e for e in events if "beat" in e}
text = EVENTS.read_text(encoding="utf-8")

plan = []
for beat, (row, key) in PICKS.items():
    image = rows.get(row, {}).get(key)
    if image is None:
        raise SystemExit(f"process: picker row {row} has no option {key}")
    src = RAW / image  # the sheet's own name is the raw draw's name
    if not src.exists():
        raise SystemExit(f"process: {src} missing")
    q = by_beat.get(beat)
    if q is None:
        raise SystemExit(f"process: no quest with beat {beat}")
    stem = f"{row}_{key}"
    if not re.fullmatch(r"[a-z0-9_]+", stem):
        raise SystemExit(f"process: bad stem {stem}")
    plan.append((beat, src, q["id"], q.get("picture"), stem))

# events.json: each quest's id line is `"id": "<id>",`; set its picture after its "payoff"
# (or "body" for a side quest without one) by replacing an existing picture line or adding one.
for beat, _, qid, old, stem in plan:
    start = text.index(f'"id": "{qid}"')
    end = text.find('"id": "', start + 1)
    end = len(text) if end < 0 else end
    block = text[start:end]
    if old is not None:
        new_block, n = re.subn(r'"picture": "[a-z0-9_]+"', f'"picture": "{stem}"', block)
        if n != 1:
            raise SystemExit(f"process: {qid}: {n} picture lines")
    else:
        m = re.search(r'\n(\s*)"trigger":', block)
        if m is None:
            raise SystemExit(f"process: {qid}: no trigger line to put the picture before")
        new_block = block[: m.start()] + f'\n{m.group(1)}"picture": "{stem}",' + block[m.start():]
    text = text[:start] + new_block + text[end:]
check = {e["id"]: e.get("picture") for e in json.loads(text)}
for _, _, qid, _, stem in plan:
    if check[qid] != stem:
        raise SystemExit(f"process: {qid} would get {check[qid]}")

for beat, src, qid, _, stem in plan:
    im = Image.open(src).convert("RGB")
    if im.size[0] * 2 != im.size[1] * 3:
        raise SystemExit(f"process: {src} is {im.size}, not 3:2")
    out = QUESTS / f"{stem}.webp"
    im.resize(SIZE, Image.LANCZOS).save(out, quality=82, method=6)
    print(f"process: {beat} {qid}: {out.relative_to(REPO)} from {src.relative_to(REPO)} ({out.stat().st_size // 1024} KB)")
EVENTS.write_text(text, encoding="utf-8", newline="\n")
old = QUESTS / "lunar_base.webp"
if old.exists():
    old.unlink()
    print("process: removed web/src/assets/quests/lunar_base.webp (1.1's stand-in)")
