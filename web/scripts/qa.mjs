// QA checker: drives the game in headless Edge or Chrome over the DevTools protocol (no
// packages) across several seeds and window widths, the way a player would (a fresh game
// at speed, buying, programs, launches, every panel, retirement, the run summary, run 2,
// a reload), captures a screenshot only at meaningful moments (a panel, a story text,
// the run summary, a new quest), and checks each of them:
//   overflow   text clipped by its box with no tooltip holding it, text past its button or
//              cell, off the right edge or cut off by a clipping column, table cells
//              touching, a word broken across lines, a swatch alone on its line (its label
//              below), a card body shorter than its picture, the page wider than the window
//   hidden     a visible button or input whose center is covered by something else, card
//              text under its picture, the story banner or notice covered
//   text       placeholder or broken text on screen (undefined, NaN, [object ...], {x}),
//              more than one program line framed, a frame on a program with no processor
//   contrast   text against its solid background under 4.5:1 (3:1 large)
//   assets     an image that failed to load; a building card picture missing, not 96 px or
//              never shown; text in a bundled font at a weight or style it doesn't ship
//   console    any page error or console.error, a failed replay check, a play log that
//              `hh-sim playlog` doesn't reproduce
//   input      a click that took longer than INPUT_MS to show on screen, a probe's wrong
//              count (click-to-buy, the program frame), a double-click on Continue reaching
//              a card
//   frame      a frame gap over FRAME_MS while the page is visible
//   memory     JS heap growth over a pass past HEAP_GROWTH_MB, a save over 2 MB
// It writes `<out>/report.json`, one PNG per moment in `<out>/<seed>-<width>-<theme>/`, and
// `<out>/contact-<seed>-<width>-<theme>.png` (a contact sheet of that pass). Exits 1 on any
// finding.
//   HH_URL     the page, required: a `vite preview` of a static build (README)
//   HH_SEEDS   comma-separated seeds (default 11,12,13)
//   HH_WIDTHS  comma-separated window widths (default 1280,1920)
//   HH_HEIGHT  the window height (default 900)
//   HH_THEMES  themes, alternated over seeds and widths (default light,dark)
//   HH_QA_OUT  output folder (default ../out/qa, wiped first)
//   HH_RUN2    days of run 2 to play (default 150)
//   HH_SPEED   run 1's speed (default 50; 3 for a real-speed pass)
//   HH_SIM     the hh-sim that replays the play log (default target/release/hh-sim)
//   HH_BROWSER a browser off the usual paths
//   HH_PACE    <seconds>: instead, a first visit left at 1x that long, a shot every 15 s
//              (`<out>/pace/`)
//   HH_LONG    <minutes>: instead, one career that long at 200x, logging heap, DOM size,
//              save size and the worst frame each minute (`<out>/long/`)
// The DevTools harness (launch, waitFor, load, panel...) is scripts/cdp.mjs.
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { READY, driver, launch as launchBrowser, pageUrl, q, sleep } from './cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('qa');
const seeds = (process.env.HH_SEEDS ?? '11,12,13').split(',').map((s) => s.trim()).filter(Boolean);
const widths = (process.env.HH_WIDTHS ?? '1280,1920').split(',').map(Number).filter(Boolean);
const outDir = resolve(process.env.HH_QA_OUT ?? resolve(here, '../../out/qa'));
const RUN2_DAYS = Number(process.env.HH_RUN2 ?? 150);
const themes = (process.env.HH_THEMES ?? 'light,dark').split(',').map((s) => s.trim()).filter(Boolean);
const artIds = readdirSync(resolve(here, '../src/assets/buildings')).filter((f) => f.endsWith('.webp')).map((f) => f.slice(0, -5));
/** Run 1's speed: 50 by default; 3 plays it as a person would (about 3 minutes to a shipment). */
const RUN1_SPEED = Number(process.env.HH_SPEED ?? 50);
const INPUT_MS = 300;
const FRAME_MS = 250;
const HEAP_GROWTH_MB = 40;
const HEIGHT = Number(process.env.HH_HEIGHT ?? 900);

rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

/** A headless browser with one page at `width` x HEIGHT, its failed loads collected too. */
const launch = (width) =>
  launchBrowser({ width, height: HEIGHT, emulate: true, log: true, portBase: 9800, profileRoot: resolve(here, '../../out/qa-profile'), args: ['--enable-precise-memory-info'] });

// ---- the checks, run in the page at each moment ------------------------------------

