"""Lays each kept 118 draw (art/ideology_emblems_r1/raw/<id>.png) into a mock axis card on the
dark theme's tokens, at 1x, into the picker folder art/pickers/ideology_emblems_r1/<id>.png.

The card is this item's guess at the redesigned Ideologies view (it is being designed alongside,
not built): 760x300, the emblem as a 240x300 panel on its left edge (the square draw
center-cropped to 4:5), a 3 px rule in the axis's chip color between the panel and the text,
then the axis's chip and name, a sample rank line and bar, and what a rank does (from
docs/rules/ideology.md, abridged). Beside the card, the full square draw at 300x300 so the
crop's loss shows. Ids come from captions.json (only kept draws have a caption). Run from the
repo root after gen.mjs.
"""

import json
import pathlib

from PIL import Image, ImageDraw, ImageFont

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
RAW = REPO / "art" / "ideology_emblems_r1" / "raw"
OUT = REPO / "art" / "pickers" / "ideology_emblems_r1"
OUT.mkdir(parents=True, exist_ok=True)
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))
CAP = json.loads((HERE / "captions.json").read_text(encoding="utf-8"))

PAGE, CARD, LINE = "#16161a", "#232329", "#34343c"
TEXT, MUTED, BAR = "#eeeae4", "#a39e96", "#3a3a42"
CHIP = {"nationalist": "#c62828", "humanist": "#2e7d32", "rationalist": "#1565c0"}
SAMPLE = {
    "nationalist": ("Rank 3", "score 512 · next rank at 812.5", 0.11,
                    "Each rank: demand up, speculators fade faster; land and Nationalist "
                    "buildings cheaper. Rank 5: speculator bursts stop landing."),
    "humanist": ("Rank 2", "score 301 · next rank at 475", 0.23,
                 "Each rank: Dream relief up; boredom grows slower, Humanist buildings "
                 "cheaper. Rank 5: Dream costs no He-3."),
    "rationalist": ("Rank 4", "score 1,020 · next rank at 1,318.75", 0.41,
                    "Each rank: buildings' science up; research and Rationalist buildings "
                    "cheaper. Rank 5: research bought in an earlier run is free."),
}
FONTS = pathlib.Path("C:/Windows/Fonts")


def font(name, size):
    return ImageFont.truetype(str(FONTS / name), size)


F_H3, F_BODY, F_SMALL, F_LABEL = (font("segoeuib.ttf", 22), font("segoeui.ttf", 15),
                                  font("segoeui.ttf", 13), font("segoeui.ttf", 11))


def wrap(d, text, f, width):
    words, lines, cur = text.split(), [], ""
    for w in words:
        t = f"{cur} {w}".strip()
        if d.textlength(t, font=f) <= width:
            cur = t
        else:
            lines.append(cur)
            cur = w
    lines.append(cur)
    return lines


def sheet(id_):
    axis = P["draws"][id_][0]
    raw = Image.open(RAW / f"{id_}.png").convert("RGB")
    w, h = raw.size
    s = min(w, h)
    sq = raw.crop(((w - s) // 2, (h - s) // 2, (w - s) // 2 + s, (h - s) // 2 + s))
    cw = round(s * 0.8)
    panel = sq.crop(((s - cw) // 2, 0, (s - cw) // 2 + cw, s)).resize((240, 300), Image.LANCZOS)

    W, H = 1120, 346
    im = Image.new("RGB", (W, H), PAGE)
    d = ImageDraw.Draw(im)
    cx, cy, CW, CH = 20, 30, 760, 300
    d.text((cx, 10), "axis card (760x300), emblem panel 240x300 on its left edge, 1x, dark theme",
           font=F_LABEL, fill=MUTED)
    d.rounded_rectangle((cx, cy, cx + CW - 1, cy + CH - 1), 8, fill=CARD, outline=LINE)
    mask = Image.new("L", (240, 300), 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, 255, 299), 8, fill=255)
    im.paste(panel, (cx, cy), mask)
    d.rectangle((cx + 240, cy, cx + 242, cy + CH - 1), fill=CHIP[axis])

    tx, tw = cx + 268, CW - 268 - 24
    y = cy + 24
    d.rounded_rectangle((tx, y + 3, tx + 22, y + 25), 4, fill=CHIP[axis])
    d.text((tx + 32, y), axis.capitalize(), font=F_H3, fill=TEXT)
    y += 44
    rank, rest, frac, eff = SAMPLE[axis]
    d.text((tx, y), rank, font=font("segoeuib.ttf", 15), fill=TEXT)
    d.text((tx + d.textlength(rank, font=font("segoeuib.ttf", 15)) + 6, y), f"· {rest}", font=F_BODY, fill=TEXT)
    y += 28
    d.rounded_rectangle((tx, y, tx + tw, y + 8), 4, fill=BAR)
    d.rounded_rectangle((tx, y, tx + round(tw * frac), y + 8), 4, fill="#4caf50")
    y += 20
    d.text((tx, y), "Career best rank: 5", font=F_SMALL, fill=MUTED)
    y += 34
    for line in wrap(d, eff, F_BODY, tw):
        d.text((tx, y), line, font=F_BODY, fill=MUTED)
        y += 22

    fx = cx + CW + 20
    d.text((fx, 10), "full draw (square), 300x300", font=F_LABEL, fill=MUTED)
    im.paste(sq.resize((300, 300), Image.LANCZOS), (fx, cy))
    d.rectangle((fx + 30, cy, fx + 30, cy + 299), fill=MUTED)
    d.rectangle((fx + 269, cy, fx + 269, cy + 299), fill=MUTED)
    im.save(OUT / f"{id_}.png", optimize=True)
    print(f"compose: {id_}.png")


for id_ in CAP:
    if not (RAW / f"{id_}.png").exists():
        raise SystemExit(f"compose: {id_} has a caption but no draw")
    sheet(id_)
