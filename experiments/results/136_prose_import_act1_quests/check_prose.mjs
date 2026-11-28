// 136's browser check against a `vite preview` build (HH_URL), the item's done-when rows:
// 1.1's story opens with "You are HELIOS-1"; completing 1.1 shows 1.2's panels, the last
// ending with 1.1's Reward line; the quest box reads "Friends in High Places" at 1.5 (the
// debug autopilot's quest steps). Shots here and in the Drive screenshots folder.
//   HH_URL=http://localhost:<port>/ node experiments/results/136_prose_import_act1_quests/check_prose.mjs
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, sleep } from '../../../web/scripts/cdp.mjs';

const url = pageUrl('check');
const here = dirname(fileURLToPath(import.meta.url));
const OUT = 'out/136_prose_import_act1_quests/helper';
const DRIVE = 'C:/Code/gdrive-sync/helium-hustle/screenshots';
mkdirSync(OUT, { recursive: true });
const b = await launch({ width: 1600, height: 900, profileRoot: resolve(`${OUT}/profile`) });
const { evaluate } = b;
const d = driver(b, { url, settle: 500 });
const { waitFor, exists, click, load, cheats, grant, speed, panel, autoStory } = d;
let fails = 0;
const check = (ok, what) => {
  if (!ok) fails++;
  console.log(`${ok ? 'PASS' : 'FAIL'} ${what}`);
};
const shot = async (name) => {
  await sleep(300);
  const file = resolve(here, name);
  await capture(b, file);
  try {
    copyFileSync(file, `${DRIVE}/${name}`);
  } catch (e) {
    console.log(`no Drive copy: ${e.message}`);
  }
};
const modal = () =>
  evaluate(`(() => {
    const m = document.querySelector('[data-testid="story-modal"]');
    if (!m) return null;
    return { quest: m.dataset.quest, kind: m.dataset.kind, text: [...m.querySelectorAll('.story-text')].map((p) => p.textContent).join('|'),
      mech: [...m.querySelectorAll('.mech-line')].map((l) => l.textContent.trim()) };
  })()`);
/** Every page of the open modal, pressing Continue between them; stops on the last. */
const pages = async (shotName) => {
  const out = [];
  for (let i = 0; i < 10; i++) {
    const p = await modal();
    if (!p) break;
    out.push(p);
    const label = await evaluate(`document.querySelector('[data-testid="story-next"]').textContent`);
    if (label !== 'Continue') {
      if (shotName) await shot(shotName);
      break;
    }
    await click('[data-testid="story-next"]', 'story next');
    await sleep(200);
  }
  return out;
};
try {
  await load(url);
  await waitFor(`!!document.querySelector('[data-testid="story-modal"]')`, "1.1's story", 8000);
  const first = await modal();
  check(first?.quest === 'qpower_boot_sequence' && /^You are HELIOS-1/.test(first.text), `1.1's story opens "${first?.text.slice(0, 40)}"`);
  await shot('136_quest_1_1_story.png');
  const intro = await pages();
  console.log(`1.1: ${intro.length} pages`);
  await click('[data-testid="story-next"]', 'begin');
  await sleep(200);
  await cheats();
  await grant('credits', 500);
  await grant('titanium', 50);
  await panel('buildings');
  await sleep(300);
  await click('[data-testid="buy-solar_panel"]', 'buy a solar panel');
  await speed(1);
  await waitFor(`!!document.querySelector('[data-testid="story-modal"]')`, "1.2's story", 8000);
  const p12 = await pages('136_quest_1_2_reward.png');
  console.log(JSON.stringify(p12, null, 1));
  check(p12.length === 2 && p12.every((p) => p.quest === 'qfirst_shipment' && p.kind === 'story'), `completing 1.1 shows 1.2's panels (${p12.length})`);
  const end = p12.at(-1)?.mech ?? [];
  check(/^Reward:.*20 propellant/.test(end.at(-1) ?? ''), `the last panel ends with the Reward line: ${end.join(' / ')}`);
  await click('[data-testid="story-next"]', 'begin');
  await sleep(200);
  // On to 1.5 by the debug autopilot's quest steps (as e2e section 11).
  await speed(0);
  await autoStory();
  const beat = () => evaluate(`document.querySelector('[data-testid="quest"]')?.textContent.match(/^1\\.(\\d+)/)?.[1] ?? '0'`).then(Number);
  for (let step = 0; step < 8 && (await beat()) < 5; step++) {
    await cheats();
    await click('#accel-quest', 'autopilot: next quest step');
    await waitFor(`document.getElementById('accel-status')?.dataset.running === '0'`, 'the autopilot step', 90000);
    if (await exists('[data-testid="continue"]')) await click('[data-testid="continue"]', 'continue');
  }
  const box = await evaluate(`document.querySelector('[data-testid="quest"]')?.textContent ?? ''`);
  check(/^1\.5 — Friends in High Places/.test(box), `the quest box at 1.5: "${box.slice(0, 50)}"`);
  await shot('136_quest_1_5_box.png');
} finally {
  await b.close();
}
process.exit(fails ? 1 : 0);
