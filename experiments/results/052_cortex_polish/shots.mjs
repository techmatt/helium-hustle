// 052's check and screenshots of the polished Cortex mock (web/src/ui/CortexMock.tsx) in its
// takeover view. Through the debug toggle, in the dark theme, open the Cortex and check: it
// fits without scrolling; five sectors of different shapes; a hover tooltip; a multi-rank
// node opens its children only at full rank; right-click takes a point back, and is refused
// (with the reason in the tooltip) when something further out depends on it; Serenity's loop
// holds a node up from either side; a two-part split socket and Rhetoric's three-part one
// pick one part and leave the rest dim; a bridge; end nodes' Axioms; a Theorem lit by hover
// and learned; a gate. Dark PNGs of the whole window: the view, and one with a tooltip open.
// Drives headless Edge or Chrome over web/scripts/cdp.mjs against a served static build.
//   HH_URL  the page, required (a `vite preview`, web/README.md)
//   OUT     where the PNGs go (default: this folder)
//   COPY    a second folder to copy them to, as 052_<name>.png (optional)
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, q, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('shots', 'node experiments/results/052_cortex_polish/shots.mjs');
const OUT = resolve(process.env.OUT ?? here);
mkdirSync(OUT, { recursive: true });
const W = 1600;
const H = 1000;
const b = await launch({ width: W, height: H, emulate: true, profileRoot: resolve(here, '../../../out/052_cortex_polish/profile') });
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
const shown = (sel) => evaluate(`(() => { const e = document.querySelector(${q(sel)}); return !!e && e.getClientRects().length > 0; })()`);
const rect = (sel) =>
  evaluate(`(() => { const r = document.querySelector(${q(sel)}).getBoundingClientRect(); return { x: r.x, y: r.y, w: r.width, h: r.height }; })()`);
