// 122 probe 2: does a project's first funding step move its −/+ buttons (the hold then
// ends under a still pointer)? Every unlocked project's first +, its y before and after
// one keyboard-free click, at a width.
// Run from the repo root: node experiments/results/122_bug_hunt/probe2.mjs <url> <width>
import { writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { driver, launch, sleep } from '../../../web/scripts/cdp.mjs';

const [url = 'http://localhost:49373/', width = '1280'] = process.argv.slice(2);
const b = await launch({ width: Number(width), height: 900, profileRoot: resolve('out/122_bug_hunt/profiles') });
const d = driver(b, { url });
try {
  await d.load();
  await d.debugMenu();
  await d.speed(0);
  await d.setValue('#scenario', 'late_unlocks');
  await d.click('#start-scenario');
  await d.autoStory();
  await d.speed(0);
  await d.panel('projects');
  await sleep(300);
  const ids = await b.evaluate(`[...document.querySelectorAll('.card.project:not(.locked)')].filter((c) => c.querySelector('.proj-row')).map((c) => c.dataset.id)`);
  for (const id of ids) {
    const sel = `.card.project[data-id="${id}"] .proj-row .tiny:nth-of-type(2)`;
    const y = () => b.evaluate(`document.querySelector('${sel}').getBoundingClientRect().y`);
    await b.send('Page.captureScreenshot', { format: 'png' }).then((r) => writeFileSync(`out/122_bug_hunt/p2-${id}-before.png`, Buffer.from(r.result.data, 'base64')));
    const y0 = await y();
    const r = await b.evaluate(`(() => { const e = document.querySelector('${sel}'); const r = e.getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()`);
    await b.send('Input.dispatchMouseEvent', { type: 'mouseMoved', x: r.x, y: r.y });
    await b.send('Input.dispatchMouseEvent', { type: 'mousePressed', x: r.x, y: r.y, button: 'left', buttons: 1, clickCount: 1 });
    await sleep(1500);
    await b.send('Input.dispatchMouseEvent', { type: 'mouseReleased', x: r.x, y: r.y, button: 'left', buttons: 0, clickCount: 1 });
    await sleep(100);
    const y1 = await y();
    await b.send('Page.captureScreenshot', { format: 'png' }).then((r) => writeFileSync(`out/122_bug_hunt/p2-${id}-after.png`, Buffer.from(r.result.data, 'base64')));
    const rate = await b.evaluate(`document.querySelector('${sel}').closest('.proj-row').querySelector('.proj-rate').textContent`);
    const top = await b.evaluate(`(() => { const p = document.querySelector('[data-panel="projects"]'); return p.getBoundingClientRect().y.toFixed(0) + ' scroll ' + p.closest('main, .col-center, .center')?.scrollTop + ' notice ' + (document.querySelector('.notice')?.textContent ?? 'none'); })()`);
    console.log(`${id}: + y ${y0.toFixed(0)} -> ${y1.toFixed(0)}; after a 1.5 s hold: ${rate}; panel ${top}`);
  }
  console.log('errors', JSON.stringify(b.errors));
} finally {
  b.close();
}
