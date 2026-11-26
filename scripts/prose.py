"""Every player-facing string in `content/`, out to one Markdown file and back.

    python scripts/prose.py export [FILE]   # write the strings (default content/prose.md)
    python scripts/prose.py import FILE     # write FILE's edited strings back into content/
    python scripts/prose.py check           # the round trip changes nothing, and
                                            # content/prose.md matches the content

The file is grouped by screen; each string is a `### \\`<file>:<id>.<field>\\`` heading with
its text under it (a `story` or `payoff` paragraph adds its index). Import matches strings by
that id only: an id missing from the file leaves its string alone, an id the content no longer
has is reported and skipped, and only strings whose text changed are rewritten, in place in
the JSON text, so every other byte (layout, escapes) stays as it was. Ids are never edited.

Covered: names, titles, summaries, quest goals and rewards, one-off events' effects,
bodies, quest story and
payoff paragraphs, descriptions,
reward descriptions, sub-objective labels, project reward texts and the messages the view
carries (`messages.json`: the run summary's line). Silent events have no
text and are left out. UI chrome written in `web/src` is not content and is not here.
"""

import json
import sys
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
TEXT = {'name', 'title', 'summary', 'goal', 'reward', 'effect', 'body', 'description', 'reward_description', 'label', 'text'}
PARAGRAPHS = {'story', 'payoff'}


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
        prose = (isinstance(last, str) and last in TEXT) or (
            isinstance(last, int) and len(path) >= 2 and path[-2] in PARAGRAPHS)
        if not prose:
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
        'heading: it is the string\'s id. A heading deleted leaves its string as it is.',
    ]
    for screen, title in SCREENS:
        if not by_screen[screen]:
            continue
        lines += ['', f'## {title}']
        for key, value in by_screen[screen]:
            lines += ['', f'### `{key}`', '']
            if value:
                lines.append(value)
    return '\n'.join(lines) + '\n'


def parse(md):
    """{key: text} from an exported (and edited) file."""
    out = {}
    key = None
    body = []

    def close():
        if key is not None:
            while body and not body[0].strip():
                body.pop(0)
            while body and not body[-1].strip():
                body.pop()
            out[key] = '\n'.join(body)

    for line in md.splitlines():
        if line.startswith('### `') and line.rstrip().endswith('`'):
            close()
            key = line.rstrip()[5:-1]
            body = []
        elif line.startswith('## ') or line.startswith('# '):
            close()
            key = None
            body = []
        elif key is not None:
            body.append(line)
    close()
    return out


def apply(texts, edits):
    """The content texts with `edits` written in; also the ids it didn't know."""
    known = set()
    out = {}
    for stem in FILES:
        text = texts[stem]
        pieces = []
        pos = 0
        for key, _, start, end, value in scan(stem, text):
            known.add(key)
            if key in edits and edits[key] != value:
                pieces.append(text[pos:start])
                pieces.append(json.dumps(edits[key], ensure_ascii=False))
                pos = end
        pieces.append(text[pos:])
        out[stem] = ''.join(pieces)
    return out, sorted(set(edits) - known)


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
        edits = parse(Path(argv[1]).read_text(encoding='utf-8'))
        new, unknown = apply(texts, edits)
        for k in unknown:
            print(f'unknown id, skipped: {k}')
        changed = [s for s in FILES if new[s] != texts[s]]
        for stem in changed:
            json.loads(new[stem])
            (CONTENT / f'{stem}.json').write_text(new[stem], encoding='utf-8', newline='\n')
        print(f'{len(edits)} strings read, {len(changed)} files changed: {", ".join(changed) or "none"}')
        return 0
    md = export(texts)
    new, unknown = apply(texts, parse(md))
    bad = [s for s in FILES if new[s] != texts[s]] + unknown
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
