"""Warning checks (CLAUDE.md, *Warning checks*). Prints one `<kind>: <text>` line per
tripped check and nothing when clean; always exits 0 (warnings never gate).

    python scripts/warning_checks.py            the static checks, and parity and balance when
                                                the branch touched what they measure
    python scripts/warning_checks.py --all      every check, whatever was touched
    python scripts/warning_checks.py --no-parity --no-balance  static checks only (no builds)

Checks:
- parity: the golden game's fingerprint natively (`hh-sim golden`) and in wasm
  (`scripts/wasm-smoke.mjs`) both equal the pinned constants, and the pins agree.
  Builds the page's wasm (web/src/wasm, `web/scripts/build-wasm.mjs`) first; that
  script skips the build when its inputs are unchanged.
- tunables: numeric literals in hh-core rule code (not tests) other than those
  `scripts/warning_checks.json` allows: they belong in a config or content.
- platform-floats: a platform float function reaching hh-core past clippy (any
  call by name not through `libm::`, an `#[allow]` of the ban, the ban's `deny`
  removed, a name missing from clippy.toml).
- docs: a doc (README, CLAUDE.md, web/README and TESTING, every page under docs/) over 15 KB, or
  inside the 1 KB margin below it (over 14,336 bytes, `wc -c`); or `content/prose.md` (a
  generated file, exempt from the cap) out of date with `content/`, or its round trip not clean.
- balance: each target the judge misses (`hh-sim balance` on experiments/balance.json,
  the judge's one home; targets in docs/principles.md).
- qa: the last `npm run qa` in this tree (`out/qa/report.json`, if any) had a pass that
  never reached the ending (its `ended` false): qa stalled unnoticed from 071 to 082.
"""

import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CORE = ROOT / "crates" / "hh-core"
SRC = CORE / "src"

FLOAT_FNS = [
    "powf", "powi", "exp", "exp2", "exp_m1", "ln", "ln_1p", "log", "log2", "log10",
    "sqrt", "cbrt", "hypot", "sin", "cos", "tan", "sin_cos", "asin", "acos", "atan",
    "atan2", "sinh", "cosh", "tanh", "asinh", "acosh", "atanh", "mul_add",
]

out = []


def warn(kind, text):
    out.append(f"{kind}: {text}")


def rel(p):
    return p.relative_to(ROOT).as_posix()


# A char literal (not a lifetime): 'a', '\n', '\x41', '\u{2014}'.
CHAR = r"'(\\u\{[0-9a-fA-F_]+\}|\\x[0-9a-fA-F]{2}|\\.|[^\\'])'"


def strip_code(text):
    """Blanks comments and string/char literals, keeping line numbers."""
    res = []
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        if text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            res.append(" " * (j - i))
            i = j
        elif text.startswith("/*", i):
            # Rust block comments nest.
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            res.append(re.sub(r"[^\n]", " ", text[i:j]))
            i = j
        elif c == "r" and re.match(r'r#*"', text[i:]):
            hashes = re.match(r"r(#*)\"", text[i:]).group(1)
            end = '"' + hashes
            j = text.find(end, i + 2 + len(hashes))
            j = n if j < 0 else j + len(end)
            res.append(re.sub(r"[^\n]", " ", text[i:j]))
            i = j
        elif c == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            j = min(j + 1, n)
            res.append(re.sub(r"[^\n]", " ", text[i:j]))
            i = j
        elif c == "'" and re.match(CHAR, text[i:]):
            m = re.match(CHAR, text[i:])
            res.append(" " * len(m.group(0)))
            i += len(m.group(0))
        else:
            res.append(c)
            i += 1
    return "".join(res)


def drop_cfg_test(code):
    """Blanks `#[cfg(test)] mod x { ... }` blocks."""
    m = re.search(r"#\[cfg\(test\)\]\s*mod\s+\w+\s*\{", code)
    while m:
        depth, j = 1, m.end()
        while j < len(code) and depth:
            depth += {"{": 1, "}": -1}.get(code[j], 0)
            j += 1
        code = code[: m.start()] + re.sub(r"[^\n]", " ", code[m.start():j]) + code[j:]
        m = re.search(r"#\[cfg\(test\)\]\s*mod\s+\w+\s*\{", code)
    return code


