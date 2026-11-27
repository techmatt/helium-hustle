// 122 probe 5: the right column folded (112): Side Quests' folded header with its peek
// (the closest-to-done quest's progress), Events and Recent Activity folded; each fold
// head's text against its box, at a width; then a reload (the folds as left).
// Run from the repo root: node experiments/results/122_bug_hunt/probe5.mjs <url> <width>
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { capture, driver, launch, sleep } from '../../../web/scripts/cdp.mjs';

const [url = 'http://localhost:49373/', width = '1280'] = process.argv.slice(2);
const out = 'out/122_bug_hunt/probe5';
mkdirSync(out, { recursive: true });
const b = await launch({ width: Number(width), height: 900, profileRoot: resolve('out/122_bug_hunt/profiles') });
const d = driver(b, { url });
const heads = () =>
  b.evaluate(`[...document.querySelectorAll('.col-right .fold-head')].map((h) => { const r = h.getBoundingClientRect(); return h.textContent.trim() + ' [' + Math.round(r.width) + 'w ' + Math.round(r.height) + 'h' + (h.scrollWidth > h.clientWidth + 1 ? ' OVERFLOW ' + h.scrollWidth : '') + ']'; })`);
try {
  await d.load();
  await d.debugMenu();
  await d.speed(0);
  await d.setValue('#scenario', 'late_unlocks');
  await d.click('#start-scenario');
  await d.readStory();
  await d.panel('buildings');
  console.log('open:', JSON.stringify(await heads(), null, 1));
  for (const t of ['side-quests', 'events-list', 'events-activity']) {
    if (await d.exists(`[data-testid="${t}"] .fold-head`)) await d.click(`[data-testid="${t}"] .fold-head`);
  }
  await sleep(200);
  console.log('folded:', JSON.stringify(await heads(), null, 1));
  await capture(b, `${out}/${width}-folded.png`);
  await d.load();
  await sleep(300);
  console.log('after reload:', JSON.stringify(await heads(), null, 1));
  console.log('errors', JSON.stringify(b.errors));
} finally {
  b.close();
}
