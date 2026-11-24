"""Writes the picker art/pickers/story_title_art_r1/manifest.json over mock.mjs's images: one
`many` row per act chapter header (`none` first, as the current one: the Story view has no
header art yet) and one for
the HELIUM HUSTLE title (the plain h1 first, as the current one). Run from the repo root last.
Stops if an image is missing.
"""

import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
PICK = REPO / "art" / "pickers" / "story_title_art_r1"
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))

FRAMING = {
    "none": "current: no art, the act title as plain text (the Story view has no header art yet)",
    "side": "subject at the right, dark open sky at the left for the title beside it",
    "wide": "panorama across the whole band, kept dark, the title laid over the center",
    "vignette": "one small central emblem, dark to both sides, the title under it",
    "fill": "the motif filling the band (a first vignette draw that came back large), the title under it",
}
TITLES = {
    "current": "current: plain Rajdhani text",
    "hull": "painted: white hull-plate letters, orange trim",
    "stencil": "painted: stenciled hull nameplate",
    "amber": "painted: amber lamp letters on a gunmetal plate",
    "regolith": "painted: letters carved from regolith",
    "earthrise": "painted: white letters with an Earth behind the U",
    "lettered-split": "lettered (CSS): HELIUM in ink, HUSTLE in amber, an amber rule; follows the theme",
    "lettered-orbit": "lettered (CSS): the current text with a small Earth-and-orbit mark; follows the theme",
}


def need(name):
    if not (PICK / name).exists():
        raise SystemExit(f"picker: {PICK / name} missing: run mock.mjs")
    return name


rows = []
for act, a in P["acts"].items():
    opts = [{"key": f, "image": need(f"act{act}_{f}.png"), "label": label} for f, label in FRAMING.items()]
    rows.append(
        {
            "id": f"act{act}",
            "kind": "many",
            "label": f"Act {act}, {a['name']}: chapter header",
            "note": "Each image: the band at the top of the act in a Story timeline, in the middle column at a 1280 px window (595 px wide) on the light page above and at a 1920 px window (1058 px) on the dark page below, the band 3:1 at both (a proposal). The painting is the same in both themes, cropped `cover`; the act title is laid on it as a mock. Subject asked for: "
            + a["scene"],
            "options": opts,
        }
    )

rows.append(
    {
        "id": "title",
        "kind": "many",
        "label": "HELIUM HUSTLE title at the top of the left column",
        "note": "Each image: the title enlarged on the light and dark panel, then at its real size (the h1's 29 px line) in the left column at a 1280 px window (287 px) and a 1920 px one (363 px), light then dark, with the first nav tiles under it for scale. Painted titles are one picture for both themes; lettered ones swap colors with the theme.",
        "options": [{"key": k, "image": need(f"title_{k}.png"), "label": v} for k, v in TITLES.items()],
    }
)
manifest = {
    "name": "story_title_art_r1",
    "title": "Act chapter headers and the HELIUM HUSTLE title, round 1",
    "intro": "Candidates only: nothing is in the game yet. For each act, select every header worth taking forward (several is fine; a pick can also be a framing to redraw with another act's subject). For the title, the same: the painted wordmarks and the two CSS-lettered ones that follow the theme.",
    "rows": rows,
}
(PICK / "manifest.json").write_text(json.dumps(manifest, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
print(f"picker: {len(rows)} rows, {sum(len(r['options']) for r in rows)} images")
