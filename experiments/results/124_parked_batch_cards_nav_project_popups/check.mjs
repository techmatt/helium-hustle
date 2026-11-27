// 124's check in the running game (vite preview at HH_URL): research cards' green header bar,
// the shorter building cards, the nav order, a project completion's popup and Events line,
// and the three small fixes (the run summary's backdrop over a reload, gray skipped repeat
// dots, jump ahead refused under a popup). From the repo root, a static build served:
//   HH_URL=http://localhost:<port>/ node experiments/results/124_parked_batch_cards_nav_project_popups/check.mjs out/124_parked_batch_cards_nav_project_popups/shots
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { launch, driver, capture, sleep, q } from '../../../web/scripts/cdp.mjs';

const url = process.env.HH_URL;
const shots = resolve(process.argv[2] ?? 'out/124_parked_batch_cards_nav_project_popups/shots');
mkdirSync(shots, { recursive: true });
const b = await launch({ width: 1600, height: 900, profileRoot: resolve('out/124_parked_batch_cards_nav_project_popups/profile') });
const { evaluate } = b;
const d = driver(b, { url, settle: 700 });
const { exists, click, setValue, speed, panel, debugMenu, cheats, grant, waitFor } = d;
let fails = 0;
const facts = {};
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) fails++;
};
const shot = (n) => capture(b, resolve(shots, `124_${n}.png`));
const scenario = async (name) => {
  await debugMenu();
  await speed(0);
  await setValue('#scenario', name);
  await click('#start-scenario');
  await sleep(300);
};
const dismiss = async () => {
  for (let i = 0; i < 60 && (await exists('[data-testid="story-next"]')); i++) {
    await click('[data-testid="story-next"]');
    await sleep(120);
  }
};
const headBg = (sel) => evaluate(`(() => { const h = document.querySelector(${q(sel)}); return h ? getComputedStyle(h).backgroundColor : null; })()`);

