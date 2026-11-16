"""Writes the picker art/pickers/building_art_r2/manifest.json over what sheets.py and
mock.mjs made, plus the two thumbnail-crop images (a variant's light and dark strips
stacked). Run from the repo root last. Stops if an image is missing.
"""

import json
import pathlib

from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
PICK = REPO / "art" / "pickers" / "building_art_r2"
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))
V = P["variants"]
CROP_VARIANT = "loose"


def stack(names, out):
    ims = [Image.open(PICK / f"{n}.png").convert("RGB") for n in names]
    sheet = Image.new("RGB", (max(i.width for i in ims), sum(i.height for i in ims) + 16 * (len(ims) - 1)), "#8a857c")
    y = 0
    for im in ims:
        sheet.paste(im, (0, y))
        y += im.height + 16
    sheet.save(PICK / out, optimize=True)


stack([f"cards_{CROP_VARIANT}_light", f"cards_{CROP_VARIANT}_dark"], "thumbcrop_whole.png")
stack([f"cards_{CROP_VARIANT}_tight_light", f"cards_{CROP_VARIANT}_tight_dark"], "thumbcrop_tight.png")


def opts(fmt, label=lambda k, v: f"{v['label']}: {v['note']}"):
    return [{"key": k, "image": fmt.format(k), "label": label(k, v)} for k, v in V.items()]


short = lambda k, v: v["label"]  # noqa: E731
rows = [
    {
        "id": "variants",
        "kind": "many",
        "label": "Which variants go forward?",
        "note": "The four cut-outs of each, large, on the light card color. Three draw against one fixed 2:1 patch guide with one sun (low, from the left; shadows fall right); the control is 018's painterly at its natural 3/4 angle, cut out.",
        "options": opts("cutouts_{}.png"),
    },
    {
        "id": "cards_light",
        "kind": "many",
        "label": "On cards, light theme: which read at 96 px?",
        "note": "Solar Panel and Smelter, the art as the 96 px thumbnail, cut out on the card color. Shot at 2x device pixels, as on a high-DPI screen.",
        "options": opts("cards_{}_light.png", short),
    },
    {
        "id": "cards_dark",
        "kind": "many",
        "label": "On cards, dark theme: which read at 96 px?",
        "note": "Regolith Excavator and Data Center, the same way.",
        "options": opts("cards_{}_dark.png", short),
    },
    {
        "id": "base_light",
        "kind": "many",
        "label": "As a base, light theme: which compose?",
        "note": "The same images, untouched, placed on a 2:1 grid by where each patch landed: two panels, an excavator, a smelter, a data center, over the theme's horizon colors. The control has no shared patch, so it is placed by its outer box.",
        "options": opts("base_{}_light.png", short),
    },
    {
        "id": "base_dark",
        "kind": "many",
        "label": "As a base, dark theme: which compose?",
        "options": opts("base_{}_dark.png", short),
    },
    {
        "id": "thumb_crop",
        "label": "On the card: the whole patch, or tight on the building?",
        "note": f"The same draws ({V[CROP_VARIANT]['label']}), light above, dark below. The whole patch keeps the base view's tile; tight cuts the patch's side corners and soil edge so the building fills the 96 px box.",
        "options": [
            {"key": "whole", "image": "thumbcrop_whole.png", "label": "Whole patch (the base tile as is)"},
            {"key": "tight", "image": "thumbcrop_tight.png", "label": "Tight square on the building"},
        ],
    },
]
manifest = {
    "name": "building_art_r2",
    "title": "Building art, round 2: painterly isometric",
    "intro": "Painterly lunar concept art (your r1 pick) locked to one isometric camera, one patch and one sun, so a single image serves the card thumbnail and a growing base. Three variants (loose and most painterly; legible; big patch with long shadows) and a control (r1's painterly at its natural 3/4, cut out), each shown as cut-outs, on cards in both themes, and as a small base. Keep any number per row; notes welcome.",
    "rows": rows,
}
for row in rows:
    for o in row["options"]:
        if not (PICK / o["image"]).exists():
            raise SystemExit(f"picker: {o['image']} missing: run sheets.py and mock.mjs first")
(PICK / "manifest.json").write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
print(PICK / "manifest.json")
