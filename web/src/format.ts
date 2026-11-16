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
  if (Number.isInteger(r)) return r.toLocaleString('en-US');
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

/** A multiplier: "×1.05". */
export function mult(x: Fx | number): string {
  const n = typeof x === 'number' ? x : Number(x);
  return `×${trim(n.toFixed(3))}`;
}

export function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}
