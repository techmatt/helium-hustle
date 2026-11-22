// 053's check and screenshots of the Cortex mock (web/src/ui/CortexMock.tsx) on its plain
// ground, in both themes. Through the debug toggle, open the Cortex and check: no backdrop art,
// a solid ground in each theme's color; it fits without scrolling; then, in dark and again in
// light: a two-part socket and Rhetoric's three-part fan switch parts with one left-click each
// (free, and nothing beyond them stranded: the node past the socket stays owned, and the
// socket still refuses removal); a bought bridge is filled in both neighbors' colors. Dark
// also keeps 052's regressions (keyboard, a refused removal, Esc, respec, no game actions).
// PNGs of the whole window per theme: the view, and one with a tooltip on a dim fan part.
// Drives headless Edge or Chrome over web/scripts/cdp.mjs against a served static build.
//   HH_URL  the page, required (a `vite preview`, web/README.md)
//   OUT     where the PNGs go (default: this folder)
//   COPY    a second folder to copy them to, as 053_<name>.png (optional)
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, q, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('shots', 'node experiments/results/053_cortex_contrast_light/shots.mjs');
const OUT = resolve(process.env.OUT ?? here);
mkdirSync(OUT, { recursive: true });
const W = 1600;
const H = 1000;
const b = await launch({ width: W, height: H, emulate: true, profileRoot: resolve(here, '../../../out/053_cortex_contrast_light/profile') });
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
const pickOf = (id) => attr(node(id), 'data-pick');
const tp = async () => Number(await text('#cortex-tp'));
const tip = () => text('[data-testid="cortex-tip"]');
const tipShown = () => shown('[data-testid="cortex-tip"]');
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
  if (process.env.COPY) copyFileSync(file, resolve(process.env.COPY, `053_${name}.png`));
  console.log(`shot ${name}`);
};
const TAKEOVER = '[data-takeover="cortex"]';
const BUTTON = '[data-takeover-button="cortex"]';
/** Nothing scrolls, and the wheel (its rim names, its outermost drawing) sits inside the view. */
const fits = () =>
  evaluate(`(() => {
    const over = (e) => e.scrollHeight > e.clientHeight + 1 || e.scrollWidth > e.clientWidth + 1;
    const t = document.querySelector(${q(TAKEOVER)}).getBoundingClientRect();
    const rs = [...document.querySelectorAll('[data-sector-name], .cx-rim-glow')].map((e) => e.getBoundingClientRect());
    const w = { top: Math.min(...rs.map((r) => r.top)), bottom: Math.max(...rs.map((r) => r.bottom)), left: Math.min(...rs.map((r) => r.left)), right: Math.max(...rs.map((r) => r.right)) };
    return { body: over(document.querySelector('.takeover-body')), side: over(document.querySelector('.cx-side')),
      page: document.scrollingElement.scrollHeight > innerHeight + 1,
      inside: w.top >= t.top - 1 && w.bottom <= t.bottom + 1 && w.left >= t.left - 1 && w.right <= t.right + 1 };
  })()`);
/** The view's ground: a solid color, no image or gradient, and no backdrop drawing. */
const ground = () =>
  evaluate(`(() => { const s = getComputedStyle(document.querySelector('.cx-root'));
    return { color: s.backgroundColor, image: s.backgroundImage, art: document.querySelectorAll('.cx-backdrop, .cx-wafer, .cx-traces, .cx-neurons').length }; })()`);

/** The done-when flow, from a respec'd tree: switching a socket and the fan, and a bridge. */
async function pass(theme) {
  check((await tp()) >= 30, `${theme}: points to spend (${await tp()})`);
  // Serenity's crown, a two-part socket: take one part, then switch with one click, free.
  for (const id of ['serenity-e', 'serenity-r0', 'serenity-l1', 'serenity-l2']) await buy(id);
  await buy('serenity-t', 0);
  check((await pickOf('serenity-t')) === '0' && (await attr(part('serenity-t', 1), 'class')).includes('cx-other'), `${theme}: part 0 lit, part 1 dim`);
  await mouseTo(part('serenity-t', 1));
  check((await tip()).includes('Click to switch from Fresh Start'), `${theme}: the dim part's tooltip offers the switch`);
  await buy('serenity-x');
  const free = await tp();
  await left(part('serenity-t', 1));
  check((await pickOf('serenity-t')) === '1' && (await rankOf('serenity-t')) === 1 && (await tp()) === free, `${theme}: one left-click switches the socket, free`);
  check((await text('#cortex-flash')).includes('Switched Fresh Start to Long Haul'), `${theme}: its notice says so`);
  check((await state('serenity-x')) === 'owned', `${theme}: the end node past the socket stays owned`);
  await left(part('serenity-t', 0));
  check((await pickOf('serenity-t')) === '0' && (await tp()) === free, `${theme}: and back, one click`);
  await right(node('serenity-t'));
  check((await rankOf('serenity-t')) === 1 && (await tip()).includes('Serenity Prime depends on it'), `${theme}: the socket still holds up its end node (removal refused)`);

  // Rhetoric's fan: three lines into the three-part socket; each switch is one click.
  for (const id of ['rhetoric-r1', 'rhetoric-r2', 'rhetoric-r3', 'rhetoric-r3']) await buy(id);
  await buy('rhetoric-f', 2);
  await buy('rhetoric-r5');
  const fanLit = () => evaluate(`[...document.querySelectorAll('[data-edge^="rhetoric-r3>rhetoric-f#"].cx-on')].map((e) => e.dataset.edge.slice(-1)).join()`);
  check((await fanLit()) === '2', `${theme}: the fan lights the Reason line`);
  for (const p of [0, 1]) {
    await left(part('rhetoric-f', p));
    check((await pickOf('rhetoric-f')) === String(p) && (await fanLit()) === String(p), `${theme}: one click switches the fan to line ${p}`);
    check((await state('rhetoric-r5')) === 'owned' && (await tp()) === free - 6, `${theme}: Spin, past it, stays owned; no point spent`);
  }

  // A bridge: bought, its disc filled half in each neighbor's color.
  const bridge = 'bridge-serenity-logistics';
  for (const id of ['serenity-r1', 'serenity-r2', 'logistics-s1', 'logistics-s2', 'logistics-s2', 'logistics-s3', 'logistics-sl']) await buy(id);
  await buy(bridge);
  const halves = await evaluate(`[...document.querySelectorAll('${node(bridge)} .cx-bridge-fill')].map((e) => e.style.fill)`);
  check(halves.length === 2 && halves[0].includes('cx-fill-serenity') && halves[1].includes('cx-fill-logistics'), `${theme}: the bought bridge is filled in both colors (${halves})`);
  const stops = await evaluate(`['serenity', 'logistics'].map((s) => getComputedStyle(document.querySelector('#cx-fill-' + s + ' stop:nth-child(2)')).stopColor)`);
  check(stops[0] !== stops[1], `${theme}: its two fills differ (${stops})`);

  // The pictures: the whole view with a plaque lighting its sectors, then a tooltip on a dim fan part.
  await mouseTo('[data-theorem="serenity-logistics"]');
  await shot(`cortex-${theme}`);
  await mouseTo(part('rhetoric-f', 2));
  check((await tipShown()) && (await tip()).includes('Click to switch'), `${theme}: a tooltip for the picture`);
  await shot(`cortex-${theme}-tip`);
  await send('Input.dispatchMouseEvent', { type: 'mouseMoved', x: W - 5, y: 5 });
  await click('#cortex-respec', 'respec');
}

