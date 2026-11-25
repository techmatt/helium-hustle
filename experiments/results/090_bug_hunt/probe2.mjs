// 090 probe 2: a real mouse click on a Commands card's Run, then Space (the pause key)
// and a held Enter: does the focused button run the command again?
import { launch, driver, sleep } from '../../../web/scripts/cdp.mjs';
import { resolve } from 'node:path';

const url = process.env.HH_URL ?? 'http://localhost:59564/';
const b = await launch({ width: 1280, height: 900, profileRoot: resolve('out/090_bug_hunt/profiles') });
const d = driver(b, { url });
const ev = b.evaluate;
const credits = () => ev(`Number(document.querySelector('[data-res="credits"] td.num, [data-res="credits"] .num')?.textContent.replace(/[^\\d.]/g, '') ?? NaN)`);
const energy = () => ev(`document.querySelector('[data-res="energy"]')?.textContent`);
const key = async (k, code, vk, repeat = false) => {
  await b.send('Input.dispatchKeyEvent', { type: 'keyDown', key: k, code, windowsVirtualKeyCode: vk, text: k === 'Enter' ? '\r' : k, autoRepeat: repeat });
  await b.send('Input.dispatchKeyEvent', { type: 'keyUp', key: k, code, windowsVirtualKeyCode: vk });
};
try {
  await d.load();
  await d.readStory();
  await d.speed(0);
  await d.panel('commands');
  const rect = await ev(`(() => { const r = document.querySelector('[data-testid="run-cloud_compute"]').getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()`);
  console.log('before', await energy());
  for (const type of ['mousePressed', 'mouseReleased']) await b.send('Input.dispatchMouseEvent', { type, x: rect.x, y: rect.y, button: 'left', clickCount: 1 });
  await sleep(200);
  console.log('after click', await energy(), 'focused:', await ev('document.activeElement?.dataset?.testid ?? document.activeElement?.tagName'));
  await key(' ', 'Space', 32);
  await sleep(300);
  console.log('after space', await energy(), 'paused:', await ev(`document.querySelector('#speeds button[data-speed="0"]')?.getAttribute('aria-pressed')`));
  for (let i = 0; i < 5; i++) await key('Enter', 'Enter', 13, i > 0);
  await sleep(300);
  console.log('after 5 Enter', await energy());
  console.log('errors', JSON.stringify(b.errors));
} catch (e) {
  console.log('PROBE FAILED', e.stack);
} finally {
  b.close();
}
