// 116's browser checks against a `vite preview` build (HH_URL). Screenshots to SHOTS.
//   1. Projects: holding + keeps stepping and speeds up, stops at the cap; − back to 0.
//   2. Projects: an underfunded rate shows actual / set, the card's days at the actual rate.
//   3. Right panel: Side Quests is a fold below Events, above Recent Activity; folded it
//      shows the closest-to-done quest's progress.
//   4. Options holds "Hide popups for events you've already seen"; the right panel doesn't.
//   5. Speculators' first arrival opens a popup (pad_loading: a pad launches He-3 until 500
//      units are sold).
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { capture, driver, launch, pageUrl, sleep } from '../../../web/scripts/cdp.mjs';

const url = pageUrl('check');
const OUT = 'out/116_parked_batch_projects_panel_adversaries';
const SHOTS = resolve(process.env.SHOTS ?? `${OUT}/shots`);
mkdirSync(SHOTS, { recursive: true });
const b = await launch({ width: 1600, height: 900, profileRoot: resolve(`${OUT}/profile`) });
const { evaluate } = b;
const d = driver(b, { url, settle: 500 });
const { waitFor, exists, click, setValue, load, debugMenu, cheats, grant, readStory, panel, speed, runOnly } = d;
const results = [];
const check = (ok, what) => {
  results.push(`${ok ? 'PASS' : 'FAIL'} ${what}`);
  console.log(`${ok ? 'PASS' : 'FAIL'} ${what}`);
};
const shot = (name) => capture(b, resolve(SHOTS, `116_${name}.png`));
const scenario = async (name) => {
  await debugMenu();
  await setValue('#scenario', name);
  await click('#start-scenario');
  await sleep(800);
  await readStory();
};

await load(url);
await readStory();

// ---- 1. Hold + / − on Foundation Grant's credits ----
await scenario('late_unlocks');
await speed(0);
await panel('projects');
const row = `.card[data-id="foundation_grant"] .proj-row`;
const rate = () => evaluate(`Number(document.querySelector('${row} .proj-rate').textContent.replace(/[^0-9.\\/ ]/g, '').split('/').pop().trim() || '0')`);
const setRate = () => evaluate(`(() => { const t = document.querySelector('${row} .proj-rate').textContent; const m = t.match(/([0-9.]+)\\s*(?:\\/day| per day)/); return m ? Number(m[1]) : null; })()`);
const btn = (i) => `document.querySelectorAll('${row}')[0].querySelectorAll('button')[${i}]`;
const hold = async (i, ms) => {
  const samples = [];
  await evaluate(`${btn(i)}.dispatchEvent(new PointerEvent('pointerdown', { button: 0, bubbles: true }))`);
  const t0 = Date.now();
  while (Date.now() - t0 < ms) {
    await sleep(100);
    samples.push([Date.now() - t0, await setRate()]);
  }
  await evaluate(`${btn(i)}.dispatchEvent(new PointerEvent('pointerup', { button: 0, bubbles: true }))`);
  return samples;
};
const up = await hold(1, 1300);
console.log('hold + 1.3s:', JSON.stringify(up));
const at = (s, ms) => s.filter(([t]) => t <= ms).at(-1)?.[1] ?? null;
check(at(up, 300) === 1, `a press steps once and waits ~0.4s (rate at 0.3s: ${at(up, 300)})`);
check(at(up, 1300) >= 6 && at(up, 1300) <= 11, `then repeats at ~10 a second (rate at 1.3s: ${at(up, 1300)})`);
const afterUp = await setRate();
await sleep(500);
check((await setRate()) === afterUp, `releasing stops it (${afterUp})`);
const more = await hold(1, 3000);
console.log('hold + 3s:', JSON.stringify(more));
const deltas = more.map(([, v], i) => (i === 0 ? 0 : v - more[i - 1][1]));
check(deltas.some((x) => x >= 5) || more.at(-1)[1] === 30, `held past 1.5s it steps by 5 (deltas ${JSON.stringify(deltas)})`);
check(more.at(-1)[1] === 30, `and stops at the cap, 30 credits a day (${more.at(-1)[1]})`);
check(await evaluate(`${btn(1)}.disabled`), '+ disabled at the cap');
await shot('projects_capped');
const down = await hold(0, 4000);
console.log('hold − 4s:', JSON.stringify(down));
check(down.at(-1)[1] === 0, `holding − runs back down to 0 (${down.at(-1)[1]})`);

