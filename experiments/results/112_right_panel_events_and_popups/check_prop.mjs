// 112: Propellant discovery on a fresh career: a popup when it fires, then one line in Events.
// The autopilot plays to 1.3 (one shipment), then shipments by hand with cheats (grants
// and jumps), every popup on the way read, until the discovery's popup opens.
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { capture, driver, launch, pageUrl, sleep } from '../../../web/scripts/cdp.mjs';

const url = pageUrl('check');
const SHOTS = resolve('out/112_right_panel_events_and_popups/shots');
mkdirSync(SHOTS, { recursive: true });
const b = await launch({ width: 1600, height: 900, profileRoot: resolve('out/112_right_panel_events_and_popups/profile') });
const { evaluate } = b;
const d = driver(b, { url, settle: 500 });
const { waitFor, exists, click, setValue, load, cheats, readStory, panel, grant, runOnly } = d;
const shot = (name) => capture(b, resolve(SHOTS, `112_${name}.png`));
const modal = () => evaluate(`(() => { const m = document.querySelector('[data-testid="story-modal"]'); return m ? { quest: m.dataset.quest, kind: m.dataset.kind, text: m.querySelector('.story-text').textContent, mech: [...m.querySelectorAll('.mechanics')].map((x) => x.textContent).join(' | ') || null, button: m.querySelector('[data-testid="story-next"]').textContent } : null; })()`);
const events = () => evaluate(`[...document.querySelectorAll('[data-testid="events-list"] [data-event]')].map((e) => ({ id: e.dataset.event, text: e.textContent, mech: !!e.querySelector('.mechanics') }))`);
let found = null;
/** Reads every popup open now; keeps the discovery's. */
const readAll = async () => {
  for (let i = 0; i < 20; i++) {
    const m = await modal();
    if (!m) return;
    if (m.quest === 'propellant_discovery' && !found) {
      found = m;
      await shot('propellant_popup');
      return;
    }
    console.log('popup on the way:', m.quest);
    await click('[data-testid="story-next"]');
  }
};

await load(url);
await readStory();
for (let i = 0; i < 2; i++) {
  await cheats();
  await click('#accel-quest');
  await sleep(500);
  await waitFor(`document.querySelector('#accel-status')?.dataset.running === '0'`, 'the autopilot to stop', 120000);
  await readAll();
}
console.log('events after the autopilot:', JSON.stringify((await events()).map((e) => e.id)));
await panel('launch_pads');
await setValue('[data-pad="0"] select', 'he3');
const lines = await evaluate(`[...document.querySelectorAll('.lines li')].map((li) => li.dataset.command)`);
await runOnly('load_pads', lines);
for (let ship = 0; ship < 6 && !found; ship++) {
  for (let i = 0; i < 6 && !found; i++) {
    await cheats();
    await grant('he3', '20');
    await grant('propellant', '30');
    await grant('energy', '100');
    await setValue('#cheat-days', '4');
    await click('#cheat-jump');
    await sleep(150);
    await readAll();
  }
  if (found) break;
  await panel('launch_pads');
  const pad = await evaluate(`(() => { const p = document.querySelector('[data-pad="0"]'); return { cargo: p.dataset.cargo, status: p.querySelector('.pad-status').textContent }; })()`);
  console.log(`ship ${ship + 1}: pad`, JSON.stringify(pad));
  if (await exists('[data-testid="launch-0"]:not([disabled])')) await click('[data-testid="launch-0"]');
  await sleep(200);
  await readAll();
  await cheats();
  await setValue('#cheat-days', '2');
  await click('#cheat-jump');
  await sleep(150);
  await readAll();
  console.log('shipments in Recent Activity:', await evaluate(`document.querySelectorAll('.activity li[data-kind="shipment"]').length`));
}
const out = [];
if (found) {
  out.push(`${/Effect/.test(found.mech ?? '') ? 'PASS' : 'FAIL'} Propellant discovery pops up with its Effect: ${JSON.stringify(found)}`);
  await click('[data-testid="story-next"]');
  await readAll();
  await sleep(300);
  const line = (await events()).find((e) => e.id === 'propellant_discovery');
  out.push(`${line && !line.mech ? 'PASS' : 'FAIL'} then one line under Events (title and summary, no Effect): ${JSON.stringify(line)}`);
  await shot('propellant_events_line');
  await click('[data-testid="events-list"] [data-event="propellant_discovery"]');
  await sleep(300);
  out.push(`${(await modal())?.quest === 'propellant_discovery' ? 'PASS' : 'FAIL'} clicking the line reopens its popup`);
  await click('[data-testid="story-next"]');
} else out.push('FAIL Propellant discovery popup not seen');
console.log(out.join('\n'));
console.log('page errors:', JSON.stringify(b.errors));
await b.close();
process.exit(0);
