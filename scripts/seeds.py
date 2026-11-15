"""Fresh seeds from the seed ledger in docs/simulator.md (*Seeds*).

    python scripts/seeds.py next N [--take] [--doc FILE]

Prints FIRST-LAST, the N seeds from the ledger's `Next free: NNNN`. With `--take` it also moves
that number to LAST + 1 in the ledger and changes nothing else: the item still writes its range
in its write-up and in the ledger's spent list. Every seed below *Next free* counts as spent.
`--doc` reads another file (a test copy). Ported from AIComplete's `scripts/bench/seeds.py`.
"""

import os
import re
import sys

DOC = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "docs", "simulator.md")
LEDGER = re.compile(r"Next free: (\d+)")


def next_range(text, n):
    """(first, last, the ledger's match) for `n` seeds out of `text`."""
    found = list(LEDGER.finditer(text))
    if len(found) != 1:
        sys.exit(f"the ledger needs one 'Next free: NNNN' line, found {len(found)}")
    m = found[0]
    first = int(m.group(1))
    return first, first + n - 1, m


def main(argv):
    doc = DOC
    if "--doc" in argv:
        i = argv.index("--doc")
        if i + 1 == len(argv):
            sys.exit("--doc needs a value")
        doc = argv[i + 1]
        del argv[i:i + 2]
    take = "--take" in argv
    argv = [a for a in argv if a != "--take"]
    if len(argv) != 2 or argv[0] != "next" or not argv[1].isdigit() or int(argv[1]) < 1:
        sys.exit(__doc__)
    with open(doc, encoding="utf-8") as f:
        text = f.read()
    first, last, m = next_range(text, int(argv[1]))
    print(f"{first}-{last}")
    if take:
        text = text[:m.start(1)] + str(last + 1) + text[m.end(1):]
        with open(doc, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
        print(f"ledger: next free {last + 1}", file=sys.stderr)


if __name__ == "__main__":
    main(sys.argv[1:])
