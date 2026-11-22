// 054's check and screenshots of the Cortex mock's Grand Theorems column (web/src/ui/CortexMock.tsx).
// Through the debug toggle, open the Cortex and, in dark and again in light: the column (heading
// to last plaque) is centered vertically on the wheel's core and still right-aligned, and the
// "GRAND THEOREMS" heading is in the rim names' style (bold, uppercase, letter-spaced) at at least
// their on-screen size. At 1280 x 720 the column fits, centered or top-aligned, never clipped.
// PNGs of the whole window per theme at 1600 x 1000, and one per theme at 1280 x 720.
// Drives headless Edge or Chrome over web/scripts/cdp.mjs against a served static build.
//   HH_URL  the page, required (a `vite preview`, web/README.md)
//   OUT     where the PNGs go (default: this folder)
//   COPY    a second folder to copy them to, as 054_<name>.png (optional)
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, q, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('shots', 'node experiments/results/054_cortex_theorem_column/shots.mjs');
const OUT = resolve(process.env.OUT ?? here);
mkdirSync(OUT, { recursive: true });
const W = 1600;
const H = 1000;
const b = await launch({ width: W, height: H, emulate: true, profileRoot: resolve(here, '../../../out/054_cortex_theorem_column/profile') });
const { evaluate, send } = b;
const page = driver(b, { url });
const { waitFor, click, load, debugMenu, exists } = page;
const check = (ok, what) => {
  if (!ok) throw new Error(`FAILED: ${what}`);
  console.log(`ok: ${what}`);
};
const size = async (w, h) => {
  await send('Emulation.setDeviceMetricsOverride', { width: w, height: h, deviceScaleFactor: 1, mobile: false });
  await sleep(300);
};
const shot = async (name, width = W, height = H) => {
  const file = resolve(OUT, `${name}.png`);
  await capture(b, file, { clip: { x: 0, y: 0, width, height, scale: 1 } });
  if (process.env.COPY) copyFileSync(file, resolve(process.env.COPY, `054_${name}.png`));
  console.log(`shot ${name}`);
};
const TAKEOVER = '[data-takeover="cortex"]';
const BUTTON = '[data-takeover-button="cortex"]';
/** The column's geometry against the core's, and the heading's style against a rim name's. */
const measure = () =>
  evaluate(`(() => {
    const side = document.querySelector('.cx-side');
    const kids = [...side.children].map((e) => e.getBoundingClientRect());
    const top = kids[0].top, bottom = kids[kids.length - 1].bottom;
    const box = side.getBoundingClientRect();
    const main = document.querySelector('.cx-main').getBoundingClientRect();
    const core = document.querySelector('[data-testid="cortex-core"] .cx-core-orb').getBoundingClientRect();
    const name = document.querySelector('[data-sector-name="serenity"]');
    const ns = getComputedStyle(name);
    const k = name.ownerSVGElement.getScreenCTM().a;
    const head = document.querySelector('.cx-side-head');
    const hs = getComputedStyle(head);
    return {
      colMid: (top + bottom) / 2, coreMid: core.top + core.height / 2,
      top, bottom, boxTop: box.top, boxBottom: box.bottom, right: box.right, mainRight: main.right,
      clipped: side.scrollHeight > side.clientHeight + 1,
      nameSize: parseFloat(ns.fontSize) * k, nameTrack: parseFloat(ns.letterSpacing) * k, nameWeight: ns.fontWeight,
      headSize: parseFloat(hs.fontSize), headTrack: parseFloat(hs.letterSpacing), headWeight: hs.fontWeight,
      headCase: hs.textTransform, headText: head.textContent,
    };
  })()`);

async function pass(theme) {
  await size(W, H);
  const m = await measure();
  const r = (x) => Math.round(x * 10) / 10;
  check(Math.abs(m.colMid - m.coreMid) <= 2, `${theme}: column centered on the core (column mid ${r(m.colMid)}, core ${r(m.coreMid)})`);
  check(m.top > m.boxTop + 20 && Math.abs(m.right - m.mainRight) <= 1, `${theme}: off the top, still in the right-hand column`);
  check(m.headSize >= m.nameSize - 0.05, `${theme}: heading ${r(m.headSize)}px, rim names ${r(m.nameSize)}px on screen`);
  check(m.headTrack >= m.nameTrack - 0.05 && m.headWeight === m.nameWeight && m.headCase === 'uppercase',
    `${theme}: letter-spaced like the rim (${r(m.headTrack)}px vs ${r(m.nameTrack)}px, weight ${m.headWeight})`);
  await shot(`cortex-${theme}`);

  await size(1280, 720);
  const s = await measure();
  const centered = Math.abs(s.colMid - s.coreMid) <= 2;
  check(!s.clipped && s.bottom <= s.boxBottom + 1 && s.top >= s.boxTop, `${theme} 1280 x 720: not clipped (${centered ? 'centered' : 'top-aligned'}, ${r(s.top)}-${r(s.bottom)} in ${r(s.boxTop)}-${r(s.boxBottom)})`);
  check(s.headSize >= s.nameSize - 0.05, `${theme} 1280 x 720: heading ${r(s.headSize)}px, rim names ${r(s.nameSize)}px`);
  await shot(`cortex-${theme}-1280`, 1280, 720);
  await size(W, H);
}

try {
  await load();
  await debugMenu();
  await click('#cortex-mock-toggle', 'the debug toggle');
  await click('[data-theme-pick="dark"]', 'dark');
  await click(BUTTON, 'open the Cortex');
  await waitFor(`!!document.querySelector(${q(TAKEOVER)})`, 'the takeover');
  await page.speed(0);
  await pass('dark');
  await click(BUTTON, 'close');
  await debugMenu();
  await click('[data-theme-pick="light"]', 'light');
  await click(BUTTON, 'reopen');
  await waitFor(`!!document.querySelector(${q(TAKEOVER)})`, 'the takeover');
  await pass('light');
  await click(BUTTON, 'close');
  check(!(await exists(TAKEOVER)), 'closed');
  check(b.errors.length === 0, `no page errors${b.errors.length ? `: ${b.errors.join('; ')}` : ''}`);
} finally {
  await b.close();
}
