// 109's check in the running game (vite preview at HH_URL): the nav has one layout all game.
// A new game, scenario late_unlocks and scenario everything show the same two-column grid
// (same height, every tile shown in both at the same place), and Research opening in a new
// game (two Solar Panels) fills its own cell, fades in, and moves no other tile. From the repo
// root, a static build served:
//   HH_URL=http://localhost:<port>/ node experiments/results/109_nav_single_layout/check.mjs out/109_nav_single_layout/shots [width height]
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { launch, driver, capture, sleep } from '../../../web/scripts/cdp.mjs';

const url = process.env.HH_URL;
const shots = resolve(process.argv[2] ?? 'out/109_nav_single_layout/shots');
const width = Number(process.argv[3] ?? 1280);
const height = Number(process.argv[4] ?? 800);
mkdirSync(shots, { recursive: true });
const b = await launch({ width, height, emulate: true, profileRoot: resolve('out/109_nav_single_layout/profile') });
const { evaluate } = b;
const d = driver(b, { url, settle: 700 });
const { exists, click, setValue, speed, debugMenu, cheats, grant, panel } = d;
let fails = 0;
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) fails++;
};
const tag = width === 1280 ? '' : `_${width}`;
const shot = async (n) => {
  const r = await evaluate(`(() => { const r = document.querySelector('.col-left').getBoundingClientRect(); return { x: r.x, y: r.y, w: r.width, h: Math.min(r.height, 520) }; })()`);
  await capture(b, resolve(shots, `109_${n}${tag}.png`), { clip: { x: r.x, y: r.y, width: r.w, height: r.h, scale: 2 } });
};
const dismiss = async () => {
  for (let i = 0; i < 60 && (await exists('[data-testid="story-next"]')); i++) await click('[data-testid="story-next"]');
};
/** The nav's box, its column count, and each shown tile's box and whether its label fits. */
const nav = () =>
  evaluate(`(() => {
    const g = document.querySelector('.nav-grid');
    const gr = g.getBoundingClientRect();
    const tiles = {};
    for (const t of g.querySelectorAll('.nav-tile')) {
      const r = t.getBoundingClientRect();
      const s = t.querySelector('span:not(.dot)');
      tiles[t.dataset.nav] = { x: Math.round(r.x - gr.x), y: Math.round(r.y - gr.y), w: Math.round(r.width), h: Math.round(r.height),
        fits: s.scrollWidth <= s.clientWidth + 1 && s.getBoundingClientRect().right <= r.right + 0.5, fresh: t.classList.contains('fresh') };
    }
    return { h: Math.round(gr.height), cols: getComputedStyle(g).gridTemplateColumns.split(' ').length, cells: g.children.length, tiles };
  })()`);
const same = (a, b) => a.x === b.x && a.y === b.y && a.w === b.w && a.h === b.h;
const compare = (a, b, what) => {
  const common = Object.keys(a.tiles).filter((id) => id in b.tiles);
  const moved = common.filter((id) => !same(a.tiles[id], b.tiles[id]));
  check(a.h === b.h && a.cols === b.cols, `${what}: same grid (${a.cols} cols, ${a.h} px vs ${b.cols} cols, ${b.h} px)`);
  check(moved.length === 0, `${what}: ${common.length} tiles in both, none moved${moved.length ? `: ${moved.join(', ')}` : ''}`);
};
const scenario = async (name) => {
  await debugMenu();
  await speed(0);
  await setValue('#scenario', name);
  await click('#start-scenario');
  await sleep(500);
  await dismiss();
  await speed(0);
};

// A new game.
await d.load();
await dismiss();
await speed(0);
await panel('buildings');
const fresh = await nav();
console.log('new game', JSON.stringify(fresh));
check(fresh.cols === 2, `two columns (${fresh.cols})`);
check(fresh.cells === 14, `a new game holds Act 1's 14 cells (${fresh.cells})`);
check(Object.values(fresh.tiles).every((t) => t.fits), 'every label fits its tile');
await shot('new_game');

// Research opening in it: two Solar Panels (cheat credits), then a day for the event.
await cheats();
await grant('credits', 1000);
await grant('titanium', 100);
await speed(1);
await sleep(1500);
await panel('buildings');
// A new game owns one already.
while (Number(await evaluate(`document.querySelector('.card.building[data-id="solar_panel"]').dataset.owned`)) < 2) {
  await d.waitFor(`!!document.querySelector('.card.building[data-id="solar_panel"].can-buy')`, 'a buyable Solar Panel', 8000);
  await click('.card.building[data-id="solar_panel"].can-buy .card-text');
}
await d.waitFor(`!!document.querySelector('.nav-tile[data-nav="research"]')`, 'the Research tile', 15000);
await speed(0);
await dismiss();
await sleep(150);
const mid = await evaluate(`getComputedStyle(document.querySelector('.nav-tile[data-nav="research"]')).opacity`);
await sleep(700);
const opened = await nav();
compare(fresh, opened, 'Research opening');
check(opened.tiles.research?.fresh === true && opened.tiles.research.x === 0 && opened.tiles.research.y > 0, `Research fills its own cell (row 2, left), fading in (opacity ${mid} soon after)`);
await shot('research_opened');

// Late Act 1: scenario late_unlocks, then everything.
await scenario('late_unlocks');
const late = await nav();
console.log('late_unlocks', JSON.stringify(late));
compare(fresh, late, 'new game vs late_unlocks');
check(Object.values(late.tiles).every((t) => t.fits), 'every label fits its tile (late_unlocks)');
await shot('late_unlocks');
await scenario('everything');
const all = await nav();
console.log('everything', JSON.stringify(all));
compare(fresh, all, 'new game vs everything');
check(Object.values(all.tiles).every((t) => t.fits), 'every label fits its tile (everything)');
await shot('everything');
// A reload keeps the layout and fades nothing.
await d.load();
await dismiss();
const reloaded = await nav();
compare(all, reloaded, 'everything reloaded');
check(Object.values(reloaded.tiles).every((t) => !t.fresh), 'after a reload no tile fades in');
// Dark theme, for the record.
await evaluate(`document.documentElement.dataset.theme = 'dark'`);
await shot('everything_dark');

await b.close();
console.log(fails ? `${fails} failed` : 'all ok');
process.exit(fails ? 1 : 0);
