// Unit checks for the client's pure helpers (Node runs the TypeScript directly).
//   node scripts/unit.mjs   (npm run unit)
import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { boredomText, commandTip, dateText, delta, fillRate, num, perDay, slotLetter } from '../src/format.ts';
import { inPanelOrder, panelEntries } from '../src/ui/resourceOrder.ts';
import { navSlots } from '../src/ui/navSlots.ts';
import { offeredSpeed, speedsOffered } from '../src/speeds.ts';
import { clearTold, newTold, tellPages } from '../src/story.ts';
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
// Whole numbers read as toLocaleString('en-US') does (num keeps one formatter, 094).
for (const n of [-0, -9999, -150, 0, 7, 100, 1000, 9999, 3.25e3]) assert.equal(num(n), n.toLocaleString('en-US'), `num(${n})`);
assert.equal(perDay(-2.5), '−2.5/day');
assert.equal(perDay(0.001), '0/day');
assert.equal(perDay(1500.4), '+1,500/day');
// An event effect's `{rate}` reads as Stats shows boredom growth: 0.199 is +0.2, not a floored 0.19.
assert.equal(fillRate('Current rate: {rate}.', '0.199'), 'Current rate: +0.2/day.');
assert.equal(fillRate('No {rate} here.', null), 'No {rate} here.');
// A command's tooltip: costs, produces, boredom, one per line; each run, then the repeat count's totals.
{
  const names = { energy: 'Energy', credits: 'Credits', boredom: 'Boredom', helium: 'Helium', regolith: 'Regolith' };
  const v = { names, resources: [{ id: 'regolith' }, { id: 'helium' }, { id: 'boredom' }, { id: 'credits' }, { id: 'energy' }] };
  const c = { costs: { energy: '2' }, produces: { credits: '5' }, boredom: '0.1' };
  assert.equal(commandTip(c, v), 'Each run:\n−2 Energy\n+5 Credits\n+0.1 Boredom');
  assert.equal(commandTip(c, v, 3), 'Each run:\n−2 Energy\n+5 Credits\n+0.1 Boredom\n\n3 runs:\n−6 Energy\n+15 Credits\n+0.3 Boredom');
  assert.equal(commandTip({ costs: {}, produces: {}, boredom: '0' }, v, 4), '');
  // A reducer (Dream) states what it removes; a command with no boredom says nothing of it.
  const dream = { costs: { energy: '1' }, produces: {}, boredom: '-2.2' };
  assert.equal(commandTip(dream, v, 2), 'Each run:\n−1 Energy\n−2.2 Boredom\n\n2 runs:\n−2 Energy\n−4.4 Boredom');
  assert.equal(commandTip({ costs: { energy: '1' }, produces: {}, boredom: '0' }, v), 'Each run:\n−1 Energy');
  // Each list in the Resources panel's order: its first group, then the engine's.
  const mixed = { costs: { helium: '1', regolith: '1', credits: '1' }, produces: {}, boredom: '0' };
  assert.equal(commandTip(mixed, v), 'Each run:\n−1 Credits\n−1 Regolith\n−1 Helium');
}
// The Resources panel's order: energy, credits, land, boredom, processors, then the engine's
// order; an id not a resource last, as given.
{
  const res = [{ id: 'helium' }, { id: 'land' }, { id: 'energy' }, { id: 'regolith' }];
  assert.deepEqual(
    inPanelOrder(res, ['zz', 'regolith', 'land', 'helium', 'credits', 'energy'], (x) => x),
    ['energy', 'credits', 'land', 'helium', 'regolith', 'zz'],
  );
  assert.deepEqual(panelEntries(res, { regolith: 1, land: 2 }), [['land', 2], ['regolith', 1]]);
}
// Dates: stored as days from 0, shown as 365-day years.
assert.equal(dateText(0, 365), 'Year 1, day 1');
assert.equal(dateText(364, 365), 'Year 1, day 365');
assert.equal(dateText(365, 365), 'Year 2, day 1');
assert.equal(dateText(3 * 365 + 111, 365), 'Year 4, day 112');
assert.equal(boredomText('0'), '');
assert.equal(boredomText('-10'), '−10');
// Program slots are lettered: A for the first.
assert.deepEqual([0, 1, 4].map(slotLetter), ['A', 'B', 'E']);
// The retirement summary's deltas round to the row's precision first: boredom growth moving
// by 0.001 (×0.98 to ×0.979) shows no delta; credits round to whole numbers.
assert.equal(delta('0.979', '0.98', 2), null);
assert.equal(delta('0.95', '0.98', 2), -0.03);
assert.equal(delta('1.07', '1.05', 2), 0.02);
assert.equal(delta('150.4', '150', 0), null);
assert.equal(delta('175', '150', 0), 25);
assert.equal(delta('175', undefined, 0), null);

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
// Every Act 1 quest (1.1-1.6: the chain before Act 2's opener, numbered by the engine; the
// side quests 1.a, 1.b and 1.d: 1.c, Blueprints for Everyone, went in 129) carries one.
const events = JSON.parse(readFileSync(new URL('../../content/events.json', import.meta.url), 'utf8'));
const act2 = JSON.parse(readFileSync(new URL('../../content/acts.json', import.meta.url), 'utf8')).find((a) => a.act === 2).opens_with;
const chain = events.filter((e) => e.category === 'story');
const act1 = [...chain.slice(0, chain.findIndex((e) => e.id === act2)), ...events.filter((e) => /^1\./.test(e.beat ?? ''))];
assert.equal(act1.length, 9, 'Act 1 has 9 quests');
for (const e of act1) assert.ok(e.picture, `${e.beat ?? e.title} ${e.id} carries a picture`);

