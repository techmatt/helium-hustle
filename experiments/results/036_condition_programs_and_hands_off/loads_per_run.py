"""Program writes, loads and saves per run, averaged over traces (runs 1 and 2-5 apart), using
035's counter (`count_loads.runs`).

    python experiments/results/036_condition_programs_and_hands_off/loads_per_run.py LABEL TRACE.txt... [-- LABEL TRACE.txt...]
"""

import importlib.util
import pathlib
import sys

sys.dont_write_bytecode = True  # no __pycache__ beside 035's script
here = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("count_loads", here.parent / "035_program_loadouts_and_rate" / "count_loads.py")
count = importlib.util.module_from_spec(spec)
sys.argv, args = [sys.argv[0]], sys.argv[1:]  # count_loads prints for its own argv: give it none
spec.loader.exec_module(count)

groups, cur = [], None
for a in args:
    if a == "--":
        cur = None
    elif cur is None:
        cur = (a, [])
        groups.append(cur)
    else:
        cur[1].append(a)

print("| Player | Runs | Writes | Loads | Saves | Loads at the run's first look |")
print("|---|---|---|---|---|---|")
for label, paths in groups:
    first, later = [], []
    for p in paths:
        rs = count.runs(p)[:5]  # runs 1-5 (the career's last look is a sixth)
        first.append(rs[0])
        later.extend(rs[1:])
    for name, rows in (("1", first), ("2-5", later)):
        n = max(len(rows), 1)
        w, l, s, f = (sum(r[k] for r in rows) / n for k in ("set", "load", "save", "first_load"))
        print(f"| {label} | {name} | {w:.1f} | {l:.1f} | {s:.1f} | {f:.1f} |")
