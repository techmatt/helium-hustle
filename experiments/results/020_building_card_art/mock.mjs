// `node mock.mjs [<building> …]`: one picker image per building, its card in Earthrise light
// and dark side by side, with the shipped picture (web/src/assets/buildings/<id>.webp) as
// the 96 px thumbnail, screenshotted at 2x device pixels into
// art/pickers/building_art_set_r1/card_<id>.png. The card is Buildings.tsx's markup under
// the real web/src/style.css and fonts (019's mock), the two theme token blocks scoped to a
// class so both sit on one page; it shows the card affordable, owned none. Needs
// playwright-core (PW_DIR, default out/020_building_card_art/tools/), system Chrome and
// `npm ci` in web/.
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..');
const PICK = path.join(REPO, 'art', 'pickers', 'building_art_set_r1');
const OUT = path.join(REPO, 'out', '020_building_card_art');
const PW_DIR = process.env.PW_DIR ?? path.join(OUT, 'tools');
const { chromium } = createRequire(path.join(PW_DIR, 'package.json'))('playwright-core');
const url = (f) => pathToFileURL(f).href;

// common.tsx RESOURCE_COLORS; names from content/resources.json.
const common = await readFile(path.join(REPO, 'web', 'src', 'ui', 'common.tsx'), 'utf8');
const COLOR = Object.fromEntries([...common.matchAll(/^\s+(\w+): '(#[0-9a-f]{6})',$/gm)].map((m) => [m[1], m[2]]));
const resources = JSON.parse(await readFile(path.join(REPO, 'content', 'resources.json'), 'utf8'));
const NAME = Object.fromEntries(resources.map((r) => [r.id, r.name]));
const buildings = JSON.parse(await readFile(path.join(REPO, 'content', 'buildings.json'), 'utf8'));

const sq = (id) => `<span class="sq" style="background:${COLOR[id] ?? '#999'};width:12px;height:12px"></span>`;
const flows = (amts, sign, suffix) =>
  Object.entries(amts)
    .map(([r, a]) => `<div class="flow ${sign < 0 ? 'neg' : 'pos'}">${sq(r)} ${sign < 0 ? '−' : '+'}${a} ${NAME[r]}${suffix}</div>`)
    .join('');

function card(b) {
  const of = (t) => Object.fromEntries(b.effects.filter((e) => e.type === t).map((e) => [e.resource, e.amount]));
  const stores = of('store');
  const head = `<div class="card-head"><h3>${b.name}</h3><span class="owned">(0/0)</span><button type="button" class="tiny" disabled>−</button><button type="button" class="tiny" disabled>+</button></div>`;
  const img = url(path.join(REPO, 'web', 'src', 'assets', 'buildings', `${b.id}.webp`));
  const body = `<div class="card-body"><img class="card-art" src="${img}" alt="" width="96" height="96"><div class="card-text"><p class="desc">${b.description}</p><div class="effects">${flows(b.produces, 1, '/day')}${flows(b.upkeep, -1, '/day')}${flows(stores, 1, ' storage')}${flows(of('grant'), 1, '')}</div></div></div>`;
  const costs = Object.entries(b.cost)
    .map(([r, a]) => `<tr><td>${sq(r)} ${NAME[r]}</td><td class="num">${a}</td></tr>`)
    .join('');
  return `<div class="card building can-buy">${head}${body}
    <hr><table class="costs"><tbody>${costs}</tbody></table>
    <table class="costs"><tbody><tr><td>${sq('land')} Land</td><td class="num">${b.land}</td></tr></tbody></table>
    <div class="card-actions"><span class="buy-hint muted"></span></div></div>`;
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
    body { background: #8a857c; padding: 0; margin: 0; width: 680px; }
    .pair { display: grid; grid-template-columns: 1fr 1fr; }
    .theme { color: var(--text); background: var(--page); padding: 10px; }
    .theme-dark { color-scheme: dark; }
    .theme .panel { background: var(--panel); padding: 8px; height: 100%; box-sizing: border-box; }
  `;
  const page_ = (body) => `<!doctype html><html><head><meta charset="utf-8">${fonts}<style>${css}${extra}</style></head><body>${body}</body></html>`;

  await mkdir(PICK, { recursive: true });
  const browser = await chromium.launch({ channel: 'chrome' });
  const pg = await browser.newPage({ viewport: { width: 680, height: 400 }, deviceScaleFactor: 2 });
  const only = process.argv.slice(2);
  for (const b of buildings) {
    if (only.length && !only.includes(b.id)) continue;
    const side = (theme) => `<div class="theme theme-${theme}"><div class="panel">${card(b)}</div></div>`;
    const html = path.join(OUT, `card_${b.id}.html`);
    await writeFile(html, page_(`<div class="pair">${side('light')}${side('dark')}</div>`), { encoding: 'utf-8' });
    await pg.goto(url(html), { waitUntil: 'load' });
    await pg.evaluate(() => document.fonts.ready);
    const fallback = await pg.evaluate(() => [...document.fonts].filter((f) => f.status === 'error').map((f) => f.family));
    if (fallback.length) throw new Error(`mock: fonts failed: ${fallback.join(', ')}`);
    const broken = await pg.evaluate(() => [...document.images].filter((i) => !i.complete || i.naturalWidth === 0).length);
    if (broken) throw new Error(`mock: ${b.id}: its picture did not load`);
    await pg.locator('.pair').screenshot({ path: path.join(PICK, `card_${b.id}.png`) });
    console.log(`mock: card_${b.id}.png`);
  }
  await browser.close();
}

await main();
