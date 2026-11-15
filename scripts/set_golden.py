"""Copies the golden game's constants from `hh-sim golden` into the two pins
(crates/hh-core/src/tests/mod.rs and scripts/wasm-smoke.mjs), as the README's
*Rules changes* asks after an intended play change. Prints what it set.

    python scripts/set_golden.py
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main():
    out = subprocess.run(["cargo", "run", "-q", "--release", "-p", "hh-sim", "--", "golden"],
                         cwd=ROOT, capture_output=True, text=True, encoding="utf-8")
    m = re.search(r"revision (\d+) fingerprint ([0-9a-f]{16}) content ([0-9a-f]{16})", out.stdout)
    if out.returncode != 0 or not m:
        sys.exit(f"hh-sim golden failed: {out.stderr.strip() or out.stdout.strip()}")
    rev, fp, content = m.groups()
    edits = {
        ROOT / "crates" / "hh-core" / "src" / "tests" / "mod.rs": [
            (r"const GOLDEN_REVISION: u32 = \d+;", f"const GOLDEN_REVISION: u32 = {rev};"),
            (r'const GOLDEN_FINGERPRINT: &str = "[0-9a-f]{16}";', f'const GOLDEN_FINGERPRINT: &str = "{fp}";'),
            (r'const GOLDEN_CONTENT_HASH: &str = "[0-9a-f]{16}";', f'const GOLDEN_CONTENT_HASH: &str = "{content}";'),
        ],
        ROOT / "scripts" / "wasm-smoke.mjs": [
            (r"const GOLDEN_REVISION = \d+;", f"const GOLDEN_REVISION = {rev};"),
            (r"const GOLDEN_FINGERPRINT = '[0-9a-f]{16}';", f"const GOLDEN_FINGERPRINT = '{fp}';"),
            (r"const GOLDEN_CONTENT_HASH = '[0-9a-f]{16}';", f"const GOLDEN_CONTENT_HASH = '{content}';"),
        ],
    }
    for path, subs in edits.items():
        text = path.read_text(encoding="utf-8")
        for pat, new in subs:
            text, n = re.subn(pat, new, text)
            if n != 1:
                sys.exit(f"{path.relative_to(ROOT)}: expected one match for {pat}")
        path.write_text(text, encoding="utf-8", newline="\n")
    print(f"golden: revision {rev} fingerprint {fp} content {content}")


if __name__ == "__main__":
    main()
