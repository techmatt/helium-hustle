"""The ladder's Hands-off measure (experiment.rs `later_credits`, `later_milestones`)
from games.jsonl files: credits a finished run from run 2 on (mean over runs),
milestones first met from run 2 on (summed over seeds), plus ships a run from run 2.

    python share.py F...
"""
import json
import sys

for path in sys.argv[1:]:
    games = [json.loads(l) for l in open(path)]
    runs = [r for g in games for r in g["runs"] if r["run"] >= 2]
    cr = sum(r["credits_earned"] for r in runs) / len(runs)
    ms = sum(sum(1 for h in g["milestones"].values() if h["run"] >= 2) for g in games)
    ships = sum(r["shipments"] for r in runs) / len(runs)
    print(f"{path.split('/')[-1]:30s} runs {len(runs):3d} credits/run {cr:9,.0f} milestones {ms:3d} ships/run {ships:5.1f}")
