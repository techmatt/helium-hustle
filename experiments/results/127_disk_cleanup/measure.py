"""Measure disk use by location without following links (034's walk, deeper).

Usage: python measure.py <out.json>. Links (junctions, symlinks) are recorded, never
followed, so a link's target counts once, at its real home.
"""
import json, os, sys

def walk(path, links):
    total = files = 0
    try:
        it = os.scandir(path)
    except OSError:
        return 0, 0
    with it:
        for e in it:
            try:
                if e.is_symlink() or (hasattr(e, "is_junction") and e.is_junction()):
                    links.append(e.path)
                    continue
                if e.is_dir(follow_symlinks=False):
                    t, f = walk(e.path, links)
                    total += t; files += f
                else:
                    total += e.stat(follow_symlinks=False).st_size
                    files += 1
            except OSError:
                pass
    return total, files

def size(p):
    if not os.path.lexists(p):
        return None
    if os.path.isfile(p):
        return {"bytes": os.path.getsize(p), "files": 1, "links": []}
    links = []
    t, f = walk(p, links)
    return {"bytes": t, "files": f, "links": links}

def children(prefix, root, depth, locs):
    if depth == 0 or not os.path.isdir(root):
        return
    try:
        names = sorted(os.listdir(root))
    except OSError:
        return
    for n in names:
        p = os.path.join(root, n)
        if os.path.islink(p) or (hasattr(os.path, "isjunction") and os.path.isjunction(p)):
            locs[prefix + "/" + n] = {"bytes": 0, "files": 0, "links": [p]}
            continue
        if os.path.isdir(p):
            locs[prefix + "/" + n] = size(p)
            children(prefix + "/" + n, p, depth - 1, locs)

MAIN = r"C:\Code\helium-hustle"
WT = r"C:\Code\helium-hustle-worktrees\helium-hustle"
DRIVE = r"C:\Code\gdrive-sync\helium-hustle"
CTRL = r"C:\Code\helium-hustle-control"
locs = {}
locs["main total"] = size(MAIN)
for name in sorted(os.listdir(MAIN)):
    locs["main/" + name] = size(os.path.join(MAIN, name))
for sub in ["web/node_modules", "web/src/wasm", "web/dist", "web/test-results",
            "web/playwright-report", "crates/hh-wasm/pkg", "out"]:
    locs["main/" + sub] = size(os.path.join(MAIN, sub))
children("main/art", os.path.join(MAIN, "art"), 2, locs)
children("main/target", os.path.join(MAIN, "target"), 1, locs)
children("main/experiments/results", os.path.join(MAIN, "experiments", "results"), 1, locs)
locs["drive total"] = size(DRIVE)
children("drive", DRIVE, 3, locs)
locs["worktrees total"] = size(WT)
for name in sorted(os.listdir(WT)):
    p = os.path.join(WT, name)
    locs["worktrees/" + name] = size(p)
    for sub in ["target", "web/node_modules", "out", ".git"]:
        locs["worktrees/" + name + "/" + sub] = size(os.path.join(p, sub))
locs["control total"] = size(CTRL)
with open(sys.argv[1], "w") as fh:
    json.dump(locs, fh, indent=1)
for k, v in locs.items():
    if v is None:
        print(f"{k:60s} missing")
    else:
        print(f"{k:60s} {v['bytes']/1e6:10.1f} MB {v['files']:8d} files links={v['links'][:3]}")
