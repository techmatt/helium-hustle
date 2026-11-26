// Contrast check: the text-on-background pairs in the hand-kept PAIRS list below, in both
// themes, read from the token blocks in src/style.css. It checks only those pairs: a new
// text or background token needs its pairs added there. WCAG 2 ratio; fails under 4.5:1
// (or a pair's own higher minimum).
// Prints one line per pair below 7:1 and exits 1 on any failure.
//   node scripts/contrast.mjs   (npm run unit runs it after unit.mjs)
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const css = readFileSync(resolve(here, '../src/style.css'), 'utf8');

function tokens(selector) {
  const at = css.indexOf(selector);
  const body = css.slice(css.indexOf('{', at) + 1, css.indexOf('}', at));
  return Object.fromEntries([...body.matchAll(/--([\w-]+):\s*(#[0-9a-f]{6})/gi)].map((m) => [m[1], m[2]]));
}

// [text, backgrounds, minimum (default 4.5)]: a #hex is a fixed color, anything else a
// token.
const PAIRS = [
  // The Resources panel's green rates (small mono figures) and the status bar's spare
  // Energy, both on the panel: held to 7:1.
  ['pos-text', ['panel'], 7],
  // A Ready pad's status on its tinted head.
  ['pos-text', ['ok-head']],
  ['text', ['page', 'panel', 'card', 'banner', 'strip', 'strip-hover', 'hover', 'disabled', 'skip-tint', 'ok-tint', 'ok-head', 'warn-head', 'bad-tint', 'sky', 'track', 'fill-green', 'fill-red', 'fill-gray', 'fill-blue']],
  ['muted', ['page', 'panel', 'card', 'banner', 'strip', 'hover', 'disabled', 'skip-tint', 'ok-tint', 'ok-head']],
  ['accent-text', ['panel', 'card', 'banner', 'hover', 'warn-head']],
  ['pos', ['page', 'panel', 'card', 'ok-tint', 'skip-tint', 'hover']],
  ['neg', ['page', 'panel', 'card', 'bad-tint', 'hover', 'ok-tint', 'disabled']],
  ['info', ['panel', 'card', 'banner']],
  // The mechanics lines' text and their muted labels (quest goals, rewards, event effects).
  ['text', ['mech-tint']],
  ['muted', ['mech-tint']],
  ['sel-text', ['sel-bg', 'sel-hover']],
  // An earned achievement's star on its medal.
  ['panel', ['accent-text']],
  ['on-neg', ['neg']],
  ['axis-nationalist', ['panel', 'card']],
  ['axis-humanist', ['panel', 'card']],
  ['axis-rationalist', ['panel', 'card']],
  ['#ffffff', ['#c62828', '#2e7d32', '#1565c0']],
];

const lum = (hex) => {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  const l = c.map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return 0.2126 * l[0] + 0.7152 * l[1] + 0.0722 * l[2];
};
const ratio = (a, b) => {
  const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p);
  return (x + 0.05) / (y + 0.05);
};

let fails = 0;
for (const [theme, sel] of [['light', ":root[data-theme='light']"], ['dark', ":root[data-theme='dark']"]]) {
  const t = tokens(sel);
  const get = (k) => (k.startsWith('#') ? k : t[k]);
  for (const [fg, bgs, min = 4.5] of PAIRS)
    for (const bg of bgs) {
      if (!get(fg) || !get(bg)) throw new Error(`${theme}: no token ${get(fg) ? bg : fg}`);
      const r = ratio(get(fg), get(bg));
      if (r < min) fails++;
      if (r < 7) console.log(`${r < min ? 'FAIL' : 'ok  '} ${theme.padEnd(5)} ${fg} on ${bg}: ${r.toFixed(2)}`);
    }
}
console.log(fails ? `${fails} pairs under their minimum` : 'every pair at its minimum (4.5:1 unless set) or better');
process.exit(fails ? 1 : 0);
