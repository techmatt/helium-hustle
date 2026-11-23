"""Days into each hands-off run (the runs after the one that completed
Conditional Logic) to key moments, medians over the runs that reached them,
with how many runs did. Reads the 060 harness's games.jsonl (its `marks`:
the day each held first, watched every tick).

    python experiments/results/060_loadout_exploration/milestones.py F...
"""
import json
import sys
from collections import defaultdict
from statistics import median

KEYS = [("pad", "1st pad"), ("ship", "1st ship"), ("dc2", "2nd DC"), ("dc3", "3rd DC"), ("ship5", "5th ship"), ("research", "1st research")]

rows = defaultdict(lambda: defaultdict(list))
nruns = defaultdict(int)
for path in sys.argv[1:]:
    for line in open(path):
        g = json.loads(line)
        first = next((r["run"] for r in g["runs"] if "conditional_logic" in r.get("projects", [])), None)
        if first is None:
            continue
        agent = g["agent"].removeprefix("agent=")
        for r in g["runs"]:
            if r["run"] <= first:
                continue
            nruns[agent] += 1
            m = g["marks"].get(str(r["run"]), {})
            for k, label in KEYS:
                if k in m:
                    rows[agent][label].append(m[k])

print("| Player | " + " | ".join(k[1] for k in KEYS) + " |")
print("|---|" + "---|" * len(KEYS))
for agent, n in nruns.items():
    cells = []
    for _, label in KEYS:
        xs = rows[agent][label]
        if not xs:
            cells.append("-")
        elif len(xs) == n:
            cells.append(f"d{median(xs):.0f}")
        else:
            cells.append(f"d{median(xs):.0f} ({len(xs)}/{n})")
    print(f"| {agent} | " + " | ".join(cells) + " |")
