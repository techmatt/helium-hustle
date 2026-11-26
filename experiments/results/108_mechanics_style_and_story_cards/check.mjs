// 108's check in the running game (vite preview at HH_URL): Story cards start collapsed on
// one unbroken spine, expanding 1.2 shows "Reward: Advanced Blueprints." in the UI font on a
// warm tint, and no mechanics text anywhere is monospace. From the repo root, a static
// build served:
//   HH_URL=http://localhost:<port>/ node experiments/results/108_mechanics_style_and_story_cards/check.mjs out/108_mechanics_style_and_story_cards/shots
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { launch, driver, capture, sleep } from '../../../web/scripts/cdp.mjs';

const url = process.env.HH_URL;
const shots = resolve(process.argv[2] ?? 'out/108_mechanics_style_and_story_cards/shots');
mkdirSync(shots, { recursive: true });
const b = await launch({ width: 1600, height: 900, profileRoot: resolve('out/108_mechanics_style_and_story_cards/profile') });
const { evaluate } = b;
const d = driver(b, { url, settle: 700 });
const { exists, click, setValue, speed, panel, debugMenu } = d;
let fails = 0;
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) fails++;
};
const shot = (n) => capture(b, resolve(shots, `108_${n}.png`));
const dismiss = async () => {
  for (let i = 0; i < 60 && (await exists('[data-testid="story-next"]')); i++) await click('[data-testid="story-next"]');
};
// Every mechanics block on the page: none monospace, none with a border.
const mono = () =>
  evaluate(`[...document.querySelectorAll('.mechanics, .mechanics *')].filter((e) => /mono|consolas/i.test(getComputedStyle(e).fontFamily) || parseFloat(getComputedStyle(e).borderLeftWidth) > 0).length`);
