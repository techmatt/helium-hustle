"""Renders a loadout file (loadouts/*.txt, ids) in the editor's own words
(display names from content/, as the program editor shows each line), one
block per program tab. Prints Markdown.

    python experiments/results/060_loadout_exploration/render.py loadouts/L4_two_tabs.txt
"""
import json
import sys

def names(path, key):
    data = json.load(open(path, encoding="utf-8"))
    rows = data if isinstance(data, list) else data.get(key, data)
    return {r["id"]: r["name"] for r in rows}

N = {}
for path, key in [("content/buildings.json", "buildings"), ("content/commands.json", "commands"),
                  ("content/resources.json", "resources"), ("content/research.json", "research"),
                  ("content/projects.json", "projects")]:
    N.update(names(path, key))

def clause(words):
    at = next(i for i, w in enumerate(words) if w in ("above", "below"))
    what = " ".join(words[:at])
    label = {"free processors": "Free processors", "free land": "Free land", "idle pads": "Idle pads"}.get(what, N.get(what, what))
    return f"{label} {words[at]} {words[at + 1]}"

def line(text):
    w = text.split()
    if w[0] == "if":
        j = next((i for i, x in enumerate(w) if x in ("and", "or")), None)
        if j is None:
            return "If " + clause(w[1:])
        return f"If {clause(w[1:j])} {w[j]} {clause(w[j + 1:])}"
    if w[0] == "build":
        return f"Build {N[w[1]]}" + (f" up to {w[4]}" if len(w) > 2 else "")
    if w[:2] == ["buy", "land"]:
        return "Buy Land"
    if w[:2] in (["launch", "pads"], ["load", "pads"]):
        return f"{w[0].capitalize()} pads {'all' if w[2] == 'all' else N[w[2]]}"
    if w[:2] == ["set", "pads"]:
        return f"Set pads {N[w[2]]} up to {w[5]}"
    if w[:2] == ["assign", "processors"]:
        return f"Assign processors Program {w[2]} {w[3]}"
    if w[0] in ("research", "pause", "resume", "fund"):
        return f"{w[0].capitalize()} {N[w[1]]}" + (f" {w[2]}" if len(w) > 2 else "")
    if w[0] == "retire":
        return "Retire"
    count = w[1][1:] if len(w) > 1 else "1"
    return f"{N[w[0]]} (x{count})"

def render(path):
    out, tab = [], None
    for raw in open(path, encoding="utf-8"):
        t = raw.split("#")[0].rstrip()
        if not t.strip() or t.startswith(("seat:", "stale:")):
            continue
        if t.startswith("["):
            if tab is not None:
                out.append("```\n")
            tab = t.strip("[]")
            out.append(f"Program {tab}:\n\n```text")
            continue
        out.append(("    " if raw.startswith(" ") else "") + line(t))
    out.append("```")
    return "\n".join(out)

if __name__ == "__main__":
    for p in sys.argv[1:]:
        print(render(p))
