// 122 probe 4: odd input on the right column's popups (112). Scenario late_unlocks at
// 10x: an Events line clicked five times with a real mouse (one popup, one page), a
// side-quest line clicked under it (refused), Space and 3 while it is up (the clock
// held), Esc, the OK double-clicked (what the second click lands on), then a reload with
// a popup up (the clock after).
// Run from the repo root: node experiments/results/122_bug_hunt/probe4.mjs <url>
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { capture, driver, launch, q, sleep } from '../../../web/scripts/cdp.mjs';

const [url = 'http://localhost:49373/'] = process.argv.slice(2);
const out = 'out/122_bug_hunt/probe4';
mkdirSync(out, { recursive: true });
const b = await launch({ width: 1280, height: 900, profileRoot: resolve('out/122_bug_hunt/profiles') });
const d = driver(b, { url });
const log = [];
const say = (s) => {
  log.push(s);
  console.log(s);
};
const pressed = () => b.evaluate(`document.querySelector('#speeds button[aria-pressed="true"]')?.dataset.speed ?? '?'`);
const at = (sel) => b.evaluate(`(() => { const e = document.querySelector(${q(sel)}); if (!e) return null; e.scrollIntoView({ block: 'center' }); const r = e.getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()`);
const click = async (p, n = 1) => {
  await b.send('Input.dispatchMouseEvent', { type: 'mouseMoved', x: p.x, y: p.y });
  for (let i = 1; i <= n; i++) {
    await b.send('Input.dispatchMouseEvent', { type: 'mousePressed', x: p.x, y: p.y, button: 'left', buttons: 1, clickCount: i });
    await b.send('Input.dispatchMouseEvent', { type: 'mouseReleased', x: p.x, y: p.y, button: 'left', buttons: 0, clickCount: i });
  }
};
const key = async (k, code, vk) => {
  await b.send('Input.dispatchKeyEvent', { type: 'keyDown', key: k, code, windowsVirtualKeyCode: vk, text: k.length === 1 ? k : undefined });
  await b.send('Input.dispatchKeyEvent', { type: 'keyUp', key: k, code, windowsVirtualKeyCode: vk });
  await sleep(150);
};
const modal = () => b.evaluate(`(() => { const m = document.querySelector('[data-testid="story-modal"]'); return m ? m.dataset.quest + ' | ' + (m.querySelector('[data-testid="story-next"]')?.textContent ?? '') : null; })()`);
try {
  await d.load();
  await d.debugMenu();
  await d.speed(0);
  await d.setValue('#scenario', 'late_unlocks');
  await d.click('#start-scenario');
  await d.readStory();
  await d.speed(10);
  await sleep(1500);
  while (await d.exists('[data-testid="story-modal"]')) await d.readStory();
  await d.speed(10);
  // Past the half second after the last page turned, when clicks outside a modal are ignored.
  await sleep(700);
  const ev = await at('[data-event]');
  say(`event line at ${JSON.stringify(ev)}; speed ${await pressed()}`);
  await click(ev);
  await sleep(40);
  await click(ev);
  await click(ev);
  await click(ev);
  await click(ev);
  await sleep(200);
  say(`after 5 clicks: modal ${await modal()}, speed ${await pressed()}`);
  const side = await at('[data-side-quest]');
  if (side) {
    await click(side);
    await sleep(150);
    say(`side line clicked under it: modal ${await modal()}`);
  }
  const day0 = await d.day();
  await key(' ', 'Space', 32);
  await key('3', 'Digit3', 51);
  await sleep(800);
  say(`space and 3: speed ${await pressed()}, day ${day0} -> ${await d.day()}`);
  await key('Escape', 'Escape', 27);
  say(`Esc: modal ${await modal()}`);
  await capture(b, `${out}/popup.png`);
  // The OK double-clicked: the second click lands on what is under it.
  const ok = await at('[data-testid="story-next"]');
  const under = await b.evaluate(`(() => { const m = document.querySelector('.modal-back, .modal-backdrop, [data-testid="story-modal"]'); return null; })()`);
  await click(ok, 2);
  await sleep(300);
  say(`OK double-clicked: modal ${await modal()}, speed ${await pressed()}, under it: ${await b.evaluate(`document.elementFromPoint(${ok.x}, ${ok.y})?.outerHTML.slice(0, 120)`)}`);
  // A popup up, then a reload.
  await d.speed(3);
  const ev2 = await at('[data-event]');
  await click(ev2);
  await sleep(300);
  say(`before reload: modal ${await modal()}, speed ${await pressed()}`);
  await d.load();
  await sleep(1500);
  say(`after reload: modal ${await modal()}, speed ${await pressed()}`);
  await capture(b, `${out}/reloaded.png`);
  say(`errors: ${JSON.stringify(b.errors)}`);
} catch (e) {
  say(`FAILED: ${e.stack}`);
  await capture(b, `${out}/failed.png`).catch(() => {});
} finally {
  writeFileSync(`${out}/notes.txt`, log.join('\n') + '\n');
  b.close();
}
