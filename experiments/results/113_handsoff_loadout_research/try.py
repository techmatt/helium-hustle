"""Play hands-off on given seeds with a loadout file; print each run's summary.
Usage: python try.py <loadout> [seeds] [retirements]"""
import json, os, subprocess, sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
lo = os.path.abspath(sys.argv[1])
seeds = sys.argv[2] if len(sys.argv) > 2 else "1"
rets = sys.argv[3] if len(sys.argv) > 3 else "3"
name = os.path.splitext(os.path.basename(lo))[0]
out = os.path.join(ROOT, "out", "113_handsoff_loadout_research", "runs", f"{name}-{seeds}-{rets}")
agent = "handsoff" if name == "base" else f"handsoff:loadout={lo}"
a = [os.path.join(ROOT, "target", "release", "hh-sim.exe"), "experiment", os.path.join(ROOT, "experiments", "balance.json"),
     "--human-models", os.path.join(ROOT, "configs", "sim", "humans.json"),
     "--milestones", os.path.join(ROOT, "configs", "sim", "milestones.json"),
     "--config", os.path.join(ROOT, "configs", "baseline.json"),
     "--agents", agent, "--humans", "mixed", "--seeds", seeds, "--retirements", rets, "--out", out, "--threads", "6"]
r = subprocess.run(a, capture_output=True, text=True)
if r.returncode:
    print(r.stdout[-2000:], r.stderr[-3000:])
    sys.exit(1)
keys = ["run", "ticks", "shipments", "buildings_built", "decisions"]
for line in open(os.path.join(out, "games.jsonl")):
    v = json.loads(line)
    print("seed", v["seed"], "fp", v["fingerprint"], "replay_ok", v["replay_ok"])
    for run in v["runs"]:
        extra = {k: run[k] for k in run if k not in keys and not isinstance(run[k], (list, dict))}
        print("  ", {k: run.get(k) for k in keys}, extra if run["run"] == 1 else "")
