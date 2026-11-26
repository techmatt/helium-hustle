// `node experiments/results/100_wire_project_art_and_research_glyphs/098_research_glyphs_r1/mock.mjs`
// from the repo root (also as `art/research_glyphs/mock.mjs`, the shared copy): picker research_glyphs_r1's images
// and manifest, into art/pickers/research_glyphs_r1/. Each candidate: the chip enlarged
// (64 px), then a research card in the light and dark themes with the chip at 22 px before
// its title (the card head's h3 is 18 px), under the real web/src/style.css and fonts (026's
// mock: both theme token blocks scoped to a class so both sit on one page). `none` is
// today's card, no icon. The `color` row shows four cards per chip-color scheme.
// Needs playwright-core and @fontsource/{rajdhani,ibm-plex-sans,ibm-plex-mono} in PW_DIR
// (default out/098_research_glyphs_r1/tools/) and system Chrome.
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { CATEGORY_COLORS, LABELS, RESEARCH_COLOR, chip, glyph, variants } from './glyphs.mjs';

const REPO = process.cwd();
const PICK = path.join(REPO, 'art', 'pickers', 'research_glyphs_r1');
const OUT = path.join(REPO, 'out', '098_research_glyphs_r1');
const PW_DIR = process.env.PW_DIR ?? path.join(OUT, 'tools');
const { chromium } = createRequire(path.join(PW_DIR, 'package.json'))('playwright-core');
const url = (f) => pathToFileURL(f).href;

const research = JSON.parse(await readFile(path.join(REPO, 'content', 'research.json'), 'utf8'));
const scienceSvg = await readFile(path.join(REPO, 'web', 'src', 'assets', 'chips', 'science.svg'), 'utf8');
const sciGlyph = scienceSvg.slice(scienceSvg.indexOf('>') + 1, scienceSvg.lastIndexOf('</svg>'));
// The Science chip as icons.tsx draws it (#b388ff takes the dark ink).
const sci = `<svg class="chip" viewBox="0 0 24 24" width="14" height="14" style="color:#1c1c21"><rect x="0.5" y="0.5" width="23" height="23" rx="3" fill="#b388ff" stroke="rgb(0 0 0 / 18%)" stroke-width="1"/><g fill="#1c1c21" transform="translate(3.6 3.6) scale(0.7)">${sciGlyph}</g></svg>`;

const box = (px, inner) => `<span class="ic" style="width:${px}px;height:${px}px">${inner}</span>`;
/** The icon for a candidate: a glyph key in a scheme (`research`, `category`, `science`, `bare`). */
function icon(r, key, px, scheme = 'research') {
  if (key === 'none') return '';
  if (scheme === 'bare') return box(px, glyph(key, 'var(--accent)'));
  const color = scheme === 'category' ? CATEGORY_COLORS[r.category] : scheme === 'science' ? '#b388ff' : RESEARCH_COLOR;
  return box(px, chip(key, color));
}
const card = (r, key, scheme) => `<div class="card research can-buy"><div class="card-head">${icon(r, key, 22, scheme)}<h3>${r.name}</h3></div>
  <p class="desc">${r.description}</p><div class="card-actions"><span>${sci} ${r.cost} Science</span><span class="buy-hint muted"></span></div></div>`;
const theme = (t, inner) => `<div class="theme theme-${t}"><div class="panel ctx">${inner}</div></div>`;

function itemShot(r, key) {
  const big = key === 'none' ? '<div class="big none">no icon</div>' : `<div class="big">${box(64, chip(key))}</div>`;
  return `<div class="pair">${theme('light', big)}${theme('light', card(r, key))}${theme('dark', card(r, key))}</div>`;
}
const SAMPLE = [['basic_blueprints', 'sheet'], ['regolith_processing', 'pick'], ['dream_protocols', 'cloud'], ['market_awareness', 'trend']];
function colorShot(scheme) {
  const cards = SAMPLE.map(([id, v]) => card(research.find((r) => r.id === id), `${id}.${v}`, scheme)).join('');
  return `<div class="pair colors">${theme('light', `<div class="cards2">${cards}</div>`)}${theme('dark', `<div class="cards2">${cards}</div>`)}</div>`;
}

