// 085's check of the swarm's clock in the browser: a fresh game shows no calendar strip;
// the story_acts scenario (1.11 done, the Telescope standing) shows a ~100-year countdown;
// a retirement moves it 5 years (1,825 days) plus the day. Drives headless Edge or Chrome
// over the DevTools protocol (web/scripts/cdp.mjs) against a served static build.
//   HH_URL  the page, required (a `vite preview`, web/README.md)
//   OUT     where the PNGs go (default experiments/results/085_swarm_clock_and_retirement_gap/shots)
import { mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('shots', 'node experiments/results/085_swarm_clock_and_retirement_gap/shots.mjs');
const OUT = resolve(process.env.OUT ?? resolve(here, 'shots'));
mkdirSync(OUT, { recursive: true });
const b = await launch({ width: 1280, height: 800, emulate: true, profileRoot: resolve(here, '../../../out/085_swarm_clock_and_retirement_gap/profile') });
const { evaluate } = b;
const page = driver(b, { url });
const { waitFor, exists, click, setValue, load, speed, panel, debugMenu } = page;
const strip = () => evaluate(`document.querySelector('[data-testid="calendar-strip"]')?.textContent ?? null`);
const full = async (name) => {
  await capture(b, resolve(OUT, `${name}.png`));
  console.log(`shot ${name}`);
};
const fail = (msg) => {
  console.error(`FAIL ${msg}`);
  process.exitCode = 1;
};

await load();
await speed(0);
const act1 = await strip();
console.log(`fresh game: calendar strip ${JSON.stringify(act1)}`);
if (act1 !== null) fail('a fresh game shows a calendar');
await full('085_act1_no_calendar');

await debugMenu();
await setValue('#scenario', 'story_acts');
await click('#start-scenario');
await speed(1);
await waitFor(`!!document.querySelector('[data-testid="calendar-strip"]')`, 'the calendar strip', 8000);
await sleep(1500);
await speed(0);
const before = await strip();
console.log(`after 1.11: ${before}`);
if (!/swarm in [\d.,]+K? days \(\d+\.\d years\)/.test(before ?? '')) fail('no countdown after 1.11');
await panel('timeline');
await full('085_countdown_after_telescope');

await panel('retirement');
await click('[data-testid="retire"]', 'retire');
await click('[data-testid="retire-confirm"]', 'confirm');
await sleep(500);
const after = await strip();
console.log(`after a retirement: ${after}`);
const day = (s) => Number((s ?? '').match(/day ([\d,]+)/)?.[1].replace(/,/g, ''));
const moved = day(after) - day(before);
console.log(`the retirement moved the countdown ${moved} days`);
if (moved < 1825 || moved > 1827) fail(`a retirement moved it ${moved} days, not 1,825 (+ a day)`);
if (await exists('[data-testid="run-summary"]')) console.log('(run summary open)');
await full('085_after_retirement');
await b.close();
