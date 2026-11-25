// 090 probe: Run by hand before Programming (paused, mashed, refused), then the
// autopilot on to 1.5 and a retirement, reading what Programming gates.
import { launch, driver, sleep, capture } from '../../../web/scripts/cdp.mjs';
import { resolve } from 'node:path';

const url = 'http://localhost:59564/';
const out = resolve('out/090_bug_hunt/probe1');
const b = await launch({ width: 1280, height: 900, profileRoot: resolve('out/090_bug_hunt/profiles') });
const d = driver(b, { url });
const shot = (n) => capture(b, `${out}/${n}.png`);
const ev = b.evaluate;
const log = (...a) => console.log(...a);
const state = async (tag) => {
  const s = await ev(`(() => ({
    day: document.getElementById('day')?.textContent,
    programsTile: !!document.querySelector('.nav-tile[data-nav="programs"]'),
    rightProgram: !!document.querySelector('.prog-tabs'),
    addButtons: document.querySelectorAll('[data-testid^="add-"]').length,
    runButtons: [...document.querySelectorAll('[data-testid^="run-"]')].map((x) => x.dataset.testid + (x.disabled ? '(off)' : '')).join(' '),
    notice: document.getElementById('notice')?.textContent ?? null,
    nav: [...document.querySelectorAll('.nav-tile')].map((x) => x.dataset.nav).join(','),
    credits: document.querySelector('[data-res="credits"]')?.textContent ?? null,
  }))()`);
  log(tag, JSON.stringify(s));
  return s;
};
try {
  await d.load();
  await d.readStory();
  await d.speed(0);
  await d.panel('commands');
  await state('fresh-commands');
  await shot('fresh-commands');
  // Mash Run on Sell Cloud Compute while paused.
  await ev(`(() => { const b = document.querySelector('[data-testid="run-cloud_compute"]'); for (let i = 0; i < 60; i++) b.click(); })()`);
  await sleep(500);
  await state('after-60-runs-paused');
  await shot('after-mash');
  // Each other Run, once.
  const ids = await ev(`[...document.querySelectorAll('[data-testid^="run-"]:not([disabled])')].map((x) => x.dataset.testid)`);
  for (const id of ids) {
    await ev(`document.querySelector('[data-testid="${id}"]').click()`);
    await sleep(150);
    log('run', id, await ev(`document.getElementById('notice')?.textContent ?? ''`));
  }
  await shot('after-each-run');
  // Resources breakdown hover: the hand source label.
  log('errors so far', JSON.stringify(b.errors));
  // Autopilot quest steps to 1.5, watching the gates.
  for (let step = 0; step < 8; step++) {
    await d.cheats();
    await d.click('#accel-quest');
    await d.waitFor(`document.getElementById('accel-status')?.dataset.running === '0'`, 'step', 90000);
    await d.readStory();
    if (await d.exists('[data-testid="continue"]')) await d.click('[data-testid="continue"]');
    await d.readStory();
    const quest = await ev(`document.querySelector('.col-right')?.textContent.match(/\\d\\.\\d+ —[^\\n]{0,40}/)?.[0] ?? ''`);
    const s = await state(`step ${step + 1} ${quest} run ${await d.run()}`);
    await shot(`step-${step + 1}`);
    if (s.programsTile && step >= 4) break;
  }
  // Programs view open, then retire via the Retirement tile if any.
  if (await d.exists('.nav-tile[data-nav="programs"]')) {
    await d.click('.nav-tile[data-nav="programs"]');
    await sleep(400);
    await shot('programs-view');
    await d.click('.nav-tile[data-nav="programs"]');
  }
  await d.panel('research');
  await shot('research-after');
  log('research cards', await ev(`[...document.querySelectorAll('[data-panel="research"] .card')].map((c) => c.querySelector('h3,.name,strong')?.textContent).join(' | ')`));
  await d.panel('commands');
  await state('commands-after');
  await shot('commands-after');
  log('errors', JSON.stringify(b.errors));
} catch (e) {
  log('PROBE FAILED', e.stack);
  await shot('failed').catch(() => {});
} finally {
  b.close();
}
