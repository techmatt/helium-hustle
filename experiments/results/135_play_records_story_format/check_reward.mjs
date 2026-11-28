// 135's browser check against a `vite preview` build (HH_URL): completing 1.1 Online (no
// payoff now) opens no page of its own; 1.2 First Shipment's story follows, one page a
// panel, its last page ending with 1.1's Reward line, 20 propellant included. Shot
// 135_quest_1_2_reward.png here and in the Drive screenshots folder.
//   HH_URL=http://localhost:<port>/ node experiments/results/135_play_records_story_format/check_reward.mjs
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, sleep } from '../../../web/scripts/cdp.mjs';

const url = pageUrl('check');
const here = dirname(fileURLToPath(import.meta.url));
const OUT = 'out/135_play_records_story_format/helper';
const DRIVE = 'C:/Code/gdrive-sync/helium-hustle/screenshots';
mkdirSync(OUT, { recursive: true });
const b = await launch({ width: 1600, height: 900, profileRoot: resolve(`${OUT}/profile`) });
const { evaluate } = b;
const d = driver(b, { url, settle: 500 });
const { waitFor, exists, click, load, cheats, grant, readStory, speed, panel } = d;
let fails = 0;
const check = (ok, what) => {
  if (!ok) fails++;
  console.log(`${ok ? 'PASS' : 'FAIL'} ${what}`);
};
const modal = () =>
  evaluate(`(() => {
    const m = document.querySelector('[data-testid="story-modal"]');
    if (!m) return null;
    return { quest: m.dataset.quest, kind: m.dataset.kind, text: [...m.querySelectorAll('.story-text')].map((p) => p.textContent).join('|'),
      mech: [...m.querySelectorAll('.mech-line')].map((l) => l.textContent.trim()) };
  })()`);
try {
  await load(url);
  const intro = await readStory();
  check(intro.length >= 1 && intro.every((p) => p.quest === 'qpower_boot_sequence'), `a new game opens 1.1's story (${intro.length} pages)`);
  await cheats();
  await grant('credits', 500);
  await grant('titanium', 50);
  await panel('buildings');
  await sleep(300);
  await click('[data-testid="buy-solar_panel"]', 'buy a solar panel');
  await sleep(300);
  const owned = await evaluate(`document.querySelector('.card.building[data-id="solar_panel"]')?.dataset.owned`);
  check(owned === '2', `a second Solar Panel bought (owned ${owned})`);
  await speed(1);
  await waitFor(`!!document.querySelector('[data-testid="story-modal"]')`, "1.2's story", 8000);
  const pages = [];
  for (let i = 0; i < 10; i++) {
    const p = await modal();
    if (!p) break;
    pages.push(p);
    const last = await evaluate(`document.querySelector('[data-testid="story-next"]').textContent`);
    if (last !== 'Continue') {
      await sleep(300);
      const file = resolve(here, '135_quest_1_2_reward.png');
      await capture(b, file);
      try {
        copyFileSync(file, `${DRIVE}/135_quest_1_2_reward.png`);
      } catch (e) {
        console.log(`no Drive copy: ${e.message}`);
      }
      break;
    }
    await click('[data-testid="story-next"]', 'story next');
    await sleep(200);
  }
  console.log(JSON.stringify(pages, null, 1));
  check(pages.length > 0 && pages.every((p) => p.quest === 'qfirst_shipment' && p.kind === 'story'), "1.1's completion has no page of its own: only 1.2's story");
  const end = pages.at(-1)?.mech ?? [];
  check(/^Goal:/.test(end[0] ?? '') && /^Reward:.*20 propellant/.test(end.at(-1) ?? ''), `the last page ends with 1.1's Reward line: ${end.join(' / ')}`);
  check(pages.slice(0, -1).every((p) => p.mech.length === 0), 'the earlier panels carry no mechanics');
  await click('[data-testid="story-next"]', 'begin');
  await sleep(200);
  check(!(await exists('[data-testid="story-modal"]')), 'Begin closes it');
} finally {
  await b.close();
}
process.exit(fails ? 1 : 0);
