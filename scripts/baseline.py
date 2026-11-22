"""A built copy of another revision, for reads against it (docs/experiments.md,
*Reads against another revision*).

    python scripts/baseline.py <rev> [--agents-from <rev2>]

Extracts <rev> (`git archive`, no worktree registered) to `out/baseline/<sha>/`
(replacing an earlier copy there), optionally lays <rev2>'s
`crates/hh-sim/src/agents/` over it (new rules, old agents: rev HEAD, rev2 main),
and builds its release `hh-sim` into `out/target-baseline` (absolute, its own
target dir). Prints the copy's root and binary: run the binary from that root,
since hh-sim reads `configs/`, `experiments/` and `reference/` from the working
directory. Uncommitted changes are not in the copy: commit or stash them first.
Builds at below-normal priority on 3 jobs; ~1 minute cold.
"""
import io
import os
import shutil
import subprocess
import sys
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def archive(rev, dest, paths=()):
    """Extracts `rev` (or only `paths` of it) into `dest`."""
    tar = subprocess.run(["git", "archive", "--format=tar", rev, *paths], cwd=ROOT,
                         capture_output=True, check=True).stdout
    with tarfile.open(fileobj=io.BytesIO(tar)) as t:
        t.extractall(dest, filter="data")


def main(argv):
    if not argv or argv[0].startswith("-"):
        sys.exit(__doc__)
    rev, agents = argv[0], None
    if len(argv) == 3 and argv[1] == "--agents-from":
        agents = argv[2]
    elif len(argv) != 1:
        sys.exit(__doc__)
    sha = subprocess.run(["git", "rev-parse", "--short", rev], cwd=ROOT, capture_output=True,
                         text=True, check=True).stdout.strip()
    name = sha + (f"-agents-{agents.replace('/', '_')}" if agents else "")
    dest = ROOT / "out" / "baseline" / name
    if dest.exists():
        shutil.rmtree(dest)
    dest.mkdir(parents=True)
    archive(rev, dest)
    if agents:
        shutil.rmtree(dest / "crates" / "hh-sim" / "src" / "agents")
        archive(agents, dest, ["crates/hh-sim/src/agents"])
    env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "out" / "target-baseline"), CARGO_BUILD_JOBS="3")
    cmd = ["cargo", "build", "--release", "-q", "-p", "hh-sim"]
    if os.name != "nt":
        cmd = ["nice", "-n", "10", *cmd]
    p = subprocess.Popen(cmd, cwd=dest, env=env,
                         creationflags=getattr(subprocess, "BELOW_NORMAL_PRIORITY_CLASS", 0))
    if p.wait() != 0:
        sys.exit("baseline: the build failed")
    exe = ROOT / "out" / "target-baseline" / "release" / ("hh-sim.exe" if os.name == "nt" else "hh-sim")
    # One target dir serves every copy: keep this build's binary beside its copy.
    kept = dest / exe.name
    shutil.copy2(exe, kept)
    print(f"root   {dest}")
    print(f"binary {kept}")
    print(f"run:   cd {dest} && ./{exe.name} ladder")


if __name__ == "__main__":
    main(sys.argv[1:])
