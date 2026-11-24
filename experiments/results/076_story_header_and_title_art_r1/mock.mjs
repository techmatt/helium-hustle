// `node mock.mjs`: the picker images for story_title_art_r1, screenshotted at 2x device pixels
// into art/pickers/story_title_art_r1/ under the real web/src/style.css and fonts (020's and
// 025's mock: the two theme token blocks scoped to a class so light and dark sit on one page).
//
// - `title_<key>.png`: first the art enlarged on the light panel (left) and the dark one
//   (right); under it the top of the left column, title then the first nav tiles, at the
//   1280 px window's column (`--left` 287 px) and the 1920 px one (363 px), light, light,
//   dark, dark, left to right. `current` is the plain h1 as it renders today; `lettered-*`
//   are CSS-only treatments on the theme tokens (LETTERED below); every other key is a
//   painted draw from art/story_title/title/ (process.py), shown at the h1's line height
//   (29 px), no wider than the column.
// - `act<N>_<framing>.png`: one act chapter header as a band at the top of a timeline in
//   the middle column: 595 px wide (the 1280 px window's column inside its padding) on the
//   light page, 1058 px (the 1920 px window's) on the dark one, the band 3:1 at both (a
//   proposal: the Story view's header size is the shipping item's call). The draw
//   (process.py's art/story_title/header/, letterbox trimmed) is `background-size: cover`
//   at its focus in crops.json, the act title laid on it where the framing leaves room
//   (`fill`, process.py's first-round vignettes, takes the title under the motif like
//   `vignette`). `none` is the act title with no art, as the row's current option.
//
// Needs playwright-core (PW_DIR, default out/076_story_header_and_title_art_r1/tools/),
// system Chrome, `npm ci` in web/, and process.py's output.
import { existsSync, readdirSync } from 'node:fs';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..');
const ART = path.join(REPO, 'art', 'story_title');
const PICK = path.join(REPO, 'art', 'pickers', 'story_title_art_r1');
const OUT = path.join(REPO, 'out', '076_story_header_and_title_art_r1');
const PW_DIR = process.env.PW_DIR ?? path.join(OUT, 'tools');
const { chromium } = createRequire(path.join(PW_DIR, 'package.json'))('playwright-core');
const url = (f) => pathToFileURL(f).href;
const P = JSON.parse(await readFile(path.join(HERE, 'prompts.json'), 'utf8'));
const CROPS = JSON.parse(await readFile(path.join(HERE, 'crops.json'), 'utf8'));

// The CSS-only titles: markup inside h1.title, styled from the theme tokens, so they swap
// with the theme like the rest of the UI.
export const LETTERED = {
  'lettered-split': {
    html: '<span class="lt-a">Helium</span> <span class="lt-b">Hustle</span>',
    css: `.lt-a { color: var(--text); } .lt-b { color: var(--accent); }
      h1.title.lettered-split { position: relative; }
      h1.title.lettered-split::after { content: ''; display: block; height: 2px; width: 40%; margin: 3px auto 0;
        background: linear-gradient(90deg, transparent, var(--accent), transparent); }`,
  },
  'lettered-orbit': {
    html: '<span class="lt-orbit"></span><span>Helium Hustle</span>',
    css: `h1.title.lettered-orbit { display: flex; align-items: center; justify-content: center; gap: 8px; letter-spacing: 0.14em; }
      .lt-orbit { flex: none; width: 18px; height: 18px; border-radius: 50%; position: relative;
        background: radial-gradient(circle at 35% 35%, #9fd0ff, #2a6fc0 60%, #12305c); }
      .lt-orbit::after { content: ''; position: absolute; inset: -4px -7px; border: 2px solid var(--accent);
        border-radius: 50%; transform: rotate(-20deg); border-left-color: transparent; }`,
  },
};

const NAV = [['buildings', 'Buildings'], ['commands', 'Commands'], ['research', 'Research']];
function navRow() {
  const tile = ([id, name]) => {
    const f = path.join(REPO, 'web', 'src', 'assets', 'nav', `${id}.webp`);
    const img = existsSync(f) ? `<img class="nav-icon" src="${url(f)}" alt="" width="34" height="34">` : '';
    return `<button type="button" class="nav-tile">${img}<span>${name}</span></button>`;
  };
  return `<nav class="nav-grid">${NAV.map(tile).join('')}</nav>`;
}

