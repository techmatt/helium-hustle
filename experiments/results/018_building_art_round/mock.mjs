// `node mock.mjs`: building cards mocked with the round's art, light and dark side by side,
// one sheet per style plus the placement and framing comparisons, screenshotted into the
// picker folder art/pickers/building_art_r1/. The cards are the Buildings panel's markup
// (web/src/ui/Buildings.tsx: head, description, flows, costs, land, actions) under the real
// web/src/style.css and fonts, with the four buildings' content numbers at zero owned; the
// only new markup is the art box. Needs playwright-core (PW_DIR, a folder holding its
// node_modules) and system Chrome; `npm ci` in web/ first for the fonts.
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..');
const R1 = path.join(REPO, 'art', 'buildings', 'r1');
const PICK = path.join(REPO, 'art', 'pickers', 'building_art_r1');
const PW_DIR = process.env.PW_DIR ?? path.join(REPO, 'out', '018_building_art_round', 'tools');
const { chromium } = createRequire(path.join(PW_DIR, 'package.json'))('playwright-core');
const P = JSON.parse(await readFile(path.join(HERE, 'prompts.json'), 'utf8'));
const url = (f) => pathToFileURL(f).href;

// The panel's colors (common.tsx RESOURCE_COLORS) and names (content/resources.json).
const COLOR = { energy: '#f5d000', regolith: '#8b5a2b', titanium: '#c4c4c4', circuits: '#35c4a4', credits: '#2ecc40', land: '#4caf50', processors: '#d63ad6' };
const NAME = { energy: 'Energy', regolith: 'Regolith', titanium: 'Titanium', circuits: 'Circuit Boards', credits: 'Credits', land: 'Land', processors: 'Processors' };
const buildings = JSON.parse(await readFile(path.join(REPO, 'content', 'buildings.json'), 'utf8'));
// Which mock cards show as affordable (green) and which short, for a realistic panel.
const SHORT = { solar_panel: [], excavator: [], smelter: ['regolith'], data_center: ['circuits'] };

const sq = (id) => `<span class="sq" style="background:${COLOR[id]};width:12px;height:12px"></span>`;
const flows = (amts, sign, suffix) =>
  Object.entries(amts)
    .map(([r, a]) => `<div class="flow ${sign < 0 ? 'neg' : 'pos'}">${sq(r)} ${sign < 0 ? '−' : '+'}${a} ${NAME[r]}${suffix}</div>`)
    .join('');

/** One building card; `art` is { place: 'banner'|'thumb', src, cut, plate } or null. */
function card(id, art) {
  const b = buildings.find((x) => x.id === id);
  const short = SHORT[id];
  const grants = Object.fromEntries(b.effects.filter((e) => e.type === 'grant').map((e) => [e.resource, e.amount]));
  const box = (cls) => (art ? `<div class="art ${cls}${art.cut ? ' cut' : ''}${art.plate ? ' plate' : ''}"><img src="${url(art.src)}"></div>` : '');
  const head = `<div class="card-head"><h3>${b.name}</h3><span class="owned">(0/0)</span><button type="button" class="tiny" disabled>−</button><button type="button" class="tiny" disabled>+</button></div>`;
  const desc = `<p class="desc">${b.description}</p>`;
  const effects = `<div class="effects">${flows(b.produces, 1, '/day')}${flows(b.upkeep, -1, '/day')}${flows(grants, 1, '')}</div>`;
  // The thumbnail sits under the title row, beside the description and flows, so the
  // title keeps its full width.
  const top = art?.place === 'thumb' ? `${head}<div class="head-row">${box('thumb')}<div class="head-col">${desc}${effects}</div></div>` : `${box('banner')}${head}${desc}${effects}`;
  const costs = Object.entries(b.cost)
    .map(([r, a]) => `<tr class="${short.includes(r) ? 'short' : ''}"><td>${sq(r)} ${NAME[r]}</td><td class="num">${a}</td></tr>`)
    .join('');
  return `<div class="card building ${short.length ? 'cannot-buy' : 'can-buy'}">${top}
    <hr><table class="costs"><tbody>${costs}</tbody></table>
    <table class="costs"><tbody><tr><td>${sq('land')} Land</td><td class="num">${b.land}</td></tr></tbody></table>
    <div class="card-actions"><span class="buy-hint muted"></span></div></div>`;
}

