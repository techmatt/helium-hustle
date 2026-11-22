// 055's check and screenshots of the Cortex mock's frames (web/src/ui/CortexMock.tsx).
// Through the debug toggle, open the Cortex and, in dark and again in light, check:
//   - the Grand Theorems case is on the left of the wheel, clear of the game's nav, centered
//     on the core, its title plate on its top edge, not clipped at 1280 x 720;
//   - hovering a plaque still lights its sectors (rim and corona);
//   - the bezel, corona and neural field are drawn, and no decoration reaches inside the
//     bezel: every point of the corona and the field (sampled along each path) lies outside it.
// PNGs of the whole window per theme at 1600 x 1000 (a few nodes bought, nothing hovered), one
// per theme at 1280 x 720, and one per theme with a Theorem hovered.
// Drives headless Edge or Chrome over web/scripts/cdp.mjs against a served static build.
//   HH_URL  the page, required (a `vite preview`, web/README.md)
//   OUT     where the PNGs go (default: this folder)
//   COPY    a second folder to copy them to, as 055_<name>.png (optional)
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, q, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('shots', 'node experiments/results/055_cortex_frames/shots.mjs');
const OUT = resolve(process.env.OUT ?? here);
mkdirSync(OUT, { recursive: true });
const W = 1600;
const H = 1000;
// The bezel's radii in wheel units (CortexMock.tsx: BEZEL_IN, BEZEL_OUT).
const BEZEL_IN = 512;
const BEZEL_OUT = 556;
// The neural field's inner limit (DECO_MIN_R).
const DECO_MIN_R = 640;
const b = await launch({ width: W, height: H, emulate: true, profileRoot: resolve(here, '../../../out/055_cortex_frames/profile') });
const { evaluate, send } = b;
const page = driver(b, { url });
const { waitFor, click, load, debugMenu, exists } = page;
const check = (ok, what) => {
  if (!ok) throw new Error(`FAILED: ${what}`);
  console.log(`ok: ${what}`);
};
const r1 = (x) => Math.round(x * 10) / 10;
const size = async (w, h) => {
  await send('Emulation.setDeviceMetricsOverride', { width: w, height: h, deviceScaleFactor: 1, mobile: false });
  await sleep(300);
};
const shot = async (name, width = W, height = H) => {
  const file = resolve(OUT, `${name}.png`);
  await capture(b, file, { clip: { x: 0, y: 0, width, height, scale: 1 } });
  if (process.env.COPY) copyFileSync(file, resolve(process.env.COPY, `055_${name}.png`));
  console.log(`shot ${name}`);
};
const mouse = (x, y) => send('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y });
const TAKEOVER = '[data-takeover="cortex"]';
const BUTTON = '[data-takeover-button="cortex"]';

/** The case's geometry against the core's, the nav's and the tree's. */
const layout = () =>
  evaluate(`(() => {
    const R = (s) => document.querySelector(s).getBoundingClientRect();
    const side = R('.cx-side'), kase = R('.cx-case'), plate = R('.cx-case-plate'), tree = R('.cx-tree'), main = R('.cx-main');
    const core = R('[data-testid="cortex-core"] .cx-core-orb');
    const well = document.querySelector('.cx-case-well');
    const title = document.querySelector('.cx-case-title');
    return {
      caseMid: (kase.top + kase.bottom) / 2, coreMid: core.top + core.height / 2,
      caseLeft: kase.left, caseRight: kase.right, caseTop: kase.top, caseBottom: kase.bottom,
      mainLeft: main.left, treeLeft: tree.left, coreX: core.left + core.width / 2,
      plateTop: plate.top, plateBottom: plate.bottom, sideTop: side.top, sideBottom: side.bottom,
      wellClipped: well.scrollHeight > well.clientHeight + 1,
      title: title.textContent, titleCase: getComputedStyle(title).textTransform,
      oldHeading: !!document.querySelector('.cx-side-head'),
    };
  })()`);

/** The smallest radius, in wheel units, reached by each decoration layer (sampled along every path). */
const decoRadii = () =>
  evaluate(`(() => {
    // Each subpath on its own (a long path's getPointAtLength is slow), in the layer's own
    // user space: the wheel's units around the core.
    const minR = (root) => {
      let m = Infinity, n = 0;
      const svg = root.ownerSVGElement ?? root;
      const probe = document.createElementNS('http://www.w3.org/2000/svg', 'path');
      svg.appendChild(probe);
      for (const p of root.querySelectorAll('path')) {
        for (const sub of p.getAttribute('d').split(/(?=M )/)) {
          if (!sub.trim()) continue;
          probe.setAttribute('d', sub);
          const len = probe.getTotalLength();
          for (let s = 0; s <= len + 0.01; s += Math.min(1.5, len || 1)) {
            const { x, y } = probe.getPointAtLength(Math.min(s, len));
            m = Math.min(m, Math.hypot(x, y));
            n++;
          }
        }
      }
      probe.remove();
      return { m, n };
    };
    const bz = document.querySelector('[data-deco="bezel"]');
    const radii = [...bz.querySelectorAll('circle')].map((c) => c.r.baseVal.value);
    return { corona: minR(document.querySelector('[data-deco="corona"]')), field: minR(document.querySelector('[data-deco="field"]')),
      bezelIn: Math.min(...radii), ticks: minR(bz), names: document.querySelectorAll('.cx-sector-name').length };
  })()`);

async function buySome() {
  await click('#cortex-grant-1000', 'grant 1,000 Insight');
  for (const id of ['serenity-e', 'serenity-r0', 'logistics-s1', 'logistics-s2', 'logic-l1', 'rhetoric-r1', 'vigilance-v1', 'vigilance-i2']) {
    const ok = await evaluate(`(() => { const g = document.querySelector('[data-node="${id}"] > g'); if (!g) return false;
      g.dispatchEvent(new MouseEvent('click', { bubbles: true })); return true; })()`);
    check(ok, `bought ${id}`);
  }
}

async function pass(theme) {
  await size(W, H);
  await mouse(5, H - 5);
  const m = await layout();
  check(!m.oldHeading && m.title === 'Grand Theorems' && m.titleCase === 'uppercase', `${theme}: a title plate replaces the loose heading`);
  check(m.caseRight < m.treeLeft && m.caseRight < m.coreX, `${theme}: the case is left of the wheel (case right ${r1(m.caseRight)}, tree left ${r1(m.treeLeft)})`);
  check(m.caseLeft - m.mainLeft >= 24, `${theme}: ${r1(m.caseLeft - m.mainLeft)}px clear of the nav`);
  check(Math.abs(m.caseMid - m.coreMid) <= 2, `${theme}: case centered on the core (case mid ${r1(m.caseMid)}, core ${r1(m.coreMid)})`);
  check(m.plateTop < m.caseTop && m.plateBottom > m.caseTop && m.plateTop >= m.sideTop, `${theme}: the title plate sits on the case's top edge`);
  check(!m.wellClipped, `${theme}: plaques not clipped`);

  const d = await decoRadii();
  check(d.corona.n > 500 && d.field.n > 500, `${theme}: corona and field drawn (${d.corona.n} and ${d.field.n} sampled points)`);
  check(d.bezelIn >= BEZEL_IN && d.ticks.m >= BEZEL_IN, `${theme}: the bezel lies outside the rim (from r ${r1(Math.min(d.bezelIn, d.ticks.m))})`);
  check(d.corona.m >= BEZEL_OUT, `${theme}: corona outside the bezel (min r ${r1(d.corona.m)} >= ${BEZEL_OUT})`);
  check(d.field.m >= DECO_MIN_R, `${theme}: neural field outside the bezel and corona (min r ${r1(d.field.m)} >= ${DECO_MIN_R})`);
  check(d.names === 5, `${theme}: the five sector names still on the rim`);
  await shot(`cortex-${theme}`);

  // Hover a Theorem: its two sectors light (rim and corona), the others don't.
  const pr = await evaluate(`(() => { const r = document.querySelector('[data-theorem="logic-rhetoric"]').getBoundingClientRect(); return [r.left + r.width / 2, r.top + r.height / 2]; })()`);
  await mouse(pr[0], pr[1]);
  await sleep(350);
  const lit = await evaluate(`[...document.querySelectorAll('.cx-sector[data-lit="1"]')].map((g) => g.dataset.sector).join(',')`);
  const corona = await evaluate(`document.querySelectorAll('.cx-corona-sector.cx-lit').length`);
  check(lit === 'logic,rhetoric' && corona === 2, `${theme}: hovering Persuasion lights ${lit} (corona ${corona})`);
  await shot(`cortex-${theme}-hover`);
  await mouse(5, H - 5);

  await size(1280, 720);
  const s = await layout();
  const centered = Math.abs(s.caseMid - s.coreMid) <= 2;
  check(!s.wellClipped && s.caseBottom <= s.sideBottom + 1 && s.plateTop >= s.sideTop, `${theme} 1280 x 720: not clipped (${centered ? 'centered' : 'top-aligned'})`);
  check(s.caseRight < s.treeLeft && s.caseLeft - s.mainLeft >= 24, `${theme} 1280 x 720: still left and clear of the nav`);
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
  await buySome();
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
