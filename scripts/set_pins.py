"""Copies the sim players' pinned fingerprints into crates/hh-sim/tests/cli.rs
from the games the pinning tests just wrote under out/hh-sim-tests/, as the
README's *Rules changes* asks after a change that moves play (`set_golden.py`
does the golden game). Run the tests first, failing pins and all:

    cargo test-all -p hh-sim
    python scripts/set_pins.py

Sets `PINNED` (the ladder's players, from ladder/games.jsonl), `HANDSOFF_PIN`
(handsoff/games.jsonl) and the oracle's four (oracle/games.jsonl, written by a
slow-lane test: `cargo test-all` runs it). A pin whose games file is missing is
left as is and named. Prints what it set.
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CLI = ROOT / "crates" / "hh-sim" / "tests" / "cli.rs"
GAMES = ROOT / "out" / "hh-sim-tests"


def games(name):
    p = GAMES / name / "games.jsonl"
    if not p.exists():
        print(f"{name}: no {p.relative_to(ROOT)}, pin left as is")
        return None
    return [json.loads(l) for l in p.read_text(encoding="utf-8").splitlines() if l.strip()]


def sub_one(text, pat, new, what):
    text, n = re.subn(pat, lambda _: new, text, flags=re.S)
    if n != 1:
        sys.exit(f"cli.rs: expected one {what}")
    return text


def main():
    text = CLI.read_text(encoding="utf-8")
    ladder = games("ladder")
    if ladder:
        rows = "".join(f'    ("{g["cell"]["agent"]}", {g["seed"]}, "{g["fingerprint"]}"),\n' for g in ladder)
        text = sub_one(text, r"const PINNED: &\[\(&str, u64, &str\)\] = &\[\n.*?\n\];",
                       "const PINNED: &[(&str, u64, &str)] = &[\n" + rows + "];", "PINNED")
        print("PINNED:\n" + rows.rstrip())
    handsoff = games("handsoff")
    if handsoff:
        fp = handsoff[0]["fingerprint"]
        text = sub_one(text, r'const HANDSOFF_PIN: &str = "[0-9a-f]{16}";', f'const HANDSOFF_PIN: &str = "{fp}";', "HANDSOFF_PIN")
        print(f"HANDSOFF_PIN: {fp}")
    oracle = games("oracle")
    if oracle:
        fps = ", ".join(f'"{g["fingerprint"]}"' for g in oracle)
        text = sub_one(text, r"assert_eq!\(fingerprints\[\.\.half\], \[[^\]]*\]\);",
                       f"assert_eq!(fingerprints[..half], [{fps}]);", "oracle pin")
        print(f"oracle: {fps}")
    CLI.write_bytes(text.encode("utf-8"))


if __name__ == "__main__":
    main()
