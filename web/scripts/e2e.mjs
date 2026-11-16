// Browser check: drives headless Edge or Chrome over the DevTools protocol (no
// packages) against a served page.
//   1. A fresh game through the normal UI only (no debug menu): a build order to the
//      first pad loaded and launched (Q1 to Q3), retirement, the run summary holding the
//      clock (a double-click on Continue buys nothing under it), run 2 (known tiles and
//      resources still shown, not cheated), the career's play log on disk as a bot's.
//   2. Every panel: cheats reach Research, Projects and Ideologies; all 11 tiles open;
//      the cheated mark, a jump ahead and the replay check.
//   3. A program's current-line frame at 10x and 50x, each speed's days per second,
//      pause, click-to-buy (a green card's text buys, its −/+ and Sell 1 don't, space after
//      a click pauses and buys nothing, a grey card says why), a reload keeping the save,
//      a save from another revision discarded, the scenarios and the replay check.
//   5. (Run before 4.) Readability: a capped resource's positive rate (Resources and the
//      status bar), an affordable card's tinted header, scenario pad_loading's full pad
//      short on fuel then Ready (green Launch, border, nav dot), and a boredom beat that
//      leaves the main quest pinned and counts unread in a folded Story until it is open
//      and on screen for a second. Shots read-*.png.
//   4. The theme: light by default; the Options and debug toggles switch it with no
//      element moving, and a reload keeps it.
// Any uncaught page error fails the run.
//   HH_URL      the page (default http://localhost:5173/)
//   HH_BROWSER  a browser off the usual paths
//   HH_PROFILE  browser profile dir (default ../out/e2e-profile, wiped first)
//   HH_SHOTS    <dir>: save a PNG of the page at each step worth seeing
import { spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const url = process.env.HH_URL ?? 'http://localhost:5173/';
const browsers = [
  process.env.HH_BROWSER,
  'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
  'C:/Program Files/Google/Chrome/Application/chrome.exe',
  '/usr/bin/google-chrome',
  '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
].filter(Boolean);
const exe = browsers.find((b) => existsSync(b));
if (!exe) throw new Error('no browser found: set HH_BROWSER');
// Absolute: given a relative one, Edge hands the URL to a running Edge and exits.
const port = 9300 + Math.floor(Math.random() * 500);
// A fresh profile per run (one per port): Windows can hold a just-closed one a while.
const profile = resolve(process.env.HH_PROFILE ?? resolve(here, '../../out/e2e-profile'), String(port));
try {
  rmSync(profile, { recursive: true, force: true });
} catch {}
mkdirSync(profile, { recursive: true });
const proc = spawn(exe, [
  '--headless=new', `--remote-debugging-port=${port}`, `--user-data-dir=${profile}`,
  // --enable-automation sets navigator.webdriver: the page files its play logs as a bot's.
  '--enable-automation', '--window-size=1600,900', '--no-first-run', '--no-default-browser-check', 'about:blank',
], { stdio: 'ignore' });

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let page;
for (let i = 0; i < 150 && !page; i++) {
  await sleep(200);
  const targets = await fetch(`http://127.0.0.1:${port}/json/list`).then((r) => r.json()).catch(() => []);
  page = targets.find((t) => t.type === 'page');
}
if (!page) {
  proc.kill();
  throw new Error(`${exe} exposed no page on port ${port} within 30 s`);
}
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener('open', r, { once: true }));
let nextId = 1;
const pending = new Map();
const pageErrors = [];
ws.addEventListener('message', (ev) => {
  const msg = JSON.parse(ev.data);
  if (msg.id && pending.has(msg.id)) {
    pending.get(msg.id)(msg);
    pending.delete(msg.id);
  } else if (msg.method === 'Runtime.exceptionThrown') {
    pageErrors.push(msg.params.exceptionDetails?.exception?.description ?? msg.params.exceptionDetails?.text);
  } else if (msg.method === 'Runtime.consoleAPICalled' && msg.params.type === 'error') {
    pageErrors.push(msg.params.args.map((a) => a.value ?? a.description).join(' '));
  }
});
const send = (method, params = {}) =>
  new Promise((r) => {
    const id = nextId++;
    pending.set(id, r);
    ws.send(JSON.stringify({ id, method, params }));
  });
