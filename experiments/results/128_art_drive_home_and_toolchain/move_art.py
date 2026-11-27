"""Move the main checkout's art/ into the project's Drive home and leave a junction at the old
path. Everything is checked before anything moves: art/ is a real folder with no link under it,
the destination does not exist, both are on one volume (so the move is a rename, never a copy
and delete). Usage: move_art.py [--do]
"""
import os, subprocess, sys

ART = r"C:\Code\helium-hustle\art"
HOME = r"C:\Code\gdrive-sync\helium-hustle\art"


def is_link(p):
    return os.path.islink(p) or os.path.isjunction(p)


def tally(p):
    n = size = 0
    links = []
    for root, dirs, files in os.walk(p, followlinks=False):
        for d in dirs:
            if is_link(os.path.join(root, d)):
                links.append(os.path.join(root, d))
        for f in files:
            q = os.path.join(root, f)
            if is_link(q):
                links.append(q)
            n += 1
            size += os.path.getsize(q)
    return n, size, links


if is_link(ART) and os.path.realpath(ART) == os.path.realpath(HOME):
    raise SystemExit("already moved: " + ART + " -> " + os.path.realpath(ART))

bad = []
if is_link(ART) or not os.path.isdir(ART):
    bad.append("not a real folder: " + ART)
if os.path.lexists(HOME):
    bad.append("exists: " + HOME)
if not os.path.isdir(os.path.dirname(HOME)):
    bad.append("no Drive home: " + os.path.dirname(HOME))
if os.path.splitdrive(ART)[0].lower() != os.path.splitdrive(HOME)[0].lower():
    bad.append("different volumes")
n, size, links = tally(ART) if not bad else (0, 0, [])
bad += ["link under art: " + q for q in links]
print(n, "files,", round(size / 1e6, 1), "MB")
if bad:
    raise SystemExit("refusing:\n" + "\n".join(bad))
if "--do" not in sys.argv:
    raise SystemExit("dry run: pass --do to move")

os.rename(ART, HOME)
print("moved", ART, "->", HOME)
r = subprocess.run(["cmd", "/c", "mklink", "/J", ART, HOME], capture_output=True, text=True)
print(r.stdout.strip(), r.stderr.strip())
if r.returncode != 0:
    raise SystemExit("JUNCTION FAILED: art is at " + HOME + "; make the junction by hand")
n2, size2, _ = tally(ART + "\\")
print("through the junction:", n2, "files,", round(size2 / 1e6, 1), "MB")
if (n2, size2) != (n, size):
    raise SystemExit("MISMATCH through the junction")
