// Real-mouse check of click-to-buy (CDP Input events): single, double and Shift clicks,
// the card's own controls, a grey card, hover cursor, screenshots. HH_URL as for e2e.mjs;
// run from the repo root against `vite preview`; profile and PNGs go to out/015_click_to_buy/.
// Scenario "rich" holds titanium at its cap of 20; a Solar Panel costs 6 x 1.5^owned
// titanium, so from 0 owned exactly two are affordable (6 + 9 = 15; the third, 13.5, isn't).
import { spawn } from 'node:child_process';
import { existsSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = resolve(dirname(fileURLToPath(import.meta.url)), '../../../out/015_click_to_buy');
const url = process.env.HH_URL;
const exe = ['C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe', 'C:/Program Files/Google/Chrome/Application/chrome.exe'].find((b) => existsSync(b));
const port = 9800 + Math.floor(Math.random() * 100);
const profile = resolve(here, 'profile', String(port));
rmSync(profile, { recursive: true, force: true });
mkdirSync(profile, { recursive: true });
const proc = spawn(exe, ['--headless=new', `--remote-debugging-port=${port}`, `--user-data-dir=${profile}`, '--enable-automation', '--window-size=1600,900', '--no-first-run', 'about:blank'], { stdio: 'ignore' });
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
  if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); }
  else if (m.method === 'Runtime.exceptionThrown') errors.push(JSON.stringify(m.params.exceptionDetails));
});
const send = (method, params = {}) => new Promise((r) => { const id = nextId++; pending.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
const ev = async (expr) => (await send('Runtime.evaluate', { expression: expr, awaitPromise: true, returnByValue: true })).result?.result?.value;
const q = JSON.stringify;
const click = (sel) => ev(`document.querySelector(${q(sel)}).click()`);
const setValue = (sel, v) => ev(`(() => { const el = document.querySelector(${q(sel)}); el.value = ${q(v)}; el.dispatchEvent(new Event(el.tagName === 'SELECT' ? 'change' : 'input', { bubbles: true })); })()`);
let failed = 0;
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) failed++; };

await send('Runtime.enable');
await send('Page.enable');
await send('Page.navigate', { url });
for (let i = 0; i < 100 && !(await ev(`!!document.querySelector('.nav-tile')`)); i++) await sleep(100);
await sleep(500);

const ID = 'solar_panel';
const card = `.card.building[data-id="${ID}"]`;
const owned = async () => Number(await ev(`document.querySelector(${q(card)}).dataset.owned`));
const titanium = () => ev(`document.querySelector('tr[data-res="titanium"]').dataset.amount`);
const center = async (sel) => ev(`(() => { const e = document.querySelector(${q(sel)}); e.scrollIntoView({ block: 'center' }); const r = e.getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; })()`);
const mouse = async (sel, { count = 1, modifiers = 0 } = {}) => {
  const [x, y] = await center(sel);
  await send('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y });
  for (let c = 1; c <= count; c++) {
    await send('Input.dispatchMouseEvent', { type: 'mousePressed', x, y, button: 'left', clickCount: c, modifiers });
    await send('Input.dispatchMouseEvent', { type: 'mouseReleased', x, y, button: 'left', clickCount: c, modifiers });
  }
  await sleep(150);
};
/** A fresh "rich" game, paused, with its Solar Panels sold back to none. */
const fresh = async () => {
  await click('.nav-tile[data-nav="options"]'); await sleep(100);
  if (!(await ev(`!!document.querySelector('#scenario')`))) await click('[data-testid="debug"] .fold-head');
  await sleep(100);
  await setValue('#scenario', 'rich');
  await click('#start-scenario'); await sleep(300);
  await click('#speeds button[data-speed="0"]');
  await click('.nav-tile[data-nav="buildings"]'); await sleep(200);
  for (let i = 0; i < 10 && (await owned()) > 0; i++) await mouse(`${card} button[title^="Sell one"]`);
};

await fresh();
check((await owned()) === 0 && (await titanium()) === '20.000000', `fresh: 0 panels, titanium ${await titanium()}`);
{
  const [x, y] = await center(`${card} .owned`); // hover while green
  await send('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y });
  await sleep(150);
}
check((await ev(`getComputedStyle(document.querySelector(${q(card)})).cursor`)) === 'pointer', 'green card shows a pointer');
console.log(`     hover background ${await ev(`getComputedStyle(document.querySelector(${q(card)})).backgroundColor`)}`);
writeFileSync(resolve(here, 'hover.png'), Buffer.from((await send('Page.captureScreenshot', { format: 'png' })).result.data, 'base64'));
await mouse(`${card} .desc`);
check((await owned()) === 1, `single click buys one (${await owned()})`);
await mouse(`${card} .desc`);
check((await owned()) === 2, `a second quick click buys the second (${await owned()})`);
await mouse(`${card} .desc`);
check((await owned()) === 2, `a third, unaffordable, buys nothing (${await owned()}); hint "${await ev(`document.querySelector(${q(card)} + ' .buy-hint').textContent`)}"`);
check((await ev(`getComputedStyle(document.querySelector(${q(card)})).cursor`)) !== 'pointer', 'grey card shows no pointer');
writeFileSync(resolve(here, 'grey.png'), Buffer.from((await send('Page.captureScreenshot', { format: 'png' })).result.data, 'base64'));

await fresh();
await mouse(`${card} h3`, { count: 2 });
check((await owned()) === 2, `double-click buys two (${await owned()})`);
check((await ev(`String(getSelection())`)) === '', 'double-click selects no text');

await fresh();
await mouse(`${card} .effects`, { modifiers: 8 });
check((await owned()) === 2, `Shift-click buys as many as affordable, up to ten (${await owned()})`);
check(!(await ev(`document.getElementById('notice')?.textContent ?? ''`)), 'and stops with no error notice');

const n = await owned();
await mouse(`${card} button[title="Run one fewer"]`);
await mouse(`${card} button[title="Run one more"]`);
check((await owned()) === n, `−/+ buy nothing (${await owned()})`);
await mouse(`${card} button[title^="Sell one"]`);
check((await owned()) === n - 1, `Sell 1 sells one, buys none (${n} -> ${await owned()})`);
// A disabled control (0 owned: −/+ both disabled) on a green card buys nothing either.
await fresh();
await mouse(`${card} button[title="Run one more"]`);
await mouse(`${card} button[title="Run one fewer"]`);
check((await owned()) === 0, `disabled −/+ on a green card buy nothing (${await owned()})`);
// Keyboard: focus the card, Enter buys.
const k0 = await owned();
await ev(`document.querySelector(${q(card)}).focus()`);
await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13 });
await send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13 });
await sleep(150);
check((await owned()) === k0 + 1, `Enter on the focused card buys one (${k0} -> ${await owned()})`);
check(errors.length === 0, `no page errors ${errors.join(' ')}`);
ws.close();
proc.kill();
console.log(failed ? `mouse FAILED: ${failed}` : 'mouse ok');
process.exit(failed ? 1 : 0);