function titleInner(key) {
  if (key === 'current') return { cls: '', html: 'Helium Hustle' };
  if (LETTERED[key]) return { cls: ` ${key}`, html: LETTERED[key].html };
  const f = path.join(ART, 'title', `${key}.png`);
  if (!existsSync(f)) throw new Error(`mock: ${f} missing: run process.py`);
  return { cls: ' painted', html: `<img src="${url(f)}" alt="Helium Hustle">` };
}

function titleColumn(key, theme, width) {
  const t = titleInner(key);
  return `<div class="theme theme-${theme}"><aside class="col-left" style="width:${width}px;height:auto;box-sizing:border-box;padding-bottom:8px">
    <h1 class="title${t.cls}">${t.html}</h1>${navRow()}</aside></div>`;
}

function titleBig(key, theme) {
  const t = titleInner(key);
  return `<div class="theme theme-${theme}"><div class="col-left big" style="padding:14px 16px"><h1 class="title${t.cls}">${t.html}</h1></div></div>`;
}

function band(act, framing, theme, width) {
  const title = `<div class="act-title"><span class="act-n">Act ${act}</span><span class="act-name">${P.acts[act].name}</span></div>`;
  if (framing === 'none') {
    return `<div class="theme theme-${theme}"><div class="col-mid-mock" style="width:${width}px">
      <div class="act-plain">${title}</div>
      <div class="act-next">(the act's first story entry follows here)</div>
    </div></div>`;
  }
  const f = path.join(ART, 'header', `act${act}-${framing}.png`);
  if (!existsSync(f)) throw new Error(`mock: ${f} missing: run process.py`);
  const y = CROPS[`act${act}-${framing}`] ?? 50;
  const place = framing === 'side' ? 'left' : framing === 'wide' ? 'center' : 'below';
  return `<div class="theme theme-${theme}"><div class="col-mid-mock" style="width:${width}px">
    <div class="act-band ${place}" style="background-image:url('${url(f)}');background-position:50% ${y}%">
      ${title}
    </div>
    <div class="act-next">(the act's first story entry follows here)</div>
  </div></div>`;
}

