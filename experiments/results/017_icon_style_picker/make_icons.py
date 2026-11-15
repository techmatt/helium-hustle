"""Icon style round 1: draws the sample icons in four styles and the picker sheets.

Every subject is one geometry on a 32-unit grid, its parts tagged by role; a style is a
way of rendering those roles (stroke weight, fill, accent use), so the four candidates
differ in style only, never in what they depict. Writes SVG only, into the shared art
folder (never committed):

    art/icons/r1/<style>/<subject>.svg       one icon, both themes' inks by CSS var
    art/pickers/icon_style_r1/*.svg          the picker sheets
    art/pickers/icon_style_r1/manifest.json  the picker

Run from the repo root: python experiments/results/017_icon_style_picker/make_icons.py
"""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
ART = ROOT / "art"
PICK = ART / "pickers" / "icon_style_r1"
ICONS_OUT = ART / "icons" / "r1"

# Earthrise tokens (web/src/style.css).
THEMES = {
    "light": dict(page="#f3f1ec", panel="#fbfaf7", card="#ffffff", line="#ddd8cf",
                  text="#211f1b", muted="#625d55", accent="#b86e00",
                  sel_bg="#f4e3c4", btn_line="#cbc5ba",
                  tile="#ebe7df", tile_line="#d6d0c5", tile_fill="#fbfaf7", tile_ink="#3d3932"),
    "dark": dict(page="#16161a", panel="#1c1c21", card="#232329", line="#34343c",
                 text="#eeeae4", muted="#a39e96", accent="#f5a524",
                 sel_bg="#3a2a10", btn_line="#4a4a54",
                 tile="#2c2c33", tile_line="#3d3d46", tile_fill="#36363e", tile_ink="#d8d3cb"),
}

# Resource colors (web/src/ui/common.tsx). `fill` is the true color; `stroke` a per-theme
# shade for when the color is a line (a pale yellow line vanishes on white, a dark brown
# one on near-black). `theme` is the nav accent: the Earthrise amber.
ACCENTS = {
    "energy": dict(fill="#f5d000", light="#b89a00", dark="#f5d000"),
    "regolith": dict(fill="#8b5a2b", light="#8b5a2b", dark="#c08850", fill_dark="#a06a36"),
    "titanium": dict(fill="#c4c4c4", light="#858585", dark="#c4c4c4"),
    "processors": dict(fill="#d63ad6", light="#b52fb5", dark="#e36fe3"),
}


def accent(res, theme, as_line):
    if res == "theme":
        return THEMES[theme]["accent"]
    a = ACCENTS[res]
    if as_line:
        return a[theme]
    return a.get("fill_" + theme, a["fill"])


# Roles: body (a closed area), detail (an open line, or a small area drawn as a line),
# ground (a ground line outside the body),
# accent (an area in the subject's resource color), aline (a line in that color).
SUBJECTS = {
    "nav_buildings": dict(label="Buildings", kind="nav", res="theme", parts=[
        ("body", '<path d="M4,24 A12,12 0 0 1 28,24 Z"/>'),
        ("detail", '<path d="M10,24 A6,12 0 0 1 22,24"/>'),
        ("detail", '<path d="M5.6,18 H26.4"/>'),
        ("accent", '<rect x="14" y="19.5" width="4" height="4.5" rx="0.5"/>'),
        ("ground", '<path d="M2,24 H30 M7,28 H25"/>'),
    ]),
    "nav_commands": dict(label="Commands", kind="nav", res="theme", parts=[
        ("body", '<rect x="3" y="6" width="26" height="20" rx="2.5"/>'),
        ("detail", '<path d="M3,10.5 H29"/>'),
        ("detail", '<path d="M8,14.5 L12,17.5 L8,20.5"/>'),
        ("aline", '<path d="M15,21 H22"/>'),
    ]),
    "nav_research": dict(label="Research", kind="nav", res="theme", parts=[
        ("body", '<path d="M13,5 V12.5 L5.6,25.2 Q4.6,28 7.6,28 H24.4 Q27.4,28 26.4,25.2 L19,12.5 V5 Z"/>'),
        ("accent", '<path d="M9.4,19 H22.6 L26.4,25.2 Q27.4,28 24.4,28 H7.6 Q4.6,28 5.6,25.2 Z"/>'),
        ("detail", '<path d="M11,5 H21"/>'),
        ("detail", '<circle cx="14" cy="23.5" r="1.2"/><circle cx="18.5" cy="22" r="0.9"/>'),
    ]),
    "solar_panel": dict(label="Solar Panel", kind="building", res="energy", parts=[
        ("body", '<rect x="14" y="18" width="4" height="7.5"/>'),
        ("ground", '<path d="M7,28 H25"/>'),
        ("accent", '<path d="M6,4 H30 L26,18 H2 Z"/>'),
        ("detail", '<path d="M14,4 L10,18 M22,4 L18,18 M4,11 H28"/>'),
    ]),
    "excavator": dict(label="Regolith Excavator", kind="building", res="regolith", parts=[
        ("detail", '<path d="M14,17 L21,11"/>'),
        ("body", '<rect x="2" y="16" width="16" height="6.5" rx="1.5"/>'),
        ("body", '<path d="M4,16 V11 Q4,10 5,10 H10.5 L12.5,16 Z"/>'),
        ("accent", '<path d="M19,3.5 H29.5 V9 L25.5,14 H22 L19,9 Z"/>'),
        ("body", '<circle cx="6" cy="25" r="3"/><circle cx="14" cy="25" r="3"/>'),
        ("ground", '<path d="M2,29 H30"/>'),
    ]),
    "smelter": dict(label="Smelter", kind="building", res="titanium", parts=[
        ("body", '<path d="M2,27 V13 L7,9.5 V13 L12,9.5 V13 H14 V4 H18 V27 Z"/>'),
        ("detail", '<path d="M5,27 V22.5 A2.5,2.5 0 0 1 10,22.5 V27"/>'),
        ("accent", '<path d="M21,19.5 H28.5 L31,27 H18.5 Z"/>'),
        ("ground", '<path d="M2,28 H30"/>'),
    ]),
    "data_center": dict(label="Data Center", kind="building", res="processors", parts=[
        ("body", '<rect x="7" y="3" width="18" height="25" rx="2"/>'),
        ("detail", '<path d="M7,11.3 H25 M7,19.6 H25"/>'),
        ("detail", '<path d="M15,7.2 H21 M15,15.5 H21 M15,23.8 H21"/>'),
        ("accent", '<circle cx="11" cy="7.2" r="1.6"/><circle cx="11" cy="15.5" r="1.6"/>'
                   '<circle cx="11" cy="23.8" r="1.6"/>'),
    ]),
}
ORDER = ["nav_buildings", "nav_commands", "nav_research",
         "solar_panel", "excavator", "smelter", "data_center"]