const IDS = Object.keys(P.buildings);
const artFor = (style, b, place, plate) => {
  const cut = P.styles[style].cut;
  const src = path.join(R1, style, `${b}_${cut ? 'cut' : place}.png`);
  return existsSync(src) ? { place, src, cut, plate } : null;
};

/** A light and a dark panel side by side, each the CORE fold with four cards. */
function pair(label, style, place, plate) {
  const panel = (theme) =>
    `<div class="theme theme-${theme}"><div class="panel"><section class="fold fold-dark"><button type="button" class="fold-head">▼ ${theme.toUpperCase()}</button><div class="fold-body"><div class="cards">${IDS.map((b) => card(b, place ? artFor(style, b, place, plate) : null)).join('')}</div></div></section></div></div>`;
  return `<h4 class="sheet-label">${label}</h4><div class="pair">${panel('light')}${panel('dark')}</div>`;
}

function originals(style) {
  const imgs = IDS.map((b) => path.join(R1, 'raw', `${style}-${b}.png`)).filter(existsSync);
  return `<h4 class="sheet-label">The draws, uncropped</h4><div class="raws ${P.styles[style].cut ? 'cut' : ''}">${imgs.map((f) => `<img src="${url(f)}">`).join('')}</div>`;
}

const PLACES = { banner: 'Banner across the top (3:1)', thumb: 'Thumbnail beside the title (96 px)' };

