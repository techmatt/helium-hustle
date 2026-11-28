"""Every player-facing string in `content/`, out to one Markdown file and back.

    python scripts/prose.py export [FILE]   # write the strings (default content/prose.md)
    python scripts/prose.py import FILE     # write FILE's edited strings back into content/
    python scripts/prose.py check           # the round trip changes nothing, and
                                            # content/prose.md matches the content

The file is grouped by screen; each string is a `### \\`<file>:<id>.<field>\\` <fp>` heading
with its text under it, as many lines as it holds (a quest's `story` and `payoff` are one text
each: panels split by a line holding only `--`, blank lines between a panel's paragraphs;
leading and trailing blank lines are dropped on import). `<fp>` is the
fingerprint of the string as the content held it at export: the first 8 hex of the sha256 of
its UTF-8 text. Import matches strings by id and accepts a partial file (only the entries being
changed): an id missing from the file leaves its string alone and is listed as not in the file;
an id the content no longer has, an entry without a fingerprint, and one whose fingerprint no
longer matches the content (the string changed since that export) are each reported and
skipped. Only strings whose text changed are rewritten, in place in the JSON text, so every
other byte (layout, escapes) stays as it was. Ids are never edited. Import never needs anyone
to read the prose: the fingerprint is the check that an edit was made against the current text.

Covered: names, titles, summaries, quest goals and rewards, one-off events' effects and
bodies, every quest's story and a chain quest's payoff (one text each: panels split by a
`--` line, paragraphs by a blank line), descriptions,
reward descriptions, sub-objective labels, project reward texts and the messages the view
carries (`messages.json`: the run summary's line). Silent events have no
text and are left out. UI chrome written in `web/src` is not content and is not here.
"""

import hashlib
import json
import sys
import textwrap
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONTENT = ROOT / 'content'
DEFAULT_OUT = CONTENT / 'prose.md'

# File stem -> its screen, in the order the export lists them. Events split three ways.
SCREENS = [
    ('events:story', 'Story: the main quest line'),
    ('events:side', 'Story: side quests'),
    ('acts', 'Story: acts'),
    ('events:ongoing', 'Events (the activity feed and event modals)'),
    ('buildings', 'Buildings'),
    ('commands', 'Commands'),
    ('research', 'Research'),
    ('projects', 'Projects'),
    ('achievements', 'Achievements'),
    ('earth_tech', 'Earth Lab'),
    ('resources', 'Resources'),
    ('rivals', 'Adversaries'),
    ('nav', 'Navigation'),
    ('messages', 'Messages (the run summary)'),
]
# The content files read and written: SCREENS' stems, each once, in its order.
FILES = list(dict.fromkeys(stem.split(':')[0] for stem, _ in SCREENS))
TEXT = {'name', 'title', 'summary', 'goal', 'reward', 'effect', 'body', 'story', 'payoff',
        'description', 'reward_description', 'label', 'text'}


class Scanner:
    """A JSON reader that records where each string value sits in the text."""

    def __init__(self, text):
        self.s = text
        self.i = 0
        self.spans = []  # (path, start, end, value)

    def ws(self):
        while self.i < len(self.s) and self.s[self.i] in ' \t\r\n':
            self.i += 1

    def value(self, path):
        self.ws()
        c = self.s[self.i]
        if c == '{':
            self.i += 1
            out = {}
            self.ws()
            if self.s[self.i] == '}':
                self.i += 1
                return out
            while True:
                self.ws()
                key = self.string()
                self.ws()
                assert self.s[self.i] == ':', self.i
                self.i += 1
                out[key] = self.value(path + [key])
                self.ws()
                if self.s[self.i] == ',':
                    self.i += 1
                    continue
                assert self.s[self.i] == '}', self.i
                self.i += 1
                return out
        if c == '[':
            self.i += 1
            out = []
            self.ws()
            if self.s[self.i] == ']':
                self.i += 1
                return out
            while True:
                out.append(self.value(path + [len(out)]))
                self.ws()
                if self.s[self.i] == ',':
                    self.i += 1
                    continue
                assert self.s[self.i] == ']', self.i
                self.i += 1
                return out
        if c == '"':
            start = self.i
            v = self.string()
            self.spans.append((path, start, self.i, v))
            return v
        start = self.i
        while self.i < len(self.s) and self.s[self.i] not in ',]} \t\r\n':
            self.i += 1
        return json.loads(self.s[start:self.i])

    def string(self):
        assert self.s[self.i] == '"', self.i
        j = self.i + 1
        while self.s[j] != '"':
            j += 2 if self.s[j] == '\\' else 1
        v = json.loads(self.s[self.i:j + 1])
        self.i = j + 1
        return v


