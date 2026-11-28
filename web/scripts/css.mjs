// Stylesheet check: no two top-level rules in src/style.css each name the same lone
// selector and set the same property (124: a second global `.card-body`, ~600 lines on,
// silently restyled every building and project card). A selector repeated inside a list
// (`.a, .a:hover { }` then `.a:hover { }`) is a deliberate cascade and passes, as do
// rules inside @media and other at-rules.
// Prints each clash and exits 1 on any.
//   node scripts/css.mjs [file]   (npm run unit runs it on src/style.css)
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const raw = readFileSync(process.argv[2] ?? resolve(here, '../src/style.css'), 'utf8');
// Comments out, keeping line breaks so line numbers hold.
const css = raw.replace(/\/\*[\s\S]*?\*\//g, (c) => c.replace(/[^\n]/g, ' '));

const lone = new Map(); // selector -> [{ line, props }]
const stack = [];
let from = 0;
let bodyAt = 0;
for (let i = 0; i < css.length; i++) {
  const ch = css[i];
  if (ch === '{') {
    stack.push(css.slice(from, i).trim());
    if (stack.length === 1) bodyAt = i + 1;
    from = i + 1;
  } else if (ch === '}') {
    const sel = stack.pop();
    if (stack.length === 0 && !sel.startsWith('@') && !sel.includes(',')) {
      const props = new Set(
        css
          .slice(bodyAt, i)
          .split(';')
          .filter((d) => d.includes(':'))
          .map((d) => d.split(':')[0].trim()),
      );
      const key = sel.split(/\s+/).join(' ');
      const line = css.slice(0, bodyAt).split('\n').length;
      if (!lone.has(key)) lone.set(key, []);
      lone.get(key).push({ line, props });
    }
    from = i + 1;
  }
}

let fails = 0;
for (const [sel, rules] of lone) {
  for (let a = 0; a < rules.length; a++) {
    for (let b = a + 1; b < rules.length; b++) {
      const shared = [...rules[a].props].filter((p) => rules[b].props.has(p));
      if (shared.length) {
        fails++;
        console.log(`style.css: ${sel} at lines ${rules[a].line} and ${rules[b].line} both set ${shared.join(', ')}: merge them or scope one`);
      }
    }
  }
}
if (fails) process.exit(1);
console.log(`css: ${lone.size} lone top-level selectors, no clashes`);
