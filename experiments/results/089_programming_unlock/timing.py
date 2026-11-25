"""Run-1 timing, main vs 089 (paired seeds 31-40, agents author, greedy and random,
careers of 2 retirements): each milestone's median career tick per agent, the run-1
length, run-1 shipments, and the tick each agent bought Programming (branch only).

    python experiments/results/089_programming_unlock/timing.py <main games.jsonl> <branch games.jsonl>

Writes timing.md beside this script.
"""
import json
import statistics
import sys
from pathlib import Path


def load(p):
    return [json.loads(line) for line in Path(p).read_text(encoding="utf-8").splitlines() if line.strip()]


def med(xs):
    return statistics.median(xs) if xs else None


def table(main, branch):
    agents = sorted({g["agent"] for g in main})
    keys = sorted({k for g in main + branch for k in g["milestones"]}, key=lambda k: int(k[1:]))
    rows = ["| Agent | Measure | main | 089 |", "|---|---|---|---|"]
    for a in agents:
        gm = [g for g in main if g["agent"] == a]
        gb = [g for g in branch if g["agent"] == a]
        def run1(gs, k):
            return med([r[k] for g in gs for r in g["runs"] if r["run"] == 1])
        rows.append(f"| {a} | run 1 days (median) | {run1(gm, 'ticks')} | {run1(gb, 'ticks')} |")
        rows.append(f"| {a} | run 1 shipments | {run1(gm, 'shipments')} | {run1(gb, 'shipments')} |")
        for k in keys:
            def tick(gs):
                ts = [g["milestones"][k]["tick"] for g in gs if g["milestones"].get(k)]
                return f"{med(ts)} ({len(ts)}/{len(gs)})"
            rows.append(f"| {a} | {k} tick (median, reached) | {tick(gm)} | {tick(gb)} |")
    return "\n".join(rows)


def main():
    main_games, branch_games = load(sys.argv[1]), load(sys.argv[2])
    text = ("# 089 run-1 timing: main vs Programming\n\nPaired seeds 31-40, careers of 2 retirements, "
            "agents author, greedy, random (`hh-sim experiment experiments/balance.json`). "
            "Milestones: configs/sim/milestones.json.\n\n" + table(main_games, branch_games) + "\n")
    out = Path(__file__).with_name("timing.md")
    out.write_text(text, encoding="utf-8", newline="\n")
    print(text)


if __name__ == "__main__":
    main()
