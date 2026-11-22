// 051's check and screenshots of the takeover view (web/src/ui/Takeover.tsx) and the radial
// Cortex mock in it (web/src/ui/CortexMock.tsx). Through the debug toggle, open the Cortex
// from the left column; check it covers the middle and right columns, the strip shows and
// the game runs on; take one prong of a fork, then the other; buy a bridge; take one of an
// exclusive pair and see its partner lock; reach an end node and its Axiom; light a pair
// Theorem by hovering it and learn it; unlock a gate; pause from the strip; close with Esc
// and with the button. Light and dark PNGs of the whole window.
// Drives headless Edge or Chrome over web/scripts/cdp.mjs against a served static build.
//   HH_URL  the page, required (a `vite preview`, web/README.md)
//   OUT     where the PNGs go (default: this folder)
//   COPY    a second folder to copy them to (optional)
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, q, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('shots', 'node experiments/results/051_cortex_radial_takeover/shots.mjs');
const OUT = resolve(process.env.OUT ?? here);
mkdirSync(OUT, { recursive: true });
const W = 1600;
const H = 1000;
const b = await launch({ width: W, height: H, emulate: true, profileRoot: resolve(here, '../../../out/051_cortex_radial_takeover/profile') });
const { evaluate, send } = b;
const page = driver(b, { url });
const { waitFor, click, load, debugMenu, exists } = page;
const text = (sel) => evaluate(`document.querySelector(${q(sel)})?.textContent ?? null`);
const attr = (sel, a) => evaluate(`document.querySelector(${q(sel)})?.getAttribute(${q(a)}) ?? null`);
const check = (ok, what) => {
  if (!ok) throw new Error(`FAILED: ${what}`);
  console.log(`ok: ${what}`);
};
const actions = async () => /(\d+) actions/.exec(await text('#debug-info'))?.[1];
const day = async () => Number((await text('#day')).replace(/\D/g, ''));
const shown = (sel) => evaluate(`(() => { const e = document.querySelector(${q(sel)}); return !!e && e.getClientRects().length > 0; })()`);
const rect = (sel) =>
  evaluate(`(() => { const r = document.querySelector(${q(sel)}).getBoundingClientRect(); return { x: r.x, y: r.y, w: r.width, h: r.height }; })()`);
const node = (id) => `[data-node="${id}"]`;
const state = (id) => attr(node(id), 'data-state');
// SVG groups have no .click(): send a real click event, as a pointer would.
const tap = async (id) => {
  const ok = await evaluate(`(() => { const n = document.querySelector(${q(node(id))}); if (!n) return false;
    n.dispatchEvent(new MouseEvent('click', { bubbles: true })); return true; })()`);
  if (!ok) throw new Error(`no node ${id}`);
};
const buy = async (id) => {
  const before = Number(await attr(node(id), 'data-rank'));
  check(['buyable', 'partial'].includes(await state(id)), `${id} buyable`);
  await tap(id);
  check(Number(await attr(node(id), 'data-rank')) === before + 1, `${id} bought`);
};
const mouseTo = async (sel) => {
  const r = await rect(sel);
  await send('Input.dispatchMouseEvent', { type: 'mouseMoved', x: r.x + r.w / 2, y: r.y + r.h / 2 });
  await sleep(150);
};
const escape = async () => {
  await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
  await sleep(100);
};
const shot = async (name) => {
  const file = resolve(OUT, `${name}.png`);
  await capture(b, file, { clip: { x: 0, y: 0, width: W, height: H, scale: 1 } });
  if (process.env.COPY) copyFileSync(file, resolve(process.env.COPY, `051_${name}.png`));
  console.log(`shot ${name}`);
};
const TAKEOVER = '[data-takeover="cortex"]';
const BUTTON = '[data-takeover-button="cortex"]';

