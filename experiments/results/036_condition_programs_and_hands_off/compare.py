"""Paired per-seed comparison of two `hh-sim ladder` reads (games.jsonl): each player's credits
a finished run, decisions a run and milestones met, A (main) against B (this branch), with the
mean paired difference and its standard error.

    python experiments/results/036_condition_programs_and_hands_off/compare.py A/games.jsonl B/games.jsonl
"""

import json
import math
import sys


def load(path):
    out = {}
    for line in open(path, encoding="utf-8"):
        g = json.loads(line)
        runs = g["runs"]
        out[(g["cell"]["agent"], g["seed"])] = {
            "credits": sum(r["credits_earned"] for r in runs) / max(len(runs), 1),
            "later": sum(r["credits_earned"] for r in runs if r["run"] >= 2) / max(sum(1 for r in runs if r["run"] >= 2), 1),
            "decisions": sum(r["decisions"] for r in runs) / max(len(runs), 1),
            "milestones": len(g["milestones"]),
        }
    return out


def stats(xs):
    n = len(xs)
    m = sum(xs) / n
    sd = math.sqrt(sum((x - m) ** 2 for x in xs) / (n - 1)) if n > 1 else 0.0
    return m, sd / math.sqrt(n) if n else 0.0


a, b = load(sys.argv[1]), load(sys.argv[2])
print("| Player | Measure | Main | Branch | Paired diff (mean ± SE) | Seed SD (main) |")
print("|---|---|---|---|---|---|")
for player in sorted({p for p, _ in a}):
    seeds = sorted(s for p, s in a if p == player and (p, s) in b)
    if not seeds:
        continue
    for key in ["credits", "later", "decisions", "milestones"]:
        xa = [a[(player, s)][key] for s in seeds]
        xb = [b[(player, s)][key] for s in seeds]
        d, se = stats([y - x for x, y in zip(xa, xb)])
        ma, _ = stats(xa)
        mb, _ = stats(xb)
        sd = math.sqrt(sum((x - ma) ** 2 for x in xa) / (len(xa) - 1)) if len(xa) > 1 else 0.0
        print(f"| {player} | {key} | {ma:.1f} | {mb:.1f} | {d:+.1f} ± {se:.1f} | {sd:.1f} |")
