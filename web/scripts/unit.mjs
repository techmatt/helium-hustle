// Unit checks for the client's pure helpers (Node runs the TypeScript directly).
//   node scripts/unit.mjs   (npm run unit)
import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { num, perDay, storyBody } from '../src/format.ts';
import { blockEnd, blockLen, canAddElse, canIndent, insertLine, moveLines, otherUnit, removeLine, sameEntry, setIndent, tidy } from '../src/ui/programEdit.ts';

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
// A story's `{rate}` reads as Stats shows boredom growth: 0.199 is +0.2, not a floored 0.19.
assert.equal(storyBody({ body: 'Current rate: {rate}.', rate: '0.199' }), 'Current rate: +0.2/day.');
assert.equal(storyBody({ body: 'No {rate} here.', rate: null }), 'No {rate} here.');

// Program line edits. A line is written 'if X' (an `if`), 'x' (a command) or '  x'
// (indented into a body); only `op` and `indented` matter to these edits.
const lines = (ls) =>
  ls.map((l) => {
    const t = l.trim();
    const e = t.startsWith('if ') ? { op: 'if', name: t } : t === 'else' ? { op: 'else' } : { op: 'command', command: t, count: 1 };
    return l.startsWith('  ') ? { ...e, indented: true } : e;
  });
const text = (es) => es.map((e) => (e.indented ? '  ' : '') + (e.op === 'if' ? e.name : e.op === 'else' ? 'else' : e.command));
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
// `else`: moves with its `if`, never alone; keeps its `if` through drags and un-indents.
const ifElse = ['a', 'if X', '  b', 'else', '  c', 'd'];
assert.equal(blockLen(lines(ifElse), 1), 4, 'an if takes its else and its body');
edit(moveLines(lines(ifElse), 1, 5), ['a', 'd', 'if X', '  b', 'else', '  c'], 'if moves with its else');
edit(moveLines(lines(ifElse), 3, 0), ifElse, 'an else never moves alone');
edit(moveLines(lines(ifElse), 5, 3), ['a', 'if X', '  b', '  d', 'else', '  c'], 'a line dropped up onto an else joins the if body');
edit(moveLines(lines(['if Y', 'if X', '  b', 'else', '  c']), 0, 2), ['if X', '  b', 'else', '  c', 'if Y'], 'an if lands past an else block');
edit(setIndent(lines(ifElse), 2, false), ['a', 'if X', 'else', '  c', 'b', 'd'], 'un-indent leaves the if and else blocks');
edit(setIndent(lines(ifElse), 5, true), ['a', 'if X', '  b', 'else', '  c', '  d'], 'indent under an else body');
edit(setIndent(lines(ifElse), 3, true), ifElse, 'an else is never indented');
edit(removeLine(lines(ifElse), 1), ['a', 'b', 'c', 'd'], 'a removed if takes its else');
edit(removeLine(lines(['if A', '  a', 'if X', 'else', '  c']), 2), ['if A', '  a', 'c'], 'a removed if with an empty body takes its else, not handing it to the if above');
edit(removeLine(lines(['if A', 'if X', 'else', '  c']), 1), ['if A', 'c'], 'and under a bare if');
edit(removeLine(lines(ifElse), 3), ['a', 'if X', '  b', 'c', 'd'], 'a removed else un-indents its body');
edit(tidy(lines(['a', 'else', '  c'])), ['a', 'c'], 'an else without its if goes');
assert.ok(canAddElse(lines(['if X', '  b'])) && canAddElse(lines(['if X'])), 'else after an if or its body');
assert.ok(!canAddElse(lines(['if X', 'else'])) && !canAddElse(lines(['a'])) && !canAddElse([]), 'no second else; none without an if');
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

// The instruction panel's insert, both Programs modes: below the cursor's line with its
// indent; into an if's or else's body; an if or else with an empty body line; an empty
// line filled in place; an empty program.
assert.deepEqual(insertLine([], 0, 'idle'), { lines: ['idle'], last: 0 });
assert.deepEqual(insertLine(['idle'], 0, 'if credits > 0'), { lines: ['idle', 'if credits > 0', '  '], last: 2 });
assert.deepEqual(insertLine(['if credits > 0', '  '], 1, 'idle'), { lines: ['if credits > 0', '  idle'], last: 1 });
assert.deepEqual(insertLine(['if credits > 0', '  idle'], 0, '4x idle'), { lines: ['if credits > 0', '  4x idle', '  idle'], last: 1 });
assert.deepEqual(insertLine(['else'], 0, 'idle'), { lines: ['else', '  idle'], last: 1 });
assert.deepEqual(insertLine(['  idle'], 0, 'else'), { lines: ['  idle', 'else', '  '], last: 2 });
edits += 6;

// Where the Blocks mode puts a new `if`: after the selected row's whole block.
{
  const c = (indented = false) => ({ op: 'command', command: 'idle', count: 1, indented });
  const iff = { op: 'if', when: { first: { reading: { kind: 'free_land' }, cmp: 'gt', value: '0' }, rest: null }, indented: false };
  const es = [c(), iff, c(true), c(true), { op: 'else', indented: false }, c(true), c()];
  assert.equal(blockEnd(es, 0), 0);
  for (const i of [1, 2, 3, 4, 5]) assert.equal(blockEnd(es, i), 5, `blockEnd(${i})`);
  assert.equal(blockEnd(es, 6), 6);
  assert.equal(blockEnd([], -1), -1);
}
edits += 8;

// Every quest `picture` in content names a file the client has (a dangling stem would show
// nothing, silently: art.tsx `questPicture`).
const pictures = JSON.parse(readFileSync(new URL('../../content/events.json', import.meta.url), 'utf8')).filter((e) => e.picture);
const pictureFiles = new Set(readdirSync(new URL('../src/assets/quests/', import.meta.url)).filter((f) => f.endsWith('.webp')).map((f) => f.slice(0, -5)));
for (const e of pictures) assert.ok(pictureFiles.has(e.picture), `${e.id}: picture ${e.picture} has no src/assets/quests/${e.picture}.webp`);
assert.ok(pictures.some((e) => e.id === 'qpower_boot_sequence'), "1.1 carries a picture");

console.log(`unit ok (${cases.length + 4 + edits + pictures.length} checks)`);
