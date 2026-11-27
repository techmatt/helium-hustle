// The Buildings view's height, before and after 124's compact cards: every card shown
// (`?all-cards`) in scenario everything, each card's height and the panel's full height.
//   HH_URL=http://localhost:<port>/ node experiments/results/124_parked_batch_cards_nav_project_popups/cards.mjs <label> <shots dir>
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { launch, driver, capture, sleep } from '../../../web/scripts/cdp.mjs';

const [label = 'now', dir = 'out/124_parked_batch_cards_nav_project_popups/shots'] = process.argv.slice(2);
const shots = resolve(dir);
mkdirSync(shots, { recursive: true });
const b = await launch({ width: 1600, height: 900, profileRoot: resolve(`out/124_parked_batch_cards_nav_project_popups/profile-${label}`), portBase: 9400 });
const d = driver(b, { url: `${process.env.HH_URL}?all-cards`, settle: 700 });
try {
  await d.load();
  for (let i = 0; i < 40 && (await d.exists('[data-testid="story-next"]')); i++) await d.click('[data-testid="story-next"]');
  await d.debugMenu();
  await d.speed(0);
  await d.setValue('#scenario', 'everything');
  await d.click('#start-scenario');
  await sleep(300);
  for (let i = 0; i < 40 && (await d.exists('[data-testid="story-next"]')); i++) await d.click('[data-testid="story-next"]');
  await d.panel('buildings');
  await sleep(400);
  const r = await b.evaluate(`(() => {
    const hs = [...document.querySelectorAll('.card.building:not([data-testid="buy-cheat_building"])')].map((c) => c.getBoundingClientRect().height).sort((a, z) => a - z);
    const p = document.querySelector('[data-panel="buildings"]') ?? document.querySelector('.col-mid');
    return { cards: hs.length, median: Math.round(hs[hs.length >> 1]), min: Math.round(hs[0]), max: Math.round(hs.at(-1)), panel: Math.round(p.scrollHeight) };
  })()`);
  console.log(JSON.stringify({ label, ...r }));
  await capture(b, resolve(shots, `124_buildings-${label}.png`));
} finally {
  await b.close();
}