const evaluate = async (expr) => {
  const r = await send('Runtime.evaluate', { expression: expr, awaitPromise: true, returnByValue: true });
  if (r.result?.exceptionDetails) throw new Error(`${expr}: ${JSON.stringify(r.result.exceptionDetails)}`);
  return r.result?.result?.value;
};
const q = (sel) => JSON.stringify(sel);
const day = async () => Number((await evaluate(`document.getElementById('day').textContent`)).replace(/\D/g, ''));
const run = async () => Number((await evaluate(`document.getElementById('run').textContent`)).replace(/\D/g, ''));
const noticeText = () => evaluate(`document.getElementById('notice')?.textContent ?? ''`);
const exists = (sel) => evaluate(`!!document.querySelector(${q(sel)})`);
const enabled = (sel) => evaluate(`(() => { const b = document.querySelector(${q(sel)}); return !!b && !b.disabled; })()`);
const click = async (sel) => {
  const ok = await evaluate(`(() => { const b = document.querySelector(${q(sel)}); if (!b) return false; b.click(); return true; })()`);
  if (!ok) throw new Error(`nothing to click at ${sel}`);
};
const setValue = (sel, value) =>
  evaluate(`(() => {
    const el = document.querySelector(${q(sel)});
    el.value = ${q(value)};
    el.dispatchEvent(new Event(el.tagName === 'SELECT' ? 'change' : 'input', { bubbles: true }));
  })()`);
const waitFor = async (expr, what, ms = 10000) => {
  for (const end = Date.now() + ms; Date.now() < end; await sleep(100)) if (await evaluate(expr)) return;
  throw new Error(`timed out waiting for ${what}`);
};
const load = async (u) => {
  // Mark the old document, so the wait below can't pass on it before the new one commits.
  await evaluate('window.__stale = true');
  await send('Page.navigate', { url: u });
  await waitFor('!window.__stale', 'the navigation');
  // "Loading…" until the engine has rendered once.
  const ready = `/^Day \\d/.test(document.getElementById('day')?.textContent ?? '') && !!document.querySelector('.nav-tile')`;
  await waitFor(ready, 'the page');
  // A dev server just after an edit may reload the page once more: let it settle.
  await sleep(700);
  await waitFor(ready, 'the page (settled)');
};
const speed = (s) => click(`#speeds button[data-speed="${s}"]`);
const panel = async (id) => {
  await click(`.nav-tile[data-nav="${id}"]`);
  await waitFor(`!!document.querySelector('[data-panel="${id}"]')`, `the ${id} panel`, 3000);
};
const navVisible = (id) => exists(`.nav-tile[data-nav="${id}"]`);
/** Opens the debug section of Options (it starts folded). */
const debugMenu = async () => {
  await panel('options');
  if (!(await exists('#scenario'))) await click('[data-testid="debug"] .fold-head');
};

/** With HH_SHOTS=<dir>, saves a PNG of the page at each named moment. */
const shot = async (name) => {
  if (!process.env.HH_SHOTS) return;
  mkdirSync(process.env.HH_SHOTS, { recursive: true });
  const r = await send('Page.captureScreenshot', { format: 'png' });
  writeFileSync(resolve(process.env.HH_SHOTS, `${name}.png`), Buffer.from(r.result.data, 'base64'));
};

const failures = [];
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) failures.push(what);
};

/** What the player reads off the screen each look: resources, buildings, pads. */
const look = () =>
  evaluate(`(() => {
    const res = {};
    for (const tr of document.querySelectorAll('tr[data-res]')) res[tr.dataset.res] = Number(tr.dataset.amount);
    const quest = document.querySelector('[data-testid="quest"]')?.dataset.quest ?? null;
    const lines = [...document.querySelectorAll('.lines li')].map((li) => li.dataset.command);
    return { res, quest, lines, day: Number(document.getElementById('day').textContent.replace(/\\D/g, '')) };
  })()`);
const buildings = () =>
  evaluate(`(() => {
    const out = {};
    for (const c of document.querySelectorAll('.card.building')) {
      out[c.dataset.id] = { owned: Number(c.dataset.owned), short: c.dataset.short.split(' ').filter(Boolean), buyable: c.classList.contains('can-buy') };
    }
    return out;
  })()`);

/** Program 1 runs just `cmd`: Reset, then add it from the Commands panel. */
const runOnly = async (cmd, lines) => {
  if (lines.length === 1 && lines[0] === cmd) return;
  await click('.prog-tabs button[data-program="0"]');
  if (lines.length) await click('.prog-procs button[title="Clear this program"]');
  await panel('commands');
  await click(`[data-testid="add-${cmd}"]`);
};

/** Program 1, one processor, lines of counts 2, 1, 3: at 10x (a day a redraw) the
 * current-line frame walks the queue in order, the lines keep their order and no line's
 * name moves; at 50x (several days a redraw) no frame hops around, and the lines show
 * what ran over the span. Leaves the game paused. */
