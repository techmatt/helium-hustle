// 075's screenshots: Story after the debug autopilot plays a new game through 1.6 (its
// step stops at 1.7), at the top (the Act 1 band) and at the active beat, in both themes,
// and the Achievements panel. Run from the repo root against a `vite preview` of a
// static build: node experiments/results/075_story_view_and_achievements_tile/shots.mjs <url>
import { mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = process.argv[2] ?? 'http://localhost:5173/';
const out = resolve(here, 'shots');
mkdirSync(out, { recursive: true });
const b = await launch({ width: 1600, height: 900, profileRoot: resolve(here, '../../../out/075_story_view_and_achievements_tile/shots-profile') });
const { evaluate } = b;
const { waitFor, exists, click, load, speed, panel, cheats, autoStory } = driver(b, { url, settle: 700 });
const shot = (name) => capture(b, resolve(out, `${name}.png`));
const theme = async (t) => {
  await panel('options');
  await click(`#theme [data-theme-pick="${t}"]`);
};
try {
  await load(url);
  await cheats();
  await speed(0);
  await click('#new-game');
  await sleep(500);
  await autoStory();
  const beat = () => evaluate(`Number(document.querySelector('[data-testid="quest"]')?.textContent.match(/^1\\.(\\d+)/)?.[1] ?? 0)`);
  for (let step = 0; step < 8 && (await beat()) < 6; step++) {
    await cheats();
    await click('#accel-quest');
    await waitFor(`document.getElementById('accel-status')?.dataset.running === '0'`, 'step', 90000);
    if (await exists('[data-testid="continue"]')) await click('[data-testid="continue"]');
  }
  for (const t of ['light', 'dark']) {
    await theme(t);
    await panel('story');
    await sleep(300);
    await shot(`story-active-${t}`);
    await evaluate(`document.querySelector('.col-mid').scrollTop = 0`);
    await sleep(200);
    await shot(`story-top-${t}`);
    await panel('achievements');
    await sleep(200);
    await shot(`achievements-${t}`);
  }
  await theme('light');
} finally {
  b.close();
}
