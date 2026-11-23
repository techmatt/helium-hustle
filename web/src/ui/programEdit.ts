// Edits to a program's line list that keep its shape one the engine accepts (an indented
// line sits under an `if`, an `else` or another indented line; an `else` follows an `if` or
// its body, one per `if`; neither is ever indented). The engine still checks every
// `set_program`: these only build the list a drag or a button meant.
// Pure (type imports only), so scripts/unit.mjs runs them under Node; the store call that
// sends an edit is in programStore.ts.

import type { Entry, Reading } from '../engine';

/** The block a line may join: the one the line above leaves open (hh-core `programs::Block`). */
export type Block = 'none' | 'if' | 'else';

/** The block open below line `e`, when `block` is open above it. */
function after(block: Block, e: Entry): Block {
  if (e.op === 'if') return 'if';
  if (e.op === 'else') return 'else';
  return e.indented ? block : 'none';
}

/** The block open above line `i`. */
export function blockAt(es: Entry[], i: number): Block {
  let block: Block = 'none';
  for (let j = 0; j < i && j < es.length; j++) block = after(block, es[j]);
  return block;
}

/** Whether an `else` may be added at the end: the last line is an `if` or in its body. */
export function canAddElse(es: Entry[]): boolean {
  return blockAt(es, es.length) === 'if';
}

/** Lines `i` moves with: an `if` takes its body and its `else` with that body; an `else`
 * its body. */
export function blockLen(es: Entry[], i: number): number {
  const op = es[i]?.op;
  if (op !== 'if' && op !== 'else') return 1;
  let n = 1;
  while (es[i + n]?.indented) n++;
  if (op === 'if' && es[i + n]?.op === 'else') {
    n++;
    while (es[i + n]?.indented) n++;
  }
  return n;
}

/** Whether line `e` may sit indented under a block `block`: it is neither an `if` nor an
 * `else`, and a block is open. */
function fitsIn(e: Entry | undefined, block: Block): boolean {
  return e?.op !== 'if' && e?.op !== 'else' && block !== 'none';
}

/** Whether a line at `i` may be indented: the line above is an `if`, an `else` or in a body. */
export function canIndent(es: Entry[], i: number): boolean {
  return fitsIn(es[i], blockAt(es, i));
}

/** Un-indents whatever lost its block; an `else` that lost its `if` goes (its body is
 * un-indented with it). */
export function tidy(es: Entry[]): Entry[] {
  const out: Entry[] = [];
  let block: Block = 'none';
  for (const e of es) {
    if (e.op === 'else' && block !== 'if') {
      block = 'none';
      continue;
    }
    const kept = e.indented && !fitsIn(e, block) ? { ...e, indented: false } : e;
    out.push(kept);
    block = after(block, kept);
  }
  return out;
}

/** Without line `i`: an `if`'s or `else`'s body is un-indented first, so it can't fall into
 * the body of a block above (an `if` removed takes its `else` line, not that body). */
export function removeLine(es: Entry[], i: number): Entry[] {
  const end = i + blockLen(es, i);
  // An `if`'s `else` goes with it, here: with an empty body above it, `tidy` could hand
  // it to a block still open above.
  const own = es[i]?.op === 'if' ? es.findIndex((e, j) => j > i && j < end && e.op === 'else') : -1;
  return tidy(es.map((e, j) => (j > i && j < end ? { ...e, indented: false } : e)).filter((_, j) => j !== i && j !== own));
}

/** Line `from` (with its block, if an `if`) dropped on line `to`: it lands after `to`
 * moving down, before it moving up. A single line joins the body it lands in; an `if`
 * dropped inside a block lands after that block (its `else` too). An `else` moves only
 * with its `if`. */
export function moveLines(es: Entry[], from: number, to: number): Entry[] {
  if (es[from]?.op === 'else') return es;
  const len = blockLen(es, from);
  if (to >= from && to < from + len) return es;
  const moved = es.slice(from, from + len);
  const rest = [...es.slice(0, from), ...es.slice(from + len)];
  let at = to > from ? to - len + 1 : to;
  if (moved[0].op === 'if') {
    while (rest[at]?.indented || rest[at]?.op === 'else') at++;
  } else {
    // Landing just above an `else` keeps it in the `if`'s body, so the `else` keeps its `if`.
    const inBody = blockAt(rest, at) !== 'none' && (to > from || !!rest[at]?.indented || rest[at]?.op === 'else');
    moved[0] = { ...moved[0], indented: inBody };
  }
  rest.splice(at, 0, ...moved);
  return tidy(rest);
}

/** Indents line `i` into the block above it, or moves it out below its block (an `if`
 * body's line below the `else`'s body too). */
export function setIndent(es: Entry[], i: number, indented: boolean): Entry[] {
  if (indented) return canIndent(es, i) ? es.map((e, j) => (j === i ? { ...e, indented: true } : e)) : es;
  let end = i + 1;
  while (es[end]?.indented) end++;
  if (es[end]?.op === 'else') {
    end++;
    while (es[end]?.indented) end++;
  }
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