async function programOrder() {
  const queue = [['idle', 2], ['cloud_compute', 1], ['idle', 3]];
  await speed(0);
  await click('.prog-tabs button[data-program="0"]');
  if (await exists('.lines .line')) await click('.prog-procs button[title="Clear this program"]');
  while (Number(await evaluate(`document.querySelector('.prog-procs span').textContent.match(/\\d+/)[0]`)) > 1) {
    await click('[data-testid="proc-minus"]');
  }
  if (!(await enabled('[data-testid="proc-minus"]'))) await click('[data-testid="proc-plus"]');
  await panel('commands');
  for (const [c] of queue) await click(`[data-testid="add-${c}"]`);
  for (const [i, [, n]] of queue.entries()) {
    for (let k = 1; k < n; k++) await click(`.lines .line[data-line="${i}"] button:nth-of-type(2)`);
  }
  const order = () => evaluate(`[...document.querySelectorAll('.lines .line')].map((l) => l.dataset.command + 'x' + l.textContent.match(/\\(x(\\d+)\\)/)[1]).join(' ')`);
  const queued = queue.map(([c, n]) => `${c}x${n}`).join(' ');
  check((await order()) === queued, `program queued in order: ${await order()}`);
  // Each redraw's pointer, and where each line's name starts (it must never move).
  await evaluate(`(() => { window.__ptr = []; window.__left = new Set(); window.__ob?.disconnect();
    window.__ob = new MutationObserver(() => {
      const p = [...document.querySelectorAll('.lines .line')].findIndex((l) => l.classList.contains('current'));
      if (window.__ptr.at(-1) !== p) window.__ptr.push(p);
      for (const n of document.querySelectorAll('.lines .line-name')) {
        const r = document.createRange();
        r.selectNodeContents(n);
        window.__left.add(Math.round(r.getBoundingClientRect().left));
      }
    });
    window.__ob.observe(document.querySelector('.lines'), { subtree: true, attributes: true, childList: true }); })()`);
  await speed(10);
  await sleep(2500);
  await speed(0);
  const ptr = await evaluate('window.__ptr');
  const forward = ptr.every((p, i) => i === 0 || p === (ptr[i - 1] + 1) % queue.length);
  check(ptr.length >= 4 && forward, `10x: the pointer walks the queue in order: ${ptr.join(',')}`);
  check((await order()) === queued, `10x: lines keep their order: ${await order()}`);
  await shot('program-lines');
  const lefts = await evaluate('[...window.__left]');
  check(lefts.length === 1, `10x: no line's text moves as the pointer walks (name starts at ${lefts.join(', ')} px)`);
  await evaluate('window.__ptr = []');
  await speed(50);
  await sleep(1000);
  // Read before pausing: paused, the frame is back.
  const fast = await evaluate('window.__ptr');
  const lit = await evaluate(`document.querySelectorAll('.lines .line[class*="out-"]').length`);
  const note = await exists('[data-testid="lines-span"]');
  const fastOrder = await order();
  await speed(0);
  check(
    fast.length > 0 && fast.every((p) => p === -1) && note && lit >= 2,
    `50x: no hopping pointer, lines lit by the span (pointer ${fast.join(',')}, ${lit} lit)`,
  );
  check(fastOrder === queued, `50x: lines keep their order: ${fastOrder}`);
  await evaluate('window.__ob.disconnect()');
}

// The opening, as a player would: Solar Panels, then the extraction chain, storage for
// the Launch Pad's titanium, the pad, and more Solar Panels for its upkeep.
const ORDER = [
  ['solar_panel', 2], ['excavator', 1], ['smelter', 1], ['refinery', 1], ['solar_panel', 3],
  ['excavator', 2], ['storage_depot', 1], ['solar_panel', 4], ['launch_pad', 1], ['solar_panel', 5],
];

async function playToFirstShipment() {
  await speed(50);
  const started = Date.now();
  let lastLog = 0;
  const seenQuests = new Set();
  while (Date.now() - started < 180_000) {
    if (await navVisible('retirement')) return true;
    const s = await look();
    if (s.quest) seenQuests.add(s.quest);
    if (Date.now() - lastLog > 5000) {
      lastLog = Date.now();
      console.log(`     day ${s.day}, quest ${s.quest}, credits ${s.res.credits}, he3 ${s.res.he3 ?? '-'}`);
    }
    await panel('buildings');
    const b = await buildings();
    const goal = ORDER.find(([id, n]) => b[id] && b[id].owned < n);
    if (goal && b[goal[0]].buyable) {
      await click(`[data-testid="buy-${goal[0]}"]`);
      continue;
    }
    const credits = s.res.credits ?? 0;
    let cmd = 'cloud_compute';
    if ((b.launch_pad?.owned ?? 0) > 0) {
      await panel('launch_pads');
      const cargo = Number(await evaluate(`document.querySelector('[data-pad="0"]')?.dataset.cargo ?? 0`));
      if (cargo >= 10 && (await enabled('[data-testid="launch-0"]'))) {
        await click('[data-testid="launch-0"]');
        continue;
      }
      if ((s.res.propellant ?? 0) < 20 && credits >= 12) cmd = 'buy_propellant';
      else if ((s.res.he3 ?? 0) >= 5 && cargo < 10) cmd = 'load_pads';
    } else if (goal) {
      const short = b[goal[0]].short;
      if (short.includes('titanium') && credits >= 25) cmd = 'buy_titanium';
      else if (short.includes('regolith') && credits >= 12) cmd = 'buy_regolith';
    }
    await runOnly(cmd, s.lines);
    await sleep(60);
  }
  return false;
}

