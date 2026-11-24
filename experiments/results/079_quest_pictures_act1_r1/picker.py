"""Writes the picker art/pickers/quest_pictures_act1_r1/manifest.json over compose.py's sheets:
one `many` row per Act 1 quest (1.1-1.11, then the side quests 1.a-1.c), its options the kept
draws in captions.json, each labeled with its one-line caption. Nothing is in the game yet, so
there is no current option. Rows start with nothing selected. Run from the repo root last;
stops if a sheet is missing or a quest has no candidate.
"""

import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
PICK = REPO / "art" / "pickers" / "quest_pictures_act1_r1"
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))
CAP = json.loads((HERE / "captions.json").read_text(encoding="utf-8"))
EV = [q for q in json.loads((REPO / "content" / "events.json").read_text(encoding="utf-8"))
      if q.get("beat", "").startswith("1.")]

NOTE = ("Each image shows the picture at its three real sizes, 1x, on the dark theme: in the "
        "story modal above the quest's paragraph (516 px), at the foot of the left column where "
        "the lunar-base scene sits (287 px), and as the Story timeline's 72x48 thumbnail.")

rows = []
for q in EV:
    beat = q["beat"]
    ids = [i for i in CAP if P["draws"][i][0] == beat]
    if not ids:
        raise SystemExit(f"picker: {beat} has no candidate")
    opts = []
    for i in ids:
        if not (PICK / f"{i}.png").exists():
            raise SystemExit(f"picker: {PICK / i}.png missing: run compose.py")
        opts.append({"key": i.split("-", 1)[1], "image": f"{i}.png", "label": CAP[i]})
    kind = "side quest" if q["category"] == "side" else "main story"
    rows.append({"id": f"q{beat.replace('.', '_')}", "kind": "many",
                 "label": f"{q['title']} ({kind})", "note": NOTE, "options": opts})

manifest = {
    "name": "quest_pictures_act1_r1",
    "title": "Act 1 quest pictures, round 1",
    "intro": ("Candidates only: nothing is in the game yet. One row per Act 1 quest, the 11 "
              "main-story beats then the side quests 1.a-1.c, each with several painted "
              "candidates drawn from the quest's story and payoff text. Earth appears only where "
              "the text puts it. Select every candidate worth taking forward; several per row "
              "is fine, and a row with none sends that quest back for another draw."),
    "rows": rows,
}
(PICK / "manifest.json").write_text(json.dumps(manifest, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
print(f"picker: {len(rows)} rows, {sum(len(r['options']) for r in rows)} options")
