"""Writes the picker art/pickers/building_art_r1/manifest.json over the sheets mock.mjs made.

Run from the repo root after mock.mjs. Stops if a sheet is missing.
"""

import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
PICK = REPO / "art" / "pickers" / "building_art_r1"
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))

NOTES = {
    "iso": "City-builder sprite on its own regolith patch, cut out. Reads best small; the patches could later populate a 'base grows' view.",
    "poster": "Mid-century screenprint, five flat inks, big crescent Earth. Most graphic; its own backdrop, same in both themes.",
    "flat": "The poster prompt's first try: flat-painted shapes in true colors, softer than the screenprint.",
    "painterly": "Lunar concept art: black sky, hard sun, long shadows, Earth on the horizon.",
    "lowpoly": "Toy-like flat-shaded facets on a regolith disc, cut out. Close to Isometric, softer and rounder.",
    "pixel": "Detailed modern pixel art, framed. The most 'game' of the set.",
    "cutaway": "Ink-and-wash technical cutaway: the building opened up, the ground in cross-section.",
}

styles = {
    "id": "styles",
    "kind": "many",
    "label": "Which styles go forward?",
    "note": "Keep any number. Each sheet: the four uncropped draws, then the cards in light (left) and dark (right), banner and thumbnail; cut-out styles also on a theme-colored lunar plate.",
    "options": [
        {"key": k, "image": f"style_{k}.png", "label": f"{st['label'].split(' (')[0]}: {NOTES[k]}"}
        for k, st in P["styles"].items()
    ],
}
placement = {
    "id": "placement",
    "label": "Where does the art sit on the card?",
    "note": "Each sheet shows a framed style (Painterly) and a cut-out (Isometric). 'none' is today's card.",
    "options": [
        {"key": "none", "image": "no_art.png", "label": "No art (today)"},
        {"key": "banner", "image": "placement_banner.png", "label": "Banner across the top, 3:1, full card width"},
        {"key": "thumb", "image": "placement_thumb.png", "label": "96 px thumbnail under the title, beside the description and flows"},
    ],
}
framing = {
    "id": "framing",
    "label": "Framed scene or cut-out?",
    "note": "A framed scene brings its own backdrop and looks the same in both themes; a cut-out sits on the card itself, or on a horizon plate drawn in the theme's colors.",
    "options": [
        {"key": "none", "image": "no_art.png", "label": "No art (today)"},
        {"key": "framed", "image": "framing_framed.png", "label": "Framed scene (Painterly)"},
        {"key": "cut", "image": "framing_cut.png", "label": "Cut-out on the card color (Isometric)"},
        {"key": "plate", "image": "framing_plate.png", "label": "Cut-out on a theme-colored lunar plate (Isometric)"},
    ],
}
base_view = {
    "id": "base_view",
    "kind": "many",
    "label": "Worth pursuing: the same isometric art as a 'base grows' view?",
    "note": "A sketch only: the card cut-outs, unchanged, tiled on an iso grid over each theme's horizon colors. Keep it to put a base view on the list; a real one needs every patch drawn to one fixed template (these vary in angle and shape).",
    "options": [{"key": "base_view", "image": "base_view.png", "label": "Isometric cut-outs as a growing base (light, dark)"}],
}
manifest = {
    "name": "building_art_r1",
    "title": "Building art, round 1: style, placement, framing",
    "intro": "Seven styles drawing the same four buildings (Solar Panel, Regolith Excavator, Smelter, Data Center), each mocked onto the real Buildings-panel cards in Earthrise light and dark. Pick the styles to take forward (any number), then a placement and a framing if the mocks settle them; notes welcome on any tile.",
    "rows": [styles, placement, framing, base_view],
}
for row in manifest["rows"]:
    for o in row["options"]:
        if not (PICK / o["image"]).exists():
            raise SystemExit(f"picker: {o['image']} missing: run mock.mjs first")
(PICK / "manifest.json").write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
print(PICK / "manifest.json")
