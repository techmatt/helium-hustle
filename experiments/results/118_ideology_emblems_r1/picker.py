"""Writes the picker art/pickers/ideology_emblems_r1/manifest.json over compose.py's sheets:
one row per ideology axis (Nationalist, Humanist, Rationalist), its options the kept draws in
captions.json, each labeled with its one-line caption. Nothing is in the game yet, so there is
no current option. Rows start with nothing selected. Run from the repo root last; stops if a
sheet is missing or an axis has no candidate.
"""

import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
PICK = REPO / "art" / "pickers" / "ideology_emblems_r1"
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))
CAP = json.loads((HERE / "captions.json").read_text(encoding="utf-8"))

NOTE = ("Each image shows the emblem at 1x on the dark theme as a 240x300 panel on the left edge "
        "of a mock 760x300 axis card (the square draw center-cropped to 4:5), with the full "
        "square draw beside it, the crop's edges marked. The card's text is a sample.")
LABEL = {"nationalist": "Nationalist (red)", "humanist": "Humanist (green)",
         "rationalist": "Rationalist (blue)"}

rows = []
for axis in P["axes"]:
    ids = [i for i in CAP if P["draws"][i][0] == axis]
    if not ids:
        raise SystemExit(f"picker: {axis} has no candidate")
    opts = []
    for i in ids:
        if not (PICK / f"{i}.png").exists():
            raise SystemExit(f"picker: {PICK / i}.png missing: run compose.py")
        opts.append({"key": i.split("-", 1)[1], "image": f"{i}.png", "label": CAP[i]})
    rows.append({"id": axis, "label": LABEL[axis], "note": NOTE, "options": opts})

manifest = {
    "name": "ideology_emblems_r1",
    "title": "Ideology emblems, round 1",
    "intro": ("Candidates only: nothing is in the game yet. One row per ideology axis, each with "
              "painted candidates in its chip color across a range of approaches: a heraldic "
              "emblem, a banner, and scenes from the axis's own content (its rank-5 project and "
              "the buildings it favors). Pick one per row, or none to send that axis back for "
              "another draw."),
    "rows": rows,
}
(PICK / "manifest.json").write_text(json.dumps(manifest, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
print(f"picker: {len(rows)} rows, {sum(len(r['options']) for r in rows)} options")
