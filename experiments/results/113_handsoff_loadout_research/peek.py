"""State of seed 1's run 3 at several days, with a loadout. Usage: python peek.py <loadout> [start_tick] [days...]"""
import os, re, subprocess, sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
lo = os.path.abspath(sys.argv[1])
start = int(sys.argv[2]) if len(sys.argv) > 2 else 3487
days = [int(d) for d in sys.argv[3:]] or [30, 80, 150, 250, 350, 500, 700]
B = os.path.join(ROOT, "target", "release", "hh-sim.exe")
for d in days:
    a = [B, "trace", os.path.join(ROOT, "experiments", "balance.json"), "--agents", f"handsoff:loadout={lo}",
         "--seeds", "1", "--retirements", "3", "--humans", "mixed", "--at", str(start + d)]
    out = subprocess.run(a, capture_output=True, text=True, cwd=ROOT).stdout
    head = out.splitlines()[0] if out else "?"
    res, blds = [], []
    for l in out.splitlines():
        m = re.match(r"\s+(energy|regolith|he3|titanium|circuits|propellant|credits|ice)\s+([\d.]+)\s+cap\s+(\S+)\s+per_day\s+(\S+)", l)
        if m:
            res.append(f"{m[1][:4]} {float(m[2]):.0f}/{m[3].split('.')[0]} {float(m[4]):+.1f}")
        m = re.match(r"\s+(\w+)\s+owned\s+(\d+)", l)
        if m and m[2] != "0":
            blds.append(f"{m[1][:6]}{m[2]}")
    print(head)
    print("   ", " | ".join(res))
    print("   ", " ".join(blds))
