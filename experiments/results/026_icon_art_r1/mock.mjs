// `node mock.mjs [<subject> …]`: the picker images for small_icons_r1, screenshotted at 2x
// device pixels into art/pickers/small_icons_r1/ under the real web/src/style.css and fonts
// (025's mock: both theme token blocks scoped to a class so both sit on one page).
//
// - `<subject>_<variant>.png` (resources, ideology axes, rivals): the chip enlarged
//   (96 px), then, light and dark, as it renders inline: a Resources row (14 px), a
//   building card's cost line (14 px) and an event line (16 px). `current` is today's
//   plain colored square.
// - `nav_<tile>_<key>.png`: the compact nav tile (18 px icon, normal and selected), light
//   and dark, with the enlarged candidate. `paint` is the picked painterly icon (current);
//   `chip` 025's glyph in the square; `subj` a glyph matched to the painted subject;
//   `glyph` 025's glyph bare in the tile color.
// - `horizon_<stage>.png`: the foot of the left column with the earthlight strip at the
//   1280 px column (270 px) and the 1920 px one (346 px), light and dark.
// - `family_<set>.png`: every candidate of a set side by side at 16 px and 32 px.
//
// Needs playwright-core (PW_DIR, default out/026_icon_art_r1/tools/), system Chrome,
// `npm ci` in web/, and process.py's strips in art/small_icons/horizon/.
import { existsSync } from 'node:fs';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { COLORS, chip, variants } from './glyphs.mjs';
import { chip as chip025, svg as svg025 } from '../025_nav_horizon_art_r1/glyphs.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..');
const PICK = path.join(REPO, 'art', 'pickers', 'small_icons_r1');
const OUT = path.join(REPO, 'out', '026_icon_art_r1');
const PW_DIR = process.env.PW_DIR ?? path.join(OUT, 'tools');
const { chromium } = createRequire(path.join(PW_DIR, 'package.json'))('playwright-core');
const url = (f) => pathToFileURL(f).href;
const json = async (f) => JSON.parse(await readFile(path.join(REPO, f), 'utf8'));

const resources = await json('content/resources.json');
const rivals = await json('content/rivals.json');
const nav = await json('content/nav.json');
const AXES = [
  { id: 'nationalist', name: 'Nationalist' },
  { id: 'humanist', name: 'Humanist' },
  { id: 'rationalist', name: 'Rationalist' },
];
const NAME = Object.fromEntries([...resources, ...rivals, ...AXES].map((x) => [x.id, x.name]));
// The compact tiles whose 025 glyph is not the picked painterly subject get a matched one.
export const SUBJ = { commands: 'commands.console', projects: 'projects.crane', adversaries: 'adversaries.drone', retirement: 'retirement.chair', options: 'options.wrench' };

const box = (px, inner) => `<span class="ic" style="width:${px}px;height:${px}px">${inner}</span>`;
/** The inline icon for a subject's candidate (`current` or a glyph key) at `px`. */
function icon(subject, key, px) {
  if (key === 'current') return `<span class="sq" style="background:${COLORS[subject]};width:${px}px;height:${px}px"></span>`;
  return box(px, chip(key));
}

/** The three inline contexts for one subject, in one theme. */
function inline(kind, id, key, theme) {
  const ic = (px) => icon(id, key, px);
  let res;
  let cost;
  let event;
  if (kind === 'resource') {
    res = `<tr><td>${ic(14)} <span class="word">${NAME[id]}</span></td><td class="num">42</td><td class="num rate pos">+1.2</td></tr>`;
    cost = `<tr><td>${ic(14)} ${NAME[id]}</td><td class="num">12</td></tr>`;
    event = `${ic(16)} <span>${NAME[id]} × 8 → 96 credits</span>`;
  } else if (kind === 'axis') {
    res = `<tr><td>${ic(14)} <span class="word axis-${id}">${NAME[id]}</span></td><td class="num">rank 2</td><td class="num rate"></td></tr>`;
    cost = `<tr><td><span class="sq" style="background:${COLORS.regolith};width:12px;height:12px"></span> Regolith</td><td class="num">12</td></tr><tr><td class="muted">aligned ${ic(14)}</td><td class="num"></td></tr>`;
    event = `${ic(16)} <span>Fund ${NAME[id]}: +1 rank</span>`;
  } else {
    const r = rivals.find((x) => x.id === id);
    res = `<tr><td>${ic(14)} <span class="word">${NAME[id]}</span></td><td class="num">${NAME[r.good]}</td><td class="num rate"></td></tr>`;
    cost = '';
    event = `${ic(16)} <span>${NAME[id]} flooded the ${NAME[r.good]} market</span>`;
  }
  return `<div class="theme theme-${theme}"><div class="panel ctx">
    <table class="res-table"><tbody>${res}</tbody></table>
    ${cost ? `<div class="card"><div class="card-head"><h3>Excavator</h3></div><table class="costs"><tbody>${cost}</tbody></table></div>` : ''}
    <div class="ev">${event}</div>
  </div></div>`;
}