// The nav grid's tiles (Matt, 2026-11-26): the open ones packed in content order, no blank
// cells; the utility tiles are nav.json's last rows, so they follow the last open play tile.
{
  const t = (id, visible) => ({ id, visible });
  const ids = (tiles) => navSlots(tiles).map((c) => c.id);
  assert.deepEqual(ids([t('a', true), t('b', false), t('c', true), t('d', false)]), ['a', 'c']);
  assert.deepEqual(ids([t('a', false)]), []);
  const nav = JSON.parse(readFileSync(new URL('../../content/nav.json', import.meta.url), 'utf8'));
  // Read row by row in two columns (Matt, 124): Buildings, Research / Commands, Programs /
  // Launch Pads, Projects / ... / Achievements, Story / Stats, Options.
  const head = ['buildings', 'research', 'commands', 'programs', 'launch_pads', 'projects'];
  const utility = ['achievements', 'story', 'stats', 'options'];
  assert.deepEqual(nav.slice(0, 6).map((n) => n.id), head, 'the play tiles lead in their order');
  assert.deepEqual(nav.slice(-4).map((n) => n.id), utility, 'the utility tiles are the last rows');
  const early = nav.map((n) => t(n.id, n.visible.type === 'always'));
  assert.deepEqual(ids(early), ['buildings', 'commands', ...utility], 'a new game packs its open tiles');
}
// 50x and 200x wait for Accelerated Scheduling (amendment #3261); a faster speed drops to 10x.
assert.deepEqual(speedsOffered(false), [0, 1, 3, 10]);
assert.deepEqual(speedsOffered(true), [0, 1, 3, 10, 50, 200]);
assert.deepEqual([offeredSpeed(200, false), offeredSpeed(50, false), offeredSpeed(3, false), offeredSpeed(200, true)], [10, 10, 3, 200]);
edits += 4;

// The story modal's popups (story.ts `tellPages`): each told once per run, a seen event hidden
// with "Hide popups" on, nothing marked seen under the autopilot or at a load.
{
  const ev = (id) => ({ id, title: id, summary: `${id} line`, goal: '', reward: '', effect: 'Rate {rate}.', body: '', rate: '0.199' });
  const view = (run, events, quest = null, done = []) => ({ run, day: 5, story: [], event_texts: events, project_done: done, quest, completed_quests: ['q0'] });
  const opts = (o = {}) => ({ pilot: false, hideSeen: true, fill: fillRate, ...o });
  const t = newTold();
  const first = tellPages(view(1, [ev('burnout')]), 'play', t, opts());
  assert.equal(first.length, 1);
  assert.equal(first[0].text, 'burnout line');
  assert.equal(first[0].effect, 'Rate +0.2/day.');
  assert.equal(tellPages(view(1, [ev('burnout')]), 'play', t, opts()).length, 0, 'told once a run');
  assert.equal(tellPages(view(2, [ev('burnout')]), 'play', t, opts()).length, 0, 'seen: hidden with Hide popups on');
  assert.equal(tellPages(view(3, [ev('burnout')]), 'play', t, opts({ hideSeen: false })).length, 1, 'seen: shown with it off');
  const quiet = newTold();
  assert.equal(tellPages(view(1, [ev('a')]), 'play', quiet, opts({ pilot: true })).length, 0);
  assert.equal(tellPages(view(1, [ev('b')]), 'load', quiet, opts()).length, 0);
  assert.deepEqual([...quiet.seenPopups], [], 'the autopilot and a load mark nothing seen');
  const q = { id: 'q1', title: 'Q1', summary: '', goal: 'Do it.', reward: '', effect: '', body: 'Begin.', picture: null };
  const opened = tellPages(view(1, [], q), 'play', newTold(), opts());
  assert.deepEqual(opened.map((p) => [p.text, p.task]), [['Begin.', 'Do it.']], 'a new quest opens with its task last');
  // A project completion: its own popup each completion, never at a load or under the pilot.
  const pd = (completion) => ({ id: 'p1', name: 'P1', day: 4, completion, costs: { credits: '100' }, reward_text: ['More.'] });
  const pt = newTold();
  assert.equal(tellPages(view(1, [], null, [pd(1)]), 'load', pt, opts()).length, 0, 'a load tells no project');
  const done2 = tellPages(view(1, [], null, [pd(1), pd(2)]), 'play', pt, opts());
  assert.deepEqual(done2.map((p) => [p.title, p.project?.rewards, p.project?.costs.credits]), [['P1', ['More.'], '100']], 'the second completion pops up alone');
  assert.equal(tellPages(view(1, [], null, [pd(1), pd(2)]), 'play', pt, opts()).length, 0, 'told once');
  assert.equal(tellPages(view(2, [], null, [pd(1)]), 'play', pt, opts({ pilot: true })).length, 0, 'nothing under the pilot');
  assert.equal(tellPages(view(3, [], null, [pd(1)]), 'play', pt, opts()).length, 1, 'a run project again next run');
  clearTold(t);
  assert.equal(t.seenPopups.size + t.stories.size, 0);
  edits += 16;
}

console.log(`unit ok (${cases.length + 4 + edits + pictures.length + act1.length} checks)`);
