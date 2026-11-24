// 086's check of the swarm's clock in the browser, in a debug career: a fresh game shows
// no calendar; the story_acts scenario (1.11 done) shows the ~100-year countdown; retiring
// (5 years each) runs it out, the swarm ends timeline 1, and timeline 2's countdown runs
// from its first day with no Telescope rebuilt, then runs out on schedule too.
// Drives headless Edge or Chrome over the DevTools protocol (web/scripts/cdp.mjs) against
// a served static build.
//   HH_URL  the page, required (a `vite preview`, web/README.md)
//   OUT     where the PNGs go (default experiments/results/086_swarm_clock_after_discovery/shots)
import { mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('shots', 'node experiments/results/086_swarm_clock_after_discovery/shots.mjs');
const OUT = resolve(process.env.OUT ?? resolve(here, 'shots'));
mkdirSync(OUT, { recursive: true });
const b = await launch({ width: 1280, height: 800, emulate: true, profileRoot: resolve(here, '../../../out/086_swarm_clock_after_discovery/profile') });
const { evaluate } = b;
const page = driver(b, { url });
const { waitFor, exists, click, setValue, load, speed, panel, debugMenu, readStory } = page;
const strip = () => evaluate(`document.querySelector('[data-testid="calendar-strip"]')?.textContent ?? null`);
const full = async (name) => {
  await capture(b, resolve(OUT, `${name}.png`));
  console.log(`shot ${name}`);
};
const fail = (msg) => {
  console.error(`FAIL ${msg}`);
  process.exitCode = 1;
};
const parse = (s) => ({
  timeline: Number((s ?? '').match(/Timeline (\d+)/)?.[1]),
  day: Number((s ?? '').match(/day ([\d,]+)/)?.[1].replace(/,/g, '')),
});
const retire = async () => {
  await panel('retirement');
  await click('[data-testid="retire"]', 'retire');
  await click('[data-testid="retire-confirm"]', 'confirm');
  await sleep(300);
  if (await exists('[data-testid="continue"]')) await click('[data-testid="continue"]', 'summary continue');
  await readStory();
};
// Retires until the countdown runs out (a retirement can carry it past the arrival day; the
// swarm comes on that day's tick), then plays until timeline `next` begins.
const runOut = async (next) => {
  let s = await strip();
  for (let i = 0; i < 30 && parse(s).timeline === next - 1 && !/swarm here/.test(s ?? ''); i++) {
    await retire();
    s = await strip();
  }
  console.log(`countdown run out: ${s}`);
  await speed(1);
  for (let i = 0; i < 40 && parse(await strip()).timeline !== next; i++) {
    await readStory();
    await sleep(250);
  }
  await speed(0);
  await readStory();
  return strip();
};

await load();
await speed(0);
const act1 = await strip();
console.log(`fresh game: calendar strip ${JSON.stringify(act1)}`);
if (act1 !== null) fail('a fresh game shows a calendar');

await debugMenu();
await setValue('#scenario', 'story_acts');
await click('#start-scenario');
await speed(1);
await waitFor(`!!document.querySelector('[data-testid="calendar-strip"]')`, 'the calendar strip', 8000);
await sleep(1000);
await speed(0);
console.log(`after 1.11: ${await strip()}`);

// Retire until the swarm ends timeline 1 (each retirement is 5 years of the 100).
let s = await runOut(2);
console.log(`timeline 1 ended: ${s}`);
if (parse(s).timeline !== 2) fail('the swarm never ended timeline 1');
if (!/swarm in [\d.,]+K? days \(\d+\.\d years\)/.test(s ?? '')) fail('no countdown in timeline 2');
if (/locat|rebuild/i.test(s ?? '')) fail('the strip still asks for the Telescope');
const start = parse(s).day;
await speed(1);
await sleep(3000);
await speed(0);
await readStory();
const later = await strip();
console.log(`timeline 2, a moment later: ${later}`);
if (!(parse(later).day > start)) fail('timeline 2 countdown does not run without the Telescope');
await panel('projects');
const telescope = await evaluate(`document.querySelector('[data-panel="projects"]')?.textContent.includes('Deep Space Telescope') ?? null`);
console.log(`Projects panel offers the Deep Space Telescope again: ${telescope}`);
await panel('timeline');
await full('086_timeline2_countdown_without_telescope');

s = await runOut(3);
console.log(`timeline 2 ended: ${s}`);
if (parse(s).timeline !== 3) fail('the swarm never ended timeline 2');
if (await exists('[data-testid="run-summary"]')) console.log('(run summary open)');
await full('086_timeline3_countdown');
await b.close();
