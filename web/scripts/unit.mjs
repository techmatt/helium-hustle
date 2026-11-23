// Unit checks for the client's pure helpers (Node runs the TypeScript directly).
//   node scripts/unit.mjs   (npm run unit)
import assert from 'node:assert/strict';
import { num, perDay } from '../src/format.ts';
import { blockLen, canIndent, moveLines, otherUnit, removeLine, sameEntry, setIndent, tidy } from '../src/ui/programEdit.ts';

const cases = [
  [0, '0'],
  [5, '5'],
  [2.5, '2.5'],
  [99.94, '99.9'],
  [1234, '1,234'],
  // Fractional amounts from 100 up round to whole numbers, with separators.
  [1234.5, '1,235'],
  [9999.4, '9,999'],
  // ...and one that rounds to 10,000 takes the suffix.
  [9999.6, '10K'],
  [10_000, '10K'],
  [15_250, '15.3K'],
  [1_234_567, '1.23M'],
  [123_456, '123K'],
  // The suffix comes from the rounded mantissa: never "1000K".
  [999_600, '1M'],
  [999_400, '999K'],
  [999_600_000, '1B'],
  [2_500_000_000, '2.5B'],
  [-999_600, '-1M'],
  [9e12, '∞'],
];
for (const [n, want] of cases) assert.equal(num(n), want, `num(${n})`);
assert.equal(perDay(-2.5), '−2.5/day');
assert.equal(perDay(0.001), '0/day');
assert.equal(perDay(1500.4), '+1,500/day');

// Program line edits. A line is written 'if X' (an `if`), 'x' (a command) or '  x'
// (indented into a body); only `op` and `indented` matter to these edits.
const lines = (ls) =>
  ls.map((l) => {
    const t = l.trim();
    const e = t.startsWith('if ') ? { op: 'if', name: t } : { op: 'command', command: t, count: 1 };
    return l.startsWith('  ') ? { ...e, indented: true } : e;
  });
const text = (es) => es.map((e) => (e.indented ? '  ' : '') + (e.op === 'if' ? e.name : e.command));
let edits = 0;
const edit = (got, want, what) => {
  assert.deepEqual(text(got), want, what);
  edits++;
};
// An `if` moves with its body.
edit(moveLines(lines(['a', 'if X', '  b', '  c', 'd']), 1, 4), ['a', 'd', 'if X', '  b', '  c'], 'if moves with body');
// A removed `if` un-indents its body, which can't fall into the block above.
edit(removeLine(lines(['if X', '  b', '  c', 'd']), 0), ['b', 'c', 'd'], 'removed if un-indents body');
edit(removeLine(lines(['if Y', '  a', 'if X', '  b']), 2), ['if Y', '  a', 'b'], 'removed if body stays out of the block above');
// A single line dropped into a body joins it, moving up or down.
edit(moveLines(lines(['if X', '  b', 'c']), 2, 1), ['if X', '  c', '  b'], 'line dropped up into a body');
edit(moveLines(lines(['a', 'if X', '  b']), 0, 2), ['if X', '  b', '  a'], 'line dropped down into a body');
// An `if` dropped inside a body lands after it.
edit(moveLines(lines(['if X', '  b', '  c', 'if Y']), 3, 1), ['if X', '  b', '  c', 'if Y'], 'if lands after a body');
// setIndent: refused under a non-`if`, allowed under an `if` or a body line, never for an `if`.
edit(setIndent(lines(['a', 'b']), 1, true), ['a', 'b'], 'no indent under a command');
edit(setIndent(lines(['if X', 'b']), 1, true), ['if X', '  b'], 'indent under an if');
edit(setIndent(lines(['if X', '  b', 'c']), 2, true), ['if X', '  b', '  c'], 'indent under a body line');
edit(setIndent(lines(['if X', 'if Y']), 1, true), ['if X', 'if Y'], 'an if is never indented');
edit(setIndent(lines(['if X', '  b', '  c', 'd']), 1, false), ['if X', '  c', 'b', 'd'], 'un-indent moves out below the block');
assert.equal(canIndent(lines(['a', 'b']), 1), false);
assert.equal(canIndent(lines(['a']), 0), false);
// tidy un-indents a line that lost its `if`.
edit(tidy(lines(['a', '  b', 'if X', '  c'])), ['a', 'b', 'if X', '  c'], 'tidy');
assert.equal(blockLen(lines(['if X', '  b', '  c', 'd']), 0), 3);
assert.equal(blockLen(lines(['a', '  b']), 0), 1);
edits += 4;

// The unit toggle: a resource's percent and amount are one entry of the reading list.
const pct = { kind: 'percent', resource: 'propellant' };
const amt = { kind: 'amount', resource: 'propellant' };
assert.deepEqual(otherUnit(pct), amt);
assert.deepEqual(otherUnit(amt), pct);
assert.deepEqual(otherUnit({ kind: 'free_land' }), { kind: 'free_land' });
assert.ok(sameEntry(pct, amt));
assert.ok(!sameEntry(pct, { kind: 'percent', resource: 'ice' }));
assert.ok(sameEntry({ kind: 'energy_per_day' }, { kind: 'energy_per_day' }));
assert.ok(!sameEntry({ kind: 'energy_per_day' }, { kind: 'days_since_purchase' }));
edits += 7;

console.log(`unit ok (${cases.length + 3 + edits} checks)`);
