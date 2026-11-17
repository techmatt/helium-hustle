// 029's screenshot set: programs paid first and the resource breakdown, on the
// `late_unlocks` scenario, at 1280 x 800, light and dark. Drives headless Edge or Chrome over
// the DevTools protocol (027's shots.mjs, no packages) against a served static build, and
// checks what each shot is for (exit 1 if not).
//   HH_URL  the page (default http://localhost:52085/)
//   OUT     where the PNGs go (default experiments/results/029_energy_priority_breakdown/shots)
import { spawn } from 'node:child_process';
import { existsSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const url = process.env.HH_URL ?? 'http://localhost:52085/';
const OUT = resolve(process.env.OUT ?? resolve(here, 'shots'));
mkdirSync(OUT, { recursive: true });
const exe = [
  'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
  'C:/Program Files/Google/Chrome/Application/chrome.exe',
].find((b) => existsSync(b));
const port = 9300 + Math.floor(Math.random() * 500);
const profile = resolve(here, '../../../out/029_energy_priority_breakdown/profile', String(port));
rmSync(profile, { recursive: true, force: true });
mkdirSync(profile, { recursive: true });
const proc = spawn(exe, ['--headless=new', `--remote-debugging-port=${port}`, `--user-data-dir=${profile}`, '--enable-automation',
  '--window-size=1280,800', '--no-first-run', '--no-default-browser-check', 'about:blank'], { stdio: 'ignore' });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let page;
for (let i = 0; i < 150 && !page; i++) {
  await sleep(200);
  const t = await fetch(`http://127.0.0.1:${port}/json/list`).then((r) => r.json()).catch(() => []);
  page = t.find((x) => x.type === 'page');
}
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener('open', r, { once: true }));
let nextId = 1;
const pending = new Map();
const errors = [];
ws.addEventListener('message', (ev) => {
  const m = JSON.parse(ev.data);
  if (m.method === 'Runtime.exceptionThrown') errors.push(m.params.exceptionDetails?.exception?.description ?? 'page error');
  if (m.id && pending.has(m.id)) {
    pending.get(m.id)(m);
    pending.delete(m.id);
  }
});
const send = (method, params = {}) => new Promise((r) => {
  const id = nextId++;
  pending.set(id, r);
  ws.send(JSON.stringify({ id, method, params }));
});
const evaluate = async (expr) => {
  const r = await send('Runtime.evaluate', { expression: expr, awaitPromise: true, returnByValue: true });
  if (r.result?.exceptionDetails) throw new Error(`${expr}: ${JSON.stringify(r.result.exceptionDetails)}`);
  return r.result?.result?.value;
};
const q = (s) => JSON.stringify(s);
const exists = (sel) => evaluate(`!!document.querySelector(${q(sel)})`);
const text = (sel) => evaluate(`document.querySelector(${q(sel)})?.textContent ?? ''`);
const click = async (sel) => {
  if (!(await evaluate(`(() => { const b = document.querySelector(${q(sel)}); if (!b) return false; b.click(); return true; })()`)))
    throw new Error(`nothing to click at ${sel}`);
};
const setValue = (sel, v) => evaluate(`(() => { const el = document.querySelector(${q(sel)}); el.value = ${q(v)};
  el.dispatchEvent(new Event(el.tagName === 'SELECT' ? 'change' : 'input', { bubbles: true })); })()`);
