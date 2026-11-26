// 104's direct checks of each parked edit in the running game (vite preview at HH_URL).
// From the repo root, a static build served:
//   HH_URL=http://localhost:<port>/ node experiments/results/104_ui_polish_parked_edits/check.mjs out/104_ui_polish_parked_edits/shots
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { launch, driver, capture, sleep } from '../../../web/scripts/cdp.mjs';

const url = process.env.HH_URL;
const shots = resolve(process.argv[2] ?? 'out/104_ui_polish_parked_edits/shots');
mkdirSync(shots, { recursive: true });
const b = await launch({ width: 1600, height: 900, profileRoot: resolve('out/104_ui_polish_parked_edits/profile') });
const { evaluate } = b;
const d = driver(b, { url, settle: 700 });
const { waitFor, exists, click, setValue, speed, panel, debugMenu, cheats, grant } = d;
let fails = 0;
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) fails++;
};
const shot = (n) => capture(b, resolve(shots, `104_${n}.png`));
const navShown = (id) => exists(`.nav-tile[data-nav="${id}"]`);

try {
  await d.load();
  await debugMenu();
  await speed(0);
  await setValue('#scenario', 'late_unlocks');
  await click('#start-scenario');
  await sleep(500);
  await speed(0);
  // Re-lock Ideologies in the save (the scenario opens it), then reload.
  await evaluate(`(() => {
    const s = JSON.parse(localStorage.getItem('hh.save'));
    const g = JSON.parse(s.state);
    g.run.enabled_panels = g.run.enabled_panels.filter((p) => p !== 'ideologies');
    g.run.research = g.run.research.filter((r) => r !== 'geopolitical_intelligence');
    delete g.run.events.ideology_unlock;
    g.career.seen_events = g.career.seen_events.filter((e) => e !== 'ideology_unlock');
    s.state = JSON.stringify(g);
    s.speed = 0;
    localStorage.setItem('hh.save', JSON.stringify(s));
    // The page saves as it leaves: keep this edit.
    Storage.prototype.setItem = () => {};
  })()`);
  await d.load();
  for (let i = 0; i < 40 && (await exists('[data-testid="story-next"]')); i++) await click('[data-testid="story-next"]');
  check(!(await navShown('ideologies')), 'Ideologies tile hidden before Geopolitical Intelligence');
  const paused = () => evaluate(`document.querySelector('#speeds button[data-speed="0"]')?.getAttribute('aria-pressed') === 'true'`);
  check(await paused(), 'paused');
  await cheats();
  await grant('science', 1000);
  const day0 = await d.day();
  await panel('research');
  const desc = await evaluate(`document.querySelector('[data-testid="research-geopolitical_intelligence"] .desc')?.textContent ?? ''`);
  check(/Reveals the Ideologies panel at once\.$/.test(desc), `card text: "${desc}"`);
  await click('[data-testid="research-geopolitical_intelligence"]');
  await sleep(300);
  check(await navShown('ideologies'), 'P3: Ideologies tile shows at once after the purchase');
  check(await exists('[data-testid="ideology-summary"]'), 'P3: the left column Ideology section too');
  check((await d.day()) === day0 && (await paused()), `still paused on day ${day0}`);
  await shot('p3_ideologies_paused');
  await panel('ideologies');
  check(await exists('[data-panel="ideologies"]'), 'P3: the Ideologies panel opens');

  // P2: Hide completed on Research.
  await panel('research');
  const cards = () => evaluate(`document.querySelectorAll('[data-panel="research"] .card.research').length`);
  const doneCards = () => evaluate(`document.querySelectorAll('[data-panel="research"] .card.research.done').length`);
  const box = (p) => `[data-panel="${p}"] [data-testid="hide-completed"] input`;
  check(!(await evaluate(`document.querySelector('${box('research')}').checked`)), 'P2: Research starts unticked');
  const nDone = await doneCards();
  check(nDone >= 2, `P2: research done cards shown (${nDone}, Geopolitical Intelligence just bought among them)`);
  await click(box('research'));
  await sleep(200);
  check((await doneCards()) === 0, 'P2: ticked hides every bought research');
  await shot('p2_research_hidden');
  // Bought while the view is open: stays until leaving.
  const target = await evaluate(`[...document.querySelectorAll('[data-panel="research"] .card.research.can-buy')][0]?.dataset.id ?? null`);
  if (target) {
    await click(`[data-testid="research-${target}"]`);
    await sleep(300);
    check(await exists(`[data-testid="research-${target}"].done`), `P2: ${target} bought with the view open stays shown`);
    await panel('buildings');
    await panel('research');
    check(!(await exists(`[data-testid="research-${target}"]`)), `P2: ${target} hidden after leaving and coming back`);
  } else check(false, 'P2: no affordable research to test the stay-shown rule');
  // Projects: starts ticked (finished one-offs hidden); unticked shows Blueprint Archive done.
  await panel('projects');
  check(await evaluate(`document.querySelector('${box('projects')}').checked`), 'P2: Projects starts ticked');
  check(!(await exists('[data-panel="projects"] .card.project[data-id="blueprint_archive"]')), 'P2: finished Blueprint Archive hidden');
  await click(box('projects'));
  await sleep(200);
  check(await exists('[data-panel="projects"] .card.project.done[data-id="blueprint_archive"]'), 'P2: unticked shows Blueprint Archive as done');
  await shot('p2_projects_shown');
  // Persists across a reload and a new game.
  await d.load();
  await panel('research');
  check(await evaluate(`document.querySelector('${box('research')}').checked`), 'P2: Research ticked after reload');
  await panel('projects');
  check(!(await evaluate(`document.querySelector('${box('projects')}').checked`)), 'P2: Projects unticked after reload');

  // P1: fixed slots in the status bar.
  const xs = () => evaluate(`[...document.querySelectorAll('footer.status > *')].map((e) => Math.round(e.getBoundingClientRect().left)).join(',')`);
  const before = await xs();
  await evaluate(`(() => {
    document.getElementById('day').textContent = 'Day 1,234,567';
    for (const e of document.querySelectorAll('.status-rate')) e.textContent = '+1,234.5/day';
  })()`);
  const wide = await xs();
  await evaluate(`(() => { for (const e of document.querySelectorAll('.status-rate')) e.textContent = '0/day'; document.getElementById('day').textContent = 'Day 1'; })()`);
  const narrow = await xs();
  check(before === wide && wide === narrow, `P1: status bar slots do not move (${before} | ${wide} | ${narrow})`);
  await d.load();

  // P1: Energy at its cap reads no "capped" in the status bar (scenario at_cap).
  await debugMenu();
  await setValue('#scenario', 'at_cap');
  await click('#start-scenario');
  await sleep(800);
  for (let i = 0; i < 40 && (await exists('[data-testid="story-next"]')); i++) await click('[data-testid="story-next"]');
  const bar = await evaluate(`document.querySelector('[data-testid="energy-rate"]').textContent`);
  const res = await evaluate(`document.querySelector('tr[data-res="energy"] [data-capped]')?.textContent ?? ''`);
  check(/^\+[\d.,]+\/day$/.test(bar) && /cap/.test(res), `P1: status bar Energy "${bar}" (Resources still "${res}")`);
  await shot('p1_status_bar');

  // P4: retire (late_unlocks again: Retirement open) and read the backdrop.
  await setValue('#scenario', 'late_unlocks');
  await click('#start-scenario');
  await sleep(500);
  await speed(0);
  for (let i = 0; i < 40 && (await exists('[data-testid="story-next"]')); i++) await click('[data-testid="story-next"]');
  await cheats();
  await grant('credits', 5000);
  await speed(1);
  await sleep(2500);
  await speed(0);
  for (let i = 0; i < 40 && (await exists('[data-testid="story-next"]')); i++) await click('[data-testid="story-next"]');
  await speed(0);
  const oldDay = await d.day();
  const oldRun = await d.run();
  const oldCredits = await evaluate(`document.querySelector('tr[data-res="credits"] td.num')?.textContent ?? ''`);
  await panel('retirement');
  check(await evaluate(`!document.querySelector('[data-testid="retire"]').disabled`), 'Retire enabled');
  await click('[data-testid="retire"]');
  await waitFor(`!!document.querySelector('[data-testid="retire-confirm"]')`, 'the confirm', 3000);
  await click('[data-testid="retire-confirm"]');
  await sleep(500);
  console.log('     after retire:', await evaluate(`JSON.stringify({ notice: document.getElementById('notice')?.textContent, run: document.getElementById('run')?.textContent, modal: !!document.querySelector('.modal'), story: !!document.querySelector('[data-testid="story-modal"]') })`), JSON.stringify(b.errors));
  await shot('debug_retire');
  await waitFor(`!!document.querySelector('[data-testid="summary"]')`, 'the summary', 5000);
  await sleep(300);
  const backDay = await d.day();
  const backRun = await d.run();
  const backCredits = await evaluate(`document.querySelector('tr[data-res="credits"] td.num')?.textContent ?? ''`);
  check(backDay === oldDay && backRun === oldRun, `P4: backdrop keeps day ${oldDay}, run ${oldRun} (shows day ${backDay}, run ${backRun})`);
  check(backCredits === oldCredits, `P4: backdrop credits ${backCredits} (were ${oldCredits})`);
  const heads = await evaluate(`[...document.querySelectorAll('[data-testid="summary"] .fold-head')].map((e) => e.textContent.replace(/^[▼▶]\s*/, '').trim())`);
  check(heads.some((h) => h.startsWith(`Run ${oldRun}`) && !h.includes('starts')) && heads.some((h) => h.startsWith(`Run ${oldRun + 1} starts with`)), `P4: headings ${JSON.stringify(heads)}`);
  const rows = await evaluate(`[...document.querySelectorAll('[data-testid="summary"] tr')].map((r) => r.textContent.trim())`);
  console.log('     summary rows:', JSON.stringify(rows));
  const delta = await evaluate(`[...document.querySelectorAll('[data-testid="summary-delta"]')].map((e) => [e.textContent.trim(), e.className, getComputedStyle(e).color])`);
  check(delta.length > 0, `#3149: deltas ${JSON.stringify(delta)}`);
  await shot('p4_summary_backdrop');
  await click('[data-testid="continue"]');
  await sleep(500);
  check((await d.run()) === oldRun + 1, `P4: after Continue the new run shows (run ${await d.run()}, day ${await d.day()})`);
  await shot('p4_after_continue');
  check(b.errors.length === 0, `no page errors ${JSON.stringify(b.errors)}`);
} catch (e) {
  console.log('FAIL', e.message);
  fails++;
} finally {
  await b.close();
}
console.log(fails ? `${fails} failed` : 'all ok');
process.exit(fails ? 1 : 0);
