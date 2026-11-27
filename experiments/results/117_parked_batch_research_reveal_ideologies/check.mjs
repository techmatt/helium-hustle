// 117's direct checks of each parked edit in the running game (vite preview at HH_URL).
// From the repo root, a static build served:
//   HH_URL=http://localhost:<port>/ node experiments/results/117_parked_batch_research_reveal_ideologies/check.mjs out/117_parked_batch_research_reveal_ideologies/shots
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { launch, driver, capture, sleep } from '../../../web/scripts/cdp.mjs';

const url = process.env.HH_URL;
const shots = resolve(process.argv[2] ?? 'out/117_parked_batch_research_reveal_ideologies/shots');
mkdirSync(shots, { recursive: true });
const b = await launch({ width: 1600, height: 1000, profileRoot: resolve('out/117_parked_batch_research_reveal_ideologies/profile') });
const { evaluate } = b;
const d = driver(b, { url, settle: 700 });
const { waitFor, exists, click, setValue, speed, panel, debugMenu, cheats, grant } = d;
let fails = 0;
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) fails++;
};
const shot = async (n, sel) => {
  const clip = sel
    ? await evaluate(`(() => { const r = document.querySelector('${sel}')?.getBoundingClientRect(); return r ? { x: r.left + scrollX, y: r.top + scrollY, width: r.width, height: Math.min(r.height, 2400), scale: 1 } : null; })()`)
    : null;
  await capture(b, resolve(shots, `117_${n}.png`), clip ? { clip, captureBeyondViewport: true } : {});
};
const story = async () => {
  for (let i = 0; i < 40 && (await exists('[data-testid="story-next"]')); i++) await click('[data-testid="story-next"]');
};
const groups = () => evaluate(`[...document.querySelectorAll('[data-panel="research"] .fold-head')].map((h) => h.textContent.replace(/^[▼▶]\\s*/, '').replace(/\\d+$/, '').trim())`);
const speedOn = (s) => evaluate(`document.querySelector('#speeds button[data-speed="${s}"]')?.getAttribute('aria-pressed') === 'true'`);