const waitFor = async (expr, what, ms = 10000) => {
  for (const end = Date.now() + ms; Date.now() < end; await sleep(100)) if (await evaluate(expr)) return;
  throw new Error(`timed out waiting for ${what}`);
};
const ready = `/^Day \\d/.test(document.getElementById('day')?.textContent ?? '') && !!document.querySelector('.nav-tile')`;
const load = async () => {
  await evaluate('window.__stale = true');
  await send('Page.navigate', { url });
  await waitFor('!window.__stale', 'navigation');
  await waitFor(ready, 'the page');
  await sleep(500);
};
const speed = (s) => click(`#speeds button[data-speed="${s}"]`);
const panel = async (id) => {
  await click(`.nav-tile[data-nav="${id}"]`);
  await waitFor(`!!document.querySelector('[data-panel="${id}"]')`, `the ${id} panel`, 3000);
};
const debugMenu = async () => {
  await panel('options');
  if (!(await exists('#scenario'))) await click('[data-testid="debug"] .fold-head');
};
const cheats = async () => {
  await debugMenu();
  if (!(await exists('[data-testid="cheats"]'))) await click('#cheats-toggle');
};
const jump = async (days) => {
  await cheats();
  await setValue('#cheat-days', String(days));
  await click('#cheat-jump');
  await sleep(300);
};
const grant = async (res, amount) => {
  await cheats();
  await setValue('#cheat-resource', res);
  await setValue('#cheat-amount', amount);
  await click('#cheat-grant');
};
const rect = (sel) => evaluate(`(() => { const r = document.querySelector(${q(sel)}).getBoundingClientRect();
  return { x: r.x, y: r.y, width: r.width, height: r.height }; })()`);
const shot = async (name, sel, { scale = 1, pad = 4 } = {}) => {
  // The columns scroll on their own: bring the element on screen first.
  await evaluate(`document.querySelector(${q(sel)}).scrollIntoView({ block: 'start' })`);
  await sleep(150);
  const r = await rect(sel);
  const clip = { x: Math.max(0, r.x - pad), y: Math.max(0, r.y - pad), width: r.width + 2 * pad, height: r.height + 2 * pad, scale };
  const s = await send('Page.captureScreenshot', { format: 'png', clip, captureBeyondViewport: true });
  writeFileSync(resolve(OUT, `${name}.png`), Buffer.from(s.result.data, 'base64'));
  console.log(`shot ${name}`);
};
const full = async (name) => {
  const s = await send('Page.captureScreenshot', { format: 'png' });
  writeFileSync(resolve(OUT, `${name}.png`), Buffer.from(s.result.data, 'base64'));
  console.log(`shot ${name}`);
};
const failures = [];
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) failures.push(what);
};

await send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 800, deviceScaleFactor: 1, mobile: false });
await send('Page.enable');
await send('Runtime.enable');
await send('Page.navigate', { url });
await waitFor(ready, 'the page');

