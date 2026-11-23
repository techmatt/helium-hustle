"""The README's first table: per player, means over the hands-off runs (the
runs after the one that completed Conditional Logic), from games.jsonl files.

    python experiments/results/060_loadout_exploration/stats.py F...
"""
import json
import statistics
import sys
from collections import defaultdict

rows = defaultdict(list)
for path in sys.argv[1:]:
    for line in open(path):
        g = json.loads(line)
        runs = g["runs"]
        first = next((r["run"] for r in runs if "conditional_logic" in r.get("projects", [])), None)
        if first is None:
            continue
        rows[g["agent"].removeprefix("agent=")].append([r for r in runs if r["run"] > first])

def mean(xs):
    return sum(xs) / len(xs) if xs else float("nan")

print("| Player | Seeds | Runs | Credits a run (SE over seeds) | Credits / 1000 days | Ships a run | Run days | Forced | Buildings a run | Research a run |")
print("|---|---|---|---|---|---|---|---|---|---|")
for agent, games in rows.items():
    runs = [r for ho in games for r in ho]
    per_seed = [mean([r["credits_earned"] for r in ho]) for ho in games]
    se = statistics.stdev(per_seed) / len(per_seed) ** 0.5 if len(per_seed) > 1 else float("nan")
    per_day = sum(r["credits_earned"] for r in runs) / sum(r["ticks"] for r in runs) * 1000
    print(f"| {agent} | {len(games)} | {len(runs)} | {mean([r['credits_earned'] for r in runs]):,.0f} ({se:,.0f}) | {per_day:,.0f} | "
          f"{mean([r['shipments'] for r in runs]):.1f} | {mean([r['ticks'] for r in runs]):,.0f} | {100 * mean([r['forced'] for r in runs]):.0f}% | "
          f"{mean([r['buildings_built'] for r in runs]):.0f} | {mean([r['research'] for r in runs]):.1f} |")