const AUDIT = `(() => {
  const out = [];
  const W = innerWidth, H = innerHeight;
  const label = (el) => {
    const id = el.id ? '#' + el.id : '';
    const t = el.dataset?.testid ? '[' + el.dataset.testid + ']' : '';
    const cls = typeof el.className === 'string' && el.className ? '.' + el.className.split(' ').filter(Boolean).join('.') : '';
    return el.tagName.toLowerCase() + id + t + cls + ' "' + (el.textContent || el.value || '').trim().slice(0, 40) + '"';
  };
  const shown = (el) => {
    const r = el.getBoundingClientRect();
    if (r.width === 0 || r.height === 0) return false;
    const cs = getComputedStyle(el);
    return cs.visibility !== 'hidden' && cs.display !== 'none';
  };
  if (document.documentElement.scrollWidth > W + 1) out.push({ kind: 'overflow', what: 'page wider than the window: ' + document.documentElement.scrollWidth + ' > ' + W });
  for (const el of document.querySelectorAll('body *')) {
    if (!shown(el) || el.closest('svg')) continue;
    const cs = getComputedStyle(el);
    const hasText = [...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim());
    // Text clipped by its own box (overflow hidden, or an ellipsis).
    // An ellipsis is fine where a tooltip (on it or a parent) carries the whole text.
    const tooltip = el.closest('[title]')?.getAttribute('title') ?? '';
    const whole = (el.textContent || '').trim();
    if (hasText && (cs.overflowX !== 'visible' || cs.textOverflow === 'ellipsis') && el.scrollWidth > el.clientWidth + 1 && cs.overflowX !== 'auto' && cs.overflowX !== 'scroll' && !(whole && tooltip.includes(whole)))
      out.push({ kind: 'overflow', what: 'text clipped: ' + label(el) + ' (' + el.scrollWidth + ' > ' + el.clientWidth + ')' });
    // Text spilling past a fixed-size box (buttons, bars, cells, tiles).
    if (hasText && /^(BUTTON|TD|TH|SPAN|DIV|LABEL|STRONG|P|LI|H\\d)$/.test(el.tagName)) {
      const parent = el.parentElement;
      const r = el.getBoundingClientRect();
      if (el.tagName === 'BUTTON' && el.scrollWidth > el.clientWidth + 2) out.push({ kind: 'overflow', what: 'button text wider than the button: ' + label(el) });
      if (r.right > W + 1 && !el.closest('.modal')) out.push({ kind: 'overflow', what: 'off the right edge: ' + label(el) + ' right ' + Math.round(r.right) });
      if (parent && /^(BUTTON|TD)$/.test(parent.tagName)) {
        const p = parent.getBoundingClientRect();
        if (r.right > p.right + 2 || r.left < p.left - 2) out.push({ kind: 'overflow', what: 'text past its ' + parent.tagName.toLowerCase() + ': ' + label(el) });
      }
    }
  }
  // Text cut off by a clipping ancestor (a column with overflow-x hidden), not its own box.
  for (const el of document.querySelectorAll('body *')) {
    if (!shown(el) || el.closest('svg')) continue;
    if (![...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim())) continue;
    const r = el.getBoundingClientRect();
    for (let p = el.parentElement; p && p !== document.body; p = p.parentElement) {
      const o = getComputedStyle(p).overflowX;
      if (o !== 'hidden' && o !== 'clip') continue;
      const pr = p.getBoundingClientRect();
      const inner = pr.left + p.clientLeft + p.clientWidth;
      if (r.right > inner + 1 || r.left < pr.left - 1) out.push({ kind: 'overflow', what: 'text cut off by ' + label(p).slice(0, 30) + ': ' + label(el) + ' right ' + Math.round(r.right) + ' > ' + Math.round(inner) });
      break;
    }
  }
  // Table cells whose texts touch ("43.6 / 50−0.2/day"): under 3 px between them.
  const extent = (el) => {
    const range = document.createRange();
    range.selectNodeContents(el);
    const rs = [...range.getClientRects()].filter((x) => x.width > 0);
    return rs.length ? { left: Math.min(...rs.map((x) => x.left)), right: Math.max(...rs.map((x) => x.right)), top: Math.min(...rs.map((x) => x.top)), bottom: Math.max(...rs.map((x) => x.bottom)) } : null;
  };
  for (const tr of document.querySelectorAll('tr')) {
    if (!shown(tr)) continue;
    const cells = [...tr.children].filter((c) => c.textContent.trim()).map((c) => [c, extent(c)]).filter(([, e]) => e);
    for (let i = 1; i < cells.length; i++) {
      const [a, ea] = cells[i - 1], [b, eb] = cells[i];
      if (eb.left - ea.right < 3 && eb.top < ea.bottom && eb.bottom > ea.top) out.push({ kind: 'overflow', what: 'table cells touch: ' + label(a) + ' | ' + label(b) });
    }
  }
  // A color swatch left alone on a line, its label wrapped under it.
  for (const sq of document.querySelectorAll('.sq')) {
    // A nav tile stacks its icon over its name by design.
    if (!shown(sq) || getComputedStyle(sq.parentElement).flexDirection === 'column') continue;
    let n = sq.nextSibling;
    while (n && n.nodeType === 3 && !n.textContent.trim()) n = n.nextSibling;
    if (!n) n = sq.parentElement?.nextSibling;
    while (n && n.nodeType === 3 && !n.textContent.trim()) n = n.nextSibling;
    if (!n) continue;
    const range = document.createRange();
    range.selectNodeContents(n.nodeType === 3 ? n.parentNode : n);
    if (n.nodeType === 3) { range.setStart(n, n.textContent.search(/\\S/)); range.setEnd(n, n.textContent.search(/\\S/) + 1); }
    const first = [...range.getClientRects()].find((x) => x.width > 0);
    const s = sq.getBoundingClientRect();
    if (first && first.top > s.bottom + 1) out.push({ kind: 'overflow', what: 'swatch alone on its line, its label below: ' + label(sq.parentElement) });
  }
  // A word broken across lines in a control or heading ("Command|s").
  for (const el of document.querySelectorAll('button, button *, h1, h2, h3, th, td, label')) {
    if (!shown(el)) continue;
    for (const n of el.childNodes) {
      if (n.nodeType !== 3) continue;
      const re = /\\S{4,}/g;
      for (let m; (m = re.exec(n.textContent)); ) {
        const range = document.createRange();
        range.setStart(n, m.index);
        range.setEnd(n, m.index + m[0].length);
        const tops = new Set([...range.getClientRects()].filter((x) => x.width > 0).map((x) => Math.round(x.top)));
        if (tops.size > 1) out.push({ kind: 'overflow', what: 'word broken across lines: "' + m[0] + '" in ' + label(el) });
      }
    }
  }
  // Hidden targets: a control whose center (inside the window) is covered.
  for (const el of document.querySelectorAll('button, input, select, .nav-tile')) {
    if (!shown(el)) continue;
    const r = el.getBoundingClientRect();
    const x = r.left + r.width / 2, y = r.top + r.height / 2;
    if (x < 0 || y < 0 || x >= W || y >= H) continue;
    // Scrolled out of its scrolling column: not a target until scrolled to.
    let clipped = false;
    for (let p = el.parentElement; p && !clipped; p = p.parentElement) {
      const o = getComputedStyle(p).overflowY;
      if (o !== 'auto' && o !== 'scroll' && o !== 'hidden') continue;
      const pr = p.getBoundingClientRect();
      clipped = y < pr.top || y > pr.bottom || x < pr.left || x > pr.right;
    }
    if (clipped) continue;
    const top = document.elementFromPoint(x, y);
    if (top && top !== el && !el.contains(top) && !top.contains(el) && !top.closest('.modal-back') && !document.querySelector('.modal-back'))
      out.push({ kind: 'hidden', what: label(el) + ' covered by ' + label(top) });
  }
  // Text that is a placeholder or a broken value.
  const text = document.body.innerText;
  for (const m of text.matchAll(/\\b(undefined|NaN|null|Infinity|TODO|TBD|FIXME|lorem)\\b|\\[object \\w+\\]|\\{[a-z_]+\\}/g)) {
    const at = Math.max(0, m.index - 40);
    out.push({ kind: 'text', what: 'placeholder text "' + m[0] + '" in: ' + text.slice(at, m.index + 40).replace(/\\s+/g, ' ') });
  }
  for (const img of document.images) if (img.complete && img.naturalWidth === 0) out.push({ kind: 'assets', what: 'image failed: ' + img.src });
  // Contrast: each element's own text against the first solid background under it
  // (WCAG: 4.5:1, 3:1 for large text; disabled controls exempt; a gradient or picture
  // behind the text is not judged).
  const rgba = (s) => { const m = s.match(/rgba?\\(([^)]+)\\)/); if (!m) return null; const p = m[1].split(/[ ,/]+/).filter(Boolean).map(Number); return [p[0], p[1], p[2], p.length > 3 ? p[3] : 1]; };
  const lum = ([r, g, b]) => { const f = (c) => { c /= 255; return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4); }; return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b); };
  const ratio = (a, b) => { const x = lum(a), y = lum(b); return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05); };
  for (const el of document.querySelectorAll('body *')) {
    if (!shown(el) || el.closest('svg')) continue;
    if (![...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim())) continue;
    if (el.closest(':disabled')) continue;
    const cs = getComputedStyle(el);
    const fg = rgba(cs.color);
    if (!fg || fg[3] < 1 || Number(cs.opacity) < 1) continue;
    let bg = null;
    for (let p = el; p; p = p.parentElement) {
      const ps = getComputedStyle(p);
      if (ps.backgroundImage !== 'none') break;
      const c = rgba(ps.backgroundColor);
      if (c && c[3] >= 1) { bg = c; break; }
      if (c && c[3] > 0) break; // translucent: not judged
    }
    if (!bg) continue;
    const size = parseFloat(cs.fontSize), bold = Number(cs.fontWeight) >= 700;
    const need = size >= 24 || (bold && size >= 18.66) ? 3 : 4.5;
    const r = ratio(fg, bg);
    if (r < need - 0.05) out.push({ kind: 'contrast', what: document.documentElement.dataset.theme + ': ' + label(el) + ' ' + r.toFixed(2) + ':1 (' + cs.color + ' on ' + 'rgb(' + bg.slice(0, 3).join(',') + '))' });
  }
  // Fonts: text in a bundled family at a weight or style it doesn't ship (the browser
  // fakes it: a smeared bold, a slanted roman).
  const faces = new Set([...document.fonts].map((f) => f.family.replace(/["']/g, '') + '|' + f.weight + '|' + f.style));
  const families = new Set([...document.fonts].map((f) => f.family.replace(/["']/g, '')));
  const fakes = new Map();
  for (const el of document.querySelectorAll('body *')) {
    if (!shown(el) || el.closest('svg')) continue;
    if (![...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim())) continue;
    const cs = getComputedStyle(el);
    const fam = cs.fontFamily.split(',')[0].trim().replace(/["']/g, '');
    if (!families.has(fam)) continue;
    const key = fam + '|' + cs.fontWeight + '|' + cs.fontStyle;
    if (!faces.has(key) && !fakes.has(key)) fakes.set(key, label(el).replace(/ ".*$/, ''));
  }
  for (const [k, where] of fakes) out.push({ kind: 'assets', what: 'font not bundled, faked by the browser: ' + k.replace(/\\|/g, ' ') + ' at ' + where });
  // Building cards: every card's picture loaded and drawn at 96 px, and the card's body
  // at least as tall as the picture (a short card grows to it, the text never under it).
  for (const c of document.querySelectorAll('.card.building')) {
    if (!shown(c)) continue;
    const img = c.querySelector('img.card-art');
    if (!img) { out.push({ kind: 'assets', what: 'building card with no picture: ' + c.dataset.id }); continue; }
    const r = img.getBoundingClientRect();
    if (img.complete && (Math.round(r.width) !== 96 || Math.round(r.height) !== 96)) out.push({ kind: 'assets', what: 'card picture not 96 px: ' + c.dataset.id + ' ' + r.width + 'x' + r.height });
    const body = c.querySelector('.card-body')?.getBoundingClientRect();
    if (body && body.height < 95.5) out.push({ kind: 'overflow', what: 'card body shorter than its picture: ' + c.dataset.id + ' ' + body.height });
    for (const t of c.querySelectorAll('.card-text, .card-actions, hr, .costs')) {
      const tr = t.getBoundingClientRect();
      if (tr.left < r.right - 1 && tr.right > r.left + 1 && tr.top < r.bottom - 1 && tr.bottom > r.top + 1 && !t.contains(img)) out.push({ kind: 'hidden', what: 'card text under its picture: ' + c.dataset.id + ' ' + t.className });
    }
  }
  // A marker dot drawn over its tile's name.
  for (const d of document.querySelectorAll('.nav-tile .dot')) {
    const name = d.parentElement.querySelector(':scope > span:not(.dot)');
    if (!shown(d) || !name) continue;
    const a = d.getBoundingClientRect();
    const range = document.createRange();
    range.selectNodeContents(name);
    for (const r of range.getClientRects()) if (r.width && a.left < r.right - 0.5 && a.right > r.left + 0.5 && a.top < r.bottom - 0.5 && a.bottom > r.top + 0.5) { out.push({ kind: 'hidden', what: 'a dot over the tile name: ' + label(d.parentElement) }); break; }
  }
  // A signed number colored against its sign: a "−3" in green, a "+3" in red.
  for (const el of document.querySelectorAll('.pos, .neg')) {
    // Boredom rising is bad news: its rate reads red by design.
    if (!shown(el) || el.children.length || el.closest('[data-res="boredom"], [data-breakdown="boredom"]')) continue;
    const t = el.textContent.trim();
    if ((el.classList.contains('pos') && /^[−-]\\d/.test(t)) || (el.classList.contains('neg') && /^\\+\\d/.test(t)))
      out.push({ kind: 'text', what: 'number colored against its sign: ' + label(el) + ' (' + el.className + ')' });
  }
  // Short, yet reading enough: "needs 150, had 150"; a bar reading "2,000 / 2,000" on an
  // unfinished project or a pad not full.
  for (const n of document.querySelectorAll('.short-note')) {
    const m = n.textContent.match(/needs\\s*([\\d.,KMBT]+), had ([\\d.,KMBT]+)/);
    if (shown(n) && m && m[1] === m[2]) out.push({ kind: 'text', what: 'short but reads equal: ' + label(n) });
  }
  for (const bar of document.querySelectorAll('.card.project .bar, .card.pad .bar')) {
    const m = bar.querySelector('.bar-label')?.textContent.match(/^([\\d.,KMBT]+) \\/ ([\\d.,KMBT]+)$/);
    const fill = parseFloat(bar.querySelector('.fill')?.style.width ?? '0');
    if (shown(bar) && m && m[1] === m[2] && fill < 100) out.push({ kind: 'text', what: 'a bar reads full short of it: ' + label(bar.closest('.card')) });
  }
  // A breakdown row with nothing in any of its number cells.
  for (const tr of document.querySelectorAll('.breakdown-table tbody tr')) {
    if (shown(tr) && [...tr.querySelectorAll('td.num')].every((td) => !td.textContent.trim())) out.push({ kind: 'text', what: 'breakdown row with no numbers: ' + label(tr) });
  }
  // One quantity, two roundings: a card's "running at N%" against its tooltip's.
  for (const s of document.querySelectorAll('[data-card-status][title]')) {
    const a = s.textContent.match(/(<?\\d+)%/), b = s.getAttribute('title').match(/(<?\\d+)%/);
    if (a && b && a[1] !== b[1]) out.push({ kind: 'text', what: 'card status says ' + a[0] + ', its tooltip ' + b[0] + ': ' + label(s.closest('.card')) });
  }
  // Pads: the head's state against the engine's flags it is read from.
  for (const p of document.querySelectorAll('.card.pad')) {
    const st = p.dataset.padState, launch = p.querySelector('[data-testid^="launch-"]');
    if (st === 'ready' && launch?.disabled) out.push({ kind: 'text', what: 'pad reads Ready with Launch disabled: ' + label(p) });
    const bar = p.querySelector('.bar-label')?.textContent.match(/([\\d.,]+)\\s*\\/\\s*([\\d.,]+)/);
    if (st === 'loading' && bar && bar[1] === bar[2]) out.push({ kind: 'text', what: 'a full pad reads loading: ' + label(p) });
  }
  // Unread counts: a section's badge against the items it marks unread.
  for (const sec of document.querySelectorAll('section[data-unread]')) {
    const n = Number(sec.dataset.unread), badge = sec.querySelector('.fold-head .unread');
    if (n > 0 && !badge) out.push({ kind: 'text', what: 'unread count ' + n + ' with no badge: ' + sec.dataset.testid });
    if (badge && Number(badge.textContent) !== n) out.push({ kind: 'text', what: 'unread badge ' + badge.textContent + ' but ' + n + ' unread: ' + sec.dataset.testid });
    const marked = sec.querySelectorAll('.is-unread').length;
    if (sec.querySelector('.fold-body') && marked > n) out.push({ kind: 'text', what: marked + ' items marked unread but the count is ' + n + ': ' + sec.dataset.testid });
  }
  // The main quest has its own slot, and it shows whenever there is a quest.
  if (document.querySelector('[data-testid="quest"]') && !document.querySelector('[data-testid="quest-pin"] [data-testid="quest"]')) out.push({ kind: 'text', what: 'the main quest drawn outside its slot' });
  // The program panel: at most one line framed as the current one.
  if (document.querySelectorAll('.lines li.current').length > 1) out.push({ kind: 'text', what: 'more than one program line framed as current' });
  if (/^\\D*0 assigned/.test(document.querySelector('.prog-procs span')?.textContent ?? '') && document.querySelector('.lines li.current')) out.push({ kind: 'text', what: 'a program with no processor frames a line as the one it runs next' });
  // The story banner and the notice sit over the sky: their text is not covered by the art.
  for (const el of document.querySelectorAll('.story-banner, #notice')) {
    const r = el.getBoundingClientRect();
    for (const [x, y] of [[r.left + 20, r.top + 12], [r.right - 20, r.top + 12], [r.right - 20, r.bottom - 12]]) {
      if (y < 0 || y > H) continue;
      const top = document.elementFromPoint(x, y);
      if (top && !el.contains(top) && !document.querySelector('.modal-back')) out.push({ kind: 'hidden', what: label(el) + ' covered at ' + Math.round(x) + ',' + Math.round(y) + ' by ' + label(top) });
    }
  }
  return out;
})()`;

