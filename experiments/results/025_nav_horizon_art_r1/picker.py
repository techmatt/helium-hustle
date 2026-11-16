"""Writes the picker art/pickers/nav_horizon_art_r1/manifest.json over mock.mjs's images: a
`many` row per nav tile (content/nav.json order), one for the horizon and one for its
progress stages, the current placeholder first in each. Run from the repo root last. Stops
if an image is missing.
"""

import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
PICK = REPO / "art" / "pickers" / "nav_horizon_art_r1"
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))
nav = json.loads((REPO / "content" / "nav.json").read_text(encoding="utf-8"))
CROPS = {"buildings": "Regolith Excavator", "commands": "Data Center", "launch_pads": "Launch Pad", "research": "Research Lab", "projects": "Fabricator"}

LABELS = {
    "current": "current: the colored square",
    "crop": "card art crop",
    "paint": "painterly icon",
    "glyph": "flat glyph, tile color",
    "chip": "the square with a glyph",
}


def need(name):
    if not (PICK / name).exists():
        raise SystemExit(f"picker: {PICK / name} missing: run mock.mjs")
    return name


rows = []
for t in nav:
    i, name = t["id"], t["name"]
    opts = []
    for k in ["current", "crop", "paint", "glyph", "chip"]:
        if k == "crop" and i not in CROPS:
            continue
        label = LABELS[k] + (f" ({CROPS[i]})" if k == "crop" else "")
        opts.append({"key": k, "image": need(f"nav_{i}_{k}.png"), "label": label})
    rows.append(
        {
            "id": f"nav_{i}",
            "kind": "many",
            "label": f"{name} tile: which go forward?",
            "note": f"Each image: the icon enlarged (96 px), then the tile as it renders at 1280 px (82 px wide, 34 px icon; normal and selected) and compact (18 px icon), light and dark, 2x device pixels. Painterly subject: {P['icons'][i]}",
            "options": opts,
        }
    )

rows.append(
    {
        "id": "horizon",
        "kind": "many",
        "label": "Lunar horizon under Resources: which go forward?",
        "note": "The foot of the left column at 1280 px (270 px wide) and 1920 px (346 px), light above, dark below, 2x device pixels. The painted strip is cut at 2.5:1 and shown cover, bottom-aligned, so a narrow column loses a little of each side.",
        "options": [
            {"key": "current", "image": need("horizon_current.png"), "label": "current: flat vector with habitat"},
            {"key": "cutout", "image": need("horizon_horizon-cutout.png"), "label": "painted, transparent sky (the panel shows above)"},
            {"key": "earthrise", "image": need("horizon_horizon-earthrise.png"), "label": "painted edge to edge, black sky and Earth"},
            {"key": "earthlight", "image": need("horizon_horizon-earthlight.png"), "label": "night under earthlight, transparent sky"},
        ],
    }
)
rows.append(
    {
        "id": "horizon_progress",
        "kind": "many",
        "label": "Horizon showing progress (on the transparent-sky strip): which stages go forward?",
        "note": "Each later stage is an edit of the one before (the earlier draw attached), so the base stays put. Stages: bare habitat; plus a launch pad and two extractors; a rocket lifting off.",
        "options": [
            {"key": "current", "image": need("horizon_current.png"), "label": "current: one fixed picture"},
            {"key": "bare", "image": need("horizon_horizon-cutout.png"), "label": "bare habitat"},
            {"key": "pad", "image": need("horizon_progress-pad-cutout.png"), "label": "plus launch pad and extractors"},
            {"key": "liftoff", "image": need("horizon_progress-liftoff-cutout.png"), "label": "a rocket lifting off"},
        ],
    }
)
manifest = {
    "name": "nav_horizon_art_r1",
    "title": "Nav tile icons and the lunar horizon, round 1",
    "intro": "Candidates only: nothing is in the UI yet. Per nav tile, select every candidate worth taking forward (several is fine): a crop of the building card art (where the tile has a building subject), a small painterly icon in the card-art style, a flat glyph in the tile's color, and the tile's colored square with the glyph knocked out. Then the painted horizon strip under Resources, and which progress stages it should show.",
    "rows": rows,
}
(PICK / "manifest.json").write_text(json.dumps(manifest, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
print(f"picker: {len(rows)} rows")