try {
  await d.load();
  await dismiss();

  // ---- nav order (P3): read row by row in two columns ----
  await scenario('everything');
  await dismiss();
  const tiles = await evaluate(`[...document.querySelectorAll('.nav-tile')].map((t) => t.dataset.nav)`);
  facts.nav = tiles;
  const head = ['buildings', 'research', 'commands', 'programs', 'launch_pads', 'projects'];
  const tail = ['achievements', 'story', 'stats', 'options'];
  check(JSON.stringify(tiles.slice(0, 6)) === JSON.stringify(head) && JSON.stringify(tiles.slice(-4)) === JSON.stringify(tail), `nav reads ${tiles.join(', ')}`);
  // Two columns: tile 0 and 1 share a row, 2 below 0.
  const rows = await evaluate(`[...document.querySelectorAll('.nav-tile')].slice(0, 4).map((t) => { const r = t.getBoundingClientRect(); return [Math.round(r.left), Math.round(r.top)]; })`);
  check(rows[0][1] === rows[1][1] && rows[2][0] === rows[0][0] && rows[2][1] > rows[0][1], `two columns, row by row: ${JSON.stringify(rows)}`);
  await shot('nav');

  // ---- research cards (P1): affordable has the green head, unaffordable doesn't ----
  await scenario('late_unlocks');
  await dismiss();
  await cheats();
  await panel('research');
  await sleep(200);
  const researchCards = () => evaluate(`[...document.querySelectorAll('.card.research:not(.done)')].map((c) => ({ id: c.dataset.id, can: c.classList.contains('can-buy'), bg: getComputedStyle(c.querySelector('.card-head')).backgroundColor, border: getComputedStyle(c).borderTopColor }))`);
  // Short of Science first (an unaffordable card), then granted some (an affordable one).
  const cannot = (await researchCards()).find((r) => !r.can);
  await shot('research-unaffordable');
  await cheats();
  await grant('science', '300');
  await panel('research');
  await sleep(200);
  const can = (await researchCards()).find((r) => r.can);
  const clear = (c) => c === 'rgba(0, 0, 0, 0)' || c === 'transparent';
  facts.research = { can, cannot };
  check(!!can && !clear(can.bg), `an affordable research (${can?.id}) has a tinted header: ${can?.bg}`);
  check(!!cannot && clear(cannot.bg), `an unaffordable research (${cannot?.id}) has none: ${cannot?.bg}`);
  check(!!can && !!cannot && can.border === cannot.border, `no green outline: borders ${can?.border} / ${cannot?.border}`);
  await shot('research');

  // ---- building cards (P2): flows beside the picture, the card short ----
  // Every card shown (`?all-cards`), so the two with a description are measured too.
  await d.load(`${url}?all-cards`);
  await dismiss();
  await scenario('everything');
  await dismiss();
  await panel('buildings');
  await sleep(300);
  const cards = await evaluate(`[...document.querySelectorAll('.card.building:not([data-testid="buy-cheat_building"])')].map((c) => {
    const img = c.querySelector('img.card-art')?.getBoundingClientRect();
    const fx = c.querySelector('.effects')?.getBoundingClientRect();
    const st = c.querySelector('[data-card-status]')?.getBoundingClientRect();
    return { id: c.dataset.id, pic: !!img, h: Math.round(c.getBoundingClientRect().height), beside: !!img && !!fx && fx.left >= img.right - 1 && fx.top < img.bottom, statusBeside: !!img && !!st && st.left >= img.right - 1, desc: !!c.querySelector('.desc'), numbers: c.querySelectorAll('.cost-item').length };
  })`);
  facts.cards = cards;
  const heights = cards.map((c) => c.h).sort((a, z) => a - z);
  const pictured = cards.filter((c) => c.pic);
  check(pictured.length > 5 && pictured.every((c) => c.beside), `every card's flows sit beside its picture (${pictured.filter((c) => !c.beside).map((c) => c.id).join(', ') || 'all'}; no picture: ${cards.filter((c) => !c.pic).map((c) => c.id).join(', ')})`);
  check(pictured.every((c) => c.statusBeside), 'the status row (Output at cap) sits beside the picture too');
  check(cards.every((c) => c.numbers >= 2), 'every card shows its cost and land');
  check(cards.filter((c) => c.desc).map((c) => c.id).sort().join() === cards.filter((c) => ['arbitrage_engine', 'microwave_receiver'].includes(c.id)).map((c) => c.id).sort().join(), `descriptions kept: ${cards.filter((c) => c.desc).map((c) => c.id).join(', ')}`);
  console.log(`     building card heights: median ${heights[heights.length >> 1]} px, ${heights[0]}-${heights.at(-1)} px over ${cards.length}`);
  facts.cardHeights = { median: heights[heights.length >> 1], min: heights[0], max: heights.at(-1) };
  await shot('buildings');

  // ---- a project completion (P4): its popup, then its Events line ----
  await scenario('late_unlocks');
  await dismiss();
  await cheats();
  for (const [r, n] of [['credits', '1000000'], ['circuits', '1000'], ['science', '50000'], ['he3', '100000'], ['titanium', '10000']]) await grant(r, n);
  await panel('projects');
  await sleep(200);
  const fundable = await evaluate(`[...document.querySelectorAll('.card.project')].filter((c) => !c.classList.contains('locked') && !c.classList.contains('done') && c.querySelector('button[title^="Fund at the most"]')).map((c) => c.dataset.id)`);
  facts.fundable = fundable;
  check(fundable.length > 0, `a fundable project: ${fundable.join(', ')}`);
  // Every fundable project at its most; the one done soonest is watched.
  for (const id of fundable) await evaluate(`[...document.querySelectorAll('.card.project[data-id="${id}"] button[title^="Fund at the most"]')].forEach((b) => b.click())`);
  await sleep(300);
  const soon = await evaluate(`[...document.querySelectorAll('.card.project')].map((c) => [c.dataset.id, Number((c.textContent.match(/Done in (\\d+) day/) ?? [0, 1e9])[1])]).sort((a, z) => a[1] - z[1])`);
  facts.soon = soon;
  // Other popups on the way (a quest) are read through until the project's.
  for (const end = Date.now() + 150000; Date.now() < end && !(await exists('[data-testid="project-done"]')); ) {
    if (await exists('[data-testid="story-next"]')) await click('[data-testid="story-next"]');
    else if (!(await evaluate(`document.querySelector('#speeds button[data-speed="10"]')?.getAttribute('aria-pressed') === 'true'`))) await speed(10);
    await sleep(250);
  }
  const popup = await evaluate(`(() => { const m = document.querySelector('[data-testid="story-modal"]'); return m && { quest: m.dataset.quest, title: m.querySelector('h2').textContent, rewards: [...m.querySelectorAll('.mech-reward')].map((x) => x.textContent), cost: m.querySelector('.mech-cost')?.textContent ?? '', pic: !!m.querySelector('.project-done img'), paused: document.querySelector('#speeds button[data-speed="0"]')?.getAttribute('aria-pressed') === 'true' }; })()`);
  facts.popup = popup;
  const pid = popup?.quest;
  check(!!popup && fundable.includes(pid) && popup.rewards.length > 0 && /Cost:/.test(popup.cost) && /\d/.test(popup.cost), `the completion pops up: ${JSON.stringify(popup)}`);
  check(!!popup?.pic, 'with its picture');
  check(!!popup?.paused, 'the clock held');
  await shot('project-popup');
  await click('[data-testid="story-next"]');
  await sleep(300);
  const line = await evaluate(`document.querySelector('[data-testid="events-list"] [data-project-done="${pid}"]')?.textContent ?? null`);
  check(!!line, `Events lists it: ${line}`);
  await evaluate(`document.querySelector('[data-testid="events-list"] [data-project-done="${pid}"]')?.click()`);
  await sleep(300);
  check(await exists('[data-testid="project-done"]'), 'a click on its Events line opens it again');
  await shot('project-events');

  // ---- 115: jump ahead refuses while a popup waits ----
  const before = await evaluate(`document.getElementById('day').textContent`);
  await cheats();
  check(await exists('[data-testid="project-done"]'), 'the popup still waits behind the debug menu');
  await evaluate(`(() => { const i = document.getElementById('cheat-days'); i.value = '30'; i.dispatchEvent(new Event('input', { bubbles: true })); })()`);
  await sleep(100);
  await evaluate(`document.getElementById('cheat-jump')?.click()`);
  await sleep(300);
  const notice = await evaluate(`document.getElementById('notice')?.textContent ?? ''`);
  const after = await evaluate(`document.getElementById('day').textContent`);
  facts.jump = { before, after, notice };
  check(before === after && /Jump ahead refused/.test(notice), `jump ahead under a popup: day ${before} → ${after}, notice "${notice}"`);
  await click('[data-testid="story-next"]');
  await sleep(200);
  await shot('jump-refused');

  // ---- 119: a repeated line's skipped runs draw gray ----
  await scenario('everything');
  await dismiss();
  await cheats();
  await grant('credits', '1000000');
  await grant('titanium', '1000000');
  await click('.prog-tabs button[data-program="0"]');
  if (await exists('.prog-procs button[title="Clear this program"]')) await click('.prog-procs button[title="Clear this program"]');
  await evaluate(`localStorage.setItem('hh-programs-mode', 'blocks')`);
  await click('.nav-tile[data-nav="programs"]');
  await waitFor(`!!document.querySelector('[data-testid="block-editor"]')`, 'the Blocks mode', 5000);
  const buy = await evaluate(`[...document.querySelectorAll('[data-group="Commands"] .ins-item')].map((b) => b.dataset.instruction).find((t) => /titanium/i.test(t) && /buy/i.test(t)) ?? null`);
  check(!!buy, `a Buy Titanium instruction: ${buy}`);
  await click(`[data-group="Commands"] [data-instruction=${q(buy)}]`);
  await sleep(250);
  await evaluate(`(() => {
    const el = document.querySelector('[data-testid="block-lines"] > li:last-child [data-testid="repeat-count"]');
    el.value = '3';
    el.dispatchEvent(new Event('input', { bubbles: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
  })()`);
  await sleep(250);
  await click('[data-takeover-close="programs"]');
  while (await evaluate(`(() => { const b = document.querySelector('[data-testid="proc-plus"]'); return !!b && !b.disabled; })()`)) await click('[data-testid="proc-plus"]');
  await click('.nav-tile[data-nav="programs"]');
  await waitFor(`!!document.querySelector('[data-testid="block-editor"]')`, 'the Blocks mode', 5000);
  await speed(1);
  await sleep(2500);
  await speed(0);
  const dots = await evaluate(`(() => { const r = document.querySelector('[data-testid="block-lines"] > li:last-child [data-repeat]'); return r && { repeat: r.dataset.repeat, skipped: r.dataset.repeatSkipped, title: r.title, dots: [...r.querySelectorAll('.rep-dot')].map((x) => x.className.replace('rep-dot ', '')) }; })()`);
  facts.dots = dots;
  check(!!dots && dots.dots.includes('rep-skipped') && !dots.dots.includes('rep-done'), `titanium at its cap: the 3x line's runs draw gray: ${JSON.stringify(dots)}`);
  await shot('repeat-skipped');
  await click('[data-takeover-close="programs"]');

  // ---- 120: a reload with the run summary up keeps the ended run's last day ----
  await scenario('late_unlocks');
  await dismiss();
  await speed(1);
  await sleep(1500);
  await speed(0);
  await dismiss();
  await debugMenu();
  await cheats();
  await setValue('#cheat-boredom', '100000');
  await sleep(150);
  await click('#cheat-set-boredom');
  await sleep(150);
  // Popups on the way (the boredom phases) are read through until the summary.
  for (const end = Date.now() + 60000; Date.now() < end && !(await exists('[data-testid="summary"]')); ) {
    if (await exists('[data-testid="story-next"]')) await click('[data-testid="story-next"]');
    else if (!(await evaluate(`document.querySelector('#speeds button[data-speed="1"]')?.getAttribute('aria-pressed') === 'true'`))) await speed(1);
    await sleep(250);
  }
  check(await exists('[data-testid="summary"]'), 'the forced retirement');
  const seen = await evaluate(`({ day: document.getElementById('day').textContent, run: document.getElementById('run').textContent })`);
  await shot('summary-before-reload');
  await d.load();
  await sleep(500);
  const reloaded = await evaluate(`({ summary: !!document.querySelector('[data-testid="summary"]'), day: document.getElementById('day').textContent, run: document.getElementById('run').textContent })`);
  facts.reload = { seen, reloaded };
  check(reloaded.summary && reloaded.day === seen.day && reloaded.run === seen.run && !/^Day 0$/.test(reloaded.day), `after a reload the summary keeps ${seen.run}, ${seen.day} behind it: ${reloaded.run}, ${reloaded.day}`);
  await shot('summary-after-reload');
  await click('[data-testid="continue"]');
  await sleep(500);
  const next = await evaluate(`document.getElementById('run').textContent`);
  check(next !== seen.run, `Continue plays the new run: ${next}`);
} catch (e) {
  console.log(`FAIL ${e.stack ?? e}`);
  fails++;
} finally {
  writeFileSync(resolve(shots, 'facts.json'), JSON.stringify(facts, null, 2));
  if (b.errors.length) console.log(`page errors: ${b.errors.join(' | ')}`);
  await b.close();
}
console.log(fails ? `${fails} failed` : 'all ok');
process.exit(fails ? 1 : 0);
