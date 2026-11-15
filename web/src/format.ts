// Number display. Every number shown is an engine value; these only format it.

import type { Fx } from './engine';

const SUFFIXES: [number, string][] = [
  [1e12, 'T'],
  [1e9, 'B'],
  [1e6, 'M'],
  [1e3, 'K'],
];

/** A display number: K/M/B/T past 10,000; whole numbers plain; else up to `digits` decimals. */
export function num(x: Fx | number, digits = 1): string {
  const n = typeof x === 'number' ? x : Number(x);
  if (!Number.isFinite(n)) return '∞';
  // Fx::MAX: a price past the number range.
  if (Math.abs(n) >= 9e12) return '∞';
  const a = Math.abs(n);
  if (a >= 10_000) {
    for (const [v, s] of SUFFIXES) {
      if (a >= v) return `${trim((n / v).toFixed(a / v >= 100 ? 0 : a / v >= 10 ? 1 : 2))}${s}`;
    }
  }
  if (Number.isInteger(n)) return n.toLocaleString('en-US');
  return trim(n.toFixed(a >= 100 ? 0 : digits));
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

/** A signed amount: "+5", "−2.5". */
export function signed(x: Fx | number, digits = 1): string {
  const n = typeof x === 'number' ? x : Number(x);
  return `${n < 0 ? '−' : '+'}${num(Math.abs(n), digits)}`;
}

/** A multiplier: "×1.05". */
export function mult(x: Fx | number): string {
  const n = typeof x === 'number' ? x : Number(x);
  return `×${trim(n.toFixed(3))}`;
}

export function pct(x: number): string {
  return `${Math.round(x * 100)}%`;
}

export function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}
