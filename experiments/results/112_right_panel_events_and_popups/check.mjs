// 112's done-when checks against a `vite preview` build (HH_URL). Screenshots to SHOTS.
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { capture, driver, launch, pageUrl, sleep } from '../../../web/scripts/cdp.mjs';

const url = pageUrl('check');
const SHOTS = resolve(process.env.SHOTS ?? 'out/112_right_panel_events_and_popups/shots');
mkdirSync(SHOTS, { recursive: true });
const b = await launch({ width: 1600, height: 900, profileRoot: resolve('out/112_right_panel_events_and_popups/profile') });
const { evaluate } = b;
const d = driver(b, { url, settle: 500 });
const { waitFor, exists, click, setValue, load, debugMenu, cheats, readStory, panel } = d;
const results = [];
const check = (ok, what) => {
  results.push(`${ok ? 'PASS' : 'FAIL'} ${what}`);
  console.log(`${ok ? 'PASS' : 'FAIL'} ${what}`);
};
const shot = (name) => capture(b, resolve(SHOTS, `112_${name}.png`));
const modal = () => evaluate(`(() => { const m = document.querySelector('[data-testid="story-modal"]'); return m ? { quest: m.dataset.quest, kind: m.dataset.kind, text: m.querySelector('.story-text').textContent, mech: [...m.querySelectorAll('.mechanics')].map((x) => x.textContent).join(' | ') || null, button: m.querySelector('[data-testid="story-next"]').textContent } : null; })()`);
const events = () => evaluate(`[...document.querySelectorAll('[data-testid="events-list"] [data-event]')].map((e) => ({ id: e.dataset.event, text: e.textContent, mech: !!e.querySelector('.mechanics') }))`);
const pilotDone = () => waitFor(`document.querySelector('#accel-status')?.dataset.running === '0'`, 'the autopilot to stop', 120000);

await load(url);
await readStory();

// ---- 1. Propellant discovery on a fresh career: a popup, then one line in Events ----
// The autopilot plays to 1.4 (its popups are skipped while it drives), then the game runs on.
const STEPS = Number(process.env.STEPS ?? 2);
for (let i = 0; i < STEPS; i++) {
  await cheats();
  await click('#accel-quest');
  await sleep(500);
  await pilotDone();
  await readStory();
  const q = await evaluate(`document.querySelector('[data-testid="quest"]')?.dataset.quest ?? null`);
  const shipped = await evaluate(`document.querySelectorAll('.activity li[data-kind="shipment"]').length`);
  console.log(`after step ${i + 1}: quest ${q}, shipments listed ${shipped}, events ${JSON.stringify((await events()).map((e) => e.id))}`);
}
const before = await evaluate(`document.querySelector('[data-testid="quest"]')?.dataset.quest ?? null`);
console.log('quest after the autopilot:', before, 'events:', JSON.stringify(await events()));
let found = null;
if ((await events()).some((e) => e.id === 'propellant_discovery')) {
  console.log('propellant_discovery fired under the autopilot: no popup to see; stepping on to the next quest');
} else {
  // The agent's programs play on at the fastest speed offered; every popup on the way is read.
  const fastest = async () => {
    const sp = await evaluate(`[...document.querySelectorAll('#speeds button[data-speed]')].map((b) => Number(b.dataset.speed)).filter((x) => !b0(x)).sort((a, b) => b - a)[0]; function b0(x) { return x === 0; }`);
    await click(`#speeds button[data-speed="${sp}"]`);
  };
  await fastest();
  for (let t = 0; t < 400 && !found; t++) {
    const m = await modal();
    if (m && m.quest === 'propellant_discovery') found = m;
    else if (m) {
      console.log('popup on the way:', m.quest);
      await click('[data-testid="story-next"]');
      await fastest();
    } else await sleep(400);
    if (t % 25 === 24) {
      const st = await evaluate(`({ day: document.getElementById('day').textContent, ships: document.querySelectorAll('.activity li[data-kind="shipment"]').length, quest: document.querySelector('[data-testid="quest"]')?.dataset.quest })`);
      console.log('waiting:', JSON.stringify(st));
      if (await exists('[data-testid="summary"]')) break;
    }
  }
}
if (found) {
  await shot('propellant_popup');
  check(/Effect/.test(found.mech ?? '') && found.button === 'OK', `Propellant discovery pops up with its Effect: ${JSON.stringify(found)}`);
  await click('[data-testid="story-next"]');
  await sleep(300);
  const list = await events();
  const line = list.find((e) => e.id === 'propellant_discovery');
  check(!!line && !line.mech, `then sits as a one-line entry under Events (no Effect): ${JSON.stringify(line)}`);
  await shot('propellant_events_line');
  await click('[data-testid="events-list"] [data-event="propellant_discovery"]');
  await sleep(300);
  check((await modal())?.quest === 'propellant_discovery', 'clicking the line reopens its popup');
  await click('[data-testid="story-next"]');
} else {
  check(false, 'Propellant discovery popup not seen (see log)');
}
check(!(await exists('[data-testid="events-completed"]')) && !(await evaluate(`/Completed \\(/.test(document.querySelector('.col-right').textContent)`)), 'no Completed section');

