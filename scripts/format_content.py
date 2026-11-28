"""Lay out every `content/*.json` file one row per line (Matt, 2026-11-27).

    python scripts/format_content.py           # rewrite any file not in the layout
    python scripts/format_content.py --check   # list files not in the layout; exit 1 if any

The layout: a top-level array, `[` on its own line, then each row as one line of JSON
(`", "` and `": "` separators, non-ASCII text written as itself, key order kept), indented two
spaces, a comma after each but the last, then `]` and a newline. The values never change: a
rewrite is checked to parse to the same data before it is written. hh-core's
`content_files_are_one_row_per_line` test holds the files to it.
"""

import json
import sys
from pathlib import Path

CONTENT = Path(__file__).resolve().parent.parent / 'content'


def layout(rows):
    if not rows:
        return '[\n]\n'
    lines = [json.dumps(r, ensure_ascii=False, separators=(', ', ': ')) for r in rows]
    return '[\n' + ',\n'.join('  ' + x for x in lines) + '\n]\n'


def main(argv):
    check = '--check' in argv
    off = []
    for p in sorted(CONTENT.glob('*.json')):
        text = p.read_text(encoding='utf-8')
        rows = json.loads(text)
        if not isinstance(rows, list):
            print(f'{p.name}: not a top-level array')
            return 2
        new = layout(rows)
        if new == text:
            continue
        if json.loads(new) != rows:
            print(f'{p.name}: the layout would change its data; left as it is')
            return 2
        off.append(p.name)
        if not check:
            p.write_text(new, encoding='utf-8', newline='\n')
    if check:
        for name in off:
            print(f'not one row per line: content/{name} (run python scripts/format_content.py)')
        return 1 if off else 0
    print(f'{len(off)} files rewritten: {", ".join(off) or "none"}')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
