// 041's done-when read: from a fresh career in the browser, the debug menu's autopilot
// (Everything, the default boost and drivers) until every milestone and main-quest step is
// met, timed in real seconds; then the replay check. Drives headless Edge or Chrome over
// the DevTools protocol (web/scripts/cdp.mjs) against a served static build.
//   HH_URL    the page, required (a `vite preview`, web/README.md)
//   SEEDS     comma-separated seeds (default 3088,3089,3090)
//   OUT       the results file (default out/041_debug_accelerators/timing.json)
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { READY, driver, launch, pageUrl } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const repo = resolve(here, '../../..');
const url = pageUrl('timing', 'node experiments/results/041_debug_accelerators/timing.mjs');
const seeds = (process.env.SEEDS ?? '3088,3089,3090').split(',');
const OUT = resolve(process.env.OUT ?? resolve(repo, 'out/041_debug_accelerators/timing.json'));
mkdirSync(dirname(OUT), { recursive: true });
const b = await launch({ width: 1280, height: 800, profileRoot: resolve(repo, 'out/041_debug_accelerators/profile') });
const { evaluate } = b;
const page = driver(b, { url });
const { waitFor, click, setValue, load, cheats } = page;
const rows = [];
await load();
await evaluate('localStorage.clear()');
await load();
for (const seed of seeds) {
  await cheats();
  await setValue('#seed', seed);
  await click('#new-game');
  await waitFor(READY, 'a new game');
  await cheats();
  const t0 = Date.now();
  await click('#accel-end');
  await waitFor(`document.getElementById('accel-status')?.dataset.running === '1'`, 'the autopilot to start', 5000);
  await waitFor(`document.getElementById('accel-status')?.dataset.running === '0'`, 'the autopilot to finish', 15 * 60 * 1000);
  const secs = (Date.now() - t0) / 1000;
  const status = await evaluate(`document.getElementById('accel-status').textContent`);
  await click('#check-replay');
  await waitFor(`/Replay/.test(document.querySelector('.notice')?.textContent ?? document.body.textContent)`, 'the replay check', 120000);
  const replay = await evaluate(`(document.body.textContent.match(/Replay (ok|MISMATCH|failed)[^·]{0,40}/) ?? [''])[0]`);
  const marked = await evaluate(`document.getElementById('accel-boost-now').textContent`);
  const row = { seed, secs, status, replay, marked };
  console.log(JSON.stringify(row));
  rows.push(row);
}
writeFileSync(OUT, JSON.stringify(rows, null, 2) + '\n');
await b.close();
