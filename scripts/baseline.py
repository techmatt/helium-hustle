"""A built copy of another revision, for reads against it (docs/experiments.md,
*Reads against another revision*).

    python scripts/baseline.py <rev> [--agents-from <rev2>]
    python scripts/baseline.py <rev> --web

--web (a UI before/after) builds the copy's web page instead of hh-sim: its
`web/node_modules` a junction to this tree's (run `npm ci` in web/ first; a base whose
package-lock differs needs its own), its wasm (`web/scripts/build-wasm.mjs`, target dir
`out/target-baseline-wasm`, ~80 s cold) and `vite build` into its `web/dist`, then prints
the `vite preview` line (Git Bash) to serve it on a port of your choosing: its play logs
go to the copy's `out/playlogs/`, stamped commit `unknown` (never this tree's).

Extracts <rev> (`git archive`, no worktree registered) to `out/baseline/<sha>/`, or
`out/baseline/<sha>-agents-<rev2>/` with --agents-from (replacing an earlier copy
there), optionally lays <rev2>'s
`crates/hh-sim/src/agents/` over it (new rules, old agents: rev HEAD, rev2 main),
and builds its release `hh-sim` into `out/target-baseline` (absolute, its own
target dir). Prints the copy's root and binary: run the binary from that root,
since hh-sim reads `configs/`, `experiments/` and `reference/` from the working
directory. Uncommitted changes are not in the copy: commit them first (a WIP commit:
the stash stack is shared by every worktree).
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


def build(cmd, cwd, env, what):
    """Runs `cmd` at below-normal priority; exits on failure."""
    if os.name != "nt":
        cmd = ["nice", "-n", "10", *cmd]
    p = subprocess.Popen(cmd, cwd=cwd, env=env,
                         creationflags=getattr(subprocess, "BELOW_NORMAL_PRIORITY_CLASS", 0))
    if p.wait() != 0:
        sys.exit(f"baseline: {what} failed")


def unlink_modules(dest):
    """Removes the copy's `web/node_modules` link itself, never what it points at (an
    rmtree through it would empty this tree's node_modules)."""
    link = dest / "web" / "node_modules"
    if link.is_symlink():
        os.unlink(link)
    elif os.path.isjunction(link):
        os.rmdir(link)


def web(dest):
    """Builds the copy's page (--web): node_modules linked from this tree, its own wasm."""
    modules = ROOT / "web" / "node_modules"
    if not (modules / "vite").exists():
        sys.exit("baseline: --web needs this tree's web/node_modules (npm ci in web/)")
    link = dest / "web" / "node_modules"
    if os.name == "nt":
        subprocess.run(["cmd", "/c", "mklink", "/J", str(link), str(modules)], check=True, capture_output=True)
    else:
        os.symlink(modules, link)
    env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "out" / "target-baseline-wasm"), CARGO_BUILD_JOBS="3")
    build(["node", "web/scripts/build-wasm.mjs"], dest, env, "the wasm build")
    vite = str(link / "vite" / "bin" / "vite.js")
    build(["node", vite, "build"], dest / "web", env, "vite build")
    print(f"root    {dest}")
    print(f"page    {dest / 'web' / 'dist'}")
    # The copy sits inside this work tree: unfenced, its play-log endpoint would stamp
    # this tree's commit on the base's logs and file people's play in the synced folder.
    # Fenced, git finds no repo (commit "unknown") and its logs stay in the copy.
    env_line = f"HH_PLAYLOG_DIR='{dest / 'out' / 'playlogs'}' GIT_CEILING_DIRECTORIES='{dest.parent}'"
    print(f"serve:  cd '{dest / 'web'}' && {env_line} node node_modules/vite/bin/vite.js preview --port <port> --strictPort")


def main(argv):
    if not argv or argv[0].startswith("-"):
        sys.exit(__doc__)
    rev, agents, page = argv[0], None, False
    if len(argv) == 3 and argv[1] == "--agents-from":
        agents = argv[2]
    elif len(argv) == 2 and argv[1] == "--web":
        page = True
    elif len(argv) != 1:
        sys.exit(__doc__)
    sha = subprocess.run(["git", "rev-parse", "--short", rev], cwd=ROOT, capture_output=True,
                         text=True, check=True).stdout.strip()
    name = sha + (f"-agents-{agents.replace('/', '_')}" if agents else "") + ("-web" if page else "")
    dest = ROOT / "out" / "baseline" / name
    if dest.exists():
        unlink_modules(dest)
        shutil.rmtree(dest)
    dest.mkdir(parents=True)
    archive(rev, dest)
    if page:
        web(dest)
        return
    if agents:
        shutil.rmtree(dest / "crates" / "hh-sim" / "src" / "agents")
        archive(agents, dest, ["crates/hh-sim/src/agents"])
    env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "out" / "target-baseline"), CARGO_BUILD_JOBS="3")
    build(["cargo", "build", "--release", "-q", "-p", "hh-sim"], dest, env, "the build")
    exe = ROOT / "out" / "target-baseline" / "release" / ("hh-sim.exe" if os.name == "nt" else "hh-sim")
    # One target dir serves every copy: keep this build's binary beside its copy.
    kept = dest / exe.name
    shutil.copy2(exe, kept)
    print(f"root   {dest}")
    print(f"binary {kept}")
    print(f"run:   cd {dest} && ./{exe.name} ladder")


if __name__ == "__main__":
    main(sys.argv[1:])
