// 114's direct check: a new game played by the debug autopilot's "Next quest step" until
// the Projects tile shows (A Voice's start, run 1); the Projects panel's cards and the
// program tabs (count, light, hover) read and captured.
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
// Run from the repo root: HH_URL=<a vite preview> node experiments/results/114_a_voice_projects_and_program_lights/check.mjs
import { launch, driver, capture, sleep } from '../../../web/scripts/cdp.mjs';

const out = resolve('out/114_a_voice_projects_and_program_lights/shots');
mkdirSync(out, { recursive: true });
const url = process.env.HH_URL;
const b = await launch({ width: 1600, height: 1000, profileRoot: resolve('out/114_a_voice_projects_and_program_lights/profiles') });
const d = driver(b, { url });
const { exists, click, waitFor, panel } = d;
const { evaluate } = b;
const log = [];
const say = (s) => { console.log(s); log.push(s); };
try {
  await d.load();
  await d.autoStory();
  const quest = () => evaluate(`document.querySelector('.quest-pin')?.textContent.slice(0, 80) ?? ''`);
  for (let step = 0; step < 14 && !(await exists('.nav-tile[data-nav="projects"]')); step++) {
    await d.cheats();
    say(`step ${step + 1} from: ${await quest()}`);
    await click('#accel-quest', 'next quest step');
    await waitFor(`document.getElementById('accel-status')?.dataset.running === '0'`, 'the autopilot step', 120000);
    if (await exists('[data-testid="continue"]')) { say('a retirement summary showed'); await click('[data-testid="continue"]'); }
  }
  await sleep(500);
  say(`projects tile: ${await exists('.nav-tile[data-nav="projects"]')}; quest: ${await quest()}`);
  say(`run: ${await evaluate(`document.getElementById('day')?.textContent ?? ''`)}`);
  await panel('projects');
  await sleep(300);
  const cards = await evaluate(`[...document.querySelectorAll('[data-panel="projects"] .card')].map((c) => c.textContent.replace(/\\s+/g, ' ').slice(0, 160))`);
  for (const c of cards) say(`card: ${c}`);
  await capture(b, `${out}/114_projects_at_a_voice.png`);
  const tabs = await evaluate(`[...document.querySelectorAll('.prog-tabs button')].map((t) => ({ text: t.textContent, light: t.querySelector('.prog-light')?.dataset.light ?? null, title: t.title, cls: t.className }))`);
  for (const t of tabs) say(`tab: ${JSON.stringify(t)}`);
  const rect = await evaluate(`(() => { const r = document.querySelector('.programs').getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: Math.min(r.height, 260) }; })()`);
  await capture(b, `${out}/114_program_tabs.png`, { clip: { ...rect, scale: 2 } });
} catch (e) {
  say(`error: ${e}`);
} finally {
  writeFileSync(`${out}/check.txt`, log.join('\n') + '\n');
  await b.close();
}