/** Frame gaps: a rAF loop that keeps the worst gap since the last read (visible page only). */
const FRAMES = `(() => {
  if (window.__qaFrames) return;
  window.__qaFrames = { worst: 0, last: performance.now() };
  const f = (t) => {
    const g = window.__qaFrames;
    if (document.visibilityState === 'visible') g.worst = Math.max(g.worst, t - g.last);
    g.last = t;
    requestAnimationFrame(f);
  };
  requestAnimationFrame(f);
})()`;

// ---- one pass: a seed at a width ----------------------------------------------------

const report = { url, seeds, widths, themes, passes: [] };
let failures = 0;

async function pass(seed, width, theme) {
  const tag = `${seed}-${width}-${theme}`;
  const dir = resolve(outDir, tag);
  mkdirSync(dir, { recursive: true });
  const b = await launch(width);
  const { evaluate, send } = b;
  const findings = [];
  const shots = [];
  const heap = [];
  const seenFindings = new Set();
  const artSeen = new Set();
  let moment = 0;
  /** Clicks and times how long the screen takes to change (a render after the click). */
  const timedClick = async (sel, what = sel) => {
    const ms = await evaluate(`new Promise((done) => {
      const b = document.querySelector(${q(sel)});
      if (!b) return done(-1);
      // Already selected: the click changes nothing, so there is nothing to time.
      if (b.classList.contains('active') || b.getAttribute('aria-selected') === 'true' || b.getAttribute('aria-pressed') === 'true') {
        b.click();
        return done(0);
      }
      const t0 = performance.now();
      const mo = new MutationObserver(() => { mo.disconnect(); done(performance.now() - t0); });
      mo.observe(document.body, { subtree: true, childList: true, characterData: true, attributes: true });
      b.click();
      setTimeout(() => { mo.disconnect(); done(0); }, 1000);
    })`);
    if (ms < 0) throw new Error(`nothing to click at ${sel}`);
    if (ms > INPUT_MS) note('input', `${what} took ${Math.round(ms)} ms to show`);
  };
  const page = driver(b, { url, poll: 80, readyMs: 30000, click: timedClick });
  const { waitFor, exists, day, run, click, setValue, speed, panel } = page;
  const load = async () => {
    await page.load();
    await evaluate(FRAMES);
  };
  const note = (kind, what) => {
    const key = `${kind}:${what.replace(/\d+(\.\d+)?/g, '#')}`;
    if (seenFindings.has(key)) return;
    seenFindings.add(key);
    findings.push({ kind, what, moment, day: lastDay });
  };
  let lastDay = 0;
  /** A meaningful moment: audit the screen and keep a screenshot. */
  const capture = async (name) => {
    moment++;
    lastDay = await day().catch(() => lastDay);
    for (const f of await evaluate(AUDIT)) note(f.kind, f.what);
    for (const id of await evaluate(`[...document.querySelectorAll('.card.building img.card-art')].filter((i) => i.complete && i.naturalWidth === 192).map((i) => i.closest('.card').dataset.id)`)) artSeen.add(id);
    const worst = await evaluate(`(() => { const g = window.__qaFrames; const w = g ? g.worst : 0; if (g) g.worst = 0; return w; })()`);
    if (worst > FRAME_MS) note('frame', `a ${Math.round(worst)} ms frame gap before "${name}"`);
    while (b.errors.length) note('console', b.errors.shift());
    const h = await send('Runtime.getHeapUsage');
    heap.push(h.result?.usedSize ?? 0);
    const r = await send('Page.captureScreenshot', { format: 'png' });
    const file = `${String(moment).padStart(3, '0')}-${name.replace(/[^\w-]+/g, '_')}.png`;
    writeFileSync(resolve(dir, file), Buffer.from(r.result.data, 'base64'));
    shots.push({ file, name, day: lastDay });
  };

  /** Real mouse input at a point (modifiers: 8 Shift), clickCount clicks in a row. */
  const mouse = async (x, y, { modifiers = 0, clicks = 1 } = {}) => {
    await send('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y, modifiers });
    for (let n = 1; n <= clicks; n++) {
      await send('Input.dispatchMouseEvent', { type: 'mousePressed', x, y, button: 'left', clickCount: n, modifiers });
      await send('Input.dispatchMouseEvent', { type: 'mouseReleased', x, y, button: 'left', clickCount: n, modifiers });
    }
    await sleep(120);
  };
  const key = async (k, code, modifiers = 0) => {
    const text = k === 'Enter' ? '\r' : k;
    await send('Input.dispatchKeyEvent', { type: 'keyDown', key: k, code, modifiers, text, windowsVirtualKeyCode: k === 'Enter' ? 13 : k.charCodeAt(0) });
    await send('Input.dispatchKeyEvent', { type: 'keyUp', key: k, code, modifiers, windowsVirtualKeyCode: k === 'Enter' ? 13 : k.charCodeAt(0) });
    await sleep(120);
  };
  /** Where an element is, scrolled into view: its center, or null. */
  const at = (sel) =>
    evaluate(`(() => { const e = document.querySelector(${q(sel)}); if (!e) return null; e.scrollIntoView({ block: 'center' }); const r = e.getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; })()`);
  const ownedOf = (id) => evaluate(`Number(document.querySelector('.card.building[data-id="${id}"]')?.dataset.owned ?? -1)`);
  const selection = () => evaluate('String(getSelection())');
  /** Click-to-buy with real input, paused: a click, a Shift-click, a double-click, a
   * click on a card's disabled control, Enter and space on a focused card. Each must buy
   * what it says and select no text; space must not also toggle the clock. */
  const cardProbe = async (when) => {
    await speed(0);
    // A run that just ended: its summary covers the cards until Continue.
    if (await exists('[data-testid="summary"]')) {
      await capture(`summary-before-cards-${when}`);
      await click('[data-testid="continue"]', 'continue');
      await speed(0);
    }
    await panel('buildings');
    // Past the 0.5 s in which cards ignore pointer clicks after a story's OK or Continue
    // (qa-findings #31): a probe right after a story would otherwise report a lost buy.
    await sleep(600);
    const card = (sel) => evaluate(`document.querySelector(${q(sel)})?.dataset.id ?? null`);
    let id = await card('.card.building.can-buy');
    if (!id) return note('input', `card probe (${when}): no affordable card`);
    const desc = `.card.building[data-id="${id}"] .desc`;
    let n0 = await ownedOf(id);
    let p = await at(desc);
    await mouse(p.x, p.y);
    if ((await ownedOf(id)) !== n0 + 1) note('input', `card probe (${when}): a click on ${id} bought ${(await ownedOf(id)) - n0}`);
    // Space after a click on a card is the pause key, not a second buy.
    const n1 = await ownedOf(id);
    await key(' ', 'Space');
    const resumed = await evaluate(`document.querySelector('#speeds button[aria-pressed="true"]')?.dataset.speed`);
    if ((await ownedOf(id)) !== n1 || resumed === '0') note('input', `card probe (${when}): space after a click on ${id} bought ${(await ownedOf(id)) - n1} and left the speed at ${resumed}`);
    await speed(0);
    // The hover state, for the contrast audit.
    await capture(`cards-hover-${when}`);
    id = await card('.card.building.can-buy');
    if (id) {
      n0 = await ownedOf(id);
      p = await at(`.card.building[data-id="${id}"] .desc`);
      await mouse(p.x, p.y, { modifiers: 8 });
      const got = (await ownedOf(id)) - n0;
      if (got < 1 || got > 10) note('input', `card probe (${when}): a Shift-click on ${id} bought ${got}`);
      if (await selection()) note('input', `card probe (${when}): a Shift-click on a card selected text "${(await selection()).slice(0, 40)}"`);
    }
    id = await card('.card.building.can-buy');
    if (id) {
      n0 = await ownedOf(id);
      p = await at(`.card.building[data-id="${id}"] h3`);
      await mouse(p.x, p.y, { clicks: 2 });
      const got = (await ownedOf(id)) - n0;
      if (got < 1 || got > 2) note('input', `card probe (${when}): a double-click on ${id} bought ${got}`);
      if (await selection()) note('input', `card probe (${when}): a double-click on a card selected text "${(await selection()).slice(0, 40)}"`);
    }
    // A disabled control on a green card: the card must not buy through it.
    id = await card('.card.building.can-buy');
    const dis = id && (await exists(`.card.building[data-id="${id}"] button:disabled`));
    if (dis) {
      n0 = await ownedOf(id);
      p = await at(`.card.building[data-id="${id}"] button:disabled`);
      await mouse(p.x, p.y);
      if ((await ownedOf(id)) !== n0) note('input', `card probe (${when}): a click on ${id}'s disabled control bought ${(await ownedOf(id)) - n0}`);
    }
    // Keyboard: Enter and space on the focused card buy one each; space leaves the clock.
    id = await card('.card.building.can-buy');
    if (id) {
      n0 = await ownedOf(id);
      await evaluate(`document.querySelector('.card.building[data-id="${id}"]').focus()`);
      await key('Enter', 'Enter');
      const afterEnter = await ownedOf(id);
      if (afterEnter !== n0 + 1) note('input', `card probe (${when}): Enter on ${id} bought ${afterEnter - n0}`);
      const now = await ownedOf(id);
      const buyable = await evaluate(`document.querySelector('.card.building[data-id="${id}"]').classList.contains('can-buy')`);
      await key(' ', 'Space');
      const paused = await evaluate(`document.querySelector('#speeds button[aria-pressed="true"]')?.dataset.speed`);
      if (paused !== '0') note('input', `card probe (${when}): space on a focused card also started the clock (speed ${paused})`);
      if (buyable && (await ownedOf(id)) !== now + 1) note('input', `card probe (${when}): space on ${id} bought ${(await ownedOf(id)) - now}`);
      await evaluate('document.activeElement?.blur()');
    }
    // A grey card says why, and the hint is readable.
    const grey = await card('.card.building.cannot-buy');
    if (grey) {
      n0 = await ownedOf(grey);
      p = await at(`.card.building[data-id="${grey}"] .desc`);
      await mouse(p.x, p.y);
      const hint = await evaluate(`document.querySelector('.card.building[data-id="${grey}"] .buy-hint').textContent`);
      if ((await ownedOf(grey)) !== n0 || !hint) note('input', `card probe (${when}): grey ${grey}: bought ${(await ownedOf(grey)) - n0}, hint "${hint}"`);
      await capture(`cards-grey-hint-${when}`);
    }
    await evaluate('getSelection().removeAllRanges(); document.querySelector(".col-mid").scrollTop = 0');
  };
  /** The program panel at a speed: below 10x, one line framed (when processors run) and
   * nothing in a line moving as the frame walks; above, no frame and the span note. */
  const frameProbe = async (s, when) => {
    await speed(s);
    const boxes = [];
    let running = false;
    let tinted = 0;
    for (let i = 0; i < 8; i++) {
      await sleep(s >= 50 ? 150 : 400);
      const r = await evaluate(`(() => {
        const p = document.querySelector('.prog-procs span')?.textContent ?? '';
        const lines = [...document.querySelectorAll('.lines li')];
        return { running: !/^\\D*0 assigned/.test(p),
          n: lines.length, cur: lines.map((l, i) => l.classList.contains('current') ? i : -1).filter((i) => i >= 0),
          out: lines.filter((l) => /out-/.test(l.className)).length,
          span: !!document.querySelector('[data-testid="lines-span"]'),
          // Note rows shown (a line keeps its row once a note has shown): a first note
          // may grow its line once; any other move is a finding.
          notes: document.querySelectorAll('.lines .line-why').length,
          box: lines.map((l) => { const a = l.getBoundingClientRect(), b = l.querySelector('.line-name').getBoundingClientRect(); return [a.top, a.height, b.left, b.top].map(Math.round).join(','); }).join(';') };
      })()`);
      if (!r.n) return;
      boxes.push(`${r.notes}#${r.box}`);
      running = r.running;
      tinted = Math.max(tinted, r.out);
      if (s > 10) {
        if (r.cur.length) note('text', `program frame (${when}) at ${s}x: a line framed as current`);
        if (!r.span) note('text', `program lines (${when}) at ${s}x: no "days per redraw" note`);
      } else if (r.running && r.cur.length !== 1) note('text', `program frame (${when}) at ${s}x: ${r.cur.length} lines framed`);
    }
    // Which boxes (line top, height, name left, name top per line) it went through.
    // Moves among samples with the same note rows (a first note's row is allowed).
    const byNotes = new Map();
    for (const k of boxes) {
      const [n, box] = k.split('#');
      byNotes.set(n, (byNotes.get(n) ?? new Set()).add(box));
    }
    if ([...byNotes.values()].some((s) => s.size > 1)) note('overflow', `program lines (${when}) at ${s}x moved as the frame walked: ${[...new Set(boxes)].slice(0, 3).join(' | ')}; why: ${await evaluate(`[...document.querySelectorAll('.lines li')].map((l) => l.className + ' ' + (l.querySelector('[data-line-why]')?.textContent ?? '-')).join(' / ')`)}`);
    if (s > 10 && running && tinted === 0) note('text', `program lines (${when}) at ${s}x: no line tinted with what ran over the redraw`);
    await capture(`program-${s}x-${when}`);
    await speed(0);
  };

  /** Resources' breakdowns, paused: a click on each row opens its breakdown (one at a
   * time; Processors has none) and a second closes it; Stats lists the breakdowns. A shot
   * of Energy's and of the first with a shortfall (the engine test
   * `every_days_breakdown_adds_up_…` checks the numbers). */
  const breakdownProbe = async (when) => {
    await speed(0);
    const ids = await evaluate(`[...document.querySelectorAll('tr.res-row')].map((t) => t.dataset.res)`);
    let shotShort = false;
    for (const id of ids) {
      // Processors' row opens nothing, so there is no change to time.
      if (id === 'processors') await evaluate(`document.querySelector('tr.res-row[data-res="processors"]').click()`);
      else await click(`tr.res-row[data-res="${id}"]`, `resource row ${id}`);
      const r = await evaluate(`(() => {
        const open = [...document.querySelectorAll('tr.res-breakdown')].map((t) => t.dataset.resBreakdown);
        const b = document.querySelector('tr.res-breakdown .breakdown');
        return { open, rows: b ? b.querySelectorAll('tbody tr').length : 0, shorts: b ? b.querySelectorAll('[data-short]').length : 0,
          empty: !!document.querySelector('tr.res-breakdown .breakdown-empty') };
      })()`);
      if (id === 'processors') {
        if (r.open.includes('processors')) note('input', `breakdown (${when}): Processors opened a breakdown`);
        continue;
      }
      if (r.open.length !== 1 || r.open[0] !== id) note('input', `breakdown (${when}): a click on ${id} left open ${JSON.stringify(r.open)}`);
      if (id === 'energy' || (r.shorts && !shotShort)) {
        if (id !== 'energy') shotShort = true;
        await capture(`breakdown-${id}-${when}`);
      }
    }
    if (ids.length) {
      const last = ids.filter((x) => x !== 'processors').at(-1);
      if (last) {
        await click(`tr.res-row[data-res="${last}"]`, `resource row ${last} again`);
        if (await exists('tr.res-breakdown')) note('input', `breakdown (${when}): a second click on ${last} did not close it`);
      }
    }
    // Stats: one breakdown per resource the engine reports, with the average column.
    await panel('stats');
    const stats = await evaluate(`[...document.querySelectorAll('[data-panel="stats"] .breakdown')].map((b) => b.dataset.breakdown)`);
    if (!stats.length) note('text', `breakdown (${when}): Stats shows no resource breakdown`);
    await capture(`stats-breakdown-${when}`);
  };
  /** Launch Pads: a loaded pad switched to another good returns what its tooltip said to
   * the old good's stockpile (a cargo_lost line for the rest), and is empty after. */
  const padProbe = async (when) => {
    await speed(0);
    if (!(await exists('.nav-tile[data-nav="launch_pads"]'))) return;
    await panel('launch_pads');
    const p = await evaluate(`(() => {
      const c = [...document.querySelectorAll('.card.pad')].find((x) => Number(x.dataset.cargo) > 0);
      if (!c) return null;
      const sel = c.querySelector('select');
      const t = c.querySelector('label[title]')?.getAttribute('title') ?? '';
      const m = t.match(/returns ([\\d.,K]+) of the ([\\d.,K]+)/);
      const other = [...sel.options].map((o) => o.value).find((g) => g !== sel.value);
      return { pad: Number(c.dataset.pad), good: sel.value, other, returns: m ? Number(m[1].replace(/,/g, '')) : null, cargo: Number(c.dataset.cargo) };
    })()`);
    // The scenario starts with its pads loading; a played run may have none loaded now.
    if (!p) return when === 'late_unlocks' ? note('input', `pad probe (${when}): no loaded pad to switch`) : undefined;
    if (p.returns === null) return note('text', `pad probe (${when}): a loaded pad's cargo has no switch tooltip`);
    const amt = (g) => evaluate(`Number(document.querySelector('tr[data-res="${g}"]')?.dataset.amount ?? 0)`);
    const lost = () => evaluate(`document.querySelectorAll('[data-testid="events-activity"] li[data-kind="cargo_lost"]').length`);
    const [a0, l0] = [await amt(p.good), await lost()];
    await capture(`pad-before-switch-${when}`);
    await setValue(`.card.pad[data-pad="${p.pad}"] select`, p.other);
    await sleep(200);
    const [a1, l1] = [await amt(p.good), await lost()];
    const cargo = Number(await evaluate(`document.querySelector('.card.pad[data-pad="${p.pad}"]').dataset.cargo`));
    if (Math.abs(a1 - a0 - p.returns) > 0.11) note('input', `pad probe (${when}): switching pad ${p.pad + 1} returned ${(a1 - a0).toFixed(2)} ${p.good}, its tooltip said ${p.returns}`);
    if (cargo !== 0) note('input', `pad probe (${when}): pad ${p.pad + 1} holds ${cargo} after switching`);
    if (p.returns < p.cargo - 0.01 && l1 !== l0 + 1) note('text', `pad probe (${when}): ${(p.cargo - p.returns).toFixed(2)} lost switching, but no cargo_lost line`);
    await capture(`pad-after-switch-${when}`);
    await setValue(`.card.pad[data-pad="${p.pad}"] select`, p.good);
  };

  /** Events' unread counts: Recent Activity folded counts what arrives while it is shut;
   * opened and on screen for over a second, the count clears. */
  const unreadProbe = async (when) => {
    const sec = '[data-testid="events-activity"]';
    const count = () => evaluate(`Number(document.querySelector('${sec}')?.dataset.unread ?? -1)`);
    if (await evaluate(`document.querySelector('${sec} .fold-head').getAttribute('aria-expanded') === 'true'`)) await click(`${sec} .fold-head`, 'activity fold');
    const n0 = await count();
    await speed(50);
    for (let i = 0; i < 40 && (await count()) <= n0; i++) await sleep(150);
    await speed(0);
    const n1 = await count();
    if (n1 <= n0) return note('input', `unread (${when}): no activity arrived in ~6 s at 50x to count`);
    await capture(`unread-folded-${when}`);
    await click(`${sec} .fold-head`, 'activity fold');
    await evaluate(`document.querySelector('${sec}').scrollIntoView({ block: 'start' })`);
    // The lines' boxes while unread and once read: being read must move nothing.
    const boxes = () => evaluate(`[...document.querySelectorAll('${sec} li')].map((l) => { const r = l.getBoundingClientRect(), t = document.createRange(); t.selectNodeContents(l); const c = t.getBoundingClientRect(); return [r.top, r.height, c.left].map(Math.round).join(','); }).join(';')`);
    const unreadBoxes = await boxes();
    await sleep(1800);
    const n2 = await count();
    if (n2 !== 0) note('input', `unread (${when}): Recent Activity open and on screen 1.8 s still counts ${n2} unread`);
    else if ((await boxes()) !== unreadBoxes) note('overflow', `unread (${when}): Recent Activity's lines moved as they were read`);
  };

  try {
    // A fresh browser: first visit, then a new game on the pass's seed from the debug menu.
    await load();
    await capture('first-visit');
    if (await exists('[data-testid="story-banner"]')) await click('[data-testid="story-banner"] button', 'story OK');
    await panel('options');
    // The pass's theme, picked from Options as a player would.
    if (theme !== 'light') await click(`#theme [data-theme-pick="${theme}"]`, `theme ${theme}`);
    if ((await evaluate('document.documentElement.dataset.theme')) !== theme) note('input', `the Options theme toggle did not switch to ${theme}`);
    await click('[data-testid="debug"] .fold-head', 'debug fold');
    await setValue('#seed', seed);
    await click('#new-game', 'new game');
    await capture('new-game');
    await click('[data-testid="story-banner"] button', 'story OK');

    // Run 1, played from the screen: buildings in a fixed order, a program that sells
    // compute or buys what the next building lacks, loads and launches.
    const ORDER = [
      ['solar_panel', 2], ['excavator', 1], ['smelter', 1], ['refinery', 1], ['solar_panel', 3],
      ['excavator', 2], ['storage_depot', 1], ['solar_panel', 4], ['launch_pad', 1], ['solar_panel', 5],
      ['excavator', 3], ['refinery', 2], ['solar_panel', 7], ['data_center', 1], ['research_lab', 1],
    ];
    const look = () =>
      evaluate(`(() => {
        const res = {};
        for (const tr of document.querySelectorAll('tr[data-res]')) res[tr.dataset.res] = Number(tr.dataset.amount);
        return { res, quest: document.querySelector('[data-testid="quest"]')?.dataset.quest ?? null,
          lines: [...document.querySelectorAll('.lines li')].map((li) => li.dataset.command),
          stories: document.querySelector('[data-testid="story-banner"]')?.dataset.story ?? null,
          summary: !!document.querySelector('[data-testid="summary"]'),
          nav: [...document.querySelectorAll('.nav-tile')].map((t) => t.dataset.nav) };
      })()`);
    const buildingsNow = () =>
      evaluate(`(() => {
        const out = {};
        for (const c of document.querySelectorAll('.card.building')) {
          out[c.dataset.id] = { owned: Number(c.dataset.owned), short: (c.dataset.short ?? '').split(' ').filter(Boolean), buyable: c.classList.contains('can-buy') };
        }
        return out;
      })()`);
    const runOnly = async (cmd, lines) => {
      if (lines.length === 1 && lines[0] === cmd) return;
      await click('.prog-tabs button[data-program="0"]', 'program 1 tab');
      if (lines.length) await click('.prog-procs button[title="Clear this program"]', 'clear program');
      await panel('commands');
      // A per-target command shares its verb's card: pick it so its Add button shows.
      await evaluate(`(() => {
        const o = document.querySelector('.panel[data-panel="commands"] select option[value="${cmd}"]');
        if (o && o.parentElement.value !== '${cmd}') {
          o.parentElement.value = '${cmd}';
          o.parentElement.dispatchEvent(new Event('change', { bubbles: true }));
        }
      })()`);
      if (await exists(`[data-testid="add-${cmd}"]:not([disabled])`)) await click(`[data-testid="add-${cmd}"]`, `add ${cmd}`);
    };
    const seenNav = new Set();
    let quest = null;
    let story = null;
    /** Screens worth a look as they come: a new tile, a story text, a new quest. */
    const moments = async (s) => {
      // Read every story text waiting, capturing each new one.
      for (let i = 0; i < 10; i++) {
        const id = await evaluate(`document.querySelector('[data-testid="story-banner"]')?.dataset.story ?? null`);
        if (!id) break;
        if (id !== story) {
          story = id;
          await capture(`story-${story}`);
        }
        await click('[data-testid="story-banner"] button', 'story OK');
      }
      if (s.quest !== quest) {
        quest = s.quest;
        await capture(`quest-${quest}`);
      }
      for (const id of s.nav)
        if (!seenNav.has(id)) {
          seenNav.add(id);
          await panel(id);
          await capture(`panel-${id}`);
        }
    };
    await speed(3);
    await sleep(1500);
    await speed(RUN1_SPEED);
    const started = Date.now();
    while (Date.now() - started < (RUN1_SPEED >= 50 ? 240_000 : 900_000)) {
      const s = await look();
      if (s.summary) break;
      await moments(s);
      if (s.nav.includes('retirement') && (await evaluate(`!document.querySelector('[data-testid="retire"]')?.disabled`).catch(() => false))) break;
      await panel('buildings');
      const bs = await buildingsNow();
      const goal = ORDER.find(([id, n]) => bs[id] && bs[id].owned < n);
      if (goal && bs[goal[0]].buyable) {
        // The run may end (a forced retirement's summary) between the read and the click.
        if (await exists(`[data-testid="buy-${goal[0]}"]`)) await click(`[data-testid="buy-${goal[0]}"]`, `buy ${goal[0]}`);
        continue;
      }
      const credits = s.res.credits ?? 0;
      let cmd = 'cloud_compute';
      if ((bs.launch_pad?.owned ?? 0) > 0) {
        await panel('launch_pads');
        const cargo = Number(await evaluate(`document.querySelector('[data-pad="0"]')?.dataset.cargo ?? 0`));
        if (cargo >= 10 && (await evaluate(`!document.querySelector('[data-testid="launch-0"]')?.disabled`))) {
          await click('[data-testid="launch-0"]', 'launch');
          await capture('launched');
          continue;
        }
        if ((s.res.propellant ?? 0) < 20 && credits >= 12) cmd = 'buy_propellant';
        else if ((s.res.he3 ?? 0) >= 5 && cargo < 10) cmd = 'load_pads';
      } else if (goal) {
        const short = bs[goal[0]].short;
        if (short.includes('titanium') && credits >= 25) cmd = 'buy_titanium';
        else if (short.includes('regolith') && credits >= 12) cmd = 'buy_regolith';
      }
      await runOnly(cmd, s.lines);
      await sleep(60);
    }
    // The program panel's frame at a person's speed and the span coloring above 10x.
    if (!(await exists('[data-testid="summary"]'))) {
      await frameProbe(3, 'run1');
      if (!(await exists('[data-testid="summary"]'))) await frameProbe(50, 'run1');
      await speed(RUN1_SPEED);
    }
    // Every panel once at the end of run 1, and the Events folds open.
    for (const id of (await look()).nav) {
      await moments(await look());
      await panel(id);
      await capture(`run1-${id}`);
    }
    await click('[data-testid="events-completed"] .fold-head', 'completed quests fold');
    await capture('run1-events-open');
    if (!(await exists('[data-testid="summary"]'))) {
      await breakdownProbe('run1');
      await padProbe('run1');
    }
    // Retire (unless boredom got there first), the summary, run 2.
    if (!(await exists('[data-testid="summary"]'))) {
      await panel('retirement');
      await click('[data-testid="retire"]', 'retire');
      await capture('retire-confirm');
      await click('[data-testid="retire-confirm"]', 'retire confirm');
      await waitFor(`!!document.querySelector('[data-testid="summary"]')`, 'the run summary', 3000);
    }
    await capture('run-summary');
    // Keys while the summary is up must not start the clock.
    const d0 = await day();
    await evaluate(`document.body.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))`);
    await evaluate(`document.body.dispatchEvent(new KeyboardEvent('keydown', { key: '3', bubbles: true }))`);
    await sleep(800);
    if ((await day()) !== d0) note('input', 'keys during the run summary started the clock');
    // A reload with the summary up: it is still up, the clock still stopped.
    await load();
    await sleep(800);
    if (!(await exists('[data-testid="summary"]')) || (await day()) !== d0) note('input', `reload during the run summary: summary ${await exists('[data-testid="summary"]')}, day ${d0} -> ${await day()}`);
    await capture('run-summary-reloaded');
    await click('[data-testid="continue"]', 'continue');
    if ((await run()) !== 2) note('input', `after Continue the run is ${await run()}, not 2`);
    await capture('run2-start');
    // Odd input: mash a Buy and the speed keys, resize narrow and back; nothing may throw.
    await panel('buildings');
    await evaluate(`(() => { const b = document.querySelector('[data-testid="buy-solar_panel"]'); for (let i = 0; i < 25; i++) b?.click(); })()`);
    await evaluate(`(() => { for (const k of '1234512345 5 ') document.body.dispatchEvent(new KeyboardEvent('keydown', { key: k, bubbles: true })); })()`);
    await send('Emulation.setDeviceMetricsOverride', { width: 1024, height: 700, deviceScaleFactor: 1, mobile: false });
    await sleep(300);
    await send('Emulation.setDeviceMetricsOverride', { width, height: HEIGHT, deviceScaleFactor: 1, mobile: false });
    await sleep(300);
    await capture('after-mashing');
    // Programs: move the processor to Program 2, build it a two-line program, bump,
    // lower and remove lines; each step must show as the engine has it.
    await click('.prog-tabs button[data-program="0"]', 'program 1 tab');
    await click('[data-testid="proc-minus"]', 'processor off program 1');
    if (!(await exists('.lines li'))) {
      await panel('commands');
      await click('[data-testid="add-cloud_compute"]', 'add cloud compute');
    }
    await capture('program-1-no-processor');
    await click('.prog-tabs button[data-program="1"]', 'program 2 tab');
    await click('[data-testid="proc-plus"]', 'processor onto program 2');
    await panel('commands');
    await click('[data-testid="add-cloud_compute"]', 'add cloud compute');
    await click('[data-testid="add-idle"]', 'add idle');
    await click('.lines li[data-line="0"] button:nth-of-type(2)', 'line +');
    await click('.lines li[data-line="0"] button:nth-of-type(2)', 'line +');
    await click('.lines li[data-line="0"] button:nth-of-type(1)', 'line −');
    await click('.lines li[data-line="1"] button[title="Remove"]', 'line ×');
    const prog = await evaluate(`[...document.querySelectorAll('.lines li')].map((li) => li.dataset.command + ' ' + li.querySelector('.line-name').textContent.trim())`);
    const procs = await evaluate(`document.querySelector('.prog-procs span').textContent`);
    if (prog.length !== 1 || !/\(x2\)$/.test(prog[0]) || !/1 assigned/.test(procs)) note('input', `program 2 edits: ${JSON.stringify(prog)}, ${procs}`);
    // Drag to reorder (HTML drag and drop, dispatched in the page): the last line dropped
    // on the first moves it to the top.
    await click('[data-testid="add-idle"]', 'add idle');
    const order = () => evaluate(`[...document.querySelectorAll('.lines li')].map((li) => li.dataset.command).join(',')`);
    const before = await order();
    // A real drag renders between its events: dispatched one evaluate apart.
    await evaluate(`(() => {
      const lines = document.querySelectorAll('.lines li');
      window.__qaDrag = new DataTransfer();
      lines[lines.length - 1].dispatchEvent(new DragEvent('dragstart', { bubbles: true, dataTransfer: window.__qaDrag }));
    })()`);
    await sleep(100);
    await evaluate(`(() => {
      const lines = document.querySelectorAll('.lines li');
      const dt = window.__qaDrag;
      lines[0].dispatchEvent(new DragEvent('dragover', { bubbles: true, cancelable: true, dataTransfer: dt }));
      lines[0].dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: dt }));
      lines[lines.length - 1].dispatchEvent(new DragEvent('dragend', { bubbles: true, dataTransfer: dt }));
    })()`);
    await sleep(150);
    const after = await order();
    const want = before.split(',');
    want.unshift(want.pop());
    if (after !== want.join(',')) note('input', `program 2: dragging the last line to the top gave ${after}, not ${want.join(',')}`);
    // A drag let go off the list, then something else dropped on a line: nothing moves.
    await evaluate(`document.querySelectorAll('.lines li')[1].dispatchEvent(new DragEvent('dragstart', { bubbles: true, dataTransfer: new DataTransfer() }))`);
    await sleep(100);
    await evaluate(`document.querySelectorAll('.lines li')[1].dispatchEvent(new DragEvent('dragend', { bubbles: true, dataTransfer: new DataTransfer() }))`);
    await sleep(100);
    await evaluate(`document.querySelectorAll('.lines li')[0].dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: new DataTransfer() }))`);
    await sleep(150);
    if ((await order()) !== after) note('input', `program 2: a drop after a cancelled drag moved a line (${after} -> ${await order()})`);
    await click('.lines li[data-line="0"] button[title="Remove"]', 'line × (the dragged one)');
    await speed(10);
    await sleep(600);
    await speed(0);
    await capture('program-2');
    await frameProbe(3, 'program2');
    await frameProbe(200, 'program2');
    // Run 2 at 200x, then a reload: the save must come back.
    await speed(200);
    const d2 = await day();
    const t2 = Date.now();
    while ((await day()) < d2 + RUN2_DAYS && Date.now() - t2 < 60_000) {
      const s = await look();
      if (s.summary) {
        await capture('run2-summary');
        await click('[data-testid="continue"]', 'continue');
      }
      await moments(s);
      await sleep(200);
    }
    await cardProbe('run2');
    for (const id of ['buildings', 'stats', 'story', 'retirement']) {
      if (!(await exists(`.nav-tile[data-nav="${id}"]`))) continue;
      await panel(id);
      await capture(`run2-${id}`);
    }
    await speed(0);
    const savedDay = await day();
    const savedRun = await run();
    await load();
    if ((await day()) !== savedDay || (await run()) !== savedRun) note('input', `reload: day ${savedDay} run ${savedRun} came back as day ${await day()} run ${await run()}`);
    await capture('reloaded');
    // The replay check over the whole career, from the debug menu.
    await panel('options');
    if (!(await exists('#check-replay'))) await click('[data-testid="debug"] .fold-head', 'debug fold');
    await click('#check-replay', 'check replay');
    const n = await evaluate(`document.getElementById('notice')?.textContent ?? ''`);
    if (!/^Replay ok/.test(n)) note('console', `replay check: ${n}`);
    await capture('replay-check');
    // With cheats, on this career (its quests far enough along for research to open
    // Projects and Ideologies): the late panels with science to spend.
    await panel('options');
    if (!(await exists('#cheats-toggle'))) await click('[data-testid="debug"] .fold-head', 'debug fold');
    if (!(await exists('[data-testid="cheats"]'))) await click('#cheats-toggle', 'cheats');
    // Grants fill to the caps, so each buy is followed by a refill (a Storage Depot first,
    // to raise the caps a Data Center's price needs), up to a Research Lab: the late
    // panels (Research, and what research opens) get audited too.
    const grant = async () => {
      await panel('options');
      if (!(await exists('#cheat-resource'))) {
        if (!(await exists('#cheats-toggle'))) await click('[data-testid="debug"] .fold-head', 'debug fold');
        if (!(await exists('[data-testid="cheats"]'))) await click('#cheats-toggle', 'cheats');
      }
      for (const [res, amt] of [['credits', '1000000'], ['science', '100000'], ['regolith', '500'], ['circuits', '200'], ['titanium', '200']]) {
        await setValue('#cheat-resource', res);
        await setValue('#cheat-amount', amt);
        await click('#cheat-grant', `grant ${res}`);
      }
    };
    await grant();
    for (const id of ['storage_depot', 'storage_depot', 'data_center', 'data_center', 'research_lab']) {
      await panel('buildings');
      if (await exists(`[data-testid="buy-${id}"].can-buy`)) await click(`[data-testid="buy-${id}"]`, `buy ${id}`);
      await grant();
    }
    if (!(await exists('.nav-tile[data-nav="research"]'))) note('input', 'the late phase never opened Research: its panels went unaudited');
    for (let round = 0; round < 4; round++) {
      await speed(10);
      await sleep(800);
      await speed(0);
      await moments(await look());
      if (!(await exists('.nav-tile[data-nav="research"]'))) continue;
      await panel('research');
      const buyable = await evaluate(`[...document.querySelectorAll('.card.research[aria-disabled="false"]')].map((b) => b.dataset.testid)`);
      for (const t of buyable) await click(`[data-testid="${t}"]`, t);
    }
    for (const id of (await look()).nav) {
      await panel(id);
      await capture(`late-${id}`);
    }
    await cardProbe('late');
    // The other theme from the debug menu: the late screens again in it.
    const other = theme === 'dark' ? 'light' : 'dark';
    await panel('options');
    if (!(await exists('#debug-theme'))) await click('[data-testid="debug"] .fold-head', 'debug fold');
    await click(`#debug-theme [data-theme-pick="${other}"]`, `theme ${other}`);
    for (const id of ['buildings', 'commands', 'launch_pads', 'research', 'projects', 'ideologies', 'stats']) {
      if (!(await exists(`.nav-tile[data-nav="${id}"]`))) continue;
      await panel(id);
      await capture(`other-theme-${other}-${id}`);
    }
    await panel('options');
    if (!(await exists('#debug-theme'))) await click('[data-testid="debug"] .fold-head', 'debug fold');
    await click(`#debug-theme [data-theme-pick="${theme}"]`, `theme ${theme}`);
    // A forced retirement at 200x: boredom set just under its cap, the clock run.
    await panel('options');
    if (!(await exists('#cheats-toggle'))) await click('[data-testid="debug"] .fold-head', 'debug fold');
    await setValue('#cheat-boredom', '995');
    await click('#cheat-set-boredom', 'set boredom');
    // The summary comes up over Buildings: a double-click on Continue must not let its
    // second click land on whatever the summary covered (a card buys on a click).
    await panel('buildings');
    const runBefore = await run();
    await speed(200);
    try {
      await waitFor(`!!document.querySelector('[data-testid="summary"]')`, 'the forced retirement', 15000);
      const said = await evaluate(`document.querySelector('[data-testid="summary"]').textContent`);
      if (!/Boredom reached its cap/.test(said)) note('text', `forced retirement summary: ${said.slice(0, 80)}`);
      const d = await day();
      await sleep(700);
      if ((await day()) !== d || (await run()) !== runBefore + 1) note('input', `forced retirement: day ${d} -> ${await day()}, run ${runBefore} -> ${await run()}`);
      await capture('forced-summary');
      const owned = `[...document.querySelectorAll('.card.building')].reduce((n, c) => n + Number(c.dataset.owned), 0)`;
      const o0 = await evaluate(owned);
      const p = await evaluate(`(() => { const r = document.querySelector('[data-testid="continue"]').getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; })()`);
      await mouse(p.x, p.y, { clicks: 2 });
      const under = await evaluate(`(() => { const e = document.elementFromPoint(${p.x}, ${p.y}); return e ? (e.closest('.card, button')?.className ?? e.tagName) : ''; })()`);
      const hint = await evaluate(`[...document.querySelectorAll('.buy-hint.short')].map((h) => h.textContent).join(' ')`);
      if ((await evaluate(owned)) !== o0 || hint) note('input', `a double-click on Continue: its second click landed on ${under} (owned ${o0} -> ${await evaluate(owned)}${hint ? `, hint "${hint}"` : ''})`);
      await sleep(500);
      if ((await evaluate(`document.querySelector('#speeds button[aria-pressed="true"]')?.dataset.speed`)) !== '200') note('input', 'Continue after a forced retirement did not restore 200x');
      await speed(0);
    } catch (e) {
      note('input', `forced retirement: ${String(e).slice(0, 120)}`);
    }
    // The save's size (localStorage holds about 5 MB).
    const saveKb = await evaluate(`(localStorage.getItem('hh.save') ?? '').length / 1024`);
    if (saveKb > 2048) note('memory', `the save is ${Math.round(saveKb)} KB`);
    // The play log on disk replays to the logged fingerprint (`hh-sim playlog`).
    await panel('options');
    const career = await evaluate(`document.querySelector('[data-testid="playlog-status"]')?.dataset.career ?? ''`);
    const logDir = await evaluate(`fetch('/__playlog?career=' + encodeURIComponent(${q(career)})).then((r) => r.ok ? r.json() : {}).then((j) => j.dir ?? '').catch(() => '')`);
    const sim = process.env.HH_SIM ?? resolve(here, '../../target/release/hh-sim' + (process.platform === 'win32' ? '.exe' : ''));
    if (career && logDir && existsSync(sim)) {
      await sleep(3000); // the page sends its log every 2 s
      const file = resolve(logDir, `${career}.jsonl`);
      const r = spawnSync(sim, ['playlog', file], { encoding: 'utf-8', cwd: resolve(here, '../..') });
      // Reproduced, or not checkable yet (no checkpoint at the last tick so far): fine.
      if (!/final state reproduced|no fingerprint at its last tick/.test(r.stdout ?? '')) note('console', `hh-sim playlog ${file}: ${(r.stdout + r.stderr).slice(0, 300)}`);
    } else note('console', `play log not checked (career ${career || '-'}, dir ${logDir || '-'}, ${existsSync(sim) ? 'hh-sim found' : `no ${sim}`})`);
    // Every debug-menu scenario, every panel it shows, a few days played. The list is in
    // the Debug fold, closed again once Options remounts: open it before reading.
    await panel('options');
    if (!(await exists('#scenario'))) await click('[data-testid="debug"] .fold-head', 'debug fold');
    const scenarios = await evaluate(`[...document.querySelectorAll('#scenario option')].map((o) => o.value)`);
    if (!scenarios.length) note('input', 'the debug menu listed no scenarios: none audited');
    for (const sc of scenarios) {
      await panel('options');
      if (!(await exists('#scenario'))) await click('[data-testid="debug"] .fold-head', 'debug fold');
      await setValue('#scenario', sc);
      await click('#start-scenario', `start ${sc}`);
      await moments(await look());
      await speed(10);
      await sleep(1500);
      await speed(0);
      for (const id of (await look()).nav) {
        await panel(id);
        await capture(`scenario-${sc}-${id}`);
      }
      // Every scenario's breakdowns (at_cap's lost-to-cap rows, rich's spending); the late
      // panels' scenario also switches cargo on its loaded pads and reads unread counts.
      await breakdownProbe(sc);
      if (sc === 'late_unlocks') {
        await padProbe(sc);
        await unreadProbe(sc);
      }
    }
    // A second tab on the same save: the first stands down with a notice.
    if (seed === seeds[0] && width === widths[0]) {
      const t = await send('Target.createTarget', { url });
      // The new tab must load and save before this one hears of it: up to 10 s on a
      // loaded box.
      let told = '';
      for (let i = 0; i < 20 && !/another tab/.test(told); i++) {
        await sleep(500);
        told = await evaluate(`document.getElementById('notice')?.textContent ?? ''`);
      }
      if (!/another tab/.test(told)) note('input', `a second tab on the same game: the first was not told (${told})`);
      // Close it first: a headless background tab never paints, so no screenshot.
      await send('Target.closeTarget', { targetId: t.result?.targetId });
      await sleep(500);
      // It sat behind the other tab meanwhile (headless keeps it "visible"): no frames is no gap.
      await evaluate(`window.__qaFrames && (window.__qaFrames.worst = 0)`);
      await capture('second-tab');
    }
    // Site data blocked (every localStorage touch throws): the game still loads and plays,
    // and says nothing is being kept. First pass only.
    if (seed === seeds[0] && width === widths[0]) {
      const s = await send('Page.addScriptToEvaluateOnNewDocument', {
        source: `Object.defineProperty(window, 'localStorage', { get() { throw new DOMException('blocked', 'SecurityError'); } });`,
      });
      try {
        await evaluate('window.__stale = true');
        await send('Page.navigate', { url });
        await waitFor('!window.__stale', 'the navigation');
        await waitFor(READY, 'the page with storage blocked', 15000);
        await speed(10);
        await sleep(1200);
        await speed(0);
        const said = await evaluate(`document.getElementById('notice')?.textContent ?? ''`);
        if (!/Autosave failed/.test(said)) note('text', `storage blocked: no autosave notice (notice "${said.slice(0, 80)}")`);
        await capture('storage-blocked');
      } catch (e) {
        note('console', `storage blocked: ${String(e).slice(0, 160)}`);
      }
      await send('Page.removeScriptToEvaluateOnNewDocument', { identifier: s.result?.identifier });
      while (b.errors.length) b.errors.shift(); // the blocked page's own warnings are expected
      await load(); // storage back: the steps below write to it
    }
    // Every building's card (`?all-cards`), in both themes, screen by screen: the pictures
    // all load at 96 px and nothing on a card overflows or sits under its picture.
    const allUrl = url + (url.includes('?') ? '&' : '?') + 'all-cards';
    for (const t of [theme, theme === 'dark' ? 'light' : 'dark']) {
      await evaluate(`localStorage.setItem('hh.theme', ${q(t)})`);
      await evaluate('window.__stale = true');
      await send('Page.navigate', { url: allUrl });
      await waitFor('!window.__stale', 'the navigation');
      await waitFor(READY, 'the page', 30000);
      await speed(0);
      await panel('buildings');
      await waitFor(`[...document.images].every((i) => i.complete)`, 'the card pictures');
      const screens = await evaluate(`Math.ceil(document.querySelector('.col-mid').scrollHeight / document.querySelector('.col-mid').clientHeight)`);
      for (let i = 0; i < screens; i++) {
        await evaluate(`document.querySelector('.col-mid').scrollTop = ${i} * document.querySelector('.col-mid').clientHeight`);
        await sleep(150);
        await capture(`all-cards-${t}-${i + 1}`);
      }
      const missing = artIds.filter((id) => !artSeen.has(id));
      if (missing.length) note('assets', `card pictures never shown loaded (${t}): ${missing.join(', ')}`);
    }
    await evaluate(`localStorage.setItem('hh.theme', ${q(theme)})`);
  } catch (e) {
    note('console', `the pass stopped: ${String(e).slice(0, 300)}`);
    await capture('failure').catch(() => {});
  }
  const grown = (Math.max(...heap) - heap[0]) / 1e6;
  if (grown > HEAP_GROWTH_MB) note('memory', `JS heap grew ${grown.toFixed(1)} MB over the pass`);
  // A contact sheet of the pass: its screenshots on one page, captured whole.
  const sheet = resolve(dir, 'contact.html');
  writeFileSync(
    sheet,
    `<!doctype html><style>body{margin:0;font:12px sans-serif;display:grid;grid-template-columns:repeat(4,1fr);gap:4px;width:2400px}figure{margin:0}img{width:100%;border:1px solid #888}</style>` +
      shots.map((s) => `<figure><img src="${s.file}"><figcaption>${s.file} (day ${s.day})</figcaption></figure>`).join(''),
    { encoding: 'utf-8' },
  );
  try {
    await send('Emulation.setDeviceMetricsOverride', { width: 2400, height: 1200, deviceScaleFactor: 1, mobile: false });
    await send('Page.navigate', { url: pathToFileURL(sheet).href });
    await sleep(1500);
    const r = await send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true });
    writeFileSync(resolve(outDir, `contact-${tag}.png`), Buffer.from(r.result.data, 'base64'));
  } catch {}
  b.close();
  const heapMb = heap.map((h) => Math.round(h / 1e5) / 10);
  const art = artIds.filter((id) => artSeen.has(id));
  report.passes.push({ seed, width, theme, moments: shots.length, heapMb, art: art.length, findings });
  failures += findings.length;
  console.log(`${tag}: ${shots.length} moments, heap ${heapMb[0]}..${Math.max(...heapMb)} MB, card art ${art.length}/${artIds.length} seen, ${findings.length} findings`);
  for (const f of findings) console.log(`  ${f.kind.padEnd(8)} m${f.moment} d${f.day} ${f.what}`);
}