def scan(stem, text):
    """(key, screen, start, end, value) for each player-facing string of one file."""
    sc = Scanner(text)
    rows = sc.value([])
    out = []
    for path, start, end, value in sc.spans:
        if not path or not isinstance(path[0], int):
            continue
        row = rows[path[0]]
        if stem == 'events' and row.get('silent'):
            continue
        last = path[-1]
        if not (isinstance(last, str) and last in TEXT):
            continue
        # A row's id, then each step: an array element with an id is named by it.
        parts = []
        node = row
        for p in path[1:]:
            if isinstance(p, int) and isinstance(node[p], dict) and 'id' in node[p]:
                parts.append(str(node[p]['id']))
            else:
                parts.append(str(p))
            node = node[p]
        rid = row.get('id', row.get('act'))
        key = f'{stem}:{rid}.' + '.'.join(parts)
        screen = f'events:{row["category"]}' if stem == 'events' else stem
        out.append((key, screen, start, end, value))
    return out


def read_all():
    return {stem: (CONTENT / f'{stem}.json').read_text(encoding='utf-8') for stem in FILES}


def fingerprint(value):
    """The first 8 hex of the sha256 of a string's UTF-8 text."""
    return hashlib.sha256(value.encode('utf-8')).hexdigest()[:8]


def export(texts):
    by_screen = {s: [] for s, _ in SCREENS}
    for stem in FILES:
        for key, screen, _, _, value in scan(stem, texts[stem]):
            by_screen[screen].append((key, value))
    lines = [
        '# Helium Hustle: player-facing text',
        '',
        'Generated by `python scripts/prose.py export` from `content/`; edit the text under any',
        'heading and bring it back with `python scripts/prose.py import <this file>`. Never edit a',
        'heading: it is the string\'s id and the fingerprint of the text it was exported with. A',
        'file may carry only the entries being changed; a heading deleted leaves its string as it',
        'is, and an entry whose string changed since this export is skipped.',
    ]
    for screen, title in SCREENS:
        if not by_screen[screen]:
            continue
        lines += ['', f'## {title}']
        for key, value in by_screen[screen]:
            lines += ['', f'### `{key}` {fingerprint(value)}', '']
            if value:
                lines.append(value)
    return '\n'.join(lines) + '\n'


def parse(md):
    """{key: (fingerprint or None, text)} from an exported (and edited) file."""
    out = {}
    key = None
    fp = None
    body = []

    def close():
        if key is not None:
            while body and not body[0].strip():
                body.pop(0)
            while body and not body[-1].strip():
                body.pop()
            out[key] = (fp, '\n'.join(body))

    # Only the export's own headings end an entry: a text line of an entry that starts with
    # `#` stays in its body (134: it cut the entry there, unsaid).
    headings = {f'## {title}' for _, title in SCREENS}
    for line in md.lstrip('﻿').splitlines():
        head = line.rstrip()
        if head.startswith('### `') and '`' in head[5:]:
            close()
            end = head.index('`', 5)
            key = head[5:end]
            fp = head[end + 1:].strip() or None
            body = []
        elif head in headings or (key is None and (line.startswith('## ') or line.startswith('# '))):
            close()
            key = None
            body = []
        elif key is not None:
            body.append(line)
    close()
    return out


