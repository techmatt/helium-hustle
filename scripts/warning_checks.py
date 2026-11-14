"""Warning checks (CLAUDE.md, *Warning checks*). Prints one `<kind>: <text>` line per
tripped check and nothing when clean; always exits 0 (warnings never gate).

    python scripts/warning_checks.py            all three checks
    python scripts/warning_checks.py --no-parity  skip the builds (static checks only)

Checks:
- parity: the golden game's fingerprint natively (`hh-sim golden`) and in wasm
  (`scripts/wasm-smoke.mjs`) both equal the pinned constants, and the pins agree.
  Rebuilds the wasm package first when it is older than the engine sources.
- tunables: numeric literals in hh-core rule code (not tests) other than those
  `scripts/warning_checks.json` allows: they belong in a config or content.
- platform-floats: a platform float function reaching hh-core past clippy (any
  call by name not through `libm::`, an `#[allow]` of the ban, the ban's `deny`
  removed, a name missing from clippy.toml).
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
            if line.lstrip().startswith("#["):
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
    for fn in FLOAT_FNS:
        if f'"f64::{fn}"' not in toml:
            warn("platform-floats", f"crates/hh-core/clippy.toml doesn't ban f64::{fn}")
    cargo = (CORE / "Cargo.toml").read_text(encoding="utf-8")
    if not re.search(r'disallowed_methods\s*=\s*"deny"', cargo):
        warn("platform-floats", "crates/hh-core/Cargo.toml no longer denies clippy::disallowed_methods")


def run(cmd, **kw):
    """Runs a tool; a missing one is a failed run (exit 127), never an exception."""
    exe = shutil.which(cmd[0]) or cmd[0]
    try:
        return subprocess.run([exe, *cmd[1:]], cwd=ROOT, capture_output=True, text=True,
                              encoding="utf-8", errors="replace", **kw)
    except OSError as e:
        return subprocess.CompletedProcess(cmd, 127, "", f"{cmd[0]}: {e}")


def newest(paths):
    best = 0.0
    for base in paths:
        if base.is_file():
            best = max(best, base.stat().st_mtime)
        elif base.is_dir():
            for p in base.rglob("*"):
                if p.is_file():
                    best = max(best, p.stat().st_mtime)
    return best


def check_parity():
    core_tests = (SRC / "tests" / "mod.rs").read_text(encoding="utf-8")
    smoke = (ROOT / "scripts" / "wasm-smoke.mjs").read_text(encoding="utf-8")
    pin = lambda pat, text: (re.search(pat, text) or [None, None])[1]
    native_pin = pin(r'GOLDEN_FINGERPRINT: &str = "([0-9a-f]{16})"', core_tests)
    wasm_pin = pin(r"GOLDEN_FINGERPRINT = '([0-9a-f]{16})'", smoke)
    if native_pin is None or native_pin != wasm_pin:
        warn("parity", f"the golden pins disagree: hh-core {native_pin}, wasm-smoke {wasm_pin}")
    r = run(["cargo", "run", "-q", "-p", "hh-sim", "--", "golden"])
    m = re.search(r"fingerprint ([0-9a-f]{16})", r.stdout)
    if r.returncode != 0 or not m:
        warn("parity", f"native golden run failed: {(r.stderr or r.stdout).strip().splitlines()[-1:]}")
    elif m.group(1) != native_pin:
        warn("parity", f"native golden fingerprint {m.group(1)}, pinned {native_pin} (a rules change without its golden update?)")
    wasm = ROOT / "crates" / "hh-wasm" / "pkg" / "hh_wasm_bg.wasm"
    sources = [CORE, ROOT / "crates" / "hh-wasm" / "src", ROOT / "crates" / "hh-wasm" / "Cargo.toml",
               ROOT / "content", ROOT / "scenarios", ROOT / "Cargo.toml", ROOT / "Cargo.lock"]
    if not wasm.exists() or wasm.stat().st_mtime < newest(sources):
        b = run(["wasm-pack", "build", "crates/hh-wasm", "--target", "web", "--release"])
        if b.returncode != 0:
            warn("parity", "wasm build failed: " + " ".join(b.stderr.strip().splitlines()[-2:]))
            return
    s = run(["node", "scripts/wasm-smoke.mjs"])
    if s.returncode != 0:
        fails = [l for l in s.stdout.splitlines() if l.startswith("FAIL")] or s.stderr.strip().splitlines()[-1:]
        warn("parity", "native != wasm: " + "; ".join(l.removeprefix("FAIL ").strip() for l in fails))


def main():
    sys.stdout.reconfigure(encoding="utf-8", newline="\n")
    try:
        settings = json.loads((ROOT / "scripts" / "warning_checks.json").read_text(encoding="utf-8"))
        check_tunables(settings)
        check_floats()
        if "--no-parity" not in sys.argv[1:]:
            check_parity()
    except Exception as e:  # a broken check is itself a warning; never gate
        warn("checks", f"scripts/warning_checks.py failed: {type(e).__name__}: {e}")
    finally:
        for line in out:
            print(line)


if __name__ == "__main__":
    main()