def rule_sources(stripped=True):
    """(path, text) for hh-core's non-test sources: comments, strings and
    `#[cfg(test)]` modules blanked; with `stripped=False`, the raw text (for
    the string-literal check)."""
    for p in sorted(SRC.rglob("*.rs")):
        if "tests" in p.relative_to(SRC).parts:
            continue
        raw = p.read_text(encoding="utf-8")
        yield p, drop_cfg_test(strip_code(raw)) if stripped else raw


SUFFIX = r"(?:_?(?:[iu](?:8|16|32|64|128|size)|f32|f64))?"
# A numeric literal: not part of a word, not a tuple index (`x.0`) but both
# bounds of a range (`0..365`), and a method call on a literal (`7.min(n)`).
NUM = re.compile(
    r"(?<!\w)(?<![\w)\]]\.)"
    r"(0x[0-9a-fA-F_]+|0o[0-7_]+|0b[01_]+|\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?)"
    + SUFFIX + r"(?!\w)(?!\.\d)"
)
# A decimal in a string handed to a number parser: `Fx::parse("0.05")`.
STR_NUM = re.compile(r"(?:Fx::parse|\.parse::<[^>]+>)\(\s*\"\s*(-?\d[\d_]*(?:\.\d+)?)\s*\"")


def canonical(lit):
    lit = lit.replace("_", "")
    for prefix, base in (("0x", 16), ("0o", 8), ("0b", 2)):
        if lit.startswith(prefix):
            return str(int(lit[2:], base))
    return lit


def check_tunables(settings):
    t = settings["tunables"]
    allowed = set(t["allowed_literals"])
    exempt = set(t["exempt_files"])
    exceptions = {(e["file"], e["literal"]) for e in t["exceptions"]}

    def report(f, ln, shown, lit):
        if lit in allowed or (f, lit) in exceptions:
            return
        warn("tunables", f"{f}:{ln}: literal {shown} in rule code "
             "(move it to a config or content, or list it in scripts/warning_checks.json)")

    for p, code in rule_sources():
        f = rel(p)
        if f in exempt:
            continue
        for ln, line in enumerate(code.splitlines(), 1):
            # Attributes, and the rules revision itself (a version, not a tunable; a bump
            # needs no exception moved).
            if line.lstrip().startswith("#[") or re.match(r"\s*pub const RULES_REVISION\b", line):
                continue
            for m in NUM.finditer(line):
                report(f, ln, m.group(0), canonical(m.group(1)))
    for p, raw in rule_sources(stripped=False):
        f = rel(p)
        if f in exempt:
            continue
        for ln, line in enumerate(raw.splitlines(), 1):
            if line.lstrip().startswith("//"):
                continue
            for m in STR_NUM.finditer(line):
                report(f, ln, f'"{m.group(1)}"', m.group(1).replace("_", ""))


def check_floats():
    names = "|".join(FLOAT_FNS)
    # A method call by name (`x.powf(`), or a path to one, called or passed (`f64::ln`).
    call = re.compile(rf"(?<!libm::)(?:\.({names})\s*\(|\b(?:f32|f64)::({names})\b)")
    silence = re.compile(r"#!?\[\s*(allow|expect|warn)\s*\([^)]*clippy::(disallowed_\w+|style|all)\b")
    for p, code in rule_sources():
        f = rel(p)
        for ln, line in enumerate(code.splitlines(), 1):
            for m in call.finditer(line):
                warn("platform-floats", f"{f}:{ln}: `{m.group(0).rstrip('(').strip()}` (use crate::math, which calls libm)")
            if silence.search(line):
                warn("platform-floats", f"{f}:{ln}: an attribute silencing clippy's float/clock/hash-map ban")
            if re.search(r"\bnum_traits\b|\bextern\s+\"C\"|core::intrinsics|std::intrinsics", line):
                warn("platform-floats", f"{f}:{ln}: a float path around the ban ({line.strip()})")
    toml = (CORE / "clippy.toml").read_text(encoding="utf-8")
    for ty in ("f64", "f32"):
        for fn in FLOAT_FNS:
            if f'"{ty}::{fn}"' not in toml:
                warn("platform-floats", f"crates/hh-core/clippy.toml doesn't ban {ty}::{fn}")
    cargo = (CORE / "Cargo.toml").read_text(encoding="utf-8")
    if not re.search(r'disallowed_methods\s*=\s*"deny"', cargo):
        warn("platform-floats", "crates/hh-core/Cargo.toml no longer denies clippy::disallowed_methods")