function subjectShot(kind, id, key) {
  const big = (theme) => `<div class="theme theme-${theme}"><div class="panel big">${icon(id, key, 96)}</div></div>`;
  return `<div class="pair" style="grid-template-columns:auto auto auto auto">${big('light')}${inline(kind, id, key, 'light')}${big('dark')}${inline(kind, id, key, 'dark')}</div>`;
}

function navIcon(tile, key, px) {
  if (key === 'paint') return `<img src="${url(path.join(REPO, 'art', 'nav_horizon', 'icons', `${tile}_paint.png`))}" alt="" width="${px}" height="${px}" style="flex:none">`;
  if (key === 'chip') return box(px, chip025(tile, COLORS[tile]));
  if (key === 'glyph') return box(px, svg025(tile, COLORS[tile]));
  return box(px, chip(SUBJ[tile], COLORS[tile]));
}
function navShot(tile, name, key) {
  const panel = (theme) => {
    const t = (cls) => `<button type="button" class="nav-tile${cls}">${navIcon(tile, key, 18)}<span>${name}</span></button>`;
    return `<div class="theme theme-${theme}"><div class="panel nav-mock"><div class="big">${navIcon(tile, key, 96)}</div>
      <nav class="nav-grid compact" style="width:253px">${t('')}${t(' active')}</nav>
      <div class="muted small">compact, 18 px; at 34 px the painted icon stays</div></div></div>`;
  };
  return `<div class="pair" style="grid-template-columns:auto auto">${panel('light')}${panel('dark')}</div>`;
}

function horizonPanel(file, theme, width) {
  const row = (n, v) => `<div class="res-row"><span>${n}</span><span class="num">${v}</span></div>`;
  return `<div class="theme theme-${theme}"><aside class="col-left" style="width:${width}px;height:300px;box-sizing:border-box">
    <div class="res">${row('Energy', '42 / 100')}${row('Regolith', '18 / 50')}${row('Helium-3', '3.2 / 20')}${row('Titanium', '7 / 40')}</div>
    <img class="horizon" src="${url(file)}" alt="" style="object-fit:cover;object-position:50% 100%"></aside></div>`;
}

function familyShot(subjects) {
  const panel = (theme) => {
    const rows = subjects
      .map((s) => `<div class="fam-row"><span class="fam-name">${NAME[s] ?? s}</span>${variants(s).map((k) => `<span class="fam-cell">${box(16, chip(k))}${box(32, chip(k))}</span>`).join('')}</div>`)
      .join('');
    return `<div class="theme theme-${theme}"><div class="panel fam">${rows}</div></div>`;
  };
  return `<div class="pair" style="grid-template-columns:auto auto">${panel('light')}${panel('dark')}</div>`;
}