let css = await readFile(path.join(REPO, 'web', 'src', 'style.css'), 'utf8');
css = css.replace(":root,\n:root[data-theme='light'] {", ':root, .theme-light {').replace(":root[data-theme='dark'] {", '.theme-dark {');
if (!css.includes('.theme-dark {') || !css.includes('.theme-light {')) throw new Error('mock: theme blocks not found in style.css');
const fonts = ['rajdhani/600.css', 'rajdhani/700.css', 'ibm-plex-sans/400.css', 'ibm-plex-sans/400-italic.css', 'ibm-plex-sans/600.css', 'ibm-plex-mono/400.css']
  .map((f) => `<link rel="stylesheet" href="${url(path.join(PW_DIR, 'node_modules', '@fontsource', f))}">`)
  .join('');
const extra = `
  html, body { height: auto; overflow: visible; }
  body { background: #8a857c; padding: 0; margin: 0; width: max-content; }
  .pair { display: grid; grid-template-columns: auto auto auto; }
  .pair.colors { grid-template-columns: auto; }
  .theme { color: var(--text); background: var(--page); padding: 8px; }
  .theme-dark { color-scheme: dark; }
  .ic { display: inline-flex; flex: none; }
  .ctx { padding: 10px; font-size: 14px; }
  .ctx > .card { width: 300px; box-sizing: border-box; }
  .cards2 { display: grid; grid-template-columns: repeat(4, 300px); gap: 8px; }
  .big { width: 64px; height: 64px; display: flex; align-items: center; justify-content: center; }
  .big.none { border: 1px dashed var(--line); color: var(--muted); font-size: 12px; box-sizing: border-box; }
`;
const page = (body) => `<!doctype html><html><head><meta charset="utf-8">${fonts}<style>${css}${extra}</style></head><body>${body}</body></html>`;

await mkdir(PICK, { recursive: true });
await mkdir(OUT, { recursive: true });
const browser = await chromium.launch({ channel: 'chrome' });
const pg = await browser.newPage({ viewport: { width: 1400, height: 400 }, deviceScaleFactor: 2 });
async function shot(name, body) {
  const html = path.join(OUT, `${name}.html`);
  await writeFile(html, page(body), { encoding: 'utf-8' });
  await pg.goto(url(html), { waitUntil: 'load' });
  await pg.evaluate(() => document.fonts.ready);
  const failed = await pg.evaluate(() => [...document.fonts].filter((f) => f.status === 'error').map((f) => f.family));
  if (failed.length) throw new Error(`mock: fonts failed: ${failed.join(', ')}`);
  await pg.locator('.pair').screenshot({ path: path.join(PICK, `${name}.png`) });
  return `${name}.png`;
}

const rows = [];
const schemes = [
  ['research', 'all research purple (the Research nav tile color): the rows below use this'],
  ['category', 'one hue per research category (proposed hues)'],
  ['science', 'the Science chip lavender, dark ink'],
  ['bare', 'no square: the glyph alone in the amber accent'],
];
const colorOpts = [];
for (const [s, label] of schemes) colorOpts.push({ key: s, image: await shot(`color_${s}`, colorShot(s)), label });
rows.push({ id: 'color', label: 'Chip color (all research icons)', note: 'Four sample glyphs per scheme. Pick one.', options: colorOpts });
for (const r of research) {
  const keys = variants(r.id);
  if (!keys.length) throw new Error(`mock: no glyphs for ${r.id}`);
  const options = [{ key: 'none', image: await shot(`${r.id}_none`, itemShot(r, 'none')), label: "today's card, no icon" }];
  for (const k of keys) options.push({ key: k.split('.')[1], image: await shot(`${r.id}_${k.split('.')[1]}`, itemShot(r, k)), label: LABELS[k] });
  rows.push({ id: r.id, label: r.name, note: `${r.category}. ${r.description}`, options });
}
await browser.close();
const manifest = {
  name: 'research_glyphs_r1',
  title: 'Research card icons',
  intro:
    "One small icon per research item, in the resource chips' style (a colored square, the glyph knocked out), at 22 px before the card's title, light and dark, with the chip enlarged at the left. Pick one option per row; `none` is today's card. The first row picks the chip color for the whole set; the item rows show research purple.",
  rows,
};
await writeFile(path.join(PICK, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n', 'utf-8');
console.log(`mock: ${rows.reduce((n, r) => n + r.options.length, 0)} images in ${rows.length} rows`);