for (const theme of ['light', 'dark']) {
  await evaluate(`localStorage.clear(); localStorage.setItem('hh.theme', ${q(theme)})`);
  await load();
  await debugMenu();
  await setValue('#scenario', 'late_unlocks');
  await click('#start-scenario');
  await waitFor(ready, 'the scenario');
  await speed(0);
  check((await evaluate(`document.querySelectorAll('.nav-tile').length`)) === 11, `${theme}: all 11 nav tiles`);
  await jump(2);

  // Programs first: Load Launch Pads ran, Buy Titanium failed and says why.
  await panel('buildings');
  const line0 = await evaluate(`document.querySelector('.lines .line[data-line="0"]')?.className ?? ''`);
  check(line0.includes('out-ran'), `${theme}: Load Launch Pads ran (${line0.trim()})`);
  const why = await text('.lines .line[data-line="1"] [data-line-why]');
  // Day 1 ran it at half (a conversion runs part way); since then it has 0 credits. The
  // chip's text is its <title>, "Credits".
  check(/needs\s*Credits20, had 0/.test(why), `${theme}: the failed line says why ("${why.trim()}")`);
  const status = await evaluate(`[...document.querySelectorAll('[data-card-status]')].map((e) => e.textContent).filter(Boolean)`);
  check(status.some((t) => /running at \d+% · Energy short/.test(t)), `${theme}: a starved card says so (${JSON.stringify(status)})`);
  const heights = await evaluate(`[...new Set([...document.querySelectorAll('[data-card-status]')].map((e) => e.getBoundingClientRect().height))]`);
  check(heights.length === 1, `${theme}: every card's status row is the same height (${heights})`);
  await full(`${theme}-buildings-1280`);
  const starved = await evaluate(`[...document.querySelectorAll('.card.building')].find((c) => /running at/.test(c.querySelector('[data-card-status]')?.textContent ?? ''))?.dataset.id`);
  await shot(`${theme}-card-starved`, `.card.building[data-id="${starved}"]`, { scale: 2 });
  await shot(`${theme}-card-full`, '.card.building[data-id="solar_panel"]', { scale: 2 });
  await shot(`${theme}-program-lines`, '.col-right .lines', { scale: 2, pad: 8 });

  // An expanded Energy row: sources, sinks, who went short.
  await click('tr[data-res="energy"]');
  await waitFor(`!!document.querySelector('[data-res-breakdown="energy"]')`, 'the Energy breakdown', 3000);
  const rows = await evaluate(`[...document.querySelectorAll('[data-res-breakdown="energy"] tr[data-source]')].map((r) => r.dataset.source)`);
  check(rows.includes('building:solar_panel') && rows.includes('program:0'), `${theme}: Energy rows ${JSON.stringify(rows)}`);
  check(await exists('[data-res-breakdown="energy"] [data-short]'), `${theme}: Energy breakdown lists who went short`);
  await full(`${theme}-energy-expanded-1280`);
  await shot(`${theme}-energy-expanded`, '.col-left');
  // One open at a time; a second click closes.
  await click('tr[data-res="credits"]');
  check(!(await exists('[data-res-breakdown="energy"]')) && (await exists('[data-res-breakdown="credits"]')), `${theme}: one row open at a time`);
  await click('tr[data-res="credits"]');
  check(!(await exists('.res-breakdown')), `${theme}: a second click closes it`);

  // The Stats table, every resource.
  await panel('stats');
  const cards = await evaluate(`document.querySelectorAll('[data-testid="stats-breakdown"] .breakdown').length`);
  check(cards >= 5, `${theme}: Stats breakdown covers ${cards} resources`);
  check(await exists('[data-testid="stats-breakdown"] [data-breakdown="energy"] [data-short]'), `${theme}: Stats Energy shows shortfalls`);
  await full(`${theme}-stats-1280`);
  // Taller than the window: the window scrolled to it, then on down.
  await evaluate(`document.querySelector('[data-testid="stats-breakdown"]').scrollIntoView({ block: 'start' })`);
  await sleep(150);
  await full(`${theme}-stats-breakdown-1280`);
  await evaluate(`document.querySelector('[data-testid="stats-breakdown"] .breakdown-card:nth-child(5)')?.scrollIntoView({ block: 'start' })`);
  await sleep(150);
  await full(`${theme}-stats-breakdown-more-1280`);

  // Pads: one paused (yellow), the other loading (neutral) or ready once fueled and full.
  await panel('launch_pads');
  await evaluate(`[...document.querySelectorAll('[data-pad="1"] button')].find((b) => /Pause loading/.test(b.textContent))?.click()`);
  await sleep(200);
  check((await evaluate(`document.querySelector('[data-pad="1"]').dataset.padState`)) === 'paused', `${theme}: a paused pad reads paused`);
  await shot(`${theme}-pads`, '[data-panel="launch_pads"] .cards', { scale: 1 });

  // Land: the whole row buys.
  await grant('credits', '100000');
  await panel('buildings');
  const before = await text('[data-testid="land-bar"] .muted');
  await click('[data-testid="land-bar"] .land-offer');
  await sleep(200);
  const after = await text('[data-testid="land-bar"] .muted');
  check(before !== after, `${theme}: a click on the Land row buys land (${before} -> ${after})`);
  await shot(`${theme}-land-row`, '[data-testid="land-bar"]', { scale: 2 });
}

check(errors.length === 0, `no page errors ${JSON.stringify(errors)}`);
ws.close();
proc.kill();
console.log(failures.length ? `${failures.length} failed` : 'done');
process.exit(failures.length ? 1 : 0);
