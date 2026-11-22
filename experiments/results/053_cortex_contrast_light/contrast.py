"""053: contrast of the Cortex mock's colors against its ground, per theme.

Reads the `--cx-*` token blocks out of web/src/ui/CortexMock.tsx (light is `.cx-root`, dark
`:root[data-theme='dark'] .cx-root`) and prints a Markdown table of WCAG contrast ratios
against `--cx-bg` (or the bar, the tooltip, a socket, as named). Usage, from the repo root:
    python experiments/results/053_cortex_contrast_light/contrast.py
"""
import re
from pathlib import Path

SRC = Path(__file__).resolve().parents[3] / "web" / "src" / "ui" / "CortexMock.tsx"
text = SRC.read_text(encoding="utf-8")


def block(sel):
    m = re.search(re.escape(sel) + r" \{(.*?)\n\}", text, re.S)
    return dict(re.findall(r"--cx-([\w-]+):\s*(#[0-9a-f]{6})", m.group(1)))


THEMES = {"light": block("\n.cx-root"), "dark": block(":root[data-theme='dark'] .cx-root")}
INK_MIX = {"light": 0.70, "dark": 1.0}  # --cx-ink-mix: the tooltip's sector line, mixed toward --cx-strong


def mix(a, b, p):
    """CSS color-mix(in srgb, a p, b)."""
    return "#" + "".join(f"{round(int(a[i:i + 2], 16) * p + int(b[i:i + 2], 16) * (1 - p)):02x}" for i in (1, 3, 5))


def lum(h):
    c = [int(h[i : i + 2], 16) / 255 for i in (1, 3, 5)]
    c = [x / 12.92 if x <= 0.04045 else ((x + 0.055) / 1.055) ** 2.4 for x in c]
    return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]


def ratio(a, b):
    la, lb = sorted((lum(a), lum(b)), reverse=True)
    return (la + 0.05) / (lb + 0.05)


# (what, foreground token, background token, the bar it should meet)
PAIRS = [(s, s, "bg", 3.0) for s in ("serenity", "logistics", "logic", "rhetoric", "vigilance")]
PAIRS += [
    ("socket bezel (open node)", "bezel", "bg", 3.0),
    ("locked bezel (meant dim)", "locked-bezel", "bg", 1.5),
    ("unbought link", "edge", "bg", 1.5),
    ("hover / focus ring", "hover", "bg", 3.0),
    ("refused ring", "refused", "bg", 3.0),
    ("bar text", "text", "bar-bg", 4.5),
    ("bar dim text", "dim", "bar-bg", 4.5),
    ("bar numbers", "num", "bar-bg", 4.5),
    ("tooltip text", "tip-text", "tip-bg", 4.5),
    ("tooltip muted", "muted", "tip-bg", 4.5),
    ("tooltip ok", "ok", "tip-bg", 4.5),
    ("tooltip owned", "own", "tip-bg", 4.5),
    ("tooltip refusal", "bad", "tip-bg", 4.5),
    ("flash notice", "flash-text", "bg", 4.5),
    ("plaque name", "brass-name", "brass-inset", 4.5),
]
SECT = ("serenity", "logistics", "logic", "rhetoric", "vigilance")
PAIRS += [(f"owned glyph on {s}", "owned-glyph", s, 3.0) for s in SECT]
PAIRS += [("tooltip picked part", "tip-pick-text", "tip-pick-bg", 4.5)]
PAIRS += [(f"tooltip {s} line", f"ink-{s}", "tip-bg", 4.5) for s in SECT]
for t, tk in THEMES.items():
    for s in SECT:
        tk[f"ink-{s}"] = mix(tk[s], tk["strong"], INK_MIX[t])

print("| Pair | Bar | Light | Dark |\n|---|---|---|---|")
for what, fg, bg, bar in PAIRS:
    cells = []
    for t in ("light", "dark"):
        tk = THEMES[t]
        r = ratio(tk[fg], tk[bg])
        cells.append(f"{r:.1f}{'' if r >= bar else ' (under)'}")
    print(f"| {what} | {bar} | {cells[0]} | {cells[1]} |")
