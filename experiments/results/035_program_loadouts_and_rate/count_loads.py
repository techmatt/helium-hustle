"""Counts, per run, the program writes (`set_program`), loads (`load_loadout`) and saves
(`save_loadout`) in `hh-sim trace` output, and how many of a run's first-look program
puts were loads. Run boundaries are the trace's retirement lines.

    python experiments/results/035_program_loadouts_and_rate/count_loads.py TRACE.txt...
"""

import re
import sys


def runs(path):
    lines = open(path, encoding="utf-8").read().splitlines()
    # The career log at the end: `<career tick>  run N day D  retired|forced_retirement`.
    ends = sorted(int(m.group(1)) for l in lines if (m := re.match(r"\s*(\d+)\s+run \d+ day\s+\d+\s+(retired|forced_retirement)", l)))
    out = [{"set": 0, "load": 0, "save": 0, "first_tick": None, "first_set": 0, "first_load": 0} for _ in range(len(ends) + 1)]
    for line in lines:
        m = re.match(r"\s*(\d+)\s+\{\"type\":\"(set_program|load_loadout|save_loadout)\"", line)
        if not m:
            continue
        tick, kind = int(m.group(1)), m.group(2)
        # An action at a retirement's tick is the new run's (ticks are between days).
        r = out[sum(1 for e in ends if e <= tick)]
        key = {"set_program": "set", "load_loadout": "load", "save_loadout": "save"}[kind]
        r[key] += 1
        if r["first_tick"] is None and key != "save":
            r["first_tick"] = tick
        if tick == r["first_tick"] and key != "save":
            r["first_" + key] += 1
    return out


for p in sys.argv[1:]:
    print(p)
    print("| Run | Writes | Loads | Saves | First look: writes / loads |")
    print("|---|---|---|---|---|")
    for i, r in enumerate(runs(p), 1):
        print(f"| {i} | {r['set']} | {r['load']} | {r['save']} | {r['first_set']} / {r['first_load']} |")
