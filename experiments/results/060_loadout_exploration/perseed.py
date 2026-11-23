"""Per-seed hands-off-run credits and ships: perseed.py F..."""
import json
import sys

for path in sys.argv[1:]:
    for line in open(path):
        g = json.loads(line)
        runs = g["runs"]
        first = next((r["run"] for r in runs if "conditional_logic" in r.get("projects", [])), None)
        ho = [r for r in runs if first is not None and r["run"] > first]
        cr = " ".join(f"{r['credits_earned']/1000:.1f}k/{r['shipments']}" for r in ho)
        print(f"{g['agent']:<14} s{g['seed']} first_ho_run {first and first+1}: {cr}")
