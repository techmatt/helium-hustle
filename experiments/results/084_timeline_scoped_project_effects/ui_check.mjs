// 084: the done-when in a debug career: scenario story_acts, jumped past the swarm's
// arrival (a timeline's end), then the global projects re-funded from the Projects panel.
// Usage (from the repo root, `vite preview` serving a build on HH_URL):
//   HH_URL=http://localhost:<port>/ node experiments/results/084_timeline_scoped_project_effects/ui_check.mjs
// Shots and the browser profile go to out/084_ui_check/.
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { mkdirSync } from 'node:fs';
import { capture, driver, launch, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = process.env.HH_URL;
const scratch = resolve(here, '../../../out/084_ui_check');
const shots = resolve(scratch, 'shots');
mkdirSync(shots, { recursive: true });
const b = await launch({ width: 1600, height: 900, profileRoot: resolve(scratch, 'profile') });
const { evaluate } = b;
const { exists, click, setValue, load, speed, panel, debugMenu, cheats, grant, autoStory } = driver(b, { url, settle: 700 });
const failures = [];
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) failures.push(what);
};
const shot = (name) => capture(b, resolve(shots, `${name}.png`));
const strip = () => evaluate(`document.querySelector('[data-testid="calendar-strip"]')?.textContent ?? ''`);
const project = (id) =>
  evaluate(`(() => { const c = document.querySelector('.card.project[data-id="${id}"]'); return c ? { cls: c.className, text: c.textContent } : null; })()`);
const researchDone = (id) => evaluate(`document.querySelector('[data-testid="research-${id}"]')?.classList.contains('done') ?? null`);

/** Whether the Programs view's All instructions offers an `if` line (Text mode). */
const ifOffered = async () => {
  await click('.nav-tile[data-nav="programs"]');
  await sleep(300);
  if (!(await exists('[data-testid="text-editor"]')) && (await exists('[data-testid="mode-text"]'))) await click('[data-testid="mode-text"]');
  await sleep(300);
  return evaluate(`[...document.querySelectorAll('[data-testid="instructions"] [data-instruction]')].some((b) => /^if\\b/.test(b.dataset.instruction))`);
};
const closeSummary = async () => {
  for (let i = 0; i < 5 && (await exists('[data-testid="summary"]')); i++) {
    await click('[data-testid="continue"]', 'continue');
    await sleep(200);
  }
};
const jump = async (days) => {
  await closeSummary();
  await cheats();
  await setValue('#cheat-days', String(days));
  await click('#cheat-jump', 'jump');
  await sleep(300);
  await closeSummary();
};

await load(url);
await debugMenu();
await speed(0);
await setValue('#scenario', 'story_acts');
await click('#start-scenario');
await autoStory();
await speed(0);
await cheats();
console.log(`     start: ${await strip()}`);
for (let i = 0; i < 40 && !/Timeline 2/.test(await strip()); i++) {
  await jump(1000);
  console.log(`     ${await strip()}`);
}
check(/Timeline 2/.test(await strip()), `a timeline ended (${await strip()})`);
await closeSummary();
await panel('projects');
for (const id of ['conditional_logic', 'blueprint_archive', 'deep_space_telescope']) {
  const p = await project(id);
  check(!!p && !/done|locked/.test(p.cls) && !/Complete/.test(p.text), `${id} is offered again (${p?.cls})`);
}
await shot('084_projects_after_timeline_end');
await panel('research');
check((await researchDone('basic_blueprints')) === false && (await researchDone('advanced_blueprints')) === false, 'the Blueprints are not pre-bought');
check((await ifOffered()) === false, '`if` is locked: All instructions offers no `if`');
// Fund Conditional Logic and the Blueprint Archive at the most, with resources granted.
await cheats();
await grant('credits', 5000);
await grant('science', 1000);
await panel('projects');
for (const id of ['conditional_logic', 'blueprint_archive']) {
  const n = await evaluate(`(() => { const bs = [...document.querySelectorAll('.card.project[data-id="${id}"] button[title^="Fund at the most"]')]; bs.forEach((x) => x.click()); return bs.length; })()`);
  await sleep(200);
  check(n === 2, `${id}: both costs funded at the most`);
}
await jump(20);
await panel('projects');
for (const id of ['conditional_logic', 'blueprint_archive']) {
  const p = await project(id);
  check(!!p && /Complete/.test(p.text), `${id} completes when re-funded`);
}
await panel('research');
check((await researchDone('basic_blueprints')) === true && (await researchDone('advanced_blueprints')) === true, 'the Blueprints are bought again');
check((await ifOffered()) === true, '`if` is open again: All instructions offers `if`');
await shot('084_if_open_again');
await shot('084_projects_refunded');
await panel('stats');
const rewards = await evaluate(`[...document.querySelectorAll('[data-testid="live-rewards"] li')].map((li) => li.textContent)`);
console.log(`     live rewards: ${JSON.stringify(rewards)}`);
check(rewards.some((t) => /Opens if/.test(t)) && rewards.some((t) => /Blueprints researched/.test(t)), 'Stats lists the two rewards again');
console.log(failures.length ? `ui check: ${failures.length} failed` : 'ui check ok');
await b.close?.();
process.exit(failures.length ? 1 : 0);
