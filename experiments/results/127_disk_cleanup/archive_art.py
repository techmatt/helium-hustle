"""Move finished pickers and superseded draws (`*.try<n>.*`) from the main checkout's art/
into the Drive archive at the same relative path. Every move is checked before any is made:
no link under a source, no existing destination. Usage: archive_art.py [--do]
"""
import os, sys

ART = r"C:\Code\helium-hustle\art"
ARCH = r"C:\Code\gdrive-sync\helium-hustle\archive\art"

def is_link(p):
    return os.path.islink(p) or os.path.isjunction(p)

def links_under(p):
    out = []
    for root, dirs, files in os.walk(p, followlinks=False):
        for n in dirs + files:
            q = os.path.join(root, n)
            if is_link(q):
                out.append(q)
    return out

moves = []
pick = os.path.join(ART, "pickers")
for n in sorted(os.listdir(pick)):
    moves.append(os.path.join("pickers", n))
for root, dirs, files in os.walk(ART, followlinks=False):
    if os.path.relpath(root, ART).startswith("pickers"):
        continue
    for f in files:
        if ".try" in f:
            moves.append(os.path.relpath(os.path.join(root, f), ART))

bad = []
total = 0
for rel in moves:
    src, dst = os.path.join(ART, rel), os.path.join(ARCH, rel)
    if is_link(src) or (os.path.isdir(src) and links_under(src)):
        bad.append("link: " + src)
    if os.path.lexists(dst):
        bad.append("exists: " + dst)
    if os.path.isdir(src):
        total += sum(os.path.getsize(os.path.join(r, f)) for r, _, fs in os.walk(src) for f in fs)
    else:
        total += os.path.getsize(src)
print(len(moves), "moves,", round(total / 1e6, 1), "MB")
if bad:
    raise SystemExit("refusing:\n" + "\n".join(bad))
if "--do" in sys.argv:
    for rel in moves:
        src, dst = os.path.join(ART, rel), os.path.join(ARCH, rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        os.rename(src, dst)
        print("moved", rel)
else:
    print("\n".join(moves))
