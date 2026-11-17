// 027's screenshot set: the art in the game at 1280 x 800, light and dark. Drives headless
// Edge or Chrome over the DevTools protocol (web/scripts/e2e.mjs's way, no packages)
// against a served static build.
//   HH_URL  the page (default http://localhost:60550/)
//   OUT     where the PNGs go (default experiments/results/027_art_in_game/shots)
// Native tooltips (an SVG <title>) never paint in a screenshot: the tooltip shot draws a
// stand-in box holding the chip's <title> text, marked as such in the README.
import { spawn } from 'node:child_process';
import { existsSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const url = process.env.HH_URL ?? 'http://localhost:60550/';
const OUT = resolve(process.env.OUT ?? resolve(here, 'shots'));
mkdirSync(OUT, { recursive: true });
const exe = [
  'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
  'C:/Program Files/Google/Chrome/Application/chrome.exe',
].find((b) => existsSync(b));
const port = 9300 + Math.floor(Math.random() * 500);
const profile = resolve(here, '../../../out/027_art_in_game/profile', String(port));
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
ws.addEventListener('message', (ev) => {
  const m = JSON.parse(ev.data);
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
const grant = async (res, amount) => {
  await cheats();
  await setValue('#cheat-resource', res);
  await setValue('#cheat-amount', amount);
  await click('#cheat-grant');
};
const rect = (sel) => evaluate(`(() => { const r = document.querySelector(${q(sel)}).getBoundingClientRect();
  return { x: r.x, y: r.y, width: r.width, height: r.height }; })()`);
/** A PNG of `sel`'s box (padded), at `scale` (2: a close-up). */
const shot = async (name, sel, { scale = 1, pad = 4 } = {}) => {
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
const stage = () => evaluate(`document.querySelector('.horizon')?.dataset.stage`);

await send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 800, deviceScaleFactor: 1, mobile: false });
await send('Page.enable');
await send('Page.navigate', { url });
await waitFor(ready, 'the page');

for (const theme of ['light', 'dark']) {
  await evaluate(`localStorage.clear(); localStorage.setItem('hh.theme', ${q(theme)})`);
  await load();
  await debugMenu();
  await click('#new-game');
  await waitFor(ready, 'a new game');
  await speed(0);
  await panel('buildings');
  // A fresh game: normal nav tiles, Resources, the bare horizon.
  await full(`${theme}-fresh-1280`);
  await shot(`${theme}-nav-normal`, '.nav-grid', { scale: 2 });
  await shot(`${theme}-resources`, '[data-testid="resources"]', { scale: 2 });
  if ((await stage()) !== 'bare') throw new Error(`fresh game: horizon ${await stage()}`);
  await shot(`${theme}-horizon-bare`, '.horizon');

  // A pad: load it, fuel it, launch.
  await debugMenu();
  await setValue('#scenario', 'pad_loading');
  await click('#start-scenario');
  await waitFor(ready, 'the scenario');
  await speed(0);
  if ((await stage()) !== 'pad') throw new Error(`a pad: horizon ${await stage()}`);
  await shot(`${theme}-horizon-pad`, '.horizon');
  await panel('commands');
  await click('[data-testid="add-load_pads"]');
  await speed(1);
  await waitFor(`Number(document.querySelector('[data-pad="0"]')?.dataset.cargo ?? 0) > 0 || !!document.querySelector('.card.pad[data-cargo]:not([data-cargo="0"])')`, 'cargo', 3000).catch(() => {});
  await sleep(2500);
  await speed(0);
  await grant('propellant', '5');
  await panel('launch_pads');
  await shot(`${theme}-markets`, '[data-panel="launch_pads"] .fold:first-of-type', { scale: 2 });
  await shot(`${theme}-pad-card`, '.card.pad', { scale: 2 });
  await click('[data-testid="launch-0"]');
  await waitFor(`document.querySelector('.horizon')?.dataset.stage === 'climb'`, 'the climb', 3000);
  await shot(`${theme}-horizon-climb`, '.horizon');
  await shot(`${theme}-left-column-climb`, '.col-left');
  await waitFor(`document.querySelector('.horizon')?.dataset.stage === 'pad'`, 'back to the pad', 8000);
  console.log('climb returned to pad');
  await shot(`${theme}-event-shipment`, '[data-testid="events-activity"]', { scale: 2 });
  await shot(`${theme}-program-lines`, '.col-right', { scale: 1 });

  // Late game: every tile (compact), ideology badges, rivals. The quest chain to
  // Ideologies: He-3 produced (refineries), a shipment (done), two Data Centers, Market
  // Awareness, then Geopolitical Intelligence.
  const build = async (id, n = 1) => {
    for (let i = 0; i < n; i++) {
      for (const [res, amt] of [['credits', '100000'], ['regolith', '500'], ['titanium', '500'], ['circuits', '500']]) await grant(res, amt);
      await panel('buildings');
      await click(`[data-testid="buy-${id}"]`);
    }
  };
  await build('storage_depot', 3);
  await build('excavator', 3);
  await build('refinery', 4);
  await build('data_center', 2);
  await build('research_lab');
  const jump = async (days) => {
    await cheats();
    await setValue('#cheat-days', String(days));
    await click('#cheat-jump');
    await sleep(300);
  };
  await jump(25);
  await grant('science', '5000');
  await panel('research');
  if (await exists('[data-testid="research-market_awareness"]:not(.done)')) await click('[data-testid="research-market_awareness"]');
  await jump(2);
  await grant('science', '5000');
  await panel('research');
  await waitFor(`!!document.querySelector('[data-testid="research-geopolitical_intelligence"]')`, 'Geopolitical Intelligence', 3000);
  await click('[data-testid="research-geopolitical_intelligence"]');
  await jump(2);
  await waitFor(`!!document.querySelector('.nav-tile[data-nav="ideologies"]')`, 'Ideologies', 3000);
  await cheats();
  await setValue('#cheat-days', '200');
  await click('#cheat-jump');
  await sleep(300);
  await shot(`${theme}-nav-compact`, '.nav-grid', { scale: 2 });
  await panel('buildings');
  await full(`${theme}-late-1280`);
  await shot(`${theme}-card`, '.card.building[data-id="solar_panel"]', { scale: 2 });
  // The ideology badge with a stand-in for its native tooltip.
  await evaluate(`(() => {
    const chip = document.querySelector('.ideology-badge .chip');
    chip.scrollIntoView({ block: 'center' });
    const r = chip.getBoundingClientRect();
    const tip = document.createElement('div');
    tip.id = 'fake-tip';
    tip.textContent = chip.querySelector('title').textContent;
    Object.assign(tip.style, { position: 'fixed', left: r.x + 6 + 'px', top: r.bottom + 14 + 'px', zIndex: 99, font: '12px Segoe UI, sans-serif',
      background: '#fff', color: '#000', border: '1px solid #767676', padding: '2px 6px', boxShadow: '1px 1px 3px rgb(0 0 0 / 30%)' });
    document.body.append(tip);
  })()`);
  const badge = await evaluate(`document.querySelector('.ideology-badge').closest('.card').dataset.id`);
  await shot(`${theme}-ideology-badge-tooltip`, `.card.building[data-id="${badge}"]`, { scale: 2, pad: 30 });
  await evaluate(`document.getElementById('fake-tip').remove()`);
  await panel('ideologies');
  await shot(`${theme}-ideologies`, '[data-panel="ideologies"]');
  await panel('adversaries');
  await shot(`${theme}-adversaries`, '[data-panel="adversaries"]', { scale: 2 });
  await shot(`${theme}-events-late`, '[data-testid="events-activity"]', { scale: 2 });
  await shot(`${theme}-ideology-summary`, '[data-testid="ideology-summary"]', { scale: 2 });
}

ws.close();
proc.kill();
console.log('done');