async function main() {
  let css = await readFile(path.join(REPO, 'web', 'src', 'style.css'), 'utf8');
  // Scope the two token blocks to a class, so both themes sit on one page.
  css = css.replace(":root,\n:root[data-theme='light'] {", ':root, .theme-light {').replace(":root[data-theme='dark'] {", '.theme-dark {');
  if (!css.includes('.theme-dark {') || !css.includes('.theme-light {')) throw new Error('mock: theme blocks not found in style.css');
  const fonts = ['rajdhani/600.css', 'rajdhani/700.css', 'ibm-plex-sans/400.css', 'ibm-plex-sans/400-italic.css', 'ibm-plex-sans/600.css', 'ibm-plex-mono/400.css']
    .map((f) => `<link rel="stylesheet" href="${url(path.join(REPO, 'web', 'node_modules', '@fontsource', f))}">`)
    .join('');
  const extra = `
    html, body { height: auto; overflow: visible; }
    body { background: #8a857c; padding: 12px; width: 1320px; }
    .sheet-title { font-family: var(--font-head); font-size: 26px; color: #fff; margin: 0 0 2px; }
    .sheet-note { color: #f3f1ec; margin: 0 0 6px; font-size: 14px; }
    .sheet-label { font-family: var(--font-head); font-size: 18px; color: #fff; margin: 12px 0 4px; }
    .pair { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
    .theme { color: var(--text); background: var(--page); color-scheme: inherit; padding: 8px; border-radius: 4px; }
    .theme-dark { color-scheme: dark; }
    .theme .panel { background: var(--panel); padding: 8px; }
    .theme .cards { grid-template-columns: repeat(2, 1fr); }
    .raws { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; }
    .raws img { width: 100%; display: block; border-radius: 3px; }
    .raws.cut img { background: repeating-conic-gradient(#d6d3cc 0 25%, #f3f1ec 0 50%) 0 0 / 16px 16px; }
    /* The art box: the only markup the mock adds to a card. */
    .card .art { overflow: hidden; flex: none; }
    .card .art img { display: block; width: 100%; height: 100%; object-fit: cover; }
    .card .art.cut img { object-fit: contain; }
    .card .art.plate { background: linear-gradient(to bottom, var(--sky) 0 56%, var(--hz-far) 56% 72%, var(--hz-near) 72%); }
    .card .art.banner { margin: -8px -10px 2px; aspect-ratio: 3 / 1; border-radius: 3px 3px 0 0; }
    .card .art.banner.cut img { padding: 6px 0 2px; }
    .head-row { display: flex; gap: 10px; align-items: flex-start; }
    .head-col { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px; }
    .card .art.thumb { width: 96px; height: 96px; border-radius: 3px; }
    .card .art.thumb.cut.plate img { padding: 6px; }
  `;
  const page_ = (body) =>
    `<!doctype html><html><head><meta charset="utf-8">${fonts}<style>${css}${extra}</style></head><body>${body}</body></html>`;

  const sheets = {};
  for (const [s, st] of Object.entries(P.styles)) {
    const kind = st.cut ? 'cut-out on transparency' : 'framed scene';
    let body = `<h1 class="sheet-title">${st.label}</h1><p class="sheet-note">${kind}; Earthrise light left, dark right.</p>`;
    body += originals(s);
    for (const [place, label] of Object.entries(PLACES)) {
      body += pair(st.cut ? `${label}, on the card color` : label, s, place, false);
      if (st.cut) body += pair(`${label}, on a lunar plate in theme colors`, s, place, true);
    }
    sheets[`style_${s}`] = body;
  }
  // Placement: one framed and one cut-out style in each, so the choice isn't a style's.
  for (const [place, label] of Object.entries(PLACES))
    sheets[`placement_${place}`] = `<h1 class="sheet-title">Placement: ${label}</h1><p class="sheet-note">Shown with Painterly (framed) and Isometric (cut-out on the card color).</p>`
      + pair(`${label}, Painterly`, 'painterly', place) + pair(`${label}, Isometric`, 'iso', place, false);
  // Framing: the same placements, three ways to sit the art on the card.
  const framing = {
    framed: ['Framed scene', 'Carries its own lunar backdrop, so it looks the same in both themes. Painterly.', 'painterly', false],
    cut: ['Cut-out on the card color', 'The building on its patch of ground, straight on the card. Isometric.', 'iso', false],
    plate: ['Cut-out on a lunar plate', 'The cut-out over a horizon drawn in the theme\'s own colors (style.css --sky, --hz-far, --hz-near), so the plate follows the theme. Isometric.', 'iso', true],
  };
  for (const [k, [title, note, style, plate]] of Object.entries(framing))
    sheets[`framing_${k}`] = `<h1 class="sheet-title">Framing: ${title}</h1><p class="sheet-note">${note}</p>`
      + pair(`${title}, banner`, style, 'banner', plate) + pair(`${title}, thumbnail`, style, 'thumb', plate);
  sheets.no_art = `<h1 class="sheet-title">Today: no art</h1><p class="sheet-note">The same cards as they are now, for comparison.</p>` + pair('Current cards', null, null);

  await mkdir(PICK, { recursive: true });
  const browser = await chromium.launch({ channel: 'chrome' });
  const pg = await browser.newPage({ viewport: { width: 1344, height: 900 }, deviceScaleFactor: 1 });
  const only = process.argv.slice(2);
  for (const [name, body] of Object.entries(sheets)) {
    if (only.length && !only.includes(name)) continue;
    const html = path.join(REPO, 'out', '018_building_art_round', `${name}.html`);
    await writeFile(html, page_(body), { encoding: 'utf-8' });
    await pg.goto(url(html), { waitUntil: 'load' });
    await pg.evaluate(() => document.fonts.ready);
    const fallback = await pg.evaluate(() => [...document.fonts].filter((f) => f.status === 'error').map((f) => f.family));
    if (fallback.length) throw new Error(`mock: fonts failed: ${fallback.join(', ')}`);
    await pg.screenshot({ path: path.join(PICK, `${name}.png`), fullPage: true });
    console.log(`mock: ${name}.png`);
  }
  await browser.close();
}

await main();
