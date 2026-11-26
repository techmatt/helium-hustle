// 105's check of quest mechanics text in the running game (vite preview at HH_URL): every
// Story card, main or side, active or completed, shows its goal and reward in the mechanics
// strip, completed side quest 1.b (Full Cluster) included, and no narrative line on a card
// carries a number. From the repo root, a static build served:
//   HH_URL=http://localhost:<port>/ node experiments/results/105_quest_mechanics_text/check.mjs out/105_quest_mechanics_text/shots
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { launch, driver, capture, sleep } from '../../../web/scripts/cdp.mjs';

const url = process.env.HH_URL;
const shots = resolve(process.argv[2] ?? 'out/105_quest_mechanics_text/shots');
mkdirSync(shots, { recursive: true });
const b = await launch({ width: 1600, height: 900, profileRoot: resolve('out/105_quest_mechanics_text/profile') });
const { evaluate } = b;
const d = driver(b, { url, settle: 700 });
const { exists, click, setValue, speed, panel, debugMenu } = d;
let fails = 0;
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) fails++;
};
const shot = (n) => capture(b, resolve(shots, `105_${n}.png`));
const dismiss = async () => {
  for (let i = 0; i < 60 && (await exists('[data-testid="story-next"]')); i++) await click('[data-testid="story-next"]');
};

try {
  await d.load();
  await debugMenu();
  await speed(0);
  await setValue('#scenario', 'story_acts');
  await click('#start-scenario');
  await sleep(500);
  await speed(0);
  await dismiss();
  await panel('story');
  await sleep(300);
  // Every card: finished beats, the active beat, side quests.
  const cards = await evaluate(`[...document.querySelectorAll('[data-testid="story-beat-done"], [data-testid="story-beat-active"], [data-testid="story-side"]')].map((el) => {
    const m = el.querySelector('.mechanics');
    return {
      id: el.dataset.beat ?? el.dataset.side,
      goal: m?.querySelector('.mech-goal')?.textContent ?? '',
      reward: m?.querySelector('.mech-reward')?.textContent ?? '',
      mono: m ? getComputedStyle(m).fontFamily : '',
      line: el.querySelector('[data-testid="story-beat-line"], [data-testid="story-side-text"]')?.textContent ?? '',
    };
  })`);
  check(cards.length >= 20, `${cards.length} quest cards on Story`);
  for (const c of cards) {
    check(c.goal.length > 5 && c.reward.length > 7, `${c.id}: goal "${c.goal}" / reward "${c.reward}"`);
    check(/Mono|monospace|Consolas/.test(c.mono), `${c.id}: mechanics in monospace (${c.mono.slice(0, 30)})`);
    check(!/\d/.test(c.line.replace(/Helium-3|He-3/g, '')), `${c.id}: narrative line has no number: "${c.line.slice(0, 70)}"`);
  }
  const fc = cards.find((c) => c.id === 'qfull_cluster');
  check(!!fc && /Research 5 items in one run/.test(fc.goal) && /50 Science/.test(fc.reward), `1.b Full Cluster: "${fc?.goal}" / "${fc?.reward}"`);
  // The 1.b card in view, then the screenshot.
  await evaluate(`(() => {
    const el = document.querySelector('[data-side="qfull_cluster"]');
    const box = el.closest('.col-mid');
    box.scrollTop += el.getBoundingClientRect().top - box.getBoundingClientRect().top - 120;
  })()`);
  await sleep(300);
  await shot('story_act1_side_quests');
  // A finished beat's replay: its last payoff page shows the reward in the strip.
  await click('[data-testid="story-beat-done"][data-beat="qfirst_shipment"]');
  await sleep(300);
  let reward = '';
  for (let i = 0; i < 10 && (await exists('[data-testid="story-modal"]')); i++) {
    reward = await evaluate(`document.querySelector('[data-testid="story-modal"] .mech-reward')?.textContent ?? ''`);
    if (reward) break;
    await click('[data-testid="story-next"]');
    await sleep(150);
  }
  check(/Advanced Blueprints/.test(reward), `1.2's payoff modal shows its reward: "${reward}"`);
  await shot('story_modal_reward');
  await dismiss();
  // The Events Story list: every entry (side quests and one-offs) shows its mechanics strip.
  const events = await evaluate(`[...document.querySelectorAll('[data-testid="events-story"] .event-item')].map((el) => ({
    id: el.dataset.event, mech: el.querySelector('.mechanics')?.textContent ?? '', body: el.querySelector('.event-body')?.textContent ?? '' }))`);
  for (const e of events) {
    check(e.mech.length > 8, `event ${e.id}: "${e.mech.slice(0, 70)}"`);
    check(!/\d/.test(e.body), `event ${e.id}: narrative has no number`);
  }
  // The main-quest box in the right column.
  const box = await evaluate(`document.querySelector('.col-right [data-testid="quest"] .mechanics, [data-testid="quest"] .mechanics')?.textContent ?? ''`);
  check(/Goal/i.test(box) && /Reward/i.test(box), `main-quest box: "${box.slice(0, 90)}"`);
} finally {
  await b.close();
}
console.log(fails ? `${fails} failed` : 'all ok');
process.exit(fails ? 1 : 0);
