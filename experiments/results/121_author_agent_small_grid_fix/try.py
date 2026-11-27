"""Play author and handsoff on seeds 1..N, 5 retirements; print each run's summary.
Usage: python try.py <tag> [seeds] [retirements] [agents]"""
import json, os, subprocess, sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
tag = sys.argv[1]
seeds = sys.argv[2] if len(sys.argv) > 2 else "4"
rets = sys.argv[3] if len(sys.argv) > 3 else "5"
agents = sys.argv[4] if len(sys.argv) > 4 else "author,handsoff"
out = os.path.join(ROOT, "out", "121_author_agent_small_grid_fix", "runs", f"{tag}-{seeds}-{rets}")
a = [os.path.join(ROOT, "target", "release", "hh-sim.exe"), "experiment", os.path.join(ROOT, "experiments", "balance.json"),
     "--human-models", os.path.join(ROOT, "configs", "sim", "humans.json"),
     "--milestones", os.path.join(ROOT, "configs", "sim", "milestones.json"),
     "--config", os.path.join(ROOT, "configs", "baseline.json"),
     "--agents", agents, "--humans", "mixed", "--seeds", seeds, "--retirements", rets, "--out", out, "--threads", "6"]
r = subprocess.run(a, capture_output=True, text=True)
if r.returncode:
    print(r.stdout[-2000:], r.stderr[-3000:])
    sys.exit(1)
for line in open(os.path.join(out, "games.jsonl")):
    v = json.loads(line)
    projs = v.get("projects")
    print(v["cell"]["agent"], "seed", v["seed"], "fp", v["fingerprint"], "replay_ok", v["replay_ok"], "career projects", projs)
    for run in v["runs"]:
        print("   run", run.get("run"), "ticks", run.get("ticks"), "ships", run.get("shipments"),
              "built", run.get("buildings_built"), "credits", run.get("credits"), "projects", run.get("projects"))
