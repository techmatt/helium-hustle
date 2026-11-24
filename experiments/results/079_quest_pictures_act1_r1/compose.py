"""Lays each kept 079 draw (art/quest_act1_r1/raw/<id>.png) out at its three real sizes, on the
dark theme's tokens, into the picker folder art/pickers/quest_pictures_act1_r1/<id>.png:

- the story modal (style.css `.modal`: 560 px wide, 18/22 px padding, so a 516x344 picture),
  with the quest's title above and its story paragraph (a side quest's body) below;
- the left column at 287 px (287x191), at the column's foot where the lunar-base scene sits;
- the Story timeline row: the beat's done dot, a 72x48 thumbnail, its title and summary.

Draws not 3:2 are center-cropped to 3:2 first. Ids come from captions.json (only kept draws
have a caption). Sheets are drawn at 1x, real size. Run from the repo root after gen.mjs.
"""

import json
import pathlib
import re

from PIL import Image, ImageDraw, ImageFont

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[2]
RAW = REPO / "art" / "quest_act1_r1" / "raw"
OUT = REPO / "art" / "pickers" / "quest_pictures_act1_r1"
OUT.mkdir(parents=True, exist_ok=True)
P = json.loads((HERE / "prompts.json").read_text(encoding="utf-8"))
CAP = json.loads((HERE / "captions.json").read_text(encoding="utf-8"))
EV = {q["beat"]: q for q in json.loads((REPO / "content" / "events.json").read_text(encoding="utf-8")) if q.get("beat")}

PAGE, PANEL, CARD, LINE = "#16161a", "#1c1c21", "#232329", "#34343c"
TEXT, MUTED, ACCENT = "#eeeae4", "#a39e96", "#f5a524"
FONTS = pathlib.Path("C:/Windows/Fonts")


def font(name, size):
    return ImageFont.truetype(str(FONTS / name), size)


F_H2, F_BODY, F_SMALL, F_BOLD, F_LABEL = (font("segoeuib.ttf", 20), font("segoeui.ttf", 14),
                                          font("segoeui.ttf", 12), font("segoeuib.ttf", 13),
                                          font("segoeui.ttf", 11))


def crop32(im):
    w, h = im.size
    if abs(w / h - 1.5) < 0.01:
        return im
    if w / h > 1.5:
        nw = round(h * 1.5)
        return im.crop(((w - nw) // 2, 0, (w - nw) // 2 + nw, h))
    nh = round(w / 1.5)
    return im.crop((0, (h - nh) // 2, w, (h - nh) // 2 + nh))


def wrap(d, text, f, width, max_lines):
    words, lines, cur = text.split(), [], ""
    for w in words:
        t = f"{cur} {w}".strip()
        if d.textlength(t, font=f) <= width:
            cur = t
        else:
            lines.append(cur)
            cur = w
    lines.append(cur)
    if len(lines) > max_lines:
        lines = lines[:max_lines]
        lines[-1] = lines[-1].rstrip(".,") + "…"
    return lines


def rounded(im, r):
    mask = Image.new("L", im.size, 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, im.size[0] - 1, im.size[1] - 1), r, fill=255)
    return im, mask


def sheet(id_):
    beat = P["draws"][id_][0]
    q = EV[beat]
    pic = crop32(Image.open(RAW / f"{id_}.png").convert("RGB"))
    W, H = 920, 600
    im = Image.new("RGB", (W, H), PAGE)
    d = ImageDraw.Draw(im)

    # The story modal.
    mx, my, mw = 20, 26, 560
    d.text((mx, 8), "story modal (560 px card, 516 px picture)", font=F_LABEL, fill=MUTED)
    para = (q.get("story") or [q.get("body", "")])[0].replace("DRAFT. ", "")
    body = wrap(d, para, F_BODY, 516, 6)
    mh = 4 + 18 + 28 + 10 + 344 + 12 + 21 * len(body) + 18
    d.rounded_rectangle((mx, my, mx + mw, my + mh), 6, fill=CARD)
    d.rectangle((mx, my, mx + mw, my + 3), fill=ACCENT)
    y = my + 4 + 18
    d.text((mx + 22, y), q["title"], font=F_H2, fill=TEXT)
    y += 28 + 10
    im.paste(pic.resize((516, 344), Image.LANCZOS), (mx + 22, y))
    y += 344 + 12
    for line in body:
        d.text((mx + 22, y), line, font=F_BODY, fill=TEXT)
        y += 21

    # The left column at 287 px: tiles above, the picture at the foot.
    lx, ly = 608, 26
    d.text((lx, 8), "left column (287 px)", font=F_LABEL, fill=MUTED)
    d.rectangle((lx, ly, lx + 287, ly + 300), fill=PANEL)
    for i in range(3):
        d.rounded_rectangle((lx + 8, ly + 8 + i * 34, lx + 279, ly + 36 + i * 34), 4, fill=CARD, outline=LINE)
    im.paste(pic.resize((287, 191), Image.LANCZOS), (lx, ly + 300 - 191))

    # The Story timeline row.
    tx, ty = 608, 350
    d.text((tx, ty - 18), "Story timeline (72x48 thumbnail)", font=F_LABEL, fill=MUTED)
    d.rectangle((tx, ty, tx + 300, ty + 88), fill=PANEL)
    d.line((tx + 17, ty, tx + 17, ty + 88), fill=ACCENT, width=2)
    d.ellipse((tx + 6, ty + 8, tx + 28, ty + 30), fill=ACCENT)
    thumb, mask = rounded(pic.resize((72, 48), Image.LANCZOS), 4)
    im.paste(thumb, (tx + 40, ty + 6), mask)
    title = re.sub(r"^\S+\s+\S+\s+", "", q["title"])
    d.text((tx + 120, ty + 6), f"{beat}  {title}", font=F_BOLD, fill=TEXT)
    for i, line in enumerate(wrap(d, q["summary"].replace("DRAFT. ", ""), F_SMALL, 175, 3)):
        d.text((tx + 120, ty + 26 + i * 16), line, font=F_SMALL, fill=MUTED)

    im.crop((0, 0, W, max(my + mh, ty + 88) + 16)).save(OUT / f"{id_}.png", optimize=True)
    print(f"compose: {id_}.png")


for id_ in CAP:
    if not (RAW / f"{id_}.png").exists():
        raise SystemExit(f"compose: {id_} has a caption but no draw")
    sheet(id_)