/** Section 5, from scenario at_cap (paused, cheats on; `grant` from section 2). Shots
 * `read-*.png`. Leaves a notice up (the replay check's) for section 4. */
async function readability(grant) {
  // Income at cap: Energy at 100 / 100 under 20 Solar Panels still shows its surplus.
  const capped = await evaluate(`document.querySelector('tr[data-res="energy"] [data-capped]')?.textContent ?? ''`);
  check(/^\+[\d.]+ cap$/.test(capped), `a capped resource shows its positive rate: "${capped}"`);
  const bar = await evaluate(`document.querySelector('[data-testid="energy-rate"]').textContent`);
  check(/^\+[\d.]+\/day \(capped\)$/.test(bar), `the status bar's Energy rate too: "${bar}"`);
  // Affordable vs not: only the affordable card's name row is tinted.
  await debugMenu();
  // (20 Solar Panels hold all the land: free some too.)
  await grant('credits', '5000');
  await grant('land', '10');
  await grant('titanium', '20');
  await grant('regolith', '50');
  await panel('buildings');
  const heads = await evaluate(`(() => {
    const bg = (sel) => { const h = document.querySelector(sel + ' .card-head'); return h ? getComputedStyle(h).backgroundColor : null; };
    const card = (sel) => { const c = document.querySelector(sel); return c ? getComputedStyle(c).backgroundColor : null; };
    return { yes: bg('.card.building.can-buy'), no: bg('.card.building.cannot-buy'), body: card('.card.building.can-buy') };
  })()`);
  check(!!heads.yes && !!heads.no && heads.yes !== heads.no && heads.yes !== heads.body, `an affordable card's header is tinted, the card body isn't: ${JSON.stringify(heads)}`);
  await shot('read-capped-and-cards');

  // A pad short on fuel, then Ready; a boredom beat fires on the way (day 200).
  await debugMenu();
  await setValue('#scenario', 'pad_loading');
  await click('#start-scenario');
  const quest = await evaluate(`document.querySelector('[data-testid="quest-pin"] [data-testid="quest"]')?.dataset.quest ?? null`);
  check(!!quest, `the main quest has its pinned slot: ${quest}`);
  // Fold Story: a folded section keeps its count.
  await click('[data-testid="events-story"] .fold-head');
  const lines = await evaluate(`[...document.querySelectorAll('.lines li')].map((li) => li.dataset.command)`);
  await runOnly('load_pads', lines);
  // Helium-3 caps at 20 and a load moves 5: top it up and play 4 days, 5 times.
  await debugMenu();
  for (let i = 0; i < 5; i++) {
    await grant('he3', '20');
    await setValue('#cheat-days', '4');
    await click('#cheat-jump');
    await sleep(100);
  }
  await panel('launch_pads');
  await waitFor(`Number(document.querySelector('[data-pad="0"]')?.dataset.cargo ?? 0) >= 100`, 'the pad full', 3000);
  await sleep(200);
  const short = await evaluate(`(() => {
    const p = document.querySelector('[data-pad="0"]');
    return { fuel: p.querySelector('[data-testid="fuel-0"]').textContent, fueled: p.querySelector('[data-testid="fuel-0"]').dataset.fueled,
      status: p.querySelector('.pad-status').textContent, ready: p.classList.contains('ready'),
      go: p.querySelector('[data-testid="launch-0"]').classList.contains('go'), dot: !!document.querySelector('[data-testid="pads-ready"]') };
  })()`);
  check(short.fuel === 'Fuel 19.4 / 20 Propellant' && short.fueled === 'false' && short.status !== 'Ready' && !short.ready && !short.go && !short.dot,
    `a full pad short on fuel: red fuel line, not Ready: ${JSON.stringify(short)}`);
  // The pads are the panel's last section: scroll only the middle column to them.
  const toPads = `document.querySelector('.col-mid').scrollTop = 1e6`;
  await evaluate(toPads);
  // The boredom beat: in Story, unread; the quest still pinned and no banner over it.
  const beat = await evaluate(`(() => ({
    pin: document.querySelector('[data-testid="quest-pin"] [data-testid="quest"]')?.dataset.quest ?? null,
    banner: document.querySelector('[data-testid="story-banner"]')?.dataset.story ?? null,
    unread: Number(document.querySelector('[data-testid="events-story"]').dataset.unread),
    head: document.querySelector('[data-testid="events-story"] .fold-head').textContent,
    // Story is folded: the beat's line in Recent Activity says it fired.
    beat: [...document.querySelectorAll('[data-testid="events-activity"] li[data-kind="event"]')].some((e) => /Boredom Rising/.test(e.textContent)),
    slot: [...document.querySelectorAll('[data-testid="quest-pin"] [data-quest]')].map((e) => e.dataset.quest),
  }))()`);
  // Q1 completes on the way (Q2 opens): the slot holds the active quest and nothing else;
  // a banner may still hold the scenario's quest texts (quests only, never the beat).
  check(/^q/.test(beat.pin ?? '') && beat.slot.length === 1 && beat.beat && !(beat.banner ?? '').startsWith('boredom') && beat.unread >= 1,
    `a boredom beat fired: the quest stays pinned, Story counts it unread: ${JSON.stringify(beat)}`);
  await shot('read-pad-short-story-unread');
  await debugMenu();
  await grant('propellant', '0.6');
  await panel('launch_pads');
  await evaluate(toPads);
  await sleep(200);
  const ready = await evaluate(`(() => {
    const p = document.querySelector('[data-pad="0"]');
    return { fuel: p.querySelector('[data-testid="fuel-0"]').textContent, fueled: p.querySelector('[data-testid="fuel-0"]').dataset.fueled,
      status: p.querySelector('.pad-status').textContent, ready: p.classList.contains('ready'),
      go: p.querySelector('[data-testid="launch-0"]').classList.contains('go'), dot: !!document.querySelector('[data-testid="pads-ready"]') };
  })()`);
  check(ready.fuel === 'Fuel 20 / 20 Propellant' && ready.fueled === 'true' && ready.status === 'Ready' && ready.ready && ready.go && ready.dot,
    `fuel enough: Ready, green Launch, green border, nav dot: ${JSON.stringify(ready)}`);
  // Unfold Story: its count clears after about a second on screen, with no click on an item.
  await click('[data-testid="events-story"] .fold-head');
  await sleep(300);
  const early = Number(await evaluate(`document.querySelector('[data-testid="events-story"]').dataset.unread`));
  await shot('read-pad-ready-story-open');
  await waitFor(`document.querySelector('[data-testid="events-story"]').dataset.unread === '0'`, 'Story read', 4000);
  check(early >= 1, `Story's count holds for a moment once open (${early}), then clears`);
  check(
    (await evaluate(`document.querySelector('[data-testid="quest-pin"] [data-testid="quest"]')?.dataset.quest ?? null`)) === beat.pin,
    'the quest is still pinned after Story is read',
  );
  await shot('read-story-read');
  await debugMenu();
  await click('#check-replay');
  check(/^Replay ok/.test(await noticeText()), `replay check after the readability steps: ${await noticeText()}`);
}

