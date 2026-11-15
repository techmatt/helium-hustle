// Browser check: drives headless Edge or Chrome over the DevTools protocol (no
// packages) against a served page. Checks each speed ticks at its rate, buying (in the
// rich scenario), the autosave surviving a reload, a revision-mismatch save discarded,
// scenarios started from the debug menu, a command set on program 1 and the replay check.
//   HH_URL      the page (default http://localhost:5173/)
//   HH_BROWSER  a browser off the usual paths
//   HH_PROFILE  browser profile dir (default ../out/e2e-profile, wiped first)
import { spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, rmSync } from 'node:fs';
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
  '--no-first-run', '--no-default-browser-check', 'about:blank',
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
ws.addEventListener('message', (ev) => {
  const msg = JSON.parse(ev.data);
  if (msg.id && pending.has(msg.id)) {
    pending.get(msg.id)(msg);
    pending.delete(msg.id);
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
const day = async () => Number((await evaluate(`document.getElementById('day').textContent`)).replace(/\D/g, ''));
const PANEL = '.building[data-id="solar_panel"]';
const owned = () => evaluate(`document.querySelector('${PANEL} span')?.textContent ?? ''`);
const noticeText = () => evaluate(`document.getElementById('notice').hidden ? '' : document.getElementById('notice').textContent`);
const click = (selector) => evaluate(`document.querySelector(${JSON.stringify(selector)}).click()`);
const waitFor = async (expr, what, ms = 10000) => {
  for (const end = Date.now() + ms; Date.now() < end; await sleep(100)) if (await evaluate(expr)) return;
  throw new Error(`timed out waiting for ${what}`);
};
const load = async (u) => {
  // Mark the old document, so the wait below can't pass on it before the new one commits.
  await evaluate('window.__stale = true');
  await send('Page.navigate', { url: u });
  await waitFor('!window.__stale', 'the navigation');
  // The page says "Loading…" until the engine has rendered once.
  await waitFor(`/^Day \\d/.test(document.getElementById('day')?.textContent ?? '') && !!document.querySelector('.building button')`, 'the page');
};

const failures = [];
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) failures.push(what);
};

try {
  await send('Page.enable');
  await load(url);
  check(true, `page loads: ${await day()}`);
  // A fresh game can't afford a building: the rich scenario can.
  await evaluate(`document.getElementById('scenario').value = 'rich'`);
  await click('#start-scenario');
  check((await day()) === 0 && /Started scenario rich/.test(await noticeText()), 'debug menu starts scenario rich');
  // Each speed's rate, measured over wall time (loose bounds: timers jitter).
  for (const [speed, label, secs] of [[1, '1x', 3], [3, '3x', 2], [10, '10x', 2], [50, '50x', 1], [200, '200x', 1]]) {
    await click(`#speeds button[data-speed="${speed}"]`);
    const d0 = await day();
    const t0 = Date.now();
    await sleep(secs * 1000);
    const rate = ((await day()) - d0) / ((Date.now() - t0) / 1000);
    check(rate > speed * 0.6 && rate < speed * 1.4, `${label}: ${rate.toFixed(1)} days/s`);
  }
  await click('#speeds button[data-speed="0"]');
  const paused = await day();
  await sleep(1200);
  check((await day()) === paused, 'pause holds the day');
  const enabled = await evaluate(`!document.querySelector('${PANEL} button').disabled`);
  check(enabled, 'Solar Panel affordable in the rich scenario');
  const before = await owned();
  await click(`${PANEL} button`);
  const after = await owned();
  check(before !== after && /\d+ owned/.test(after), `buy: ${before} -> ${after}`);
  const savedDay = await day();
  await load(url);
  check((await day()) === savedDay && (await owned()) === after, `reload keeps the save (day ${savedDay}, ${after})`);
  // As if the next load ran a newer engine: retag the save after each of the page's own
  // leaving saves (pagehide, then visibilitychange; listeners run in order, so ours last).
  await evaluate(`(() => {
    const retag = () => { const s = JSON.parse(localStorage.getItem('hh.save')); s.revision = 999; localStorage.setItem('hh.save', JSON.stringify(s)); };
    window.addEventListener('pagehide', retag);
    document.addEventListener('visibilitychange', retag);
  })()`);
  await load(url);
  const n = await noticeText();
  check(/rules revision 999/.test(n) && (await day()) < savedDay,
    `revision mismatch discards the save: ${n || `no notice; day ${await day()}, save ${await evaluate(`localStorage.getItem('hh.save')?.slice(0, 40)`)}`}`);
  await evaluate(`document.getElementById('scenario').value = 'at_cap'`);
  await click('#start-scenario');
  check((await day()) === 100 && /20 owned/.test(await owned()), 'debug menu starts scenario at_cap');
  await click('#speeds button[data-speed="50"]');
  await sleep(1000);
  await click('#speeds button[data-speed="0"]');
  await click('#commands button[data-id="cloud_compute"]');
  check(/Running Sell Cloud Compute/.test(await evaluate(`document.getElementById('program').textContent`)), 'a command runs on program 1');
  await click('#speeds button[data-speed="50"]');
  await sleep(500);
  await click('#speeds button[data-speed="0"]');
  await click('.building button:not([disabled])').catch(() => {});
  await click('#check-replay');
  check(/^Replay ok/.test(await noticeText()), `replay check: ${await noticeText()}`);
  const errors = await evaluate(`document.getElementById('notice').textContent.includes('failed')`);
  check(!errors, 'no engine errors');
} catch (e) {
  failures.push(String(e));
  console.error(e);
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