STYLES = {
    "a": dict(name="Hairline", blurb="1.5 px monoline outline, no fill; the resource color as a line"),
    "b": dict(name="Duotone", blurb="2 px outline over a faint ink tint; the resource part filled in its color"),
    "c": dict(name="Solid", blurb="filled ink silhouette, details cut out; the resource part solid"),
    "d": dict(name="Tile", blurb="thin line drawing on a rounded lunar-gray tile; resource part filled"),
}


def parts_svg(subject, style, theme, unit):
    """The subject's parts in a style; `unit` is one display pixel in grid units."""
    t = THEMES[theme]
    res = subject["res"]
    out = []
    for role, el in subject["parts"]:
        line_acc = accent(res, theme, True)
        fill_acc = accent(res, theme, False)
        if style == "a":
            sw = 1.5 * unit
            col = line_acc if role in ("accent", "aline") else t["text"]
            a = f'fill="none" stroke="{col}" stroke-width="{sw:.3f}"'
        elif style == "b":
            sw = 2.0 * unit
            if role == "body":
                a = f'fill="{t["text"]}" fill-opacity="0.10" stroke="{t["text"]}" stroke-width="{sw:.3f}"'
            elif role == "accent":
                a = f'fill="{fill_acc}" stroke="{t["text"]}" stroke-width="{sw:.3f}"'
            elif role == "aline":
                a = f'fill="none" stroke="{line_acc}" stroke-width="{sw * 1.15:.3f}"'
            else:
                a = f'fill="none" stroke="{t["text"]}" stroke-width="{sw:.3f}"'
        elif style == "c":
            if role == "body":
                a = f'fill="{t["text"]}" stroke="{t["text"]}" stroke-width="{1.0 * unit:.3f}"'
            elif role == "accent":
                a = f'fill="{fill_acc}" stroke="none"'
            elif role == "aline":
                a = f'fill="none" stroke="{fill_acc}" stroke-width="{2.4 * unit:.3f}"'
            else:
                # Details cut through the silhouette in the background color; ground lines
                # sit outside it and draw in ink.
                col = t["text"] if role == "ground" else "var(--bg)"
                a = f'fill="none" stroke="{col}" stroke-width="{1.75 * unit:.3f}"'
        else:  # d
            sw = 1.3 * unit
            ink = t["tile_ink"]
            if role == "body":
                a = f'fill="{t["tile_fill"]}" stroke="{ink}" stroke-width="{sw:.3f}"'
            elif role == "accent":
                a = f'fill="{fill_acc}" stroke="{ink}" stroke-width="{sw:.3f}"'
            elif role == "aline":
                a = f'fill="none" stroke="{line_acc}" stroke-width="{sw * 1.5:.3f}"'
            else:
                a = f'fill="none" stroke="{ink}" stroke-width="{sw:.3f}"'
        out.append(f'<g {a}>{el}</g>')
    return "".join(out)