// The spine: one element from the first dot to the last, every dot centered on it.
const spine = () =>
  evaluate(`(() => {
    const t = document.querySelector('[data-testid="story-timeline"]');
    const s = getComputedStyle(t, '::before');
    const tr = t.getBoundingClientRect();
    const left = tr.left + parseFloat(s.left), w = parseFloat(s.width);
    const top = tr.top + parseFloat(s.top), bottom = tr.bottom - parseFloat(s.bottom);
    const dots = [...t.querySelectorAll('.beat-dot')].map((d) => d.getBoundingClientRect());
    return {
      covers: dots.every((r) => r.top >= top - 1 && r.bottom <= bottom + 1),
      centered: dots.every((r) => Math.abs((r.left + r.right) / 2 - (left + w / 2)) <= 1.5),
      offs: dots.map((r) => Math.round(((r.left + r.right) / 2 - (left + w / 2)) * 10) / 10),
      // Anything opaque painted over the spine between the first and last dot (dots aside).
      hidden: (() => {
        const x = left + w / 2;
        let n = 0;
        for (let y = dots[0].bottom + 2; y < dots[dots.length - 1].top - 2; y += 4) {
          const el = document.elementFromPoint(x, y);
          if (el && !t.contains(el)) continue;
          if (el && el.closest('.act-band, .beat-dot')) continue;
          if (el && el !== t && getComputedStyle(el).backgroundColor !== 'rgba(0, 0, 0, 0)') n++;
        }
        return n;
      })(),
    };
  })()`);

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
  const cards = await evaluate(`[...document.querySelectorAll('[data-testid="story-beat-done"], [data-testid="story-beat-active"], [data-testid="story-side"]')].map((el) => ({
    id: el.dataset.beat ?? el.dataset.side,
    done: el.matches('[data-testid="story-beat-done"], .is-done'),
    tick: /✓/.test(el.querySelector('.beat-title, .side-title')?.textContent ?? ''),
    body: !!el.querySelector('[data-testid="story-card-body"]'),
    mech: !!el.querySelector('.mechanics'),
    line: el.querySelector('[data-testid="story-card-line"]')?.textContent ?? '',
    pic: !!el.querySelector('[data-testid="quest-pic-thumb"]'),
  }))`);
  check(cards.length >= 20, `${cards.length} cards on Story`);
  check(cards.every((c) => !c.body && !c.mech), `every card collapsed: no story body, no mechanics (${cards.filter((c) => c.body || c.mech).map((c) => c.id).join(',')})`);
  check(cards.every((c) => c.line.length > 5), 'every card shows its first line');
  check(cards.every((c) => c.tick === c.done), `a ✓ on each finished card's title, none on open ones`);
  check(!(await evaluate(`/\\bDONE\\b|Done:/.test(document.querySelector('[data-panel="story"]').textContent)`)), 'no "Done" label on Story');
  const sp = await spine();
  check(sp.covers && sp.centered && sp.hidden === 0, `one spine behind every dot (covers ${sp.covers}, centered ${sp.centered} ${JSON.stringify(sp.offs)}, covered points ${sp.hidden})`);
  await evaluate(`document.querySelector('.col-mid').scrollTop = 0`);
  await sleep(200);
  await shot('story_collapsed_top');
  await evaluate(`(() => { const el = document.querySelector('[data-testid="story-beat-active"]'); const box = el.closest('.col-mid'); box.scrollTop += el.getBoundingClientRect().top - box.getBoundingClientRect().top - 300; })()`);
  await sleep(200);
  await shot('story_collapsed_active');
  // Expand 1.2.
  await evaluate(`document.querySelector('.col-mid').scrollTop = 0`);
  await click('[data-testid="story-beat-done"][data-beat="qfirst_shipment"] [data-testid="story-beat-toggle"]');
  await sleep(200);
  const e12 = await evaluate(`(() => {
    const c = document.querySelector('[data-beat="qfirst_shipment"]');
    const m = c.querySelector('[data-testid="story-card-body"] .mechanics');
    const cs = m && getComputedStyle(m);
    return {
      goal: m?.querySelector('.mech-goal')?.textContent ?? '',
      reward: m?.querySelector('.mech-reward')?.textContent ?? '',
      font: cs?.fontFamily ?? '', size: cs?.fontSize ?? '', bg: cs?.backgroundColor ?? '', border: cs?.borderLeftWidth ?? '',
      cardSize: getComputedStyle(c.querySelector('.beat-card')).fontSize,
      paras: c.querySelectorAll('.card-para').length,
    };
  })()`);
  check(e12.reward === 'Reward: Advanced Blueprints.', `1.2 expanded: "${e12.reward}"`);
  check(e12.goal.startsWith('Goal: '), `1.2 expanded, finished: "${e12.goal.slice(0, 40)}"`);
  check(/Plex Sans/.test(e12.font) && parseFloat(e12.size) === parseFloat(e12.cardSize) - 1, `UI font one size smaller: ${e12.font.slice(0, 20)} ${e12.size} (card ${e12.cardSize})`);
  const [r, g, bl] = e12.bg.match(/\d+/g).map(Number);
  check(r > bl && g > bl && e12.border === '0px', `a faint warm tint, no border: ${e12.bg}, border ${e12.border}`);
  check(e12.paras > 1, `the whole story: ${e12.paras} paragraphs`);
  await evaluate(`(() => { const el = document.querySelector('[data-beat="qfirst_shipment"]'); const box = el.closest('.col-mid'); box.scrollTop += el.getBoundingClientRect().top - box.getBoundingClientRect().top - 120; })()`);
  await sleep(200);
  await shot('story_1_2_expanded');
  // Expand a side quest (1.c) too.
  if (await exists('[data-side="qblueprints_for_everyone"]')) {
    await click('[data-side="qblueprints_for_everyone"] [data-testid="story-side-toggle"]');
    await sleep(200);
    const rc = await evaluate(`document.querySelector('[data-side="qblueprints_for_everyone"] .mech-reward')?.textContent ?? ''`);
    check(/^Reward: Every run starts with Basic and Advanced Blueprints/.test(rc), `1.c expanded: "${rc}"`);
    await evaluate(`(() => { const el = document.querySelector('[data-side="qblueprints_for_everyone"]'); const box = el.closest('.col-mid'); box.scrollTop += el.getBoundingClientRect().top - box.getBoundingClientRect().top - 200; })()`);
    await sleep(200);
    await shot('story_side_expanded');
  }
  const sp2 = await spine();
  check(sp2.covers && sp2.centered && sp2.hidden === 0, `spine unbroken with cards expanded (covered points ${sp2.hidden})`);
  check((await mono()) === 0, 'no monospace or bordered mechanics on Story');
  // The same in the dark theme.
  const theme = await evaluate(`document.documentElement.dataset.theme`);
  await evaluate(`document.documentElement.dataset.theme = 'dark'`);
  await sleep(200);
  await shot('story_side_expanded_dark');
  await evaluate(`document.documentElement.dataset.theme = ${JSON.stringify(theme ?? 'light')}`);
  // Replay from the expanded card: the modal's mechanics in the new style.
  await click('[data-beat="qfirst_shipment"] [data-testid="story-replay"]');
  await sleep(300);
  let reward = '';
  for (let i = 0; i < 10 && (await exists('[data-testid="story-modal"]')); i++) {
    reward = await evaluate(`document.querySelector('[data-testid="story-modal"] .mech-reward')?.textContent ?? ''`);
    if (reward) break;
    await click('[data-testid="story-next"]');
    await sleep(150);
  }
  check(reward === 'Reward: Advanced Blueprints.', `1.2's payoff modal: "${reward}"`);
  check((await mono()) === 0, 'no monospace or bordered mechanics in the story modal');
  await shot('story_modal_reward');
  await dismiss();
  // Events: every entry's mechanics, Effect lines always shown, none monospace.
  const ev = await evaluate(`[...document.querySelectorAll('[data-testid="events-story"] .event-item')].map((el) => el.querySelector('.mechanics')?.textContent ?? '')`);
  check(ev.length > 0 && ev.every((t) => /^(Goal|Effect): /.test(t)), `${ev.length} Events entries, each with its mechanics line: ${JSON.stringify(ev.slice(0, 2))}`);
  check((await mono()) === 0, 'no monospace or bordered mechanics in Events');
  await evaluate(`(() => { const el = document.querySelector('[data-testid="events-story"]'); el?.scrollIntoView(); })()`);
  await sleep(200);
  await shot('events');
} finally {
  await b.close();
}
console.log(fails ? `${fails} failed` : 'all ok');
process.exit(fails ? 1 : 0);
