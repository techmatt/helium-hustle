// Edits to a program's line list that keep its shape one the engine accepts (an indented
// line sits under an `if` or another indented line; an `if` is never indented). The engine
// still checks every `set_program`: these only build the list a drag or a button meant.
// Pure (type imports only), so scripts/unit.mjs runs them under Node; the store call that
// sends an edit is in programStore.ts.

import type { Entry, Reading } from '../engine';

/** Lines `i` moves with: an `if` takes its body. */
export function blockLen(es: Entry[], i: number): number {
  if (es[i]?.op !== 'if') return 1;
  let n = 1;
  while (es[i + n]?.indented) n++;
  return n;
}

/** Whether line `e` may sit indented under `above`: it isn't an `if`, and `above` is an
 * `if` or in a body. */
function fitsUnder(e: Entry | undefined, above: Entry | undefined): boolean {
  return e?.op !== 'if' && !!above && (above.op === 'if' || !!above.indented);
}

/** Whether a line at `i` may be indented: the line above is an `if` or in a body. */
export function canIndent(es: Entry[], i: number): boolean {
  return fitsUnder(es[i], es[i - 1]);
}

/** Un-indents whatever lost its `if`. */
export function tidy(es: Entry[]): Entry[] {
  const out: Entry[] = [];
  for (const e of es) {
    out.push(e.indented && !fitsUnder(e, out[out.length - 1]) ? { ...e, indented: false } : e);
  }
  return out;
}

/** Without line `i`: an `if`'s body is un-indented first, so it can't fall into the
 * body of a block above. */
export function removeLine(es: Entry[], i: number): Entry[] {
  const end = i + blockLen(es, i);
  return tidy(es.map((e, j) => (j > i && j < end ? { ...e, indented: false } : e)).filter((_, j) => j !== i));
}

/** Line `from` (with its body, if an `if`) dropped on line `to`: it lands after `to`
 * moving down, before it moving up. A single line joins the body it lands in; an `if`
 * dropped inside a body lands after that body. */
export function moveLines(es: Entry[], from: number, to: number): Entry[] {
  const len = blockLen(es, from);
  if (to >= from && to < from + len) return es;
  const moved = es.slice(from, from + len);
  const rest = [...es.slice(0, from), ...es.slice(from + len)];
  let at = to > from ? to - len + 1 : to;
  if (moved[0].op === 'if') {
    while (rest[at]?.indented) at++;
  } else {
    const above = rest[at - 1];
    const inBody = !!above && (above.op === 'if' || !!above.indented) && (to > from || !!rest[at]?.indented);
    moved[0] = { ...moved[0], indented: inBody };
  }
  rest.splice(at, 0, ...moved);
  return tidy(rest);
}

/** Indents line `i` into the body above it, or moves it out below its block. */
export function setIndent(es: Entry[], i: number, indented: boolean): Entry[] {
  if (indented) return canIndent(es, i) ? es.map((e, j) => (j === i ? { ...e, indented: true } : e)) : es;
  let end = i + 1;
  while (es[end]?.indented) end++;
  const next = [...es];
  const [e] = next.splice(i, 1);
  next.splice(end - 1, 0, { ...e, indented: false });
  return next;
}

export function sameReading(a: Reading, b: Reading): boolean {
  return JSON.stringify(Object.entries(a).sort()) === JSON.stringify(Object.entries(b).sort());
}

/** Whether two readings are one entry of the reading list: the same reading, or a
 * resource's percent and amount (the unit toggle switches between them). */
export function sameEntry(a: Reading, b: Reading): boolean {
  const res = (r: Reading) => (r.kind === 'amount' || r.kind === 'percent' ? r.resource : null);
  const ra = res(a);
  return ra !== null ? ra === res(b) : sameReading(a, b);
}

/** A capped reading in the other unit: a percent as an amount, an amount as a percent. */
export function otherUnit(r: Reading): Reading {
  if (r.kind === 'percent') return { kind: 'amount', resource: r.resource };
  if (r.kind === 'amount') return { kind: 'percent', resource: r.resource };
  return r;
}
