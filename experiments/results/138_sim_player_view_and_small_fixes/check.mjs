// 138's browser check, against a `vite preview` of web/ (HH_URL):
//   1. 1.1's completion shows "Buy Science and 20 propellant." (a second Solar Panel bought).
//   2. The Solar Panel card and the titanium row agree: 0.04 short of the price, the row reads
//      under it and the card says short; at the price, the card buys.
//   3. A reload with the run summary up keeps the ended run's day behind it.
// Run from web/:  HH_URL=http://localhost:<port>/ node ../experiments/results/138_sim_player_view_and_small_fixes/check.mjs <out dir>
import { resolve } from 'node:path';
import { mkdirSync } from 'node:fs';
import { capture, launch, driver, sleep } from '../../../web/scripts/cdp.mjs';

const url = process.env.HH_URL;
const out = resolve(process.argv[2] ?? '.');
mkdirSync(out, { recursive: true });
const b = await launch({ width: 1440, height: 900, profileRoot: resolve(out, 'profiles') });
const d = driver(b, { url });
const ev = b.evaluate;
const results = {};
try {
  await d.load();
  // The opening story modal(s), then pause.
  await d.readStory();
  await d.speed(0);
  await d.cheats();
  await d.grant('credits', 200);
  await d.grant('titanium', 30);
  await d.panel('buildings');
  await d.click('[data-testid="buy-solar_panel"]', 'buy solar panel');
  await sleep(500);
  await d.speed(1);
  await d.waitFor(`!!document.querySelector('[data-testid="story-modal"]')`, '1.1 completion', 8000);
  await d.speed(0).catch(() => {});
  // Every page of the chain, the one with 1.1's reward captured.
  const pages = [];
  for (let i = 0; i < 20 && (await d.exists('[data-testid="story-modal"]')); i++) {
    const text = await ev(`document.querySelector('[data-testid="story-modal"]').innerText`);
    pages.push(text);
    if (text.includes('Buy Science')) await capture(b, `${out}/138_reward_line.png`);
    await d.click('[data-testid="story-next"]', 'story next');
    await sleep(200);
  }
  const all = pages.join('\n');
  results.reward = { shown: all.includes('Buy Science and 20 propellant.'), oldText: all.includes('Research panel'), pages: pages.length };
  await d.speed(0);

  // 2. Titanium against the Solar Panel's price.
  await d.panel('buildings');
  const state = () =>
    ev(`(() => {
      const card = document.querySelector('[data-testid="buy-solar_panel"]');
      const price = Number(card.querySelector('.cost-item[data-res="titanium"]').textContent.replace(/[^0-9.]/g, ''));
      const row = document.querySelector('tr[data-res="titanium"]');
      return { price, amount: Number(row.dataset.amount), rowText: row.innerText.replace(/\\s+/g, ' '), short: card.dataset.short, buyable: card.classList.contains('can-buy') };
    })()`);
  let s = await state();
  await d.cheats();
  await d.grant('titanium', (s.price - s.amount - 0.04).toFixed(6));
  await d.panel('buildings');
  await sleep(300);
  results.under = await state();
  await capture(b, `${out}/138_titanium_under.png`);
  await d.cheats();
  await d.grant('titanium', '0.04');
  await d.panel('buildings');
  await sleep(300);
  results.at = await state();
  await capture(b, `${out}/138_titanium_at.png`);

  // 3. The run summary over a reload.
  await d.debugMenu();
  await d.setValue('#scenario', 'everything');
  await d.click('#start-scenario', 'scenario');
  await sleep(500);
  await d.readStory();
  await d.speed(1);
  await sleep(3000);
  await d.speed(0);
  await d.readStory();
  await d.panel('retirement');
  await d.click('[data-testid="retire"]', 'retire');
  await d.click('[data-testid="retire-confirm"]', 'retire confirm');
  await d.waitFor(`!!document.querySelector('[data-testid="summary"]')`, 'the run summary', 3000);
  const before = { day: await d.day(), bar: await ev(`document.getElementById('day').textContent`) };
  await d.load();
  await sleep(800);
  const after = { day: await d.day(), bar: await ev(`document.getElementById('day').textContent`), summary: await d.exists('[data-testid="summary"]') };
  await capture(b, `${out}/138_summary_reloaded.png`);
  results.summary = { before, after };
  results.errors = b.errors;
} finally {
  console.log(JSON.stringify(results, null, 2));
  b.close();
}