def run(cmd, **kw):
    """Runs a tool; a missing one is a failed run (exit 127), never an exception."""
    exe = shutil.which(cmd[0]) or cmd[0]
    # A relative CARGO_TARGET_DIR would resolve against each tool's own directory (the wasm
    # build runs cargo in crates/hh-wasm, leaving a stray target there): pin it to the root.
    env = dict(os.environ)
    if env.get("CARGO_TARGET_DIR") and not os.path.isabs(env["CARGO_TARGET_DIR"]):
        env["CARGO_TARGET_DIR"] = str(ROOT / env["CARGO_TARGET_DIR"])
    try:
        return subprocess.run([exe, *cmd[1:]], cwd=ROOT, capture_output=True, text=True,
                              encoding="utf-8", errors="replace", env=env, **kw)
    except OSError as e:
        return subprocess.CompletedProcess(cmd, 127, "", f"{cmd[0]}: {e}")


def check_parity():
    core_tests = (SRC / "tests" / "mod.rs").read_text(encoding="utf-8")
    smoke = (ROOT / "scripts" / "wasm-smoke.mjs").read_text(encoding="utf-8")
    pin = lambda pat, text: (re.search(pat, text) or [None, None])[1]
    native_pin = pin(r'GOLDEN_FINGERPRINT: &str = "([0-9a-f]{16})"', core_tests)
    wasm_pin = pin(r"GOLDEN_FINGERPRINT = '([0-9a-f]{16})'", smoke)
    if native_pin is None or native_pin != wasm_pin:
        warn("parity", f"the golden pins disagree: hh-core {native_pin}, wasm-smoke {wasm_pin}")
    # --release, as the balance check and set_golden.py: one build serves all three.
    r = run(["cargo", "run", "-q", "--release", "-p", "hh-sim", "--", "golden"])
    m = re.search(r"fingerprint ([0-9a-f]{16})", r.stdout)
    if r.returncode != 0 or not m:
        warn("parity", f"native golden run failed: {(r.stderr or r.stdout).strip().splitlines()[-1:]}")
    elif m.group(1) != native_pin:
        warn("parity", f"native golden fingerprint {m.group(1)}, pinned {native_pin} (a rules change without its golden update?)")
    # The page's build, web/src/wasm: build-wasm.mjs skips in under a second when a hash
    # of its inputs is unchanged, and needs only node and wasm-pack (no web/node_modules).
    b = run(["node", "web/scripts/build-wasm.mjs"])
    if b.returncode != 0:
        tail = (b.stderr.strip() or b.stdout.strip()).splitlines()[-2:]
        warn("parity", "wasm build failed: " + " ".join(tail))
        return
    s = run(["node", "scripts/wasm-smoke.mjs"])
    if s.returncode != 0:
        fails = [l for l in s.stdout.splitlines() if l.startswith("FAIL")] or s.stderr.strip().splitlines()[-1:]
        warn("parity", "native != wasm: " + "; ".join(l.removeprefix("FAIL ").strip() for l in fails))


DOC_CAP = 15 * 1024
DOC_MARGIN = 1024


