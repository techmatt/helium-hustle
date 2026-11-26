// 096: the done-when checks in a browser, with screenshots of each, both themes.
//   cd web && npm run build && npx vite preview --port <p> --strictPort
//   HH_URL=http://localhost:<p>/ node ../experiments/results/096_ui_playtest_fixes/shots.mjs [outDir]
// 1. A new game shows no timeline wording (and no Projects tile: it opens at the first
//    retirement), so Projects is read in scenario late_unlocks: only fundable cards, no
//    "timeline" anywhere on the page.
// 2. Story (late_unlocks: Act 1, 1.6 active; story_acts: Act 4): one-line finished beats
//    and side quests, and no glow from the active dot washing the spine out around it.
// 3. Recent Activity (scenario pad_loading, one launch): shipments in the feed; unchecking Shipments
//    hides them, and the choice survives a reload.
import { mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('shots', 'node shots.mjs');
const outDir = resolve(process.argv[2] ?? here);
mkdirSync(outDir, { recursive: true });
const b = await launch({ width: 1600, height: 900, profileRoot: resolve(here, '../../../out/096_ui_playtest_fixes/profile') });
const { evaluate } = b;
const { waitFor, exists, click, setValue, load, speed, panel, debugMenu, cheats, grant, runOnly, autoStory } = driver(b, { url, settle: 700 });

const failures = [];
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) failures.push(what);
};
const shot = async (name) => {
  await sleep(250);
  await capture(b, resolve(outDir, `${name}.png`));
};
const theme = (t) => evaluate(`document.documentElement.dataset.theme = '${t}'`);
const scenario = async (name) => {
  await debugMenu();
  await speed(0);
  await setValue('#scenario', name);
  await click('#start-scenario');
  await autoStory();
  await sleep(500);
};

await load();
await autoStory();
// A new game: no Projects tile, and no timeline wording anywhere.
check(!(await exists('.nav-tile[data-nav="projects"]')), 'a new game has no Projects tile yet');
check(!/timeline/i.test(await evaluate('document.body.innerText')), 'a new game names no timeline');

// 1. Projects, in late_unlocks (Projects open, swarm not revealed).
await scenario('late_unlocks');
await panel('projects');
const pr = await evaluate(`(() => {
  const p = document.querySelector('[data-panel="projects"]');
  return {
    cards: [...p.querySelectorAll('.card.project')].map((c) => ({ id: c.dataset.id, locked: c.classList.contains('locked') || /Locked:/.test(c.textContent), done: c.classList.contains('done') })),
    heads: [...p.querySelectorAll('.fold-head, h3')].map((h) => h.textContent),
    text: p.innerText,
    page: document.body.innerText,
  };
})()`);
check(pr.cards.length > 0 && pr.cards.every((c) => !c.locked && !c.done), `Projects shows only fundable cards: ${pr.cards.map((c) => c.id).join(', ')}`);
check(!/timeline/i.test(pr.page), `no "timeline" on the page before the reveal: ${(pr.page.match(/.{0,40}timeline.{0,40}/i) ?? [''])[0]}`);
await shot('096_projects_light');
await theme('dark');
await shot('096_projects_dark');
await theme('light');

// 2. Story, Act 1.
const storyRead = () =>
  evaluate(`(() => {
    const p = document.querySelector('[data-panel="story"]');
    const lines = [...p.querySelectorAll('[data-testid="story-beat-line"], [data-testid="story-side-text"]')];
    const lh = (l) => { const s = getComputedStyle(l); return parseFloat(s.lineHeight) || parseFloat(s.fontSize) * 1.4; };
    const one = lines.every((l) => l.getBoundingClientRect().height <= lh(l) * 1.3);
    const a = p.querySelector('[data-testid="story-beat-active"] .beat-dot');
    const spine = p.querySelector('.story-timeline');
    const r = a?.getBoundingClientRect();
    // What draws at the spine's x just above and just below the active dot: the timeline
    // itself (its ::before line), not another box over it.
    // The dot's own glow (a box-shadow) is what used to wash the spine out around it.
    const glow = a ? getComputedStyle(a).boxShadow : '';
    return { n: lines.length, one, texts: lines.slice(0, 3).map((l) => l.textContent), glow, active: !!a };
  })()`);
await panel('story');
await sleep(400);
let st = await storyRead();
check(st.active && st.n > 0 && st.one, `Act 1: ${st.n} finished cards, each one line: ${JSON.stringify(st.texts)}`);
check(st.texts.every((t) => t.endsWith('…') || !t.replace(/^DRAFT\. /, '').includes('. ')), 'each line is a first sentence');
check(st.glow === 'none', `the active dot casts no glow over the spine: ${st.glow}`);
await shot('096_story_act1_light');
await theme('dark');
await shot('096_story_act1_dark');
await theme('light');
// A side quest's line opens in place.
if (await exists('[data-testid="story-side-text"]')) {
  await click('[data-testid="story-side-text"]');
  const open = await evaluate(`document.querySelector('[data-testid="story-side-text"]').getAttribute('aria-expanded')`);
  check(open === 'true', 'a finished side quest\'s line opens its whole text on a click');
  await click('[data-testid="story-side-text"]');
}

// 3. Recent Activity: one shipment in the feed.
// As e2e's section 3: pad_loading's pad given He-3 cargo, loaded full, fueled, launched.
await scenario('pad_loading');
await panel('launch_pads');
await setValue('[data-pad="0"] select', 'he3');
await sleep(200);
await runOnly('load_pads', await evaluate(`[...document.querySelectorAll('.lines li')].map((li) => li.dataset.command)`));
await cheats();
for (let i = 0; i < 5; i++) {
  await grant('he3', '20');
  await setValue('#cheat-days', '4');
  await click('#cheat-jump');
  await sleep(100);
}
await grant('propellant', '5');
await panel('launch_pads');
await waitFor(`!!document.querySelector('[data-testid="launch-0"].go')`, 'the pad ready', 5000);
await click('[data-testid="launch-0"]');
await waitFor(`!!document.querySelector('[data-testid="events-activity"] li[data-kind="shipment"]')`, 'a shipment in Recent Activity', 5000);
await autoStory();
const ships = () => evaluate(`document.querySelectorAll('[data-testid="events-activity"] li[data-kind="shipment"]').length`);
check((await ships()) > 0 && (await exists('[data-testid="activity-filter"] input[data-group="shipments"]')), `shipments listed (${await ships()}) with a Shipments checkbox`);
await shot('096_activity_all');
await click('[data-testid="activity-filter"] input[data-group="shipments"]');
await sleep(200);
check((await ships()) === 0, 'unchecking Shipments hides them');
await shot('096_activity_no_shipments');
await load();
await autoStory();
check((await ships()) === 0 && !(await evaluate(`document.querySelector('[data-testid="activity-filter"] input[data-group="shipments"]').checked`)), 'the choice survives a reload');
await click('[data-testid="activity-filter"] input[data-group="shipments"]');
await sleep(200);
check((await ships()) > 0, 'checking it again shows them');

// Story, Act 4 (after the reveal).
await scenario('story_acts');
await panel('story');
await sleep(400);
st = await storyRead();
check(st.active && st.n > 0 && st.one, `Act 4: ${st.n} finished cards, each one line`);
check(st.glow === 'none', `Act 4: the active dot casts no glow over the spine`);
await shot('096_story_act4_light');
await theme('dark');
await shot('096_story_act4_dark');

check(b.errors.length === 0, `no page errors: ${b.errors.join(' | ')}`);
await b.close();
console.log(failures.length ? `${failures.length} failed` : 'all ok');
process.exit(failures.length ? 1 : 0);