async function main() {
  let css = await readFile(path.join(REPO, 'web', 'src', 'style.css'), 'utf8');
  css = css.replace(":root,\n:root[data-theme='light'] {", ':root, .theme-light {').replace(":root[data-theme='dark'] {", '.theme-dark {');
  if (!css.includes('.theme-dark {') || !css.includes('.theme-light {')) throw new Error('mock: theme blocks not found in style.css');
  const fonts = ['rajdhani/600.css', 'rajdhani/700.css', 'ibm-plex-sans/400.css', 'ibm-plex-sans/600.css', 'ibm-plex-mono/400.css']
    .map((f) => `<link rel="stylesheet" href="${url(path.join(REPO, 'web', 'node_modules', '@fontsource', f))}">`)
    .join('');
  const extra = `
    html, body { height: auto; overflow: visible; }
    body { background: #8a857c; padding: 0; margin: 0; width: max-content; }
    .pair { display: grid; grid-template-columns: auto auto; }
    .theme { color: var(--text); background: var(--page); padding: 8px; }
    .theme-dark { color-scheme: dark; }
    .col-left { overflow: visible; }
    h1.title.painted { height: 29px; display: flex; align-items: center; justify-content: center; box-sizing: content-box; }
    h1.title.painted img { max-height: 29px; max-width: 100%; }
    .big { width: 600px; }
    .big h1.title { font-size: 48px; border: 0; }
    .big h1.title.painted { height: 58px; }
    .big h1.title.painted img { max-height: 58px; }
    .big .lt-orbit { transform: scale(2); margin-right: 18px; }
    ${Object.values(LETTERED).map((l) => l.css).join('\n')}
    .col-mid-mock { background: var(--page); padding: 10px 0; }
    .act-plain .act-title { color: var(--text); text-shadow: none; }
    .act-plain .act-n { color: var(--accent-text); }
    .act-band { position: relative; aspect-ratio: 3 / 1; background-size: cover; border-radius: 6px; border: 1px solid var(--line); display: flex; }
    .act-band.left { align-items: center; padding-left: 28px; }
    .act-band.center { align-items: center; justify-content: center; }
    .act-band.center::before { content: ''; position: absolute; inset: 0; border-radius: 6px;
      background: radial-gradient(ellipse 45% 40% at 50% 50%, rgba(0,0,0,0.55), transparent); }
    .act-band.below { align-items: flex-end; justify-content: center; padding-bottom: 14px; }
    .act-band.below::before { content: ''; position: absolute; inset: 0; border-radius: 6px;
      background: linear-gradient(to top, rgba(0,0,0,0.7), transparent 50%); }
    .act-title { position: relative; display: flex; flex-direction: column; color: #eeeae4; text-shadow: 0 1px 3px rgba(0,0,0,0.9), 0 0 12px rgba(0,0,0,0.7); }
    .act-band.center .act-title, .act-band.below .act-title { align-items: center; }
    .act-n { font-family: var(--font-head); font-weight: 600; font-size: 13px; letter-spacing: 0.3em; text-transform: uppercase; color: #f7b955; }
    .act-name { font-family: var(--font-head); font-weight: 700; font-size: 30px; letter-spacing: 0.04em; }
    .act-next { font-size: 13px; color: var(--muted); padding: 8px 4px 0; }
  `;
  const page_ = (body) => `<!doctype html><html><head><meta charset="utf-8">${fonts}<style>${css}${extra}</style></head><body>${body}</body></html>`;

  await mkdir(PICK, { recursive: true });
  const browser = await chromium.launch({ channel: 'chrome' });
  const pg = await browser.newPage({ viewport: { width: 1400, height: 900 }, deviceScaleFactor: 2 });
  async function shot(name, body) {
    const html = path.join(OUT, `${name}.html`);
    await writeFile(html, page_(body), { encoding: 'utf-8' });
    await pg.goto(url(html), { waitUntil: 'load' });
    await pg.evaluate(() => document.fonts.ready);
    const failedFonts = await pg.evaluate(() => [...document.fonts].filter((f) => f.status === 'error').map((f) => f.family));
    if (failedFonts.length) throw new Error(`mock: fonts failed: ${failedFonts.join(', ')}`);
    const broken = await pg.evaluate(() => [...document.images].filter((i) => !i.complete || i.naturalWidth === 0).length);
    if (broken) throw new Error(`mock: ${name}: an image did not load`);
    await pg.locator('.sheet').screenshot({ path: path.join(PICK, `${name}.png`) });
    console.log(`mock: ${name}.png`);
  }

  const painted = existsSync(path.join(ART, 'title')) ? readdirSync(path.join(ART, 'title')).filter((f) => f.endsWith('.png')).map((f) => f.slice(0, -4)) : [];
  for (const key of ['current', ...painted, ...Object.keys(LETTERED)]) {
    const body = `<div class="sheet">
      <div class="pair">${titleBig(key, 'light')}${titleBig(key, 'dark')}</div>
      <div class="pair" style="grid-template-columns:auto auto auto auto">${titleColumn(key, 'light', 287)}${titleColumn(key, 'light', 363)}${titleColumn(key, 'dark', 287)}${titleColumn(key, 'dark', 363)}</div>
    </div>`;
    await shot(`title_${key}`, body);
  }
  for (const act of Object.keys(P.acts)) {
    for (const framing of ['none', ...Object.keys(P.header_framing), 'fill']) {
      if (framing !== 'none' && !existsSync(path.join(ART, 'header', `act${act}-${framing}.png`))) continue;
      const body = `<div class="sheet"><div class="pair" style="grid-template-columns:auto">${band(act, framing, 'light', 595)}${band(act, framing, 'dark', 1058)}</div></div>`;
      await shot(`act${act}_${framing}`, body);
    }
  }
  await browser.close();
}

await main();
