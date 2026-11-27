// Number display. Every number shown is an engine value; these only format it.

import type { CommandView, Fx } from './engine';
// With its extension: Node runs this file directly in the unit checks.
import { panelEntries } from './ui/resourceOrder.ts';

const SUFFIXES: [number, string][] = [
  [1e12, 'T'],
  [1e9, 'B'],
  [1e6, 'M'],
  [1e3, 'K'],
];

/** `toLocaleString('en-US')`'s format, made once: that call builds a formatter each
 * time, and was the page's costliest function at 200x (094). */
const WHOLE = new Intl.NumberFormat('en-US');

/** A display number: K/M/B/T past 10,000; whole numbers plain; else up to `digits` decimals. */
export function num(x: Fx | number, digits = 1): string {
  const n = typeof x === 'number' ? x : Number(x);
  if (!Number.isFinite(n)) return '∞';
  // Fx::MAX: a price past the number range.
  if (Math.abs(n) >= 9e12) return '∞';
  const a = Math.abs(n);
  // From 100 up, whole numbers: rounded first, so 9,999.6 is 10K and 1,234.5 is 1,235.
  const r = a >= 100 ? Math.round(n) : n;
  if (Math.abs(r) >= 10_000) {
    // The suffix from the rounded mantissa: 999,600 is 1M, not 1000K.
    for (let i = SUFFIXES.length - 1; i >= 0; i--) {
      const [v, s] = SUFFIXES[i];
      const m = a / v;
      const shown = Number(m.toFixed(m >= 100 ? 0 : m >= 10 ? 1 : 2));
      if (shown < 1000 || i === 0) return `${n < 0 ? '-' : ''}${trim(String(shown))}${s}`;
    }
  }
  if (Number.isInteger(r)) return WHOLE.format(r);
  return trim(n.toFixed(digits));
}

function trim(s: string): string {
  return s.includes('.') ? s.replace(/\.?0+$/, '') : s;
}

/** A signed rate: "+5/day", "−2.5/day". */
export function perDay(x: Fx | number, digits = 1): string {
  const n = typeof x === 'number' ? x : Number(x);
  const shown = num(Math.abs(n), digits);
  if (shown === '0') return '0/day';
  return `${n < 0 ? '−' : '+'}${shown}/day`;
}

/** An event's mechanics text (`effect`) with its `{rate}` filled as the Stats panel shows
 * boredom growth. */
export function fillRate(text: string, rate?: Fx | null): string {
  return rate == null ? text : text.replaceAll('{rate}', perDay(rate, 2));
}

/** A multiplier: "×1.05". */
export function mult(x: Fx | number): string {
  const n = typeof x === 'number' ? x : Number(x);
  return `×${trim(n.toFixed(3))}`;
}

/** A value's change from an earlier one, rounded to `places` (the row's display precision:
 * credits 0, multipliers 2); null when unknown or when it rounds to zero, so a change too
 * small to read is neither shown nor colored. */
export function delta(now: Fx | number, was: Fx | number | undefined, places: number): number | null {
  if (was === undefined) return null;
  const r = Number((Number(now) - Number(was)).toFixed(places));
  return r === 0 ? null : r;
}

/** What a command tooltip reads from the view: names, and the resources' order. */
export type TipView = { names: Record<string, string>; resources: readonly { id: string }[] };

/** A command's "−2 Energy" lines: costs, then produces, each in the Resources panel's order,
 * then boredom (the Commands card's order). */
function flowLines(c: CommandView, v: TipView, times: number): string[] {
  const nm = (id: string) => v.names[id] ?? id;
  const out: string[] = [];
  for (const [res, amt] of panelEntries(v.resources, c.costs)) out.push(`−${num(Number(amt) * times)} ${nm(res)}`);
  for (const [res, amt] of panelEntries(v.resources, c.produces)) out.push(`+${num(Number(amt) * times)} ${nm(res)}`);
  const b = boredomText(Number(c.boredom) * times);
  if (b) out.push(`${b} ${nm('boredom')}`);
  return out;
}

/** A command's signed boredom figure ("+0.1", "−2"); empty for none. The view's value
 * (already × its axis), only formatted. */
export function boredomText(x: Fx | number): string {
  const n = Number(x);
  const shown = num(Math.abs(n), 2);
  return n === 0 || shown === '0' ? '' : `${n < 0 ? '−' : '+'}${shown}`;
}

/** A calendar date from a day count (stored as days, 0 the first): "Year 4, day 112". */
export function dateText(day: number, daysPerYear: number): string {
  const d = Math.max(0, Math.floor(day));
  return `Year ${Math.floor(d / daysPerYear) + 1}, day ${(d % daysPerYear) + 1}`;
}

/** A command's expenses and incomes as tooltip text, one per line ("−2 Energy", "+5 Credits",
 * "+0.1 Boredom", or for a reducer such as Dream "−2 Boredom" at its axis's rank now; nothing
 * about boredom for a command with no effect on it): "Each run:", then for a repeat count over
 * 1 "N runs:" and the totals. The one builder for every hover that names a command. The
 * figures are the view's; the total only multiplies them for display. Empty when the command
 * costs and makes nothing. */
export function commandTip(c: CommandView, v: TipView, count = 1): string {
  const each = flowLines(c, v, 1);
  if (each.length === 0) return '';
  const parts = [`Each run:\n${each.join('\n')}`];
  if (count > 1) parts.push(`${count} runs:\n${flowLines(c, v, count).join('\n')}`);
  return parts.join('\n\n');
}

/** A program slot's name in the UI: its letter, A for the first (index 0). Past Z (a config
 * with more tabs than letters), its number. */
export function slotLetter(i: number): string {
  return i >= 0 && i < 26 ? String.fromCharCode(65 + i) : String(i + 1);
}

export function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}
