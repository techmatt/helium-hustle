// Unit checks for the client's pure helpers (Node runs the TypeScript directly).
//   node scripts/unit.mjs   (npm run unit)
import assert from 'node:assert/strict';
import { num, perDay } from '../src/format.ts';

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
console.log(`unit ok (${cases.length + 3} checks)`);
