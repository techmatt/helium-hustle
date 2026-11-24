"""Writes the picker art/pickers/story_title_art_r2/manifest.json over compose.py's images: one
`many` row per act (the plain band the game draws today first, as the current option), a row
for the timeline's spine, one of whole-window Story pages, and one for the HELIUM HUSTLE
title (the plain h1 first). Rows start with nothing selected. Run from the repo root last;
stops if an image is missing.
"""

import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
PICK = REPO / "art" / "pickers" / "story_title_art_r2"
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))
ACTS = json.loads((REPO / "content" / "acts.json").read_text(encoding="utf-8"))


def need(name):
    if not (PICK / f"{name}.png").exists():
        raise SystemExit(f"picker: {PICK / name}.png missing: run mock.mjs and compose.py")
    return f"{name}.png"


def scene(id_):
    return P["draws"][id_][1]


SHOT_NOTE = (
    "Each image: the real Story panel in the running game (a new game played to 1.7, later acts "
    "appended in the panel's own markup), the middle column at a 1280 px window (595 px of "
    "content) on the light theme at the left and at a 1920 px window (1058 px) on the dark theme "
    "at the right, at real size. The painting is the same in both themes."
)

rows = []
for a in ACTS:
    n = a["act"]
    opts = [
        {"key": "current", "image": need(f"act{n}_current"), "label": "current: the plain styled band 075 ships"},
        {"key": "side", "image": need(f"act{n}_side"), "label": "round 1's picked side header, at round 1's crop, for comparison"},
    ]
    for v in (2, 3, 4):
        id_ = f"act{n}-side{v}"
        if (PICK / f"act{n}_side{v}.png").exists():
            opts.append({"key": f"side{v}", "image": need(f"act{n}_side{v}"), "label": f"side, new draw: {scene(id_).split(': ', 1)[1]}"})
    if (PICK / f"act{n}_tall.png").exists():
        opts.append({"key": "tall", "image": need(f"act{n}_tall"), "label": "tall backdrop: a 2:3 painting behind the act, the spine and beats running over it, the painting fading out down the act (it scrolls with the act; shown at one scroll position)"})
    opts.append({"key": "bleed", "image": need(f"act{n}_bleed"), "label": "bleed: the picked side painting shown whole (uncropped 3:2), the title on it, the art running on behind the first beats and fading"})
    if n == 1 and (PICK / "act1_sky.png").exists():
        opts.append({"key": "sky", "image": need("act1_sky"), "label": "sky: the base painted with no Earth, reaching up into the column's sky so the page's own Earth hangs over it (the page's sky gradient dropped; the painting runs behind the Story heading)"})
    if (PICK / f"act{n}_bridge.png").exists():
        opts.append({"key": "bridge", "image": need(f"act{n}_bridge"), "label": f"bridge: a 5:1 transition strip from Act {n - 1} into this act, edges fading into the page, then the picked side band"})
    rows.append(
        {
            "id": f"act{n}",
            "kind": "many",
            "label": f"Act {n}, {a['name']}: header compositions",
            "note": SHOT_NOTE + " Select every composition worth taking forward; a composition picked on one act can carry to the others.",
            "options": opts,
        }
    )

spine = [{"key": "current", "image": need("spine_current"), "label": "current: the amber gradient line"}]
for s, label in (
    ("conduit", "painted power conduit, the beat dots its amber lamps"),
    ("trail", "painted launch trail"),
    ("beam", "painted sunlight beam with girders"),
    ("orbit", "CSS only: a dashed orbit line with a faint second track"),
):
    if (PICK / f"spine_{s}.png").exists():
        spine.append({"key": s, "image": need(f"spine_{s}"), "label": label})
rows.append(
    {
        "id": "spine",
        "kind": "many",
        "label": "The timeline's spine drawn as part of the painting",
        "note": SHOT_NOTE + " Scrolled to the active beat (1.7) with a side quest branching off 1.6 and the dim ???. Painted strips repeat down the spine; their black ground drops out on the light page.",
        "options": spine,
    }
)

PAGES = [
    ("current", "current: the Story page as 075 built it"),
    ("side", "Act 1's picked side band in the page, the Earth above it"),
    ("bleed-conduit", "bleed composition with the conduit spine"),
    ("sky", "sky: the page's Earth hanging over the painted base"),
    ("tall", "tall backdrop at the top of the page"),
    ("act2-bridge", "Act 2 reached: the bridge strip and the swarm band, the launch-trail spine"),
    ("takeover", "a full-width Story takeover (like the Cortex) at 1.7: the panorama, acts as columns, unreached acts fogged"),
    ("takeover-late", "the same takeover late in the game, every act reached and the panorama unfogged"),
]
rows.append(
    {
        "id": "page",
        "kind": "many",
        "label": "Whole Story pages",
        "note": "Each image: the whole game window, 1280x800 light at the left and 1920x1080 dark at the right, at real size, the Story panel open with a composition in place (the left column's title and lunar base, the middle column's Earth and the right column as they are).",
        "options": [{"key": k, "image": need(f"page_{k}"), "label": v} for k, v in PAGES if (PICK / f"page_{k}.png").exists()],
    }
)

TITLES = {
    "current": "current: plain Rajdhani text",
    "lettered-split": "round 1's pick as it was: HELIUM in ink, HUSTLE in amber, a short amber rule",
    "split-heavy": "bigger and tighter, a full-width amber rule under it",
    "split-stack": "two lines: HELIUM small and wide-spaced in muted ink over a large amber HUSTLE",
    "split-outline": "HELIUM as outlined letters, HUSTLE solid amber",
    "split-plate": "HUSTLE in ink on an amber plate, like the selected nav tile",
    "split-he3": "a small amber superscript 3 before HELIUM (helium-3)",
    "split-glow": "HUSTLE with a soft amber glow, a longer glowing rule (the active beat's glow)",
    "tie-spine": "ties to Story: the rule is a small timeline, two lit beats, the glowing active one, a dim ???",
    "tie-earth": "ties to the middle column: the page's own Earth between the words",
    "tie-horizon": "ties to the left column's lunar base: the horizon art behind the title (light letters in both themes)",
    "tie-band": "ties to the act bands: the title on the bands' night-sky plate",
}
rows.append(
    {
        "id": "title",
        "kind": "many",
        "label": "HELIUM HUSTLE title at the top of the left column",
        "note": "Each image: the top of the real left column in the running game, at real size shot at 2x pixels: a 1280 px window (287 px column) and a 1920 px one (363 px), light, light, dark, dark. All CSS-lettered on the theme tokens, so they swap with the theme.",
        "options": [{"key": k, "image": need(f"title_{k}"), "label": v} for k, v in TITLES.items()],
    }
)

manifest = {
    "name": "story_title_art_r2",
    "title": "Story headers, page compositions and the HELIUM HUSTLE title, round 2",
    "intro": "Candidates only: nothing is in the game yet. Every image is the real game page with the candidate laid in. Round 1 picked the side framing for every act and the lettered-split title; this round adds new side draws per act, compositions beyond the band (tall backdrop, bleed, sky, bridge, painted spines, a full-width takeover) and title variants. Select every option worth taking forward, several per row is fine.",
    "rows": rows,
}
(PICK / "manifest.json").write_text(json.dumps(manifest, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
print(f"picker: {len(rows)} rows, {sum(len(r['options']) for r in rows)} images")
