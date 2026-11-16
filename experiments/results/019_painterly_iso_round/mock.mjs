// `node mock.mjs [sheet …]`: round 2's card strips, a couple of cards each, the art as the
// 96 px thumbnail under the title (Matt's r1 placement), cut out on the card color (his r1
// framing), screenshotted into art/pickers/building_art_r2/ at 2x device pixels so the art
// can be judged. The cards are 018's mock (Buildings.tsx's markup under the real
// web/src/style.css and fonts, the theme token blocks scoped to a class); the only added
// markup is the art box. Light shows Solar Panel and Smelter, dark Excavator and Data
// Center, so each building appears on a card once per variant. Needs playwright-core
// (PW_DIR, default out/019_painterly_iso_round/tools/) and system Chrome; `npm ci` in web/.
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..');
const R2 = path.join(REPO, 'art', 'buildings', 'r2');
const PICK = path.join(REPO, 'art', 'pickers', 'building_art_r2');
const OUT = path.join(REPO, 'out', '019_painterly_iso_round');
const PW_DIR = process.env.PW_DIR ?? path.join(OUT, 'tools');
const { chromium } = createRequire(path.join(PW_DIR, 'package.json'))('playwright-core');
const P = JSON.parse(await readFile(path.join(HERE, 'prompts.json'), 'utf8'));
const url = (f) => pathToFileURL(f).href;

// The panel's colors (common.tsx RESOURCE_COLORS) and names (content/resources.json).
const COLOR = { energy: '#f5d000', regolith: '#8b5a2b', titanium: '#c4c4c4', circuits: '#35c4a4', credits: '#2ecc40', land: '#4caf50', processors: '#d63ad6' };
const NAME = { energy: 'Energy', regolith: 'Regolith', titanium: 'Titanium', circuits: 'Circuit Boards', credits: 'Credits', land: 'Land', processors: 'Processors' };
const buildings = JSON.parse(await readFile(path.join(REPO, 'content', 'buildings.json'), 'utf8'));
const SHORT = { solar_panel: [], excavator: [], smelter: ['regolith'], data_center: ['circuits'] };
const STRIPS = { light: ['solar_panel', 'smelter'], dark: ['excavator', 'data_center'] };

const sq = (id) => `<span class="sq" style="background:${COLOR[id]};width:12px;height:12px"></span>`;
const flows = (amts, sign, suffix) =>
  Object.entries(amts)
    .map(([r, a]) => `<div class="flow ${sign < 0 ? 'neg' : 'pos'}">${sq(r)} ${sign < 0 ? '−' : '+'}${a} ${NAME[r]}${suffix}</div>`)
    .join('');

/** One building card with its art (a cut-out path) as the 96 px thumbnail. */
function card(id, src) {
  const b = buildings.find((x) => x.id === id);
  const short = SHORT[id];
  const grants = Object.fromEntries(b.effects.filter((e) => e.type === 'grant').map((e) => [e.resource, e.amount]));
  const head = `<div class="card-head"><h3>${b.name}</h3><span class="owned">(0/0)</span><button type="button" class="tiny" disabled>−</button><button type="button" class="tiny" disabled>+</button></div>`;
  const art = src ? `<div class="art thumb"><img src="${url(src)}"></div>` : '';
  const top = `${head}<div class="head-row">${art}<div class="head-col"><p class="desc">${b.description}</p><div class="effects">${flows(b.produces, 1, '/day')}${flows(b.upkeep, -1, '/day')}${flows(grants, 1, '')}</div></div></div>`;
  const costs = Object.entries(b.cost)
    .map(([r, a]) => `<tr class="${short.includes(r) ? 'short' : ''}"><td>${sq(r)} ${NAME[r]}</td><td class="num">${a}</td></tr>`)
    .join('');
  return `<div class="card building ${short.length ? 'cannot-buy' : 'can-buy'}">${top}
    <hr><table class="costs"><tbody>${costs}</tbody></table>
    <table class="costs"><tbody><tr><td>${sq('land')} Land</td><td class="num">${b.land}</td></tr></tbody></table>
    <div class="card-actions"><span class="buy-hint muted"></span></div></div>`;
}

