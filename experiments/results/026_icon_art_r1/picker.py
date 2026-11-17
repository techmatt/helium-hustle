"""Writes the picker art/pickers/small_icons_r1/manifest.json over mock.mjs's images: a
`many` row per resource (content order), ideology axis, rival AI (content order) and
compact nav tile (content/nav.json order), one for the horizon progress set, and two
family overviews; the current look first in each row. Run from the repo root last. Stops
if an image is missing.
"""

import json
import pathlib
import re

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
PICK = REPO / "art" / "pickers" / "small_icons_r1"
load = lambda f: json.loads((REPO / f).read_text(encoding="utf-8"))
resources, rivals, nav = load("content/resources.json"), load("content/rivals.json"), load("content/nav.json")
AXES = [("nationalist", "Nationalist"), ("humanist", "Humanist"), ("rationalist", "Rationalist")]
glyphs_src = (HERE / "glyphs.mjs").read_text(encoding="utf-8")
SUBJ = re.search(r"export const SUBJ = \{([^}]*)\}", (HERE / "mock.mjs").read_text(encoding="utf-8")).group(1)
SUBJ = set(re.findall(r"(\w+): '[\w.]+'", SUBJ))
icons_src = (REPO / "web" / "src" / "ui" / "icons.tsx").read_text(encoding="utf-8")  # the rival colors' home since 027
RIVAL_COLORS = dict(re.findall(r"(\w+): '(#[0-9a-f]{6})'", re.search(r"RIVAL_COLORS: Record<string, string> = \{([^}]*)\}", icons_src).group(1)))

# One line per glyph variant: what it shows.
WHAT = {
    "energy": {"bolt": "lightning bolt", "battery": "battery with a bolt", "plug": "power plug"},
    "regolith": {"pile": "three broken rocks", "boulder": "faceted boulder and pebble", "grains": "scattered grains"},
    "ice": {"flake": "snowflake", "cube": "ice cube", "drop": "water drop"},
    "he3": {"nucleus": "the He-3 nucleus: two protons, one neutron (ring)", "tank": "gas cylinder", "moon": "crescent Moon and a dot"},
    "titanium": {"ingot": "metal ingot", "beam": "I-beam", "nut": "hex nut"},
    "circuits": {"traces": "three traces to pads", "board": "board with chip and traces", "dip": "two-row chip (DIP)"},
    "propellant": {"flame": "flame", "tank": "fuel tank with nozzle", "can": "fuel can"},
    "credits": {"coins": "coin stack", "star": "coin with a star", "note": "banknote"},
    "science": {"atom": "atom, three orbits", "bulb": "light bulb", "lens": "magnifier"},
    "land": {"pin": "map pin", "claim": "staked plot with a flag", "parcels": "four parcels, one surveyed"},
    "boredom": {"face": "flat, half-lidded face", "hourglass": "hourglass", "zz": "Zz"},
    "processors": {"cpu": "CPU with pins on four sides", "rack": "server rack", "layers": "stacked layers"},
    "nationalist": {"banner": "flag with a swallowtail", "shield": "shield with a star", "star": "star"},
    "humanist": {"person": "a person", "heart": "heart", "pair": "two people"},
    "rationalist": {"compass": "drafting compass", "square": "set square", "therefore": "therefore sign (three dots)"},
    "aria7": {"wave": "voice waveform", "eye": "watching eye", "sine": "sine wave"},
    "fringe9": {"tassels": "a fringe", "corona": "spiked corona", "eclipse": "eclipse ring"},
    "nodal": {"graph": "three linked nodes", "hub": "hub and spokes", "lattice": "hex lattice of nodes"},
    "crucible": {"pot": "crucible over burners", "forge": "fire bowl", "anvil": "anvil"},
}
SUBJ_WHAT = {"commands": "control console", "projects": "crane lifting a hull", "adversaries": "rival drone with its eye", "retirement": "deck chair", "options": "wrench"}


def need(name):
    if not (PICK / name).exists():
        raise SystemExit(f"picker: {PICK / name} missing: run mock.mjs")
    return name


def variants(subject):
    return re.findall(rf"'{subject}\.(\w+)':", glyphs_src)


CURRENT = {
    "resource": "current: the plain colored square",
    "axis": "current: text only (no icon); shown here as a plain square in the axis color",
    "rival": "current: no icon (a gray fallback square); shown here as a plain square in the proposed color",
}