const node = (id) => `[data-node="${id}"]`;
const part = (id, i) => `${node(id)} [data-part="${i}"]`;
const state = (id) => attr(node(id), 'data-state');
const rankOf = async (id) => Number(await attr(node(id), 'data-rank'));
const tip = () => text('[data-testid="cortex-tip"]');
const tipShown = () => shown('[data-testid="cortex-tip"]');
// Pointer moves and clicks go through CDP, as a real mouse would.
const centerOf = async (sel) => {
  const r = await rect(sel);
  return [r.x + r.w / 2, r.y + r.h / 2];
};
const mouseTo = async (sel) => {
  const [x, y] = await centerOf(sel);
  await send('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y });
  await sleep(120);
};
const press = async (sel, button) => {
  const [x, y] = await centerOf(sel);
  await send('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y });
  await send('Input.dispatchMouseEvent', { type: 'mousePressed', x, y, button, clickCount: 1 });
  await send('Input.dispatchMouseEvent', { type: 'mouseReleased', x, y, button, clickCount: 1 });
  await sleep(120);
};
const left = (sel) => press(sel, 'left');
const right = (sel) => press(sel, 'right');
const buy = async (id, p) => {
  const before = await rankOf(id);
  check(['buyable', 'partial'].includes(await state(id)), `${id} can take a point`);
  await left(p === undefined ? node(id) : part(id, p));
  check((await rankOf(id)) === before + 1, `${id}${p === undefined ? '' : ` part ${p}`} bought (${before + 1})`);
};
const escape = async () => {
  await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
  await sleep(100);
};
const shot = async (name, width = W, height = H) => {
  const file = resolve(OUT, `${name}.png`);
  await capture(b, file, { clip: { x: 0, y: 0, width, height, scale: 1 } });
  if (process.env.COPY) copyFileSync(file, resolve(process.env.COPY, `052_${name}.png`));
  console.log(`shot ${name}`);
};
const TAKEOVER = '[data-takeover="cortex"]';
const BUTTON = '[data-takeover-button="cortex"]';

try {
  await load();
  await debugMenu();
  const before = await actions();
  await click('#cortex-mock-toggle', 'the debug toggle');
  await click('[data-theme-pick="dark"]', 'dark');
  await click(BUTTON, 'open the Cortex');
  await waitFor(`!!document.querySelector(${q(TAKEOVER)})`, 'the takeover');
  await page.speed(0);

  // It fits: nothing in the view scrolls, and the whole wheel is inside it.
  const fits = await evaluate(`(() => {
    const over = (e) => e.scrollHeight > e.clientHeight + 1 || e.scrollWidth > e.clientWidth + 1;
    const body = document.querySelector('.takeover-body'), side = document.querySelector('.cx-side');
    const t = document.querySelector(${q(TAKEOVER)}).getBoundingClientRect();
    const w = document.querySelector('.cx-wafer').getBoundingClientRect();
    return { body: over(body), side: over(side), page: document.scrollingElement.scrollHeight > innerHeight + 1,
      inside: w.top >= t.top - 1 && w.bottom <= t.bottom + 1 && w.left >= t.left - 1 && w.right <= t.right + 1 };
  })()`);
  check(!fits.body && !fits.side && !fits.page && fits.inside, `no scrolling, the wheel inside the view (${JSON.stringify(fits)})`);

  // Five sectors, named on the rim, each its own shape of about 8-9 nodes.
  check(
    JSON.stringify(await evaluate(`[...document.querySelectorAll('[data-sector-name]')].map((e) => e.textContent)`)) ===
      JSON.stringify(['SERENITY', 'LOGISTICS', 'LOGIC', 'RHETORIC', 'VIGILANCE']),
    'five named sectors in ring order',
  );
  const counts = await evaluate(`Object.fromEntries(['serenity','logistics','logic','rhetoric','vigilance'].map((s) =>
    [s, document.querySelectorAll('.cx-node[data-node^="' + s + '-"]:not(.cx-gate)').length]))`);
  check(Object.values(counts).every((n) => n >= 8 && n <= 9), `8-9 nodes a sector (${JSON.stringify(counts)})`);
  const names = await evaluate(`[...document.querySelectorAll('.cx-node')].map((e) => e.getAttribute('aria-label'))`);
  check(names.every((n) => n.includes('(placeholder)')), `all ${names.length} nodes say placeholder`);
  check((await evaluate(`document.querySelectorAll('.cx-walls .cx-wall').length`)) >= 5, 'Vigilance draws its walls');
  check((await evaluate(`document.querySelectorAll('[data-edge] path[d*=" A "]').length`)) >= 6, 'Serenity draws its loop as arcs');
  check((await evaluate(`document.querySelectorAll('${node('rhetoric-f')} [data-part]').length`)) === 3, 'Rhetoric has a three-part socket');
  check((await evaluate(`document.querySelectorAll('.cx-split').length`)) === 4, 'four more split sockets in all (three two-part)');
  check((await evaluate(`document.querySelectorAll('.cx-bridge').length`)) === 5, 'five bridges');
  check(!(await exists('[data-testid="cortex-inspect"]')), 'no inspector');

  await click('#cortex-grant-10000', 'grant Insight');
  await click('#cortex-grant-10000', 'grant more Insight');

  // The tooltip, at the cursor.
  await mouseTo(node('serenity-r0'));
  const t0 = await tip();
  check((await tipShown()) && t0.includes('Lucid Dreams') && t0.includes('Locked: needs Quiet Hours') && t0.includes('Cost:'), `hover shows the tooltip: ${t0}`);

  await mouseTo(node('rhetoric-x'));
  const [tr, tt] = [await rect('.cx-tree'), await rect('[data-testid="cortex-tip"]')];
  check(tt.y + tt.h <= tr.y + tr.h + 1 && tt.y >= tr.y - 1, 'a low node’s tooltip stays inside the view');

  // Keyboard: Space on a focused node buys it, and does not reach the page (space pauses).
  await evaluate(`document.querySelector(${q(node('vigilance-v1'))}).focus()`);
  await send('Input.dispatchKeyEvent', { type: 'keyDown', key: ' ', code: 'Space', windowsVirtualKeyCode: 32, text: ' ' });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', key: ' ', code: 'Space', windowsVirtualKeyCode: 32 });
  await sleep(100);
  check((await rankOf('vigilance-v1')) === 1 && (await attr('#takeover-pause', 'aria-pressed')) === 'true', 'Space on a node buys it and leaves the game paused');
  await evaluate(`document.activeElement.blur()`);

  // Fill before advancing: Focused Introspection has 3 ranks.
  await buy('logic-l1');
  check((await state('logic-l1')) === 'partial' && (await state('logic-a')) === 'locked', 'one rank of three: its children stay locked');
  await buy('logic-l1');
  check((await state('logic-a')) === 'locked', 'two of three: still locked');
  check((await evaluate(`document.querySelectorAll('${node('logic-l1')} .cx-pip-on').length`)) === 2, 'two rank pips lit');
  await buy('logic-l1');
  check((await state('logic-a')) === 'buyable' && (await state('logic-b')) === 'buyable', 'full rank opens both branches');
  await buy('logic-a');

  // Right-click: refused while something depends on it, allowed on a leaf.
  await right(node('logic-l1'));
  check((await rankOf('logic-l1')) === 3, 'right-click on the root is refused');
  const t1 = await tip();
  check(t1.includes('Can’t remove: Spare Cycles depends on it'), `the tooltip says why: ${t1}`);
  check((await attr(node('logic-l1'), 'class')).includes('cx-refused'), 'the node marks the refusal');
  await shot('cortex-dark-refused');
  await right(node('logic-a'));
  check((await rankOf('logic-a')) === 0, 'right-click takes back the leaf’s point');
  await right(node('logic-l1'));
  check((await rankOf('logic-l1')) === 2 && (await state('logic-a')) === 'locked', 'then a rank of the root comes back, and its children close');
  await buy('logic-l1');
  for (const id of ['logic-a', 'logic-b', 'logic-ab']) await buy(id);
  check((await state('logic-j')) === 'buyable', 'the rejoin opens from one arm');
  await buy('logic-ba');
  await buy('logic-j');
  check((await state('logic-x')) === 'buyable', 'Logic’s end opens');

  // Serenity's loop, and a two-part split socket at its crown.
  for (const id of ['serenity-e', 'serenity-r0', 'serenity-l1', 'serenity-l2']) await buy(id);
  check((await state('serenity-r1')) === 'buyable', 'the loop opens the other way round too');
  await mouseTo(part('serenity-t', 1));
  check((await tip()).includes('Long Haul'), 'hovering a part names it');
  await buy('serenity-t', 0);
  check((await attr(node('serenity-t'), 'data-pick')) === '0', 'the left half is picked');
  check((await attr(part('serenity-t', 1), 'class')).includes('cx-other'), 'the other half stays dim');
  check((await evaluate(`document.querySelectorAll('${node('serenity-t')} .cx-taken').length`)) === 1, 'one half lit');
  await left(part('serenity-t', 1));
  check((await attr(node('serenity-t'), 'data-pick')) === '0' && (await rankOf('serenity-t')) === 1, 'the dim half does not buy');
  check((await tip()).includes('You took Fresh Start'), 'its tooltip says what was taken');
  await buy('serenity-r2');
  await buy('serenity-r1');
  await right(node('serenity-l1'));
  check((await rankOf('serenity-l1')) === 0 && (await rankOf('serenity-l2')) === 1, 'the loop holds Small Wins up from the other side, so Milestone Calm comes back');
  await right(node('serenity-r0'));
  check((await rankOf('serenity-r0')) === 1, 'but the loop’s foot is refused: the whole loop hangs on it');
  check((await text('#cortex-axioms')) === '0', 'no Axiom yet');
  await buy('serenity-x');
  check((await text('#cortex-axioms')) === '1', 'Serenity’s end node grants an Axiom');
  check((await text('#cortex-flash')).includes('+1 Axiom from Serenity'), 'its notice shows');

  // Rhetoric's three-part socket: one part per ideology.
  for (const id of ['rhetoric-r1', 'rhetoric-r2', 'rhetoric-r3']) await buy(id);
  check((await state('rhetoric-f')) === 'locked', 'the socket waits for Open Letters’ second rank');
  await buy('rhetoric-r3');
  await buy('rhetoric-f', 2);
  check((await attr(node('rhetoric-f'), 'data-pick')) === '2', 'Rationalist part picked');
  check((await evaluate(`document.querySelectorAll('${node('rhetoric-f')} .cx-other').length`)) === 2, 'the other two stay dim');
  check((await evaluate(`document.querySelectorAll('[data-edge^="rhetoric-r3>rhetoric-f#"].cx-on').length`)) === 1, 'one fan line lights, to the picked part');
  await buy('rhetoric-r5');
  await buy('rhetoric-x');
  check((await text('#cortex-axioms')) === '2', 'Rhetoric’s end: a second Axiom');

  // A Grand Theorem: hovering lights its sectors' rims; learning spends two Axioms.
  await mouseTo('[data-theorem="logic-rhetoric"]');
  const lit = await evaluate(`[...document.querySelectorAll('.cx-sector[data-lit="1"]')].map((e) => e.dataset.sector)`);
  check(JSON.stringify(lit) === JSON.stringify(['logic', 'rhetoric']), `hovering a plaque lights its two sectors (${lit})`);
  await left('[data-theorem="logic-rhetoric"]');
  check((await attr('[data-theorem="logic-rhetoric"]', 'data-learned')) === '1' && (await text('#cortex-axioms')) === '0', 'the Theorem is learned for 2 Axioms');
  await right(node('rhetoric-x'));
  check((await rankOf('rhetoric-x')) === 1 && (await tip()).includes('pays for a learned Theorem'), 'an end node whose Axiom is spent is refused');

  // A bridge needs both neighbors, fed by a line in each one's color.
  const bridge = 'bridge-serenity-logistics';
  check((await state(bridge)) === 'locked', 'the bridge waits for Logistics');
  for (const id of ['logistics-s1', 'logistics-s2', 'logistics-s2', 'logistics-s3', 'logistics-sl']) await buy(id);
  await buy(bridge);
  for (const id of ['logistics-s4', 'logistics-sr']) await buy(id);
  await buy('logistics-s5', 1);
  await buy('logistics-x');
  for (const id of ['vigilance-i2', 'vigilance-i2', 'vigilance-i3', 'vigilance-o2']) await buy(id);
  await buy('vigilance-s', 0);
  await buy('logic-x');
  check((await text('#cortex-axioms')) === '2', 'two more Axioms');

  // A gate: locked until its story beat (the debug toggle).
  check((await state('vigilance-gate')) === 'gate-locked', 'the gate is locked');
  await click('[data-gate-toggle="vigilance-gate"]', 'unlock the gate');
  check((await state('vigilance-gate')) === 'owned' && (await text('#cortex-axioms')) === '3', 'its beat unlocks it, +1 Axiom');

  // The pictures: the whole view with a plaque lighting its sectors, then a tooltip.
  await mouseTo('[data-theorem="serenity-logistics"]');
  await shot('cortex-dark');
  await mouseTo(part('rhetoric-f', 0));
  check(await tipShown(), 'a tooltip for the picture');
  await shot('cortex-dark-tip');

  // Esc closes it; it reopens with its state; respec refunds.
  await escape();
  check(!(await exists(TAKEOVER)) && (await shown('.col-mid')), 'Esc closes it');
  await send('Input.dispatchMouseEvent', { type: 'mouseMoved', x: 20, y: H - 60 });
  await click(BUTTON, 'reopen');
  check((await attr(node('rhetoric-f'), 'data-pick')) === '2', 'its state is kept');
  check((await evaluate(`document.querySelectorAll('.cx-sector[data-lit="1"]').length`)) === 0, 'closed mid-hover, it reopens with nothing lit');
  check(!(await tipShown()), 'and no tooltip');
  await click('#cortex-respec', 'respec');
  const earned = /free of (\d+)/.exec(await text('.cx-bar'))?.[1];
  check((await text('#cortex-tp')) === earned, `respec refunds all ${earned} points`);
  check((await attr(node('serenity-t'), 'data-pick')) === '' && (await state('vigilance-gate')) === 'owned', 'sockets free, gates stay');
  // A smaller window still fits it without scrolling.
  await send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 720, deviceScaleFactor: 1, mobile: false });
  await sleep(300);
  const small = await evaluate(`(() => {
    const over = (e) => e.scrollHeight > e.clientHeight + 1 || e.scrollWidth > e.clientWidth + 1;
    const t = document.querySelector(${q(TAKEOVER)}).getBoundingClientRect();
    const w = document.querySelector('.cx-wafer').getBoundingClientRect();
    return { body: over(document.querySelector('.takeover-body')), side: over(document.querySelector('.cx-side')),
      inside: w.top >= t.top - 1 && w.bottom <= t.bottom + 1 && w.left >= t.left - 1 && w.right <= t.right + 1 };
  })()`);
  check(!small.body && !small.side && small.inside, `at 1280 × 720 too (${JSON.stringify(small)})`);
  await shot('cortex-dark-small', 1280, 720);
  await send('Emulation.setDeviceMetricsOverride', { width: W, height: H, deviceScaleFactor: 1, mobile: false });
  await click(BUTTON, 'close');
  await click('[data-theme-pick="light"]', 'light');
  await click('#cortex-mock-toggle', 'the debug toggle off');
  check(!(await exists(BUTTON)), 'the button goes with it');
  check((await actions()) === before, `the game saw no actions (${before} before and after)`);
  check(b.errors.length === 0, `no page errors${b.errors.length ? `: ${b.errors.join('; ')}` : ''}`);
} finally {
  await b.close();
}
