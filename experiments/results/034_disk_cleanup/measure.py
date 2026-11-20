"""Measure disk use by location without following links. Usage: measure.py <label> -> JSON to out."""
import json, os, sys

def walk(path, links):
    total = 0
    files = 0
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
    links = []
    if not os.path.exists(p):
        return None
    if os.path.isfile(p):
        return {"bytes": os.path.getsize(p), "files": 1, "links": []}
    t, f = walk(p, links)
    return {"bytes": t, "files": f, "links": links}

MAIN = r"C:\Code\helium-hustle"
WT = r"C:\Code\helium-hustle-worktrees\helium-hustle"
DRIVE = r"C:\Code\gdrive-sync\helium-hustle"
CTRL = r"C:\Code\helium-hustle-control"
locs = {}
locs["main total"] = size(MAIN)
for name in sorted(os.listdir(MAIN)):
    locs["main/" + name] = size(os.path.join(MAIN, name))
for sub in ["web/node_modules", "web/src/wasm", "web/dist", "crates/hh-wasm/pkg", "out"]:
    locs["main/" + sub] = size(os.path.join(MAIN, sub))
tgt = os.path.join(MAIN, "target")
if os.path.isdir(tgt):
    for name in sorted(os.listdir(tgt)):
        locs["main/target/" + name] = size(os.path.join(tgt, name))
locs["drive total"] = size(DRIVE)
if os.path.isdir(DRIVE):
    for name in sorted(os.listdir(DRIVE)):
        locs["drive/" + name] = size(os.path.join(DRIVE, name))
locs["worktrees total"] = size(WT)
for name in sorted(os.listdir(WT)):
    locs["worktrees/" + name] = size(os.path.join(WT, name))
    p = os.path.join(WT, name)
    for sub in ["target", "web/node_modules", "out", ".git"]:
        locs["worktrees/" + name + "/" + sub] = size(os.path.join(p, sub))
locs["control total"] = size(CTRL)
out = sys.argv[1]
with open(out, "w") as fh:
    json.dump(locs, fh, indent=1)
for k, v in locs.items():
    if v is None:
        print(f"{k:55s} missing")
    else:
        print(f"{k:55s} {v['bytes']/1e6:10.1f} MB {v['files']:8d} files links={v['links'][:3]}")