const strip = (theme, ids, srcOf) =>
  `<div class="theme theme-${theme}"><div class="panel"><div class="cards">${ids.map((b) => card(b, srcOf(b))).join('')}</div></div></div>`;

async function main() {
  let css = await readFile(path.join(REPO, 'web', 'src', 'style.css'), 'utf8');
  css = css.replace(":root,\n:root[data-theme='light'] {", ':root, .theme-light {').replace(":root[data-theme='dark'] {", '.theme-dark {');
  if (!css.includes('.theme-dark {') || !css.includes('.theme-light {')) throw new Error('mock: theme blocks not found in style.css');
  const fonts = ['rajdhani/600.css', 'rajdhani/700.css', 'ibm-plex-sans/400.css', 'ibm-plex-sans/400-italic.css', 'ibm-plex-sans/600.css', 'ibm-plex-mono/400.css']
    .map((f) => `<link rel="stylesheet" href="${url(path.join(REPO, 'web', 'node_modules', '@fontsource', f))}">`)
    .join('');
  const extra = `
    html, body { height: auto; overflow: visible; }
    body { background: #8a857c; padding: 0; margin: 0; width: 680px; }
    .theme { color: var(--text); background: var(--page); padding: 10px; }
    .theme-dark { color-scheme: dark; }
    .theme .panel { background: var(--panel); padding: 8px; }
    .theme .cards { grid-template-columns: repeat(2, 1fr); }
    .card .art { overflow: hidden; flex: none; }
    .card .art img { display: block; width: 100%; height: 100%; object-fit: contain; }
    .head-row { display: flex; gap: 10px; align-items: flex-start; }
    .head-col { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px; }
    .card .art.thumb { width: 96px; height: 96px; }
  `;
  const page_ = (body) => `<!doctype html><html><head><meta charset="utf-8">${fonts}<style>${css}${extra}</style></head><body>${body}</body></html>`;

  const sheets = {};
  for (const v of Object.keys(P.variants))
    for (const [theme, ids] of Object.entries(STRIPS)) {
      const src = (b) => path.join(R2, v, `${b}_cut.png`);
      if (ids.every((b) => existsSync(src(b)))) sheets[`cards_${v}_${theme}`] = strip(theme, ids, src);
      const tight = (b) => path.join(R2, v, `${b}_tight.png`);
      if (ids.every((b) => existsSync(tight(b)))) sheets[`cards_${v}_tight_${theme}`] = strip(theme, ids, tight);
    }
  for (const [theme, ids] of Object.entries(STRIPS)) sheets[`cards_none_${theme}`] = strip(theme, ids, () => null);

  await mkdir(PICK, { recursive: true });
  const browser = await chromium.launch({ channel: 'chrome' });
  const pg = await browser.newPage({ viewport: { width: 680, height: 400 }, deviceScaleFactor: 2 });
  const only = process.argv.slice(2);
  for (const [name, body] of Object.entries(sheets)) {
    if (only.length && !only.includes(name)) continue;
    const html = path.join(OUT, `${name}.html`);
    await writeFile(html, page_(body), { encoding: 'utf-8' });
    await pg.goto(url(html), { waitUntil: 'load' });
    await pg.evaluate(() => document.fonts.ready);
    const fallback = await pg.evaluate(() => [...document.fonts].filter((f) => f.status === 'error').map((f) => f.family));
    if (fallback.length) throw new Error(`mock: fonts failed: ${fallback.join(', ')}`);
    await pg.locator('.theme').screenshot({ path: path.join(PICK, `${name}.png`) });
    console.log(`mock: ${name}.png`);
  }
  await browser.close();
}

await main();
