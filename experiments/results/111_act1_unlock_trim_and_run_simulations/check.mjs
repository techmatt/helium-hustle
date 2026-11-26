// 111's check in the running game (vite preview at HH_URL). A new game's Research tile opens
// with its unread dot fully inside its own tile (right edge, centered on the label, padded from
// the border; parked P1). Scenario late_unlocks (1.1-1.4 done, 1.5 A Voice active): Story lists
// 1.1-1.3 then Trade as 1.4 with A Voice as 1.5; the research view shows Regolith Processing and
// Liquid Cooling, never Process Telemetry or Accelerated Scheduling; Run Simulations is offered
// once Liquid Cooling is bought. From the repo root, a static build served:
//   HH_URL=http://localhost:<port>/ node experiments/results/111_act1_unlock_trim_and_run_simulations/check.mjs <shots dir>
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { launch, driver, capture, sleep } from '../../../web/scripts/cdp.mjs';

const url = process.env.HH_URL;
const shots = resolve(process.argv[2] ?? 'out/111_act1_unlock_trim_and_run_simulations/shots');
mkdirSync(shots, { recursive: true });
const b = await launch({ width: 1280, height: 800, emulate: true, profileRoot: resolve('out/111_act1_unlock_trim_and_run_simulations/profile') });
const { evaluate } = b;
const d = driver(b, { url, settle: 700 });
const { exists, click, setValue, speed, debugMenu, cheats, grant, panel } = d;
let fails = 0;
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) fails++;
};
const shot = async (n, sel) => {
  const r = await evaluate(`(() => { const r = document.querySelector(${JSON.stringify(sel)}).getBoundingClientRect(); return { x: r.x, y: r.y, w: r.width, h: Math.min(r.height, 760) }; })()`);
  await capture(b, resolve(shots, `111_${n}.png`), { clip: { x: r.x, y: r.y, width: r.w, height: r.h, scale: 2 } });
};
const dismiss = async () => {
  for (let i = 0; i < 60 && (await exists('[data-testid="story-next"]')); i++) await click('[data-testid="story-next"]');
};

// A new game: two Solar Panels open Research, with its unread dot.
await d.load();
await dismiss();
await speed(0);
await cheats();
await grant('credits', 1000);
await grant('titanium', 100);
await speed(1);
await sleep(1500);
await panel('buildings');
while (Number(await evaluate(`document.querySelector('.card.building[data-id="solar_panel"]').dataset.owned`)) < 2) {
  await d.waitFor(`!!document.querySelector('.card.building[data-id="solar_panel"].can-buy')`, 'a buyable Solar Panel', 8000);
  await click('.card.building[data-id="solar_panel"].can-buy .card-text');
}
await d.waitFor(`!!document.querySelector('.nav-tile[data-nav="research"] .dot-new')`, "Research's unread dot", 15000);
await speed(0);
await dismiss();
await sleep(700);
const dot = await evaluate(`(() => {
  const t = document.querySelector('.nav-tile[data-nav="research"]');
  const tr = t.getBoundingClientRect(), dr = t.querySelector('.dot-new').getBoundingClientRect();
  const lr = t.querySelector(':scope > span:not(.dot)').getBoundingClientRect();
  return { inside: dr.left >= tr.left && dr.right <= tr.right && dr.top >= tr.top && dr.bottom <= tr.bottom,
    pad: Math.round((tr.right - dr.right) * 10) / 10, mid: Math.round(((dr.top + dr.bottom) / 2 - (lr.top + lr.bottom) / 2) * 10) / 10,
    clear: lr.right <= dr.left + 0.5 };
})()`);
console.log('research dot', JSON.stringify(dot));
check(dot.inside && dot.pad >= 3, `the unread dot sits inside Research's tile, ${dot.pad} px from its right border`);
check(Math.abs(dot.mid) <= 1.5, `centered on the label row (off by ${dot.mid} px)`);
check(dot.clear, 'the label stops short of the dot');
await shot('nav_dot', '.nav-grid');

// Scenario late_unlocks: 1.1-1.4 done, A Voice active.
await debugMenu();
await speed(0);
await setValue('#scenario', 'late_unlocks');
await click('#start-scenario');
await sleep(500);
await dismiss();
await speed(0);
await panel('story');
await sleep(300);
const st = await evaluate(`(() => {
  const p = document.querySelector('[data-panel="story"]');
  const title = (c) => (c.querySelector('[data-testid="story-beat-toggle"]') ?? c).textContent.match(/\\d\\.\\d — [A-Za-z ]+/)?.[0].trim() ?? c.textContent.slice(0, 40);
  return { done: [...p.querySelectorAll('[data-testid="story-beat-done"]')].map(title), active: [...p.querySelectorAll('[data-testid="story-beat-active"]')].map(title) };
})()`);
console.log('story', JSON.stringify(st));
check(st.done.join(' | ') === '1.1 — Online | 1.2 — First Shipment | 1.3 — Full Load | 1.4 — Trade', `Story: ${st.done.join(' | ')}`);
check(st.active[0]?.startsWith('1.5 — A Voice'), `active: ${st.active.join()}`);
await shot('story', '[data-panel="story"]');
await panel('research');
await sleep(300);
const research = await evaluate(`[...document.querySelectorAll('[data-testid^="research-"]')].map((e) => e.dataset.testid.slice(9))`);
console.log('research', JSON.stringify(research));
check(research.includes('regolith_processing') && research.includes('liquid_cooling'), 'Regolith Processing and Liquid Cooling on offer');
check(!research.includes('process_telemetry') && !research.includes('accelerated_scheduling'), 'no Process Telemetry or Accelerated Scheduling');
await debugMenu();
await grant('science', 200);
await panel('research');
await click('[data-testid="research-liquid_cooling"]');
await sleep(300);
await panel('commands');
await sleep(300);
check(await exists('[data-panel="commands"] [data-id="run_simulations"]'), 'Run Simulations offered once Liquid Cooling is bought');
await shot('research_commands', '[data-panel="commands"]');

await b.close();
console.log(fails ? `${fails} failed` : 'all ok');
process.exit(fails ? 1 : 0);