async function main() {
  let css = await readFile(path.join(REPO, 'web', 'src', 'style.css'), 'utf8');
  css = css.replace(":root,\n:root[data-theme='light'] {", ':root, .theme-light {').replace(":root[data-theme='dark'] {", '.theme-dark {');
  if (!css.includes('.theme-dark {') || !css.includes('.theme-light {')) throw new Error('mock: theme blocks not found in style.css');
  const fonts = ['rajdhani/600.css', 'rajdhani/700.css', 'ibm-plex-sans/400.css', 'ibm-plex-sans/400-italic.css', 'ibm-plex-sans/600.css', 'ibm-plex-mono/400.css']
    .map((f) => `<link rel="stylesheet" href="${url(path.join(REPO, 'web', 'node_modules', '@fontsource', f))}">`)
    .join('');
  const extra = `
    html, body { height: auto; overflow: visible; }
    body { background: #8a857c; padding: 0; margin: 0; width: max-content; }
    .pair { display: grid; }
    .theme { color: var(--text); background: var(--page); padding: 8px; }
    .theme-dark { color-scheme: dark; }
    .ic { display: inline-flex; flex: none; vertical-align: -2px; }
    .big { padding: 10px; display: flex; }
    .ctx { width: 250px; padding: 10px; font-size: 13px; }
    .ctx .card { margin: 8px 0; padding: 6px 8px; }
    .ctx .costs td.num { text-align: right; }
    .ev { display: flex; align-items: center; gap: 6px; font-size: 13px; padding-top: 4px; }
    .nav-mock { background: var(--panel); padding: 10px; display: grid; grid-template-columns: auto 253px; gap: 6px 14px; align-items: start; }
    .nav-mock .big { grid-row: span 2; width: 96px; height: 96px; padding: 0; }
    .nav-mock .nav-grid.compact { grid-template-columns: repeat(2, 1fr); }
    .small { font-size: 11px; }
    .res { font-size: 13px; padding-top: 6px; }
    .res-row { display: flex; justify-content: space-between; padding: 2px 0; }
    .fam { padding: 10px; font-size: 13px; }
    .fam-row { display: flex; align-items: center; gap: 14px; padding: 3px 0; }
    .fam-name { width: 100px; }
    .fam-cell { display: inline-flex; align-items: center; gap: 6px; width: 60px; }
  `;
  const page_ = (body) => `<!doctype html><html><head><meta charset="utf-8">${fonts}<style>${css}${extra}</style></head><body>${body}</body></html>`;

  await mkdir(PICK, { recursive: true });
  await mkdir(OUT, { recursive: true });
  const browser = await chromium.launch({ channel: 'chrome' });
  const pg = await browser.newPage({ viewport: { width: 1400, height: 400 }, deviceScaleFactor: 2 });
  async function shot(name, body) {
    const html = path.join(OUT, `${name}.html`);
    await writeFile(html, page_(body), { encoding: 'utf-8' });
    await pg.goto(url(html), { waitUntil: 'load' });
    await pg.evaluate(() => document.fonts.ready);
    const fallback = await pg.evaluate(() => [...document.fonts].filter((f) => f.status === 'error').map((f) => f.family));
    if (fallback.length) throw new Error(`mock: fonts failed: ${fallback.join(', ')}`);
    const broken = await pg.evaluate(() => [...document.images].filter((i) => !i.complete || i.naturalWidth === 0).length);
    if (broken) throw new Error(`mock: ${name}: an image did not load`);
    await pg.locator('.pair').screenshot({ path: path.join(PICK, `${name}.png`) });
  }
  const only = process.argv.slice(2);
  const want = (id) => !only.length || only.includes(id);
  const subjects = [...resources.map((r) => ['resource', r.id]), ...AXES.map((a) => ['axis', a.id]), ...rivals.map((r) => ['rival', r.id])];
  let n = 0;
  for (const [kind, id] of subjects) {
    if (!want(id)) continue;
    if (!variants(id).length) throw new Error(`mock: no glyphs for ${id}`);
    for (const key of ['current', ...variants(id)]) {
      await shot(`${id}_${key === 'current' ? key : key.split('.')[1]}`, subjectShot(kind, id, key));
      n++;
    }
  }
  for (const { id, name } of nav) {
    if (!want(id) && !want('nav')) continue;
    for (const key of ['paint', 'chip', SUBJ[id] ? 'subj' : 'glyph']) {
      await shot(`nav_${id}_${key}`, navShot(id, name, key));
      n++;
    }
  }
  if (want('horizon')) {
    for (const s of ['bare', 'pad', 'liftoff', 'climb', 'hop']) {
      const f = path.join(REPO, 'art', 'small_icons', 'horizon', `${s}.png`);
      if (!existsSync(f)) throw new Error(`mock: ${f} missing: run process.py`);
      await shot(`horizon_${s}`, `<div class="pair" style="grid-template-columns:repeat(4,auto)">${horizonPanel(f, 'light', 270)}${horizonPanel(f, 'light', 346)}${horizonPanel(f, 'dark', 270)}${horizonPanel(f, 'dark', 346)}</div>`);
      n++;
    }
  }
  if (want('family')) {
    await shot('family_resources', familyShot(resources.map((r) => r.id)));
    await shot('family_ideologies_rivals', familyShot([...AXES.map((a) => a.id), ...rivals.map((r) => r.id)]));
    n += 2;
  }
  await browser.close();
  console.log(`mock: ${n} images`);
}

await main();
