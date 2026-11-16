// `node mock.mjs`: the picker images for nav_horizon_art_r1, screenshotted at 2x device
// pixels into art/pickers/nav_horizon_art_r1/ under the real web/src/style.css and fonts
// (020's mock: the two theme token blocks scoped to a class so both sit on one page).
//
// - `nav_<tile>_<key>.png`: one icon candidate: enlarged (96 px) on each theme's panel,
//   then on the nav tile itself in light and dark, as it renders: the tile at the 1280 px
//   column's width (82 px: 253 px inside its padding and border, three columns, 4 px gaps), normal and selected with the 34 px icon, and compact (past
//   nine tiles) with the 18 px icon. Keys: `current` (the colored square), `crop` (card
//   art crop), `paint` (painterly icon), `glyph` (flat glyph in the tile color), `chip`
//   (the colored square with the glyph knocked out).
// - `horizon_<key>.png`: the foot of the left column with the strip, at the 1280 px
//   column (270 px) and the 1920 px one (18vw, 346 px), light and dark.
//
// Needs playwright-core (PW_DIR, default out/025_nav_horizon_art_r1/tools/), system Chrome,
// `npm ci` in web/, and process.py's output in art/nav_horizon/.
import { existsSync } from 'node:fs';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { chip, svg } from './glyphs.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..');
const ART = path.join(REPO, 'art', 'nav_horizon');
const PICK = path.join(REPO, 'art', 'pickers', 'nav_horizon_art_r1');
const OUT = path.join(REPO, 'out', '025_nav_horizon_art_r1');
const PW_DIR = process.env.PW_DIR ?? path.join(OUT, 'tools');
const { chromium } = createRequire(path.join(PW_DIR, 'package.json'))('playwright-core');
const url = (f) => pathToFileURL(f).href;

const common = await readFile(path.join(REPO, 'web', 'src', 'ui', 'common.tsx'), 'utf8');
const navBlock = common.slice(common.indexOf('const NAV_COLORS'));
const COLOR = Object.fromEntries([...navBlock.slice(0, navBlock.indexOf('};')).matchAll(/^\s+(\w+): '(#[0-9a-f]{6})',$/gm)].map((m) => [m[1], m[2]]));
const nav = JSON.parse(await readFile(path.join(REPO, 'content', 'nav.json'), 'utf8'));
// The current horizon: art.tsx's <Horizon> markup as it stands (plain SVG, no expressions).
const artTsx = await readFile(path.join(REPO, 'web', 'src', 'ui', 'art.tsx'), 'utf8');
const CURRENT_HORIZON = artTsx.match(/<svg class="horizon"[\s\S]*?<\/svg>/)[0];

/** The icon element for a candidate at `px` CSS pixels. */
function icon(tile, key, px) {
  if (key === 'current') return `<span class="sq" style="background:${COLOR[tile]};width:${px}px;height:${px}px"></span>`;
  if (key === 'glyph') return `<span style="display:inline-flex;flex:none;width:${px}px;height:${px}px">${svg(tile, COLOR[tile])}</span>`;
  if (key === 'chip') return `<span style="display:inline-flex;flex:none;width:${px}px;height:${px}px">${chip(tile, COLOR[tile])}</span>`;
  const f = path.join(ART, 'icons', `${tile}_${key}.png`);
  if (!existsSync(f)) throw new Error(`mock: ${f} missing: run process.py`);
  return `<img src="${url(f)}" alt="" width="${px}" height="${px}" style="flex:none">`;
}

function navPanel(tile, name, key, theme) {
  const t = (cls, px) => `<button type="button" class="nav-tile${cls}">${icon(tile, key, px)}<span>${name}</span></button>`;
  return `<div class="theme theme-${theme}"><div class="panel nav-mock">
    <div class="big">${icon(tile, key, 96)}</div>
    <nav class="nav-grid" style="width:253px">${t('', 34)}${t(' active', 34)}</nav>
    <nav class="nav-grid compact" style="width:253px;margin-top:6px">${t('', 18)}${t(' active', 18)}</nav>
  </div></div>`;
}

function horizonPanel(key, theme, width) {
  const strip =
    key === 'current'
      ? CURRENT_HORIZON
      : `<img class="horizon" src="${url(path.join(ART, 'horizon', `${key}.png`))}" alt="" style="object-fit:cover;object-position:50% 100%">`;
  const row = (n, v) => `<div class="res-row"><span>${n}</span><span class="num">${v}</span></div>`;
  return `<div class="theme theme-${theme}"><aside class="col-left" style="width:${width}px;height:300px;box-sizing:border-box">
    <div class="res">${row('Energy', '42 / 100')}${row('Regolith', '18 / 50')}${row('Helium-3', '3.2 / 20')}${row('Titanium', '7 / 40')}</div>
    ${strip}</aside></div>`;
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
    .pair { display: grid; grid-template-columns: auto auto; }
    .theme { color: var(--text); background: var(--page); padding: 8px; }
    .theme-dark { color-scheme: dark; }
    .nav-mock { background: var(--panel); padding: 10px; display: grid; grid-template-columns: auto 253px; grid-template-rows: auto auto; gap: 6px 14px; align-items: start; }
    .nav-mock .big { grid-row: span 2; width: 96px; height: 96px; display: flex; }
    .nav-mock .nav-grid { grid-template-columns: repeat(3, 1fr); }
    .nav-mock .nav-grid.compact { grid-template-columns: repeat(2, 1fr); }
    .res { font-size: 13px; padding-top: 6px; }
    .res-row { display: flex; justify-content: space-between; padding: 2px 0; }
  `;
  const page_ = (body) => `<!doctype html><html><head><meta charset="utf-8">${fonts}<style>${css}${extra}</style></head><body>${body}</body></html>`;

  await mkdir(PICK, { recursive: true });
  const browser = await chromium.launch({ channel: 'chrome' });
  const pg = await browser.newPage({ viewport: { width: 900, height: 400 }, deviceScaleFactor: 2 });
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
    console.log(`mock: ${name}.png`);
  }
  const keys = (tile) => ['current', ...['crop', 'paint'].filter((k) => existsSync(path.join(ART, 'icons', `${tile}_${k}.png`))), 'glyph', 'chip'];
  const only = process.argv.slice(2);
  for (const { id, name } of nav) {
    if (only.length && !only.includes(id)) continue;
    for (const k of keys(id)) await shot(`nav_${id}_${k}`, `<div class="pair" style="grid-template-columns:auto">${navPanel(id, name, k, 'light')}${navPanel(id, name, k, 'dark')}</div>`);
  }
  if (!only.length || only.includes('horizon')) {
    const hz = ['current', ...['horizon-cutout', 'horizon-earthrise', 'horizon-earthlight', 'progress-pad-cutout', 'progress-liftoff-cutout'].filter((k) => existsSync(path.join(ART, 'horizon', `${k}.png`)))];
    for (const k of hz) {
      const body = `<div class="pair">${horizonPanel(k, 'light', 270)}${horizonPanel(k, 'light', 346)}${horizonPanel(k, 'dark', 270)}${horizonPanel(k, 'dark', 346)}</div>`;
      await shot(`horizon_${k}`, body);
    }
  }
  await browser.close();
}

await main();