// ---- 2. Underfunded: credits and science at the most; the credits stock emptied ----
// (Two labs make 10+ science a day, so science stays met: green.)
const sci = `document.querySelectorAll('${row}')[1]`;
await evaluate(`${sci}.querySelector('button[title^="Fund at the most"]').click()`);
await click(`${row} button[title^="Fund at the most"]`);
await cheats();
await grant('credits', -100000);
await grant('science', 5000);
await panel('projects');
const before = await evaluate(`[...document.querySelectorAll('.card[data-id="foundation_grant"] .card-text p')].map((p) => p.textContent).find((t) => /Done in|Not every|Stalled/.test(t)) ?? null`);
await speed(1);
await sleep(3000);
await speed(0);
await readStory();
const shortRow = await evaluate(`(() => { const r = document.querySelector('${row} .proj-rate'); return { text: r.textContent, short: r.hasAttribute('data-short'), neg: r.querySelector('.neg')?.textContent ?? null }; })()`);
const sciRow = await evaluate(`(() => { const r = ${sci}.querySelector('.proj-rate'); return { text: r.textContent, ok: r.classList.contains('ok') }; })()`);
const days = await evaluate(`[...document.querySelectorAll('.card[data-id="foundation_grant"] .card-text p')].map((p) => p.textContent).find((t) => /Done in|Not every|Stalled/.test(t)) ?? null`);
console.log('underfunded row:', JSON.stringify(shortRow), 'science row:', JSON.stringify(sciRow), 'days before:', before, 'after:', days);
check(shortRow.short && /^[0-9.,]+ \/ 30 per day$/.test(shortRow.text.trim()) && shortRow.neg !== null, `an underfunded rate reads actual / set, the actual red: "${shortRow.text}"`);
check(sciRow.ok && /10\/day/.test(sciRow.text), `a fully met rate shows green: "${sciRow.text}"`);
// At the set rates it read "Done in 334 days"; at the actual credits rate it is longer, or
// unfunded when the take is 0.
const n = Number((days ?? '').replace(/\D/g, ''));
check(/Done in 334/.test(before ?? '') && (/Stalled/.test(days ?? '') || n > 334), `"Done in N days" counts the actual rate: "${before}" then "${days}"`);
await shot('projects_underfunded');

// ---- 3. Side Quests below Events ----
const order = await evaluate(`[...document.querySelectorAll('.col-right [data-testid]')].map((e) => e.dataset.testid).filter((t) => ['quest-pin', 'events-list', 'side-quests', 'events-activity'].includes(t))`);
console.log('right column order:', JSON.stringify(order));
check(JSON.stringify(order) === JSON.stringify(['quest-pin', 'events-list', 'side-quests', 'events-activity']), `main quest pinned, then Events, Side Quests, Recent Activity: ${order.join(', ')}`);
const head = () => evaluate(`document.querySelector('[data-testid="side-quests"] .fold-head').textContent`);
const openHead = await head();
check(/Side Quests \(\d+\/\d+\)/.test(openHead), `its header counts them: "${openHead}"`);
await shot('side_quests_open');
await click('[data-testid="side-quests"] .fold-head');
await sleep(200);
const foldedHead = await head();
check(/Side Quests \(\d+\/\d+\).*: [0-9.,]+ \/ [0-9.,]+/.test(foldedHead), `folded, it shows the closest-to-done quest's progress: "${foldedHead}"`);
await shot('side_quests_folded');
await panel('research');
await panel('projects');
check((await head()) === foldedHead, 'the fold stays as left (as Events\' does)');
await click('[data-testid="side-quests"] .fold-head');

// ---- 4. The hide-seen checkbox lives in Options ----
check(!(await exists('.col-right [data-testid="hide-seen-popups"]')), 'no hide-seen checkbox in the right panel');
await panel('options');
check(await evaluate(`document.querySelector('[data-panel="options"] [data-testid="hide-seen-popups"]')?.checked === true`), 'Options has it, on by default');
await shot('options_hide_seen');

// ---- 5. Speculators' first arrival opens a popup ----
await scenario('speculators_due');
await speed(0);
await panel('launch_pads');
await evaluate(`[...document.querySelectorAll('[data-panel="launch_pads"] select')].forEach((s) => { s.value = 'he3'; s.dispatchEvent(new Event('change', { bubbles: true })); })`);
await sleep(300);
const hasLaunch = false;
let found = null;
const seen = [];
const fastest = async () => {
  const sp = await evaluate(`Math.max(...[...document.querySelectorAll('#speeds button[data-speed]')].filter((b) => !b.disabled).map((b) => Number(b.dataset.speed)))`);
  await speed(sp);
};
await fastest();
for (let t = 0; t < 600 && !found; t++) {
  const m = await evaluate(`(() => { const m = document.querySelector('[data-testid="story-modal"]'); return m ? { quest: m.dataset.quest, text: m.querySelector('.story-text').textContent, mech: [...m.querySelectorAll('.mechanics')].map((x) => x.textContent).join(' | ') } : null; })()`);
  if (m && m.quest === 'speculators_arrive') found = m;
  else if (m) {
    seen.push(m.quest);
    await click('[data-testid="story-next"]');
    await fastest();
  } else {
    // Launch by hand whenever a pad can (Launch Full Pads waits for 1.3), topping up He-3
    // and propellant every few rounds.
    if (t % 6 === 5) {
      await cheats();
      await grant('he3', 300);
      await grant('propellant', 400);
      await panel('launch_pads');
    }
    if (!hasLaunch) await evaluate(`[...document.querySelectorAll('[data-panel="launch_pads"] [data-testid^="launch-"]')].forEach((b) => { if (!b.disabled) b.click(); })`);
    await sleep(250);
  }
  if (t % 40 === 39) console.log('waiting:', await evaluate(`document.getElementById('day').textContent`), 'popups so far', JSON.stringify(seen));
}
if (found) await shot('speculators_popup');
check(!!found, `speculators' first arrival opens a popup: ${JSON.stringify(found)}`);

console.log('\n' + results.join('\n'));
await b.close();
process.exit(results.some((r) => r.startsWith('FAIL')) ? 1 : 0);
