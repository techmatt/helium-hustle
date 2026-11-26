"""Cuts the project candidates and writes the picker project_art_r1.

Per option art/projects_r1/raw/<project>-<option>.png gives:
- art/projects_r1/<project>-<option>_cut.png: trimmed cut-out, at most 600 px.
- art/projects_r1/<project>-<option>_192.png: the card picture as 020 ships building art
  (020 process.py's tight crop, 192 px square, 2x the card's 96 px box).
- art/pickers/project_art_r1/<project>-<option>.png: the picker image: the cut-out large on
  the dark card color, then the 96 px card picture at 1x on the dark and the light card.
And art/pickers/project_art_r1/manifest.json, plus out/097_project_art_r1/sheet.png.
Run from the repo root: python experiments/results/100_wire_project_art_and_research_glyphs/097_project_art_r1/picker.py
"""

import importlib.util
import json
import pathlib

from PIL import Image, ImageDraw

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[3]
spec = importlib.util.spec_from_file_location("p020", REPO / "experiments/results/020_building_card_art/process.py")
p020 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(p020)

RAW = REPO / "art" / "projects_r1" / "raw"
SET = REPO / "art" / "projects_r1"
PICK = REPO / "art" / "pickers" / "project_art_r1"
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))
DARK, LIGHT = p020.CARDS["dark"], p020.CARDS["light"]
BIG = 400


def cut_of(src):
    im = Image.open(src).convert("RGBA")
    if im.getchannel("A").getextrema()[0] > 0:
        print(f"picker: {src.name}: no transparency, keyed the white out")
        im = p020.key_white(im)
    box = im.getchannel("A").point(lambda x: 255 if x > 24 else 0).getbbox()
    return im.crop(box)


def compose(cut, thumb):
    w, h = BIG + 24 + 96 + 32 + 16, BIG + 32
    img = Image.new("RGB", (w, h), (24, 24, 28))
    d = ImageDraw.Draw(img)
    d.rectangle([8, 8, 8 + BIG + 16, 8 + BIG + 16], fill=DARK)
    t = cut.copy()
    t.thumbnail((BIG, BIG), Image.LANCZOS)
    img.paste(t, (16 + (BIG - t.width) // 2, 16 + (BIG - t.height) // 2), t)
    x = BIG + 40
    small = thumb.resize((96, 96), Image.LANCZOS)
    for i, color in enumerate((DARK, LIGHT)):
        y = 8 + i * 128
        d.rectangle([x - 8, y, x + 96 + 8, y + 96 + 16], fill=color)
        img.paste(small, (x, y + 8), small)
    return img


def main():
    PICK.mkdir(parents=True, exist_ok=True)
    rows, sheet_items = [], []
    for pid, pr in P["projects"].items():
        opts = []
        for key, o in pr["options"].items():
            src = RAW / f"{pid}-{key}.png"
            if not src.exists():
                print(f"picker: {pid}-{key}: no draw yet")
                continue
            cut = cut_of(src)
            if o.get("float"):  # no patch to crop or fade: the whole subject, centered
                side = round(max(cut.size) * 1.04)
                sq = Image.new("RGBA", (side, side), (0, 0, 0, 0))
                sq.paste(cut, ((side - cut.width) // 2, (side - cut.height) // 2))
                sq = sq.resize((p020.SIDE, p020.SIDE), Image.LANCZOS)
            else:
                sq = p020.tight(cut).resize((p020.SIDE, p020.SIDE), Image.LANCZOS)
            sq.save(SET / f"{pid}-{key}_192.png", optimize=True)
            m = round(max(cut.size) * 0.04)
            pad = Image.new("RGBA", (cut.width + 2 * m, cut.height + 2 * m), (0, 0, 0, 0))
            pad.paste(cut, (m, m))
            pad.thumbnail((600, 600), Image.LANCZOS)
            pad.save(SET / f"{pid}-{key}_cut.png", optimize=True)
            compose(pad, sq).save(PICK / f"{pid}-{key}.png", optimize=True)
            sheet_items.append(sq)
            opts.append({"key": key, "image": f"{pid}-{key}.png", "label": o["label"]})
        rows.append({"id": pid, "label": pr["name"], "options": opts})
    manifest = {
        "name": "project_art_r1",
        "title": "Project card art, round 1",
        "intro": (
            "Candidates only: nothing is in the game yet (projects have no art today). One row per project in "
            "content/projects.json, three painted candidates each, in the building cards' style (painterly cut-out "
            "at a natural 3/4 view on a small regolith patch, white and orange, sun from the left). Each image shows "
            "the cut-out large on the dark card color, then the picture as a building card shows it: the 96 px "
            "thumbnail at 1x on the dark and the light card. Pick one per row; a row left empty sends that project "
            "back for another draw."
        ),
        "rows": rows,
    }
    (PICK / "manifest.json").write_text(json.dumps(manifest, indent=1) + "\n", encoding="utf-8")
    cols = 9
    n = len(sheet_items)
    sheet = Image.new("RGB", (cols * 104 + 8, ((n + cols - 1) // cols) * 104 + 8), DARK)
    for i, im in enumerate(sheet_items):
        t = im.resize((96, 96), Image.LANCZOS)
        sheet.paste(t, (8 + (i % cols) * 104, 8 + (i // cols) * 104), t)
    scratch = REPO / "out" / "097_project_art_r1"
    scratch.mkdir(parents=True, exist_ok=True)
    sheet.save(scratch / "sheet.png")
    print(f"picker: {n} options in {len(rows)} rows")


if __name__ == "__main__":
    main()