// ---- 2. A side quest: listed while open, a reward popup on completion, then gone ----
await debugMenu();
await setValue('#scenario', 'late_unlocks');
await click('#start-scenario');
await sleep(800);
await readStory();
await click('#speeds button[data-speed="1"]');
await sleep(1000);
await click('#speeds button[data-speed="0"]');
await readStory();
const open = await evaluate(`[...document.querySelectorAll('[data-testid="side-quests"] [data-side-quest]')].map((e) => e.dataset.sideQuest)`);
await shot('side_quests_open');
console.log('open side quests:', open);
if (open.length) {
  await click(`[data-side-quest="${open[0]}"]`);
  await sleep(300);
  const p = await modal();
  check(p?.quest === open[0] && /Goal/.test(p.mech ?? ''), `a side quest's click opens its popup (story, goal, reward): ${JSON.stringify(p)}`);
  await shot('side_quest_popup');
  await click('[data-testid="story-next"]');
}
await cheats();
await click('#cheat-complete-research');
await click('#speeds button[data-speed="1"]');
let side = null;
for (let t = 0; t < 60 && !side; t++) {
  const m = await modal();
  if (m?.quest === 'qfull_cluster') side = m;
  else if (m) await click('[data-testid="story-next"]');
  else await sleep(300);
}
if (side) {
  const listed = await exists('[data-side-quest="qfull_cluster"]');
  await shot('side_quest_done_popup');
  check(side.kind === 'payoff' && /Reward/.test(side.mech ?? '') && !listed, `1.b's completion pops up its reward and leaves the right panel: ${JSON.stringify({ side, listed })}`);
  await click('[data-testid="story-next"]');
} else check(false, '1.b completion popup not seen');
check(!(await exists('[data-testid="story-banner"]')), 'no side-quest banner');

// ---- 3. Burnout pops in run 1; with the checkbox on, nothing boredom pops in run 2 ----
await debugMenu();
// A new career (the debug menu's New game) so no popup has been seen yet.
const fresh = await exists('#new-game');
if (fresh) await click('#new-game');
await sleep(500);
await readStory();
// The checkbox lives in Options since 116.
await panel('options');
check(await evaluate(`document.querySelector('[data-testid="hide-seen-popups"]')?.checked === true`), 'the checkbox starts on');
await cheats();
await setValue('#cheat-days', '205');
await click('#cheat-jump');
await sleep(500);
const run1 = [];
for (let i = 0; i < 10 && (await exists('[data-testid="story-modal"]')); i++) {
  const m = await modal();
  run1.push(m.quest);
  if (m.quest === 'boredom_bites') await shot('burnout_popup');
  await click('[data-testid="story-next"]');
}
// Since 113, Boredom Rising is skipped in the run Burnout fires in (no popup, no line).
check(run1.includes('boredom_bites') && !run1.includes('boredom_phase_2'), `run 1: Burnout pops up, Boredom Rising skipped: ${run1}`);
const ev1 = (await events()).map((e) => e.id);
check(ev1.includes('boredom_bites') && !ev1.includes('boredom_phase_2'), `run 1's Events: Burnout stands for the day (phase 2 superseded): ${ev1}`);
await shot('run1_events');
// Retire at the cap.
await cheats();
await setValue('#cheat-boredom', '100000');
await click('#cheat-set-boredom');
await click('#speeds button[data-speed="1"]');
await waitFor(`!!document.querySelector('[data-testid="summary"]')`, 'the run summary', 20000);
await click('[data-testid="continue"]');
await sleep(500);
await readStory();
await cheats();
await setValue('#cheat-days', '205');
await click('#cheat-jump');
await sleep(500);
const run2 = [];
for (let i = 0; i < 10 && (await exists('[data-testid="story-modal"]')); i++) {
  run2.push((await modal()).quest);
  await click('[data-testid="story-next"]');
}
const ev2 = (await events()).map((e) => e.id);
const run = await evaluate(`document.getElementById('run').textContent`);
// Burnout fires once a career; Boredom Rising, first fired here (113), pops up once.
check(!run2.includes('boredom_bites') && ev2.includes('boredom_phase_2'), `${run}: no Burnout popup, Events lists Boredom Rising: popups ${run2}, events ${ev2}`);
await shot('run2_events');
console.log(results.join('\n'));
console.log('page errors:', JSON.stringify(b.errors));
await b.close();
process.exit(0);