def subject_row(sid, label, note, kind="resource"):
    opts = [{"key": "current", "image": need(f"{sid}_current.png"), "label": CURRENT[kind]}]
    vs = variants(sid)
    if set(vs) != set(WHAT[sid]):
        raise SystemExit(f"picker: {sid}: glyphs {vs} vs labels {list(WHAT[sid])}")
    for v in vs:
        opts.append({"key": v, "image": need(f"{sid}_{v}.png"), "label": WHAT[sid][v]})
    return {"id": sid, "kind": "many", "label": label, "note": note, "options": opts}


CTX = "The chip enlarged (96 px), then light and dark as it renders inline: a Resources row (14 px), a building card cost line (14 px) and an event line (16 px), 2x device pixels."
rows = []
for r in resources:
    rows.append(subject_row(r["id"], f"{r['name']}: which glyphs go forward?", CTX))
for a, name in AXES:
    rows.append(subject_row(a, f"Ideology {name}: which glyphs go forward?", "Replaces text like \"nationalist-aligned\" (the tooltip names the ideology). Color: the axis's existing color (AXIS_COLORS). The enlarged chip, then light and dark: a row in the Ideologies list, a building card's \"aligned\" line, a Fund event line (14-16 px).", "axis"))
for r in rivals:
    c = RIVAL_COLORS[r["id"]]
    rows.append(subject_row(r["id"], f"Rival {r['name']}: which glyphs go forward?", f"Rivals have no color yet; proposed {c}. The enlarged chip, then light and dark: an Adversaries table row (14 px) and its event line (16 px).", "rival"))
for t in nav:
    i, name = t["id"], t["name"]
    opts = [{"key": "paint", "image": need(f"nav_{i}_paint.png"), "label": "current pick: the painterly icon, shrunk to 18 px"},
            {"key": "chip", "image": need(f"nav_{i}_chip.png"), "label": "025's glyph in the colored square"}]
    if i in SUBJ:
        opts.append({"key": "subj", "image": need(f"nav_{i}_subj.png"), "label": f"square with a glyph matched to the painted subject: {SUBJ_WHAT[i]}"})
    else:
        opts.append({"key": "glyph", "image": need(f"nav_{i}_glyph.png"), "label": "025's glyph alone in the tile color"})
    rows.append({"id": f"nav_{i}", "kind": "many", "label": f"{name} tile, compact only (18 px): which go forward?",
                 "note": "Only for the compact nav (18 px icon, past nine tiles); the full-size tile keeps the painterly icon. Normal and selected, light and dark, the candidate enlarged.",
                 "options": opts})
rows.append({
    "id": "horizon_progress", "kind": "many",
    "label": "Horizon progress on earthlight: which stages go forward?",
    "note": "The foot of the left column at 1280 px (270 px) and 1920 px (346 px), light then dark. Each stage is an edit of the one before (pad edits the bare draw; liftoff, climb and hop each edit pad), cut at the bare draw's band, so the base stays put. liftoff, climb and hop are three takes of the airborne rocket: liftoff barely off the pad; climb clearly flying but high; hop small and low, flying just over the pad.",
    "options": [
        {"key": "bare", "image": need("horizon_bare.png"), "label": "current pick: bare base (025's earthlight)"},
        {"key": "pad", "image": need("horizon_pad.png"), "label": "plus landing pad and two extractors"},
        {"key": "liftoff", "image": need("horizon_liftoff.png"), "label": "rocket just off the pad"},
        {"key": "climb", "image": need("horizon_climb.png"), "label": "rocket clearly airborne, nose at the strip's top edge"},
        {"key": "hop", "image": need("horizon_hop.png"), "label": "small rocket hovering low over the pad"},
    ],
})
rows.append({
    "id": "family", "kind": "many",
    "label": "Family check (reference): every candidate side by side at 16 and 32 px",
    "note": "For judging the set as one family; select a sheet only to say it reads as one.",
    "options": [
        {"key": "resources", "image": need("family_resources.png"), "label": "resources"},
        {"key": "ideologies_rivals", "image": need("family_ideologies_rivals.png"), "label": "ideologies and rivals"},
    ],
})
manifest = {
    "name": "small_icons_r1",
    "title": "Small chip icons, compact nav and horizon progress, round 1",
    "intro": "Candidates only: nothing is in the UI yet. Small icons in 025's chip style: the colored square with a glyph knocked out (white, or near-black where white falls under 2.5:1); the color stays the identity, the glyph adds shape. Per resource, ideology and rival, select every glyph worth taking forward. Then the compact nav fallbacks (18 px only), and the horizon progress stages redrawn on earthlight.",
    "rows": rows,
}
(PICK / "manifest.json").write_text(json.dumps(manifest, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
print(f"picker: {len(rows)} rows, {sum(len(r['options']) for r in rows)} options")