try {
  await d.load();
  await debugMenu();
  await speed(0);
  await setValue('#scenario', 'late_unlocks');
  await click('#start-scenario');
  await sleep(500);
  await speed(0);
  // Research bought in an earlier run (the career's `researched`), and Propellant Synthesis
  // bought this run so the Electrolysis Plant's card shows; then reload.
  await evaluate(`(() => {
    const s = JSON.parse(localStorage.getItem('hh.save'));
    const g = JSON.parse(s.state);
    for (const id of ['regolith_processing', 'liquid_cooling', 'market_awareness', 'dream_protocols', 'propellant_synthesis'])
      if (!g.career.researched.includes(id)) g.career.researched.push(id);
    if (!g.run.research.includes('propellant_synthesis')) g.run.research.push('propellant_synthesis');
    // Scores on each axis, so the rank bars show a fill (one heading down).
    g.run.ideology = { nationalist: '640', humanist: '180', rationalist: '-60' };
    s.state = JSON.stringify(g);
    s.speed = 0;
    localStorage.setItem('hh.save', JSON.stringify(s));
    // The page saves as it leaves: keep this edit.
    Storage.prototype.setItem = () => {};
  })()`);
  await d.load();
  await story();
  await speed(0);
  const bought = await evaluate(`JSON.parse(JSON.parse(localStorage.getItem('hh.save')).state).career.researched`);
  check(['regolith_processing', 'market_awareness', 'dream_protocols'].every((x) => bought.includes(x)), `the career has bought ${JSON.stringify(bought)}`);

  // P3: the Electrolysis Plant's card.
  await panel('buildings');
  const flows = await evaluate(`document.querySelector('[data-testid="buy-electrolysis"] .effects')?.textContent ?? ''`);
  check(/\+1 Propellant\/day/.test(flows), `P3: Electrolysis Plant card reads "${flows}"`);
  await shot('p3_electrolysis', '[data-testid="buy-electrolysis"]');

  // P4: the Ideologies view.
  await panel('ideologies');
  const m = await evaluate(`(() => {
    const p = document.querySelector('[data-panel="ideologies"]').getBoundingClientRect();
    return [...document.querySelectorAll('[data-panel="ideologies"] .axis')].map((a) => {
      const r = a.getBoundingClientRect();
      const e = a.querySelector('.axis-emblem').getBoundingClientRect();
      const bar = a.querySelector('.bar').getBoundingClientRect();
      const n = a.querySelector('.axis-rank-num');
      const name = a.querySelector('.axis-name');
      return { axis: a.dataset.axis, card: [Math.round(r.width), Math.round(r.height)], emblem: [Math.round(e.width), Math.round(e.height)],
        bar: [Math.round(bar.width), Math.round(bar.height)], panel: Math.round(p.width),
        rank: n.textContent, rankPx: getComputedStyle(n).fontSize, namePx: getComputedStyle(name).fontSize, nameAlign: getComputedStyle(name).textAlign,
        label: a.querySelector('.bar-label')?.textContent ?? '' };
    });
  })()`);
  console.log('     axes:', JSON.stringify(m));
  check(m.length === 3, 'P4: three axis cards');
  check(m.every((a) => a.emblem[0] === 160 && a.emblem[1] >= 190), `P4: emblem slot 160 px wide, the card's full height (${m.map((a) => a.emblem.join('x')).join(', ')})`);
  check(m.every((a) => parseFloat(a.rankPx) >= 48 && parseFloat(a.namePx) >= 28 && a.nameAlign === 'center'), 'P4: large rank numeral, centered name');
  check(m.every((a) => a.bar[1] >= 28 && a.bar[0] >= a.card[0] - 160 - 60), `P4: full-width rank bar (${m.map((a) => a.bar.join('x')).join(', ')})`);
  await shot('p4_ideologies', '[data-panel="ideologies"]');
  await evaluate(`document.documentElement.dataset.theme = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark'`);
  await sleep(200);
  await shot('p4_ideologies_other_theme', '[data-panel="ideologies"]');
  await evaluate(`document.documentElement.dataset.theme = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark'`);

  // P1: retire at 10x; Continue plays the new run at 1x.
  await speed(10);
  await sleep(300);
  await speed(0);
  await story();
  await speed(10);
  await panel('retirement');
  await click('[data-testid="retire"]');
  await waitFor(`!!document.querySelector('[data-testid="retire-confirm"]')`, 'the confirm', 3000);
  await click('[data-testid="retire-confirm"]');
  await waitFor(`!!document.querySelector('[data-testid="summary"]')`, 'the summary', 5000);
  await sleep(300);
  await click('[data-testid="continue"]');
  await sleep(150);
  await story();
  const at1 = await speedOn(1);
  await speed(0);
  check(at1, 'P1: after the retirement the new run plays at 1x (was 10x)');
  check((await d.run()) === 2, `run 2 (day ${await d.day()})`);

  // P2: run 2's first day shows Blueprints and (A Voice active) Political Influence and Programs only.
  const day = await d.day();
  await panel('research');
  const g0 = await groups();
  check(g0.includes('Blueprints') && g0.includes('Political Influence') && g0.every((x) => ['Blueprints', 'Political Influence', 'Programs'].includes(x)), `P2: run 2 day ${day} research groups ${JSON.stringify(g0)}`);
  await shot('p2_run2_day0_research', '[data-panel="research"]');
  // A Research Lab: Self-Maintenance and Extraction appear (the one-off fires at the day's end), with the nav dot.
  await cheats();
  await grant('credits', 5000);
  await grant('circuits', 100);
  await panel('buildings');
  await click('[data-testid="buy-research_lab"]');
  await speed(1);
  await waitFor(`!!document.querySelector('.nav-tile[data-nav="research"] .dot-new')`, 'the research dot', 6000).catch(() => {});
  await speed(0);
  check(await exists('.nav-tile[data-nav="research"] .dot-new'), 'P2: the Research tile gets its dot when the groups appear');
  await shot('p2_nav_dot', '.nav-grid');
  await panel('research');
  const g1 = await groups();
  check(['Self-Maintenance', 'Extraction'].every((x) => g1.includes(x)) && !g1.includes('Market Analysis'), `P2: after the Lab ${JSON.stringify(g1)}`);
  await shot('p2_after_lab', '[data-panel="research"]');

  // P2: Market Analysis waits for this run's speculators (116's speculators_due scenario,
  // Market Awareness bought in an earlier run, Research open).
  await d.load();
  await debugMenu();
  await setValue('#scenario', 'speculators_due');
  await click('#start-scenario');
  await sleep(800);
  await speed(0);
  await evaluate(`(() => {
    const s = JSON.parse(localStorage.getItem('hh.save'));
    const g = JSON.parse(s.state);
    g.career.researched.push('market_awareness');
    g.run.enabled_panels.push('research');
    s.state = JSON.stringify(g);
    s.speed = 0;
    localStorage.setItem('hh.save', JSON.stringify(s));
    Storage.prototype.setItem = () => {};
  })()`);
  await d.load();
  await story();
  await speed(0);
  await panel('research');
  const s0 = await groups();
  check(!s0.includes('Market Analysis'), `P2: before speculators ${JSON.stringify(s0)}`);
  await panel('launch_pads');
  await evaluate(`[...document.querySelectorAll('[data-panel="launch_pads"] select')].forEach((s) => { s.value = 'he3'; s.dispatchEvent(new Event('change', { bubbles: true })); })`);
  await sleep(300);
  await speed(10);
  let arrived = false;
  for (let t = 0; t < 400 && !arrived; t++) {
    const q = await evaluate(`document.querySelector('[data-testid="story-modal"]')?.dataset.quest ?? null`);
    if (q === 'speculators_arrive') arrived = true;
    else if (q) {
      await click('[data-testid="story-next"]');
      await speed(10);
    } else {
      if (t % 6 === 5) {
        await cheats();
        await grant('he3', 300);
        await grant('propellant', 400);
        await panel('launch_pads');
      }
      await evaluate(`[...document.querySelectorAll('[data-panel="launch_pads"] [data-testid^="launch-"]')].forEach((b) => { if (!b.disabled) b.click(); })`);
      await sleep(250);
    }
  }
  check(arrived, 'speculators arrived');
  await story();
  await speed(0);
  await panel('research');
  const s1 = await groups();
  check(s1.includes('Market Analysis'), `P2: after speculators arrive ${JSON.stringify(s1)}`);
  await shot('p2_market_analysis', '[data-panel="research"]');
  check(b.errors.length === 0, `no page errors ${JSON.stringify(b.errors)}`);
} catch (e) {
  console.log('FAIL', e.message);
  fails++;
} finally {
  await b.close();
}
console.log(fails ? `${fails} failed` : 'all ok');
process.exit(fails ? 1 : 0);
