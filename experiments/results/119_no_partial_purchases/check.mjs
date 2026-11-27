// 119's browser checks against a `vite preview` build (HH_URL). Screenshots to SHOTS.
//   1. No partial purchase: Buy Titanium on 10 credits (it costs 20) does nothing that day,
//      its line's dot is the failed (red) one and names credits; nothing is paid or made.
//   2. A buy that would overflow its cap doesn't run: titanium at 19.5 of 20, credits plenty.
//   3. Loading pads still tops up partially: 2 He-3 on hand, a pad loads the 2.
//   4. The Ideologies view shows each axis's picked emblem (amendment #3518).
//   HH_URL=http://localhost:<port>/ node experiments/results/119_no_partial_purchases/check.mjs
import { copyFileSync, mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { capture, driver, launch, pageUrl, sleep } from '../../../web/scripts/cdp.mjs';

const url = pageUrl('check');
const OUT = 'out/119_no_partial_purchases';
const SHOTS = resolve(process.env.SHOTS ?? `${OUT}/shots`);
const DRIVE = 'C:/Code/gdrive-sync/helium-hustle/screenshots';
mkdirSync(SHOTS, { recursive: true });
const b = await launch({ width: 1600, height: 900, profileRoot: resolve(`${OUT}/profile`) });
const { evaluate } = b;
const d = driver(b, { url, settle: 500 });
const { waitFor, exists, click, setValue, load, debugMenu, cheats, grant, readStory, panel, speed, runOnly } = d;
const results = [];
const check = (ok, what) => {
  results.push(`${ok ? 'PASS' : 'FAIL'} ${what}`);
  console.log(`${ok ? 'PASS' : 'FAIL'} ${what}`);
};
const shot = async (name) => {
  const file = resolve(SHOTS, `119_${name}.png`);
  await capture(b, file);
  try {
    copyFileSync(file, `${DRIVE}/119_${name}.png`);
  } catch (e) {
    console.log(`no Drive copy: ${e.message}`);
  }
};
const scenario = async (name) => {
  await debugMenu();
  await setValue('#scenario', name);
  await click('#start-scenario');
  await sleep(800);
  await readStory();
};
const amount = (res) => evaluate(`Number(document.querySelector('tr[data-res="${res}"]')?.dataset.amount ?? 'NaN')`);
const day = () => evaluate(`Number(document.getElementById('day').textContent.replace(/\\D/g, ''))`);
/** Plays one day or a little more at 1x, then pauses: the days played. */
const oneDay = async () => {
  const d0 = await day();
  await speed(1);
  await waitFor(`Number(document.getElementById('day').textContent.replace(/\\D/g, '')) > ${d0}`, 'a day', 5000);
  await speed(0);
  await sleep(300);
  await readStory();
  return (await day()) - d0;
};
/** Opens the Programs view (its gutter has the line dots), in Blocks mode. */
const programsView = async () => {
  await click('.nav-tile[data-nav="programs"]');
  await waitFor(`!!document.querySelector('[data-testid="mode-text"]')`, 'the Programs view', 3000);
  if (!(await exists('[data-testid="block-editor"]'))) await click('[data-testid="mode-blocks"]');
  await sleep(300);
};
const closeView = async () => {
  await b.send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
  await b.send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
  await sleep(300);
};
/** Every gutter dot on program line 0: its status and title. */
const dot0 = () =>
  evaluate(`[...document.querySelectorAll('[data-line="0"][data-line-status]')].map((e) => ({ status: e.dataset.lineStatus, title: e.getAttribute('title') }))`);
/** Line 0's row title (the tooltip naming what it did and why). */
const row0 = () => evaluate(`[...document.querySelectorAll('li[data-line="0"]')].map((e) => e.getAttribute('title'))`);

await load(url);
await readStory();

// ---- 1. Buy Titanium short of credits ----
// at_cap starts on 50 credits: two buys at 20 leave 10, and the third day's buy is short.
await scenario('at_cap');
await speed(0);
await runOnly('buy_titanium', ['?']);
await oneDay();
await oneDay();
const t0 = await amount('titanium');
const c0 = await amount('credits');
const played = await oneDay();
const c1 = await amount('credits');
const t1 = await amount('titanium');
// The gutter's dots are in the Programs view.
await programsView();
const dots = await dot0();
await closeView();
const rows = await row0();
console.log('short:', JSON.stringify({ played, c0, c1, t0, t1, dots, rows }));
check(played === 1 && c0 === 10 && c1 === 10 && t0 === 2 && t1 === 2, `on 10 credits Buy Titanium (20) does nothing: credits ${c0} -> ${c1}, titanium ${t0} -> ${t1}`);
check(
  dots.length > 0 && dots.every((x) => x.status === 'failed' && /credits/i.test(x.title ?? '')),
  `its dot is the failed (red) one, naming credits: ${JSON.stringify(dots)}`,
);
await shot('short_red_dot');

// ---- 2. A buy that would overflow the titanium cap ----
await cheats();
await grant('credits', 1000);
await grant('titanium', 19.5 - (await amount('titanium')));
const c2a = await amount('credits');
await oneDay();
await programsView();
const c2 = await amount('credits');
const t2 = await amount('titanium');
const dots2 = await dot0();
await closeView();
console.log('overflow:', JSON.stringify({ c2a, c2, t2, dots2 }));
check(c2 === c2a && t2 === 19.5, `at 19.5 of 20 titanium Buy Titanium doesn't run: credits ${c2a} -> ${c2}, titanium ${t2}`);
check(
  dots2.length > 0 && dots2.every((x) => x.status === 'gray' && /cap/.test(x.title ?? '')),
  `its dot is gray, saying the output would pass its cap: ${JSON.stringify(dots2)}`,
);
await shot('overflow_skipped');

// ---- 3. Loading tops a pad up partially ----
// pad_loading holds no titanium: the pad carries Titanium, 2 granted, less than a load.
await scenario('pad_loading');
await speed(0);
await panel('launch_pads');
await evaluate(`[...document.querySelectorAll('[data-panel="launch_pads"] select')].forEach((s) => { if ([...s.options].some((o) => o.value === 'titanium')) { s.value = 'titanium'; s.dispatchEvent(new Event('change', { bubbles: true })); } })`);
await runOnly('load_pads', ['?']);
await cheats();
await grant('titanium', 2);
const ti3a = await amount('titanium');
await oneDay();
const ti3 = await amount('titanium');
await programsView();
const dots3 = await dot0();
await closeView();
await panel('launch_pads');
const cargo = await evaluate(`document.querySelector('[data-panel="launch_pads"]')?.textContent ?? ''`);
console.log('load:', JSON.stringify({ ti3a, ti3, dots3, cargo: cargo.slice(0, 600) }));
check(ti3a === 2 && ti3 === 0 && dots3.length > 0 && dots3.every((x) => x.status === 'green'), `Load Launch Pads on 2 titanium ran and loaded the 2: titanium ${ti3a} -> ${ti3}, ${JSON.stringify(dots3)}`);
await shot('partial_load');

// ---- 4. Ideology emblems ----
await scenario('everything');
await speed(0);
await panel('ideologies');
await waitFor(`[...document.querySelectorAll('.axis-emblem img')].length === 3 && [...document.querySelectorAll('.axis-emblem img')].every((i) => i.complete && i.naturalWidth > 0)`, 'the emblems to load', 5000).catch(() => {});
const emblems = await evaluate(`[...document.querySelectorAll('[data-testid^="axis-emblem-"]')].map((e) => { const i = e.querySelector('img'); return { axis: e.dataset.testid, src: i?.getAttribute('src') ?? null, loaded: !!i && i.complete && i.naturalWidth > 0, w: e.getBoundingClientRect().width, h: e.getBoundingClientRect().height }; })`);
console.log('emblems:', JSON.stringify(emblems));
check(
  emblems.length === 3 && emblems.every((e) => e.loaded && e.src.includes(e.axis.replace('axis-emblem-', ''))),
  `each axis shows its emblem: ${emblems.map((e) => `${e.axis}=${e.loaded}`).join(', ')}`,
);
await shot('ideology_emblems');

console.log('\n' + results.join('\n'));
await b.close();
process.exit(results.some((r) => r.startsWith('FAIL')) ? 1 : 0);
