// 050's check and screenshots of the Cortex mock (web/src/ui/CortexMock.tsx): through the
// debug menu, grant Insight, buy Serenity down to its end, see the Axiom arrive, finish
// Logistics, learn their pair Theorem, then respec. Light and dark PNGs of the view.
// Drives headless Edge or Chrome over web/scripts/cdp.mjs against a served static build.
//   HH_URL  the page, required (a `vite preview`, web/README.md)
//   OUT     where the PNGs go (default: this folder)
//   COPY    a second folder to copy them to (optional)
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, q } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('shots', 'node experiments/results/050_cortex_ui_mock/shots.mjs');
const OUT = resolve(process.env.OUT ?? here);
mkdirSync(OUT, { recursive: true });
const b = await launch({ width: 1440, height: 1000, emulate: true, profileRoot: resolve(here, '../../../out/050_cortex_ui_mock/profile') });
const { evaluate } = b;
const page = driver(b, { url });
const { waitFor, click, load, debugMenu } = page;
const text = (sel) => evaluate(`document.querySelector(${q(sel)})?.textContent ?? null`);
const attr = (sel, a) => evaluate(`document.querySelector(${q(sel)})?.getAttribute(${q(a)}) ?? null`);
const check = (ok, what) => {
  if (!ok) throw new Error(`FAILED: ${what}`);
  console.log(`ok: ${what}`);
};
const actions = async () => /(\d+) actions/.exec(await text('#debug-info'))?.[1];
const buyDown = async (branch) => {
  for (let i = 1; i <= 6; i++) {
    const sel = `[data-node="${branch}-${i}"]`;
    while (!(await evaluate(`(() => { const n = document.querySelector(${q(sel)}); return n.classList.contains('cx-have'); })()`))) {
      check(!(await evaluate(`document.querySelector(${q(sel)}).disabled`)), `${branch}-${i} buyable`);
      await click(sel, sel);
    }
  }
};
const shot = async (name) => {
  const r = await evaluate(`(() => { const r = document.querySelector('[data-testid="cortex-mock"]').getBoundingClientRect();
    return { x: r.x, y: r.y, width: r.width, height: r.height }; })()`);
  const file = resolve(OUT, `${name}.png`);
  await capture(b, file, { clip: { x: r.x - 4, y: r.y - 4, width: r.width + 8, height: r.height + 8, scale: 1 }, captureBeyondViewport: true });
  if (process.env.COPY) copyFileSync(file, resolve(process.env.COPY, `050_${name}.png`));
  console.log(`shot ${name}`);
};

try {
  await load();
  await debugMenu();
  const before = await actions();
  await click('#cortex-open', 'open the mock');
  await waitFor(`!!document.querySelector('[data-testid="cortex-mock"]')`, 'the mock');
  check((await text('#cortex-tp')) === '0' && (await text('#cortex-axioms')) === '0', 'starts with no points or Axioms');
  const names = await evaluate(`[...document.querySelectorAll('.cx-node .cx-name, .cx-theorem .cx-name')].map((e) => e.textContent)`);
  check(names.length === 36 && names.every((n) => n.includes('(placeholder)')), `all ${names.length} names say placeholder`);
  check(
    JSON.stringify(await evaluate(`[...document.querySelectorAll('.cx-col h3')].map((e) => e.textContent)`)) ===
      JSON.stringify(['Serenity', 'Logistics', 'Logic', 'Rhetoric', 'Vigilance']),
    'five named columns',
  );
  check((await attr('[data-node="serenity-2"]', 'disabled')) !== null, 'a node needs the one before it');
  await click('#cortex-grant-1000', 'grant Insight');
  check((await text('#cortex-insight')) === '1,000', 'Insight granted');
  await buyDown('serenity');
  check((await text('#cortex-axioms')) === '1', 'the Axiom arrives at Serenity’s end');
  check((await text('#cortex-flash')).includes('+1 Axiom from Serenity'), 'its notice shows');
  check((await attr('[data-theorem="serenity-logistics"]', 'disabled')) !== null, 'a pair Theorem waits for 2 Axioms');
  await buyDown('logistics');
  check((await text('#cortex-axioms')) === '2', 'a second Axiom from Logistics');
  await click('[data-theorem="serenity-logistics"]', 'learn a pair Theorem');
  check((await attr('[data-theorem="serenity-logistics"]', 'data-learned')) === '1', 'the pair Theorem is learned');
  check((await text('#cortex-axioms')) === '0', 'it spent 2 Axioms');
  check((await attr('[data-theorem="unified"]', 'disabled')) !== null, 'the Unified Theorem waits for 5');
  // A little more Insight, a few more nodes, and two points left free for the picture.
  await click('#cortex-grant-1000', 'grant more Insight');
  for (const n of ['logic-1', 'logic-2', 'rhetoric-1']) await click(`[data-node="${n}"]`, n);
  check((await text('#cortex-tp')) === '2', 'two points left free');
  await shot('cortex-light');
  await click('#cortex-close', 'close');
  await click('[data-theme-pick="dark"]', 'dark');
  await click('#cortex-open', 'reopen');
  check((await text('#cortex-axioms')) === '0' && (await attr('[data-theorem="serenity-logistics"]', 'data-learned')) === '1', 'state kept across close and reopen');
  await shot('cortex-dark');
  await click('#cortex-respec', 'respec');
  const earned = /free of (\d+)/.exec(await text('.cx-bar'))?.[1];
  check((await text('#cortex-tp')) === earned && (await text('#cortex-axioms')) === '0', `respec refunds all ${earned} points`);
  check((await attr('[data-theorem="serenity-logistics"]', 'data-learned')) === '0', 'and the Theorem');
  await click('#cortex-close', 'close');
  await click('[data-theme-pick="light"]', 'light');
  check((await actions()) === before, `the game saw no actions (${before} before and after)`);
  check(b.errors.length === 0, `no page errors${b.errors.length ? `: ${b.errors.join('; ')}` : ''}`);
} finally {
  await b.close();
}
