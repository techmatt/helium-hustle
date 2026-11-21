// 042's done-when read: a fresh career in the browser on the accelerators, the debug menu's
// "Next quest step" pressed until the ending shows, opening each later-act view as its tile
// appears (a screenshot each). Drives headless Edge or Chrome over the DevTools protocol
// (web/scripts/cdp.mjs) against a served static build (`vite preview`).
//   HH_URL  the page, required
//   SEED    the career's seed (default 3092)
//   OUT     the results folder (default out/042_draft_whole_arc/arc)
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { READY, capture, driver, launch, pageUrl } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const repo = resolve(here, '../../..');
const url = pageUrl('arc', 'node experiments/results/042_draft_whole_arc/arc.mjs');
const seed = process.env.SEED ?? '3092';
const OUT = resolve(process.env.OUT ?? resolve(repo, 'out/042_draft_whole_arc/arc'));
mkdirSync(OUT, { recursive: true });
const VIEWS = ['timeline', 'earth_lab', 'meta', 'vault', 'dyson'];
const b = await launch({ width: 1280, height: 860, profileRoot: resolve(repo, 'out/042_draft_whole_arc/profile') });
const { evaluate } = b;
const page = driver(b, { url });
const { waitFor, click, setValue, load, cheats, exists } = page;
await load();
await evaluate('localStorage.clear()');
await load();
await cheats();
await setValue('#seed', seed);
await click('#new-game');
await waitFor(READY, 'a new game');
const t0 = Date.now();
const rows = [];
const seen = new Set();
let steps = 0;
let ended = false;
while (!ended && Date.now() - t0 < 25 * 60 * 1000 && steps < 80) {
  await cheats();
  if (await exists('#accel-quest[disabled]')) break;
  const before = await evaluate(`document.getElementById('accel-status')?.textContent ?? ''`);
  await click('#accel-quest');
  // A short step can start and end between two polls: wait for the status to change.
  await waitFor(
    `document.getElementById('accel-status')?.dataset.running === '1' || document.getElementById('accel-status')?.textContent !== ${JSON.stringify(before)}`,
    'the autopilot to start',
    10000,
  );
  await waitFor(`document.getElementById('accel-status')?.dataset.running === '0'`, 'the autopilot to finish', 15 * 60 * 1000);
  steps++;
  const status = await evaluate(`document.getElementById('accel-status').textContent`);
  const day = await evaluate(`document.getElementById('day')?.textContent ?? ''`);
  const strip = await evaluate(`document.querySelector('[data-testid="calendar-strip"]')?.textContent ?? ''`);
  ended = await exists('[data-testid="ending"]');
  const row = { step: steps, secs: (Date.now() - t0) / 1000, status, day, strip, ended };
  console.log(JSON.stringify(row));
  rows.push(row);
  if (ended) await capture(b, resolve(OUT, 'ending.png'));
  if (ended) await click('[data-testid="ending"] button', 'close the ending');
  for (const id of VIEWS) {
    if (seen.has(id) || !(await exists(`.nav-tile[data-nav="${id}"]`))) continue;
    await click(`.nav-tile[data-nav="${id}"]`, `panel ${id}`);
    await waitFor(`!!document.querySelector('[data-panel="${id}"]')`, `the ${id} panel`, 3000);
    await capture(b, resolve(OUT, `${id}.png`));
    seen.add(id);
    rows.push({ view: id, step: steps });
  }
  if (/not reached/i.test(status)) break;
}
await click('#check-replay').catch(() => {});
const replay = await waitFor(`/Replay/.test(document.body.textContent)`, 'the replay check', 180000)
  .then(() => evaluate(`(document.body.textContent.match(/Replay (ok|MISMATCH|failed)[^·]{0,40}/) ?? [''])[0]`))
  .catch(() => 'no replay result');
const result = { seed, secs: (Date.now() - t0) / 1000, steps, ended, views: [...seen], replay, rows };
writeFileSync(resolve(OUT, 'arc.json'), JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ seed, secs: result.secs, steps, ended, views: result.views, replay }));
await b.close();
