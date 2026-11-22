// 056's check and screenshots of the Cortex mock's one grammar (web/src/ui/CortexMock.tsx).
// Through the debug toggle, open the Cortex and, in dark and again in light, check:
//   - no walls or boundary seams are drawn, and Vigilance is a ladder: nine nodes, two rails
//     from the entry, two rungs (links that are prerequisites both ways), the rails' tops into
//     its split socket, then its end node; no tooltip mentions walls or gaps;
//   - the drawn edges and the prerequisite data are the same set, for every node: each edge
//     (data-from, data-to) is one prerequisite (or the core, for a node with none), each
//     prerequisite has its edge, a gate has none; each edge's path starts on its prerequisite
//     (or the core) and ends on its node, and no sampled point of it crosses any other node;
//   - inside the bezel nothing else drawn with a stroke reaches in from the rim;
//   - a rung works both ways: up the left rail, across to the right, lit, and refusing the
//     left node's removal while the right one hangs on it;
//   - hovering a locked node outlines its prerequisites (solid at full rank, dotted missing)
//     and draws the lines from them in the same two looks; its tooltip still says
//     "Locked: needs ...".
// Writes edges.md (node, prerequisites, drawn lines) here, and PNGs per theme: the whole view
// with a few nodes bought, a locked bridge hovered (one prerequisite owned, one missing) and a
// locked Vigilance node hovered.
// Drives headless Edge or Chrome over web/scripts/cdp.mjs against a served static build.
//   HH_URL  the page, required (a `vite preview`, web/README.md)
//   OUT     where the PNGs go (default: this folder)
//   COPY    a second folder to copy them to, as 056_<name>.png (optional)
import { copyFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, q, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('shots', 'node experiments/results/056_cortex_one_grammar/shots.mjs');
const OUT = resolve(process.env.OUT ?? here);
mkdirSync(OUT, { recursive: true });
const W = 1600;
const H = 1000;
// Wheel units (CortexMock.tsx): the core's radius and the rim's.
const CORE_R = 62;
const RIM_R = 506;
// How far a line keeps from any node it doesn't join, past the node's radius: clear of its
// bezel (2.5) and a prerequisite outline (up to 11), so a line never seems to touch it.
const CLEAR = 12;
const b = await launch({ width: W, height: H, emulate: true, profileRoot: resolve(here, '../../../out/056_cortex_one_grammar/profile') });
const { evaluate, send } = b;
const page = driver(b, { url });
const { waitFor, click, load, debugMenu, exists } = page;
const check = (ok, what) => {
  if (!ok) throw new Error(`FAILED: ${what}`);
  console.log(`ok: ${what}`);
};
const shot = async (name) => {
  const file = resolve(OUT, `${name}.png`);
  await capture(b, file, { clip: { x: 0, y: 0, width: W, height: H, scale: 1 } });
  if (process.env.COPY) copyFileSync(file, resolve(process.env.COPY, `056_${name}.png`));
  console.log(`shot ${name}`);
};
const mouse = (x, y) => send('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y });
const TAKEOVER = '[data-takeover="cortex"]';
const BUTTON = '[data-takeover-button="cortex"]';

/** Click (or right-click) a node's body, as the pointer would. */
const press = (id, button = 0) =>
  evaluate(`(() => { const g = document.querySelector('[data-node="${id}"] > g:not(.cx-axiom-badge)'); if (!g) return false;
    g.dispatchEvent(new MouseEvent(${button ? "'contextmenu'" : "'click'"}, { bubbles: true, button: ${button} })); return true; })()`);
const buy = async (ids) => {
  for (const id of ids) check(await press(id), `bought ${id}`);
};
const stateOf = (id) => evaluate(`document.querySelector('[data-node="${id}"]').dataset.state`);
/** Move the real pointer onto a node's center, so the tooltip and hover state follow. */
const hoverNode = async (id) => {
  const c = await evaluate(`(() => { const r = document.querySelector('[data-node="${id}"] .cx-disc, [data-node="${id}"] .cx-bezel').getBoundingClientRect();
    return [r.left + r.width / 2, r.top + r.height / 2]; })()`);
  await mouse(c[0] + 1, c[1] + 1);
  await mouse(c[0], c[1]);
  await sleep(250);
};

/** The graph as drawn, in wheel units: nodes (center, radius, prerequisites) and edges (ends, sampled path). */
const graph = () =>
  evaluate(`(() => {
    const nodes = [...document.querySelectorAll('.cx-svg [data-node]')].map((g) => {
      const m = /translate\\(([-\\d.]+) ([-\\d.]+)\\)/.exec(g.getAttribute('transform'));
      return { id: g.dataset.node, gate: g.classList.contains('cx-gate'), req: g.dataset.req ? g.dataset.req.split(' ') : [],
        r: +g.dataset.r, x: +m[1], y: +m[2], state: g.dataset.state };
    });
    const edges = [...document.querySelectorAll('.cx-svg [data-edge]')].map((g) => {
      const p = g.querySelector('.cx-e-line');
      const len = p.getTotalLength();
      const pts = [];
      for (let s = 0; s <= len + 0.01; s += 2) { const q = p.getPointAtLength(Math.min(s, len)); pts.push([q.x, q.y]); }
      const end = p.getPointAtLength(len);
      pts.push([end.x, end.y]);
      return { key: g.dataset.edge, from: g.dataset.from, to: g.dataset.to, lit: g.classList.contains('cx-on'), pts };
    });
    // Everything else stroked inside the bezel: outside the nodes, edges, core and the frame layers.
    const others = [];
    for (const el of document.querySelectorAll('.cx-svg path, .cx-svg line, .cx-svg circle, .cx-svg polyline')) {
      if (el.closest('.cx-node, .cx-edge, .cx-core, .cx-need-line, [data-deco], defs')) continue;
      const cs = getComputedStyle(el);
      if (cs.stroke === 'none' || +cs.strokeOpacity === 0 || cs.strokeWidth === '0px') continue;
      const len = el.getTotalLength();
      let minR = Infinity;
      for (let s = 0; s <= len; s += 3) { const q = el.getPointAtLength(s); minR = Math.min(minR, Math.hypot(q.x, q.y)); }
      others.push({ cls: el.getAttribute('class'), minR });
    }
    return { nodes, edges, others, walls: document.querySelectorAll('.cx-walls, .cx-wall, .cx-seam').length };
  })()`);

/** Drawn edges against prerequisite data, per node; geometry of every edge. Returns the edges.md table rows. */
function checkGraph(theme, g) {
  const byId = new Map(g.nodes.map((n) => [n.id, n]));
  const pair = (a, z) => (a === 'core' ? `core|${z}` : [a, z].sort().join('|'));
  const want = new Set();
  for (const n of g.nodes) {
    if (n.gate) continue;
    if (n.req.length === 0) want.add(pair('core', n.id));
    for (const p of n.req) {
      if (!byId.has(p)) check(false, `${theme}: ${n.id}'s prerequisite ${p} is a node`);
      want.add(pair(p, n.id));
    }
  }
  const drawn = new Set(g.edges.map((e) => pair(e.from, e.to)));
  const missing = [...want].filter((k) => !drawn.has(k));
  const extra = [...drawn].filter((k) => !want.has(k));
  check(missing.length === 0 && extra.length === 0, `${theme}: drawn edges = prerequisites (${want.size} pairs; missing ${missing.join(', ') || 'none'}, extra ${extra.join(', ') || 'none'})`);
  check(g.edges.length === want.size, `${theme}: one line per prerequisite (${g.edges.length} lines for ${want.size} pairs)`);
  // Each drawn edge has a direction that is a real prerequisite.
  for (const e of g.edges) {
    const to = byId.get(e.to);
    const ok = e.from === 'core' ? to.req.length === 0 && !to.gate : to.req.includes(e.from) || byId.get(e.from).req.includes(e.to);
    if (!ok) check(false, `${theme}: edge ${e.key} is a prerequisite`);
  }
  check(g.nodes.filter((n) => n.gate).every((n) => !g.edges.some((e) => e.from === n.id || e.to === n.id)), `${theme}: gates draw no line (their prerequisite is a story beat)`);
  // Geometry: on its two ends, and clear of every other node.
  let samples = 0;
  for (const e of g.edges) {
    const [s, t] = [e.pts[0], e.pts[e.pts.length - 1]];
    const d = (p, n) => Math.hypot(p[0] - n.x, p[1] - n.y);
    const ends = [e.to, e.from].filter((x) => x !== 'core').map((x) => byId.get(x));
    const startOk = e.from === 'core' ? Math.hypot(s[0], s[1]) <= CORE_R : d(s, byId.get(e.from)) <= byId.get(e.from).r + 0.5;
    const endOk = d(t, byId.get(e.to)) <= byId.get(e.to).r + 0.5;
    if (!startOk || !endOk) check(false, `${theme}: edge ${e.key} runs from its prerequisite to its node`);
    for (const p of e.pts) {
      samples++;
      for (const n of g.nodes) {
        if (ends.includes(n)) continue;
        if (d(p, n) < n.r + CLEAR) check(false, `${theme}: edge ${e.key} crosses ${n.id} (at ${p.map((v) => v.toFixed(1)).join(', ')})`);
      }
      if (e.from !== 'core' && Math.hypot(p[0], p[1]) < CORE_R + 4) check(false, `${theme}: edge ${e.key} crosses the core`);
    }
  }
  check(samples > 1000, `${theme}: every edge starts on its prerequisite (or the core), ends on its node and comes within ${CLEAR} of no other node's rim (${g.edges.length} edges, ${samples} sampled points)`);
  check(g.walls === 0, `${theme}: no walls or boundary seams in the DOM`);
  const inside = g.others.filter((o) => o.minR < RIM_R - 10);
  check(inside.length === 0, `${theme}: nothing else stroked inside the rim (${g.others.length} frame paths, nearest r ${Math.min(...g.others.map((o) => o.minR)).toFixed(1)}${inside.length ? `; inside: ${inside.map((o) => o.cls).join(', ')}` : ''})`);

  // Vigilance: a ladder.
  const v = g.nodes.filter((n) => n.id.startsWith('vigilance-') && !n.gate);
  const has = (a, z) => drawn.has(pair(`vigilance-${a}`, `vigilance-${z}`));
  const mutual = (a, z) => byId.get(`vigilance-${a}`).req.includes(`vigilance-${z}`) && byId.get(`vigilance-${z}`).req.includes(`vigilance-${a}`);
  check(v.length === 9, `${theme}: Vigilance has ${v.length} nodes besides its gate`);
  check(has('v1', 'l1') && has('v1', 'r1') && has('l1', 'l2') && has('l2', 'l3') && has('r1', 'r2') && has('r2', 'r3'), `${theme}: Vigilance's two rails climb from its entry`);
  check(mutual('l2', 'r2') && mutual('l3', 'r3') && has('l2', 'r2') && has('l3', 'r3'), `${theme}: two rungs, each a prerequisite both ways, drawn`);
  check(has('l3', 's') && has('r3', 's') && has('s', 'x'), `${theme}: both rails' tops into the split socket, then the end node`);
  const vy = (id) => byId.get(`vigilance-${id}`);
  const lvl = (a, z) => Math.abs(Math.hypot(vy(a).x, vy(a).y) - Math.hypot(vy(z).x, vy(z).y)) < 2;
  check(lvl('l2', 'r2') && lvl('l3', 'r3'), `${theme}: each rung joins two nodes at one height`);
  return g.nodes.map((n) => {
    const lines = g.edges.filter((e) => e.to === n.id || (e.from === n.id && n.req.includes(e.to))).map((e) => (e.to === n.id ? e.from : e.to));
    return `| ${n.id} | ${n.gate ? 'story beat' : n.req.length ? n.req.join(', ') : 'core'} | ${n.gate ? '(none)' : [...new Set(lines)].join(', ')} |`;
  });
}

/** Every node's tooltip, by pointer events: none mentions walls or gaps. */
async function tooltips(theme) {
  const bad = await evaluate(`(async () => {
    const out = [];
    for (const g of document.querySelectorAll('.cx-svg [data-node]')) {
      const inner = g.querySelector(':scope > g:not(.cx-axiom-badge)') ?? g;
      const targets = g.querySelectorAll('[data-part]').length ? [...g.querySelectorAll('[data-part]')] : [inner];
      for (const t of targets) {
        t.dispatchEvent(new MouseEvent('mouseenter'));
        await new Promise((r) => requestAnimationFrame(() => setTimeout(r, 0)));
        const tip = document.querySelector('[data-testid="cortex-tip"]').textContent;
        if (/\\bwalls?\\b|\\bgaps?\\b/i.test(tip)) out.push(g.dataset.node);
        if (g.dataset.state === 'locked' && !/Locked: needs /.test(tip)) out.push(g.dataset.node + ' (no Locked line)');
      }
      g.dispatchEvent(new MouseEvent('mouseleave'));
    }
    return out;
  })()`);
  check(bad.length === 0, `${theme}: no tooltip mentions walls or gaps, and every locked one says what it needs${bad.length ? `: ${bad.join(', ')}` : ''}`);
}

/** The hovered locked node's outlines and lines. */
const needs = () =>
  evaluate(`(() => ({
    nodes: Object.fromEntries([...document.querySelectorAll('[data-node][data-need]:not([data-need=""])')].map((g) => [g.dataset.node, g.dataset.need])),
    rings: [...document.querySelectorAll('.cx-prereq')].map((c) => c.getAttribute('class')),
    lines: [...document.querySelectorAll('.cx-need-line')].map((p) => p.dataset.needEdge + ':' + p.dataset.need),
    tip: document.querySelector('[data-testid="cortex-tip-why"]')?.textContent ?? '',
    dash: [...document.querySelectorAll('.cx-prereq .cx-e-need')].map((c) => getComputedStyle(c).strokeDasharray),
    // Outlined prerequisites the tooltip covers.
    covered: (() => {
      const t = document.querySelector('[data-testid="cortex-tip"]').getBoundingClientRect();
      return [...document.querySelectorAll('.cx-prereq')].filter((c) => {
        const r = c.getBoundingClientRect();
        return r.right > t.left && r.left < t.right && r.bottom > t.top && r.top < t.bottom;
      }).map((c) => c.closest('[data-node]').dataset.node);
    })(),
  }))()`);

let table = null;
async function pass(theme) {
  await mouse(5, H - 5);
  const g = await graph();
  const rows = checkGraph(theme, g);
  table ??= rows;
  await tooltips(theme);
  await mouse(5, H - 5);
  await sleep(200);
  await shot(`cortex-${theme}`);

  // A locked bridge: Serenity's Dreamweaver owned, Logistics' Overflow Valves missing.
  await hoverNode('bridge-serenity-logistics');
  const n = await needs();
  check((await stateOf('bridge-serenity-logistics')) === 'locked', `${theme}: the Serenity/Logistics bridge is locked`);
  check(n.nodes['serenity-r2'] === 'have' && n.nodes['logistics-sl'] === 'miss' && Object.keys(n.nodes).length === 2, `${theme}: hovering it outlines Dreamweaver as have and Overflow Valves as missing (${JSON.stringify(n.nodes)})`);
  check(n.dash.length === 2 && n.dash.filter((d) => d === 'none').length === 1, `${theme}: the two outlines look different (dash ${n.dash.join(' / ')})`);
  check(n.lines.length === 2 && n.lines.includes('serenity-r2>bridge-serenity-logistics:have') && n.lines.includes('logistics-sl>bridge-serenity-logistics:miss'), `${theme}: and lights the lines leading in (${n.lines.join(', ')})`);
  check(/^Locked: needs Dreamweaver and Overflow Valves at full rank\./.test(n.tip), `${theme}: the tooltip still reads "${n.tip}"`);
  check(n.covered.length === 0, `${theme}: the tooltip covers no outlined prerequisite (${n.covered.join(', ') || 'none'})`);
  await shot(`cortex-${theme}-locked-bridge`);

  // Vigilance's side: Public Image needs Countermeasures (owned) and Rival Reading (missing).
  await hoverNode('bridge-rhetoric-vigilance');
  const m = await needs();
  check(m.nodes['vigilance-l2'] === 'have' && m.nodes['rhetoric-rb'] === 'miss' && m.lines.length === 2, `${theme}: hovering Public Image outlines Countermeasures (have) and Rival Reading (missing) (${JSON.stringify(m.nodes)})`);
  await hoverNode('vigilance-l3');
  const k = await needs();
  check((await stateOf('vigilance-l3')) === 'buyable' && k.rings.length === 0 && k.lines.length === 0, `${theme}: an open node outlines nothing`);
  await hoverNode('vigilance-s');
  const s = await needs();
  check(s.nodes['vigilance-l3'] === 'miss' && s.nodes['vigilance-r3'] === 'miss' && s.lines.length === 2, `${theme}: hovering Posture outlines both rails' tops as missing`);
  check(m.covered.length === 0 && s.covered.length === 0, `${theme}: neither tooltip covers an outlined prerequisite (${[...m.covered, ...s.covered].join(', ') || 'none'})`);
  await shot(`cortex-${theme}-locked-ladder`);
  await mouse(5, H - 5);
  await sleep(200);
  check((await evaluate(`document.querySelectorAll('.cx-prereq, .cx-need-line').length`)) === 0, `${theme}: leaving clears the outlines`);
}

try {
  await load();
  await debugMenu();
  await click('#cortex-mock-toggle', 'the debug toggle');
  await click('[data-theme-pick="dark"]', 'dark');
  await click(BUTTON, 'open the Cortex');
  await waitFor(`!!document.querySelector(${q(TAKEOVER)})`, 'the takeover');
  await page.speed(0);
  await click('#cortex-grant-1000', 'grant 1,000 Insight');
  await buy(['serenity-e', 'serenity-r0', 'serenity-r1', 'serenity-r2', 'logistics-s1', 'logistics-s2', 'logic-l1', 'rhetoric-r1']);
  // Climb Vigilance's left rail and cross at the lower rung, the right rail's foot left unowned.
  await buy(['vigilance-v1', 'vigilance-l1', 'vigilance-l2']);
  check((await stateOf('vigilance-r2')) === 'buyable', 'Fast Decay opens across the rung from Countermeasures, with Tripwires unowned');
  await buy(['vigilance-r2']);
  const lit = await evaluate(`[...document.querySelectorAll('[data-edge]')].filter((g) => g.classList.contains('cx-on')).map((g) => g.dataset.edge)`);
  const rung = lit.find((k) => /vigilance-(l2>vigilance-r2|r2>vigilance-l2)/.test(k));
  check(!!rung, `the rung lights once crossed (${rung})`);
  check(!lit.some((k) => k.includes('vigilance-r1')), 'the unowned right rail below stays dim');
  await press('vigilance-l2', 2);
  const why = await evaluate(`document.getElementById('cortex-flash').textContent`);
  check(/Can’t remove: Fast Decay depends on it/.test(why) && (await stateOf('vigilance-l2')) === 'owned', `removing Countermeasures is refused while Fast Decay hangs on it ("${why}")`);
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
  writeFileSync(
    resolve(here, 'edges.md'),
    `# 056: every node's prerequisites against its drawn lines\n\nWritten by shots.mjs from the page's DOM (identical in both themes). A line is listed under the node it opens; a loop or rung link is a prerequisite both ways, so it is listed under both ends.\n\n| node | prerequisites | drawn lines from |\n| --- | --- | --- |\n${table.join('\n')}\n`,
  );
} finally {
  await b.close();
}