def icon(sid, style, theme, x, y, size, bg):
    """One icon as a nested <svg> at (x, y), `size` px square, drawn over `bg`."""
    s = SUBJECTS[sid]
    t = THEMES[theme]
    unit = 32 / size
    body = ""
    if style == "d":
        # The tile fills the square; the drawing sits inside at 78 %.
        k = 0.78
        off = 16 * (1 - k)
        body += (f'<rect x="{0.5 * unit:.3f}" y="{0.5 * unit:.3f}" width="{32 - unit:.3f}" '
                 f'height="{32 - unit:.3f}" rx="7" fill="{t["tile"]}" stroke="{t["tile_line"]}" '
                 f'stroke-width="{unit:.3f}"/>')
        body += (f'<g transform="translate({off:.3f},{off:.3f}) scale({k})">'
                 f'{parts_svg(s, style, theme, unit / k)}</g>')
    else:
        body += parts_svg(s, style, theme, unit)
    body = body.replace("var(--bg)", bg)
    return (f'<svg x="{x}" y="{y}" width="{size}" height="{size}" viewBox="0 0 32 32" '
            f'stroke-linecap="round" stroke-linejoin="round">{body}</svg>')


FONT_HEAD = "Rajdhani, Bahnschrift, 'Segoe UI', sans-serif"
FONT_BODY = "'IBM Plex Sans', 'Segoe UI', system-ui, sans-serif"


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;")


def text(x, y, s, size, fill, weight=400, head=False, anchor="start"):
    f = FONT_HEAD if head else FONT_BODY
    return (f'<text x="{x}" y="{y}" font-family="{f}" font-size="{size}" font-weight="{weight}" '
            f'fill="{fill}" text-anchor="{anchor}">{esc(s)}</text>')


BUILDINGS = {b["id"]: b for b in json.loads((ROOT / "content" / "buildings.json").read_text("utf-8"))}
RES_NAMES = {r["id"]: r["name"] for r in json.loads((ROOT / "content" / "resources.json").read_text("utf-8"))}
SQ = {"energy": "#f5d000", "regolith": "#8b5a2b", "titanium": "#c4c4c4", "processors": "#d63ad6",
      "credits": "#2ecc40", "circuits": "#35c4a4"}


def amounts_line(x, y, amounts, t, prefix):
    out, cx = [], x
    for rid, amt in amounts.items():
        out.append(f'<rect x="{cx}" y="{y - 9}" width="10" height="10" rx="2" fill="{SQ.get(rid, "#888")}" '
                   f'stroke="rgb(0 0 0 / 18%)"/>')
        s = f"{prefix}{amt} {RES_NAMES[rid]}"
        out.append(text(cx + 14, y, s, 12, t["muted"]))
        cx += 14 + len(s) * 6.6 + 12
    return "".join(out)


def building_card(sid, style, theme, x, y, w, size):
    t = THEMES[theme]
    b = BUILDINGS[sid]
    h = max(size + 20, 78)
    out = [f'<rect x="{x + 0.5}" y="{y + 0.5}" width="{w - 1}" height="{h - 1}" rx="6" fill="{t["card"]}" '
           f'stroke="{t["line"]}"/>']
    out.append(icon(sid, style, theme, x + 10, y + (h - size) / 2, size, t["card"]))
    tx = x + 10 + size + 12
    out.append(text(tx, y + 24, b["name"], 17, t["text"], 700, head=True))
    out.append(text(x + w - 12, y + 24, "x3", 13, t["muted"], anchor="end"))
    if b["produces"]:
        out.append(amounts_line(tx, y + 45, {k: f"{v}/s" for k, v in b["produces"].items()}, t, "+"))
    else:
        out.append(amounts_line(tx, y + 45, {"processors": "1"}, t, "+"))
    out.append(amounts_line(tx, y + 64, b["cost"], t, ""))
    return "".join(out), h


def nav_tile(sid, style, theme, x, y, w, active):
    t = THEMES[theme]
    bg = t["sel_bg"] if active else t["card"]
    edge = t["accent"] if active else t["btn_line"]
    out = [f'<rect x="{x + 0.5}" y="{y + 0.5}" width="{w - 1}" height="65" rx="5" fill="{bg}" stroke="{edge}"/>']
    out.append(icon(sid, style, theme, x + (w - 34) / 2, y + 8, 34, bg))
    out.append(text(x + w / 2, y + 58, SUBJECTS[sid]["label"], 13, t["text"], anchor="middle"))
    return "".join(out)