/** HH_PACE=<seconds>: a first visit left at 1x for that long, untouched, a screenshot
 * every 15 s and the day each time: what a new player sees while reading, at real speed. */
async function pace(seconds) {
  const dir = resolve(outDir, 'pace');
  mkdirSync(dir, { recursive: true });
  const b = await launch(widths[0]);
  await b.send('Page.navigate', { url });
  const t0 = Date.now();
  const days = [];
  for (let i = 0; Date.now() - t0 < seconds * 1000; i++) {
    await sleep(Math.max(0, i * 15_000 - (Date.now() - t0)));
    const d = await b.evaluate(`document.getElementById('day')?.textContent ?? ''`).catch(() => '');
    days.push(`${Math.round((Date.now() - t0) / 1000)} s: ${d}`);
    const r = await b.send('Page.captureScreenshot', { format: 'png' });
    writeFileSync(resolve(dir, `${String(i).padStart(2, '0')}.png`), Buffer.from(r.result.data, 'base64'));
  }
  const errs = b.errors.splice(0);
  b.close();
  console.log(`pace: ${days.join(', ')}${errs.length ? `; errors: ${errs.join(' | ')}` : ''}`);
  failures += errs.length;
  report.pace = { days, errors: errs };
}

/** HH_LONG=<minutes>: one career played that long at 200x (Sell Cloud Compute in Program
 * 1, every summary continued, a reload every 2 minutes, the theme flipped every 3): each
 * minute the JS heap, the DOM size, the save's size and the worst frame gap; a finding
 * on heap or DOM growth past the first minute's, a frame gap, a save over 2 MB or an error. */
