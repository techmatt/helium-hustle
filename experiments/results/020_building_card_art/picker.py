"""Writes the picker art/pickers/building_art_set_r1/manifest.json over mock.mjs's card
images: a row per building showing its card (light and dark), then the redraw row
(`kind: many`, the same images as tiles). Run from the repo root last. Stops if an image
is missing.
"""

import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
PICK = REPO / "art" / "pickers" / "building_art_set_r1"
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))
buildings = json.loads((REPO / "content" / "buildings.json").read_text(encoding="utf-8"))
reused = set(P["reused"])

rows = []
tiles = []
for b in buildings:
    i, name = b["id"], b["name"]
    card = PICK / f"card_{i}.png"
    if not card.exists():
        raise SystemExit(f"picker: {card} missing: run mock.mjs")
    origin = "drawn in 019, reused" if i in reused else "drawn in 020"
    rows.append(
        {
            "id": i,
            "label": f"{name} on its card",
            "note": f"{origin}. Subject: {P['buildings'][i]}",
            "options": [{"key": "set", "image": f"card_{i}.png", "label": "Earthrise light and dark, 2x device pixels"}],
        }
    )
    tiles.append({"key": i, "image": f"card_{i}.png", "label": name})

rows.append(
    {
        "id": "redraw",
        "kind": "many",
        "label": "Which need a redraw?",
        "note": "Select the buildings to draw again (leave the rest unselected); add what's wrong in the notes. The same card images as the rows above.",
        "options": tiles,
    }
)
manifest = {
    "name": "building_art_set_r1",
    "title": "Building card art, the full set",
    "intro": "Every building in content/ in 019's cut34 style (painterly lunar concept art, its own 3/4 view, a small regolith patch, cut out), as the 96 px thumbnail on its card in the game, light and dark. The rows above are for review; the last row is the one to answer: flag the ones to redraw.",
    "rows": rows,
}
(PICK / "manifest.json").write_text(json.dumps(manifest, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
print(f"picker: {len(rows)} rows")