try {
  await send('Page.enable');
  await send('Runtime.enable');
  await load(url);
  check((await day()) < 5 && (await run()) === 1, `a fresh game loads: day ${await day()}, run ${await run()}`);
  check(!(await exists('[data-testid="cheats"]')), 'cheats are off by default');
  check(!(await navVisible('retirement')) && !(await navVisible('launch_pads')), 'gated nav tiles start hidden');
  check(/Boot Sequence/.test(await evaluate(`document.querySelector('[data-testid="quest"]')?.textContent ?? ''`)), 'the first quest shows');

  check((await evaluate(`document.querySelector('[data-testid="story-banner"]')?.dataset.story`)) === 'qpower_boot_sequence', 'a new game tells its first quest');
  await click('[data-testid="story-banner"] button');
  // Two quick clicks (no render between them) both land.
  await panel('commands');
  await evaluate(`['add-cloud_compute', 'add-idle'].forEach((t) => document.querySelector('[data-testid="' + t + '"]').click())`);
  await sleep(100);
  const two = await evaluate(`[...document.querySelectorAll('.lines li')].map((li) => li.dataset.command).join(',')`);
  check(two === 'cloud_compute,idle', `two quick adds both land: ${two}`);

  // ---- 1. a fresh game to retirement and run 2, through the normal UI ----
  const shipped = await playToFirstShipment();
  check(shipped, `played to the first shipment by day ${await day()}`);
  await click('[data-testid="events-completed"] .fold-head'); // it starts folded
  const done = await evaluate(`[...document.querySelectorAll('[data-testid="events-completed"] [data-quest]')].map((li) => li.dataset.quest)`);
  check(
    ['qpower_boot_sequence', 'qextract_first_extraction', 'qshipment_proof_of_concept'].every((x) => done.includes(x)),
    `QPower, QExtract and QShipment complete: ${done.join(', ')}`,
  );
  check(await evaluate(`!!document.querySelector('.activity li[data-kind="shipment"]')`), 'the shipment is in Recent Activity');
  check(await exists('tr[data-res="propellant"]'), 'Propellant shows in Resources once bought');
  const told = await evaluate(`document.querySelector('[data-testid="story-banner"]')?.textContent ?? ''`);
  check(/First Extraction|Proof of Concept|Automation/.test(told), `later quests are told as they open: ${told.slice(0, 40)}`);
  check(await evaluate(`!!document.querySelector('.nav-tile[data-nav="retirement"] .dot')`), 'the new Retirement tile carries a "new" dot');
  await panel('retirement');
  check(!(await evaluate(`!!document.querySelector('.nav-tile[data-nav="retirement"] .dot')`)), 'opening it clears the dot');
  await shot('retirement-panel');
  await panel('buildings');
  await shot('run1-buildings');
  await panel('launch_pads');
  await shot('run1-launch-pads');
  await panel('retirement');
  const runDays = await day();
  await click('[data-testid="retire"]');
  await click('[data-testid="retire-confirm"]');
  await waitFor(`!!document.querySelector('[data-testid="summary"]')`, 'the run summary', 3000);
  const paused1 = await day();
  await sleep(600);
  check((await day()) === paused1 && (await run()) === 2, `the run summary pauses the game (run 2, day ${paused1})`);
  await shot('run-summary');
  // A real double-click on Continue over the Buildings panel: its second click lands on
  // whatever the summary covered, and must not reach a card.
  await click('.nav-tile[data-nav="buildings"]');
  {
    const owned = `[...document.querySelectorAll('.card.building')].reduce((n, c) => n + Number(c.dataset.owned), 0)`;
    const o0 = await evaluate(owned);
    const p = await evaluate(`(() => { const r = document.querySelector('[data-testid="continue"]').getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; })()`);
    for (const clickCount of [1, 2])
      for (const type of ['mousePressed', 'mouseReleased']) await send('Input.dispatchMouseEvent', { type, x: p.x, y: p.y, button: 'left', clickCount });
    await sleep(200);
    const hint = await evaluate(`[...document.querySelectorAll('.buy-hint.short')].map((h) => h.textContent).join(' ')`);
    check((await evaluate(owned)) === o0 && !hint, `a double-click on Continue buys nothing under it (${o0} -> ${await evaluate(owned)}${hint ? `, "${hint}"` : ''})`);
  }
  await sleep(800);
  check((await run()) === 2 && (await day()) > 0 && !(await exists('[data-testid="summary"]')), `run 2 plays on (day ${await day()}; run 1 lasted ${runDays} days)`);
  check(await exists('#speeds button[data-speed="50"][aria-pressed="true"]'), 'Continue restores the speed before the summary');
  check(await navVisible('retirement') && await navVisible('launch_pads'), 'known nav tiles stay visible in run 2');
  check(await evaluate(`!!document.querySelector('tr[data-res="he3"]')`), 'known resources stay visible in run 2');
  check(!(await evaluate(`document.getElementById('status')?.textContent ?? document.querySelector('.status').textContent`)).includes('CHEATED'), 'an honest game is not marked');

  // The play log of that career, on disk through the dev server.
  await panel('options');
  const career = await evaluate(`document.querySelector('[data-testid="playlog-status"]').dataset.career`);
  const logDir = await evaluate(`fetch('/__playlog?career=${career}').then((r) => r.json()).then((j) => j.dir)`);
  check(career.startsWith('bot_'), `an automated browser's career is filed as a bot's: ${career}`);
  await sleep(2500); // the page sends its log every 2 s
  const logFile = resolve(logDir, `${career}.jsonl`);
  const logLines = existsSync(logFile) ? readFileSync(logFile, 'utf8').trim().split('\n').map((l) => JSON.parse(l)) : [];
  const kinds = new Set(logLines.map((l) => l.kind));
  check(
    logLines[0]?.kind === 'header' && logLines[0].commit && ['action', 'speed', 'panel', 'retirement', 'checkpoint', 'session_start'].every((k) => kinds.has(k)),
    `the play log is on disk: ${logFile} (${logLines.length} lines, ${logLines.filter((l) => l.kind === 'action').length} actions)`,
  );

  // ---- 2. every panel, reaching the gated ones with cheats ----
  await speed(0);
  await debugMenu();
  await click('#cheats-toggle');
  check(await exists('[data-testid="cheats"]'), 'the cheats section opens');
  const grant = async (res, amount) => {
    await setValue('#cheat-resource', res);
    await setValue('#cheat-amount', amount);
    await click('#cheat-grant');
  };
  const buy = async (id) => {
    await panel('buildings');
    await click(`[data-testid="buy-${id}"]`);
    await debugMenu();
  };
  await grant('credits', '100000');
  await grant('regolith', '50');
  await grant('titanium', '20');
  await buy('storage_depot');
  await grant('titanium', '45');
  await grant('circuits', '30');
  await buy('data_center');
  await grant('circuits', '30');
  await buy('research_lab');
  await grant('science', '5000');
  check(await navVisible('research'), 'Research appears with a Research Lab');
  await panel('research');
  await click('[data-testid="research-market_awareness"]');
  // Quests and events check at the end of a day: let days pass.
  await speed(1);
  await waitFor(`!!document.querySelector('[data-testid="research-geopolitical_intelligence"]')`, 'Geopolitical Intelligence', 5000);
  await speed(0);
  check(await navVisible('projects'), 'Projects appears with Market Awareness');
  await click('[data-testid="research-geopolitical_intelligence"]');
  await speed(1);
  await waitFor(`!!document.querySelector('.nav-tile[data-nav="ideologies"]')`, 'the Ideologies tile', 5000);
  await speed(0);
  const tiles = await evaluate(`[...document.querySelectorAll('.nav-tile')].map((t) => t.dataset.nav)`);
  for (const id of tiles) {
    await panel(id);
    const text = await evaluate(`document.querySelector('[data-panel="${id}"]').textContent.length`);
    check(text > 20, `panel ${id} opens (${text} chars)`);
    await shot(`panel-${id}`);
  }
  check(tiles.length === 11, `all 11 nav tiles reachable: ${tiles.join(', ')}`);
  check((await evaluate(`document.querySelector('.status').textContent`)).includes('CHEATED'), 'a cheated game is marked');
  await debugMenu();
  await setValue('#cheat-days', '30');
  const d0 = await day();
  await click('#cheat-jump');
  check((await day()) === d0 + 30, `jump ahead 30 days (${d0} -> ${await day()})`);
  await click('#check-replay');
  check(/^Replay ok/.test(await noticeText()), `replay check of the whole career: ${await noticeText()}`);

  // ---- 3. speeds, saves and scenarios ----
  await setValue('#scenario', 'rich');
  await click('#start-scenario');
  check((await day()) === 0 && /Started scenario rich/.test(await noticeText()), 'debug menu starts scenario rich');
  await programOrder();
  for (const [s, secs] of [[1, 3], [3, 2], [10, 2], [50, 1], [200, 1]]) {
    await speed(s);
    const a = await day();
    const t0 = Date.now();
    await sleep(secs * 1000);
    const rate = ((await day()) - a) / ((Date.now() - t0) / 1000);
    check(rate > s * 0.6 && rate < s * 1.4, `${s}x: ${rate.toFixed(1)} days/s`);
  }
  await speed(0);
  const paused = await day();
  await sleep(1200);
  check((await day()) === paused, 'pause holds the day');
  await panel('buildings');
  const owned = () => evaluate(`document.querySelector('.card.building[data-id="solar_panel"]').dataset.owned`);
  const before = await owned();
  // The card is the Buy button: a click anywhere on its text buys one.
  await click('.card.building[data-id="solar_panel"].can-buy .desc');
  const after = await owned();
  check(Number(after) === Number(before) + 1, `a click on a green card buys a Solar Panel: ${before} -> ${after}`);
  // Its own controls keep their jobs and buy nothing.
  await click('.card.building[data-id="solar_panel"] button[title="Run one more"]');
  await click('.card.building[data-id="solar_panel"] button[title="Run one fewer"]');
  check((await owned()) === after, `the card's −/+ buttons buy nothing (${await owned()})`);
  await click('.card.building[data-id="solar_panel"] button.sell[title^="Sell one"]');
  check(Number(await owned()) === Number(after) - 1, `the card's Sell 1 sells one and buys none (${after} -> ${await owned()})`);
  // A real mouse press on the card, then space: the pause key, not a second buy (a press
  // must not leave the card focused).
  {
    const p = await evaluate(`(() => { const e = document.querySelector('.card.building[data-id="solar_panel"] .desc'); e.scrollIntoView({ block: 'center' }); const r = e.getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; })()`);
    const n0 = Number(await owned());
    const buyable = await exists('.card.building[data-id="solar_panel"].can-buy');
    for (const type of ['mousePressed', 'mouseReleased']) await send('Input.dispatchMouseEvent', { type, x: p.x, y: p.y, button: 'left', clickCount: 1 });
    await sleep(150);
    const n1 = Number(await owned());
    for (const type of ['keyDown', 'keyUp']) await send('Input.dispatchKeyEvent', { type, key: ' ', code: 'Space', text: type === 'keyDown' ? ' ' : undefined, windowsVirtualKeyCode: 32 });
    await sleep(150);
    const sp = await evaluate(`document.querySelector('#speeds button[aria-pressed="true"]')?.dataset.speed`);
    check((!buyable || n1 === n0 + 1) && Number(await owned()) === n1 && sp !== '0', `space after a click on a card resumes the clock and buys nothing (${n0} -> ${n1} -> ${await owned()}, speed ${sp})`);
    await speed(0);
    if (Number(await owned()) > n0) await click('.card.building[data-id="solar_panel"] button.sell[title^="Sell one"]');
  }
  const kept = await owned();
  const grey =await evaluate(`document.querySelector('.card.building.cannot-buy')?.dataset.id ?? null`);
  if (grey) {
    const greyOwned = () => evaluate(`document.querySelector('.card.building[data-id="${grey}"]').dataset.owned`);
    const g0 = await greyOwned();
    await click(`[data-testid="buy-${grey}"]`);
    const hint = await evaluate(`document.querySelector('.card.building[data-id="${grey}"] .buy-hint').textContent`);
    check((await greyOwned()) === g0 && hint.length > 0, `a grey card buys nothing and says why (${grey}: "${hint}")`);
  }
  const savedDay = await day();
  await load(url);
  await panel('buildings');
  check((await day()) === savedDay && (await owned()) === kept, `reload keeps the save (day ${savedDay}, ${kept} panels)`);
  // As if the next load ran a newer engine: retag the save after each of the page's own
  // leaving saves (pagehide, then visibilitychange; listeners run in order, so ours last).
  await evaluate(`(() => {
    const retag = () => { const s = JSON.parse(localStorage.getItem('hh.save')); s.revision = 999; localStorage.setItem('hh.save', JSON.stringify(s)); };
    window.addEventListener('pagehide', retag);
    document.addEventListener('visibilitychange', retag);
  })()`);
  await load(url);
  const n = await noticeText();
  check(/rules revision 999/.test(n) && (await day()) < savedDay, `revision mismatch discards the save: ${n}`);
  await debugMenu();
  // Paused: a scenario keeps the speed in play, and on a loaded box a day can pass
  // between the start and the read.
  await speed(0);
  await setValue('#scenario', 'at_cap');
  await click('#start-scenario');
  await panel('buildings');
  const [capDay, capOwned] = [await day(), await owned()];
  check(capDay === 100 && capOwned === '20', `debug menu starts scenario at_cap (day ${capDay}, ${capOwned} Solar Panels)`);
  await debugMenu();
  await click('#check-replay');
  check(/^Replay ok/.test(await noticeText()), `replay check: ${await noticeText()}`);

  // ---- 5. readability: capped income, affordable headers, pad readiness, quest pin ----
  await readability(grant);

  // ---- 4. the theme: light by default; the Options toggle switches it, moving nothing ----
  const theme = () => evaluate('document.documentElement.dataset.theme');
  // Every element's box, to the hundredth of a pixel.
  const layout = () =>
    evaluate(`JSON.stringify([...document.querySelectorAll('body *')].map((e) => {
      const r = e.getBoundingClientRect();
      return [r.x, r.y, r.width, r.height].map((n) => Math.round(n * 100)).join();
    }))`);
  const pick = async (t, where = '#theme') => {
    const before = await layout();
    await click(`${where} [data-theme-pick="${t}"]`);
    await sleep(200);
    check((await theme()) === t && (await layout()) === before, `${where} switches to ${t} with nothing moving`);
  };
  check((await theme()) === 'light' && (await evaluate(`localStorage.getItem('hh.theme')`)) === null, 'the game opens in the light theme');
  await speed(0);
  await click('#notice button');
  await panel('commands');
  await evaluate(`[...document.querySelectorAll('[data-testid^="add-"]')].filter((b) => !b.disabled).slice(0, 3).forEach((b) => b.click())`);
  await panel('buildings');
  const looks = {};
  for (const t of ['light', 'dark']) {
    if (t === 'dark') {
      await panel('options');
      await pick('dark');
    }
    for (const p of ['buildings', 'commands']) {
      await panel(p);
      await sleep(200);
      looks[`${t} ${p}`] = await layout();
      await shot(`theme-${t}-${p}`);
    }
  }
  for (const p of ['buildings', 'commands']) check(looks[`light ${p}`] === looks[`dark ${p}`], `${p}: the same layout in both themes`);
  await load(url);
  check((await theme()) === 'dark', 'a reload keeps the dark theme');
  await speed(0);
  await panel('options');
  await debugMenu();
  await pick('light', '#debug-theme');
  await pick('dark');
  await pick('light');
  check(pageErrors.length === 0, `no page errors${pageErrors.length ? `: ${pageErrors.join(' | ')}` : ''}`);
} catch (e) {
  failures.push(String(e));
  console.error(e);
  if (pageErrors.length) console.error(`page errors: ${pageErrors.join(' | ')}`);
  await shot('failure');
} finally {
  ws.close();
  // The browser's child processes outlive a plain kill on Windows (and hold the profile).
  if (process.platform === 'win32') spawnSync('taskkill', ['/T', '/F', '/PID', String(proc.pid)], { stdio: 'ignore' });
  else proc.kill();
}
if (failures.length) {
  console.error(`e2e FAILED: ${failures.length}`);
  process.exit(1);
}
console.log('e2e ok');