def apply(texts, entries):
    """The content texts with `entries` written in, and what was skipped or left out.

    Returns (texts, gone, unstamped, stale, absent): ids the content no longer has, entries
    without a fingerprint, entries whose fingerprint no longer matches the content, and ids
    the content has that `entries` doesn't carry.
    """
    known = []
    unstamped = []
    stale = []
    out = {}
    for stem in FILES:
        text = texts[stem]
        pieces = []
        pos = 0
        for key, _, start, end, value in scan(stem, text):
            known.append(key)
            if key not in entries:
                continue
            fp, new = entries[key]
            if fp is None:
                unstamped.append(key)
                continue
            if fp != fingerprint(value):
                stale.append(key)
                continue
            if new != value:
                pieces.append(text[pos:start])
                pieces.append(json.dumps(new, ensure_ascii=False))
                pos = end
        pieces.append(text[pos:])
        out[stem] = ''.join(pieces)
    known_set = set(known)
    gone = sorted(set(entries) - known_set)
    absent = [k for k in known if k not in entries]
    return out, gone, unstamped, stale, absent


def show(label, keys):
    """A count and the ids on wrapped lines, or nothing for none."""
    if not keys:
        return
    print(f'{label}: {len(keys)}')
    print(textwrap.fill(' '.join(keys), width=100, initial_indent='  ',
                        subsequent_indent='  ', break_on_hyphens=False))


def main(argv):
    if not argv or argv[0] not in ('export', 'import', 'check'):
        print(__doc__)
        return 2
    texts = read_all()
    if argv[0] == 'export':
        path = Path(argv[1]) if len(argv) > 1 else DEFAULT_OUT
        path.write_text(export(texts), encoding='utf-8', newline='\n')
        print(f'wrote {path}')
        return 0
    if argv[0] == 'import':
        if len(argv) < 2:
            print('import needs the edited file')
            return 2
        entries = parse(Path(argv[1]).read_text(encoding='utf-8'))
        new, gone, unstamped, stale, absent = apply(texts, entries)
        for k in gone:
            print(f'skipped, id gone from content: {k}')
        for k in unstamped:
            print(f'skipped, no fingerprint beside the id (re-export and edit that): {k}')
        for k in stale:
            print(f'skipped, fingerprint no longer matches (the string changed since that export): {k}')
        # New since the export: ids in content now that the exported content/prose.md
        # (the file partial edits are cut from) doesn't carry.
        if DEFAULT_OUT.exists():
            exported = parse(DEFAULT_OUT.read_text(encoding='utf-8'))
            show('ids new since the export (not in content/prose.md: export again to edit them)',
                 [k for k in absent if k not in exported])
        show('ids in content but not in this file (left as they are)', absent)
        changed =[s for s in FILES if new[s] != texts[s]]
        for stem in changed:
            json.loads(new[stem])
            (CONTENT / f'{stem}.json').write_text(new[stem], encoding='utf-8', newline='\n')
        skipped = len(gone) + len(unstamped) + len(stale)
        print(f'{len(entries)} strings read, {skipped} skipped, {len(changed)} files changed: '
              f'{", ".join(changed) or "none"}')
        return 0
    md = export(texts)
    new, gone, unstamped, stale, absent = apply(texts, parse(md))
    bad = [s for s in FILES if new[s] != texts[s]] + gone + unstamped + stale + absent
    if bad:
        print(f'round trip changed: {bad}')
        return 1
    if not DEFAULT_OUT.exists() or DEFAULT_OUT.read_text(encoding='utf-8') != md:
        print('content/prose.md is stale: run python scripts/prose.py export')
        return 1
    print('round trip clean; content/prose.md current')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