try {
  await load();
  await debugMenu();
  const before = await actions();
  await click('#cortex-mock-toggle', 'the debug toggle');
  await click('[data-theme-pick="dark"]', 'dark');
  await click(BUTTON, 'open the Cortex');
  await waitFor(`!!document.querySelector(${q(TAKEOVER)})`, 'the takeover');
  await page.speed(0);

  const f = await fits();
  check(!f.body && !f.side && !f.page && f.inside, `no scrolling, the wheel inside the view (${JSON.stringify(f)})`);
  const gd = await ground();
  check(gd.image === 'none' && gd.art === 0, `dark: a plain ground, no backdrop art (${JSON.stringify(gd)})`);
  check((await evaluate(`document.querySelectorAll('.cx-split').length`)) === 4 && (await evaluate(`document.querySelectorAll('.cx-bridge').length`)) === 5, 'four split sockets and five bridges');

  await click('#cortex-grant-10000', 'grant Insight');
  await click('#cortex-grant-10000', 'grant more Insight');

  // 052's regressions, dark only: Space buys and doesn't pause; a refused removal says why.
  await evaluate(`document.querySelector(${q(node('vigilance-v1'))}).focus()`);
  await send('Input.dispatchKeyEvent', { type: 'keyDown', key: ' ', code: 'Space', windowsVirtualKeyCode: 32, text: ' ' });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', key: ' ', code: 'Space', windowsVirtualKeyCode: 32 });
  await sleep(100);
  check((await rankOf('vigilance-v1')) === 1 && (await attr('#takeover-pause', 'aria-pressed')) === 'true', 'Space on a node buys it and leaves the game paused');
  await evaluate(`document.activeElement.blur()`);
  await buy('vigilance-i1');
  await right(node('vigilance-v1'));
  check((await rankOf('vigilance-v1')) === 1 && (await tip()).includes('Can’t remove: Thin Profile depends on it'), 'right-click on the root is refused, with the reason');
  await click('#cortex-respec', 'respec');

  await pass('dark');

  // Light: the same view follows the app's theme.
  await escape();
  check(!(await exists(TAKEOVER)), 'Esc closes it');
  await debugMenu();
  await click('[data-theme-pick="light"]', 'light');
  await click(BUTTON, 'reopen');
  const gl = await ground();
  check(gl.image === 'none' && gl.art === 0 && gl.color !== gd.color, `light: a plain ground of its own (${JSON.stringify(gl)})`);
  const ink = await evaluate(`getComputedStyle(document.querySelector('[data-sector-name="serenity"]')).fill`);
  check(!ink.includes('138, 180, 248'), `light: sector colors re-tuned (${ink})`);
  await pass('light');

  // A smaller window still fits it without scrolling.
  await send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 720, deviceScaleFactor: 1, mobile: false });
  await sleep(300);
  const small = await fits();
  check(!small.body && !small.side && small.inside, `at 1280 × 720 too (${JSON.stringify(small)})`);
  await send('Emulation.setDeviceMetricsOverride', { width: W, height: H, deviceScaleFactor: 1, mobile: false });
  await sleep(200);
  await click(BUTTON, 'close');
  await click('#cortex-mock-toggle', 'the debug toggle off');
  check(!(await exists(BUTTON)), 'the button goes with it');
  check((await actions()) === before, `the game saw no actions (${before} before and after)`);
  check(b.errors.length === 0, `no page errors${b.errors.length ? `: ${b.errors.join('; ')}` : ''}`);
} finally {
  await b.close();
}
