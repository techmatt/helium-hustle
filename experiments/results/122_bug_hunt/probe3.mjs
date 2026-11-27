// 122 probe 3: popups across a retirement. Scenario late_unlocks at 10x, every popup
// logged (id, kind, the speed it held, the speed after OK); retire by the panel, the
// summary's Continue, run 2 at 10x: with "Hide popups for events you've already seen" on
// (the default) run 1's events must not pop up again, yet must be listed. Then a reload
// mid-run, and the speed after each popup.
// Run from the repo root: node experiments/results/122_bug_hunt/probe3.mjs <url> <seconds per run>
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { capture, driver, launch, sleep } from '../../../web/scripts/cdp.mjs';

const [url = 'http://localhost:49373/', secs = '25'] = process.argv.slice(2);
const out = 'out/122_bug_hunt/probe3';
mkdirSync(out, { recursive: true });
const b = await launch({ width: 1280, height: 900, profileRoot: resolve('out/122_bug_hunt/profiles') });
const d = driver(b, { url });
const log = [];
const say = (s) => {
  log.push(s);
  console.log(s);
};
const pressed = () => b.evaluate(`document.querySelector('#speeds button[aria-pressed="true"]')?.dataset.speed ?? '?'`);
let n = 0;
/** Plays at `speed` for `ms`, reading each popup as it comes. */
const play = async (speed, ms, tag) => {
  await d.speed(speed);
  const seen = [];
  for (const end = Date.now() + ms; Date.now() < end; ) {
    if (await d.exists('[data-testid="summary"]')) return seen;
    if (await d.exists('[data-testid="story-modal"]')) {
      const m = await b.evaluate(`(() => { const m = document.querySelector('[data-testid="story-modal"]'); return { quest: m.dataset.quest, kind: m.dataset.kind, title: m.querySelector('h2,h3')?.textContent ?? '' }; })()`);
      const held = await pressed();
      if (n < 12) await capture(b, `${out}/${String(n++).padStart(2, '0')}-${tag}-${m.quest}.png`);
      await sleep(600);
      await d.click('[data-testid="story-next"]');
      await sleep(150);
      const after = await pressed();
      seen.push(`${m.kind}:${m.quest} "${m.title}" held ${held} after ${after}`);
      if (!(await d.exists('[data-testid="story-modal"]')) && after !== String(speed)) say(`  !! after ${m.quest} the speed is ${after}, was ${speed}`);
      continue;
    }
    await sleep(150);
  }
  await d.speed(0);
  return seen;
};
const events = () => b.evaluate(`[...document.querySelectorAll('[data-event]')].map((x) => x.dataset.event).join(',')`);
try {
  await d.load();
  await d.debugMenu();
  await d.speed(0);
  await d.setValue('#scenario', 'late_unlocks');
  await d.click('#start-scenario');
  await d.readStory();
  say(`run 1 popups: ${JSON.stringify(await play(10, Number(secs) * 1000, 'r1'), null, 1)}`);
  say(`run 1 events listed: ${await events()} day ${await d.day()}`);
  await d.panel('retirement');
  await d.click('[data-testid="retire"]');
  await d.click('[data-testid="retire-confirm"]');
  await d.waitFor(`!!document.querySelector('[data-testid="summary"]')`, 'the summary', 5000);
  await capture(b, `${out}/summary.png`);
  say(`summary up; story modal too: ${await d.exists('[data-testid="story-modal"]')}`);
  await d.click('[data-testid="continue"]');
  await sleep(300);
  say(`after Continue: speed ${await pressed()}, modal ${await d.exists('[data-testid="story-modal"]')}`);
  if (await d.exists('[data-testid="story-modal"]')) say(`  pages: ${JSON.stringify(await d.readStory())}; speed then ${await pressed()}`);
  say(`run 2 popups: ${JSON.stringify(await play(10, Number(secs) * 1000, 'r2'), null, 1)}`);
  say(`run 2 events listed: ${await events()} day ${await d.day()} run ${await d.run()}`);
  await d.speed(3);
  await d.load();
  await sleep(500);
  say(`after reload: speed ${await pressed()}, modal ${await d.exists('[data-testid="story-modal"]')}, events ${await events()}`);
  await capture(b, `${out}/reloaded.png`);
  say(`errors: ${JSON.stringify(b.errors)}`);
} catch (e) {
  say(`FAILED: ${e.stack}`);
  await capture(b, `${out}/failed.png`).catch(() => {});
} finally {
  writeFileSync(`${out}/notes.txt`, log.join('\n') + '\n');
  b.close();
}