def theme_panel(style, theme, x, y, w):
    """One theme's half of a style sheet: nav tiles, building cards, a size strip."""
    t = THEMES[theme]
    out = [f'<rect x="{x}" y="{y}" width="{w}" height="560" fill="{t["page"]}"/>']
    out.append(text(x + 16, y + 26, f"{theme.capitalize()}", 13, t["muted"], 600))
    tw = (w - 32 - 12) / 3
    for i, sid in enumerate(ORDER[:3]):
        out.append(nav_tile(sid, style, theme, x + 16 + i * (tw + 6), y + 38, tw, active=(i == 0)))
    cy = y + 118
    for sid in ORDER[3:]:
        svg, h = building_card(sid, style, theme, x + 16, cy, w - 32, 56)
        out.append(svg)
        cy += h + 8
    # The same icons at 24, 32, 48 and 64 px on the page, to judge where they break.
    cy += 6
    cx = x + 16
    for size in (24, 32, 48, 64):
        for sid in ("solar_panel", "nav_research"):
            out.append(icon(sid, style, theme, cx, cy + (64 - size), size, t["page"]))
            cx += size + 6
        cx += 8
    return "".join(out)


def sheet(style):
    w = 470
    s = STYLES[style]
    head = 44
    W, H = 2 * w, head + 560
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">',
           f'<rect width="{W}" height="{head}" fill="#ffffff"/>',
           text(16, 28, f"{style.upper()}. {s['name']}", 20, "#211f1b", 700, head=True),
           text(190, 28, s["blurb"], 13, "#625d55")]
    out.append(theme_panel(style, "light", 0, head, w))
    out.append(theme_panel(style, "dark", w, head, w))
    out.append("</svg>")
    return "".join(out)


def subject_sheet(sid, style):
    """One subject in one style, on both themes: page and card (or selected tile) grounds."""
    kind = SUBJECTS[sid]["kind"]
    size = 34 if kind == "nav" else 56
    big = 64
    cell = big + 24
    W, H = 4 * cell, cell
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">']
    grounds = [("light", "card"), ("light", "page"), ("dark", "card"), ("dark", "page")]
    if kind == "nav":
        grounds[1] = ("light", "sel_bg")
        grounds[3] = ("dark", "sel_bg")
    for i, (theme, key) in enumerate(grounds):
        bg = THEMES[theme][key]
        out.append(f'<rect x="{i * cell}" y="0" width="{cell}" height="{H}" fill="{bg}"/>')
        out.append(icon(sid, style, theme, i * cell + (cell - size) / 2, (H - size) / 2, size, bg))
    out.append("</svg>")
    return "".join(out)


def standalone(sid, style, theme):
    t = THEMES[theme]
    inner = icon(sid, style, theme, 0, 0, 64, t["card"])
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">{inner}</svg>'


def write(p, s):
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(s, encoding="utf-8", newline="\n")


def main():
    for st in STYLES:
        write(PICK / f"style_{st}.svg", sheet(st))
        for sid in ORDER:
            write(PICK / "subjects" / f"{sid}_{st}.svg", subject_sheet(sid, st))
            for theme in THEMES:
                write(ICONS_OUT / f"{st}_{STYLES[st]['name'].lower()}" / f"{sid}_{theme}.svg",
                      standalone(sid, st, theme))
    rows = [dict(id=f"style_{st}", kind="many",
                 label=f"Style {st.upper()}: {s['name']}",
                 note=s["blurb"] + ". Left half light theme, right half dark.",
                 options=[dict(key=st, image=f"style_{st}.svg", label=s["name"])])
            for st, s in STYLES.items()]
    for sid in ORDER:
        sub = SUBJECTS[sid]
        where = "card, selected tile" if sub["kind"] == "nav" else "card, page"
        rows.append(dict(id=f"subject_{sid}", kind="many",
                         label=f"{sub['label']} ({'nav tile' if sub['kind'] == 'nav' else 'building'}) across styles",
                         note=f"Each tile: light {where}, then dark {where}.",
                         options=[dict(key=st, image=f"subjects/{sid}_{st}.svg",
                                       label=f"{st.upper()}. {STYLES[st]['name']}") for st in STYLES]))
    manifest = dict(
        name="icon_style_r1",
        title="Icon style, round 1: nav tiles and buildings",
        intro=("Which styles go forward? Keep any number. Four styles drawing the same seven "
               "samples (3 nav tiles, 4 buildings) on the Earthrise light and dark backgrounds. "
               "The first four rows are whole styles; the rest compare one subject across styles. "
               "Each building carries its main resource's color: Solar Panel Energy, Excavator "
               "Regolith, Smelter Titanium, Data Center Processors; nav icons carry the theme amber. "
               "There is no current option: today's icons are placeholder squares."),
        rows=rows)
    write(PICK / "manifest.json", json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
