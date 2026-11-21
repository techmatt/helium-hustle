"""Per-run read of the hands-off player from a ladder's games.jsonl.

python experiments/results/039_days_since_purchase_and_fund_project/runs.py GAMES.jsonl [GAMES.jsonl ...]

For each file, the hands-off careers' runs from run 2 on: how many, mean
length (ticks), forced retirements, mean credits earned, mean buildings, the
decisions and actions the player took in them (0 = no hand), and the
projects the career completed.
"""

import json
import sys


def read(path):
    games = [json.loads(line) for line in open(path, encoding="utf-8")]
    rows = []
    for g in games:
        if g["agent"] != "handsoff":
            continue
        runs = [r for r in g["runs"] if r["run"] >= 2]
        n = len(runs) or 1
        rows.append(
            {
                "seed": g["seed"],
                "runs": len(runs),
                "ticks": sum(r["ticks"] for r in runs) / n,
                "forced": sum(1 for r in runs if r["forced"]),
                "credits": sum(r["credits_earned"] for r in runs) / n,
                "built": sum(r["buildings_built"] for r in runs) / n,
                "decisions": sum(r["decisions"] for r in runs),
                "actions": sum(r["actions"] for r in runs),
                "projects": ",".join(g["projects"]) or "-",
            }
        )
    return rows


def main():
    for path in sys.argv[1:]:
        print(f"## {path}\n")
        print("| Seed | Runs 2+ | Mean ticks | Forced | Credits/run | Built/run | Decisions | Actions | Projects completed |")
        print("|---|---|---|---|---|---|---|---|---|")
        for r in read(path):
            print(
                f"| {r['seed']} | {r['runs']} | {r['ticks']:.0f} | {r['forced']} | {r['credits']:.0f} | {r['built']:.1f} | "
                f"{r['decisions']} | {r['actions']} | {r['projects']} |"
            )
        print()


if __name__ == "__main__":
    main()
