// `node shots.mjs <url> [sizes|final|banner]` (banner: 1.b's completion banner, below): 080's screenshots of Act 1's quest pictures in the
// running game (a `vite preview` of a static build at <url>), into this folder's shots/ (PNG).
//   sizes  the thumbnail-size comparison: scenario story_acts (every Act 1 quest and side
//          quest done), Story scrolled to the Act 1 band, with the thumbnail forced to 48x32
//          (078), 60x40 and 72x48 (079) by an injected style (a done side quest's at 3/4),
//          1280x800 light and 1920x1080 dark: thumbs-<w>x<h>__<combo>.png (middle column)
//   final  the shipped size, both combos: a new game's 1.1 modal and left column (intro,
//          whole window), Story's Act 1 in story_acts (act1), a replayed 1.7 modal (replay,
//          whole window), and 1.2 active after the autopilot's first step (left column, left2)
import { mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, sleep } from '../../../web/scripts/cdp.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..');
const OUT = path.join(HERE, 'shots');
mkdirSync(OUT, { recursive: true });
const url = process.argv[2];
const mode = process.argv[3] ?? 'final';
if (!url) throw new Error('shots: give the preview url');

const COMBOS = [
  { w: 1280, h: 800, theme: 'light' },
  { w: 1920, h: 1080, theme: 'dark' },
];
const SIZES = [
  [48, 32],
  [60, 40],
  [72, 48],
];
const b = await launch({ width: 1280, height: 800, emulate: true, profileRoot: path.join(REPO, 'out', '080_ship_act1_quest_pictures', 'profile') });
const { evaluate, send } = b;
const { waitFor, exists, click, load, speed, panel, cheats, debugMenu, autoStory } = driver(b, { url, settle: 700 });
const rect = (sel) => evaluate(`(() => { const r = document.querySelector('${sel}').getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height }; })()`);
const shot = async (name, clip) => {
  await capture(b, path.join(OUT, `${name}.png`), { clip: { ...clip, scale: 1 } });
  console.log(`shots: ${name}`);
};
const theme = async (t) => {
  await panel('options');
  await click(`#theme [data-theme-pick="${t}"]`);
};
const storyActs = async () => {
  await debugMenu();
  await evaluate(`(() => { const s = document.getElementById('scenario'); s.value = 'story_acts'; s.dispatchEvent(new Event('change', { bubbles: true })); s.dispatchEvent(new Event('input', { bubbles: true })); })()`);
  await click('#start-scenario');
  await sleep(800);
  await speed(0);
  await panel('story');
  await sleep(500);
  await evaluate(`(() => { const band = document.querySelector('.act-band[data-act="1"]'); const box = document.querySelector('.col-mid');
    box.scrollTop += band.getBoundingClientRect().top - box.getBoundingClientRect().top - 8; })()`);
  await sleep(300);
};
const forceThumb = (w, h) =>
  evaluate(`(() => { let s = document.getElementById('thumb-size'); if (!s) { s = document.createElement('style'); s.id = 'thumb-size'; document.head.append(s); }
    s.textContent = '.quest-pic-thumb { width: ${w}px !important; height: ${h}px !important; } .side-title .quest-pic-thumb { width: ${Math.round((w * 3) / 4)}px !important; height: ${Math.round((h * 3) / 4)}px !important; }'; })()`);
const begin = async (c) => {
  await send('Emulation.setDeviceMetricsOverride', { width: c.w, height: c.h, deviceScaleFactor: 1, mobile: false });
  await load(url);
  if (await exists('[data-testid="story-next"]')) for (let i = 0; i < 10 && (await exists('[data-testid="story-next"]')); i++) await click('[data-testid="story-next"]');
  await theme(c.theme);
  await cheats();
  await speed(0);
};
try {
  for (const c of COMBOS) {
    const tag = `${c.w}-${c.theme}`;
    await begin(c);
    if (mode === 'banner') {
      // Scenario late_unlocks (1.8 done, 1.b open): Complete all research finishes 1.b,
      // whose completion banner shows its picture (middle column).
      await debugMenu();
      await evaluate(`(() => { const s = document.getElementById('scenario'); s.value = 'late_unlocks'; s.dispatchEvent(new Event('change', { bubbles: true })); s.dispatchEvent(new Event('input', { bubbles: true })); })()`);
      await click('#start-scenario');
      await sleep(800);
      await autoStory();
      await speed(1);
      await sleep(1500);
      await speed(0);
      await debugMenu();
      await click('#cheat-complete-research');
      await speed(1);
      await waitFor(`!!document.querySelector('[data-testid="story-banner"] [data-testid="quest-pic-banner"]')`, "a side quest's banner with its picture", 20000);
      await speed(0);
      const got = await evaluate(`[...document.querySelectorAll('[data-testid="story-banner"]')].map((e) => e.dataset.story + ':' + e.querySelector('[data-testid="quest-pic-banner"]')?.dataset.picture).join(' ')`);
      console.log(`shots: banner ${got}`);
      await evaluate(`document.querySelector('.col-mid').scrollTop = 0`);
      await sleep(300);
      await shot(`banner__${tag}`, await rect('.col-mid'));
      continue;
    }
    if (mode === 'sizes') {
      await storyActs();
      for (const [w, h] of SIZES) {
        await forceThumb(w, h);
        await sleep(300);
        await shot(`thumbs-${w}x${h}__${tag}`, await rect('.col-mid'));
      }
      continue;
    }
    await click('#new-game');
    await waitFor(`!!document.querySelector('[data-testid="story-modal"]')`, 'the intro modal');
    await sleep(600);
    await shot(`intro__${tag}`, { x: 0, y: 0, width: c.w, height: c.h });
    await autoStory();
    await cheats();
    await click('#accel-quest');
    await waitFor(`document.getElementById('accel-status')?.dataset.running === '0'`, 'a quest step', 90000);
    await speed(0);
    await sleep(500);
    await shot(`left2__${tag}`, await rect('.col-left'));
    await storyActs();
    await shot(`act1__${tag}`, await rect('.col-mid'));
    // Act 1's done side quests, each with its thumbnail (listed in the console).
    const sides = await evaluate(`(() => { const s = [...document.querySelectorAll('[data-side]')].filter((e) => /^1\\.[a-c]/.test(e.textContent.trim()));
      const box = document.querySelector('.col-mid'); if (s[0]) box.scrollTop += s[0].getBoundingClientRect().top - box.getBoundingClientRect().top - 120;
      return s.map((e) => e.dataset.side + ':' + (e.querySelector('[data-testid="quest-pic-thumb"]')?.dataset.picture ?? '-')); })()`);
    console.log(`shots: side quests ${sides.join(' ')}`);
    await sleep(300);
    await shot(`sides__${tag}`, await rect('.col-mid'));
    await evaluate(`window.__autoStory?.disconnect(); window.__autoStory = undefined`);
    await click('[data-beat="qburnout"]');
    await waitFor(`!!document.querySelector('[data-testid="story-modal"] [data-testid="quest-pic-modal"]')`, "1.7's replayed modal");
    await sleep(600);
    await shot(`replay__${tag}`, { x: 0, y: 0, width: c.w, height: c.h });
    for (let i = 0; i < 10 && (await exists('[data-testid="story-next"]')); i++) await click('[data-testid="story-next"]');
  }
} finally {
  b.close();
}