try {
  await load();
  await debugMenu();
  const before = await actions();
  check(!(await exists(BUTTON)), 'no Cortex button in the left column by default');
  await click('#cortex-mock-toggle', 'the debug toggle');
  check(await shown(`.col-left ${BUTTON}`), 'the toggle shows the Cortex button in the left column');

  // The game runs at 1x while the view is open.
  await page.speed(1);
  await click(BUTTON, 'open the Cortex');
  await waitFor(`!!document.querySelector(${q(TAKEOVER)})`, 'the takeover');
  check(!(await shown('.col-mid')) && !(await shown('.col-right')), 'the middle and right columns are covered');
  check(await shown('.col-left') && (await shown('#speeds')), 'the left column stays');
  const left = await rect('.col-left');
  const t = await rect(TAKEOVER);
  check(Math.abs(t.x - (left.x + left.w)) < 2 && Math.abs(t.x + t.w - W) < 2, `the view spans from the left column to the window's edge (${Math.round(t.x)}..${Math.round(t.x + t.w)})`);
  check(await shown('[data-testid="takeover-strip"] .bar'), 'the strip shows boredom');
  check(await shown('[data-strip-res="energy"]'), 'and Energy');
  check(await shown('#takeover-pause'), 'and pause');
  check((await attr(BUTTON, 'aria-pressed')) === 'true', 'the button reads as pressed');
  const d0 = await day();
  await waitFor(`Number(document.getElementById('day').textContent.replace(/\\D/g, '')) > ${d0}`, 'a day to pass', 5000);
  check(true, `the game runs on behind it (day ${d0} → ${await day()})`);

  check(
    JSON.stringify(await evaluate(`[...document.querySelectorAll('.cx-sector-name')].map((e) => e.textContent)`)) ===
      JSON.stringify(['Serenity', 'Logistics', 'Logic', 'Rhetoric', 'Vigilance']),
    'five named sectors in ring order',
  );
  const names = await evaluate(`[...document.querySelectorAll('.cx-node')].map((e) => e.getAttribute('aria-label'))`);
  check(names.every((n) => n.includes('(placeholder)')), `all ${names.length} nodes say placeholder`);
  for (const k of ['prong', 'join', 'bridge', 'gate', 'exclusive', 'end'])
    check((await evaluate(`document.querySelectorAll('.cx-node.cx-${k}').length`)) > 0, `a ${k} node is drawn`);

  await click('#cortex-grant-10000', 'grant Insight');
  await click('#cortex-grant-10000', 'grant more Insight');

  // A fork: race one prong to the rejoin, then take the other.
  check((await state('serenity-1')) === 'buyable', 'the entry opens from the core');
  await buy('serenity-1');
  check((await state('serenity-2a')) === 'buyable' && (await state('serenity-2b')) === 'buyable', 'both prongs open');
  await buy('serenity-2a');
  await buy('serenity-3a');
  check((await state('serenity-4')) === 'buyable', 'one prong opens the rejoin');
  await buy('serenity-2b');
  await buy('serenity-3b');
  check((await state('serenity-3b')) === 'owned', 'then the other prong too');

  // A bridge needs both neighbors.
  const bridge = 'bridge-serenity-logistics';
  check((await state(bridge)) === 'locked', 'the bridge waits for Logistics');
  await buy('logistics-1');
  await buy('logistics-2a');
  check((await state('logistics-2a')) === 'partial', 'a two-rank node shows a rank of two');
  await buy('logistics-3a');
  check((await state(bridge)) === 'buyable', 'both neighbors open the bridge');
  await buy(bridge);
  check((await state(bridge)) === 'owned', 'the bridge is bought');

  // An exclusive pair.
  await buy('serenity-4');
  await buy('serenity-5a');
  check((await state('serenity-5b')) === 'excluded', 'taking one of a pair locks its partner');
  check((await attr('[data-or="serenity-5a"]', 'class')).includes('cx-taken'), 'its "or" turns');
  await tap('serenity-5b');
  check((await attr(node('serenity-5b'), 'data-rank')) === '0', 'the locked partner does not buy');
  check((await text('[data-testid="cortex-inspect"]')).includes('exclusive partner'), 'the inspector says why');

  // An end node and its Axiom.
  check((await text('#cortex-axioms')) === '0', 'no Axiom yet');
  await buy('serenity-6');
  check((await text('#cortex-axioms')) === '1', 'the end node grants an Axiom');
  check((await text('#cortex-flash')).includes('+1 Axiom from Serenity'), 'its notice shows');

  // Logistics to its end, taking the other side of its pair.
  await buy('logistics-4');
  await buy('logistics-5b');
  check((await state('logistics-5a')) === 'excluded', 'Logistics’ pair locks too');
  await buy('logistics-6');
  check((await text('#cortex-axioms')) === '2', 'a second Axiom');

  // A pair Theorem from the side list: hovering lights its two sectors.
  const th = '[data-theorem="serenity-logistics"]';
  await mouseTo(th);
  const lit = await evaluate(`[...document.querySelectorAll('.cx-wedge[data-lit="1"]')].map((e) => e.dataset.sector)`);
  check(JSON.stringify(lit) === JSON.stringify(['serenity', 'logistics']), `hovering it lights its two sectors (${lit})`);
  await click(th, 'learn the pair Theorem');
  check((await attr(th, 'data-learned')) === '1', 'the pair Theorem is learned');
  check((await text('#cortex-axioms')) === '0', 'it spent 2 Axioms');
  check((await attr('[data-theorem="unified"]', 'disabled')) !== null, 'the Unified Theorem waits for 5');

  // A gate: locked until its story beat (the debug toggle).
  check((await state('serenity-gate')) === 'gate-locked', 'the gate is locked');
  await click('[data-gate-toggle="serenity-gate"]', 'unlock the gate');
  check((await state('serenity-gate')) === 'owned' && (await text('#cortex-axioms')) === '1', 'its beat unlocks it, +1 Axiom');

  // A few more for the picture.
  for (const id of ['logic-1', 'logic-1', 'logic-2b', 'vigilance-1', 'vigilance-2a', 'rhetoric-1']) await buy(id);

  // Pause from the strip.
  await click('#takeover-pause', 'pause from the strip');
  check((await attr('#takeover-pause', 'aria-pressed')) === 'true' && (await attr('#speeds [data-speed="0"]', 'aria-pressed')) === 'true', 'the strip pauses the game');

  await mouseTo('[data-theorem="logistics-logic"]');
  await mouseTo(node('serenity-5b'));
  await mouseTo('[data-theorem="logistics-logic"]');
  await shot('cortex-light');

  // Esc closes it; the button reopens it with its state; the button closes it again.
  await escape();
  check(!(await exists(TAKEOVER)) && (await shown('.col-mid')) && (await shown('.col-right')), 'Esc closes it and the columns return');
  await click(BUTTON, 'reopen');
  check((await attr(th, 'data-learned')) === '1' && (await state('serenity-5b')) === 'excluded', 'its state is kept');
  check((await evaluate(`document.querySelectorAll('.cx-wedge[data-lit="1"]').length`)) === 0, 'closed mid-hover, it reopens with no sector lit');
  await click(BUTTON, 'the button again');
  check(!(await exists(TAKEOVER)), 'the same button closes it');

  await click('[data-theme-pick="dark"]', 'dark');
  await click(BUTTON, 'reopen in dark');
  await mouseTo('[data-theorem="logistics-logic"]');
  await shot('cortex-dark');
  await click('#cortex-respec', 'respec');
  const earned = /free of (\d+)/.exec(await text('.cx-bar'))?.[1];
  check((await text('#cortex-tp')) === earned, `respec refunds all ${earned} points`);
  check((await attr(th, 'data-learned')) === '0', 'and the Theorem');
  check((await state('serenity-5b')) === 'locked', 'and frees the exclusive pair');
  check((await state('serenity-gate')) === 'owned', 'gates stay (the story granted them)');
  await escape();
  await click('[data-theme-pick="light"]', 'light');
  await click('#cortex-mock-toggle', 'the debug toggle off');
  check(!(await exists(BUTTON)), 'the button goes with it');
  check((await actions()) === before, `the game saw no actions (${before} before and after)`);
  check(b.errors.length === 0, `no page errors${b.errors.length ? `: ${b.errors.join('; ')}` : ''}`);
} finally {
  await b.close();
}
