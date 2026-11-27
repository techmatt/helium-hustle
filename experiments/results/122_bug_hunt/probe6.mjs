// 122 probe 6: long frames in scenario everything (a late career): opening Projects, then
// a held + (each step an action: apply, save, redraw), read by a longtask observer.
// Run from the repo root: node experiments/results/122_bug_hunt/probe6.mjs <url>
import { resolve } from 'node:path';
import { driver, launch, sleep } from '../../../web/scripts/cdp.mjs';

const [url = 'http://localhost:49373/'] = process.argv.slice(2);
const b = await launch({ width: 1280, height: 900, profileRoot: resolve('out/122_bug_hunt/profiles') });
const d = driver(b, { url });
const longs = async (what) => {
  const l = await b.evaluate(`(() => { const l = window.__long.splice(0); return l.map((x) => Math.round(x)).join(','); })()`);
  console.log(`${what}: long tasks [${l}]`);
};
try {
  await d.load();
  await d.debugMenu();
  await d.speed(0);
  await d.setValue('#scenario', 'everything');
  await d.click('#start-scenario');
  await d.autoStory();
  await d.speed(0);
  await sleep(1500);
  await b.evaluate(`window.__long = []; new PerformanceObserver((l) => { for (const e of l.getEntries()) window.__long.push(e.duration); }).observe({ type: 'longtask' })`);
  console.log(`save size ${await b.evaluate(`(localStorage.getItem('hh.save') ?? '').length`)}`);
  await d.panel('projects');
  await sleep(500);
  await longs('open Projects');
  const at = await b.evaluate(`(() => { const r = document.querySelector('.card.project[data-id] .proj-row'); const bt = [...r.querySelectorAll('button')].find((x) => x.textContent.trim() === '+'); bt.scrollIntoView({ block: 'center' }); const q = bt.getBoundingClientRect(); return { x: q.x + q.width / 2, y: q.y + q.height / 2 }; })()`);
  await sleep(300);
  await longs('scroll');
  await b.send('Input.dispatchMouseEvent', { type: 'mouseMoved', x: at.x, y: at.y });
  await b.send('Input.dispatchMouseEvent', { type: 'mousePressed', x: at.x, y: at.y, button: 'left', buttons: 1, clickCount: 1 });
  await sleep(1200);
  await b.send('Input.dispatchMouseEvent', { type: 'mouseReleased', x: at.x, y: at.y, button: 'left', buttons: 0, clickCount: 1 });
  await sleep(300);
  await longs('hold + 1.2 s');
  const t = await b.evaluate(`(() => { const t0 = performance.now(); document.querySelector('.card.project[data-id] .proj-row button').click(); return performance.now() - t0; })()`);
  console.log(`one − click: ${Math.round(t)} ms in the handler`);
  // The Cortex mock (qa opens it just before its project probe): open, Esc, toggle off.
  await d.panel('options');
  if (!(await d.exists('#cortex-mock-toggle'))) await d.click('[data-testid="debug"] .fold-head');
  await d.click('#cortex-mock-toggle');
  await sleep(300);
  await longs('cortex toggle on');
  await d.click('[data-takeover-button="cortex"]');
  await sleep(500);
  await longs('cortex open');
  await b.evaluate(`document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))`);
  await sleep(500);
  await longs('cortex closed by Esc');
  await d.click('#cortex-mock-toggle');
  await sleep(300);
  await longs('cortex toggle off');
  await d.panel('projects');
  await sleep(500);
  await longs('Projects again');
  console.log('errors', JSON.stringify(b.errors));
} finally {
  b.close();
}
