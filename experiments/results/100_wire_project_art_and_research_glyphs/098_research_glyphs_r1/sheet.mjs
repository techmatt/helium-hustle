// `node experiments/results/100_wire_project_art_and_research_glyphs/098_research_glyphs_r1/sheet.mjs [prefix]`
// from the repo root (also as `art/research_glyphs/sheet.mjs`, the shared copy): a working contact sheet of
// every research glyph (chip at 96, 32 and 22 px, the bare glyph at 32) into
// out/098_research_glyphs_r1/sheet.png, for checking shapes while drawing. Needs
// playwright-core in PW_DIR (default out/098_research_glyphs_r1/tools/) and system Chrome.
import { mkdir, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { GLYPHS, chip, glyph } from './glyphs.mjs';

const REPO = process.cwd();
const OUT = path.join(REPO, 'out', '098_research_glyphs_r1');
const PW_DIR = process.env.PW_DIR ?? path.join(OUT, 'tools');
const { chromium } = createRequire(path.join(PW_DIR, 'package.json'))('playwright-core');

const only = process.argv[2];
const keys = Object.keys(GLYPHS).filter((k) => !only || k.startsWith(only));
const cell = (k) => `<div class="c"><span style="width:96px;height:96px">${chip(k)}</span><span style="width:32px;height:32px">${chip(k)}</span><span style="width:22px;height:22px">${chip(k)}</span><span style="width:32px;height:32px">${glyph(k, '#8a5cf0', '#fff')}</span><b>${k}</b></div>`;
const html = `<!doctype html><html><body style="margin:0;background:#fff;font:11px sans-serif;width:1500px"><div style="display:flex;flex-wrap:wrap;gap:6px;padding:6px">${keys.map(cell).join('')}</div></body></html>`;
await mkdir(OUT, { recursive: true });
const f = path.join(OUT, 'sheet.html');
await writeFile(f, html, 'utf-8');
const browser = await chromium.launch({ channel: 'chrome' });
const pg = await browser.newPage({ viewport: { width: 1500, height: 400 } });
await pg.goto(pathToFileURL(f).href);
await pg.addStyleTag({ content: '.c{display:flex;align-items:center;gap:6px;width:290px;border:1px solid #ddd;padding:4px}.c span{display:inline-flex;flex:none}.c b{font-weight:400;width:80px;overflow-wrap:anywhere}' });
await pg.screenshot({ path: path.join(OUT, `sheet${only ? `-${only}` : ''}.png`), fullPage: true });
await browser.close();
console.log(`sheet: ${keys.length} glyphs`);
