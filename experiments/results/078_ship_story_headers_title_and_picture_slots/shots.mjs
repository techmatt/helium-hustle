// `node shots.mjs <url>`: 078's screenshots of the shipped Story art, title and quest-picture
// slots, in the running game (a `vite preview` of a static build at <url>), at 1280x800 light
// and 1920x1080 dark:
//   intro      a new game: 1.1's story modal with its picture, the left column's foot showing
//              1.1's picture, the split-glow title, no Earth globe (whole window)
//   story11    Story on 1.1: the Act 1 band and the active beat with 1.1's thumbnail
//   act1       1.1 finished (the autopilot's first quest step): Story's Act 1 band and 1.1's
//              thumbnail on its finished beat; the left column back on the horizon (1.2 has
//              no picture) (whole window)
//   acts       scenario story_acts (Acts 1-3 done): Story scrolled to each act band (middle
//              column)
//   title      the left column's top at 2x pixels
// Shots go to this folder's shots/ (PNG).
import { mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, sleep } from '../../../web/scripts/cdp.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..');
const OUT = path.join(HERE, 'shots');
mkdirSync(OUT, { recursive: true });
const url = process.argv[2];
if (!url) throw new Error('shots: give the preview url');

const COMBOS = [
  { w: 1280, h: 800, theme: 'light' },
  { w: 1920, h: 1080, theme: 'dark' },
];
const b = await launch({ width: 1280, height: 800, emulate: true, profileRoot: path.join(REPO, 'out', '078_ship_story_headers_title_and_picture_slots', 'profile') });
const { evaluate, send } = b;
const { waitFor, exists, click, load, speed, panel, cheats, debugMenu, autoStory } = driver(b, { url, settle: 700 });
const rect = (sel) => evaluate(`(() => { const r = document.querySelector('${sel}').getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height }; })()`);
const shot = async (name, clip, scale = 1) => {
  await capture(b, path.join(OUT, `${name}.png`), { clip: { ...clip, scale } });
  console.log(`shots: ${name}`);
};
const theme = async (t) => {
  await panel('options');
  await click(`#theme [data-theme-pick="${t}"]`);
};
try {
  for (const c of COMBOS) {
    const tag = `${c.w}-${c.theme}`;
    await send('Emulation.setDeviceMetricsOverride', { width: c.w, height: c.h, deviceScaleFactor: 1, mobile: false });
    await load(url);
    if (await exists('[data-testid="story-next"]')) for (let i = 0; i < 10 && (await exists('[data-testid="story-next"]')); i++) await click('[data-testid="story-next"]');
    await theme(c.theme);
    await cheats();
    await speed(0);
    await click('#new-game');
    await waitFor(`!!document.querySelector('[data-testid="story-modal"]')`, 'the intro modal');
    await sleep(500);
    await shot(`intro__${tag}`, { x: 0, y: 0, width: c.w, height: c.h });
    await shot(`title__${tag}`, await evaluate(`(() => { const a = document.querySelector('.col-left').getBoundingClientRect(); const n = document.querySelector('.nav-grid').getBoundingClientRect(); return { x: a.x, y: a.y, width: a.width, height: n.bottom + 8 - a.y }; })()`), 2);
    await autoStory();
    await panel('story');
    await sleep(400);
    await shot(`story11__${tag}`, await rect('.col-mid'));
    await cheats();
    await click('#accel-quest');
    await waitFor(`document.getElementById('accel-status')?.dataset.running === '0'`, 'a quest step', 90000);
    await speed(0);
    await panel('story');
    await sleep(500);
    await shot(`act1__${tag}`, { x: 0, y: 0, width: c.w, height: c.h });
    await debugMenu();
    await evaluate(`(() => { const s = document.getElementById('scenario'); s.value = 'story_acts'; s.dispatchEvent(new Event('change', { bubbles: true })); s.dispatchEvent(new Event('input', { bubbles: true })); })()`);
    await click('#start-scenario');
    await sleep(800);
    await speed(0);
    await panel('story');
    await sleep(500);
    for (const n of [1, 2, 3, 4]) {
      await evaluate(`(() => { const band = document.querySelector('.act-band[data-act="${n}"]'); const box = document.querySelector('.col-mid');
        box.scrollTop += band.getBoundingClientRect().top - box.getBoundingClientRect().top - 40; })()`);
      await sleep(250);
      await shot(`act${n}_band__${tag}`, await rect('.col-mid'));
    }
  }
} finally {
  b.close();
}