async function longRun(minutes) {
  const dir = resolve(outDir, 'long');
  mkdirSync(dir, { recursive: true });
  const b = await launch(widths[0]);
  const { evaluate, send } = b;
  const findings = [];
  const rows = [];
  const until = async (expr, ms = 30000) => {
    for (const end = Date.now() + ms; Date.now() < end; await sleep(100)) if (await evaluate(expr)) return true;
    return false;
  };
  const js = (s) => evaluate(`(() => { ${s} })()`);
  const open = async () => {
    await evaluate('window.__stale = true');
    await send('Page.navigate', { url });
    await until('!window.__stale');
    await until(READY);
    await evaluate(FRAMES);
  };
  await open();
  await js(`document.querySelector('[data-testid="story-banner"] button')?.click()`);
  await js(`document.querySelector('.nav-tile[data-nav="commands"]').click()`);
  await sleep(200);
  await js(`document.querySelector('[data-testid="add-cloud_compute"]')?.click()`);
  await js(`document.querySelector('#speeds button[data-speed="200"]').click()`);
  const t0 = Date.now();
  let lastReload = t0;
  let lastTheme = t0;
  let minute = 0;
  while (Date.now() - t0 < minutes * 60_000) {
    await sleep(2000);
    // Story texts, summaries and an empty program after a new run: as a player would.
    await js(`for (let i = 0; i < 10; i++) document.querySelector('[data-testid="story-banner"] button')?.click();
      document.querySelector('[data-testid="continue"]')?.click();
      if (!document.querySelector('.lines li') && document.querySelector('[data-panel="commands"]')) document.querySelector('[data-testid="add-cloud_compute"]')?.click();
      if (!document.querySelector('[data-panel="commands"]')) document.querySelector('.nav-tile[data-nav="commands"]')?.click();
      if (document.querySelector('#speeds button[aria-pressed="true"]')?.dataset.speed !== '200') document.querySelector('#speeds button[data-speed="200"]').click();`);
    if (Date.now() - lastTheme > 180_000) {
      lastTheme = Date.now();
      await js(`localStorage.setItem('hh.theme', document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark'); document.documentElement.dataset.theme = localStorage.getItem('hh.theme')`);
    }
    if (Date.now() - lastReload > 120_000) {
      lastReload = Date.now();
      await open();
    }
    if (Date.now() - t0 < (minute + 1) * 60_000) continue;
    minute++;
    await send('HeapProfiler.collectGarbage').catch(() => {});
    const heap = (await send('Runtime.getHeapUsage')).result?.usedSize ?? 0;
    const row = await evaluate(`(() => { const g = window.__qaFrames; const w = g ? g.worst : 0; if (g) g.worst = 0;
      return { day: document.getElementById('day').textContent, run: document.getElementById('run').textContent,
        nodes: document.getElementsByTagName('*').length, saveKb: Math.round((localStorage.getItem('hh.save') ?? '').length / 1024), worstFrame: Math.round(w) }; })()`);
    rows.push({ minute, heapMb: Math.round(heap / 1e5) / 10, ...row });
    const r = await send('Page.captureScreenshot', { format: 'png' });
    writeFileSync(resolve(dir, `${String(minute).padStart(2, '0')}.png`), Buffer.from(r.result.data, 'base64'));
    console.log(`long: ${JSON.stringify(rows.at(-1))}`);
  }
  const errs = b.errors.splice(0);
  b.close();
  const first = rows[0];
  const last = rows.at(-1);
  if (first && last) {
    if (last.heapMb - first.heapMb > HEAP_GROWTH_MB / 2) findings.push(`heap grew ${first.heapMb} -> ${last.heapMb} MB`);
    if (last.nodes > first.nodes * 2 + 200) findings.push(`DOM grew ${first.nodes} -> ${last.nodes} elements`);
    if (last.saveKb > 2048) findings.push(`the save is ${last.saveKb} KB`);
    for (const r of rows) if (r.worstFrame > FRAME_MS) findings.push(`minute ${r.minute}: a ${r.worstFrame} ms frame gap`);
  }
  for (const e of errs) findings.push(`console: ${e}`);
  report.long = { rows, findings };
  failures += findings.length;
  for (const f of findings) console.log(`  long: ${f}`);
}

if (process.env.HH_PACE) await pace(Number(process.env.HH_PACE));
else if (process.env.HH_LONG) await longRun(Number(process.env.HH_LONG));
else {
  // Themes alternate like a checkerboard over seeds and widths, so every width meets each.
  for (const [i, seed] of seeds.entries()) for (const [j, width] of widths.entries()) await pass(seed, width, themes[(i + j) % themes.length]);
}
writeFileSync(resolve(outDir, 'report.json'), JSON.stringify(report, null, 1), { encoding: 'utf-8' });
console.log(`qa: ${failures} findings over ${report.passes.length} passes; ${outDir}`);
process.exit(failures ? 1 : 0);
