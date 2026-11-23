"""Hands-off runs from a ladder's games.jsonl, after its loadout loads (the
author's runs left out: those with more than a few decisions): how each ended
(boredom, or a voluntary retirement), hand decisions a run, credits a run.

    python runs.py <label>=<games.jsonl> ...
"""
import json
import sys

for arg in sys.argv[1:]:
    label, path = arg.split("=", 1)
    games = [json.loads(l) for l in open(path, encoding="utf-8") if '"handsoff"' in l]
    runs = []
    for g in games:
        if g["cell"]["agent"] != "handsoff":
            continue
        # The loadout's runs: from the second run with as few decisions as a load.
        loaded = [r for r in g["runs"] if r["decisions"] <= 6]
        runs += loaded
    if not runs:
        print(label, "no runs")
        continue
    n = len(runs)
    forced = sum(r["forced"] for r in runs)
    dec = {}
    for r in runs:
        if not r["forced"]:
            dec[r["decisions"]] = dec.get(r["decisions"], 0) + 1
    cr = sum(r["credits_earned"] for r in runs) / n
    days = sum(r["ticks"] for r in runs) / n
    print(f"{label:12s} runs {n:3d} credits/run {cr:8,.0f} run days {days:6,.0f} "
          f"boredom-ended {forced:3d} voluntary {n - forced:3d} (by decisions a run: {dict(sorted(dec.items()))}) "
          f"research/run {sum(r['research'] for r in runs) / n:.2f}")