def check_docs():
    """Each doc under about 15 KB (CLAUDE.md, *Docs rules*): over the cap is a warning,
    and so is a doc inside the 1 KB margin the rules ask to leave free."""
    docs = [ROOT / "README.md", ROOT / "CLAUDE.md", ROOT / "web" / "README.md", ROOT / "web" / "TESTING.md",
            *sorted((ROOT / "docs").rglob("*.md"))]
    for p in docs:
        n = p.stat().st_size
        if n > DOC_CAP:
            warn("docs", f"{rel(p)} is {n:,} B, over the 15 KB cap")
        elif n > DOC_CAP - DOC_MARGIN:
            warn("docs", f"{rel(p)} is {n:,} B, inside the 1 KB margin (trim it before adding)")
    prose = subprocess.run([sys.executable, str(ROOT / "scripts" / "prose.py"), "check"],
                           capture_output=True, text=True, encoding="utf-8")
    if prose.returncode != 0:
        warn("docs", f"content/prose.md: {prose.stdout.strip() or prose.stderr.strip()}")


def check_balance():
    """The judge's cells (experiments/balance.json) against the targets in
    configs/sim/milestones.json: `hh-sim balance` prints the `balance:` lines.
    Its cost: docs/experiments.md, *Costs*."""
    r = run(["cargo", "run", "--release", "-q", "-p", "hh-sim", "--", "balance"], timeout=300)
    if r.returncode != 0:
        warn("balance", f"hh-sim balance failed: {(r.stderr or r.stdout).strip().splitlines()[-1:]}")
        return
    for line in r.stdout.splitlines():
        if line.startswith("balance: "):
            out.append(line)


def check_qa():
    """The last qa run's passes each reached the ending (web/scripts/qa.mjs writes
    `ended` per pass; a report from before 089 has none and is skipped)."""
    p = ROOT / "out" / "qa" / "report.json"
    if not p.exists():
        return
    passes = json.loads(p.read_text(encoding="utf-8")).get("passes", [])
    short = [x for x in passes if x.get("ended") is False]
    if short:
        tags = ", ".join(f"seed {x['seed']} {x['width']} {x['theme']}" for x in short)
        warn("qa", f"{len(short)} of {len(passes)} qa passes never reached the ending ({tags}): {rel(p)}")


# What each heavy check measures: it runs only when the branch (commits since it left
# main, and the work tree) touched one of these, or with --all. An item that changed
# none of them would read main's numbers, at the cost of a release and a wasm build.
PARITY_PATHS = ("crates/hh-core/", "crates/hh-wasm/", "content/", "scenarios/", "configs/baseline.json",
                "scripts/wasm-smoke.mjs", "web/scripts/build-wasm.mjs", "Cargo.")
BALANCE_PATHS = ("crates/hh-core/", "crates/hh-sim/", "content/", "scenarios/", "configs/",
                 "experiments/balance.json", "Cargo.")


def touched():
    """Paths changed since the branch left main, committed or not; None when git
    can't say (then every check runs)."""
    git = lambda *a: run(["git", *a])
    base = git("merge-base", "HEAD", "main")
    if base.returncode != 0:
        return None
    diff = git("diff", "--name-only", base.stdout.strip())
    new = git("ls-files", "--others", "--exclude-standard")
    if diff.returncode != 0 or new.returncode != 0:
        return None
    return set(diff.stdout.split()) | set(new.stdout.split())


def wanted(flag, paths, changed):
    if flag in sys.argv[1:]:
        return False
    if "--all" in sys.argv[1:] or changed is None:
        return True
    return any(p.startswith(paths) for p in changed)


def main():
    sys.stdout.reconfigure(encoding="utf-8", newline="\n")
    try:
        settings = json.loads((ROOT / "scripts" / "warning_checks.json").read_text(encoding="utf-8"))
        check_tunables(settings)
        check_floats()
        check_docs()
        check_qa()
        changed = touched()
        if wanted("--no-parity", PARITY_PATHS, changed):
            check_parity()
        if wanted("--no-balance", BALANCE_PATHS, changed):
            check_balance()
    except Exception as e:  # a broken check is itself a warning; never gate
        warn("checks", f"scripts/warning_checks.py failed: {type(e).__name__}: {e}")
    finally:
        for line in out:
            print(line)


if __name__ == "__main__":
    main()
