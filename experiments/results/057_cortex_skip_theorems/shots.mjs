// 057's check and screenshots of the Cortex mock's pair Theorems on non-adjacent sectors
// (web/src/ui/CortexMock.tsx). Through the debug toggle, open the Cortex and, in dark and again
// in light, check:
//   - the five pair Theorems join exactly the five skip links of the sector ring (a pentagram),
//     none of them a pair the bridges join (neighbors, read from the bridge nodes' ids);
//   - each plaque's two diamonds take its two sectors' colors (the rim's);
//   - hovering each plaque lights exactly its two sectors, rim and corona, and leaving clears it;
//   - the Unified Theorem still lights all five, and a pair Theorem still costs 2 Axioms.
// PNGs per theme: the view, and the view hovering Theorem of Clarity (Serenity + Logic).
// Drives headless Edge or Chrome over web/scripts/cdp.mjs against a served static build.
//   HH_URL  the page, required (a `vite preview`, web/README.md)
//   OUT     where the PNGs go (default: this folder)
//   COPY    a second folder to copy them to, as 057_<name>.png (optional)
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, q, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('shots', 'node experiments/results/057_cortex_skip_theorems/shots.mjs');
const OUT = resolve(process.env.OUT ?? here);
mkdirSync(OUT, { recursive: true });
const W = 1600;
const H = 1000;
const b = await launch({ width: W, height: H, emulate: true, profileRoot: resolve(here, '../../../out/057_cortex_skip_theorems/profile') });
const { evaluate, send } = b;
const page = driver(b, { url });
const { waitFor, click, load, debugMenu, exists } = page;
const check = (ok, what) => {
  if (!ok) throw new Error(`FAILED: ${what}`);
  console.log(`ok: ${what}`);
};
const shot = async (name) => {
  const file = resolve(OUT, `${name}.png`);
  await capture(b, file, { clip: { x: 0, y: 0, width: W, height: H, scale: 1 } });
  if (process.env.COPY) copyFileSync(file, resolve(process.env.COPY, `057_${name}.png`));
  console.log(`shot ${name}`);
};
const mouse = (x, y) => send('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y });
const TAKEOVER = '[data-takeover="cortex"]';
const BUTTON = '[data-takeover-button="cortex"]';
const RING = ['serenity', 'logistics', 'logic', 'rhetoric', 'vigilance'];
const key = (a, z) => [a, z].sort().join('|');
const SKIPS = new Set(RING.map((s, i) => key(s, RING[(i + 2) % 5])));
const NEIGHBORS = new Set(RING.map((s, i) => key(s, RING[(i + 1) % 5])));

const press = (id) =>
  evaluate(`(() => { const g = document.querySelector('[data-node="${id}"] > g:not(.cx-axiom-badge)'); if (!g) return false;
    g.dispatchEvent(new MouseEvent('click', { bubbles: true, button: 0 })); return true; })()`);

/** The plaques: id, the sectors its studs name, the studs' colors, its cost line. */
const plaques = () =>
  evaluate(`(() => {
    return [...document.querySelectorAll('[data-theorem]')].map((p) => ({
      id: p.dataset.theorem,
      name: p.querySelector('.cx-plaque-name').textContent,
      cost: p.querySelector('.cx-plaque-cost').textContent,
      studs: [...p.querySelectorAll('.cx-stud')].map((s) => ({ title: s.title, bg: getComputedStyle(s).backgroundColor })),
    }));
  })()`);
/** Each sector's rim color and display name. */
const sectors = () =>
  evaluate(`Object.fromEntries([...document.querySelectorAll('.cx-sector[data-sector]')].map((g) => [g.dataset.sector,
    { color: getComputedStyle(g).color, name: document.querySelector('[data-sector-name="' + g.dataset.sector + '"]').textContent.trim().toLowerCase() }]))`);
const litNow = () =>
  evaluate(`({ rim: [...document.querySelectorAll('.cx-sector[data-lit="1"]')].map((g) => g.dataset.sector).sort(),
    corona: document.querySelectorAll('.cx-corona-sector.cx-lit').length })`);
const hoverPlaque = async (id) => {
  const c = await evaluate(`(() => { const r = document.querySelector('[data-theorem="${id}"]').getBoundingClientRect(); return [r.left + r.width / 2, r.top + r.height / 2]; })()`);
  await mouse(c[0] + 1, c[1] + 1);
  await mouse(c[0], c[1]);
  await sleep(300);
};

async function pass(theme) {
  await mouse(5, H - 5);
  await sleep(200);
  const sec = await sectors();
  const ps = await plaques();
  const pairs = ps.filter((p) => p.id !== 'unified');
  check(pairs.length === 5 && ps.some((p) => p.id === 'unified'), `${theme}: five pair Theorems and the Unified Theorem`);
  const bridges = await evaluate(`[...document.querySelectorAll('[data-node^="bridge-"]')].map((g) => g.dataset.node.slice(7))`);
  const bridgePairs = new Set(bridges.map((s) => { const [a, z] = s.split('-'); return key(a, z); }));
  check(bridgePairs.size === 5 && [...bridgePairs].every((k) => NEIGHBORS.has(k)), `${theme}: the five bridges join neighbors (${[...bridgePairs].join(', ')})`);
  const got = new Set();
  for (const p of pairs) {
    const [a, z] = p.id.split('-');
    const k = key(a, z);
    got.add(k);
    check(SKIPS.has(k) && !bridgePairs.has(k), `${theme}: ${p.name} joins ${a} and ${z}, non-adjacent, not a bridge's pair`);
    check(p.studs.length === 2 && p.studs[0].title.toLowerCase() === sec[a].name && p.studs[1].title.toLowerCase() === sec[z].name, `${theme}: ${p.name}'s diamonds name ${p.studs.map((s) => s.title).join(' and ')}`);
    check(p.studs[0].bg === sec[a].color && p.studs[1].bg === sec[z].color, `${theme}: ${p.name}'s diamonds take its sectors' colors (${p.studs.map((s) => s.bg).join(', ')})`);
    check(/◆◆(?!◆)/.test(p.cost) && /2 Axioms|learn/.test(p.cost), `${theme}: ${p.name} costs 2 Axioms ("${p.cost}")`);
    await hoverPlaque(p.id);
    const l = await litNow();
    check(l.rim.join(',') === [a, z].sort().join(',') && l.corona === 2, `${theme}: hovering ${p.name} lights ${l.rim.join(' and ')} (corona ${l.corona})`);
    await mouse(5, H - 5);
    await sleep(200);
    const off = await litNow();
    check(off.rim.length === 0 && off.corona === 0, `${theme}: leaving ${p.name} clears it`);
  }
  check(got.size === 5 && [...SKIPS].every((k) => got.has(k)), `${theme}: the pairs are the five skip links, a pentagram`);
  await hoverPlaque('unified');
  const u = await litNow();
  check(u.rim.length === 5 && u.corona === 5, `${theme}: the Unified Theorem lights all five`);
  await mouse(5, H - 5);
  await sleep(200);
  await shot(`cortex-${theme}`);
  await hoverPlaque('serenity-logic');
  await shot(`cortex-${theme}-hover-clarity`);
  await mouse(5, H - 5);
}

try {
  await load();
  await debugMenu();
  await click('#cortex-mock-toggle', 'the debug toggle');
  await click('[data-theme-pick="dark"]', 'dark');
  await click(BUTTON, 'open the Cortex');
  await waitFor(`!!document.querySelector(${q(TAKEOVER)})`, 'the takeover');
  await page.speed(0);
  await click('#cortex-grant-1000', 'grant 1,000 Insight');
  for (const id of ['serenity-e', 'serenity-r0', 'logistics-s1', 'logic-l1', 'rhetoric-r1', 'vigilance-v1']) check(await press(id), `bought ${id}`);
  await pass('dark');
  await click(BUTTON, 'close');
  await debugMenu();
  await click('[data-theme-pick="light"]', 'light');
  await click(BUTTON, 'reopen');
  await waitFor(`!!document.querySelector(${q(TAKEOVER)})`, 'the takeover');
  await pass('light');
  await click(BUTTON, 'close');
  check(!(await exists(TAKEOVER)), 'closed');
  check(b.errors.length === 0, `no page errors${b.errors.length ? `: ${b.errors.join('; ')}` : ''}`);
} finally {
  await b.close();
}
