// 122 probe 1: Projects' hold-to-repeat under a real mouse, the right column's popups
// (side quests, Events), and screenshots of the changed panels at a width.
// Run from the repo root: node experiments/results/122_bug_hunt/probe1.mjs <url> <width> <outdir>
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { capture, driver, launch, q, sleep } from '../../../web/scripts/cdp.mjs';

const [url = 'http://localhost:49373/', width = '1280', out = 'out/122_bug_hunt/probe1'] = process.argv.slice(2);
const W = Number(width);
mkdirSync(out, { recursive: true });
const b = await launch({ width: W, height: 900, profileRoot: resolve('out/122_bug_hunt/profiles') });
const d = driver(b, { url });
const notes = [];
const note = (s) => {
  notes.push(s);
  console.log(s);
};
const shot = (name) => capture(b, `${out}/${W}-${name}.png`);
const center = (sel) =>
  b.evaluate(`(() => { const e = document.querySelector(${q(sel)}); if (!e) return null; const r = e.getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2, dis: e.disabled }; })()`);
const mouse = (type, x, y) => b.send('Input.dispatchMouseEvent', { type, x, y, button: 'left', buttons: type === 'mouseReleased' ? 0 : 1, clickCount: 1 });
const rateOf = (id, res) => b.evaluate(`(() => { const p = window.__hh?.store?.view?.projects?.find((x) => x.id === ${q(id)}); return p ? p.rates[${q(res)}] : null; })()`);
const rateText = (sel) => b.evaluate(`document.querySelector(${q(sel)})?.closest('.proj-row')?.querySelector('.proj-rate')?.textContent ?? null`);

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
  await shot('projects');
  // The first unlocked project's first + and −.
  const rows = await b.evaluate(`[...document.querySelectorAll('.card.project:not(.locked) .proj-row')].map((r, i) => ({ i, card: r.closest('.card').dataset.id, res: r.querySelector('.proj-res').textContent.trim() }))`);
  note(`project rows: ${JSON.stringify(rows.slice(0, 6))}`);
  if (rows.length) {
    const card = rows[0].card;
    const plus = `.card.project[data-id="${card}"] .proj-row button:nth-of-type(2)`;
    const minus = `.card.project[data-id="${card}"] .proj-row button:nth-of-type(1)`;
    const max = `.card.project[data-id="${card}"] .proj-row button:nth-of-type(3)`;
    note(`buttons: ${await b.evaluate(`[...document.querySelectorAll('.card.project[data-id="${card}"] .proj-row:first-of-type button')].map((x) => x.textContent + (x.disabled ? '(dis)' : '')).join(' ')`)}`);
    const p = await center(plus);
    const t0 = await rateText(plus);
    await mouse('mouseMoved', p.x, p.y);
    await mouse('mousePressed', p.x, p.y);
    await sleep(1800);
    const held = await rateText(plus);
    await mouse('mouseReleased', p.x, p.y);
    await sleep(100);
    const rel = await rateText(plus);
    await sleep(700);
    const after = await rateText(plus);
    note(`hold +: before ${t0}, at 1.8s ${held}, at release ${rel}, 0.7s later ${after} (${rel === after ? 'stopped' : 'KEPT STEPPING'})`);
    // Press, then leave the button while held: the hold must end.
    await mouse('mousePressed', p.x, p.y);
    await sleep(600);
    await mouse('mouseMoved', p.x, p.y + 200);
    const left = await rateText(plus);
    await sleep(700);
    const left2 = await rateText(plus);
    await mouse('mouseReleased', p.x, p.y + 200);
    note(`hold + then leave: at leave ${left}, 0.7s later ${left2} (${left === left2 ? 'stopped' : 'KEPT STEPPING'})`);
    // Hold − to the bound.
    const m = await center(minus);
    await mouse('mouseMoved', m.x, m.y);
    await mouse('mousePressed', m.x, m.y);
    await sleep(4000);
    const low = await rateText(minus);
    await mouse('mouseReleased', m.x, m.y);
    note(`hold − 4s: ${low}; − disabled: ${(await center(minus)).dis}`);
    // Max, then hold + at the cap (disabled: nothing).
    await d.click(max);
    await sleep(200);
    note(`after Max: ${await rateText(plus)}; + disabled: ${(await center(plus)).dis}`);
    // Run a few days funded at 1x and read the row.
    await d.speed(1);
    await sleep(3000);
    await d.speed(0);
    await shot('projects-funded');
    note(`funded card text: ${await b.evaluate(`document.querySelector('.card.project[data-id="${card}"]').innerText.replace(/\\s+/g, ' ')`)}`);
  }
  for (const id of ['research', 'ideologies']) {
    try {
      await d.panel(id);
      await sleep(300);
      await shot(id);
    } catch (e) {
      note(`panel ${id}: ${e.message}`);
    }
  }
  // Right column: side quests and Events.
  const side = await b.evaluate(`[...document.querySelectorAll('[data-side-quest]')].map((x) => x.dataset.sideQuest)`);
  note(`side quests listed: ${JSON.stringify(side)}`);
  await b.evaluate('window.__autoStory?.disconnect(); window.__autoStory = null');
  await d.speed(3);
  if (side.length) {
    await d.click(`[data-side-quest="${side[0]}"]`);
    await sleep(200);
    const modal = await b.evaluate(`(() => { const m = document.querySelector('[data-testid="story-modal"]'); return m ? m.innerText.replace(/\\s+/g, ' ').slice(0, 300) : null; })()`);
    const paused = await b.evaluate(`document.querySelector('#speeds button[aria-pressed="true"]')?.dataset.speed`);
    await shot('side-popup');
    note(`side quest popup: ${modal}; speed while open ${paused}`);
    if (modal) await d.click('[data-testid="story-next"]');
    await sleep(200);
    note(`speed after closing: ${await b.evaluate(`document.querySelector('#speeds button[aria-pressed="true"]')?.dataset.speed`)}`);
  }
  const ev = await b.evaluate(`[...document.querySelectorAll('[data-event]')].map((x) => x.dataset.event)`);
  note(`events listed: ${JSON.stringify(ev)}`);
  if (ev.length) {
    await d.click(`[data-event="${ev[0]}"]`);
    await sleep(200);
    await shot('event-popup');
    note(`event popup: ${await b.evaluate(`document.querySelector('[data-testid="story-modal"]')?.innerText.replace(/\\s+/g, ' ').slice(0, 300) ?? null`)}`);
    await d.readStory();
  }
  await d.speed(0);
  await shot('right');
  note(`errors: ${JSON.stringify(b.errors)}`);
} catch (e) {
  note(`FAILED: ${e.stack}`);
  await shot('failed').catch(() => {});
} finally {
  writeFileSync(`${out}/${W}-notes.txt`, notes.join('\n') + '\n');
  b.close();
}
