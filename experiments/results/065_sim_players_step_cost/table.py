"""065's before/after ladder table from ladder reads' games.jsonl.

usage: python table.py NAME=DIR/games.jsonl [NAME=...]   (run from anywhere)

Per side and player (author, handsoff): every finished run's mean credits earned,
ships and ticks; the mean from run 2 on; the share of runs forced (boredom); run 1's
median ticks.
"""
import json
import statistics
import sys


def rows(path):
    out = {}
    for line in open(path, encoding="utf-8"):
        g = json.loads(line)
        agent = g["cell"]["agent"] if isinstance(g.get("cell"), dict) and "agent" in g["cell"] else g["agent"]
        if agent not in ("author", "handsoff"):
            continue
        out.setdefault(agent, []).append(g)
    return out


def summary(games):
    runs = [r for g in games for r in g["runs"]]
    late = [r for r in runs if r["run"] >= 2]
    first = [r["ticks"] for r in runs if r["run"] == 1]
    mean = lambda xs: sum(xs) / len(xs) if xs else 0.0
    return {
        "credits": mean([r["credits_earned"] for r in runs]),
        "credits2": mean([r["credits_earned"] for r in late]),
        "ships": mean([r["shipments"] for r in runs]),
        "ticks": mean([r["ticks"] for r in runs]),
        "forced": mean([1.0 if r["forced"] else 0.0 for r in runs]),
        "run1": statistics.median(first) if first else 0,
    }


def main():
    sides = [a.split("=", 1) for a in sys.argv[1:]]
    print("| Player | Side | Run credits | From run 2 | Ships/run | Run ticks | Forced | Run 1 ticks |")
    print("|---|---|---|---|---|---|---|---|")
    for agent in ("author", "handsoff"):
        for name, path in sides:
            s = summary(rows(path).get(agent, []))
            print(f"| {agent} | {name} | {s['credits']:,.0f} | {s['credits2']:,.0f} | {s['ships']:.1f} | {s['ticks']:,.0f} | {s['forced']:.0%} | {s['run1']:.0f} |")


if __name__ == "__main__":
    main()
