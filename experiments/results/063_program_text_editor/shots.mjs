// 063's screenshots of the program text editor (web/src/ui/TextEditor.tsx) and of the block
// editor's new rows. From scenario everything, once program logic opens: every processor on
// Program 1, L4's Program 1 (L4_program1.txt) pasted with a junk line (red) and a locked
// building (amber), applied, then days played so lines light green and grey and the if
// lines count their days true. In dark, then light:
//   063_text_editor_<theme>.png   the takeover: green, grey, red and amber lines, fired counts
//   063_block_editor_<theme>.png  the block editor: a note, an unparsed and a locked row
// Drives headless Edge or Chrome over web/scripts/cdp.mjs against a served static build.
//   HH_URL  the page, required (a `vite preview`, web/README.md)
//   OUT     where the PNGs go (default: this folder)
//   COPY    a second folder to copy them to (optional)
import { copyFileSync, mkdirSync, readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, q, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('shots', 'node experiments/results/063_program_text_editor/shots.mjs');
const OUT = resolve(process.env.OUT ?? here);
mkdirSync(OUT, { recursive: true });
const W = 1600;
const H = 1000;
const b = await launch({ width: W, height: H, emulate: true, profileRoot: resolve(here, '../../../out/063_program_text_editor/profile') });
const { evaluate } = b;
const page = driver(b, { url });
const { waitFor, click, load, debugMenu, setValue, exists } = page;
const check = (ok, what) => {
  if (!ok) throw new Error(`FAILED: ${what}`);
  console.log(`ok: ${what}`);
};
const shot = async (name) => {
  const file = resolve(OUT, `063_${name}.png`);
  await capture(b, file, { clip: { x: 0, y: 0, width: W, height: H, scale: 1 } });
  if (process.env.COPY) copyFileSync(file, resolve(process.env.COPY, `063_${name}.png`));
  console.log(`shot 063_${name}`);
};
const AREA = '[data-testid="text-editor-area"]';
const statuses = () => evaluate(`[...document.querySelectorAll('.te-gutter [data-line-status]')].map((d) => d.dataset.lineStatus)`);
const text =
  readFileSync(resolve(here, 'L4_program1.txt'), 'utf8').replace(/\r/g, '').trimEnd() + '\n# a typo, then a building not open yet\nfrobnicate 3\nbuild fuel cell array';

/** Opens the text editor and plays days until green and grey lines and a fired count show. */
async function editorShot(theme) {
  // The Buildings panel behind, for the block editor's shot.
  await page.panel('buildings');
  await click('[data-testid="text-editor-open"]', 'open the text editor');
  await waitFor(`!!document.querySelector('[data-testid="text-editor"]')`, 'the text editor');
  await page.speed(3);
  // At least ten days counted, and a line of each color on the last.
  await waitFor(`(() => { const s = [...document.querySelectorAll('.te-gutter [data-line-status]')].map((d) => d.dataset.lineStatus); return s.includes('green') && s.includes('grey') && !!document.querySelector('.te-gutter [data-fired$="/10"]'); })()`, 'green, grey and a fired count of 10 days', 20000);
  await page.speed(0);
  const s = await statuses();
  check(['green', 'grey', 'red', 'amber', 'note'].every((x) => s.includes(x)), `${theme}: the gutter shows ${[...new Set(s)].join(', ')}`);
  await shot(`text_editor_${theme}`);
  await click('[data-takeover-close="text_editor"]', 'close');
  await sleep(100);
  // The block editor, scrolled so the note (top) to the unparsed and locked rows (foot) show.
  check((await exists('.lines li.note-line')) && (await exists('.lines li.unparsed-line')) && (await exists('.lines li.is-locked')), `${theme}: note, unparsed and locked rows`);
  await evaluate(`document.querySelector('.lines li.is-locked').scrollIntoView({ block: 'end' })`);
  await sleep(200);
  await shot(`block_editor_${theme}`);
  await evaluate(`document.querySelector('.col-right').scrollTop = 0`);
}

try {
  await load();
  await debugMenu();
  await click('[data-theme-pick="dark"]', 'dark');
  await page.speed(0);
  await setValue('#scenario', 'everything');
  await click('#start-scenario', 'scenario everything');
  await page.speed(1);
  await waitFor(`!!document.querySelector('[data-testid="logic-add"]')`, 'program logic to open', 8000);
  await page.speed(0);
  await click('.prog-tabs button[data-program="0"]', 'program 1');
  while (await evaluate(`!document.querySelector('[data-testid="proc-plus"]').disabled`)) await click('[data-testid="proc-plus"]', 'a processor');
  await click('[data-testid="text-editor-open"]', 'open the text editor');
  await setValue(AREA, text);
  await sleep(300);
  await click('[data-testid="text-editor-apply"]', 'Apply');
  await sleep(200);
  check(await evaluate(`document.querySelector(${q('[data-testid="text-editor-apply"]')}).disabled`), 'applied (Apply off again)');
  await click('[data-takeover-close="text_editor"]', 'close');
  await editorShot('dark');
  await debugMenu();
  await click('[data-theme-pick="light"]', 'light');
  await editorShot('light');
  check(b.errors.length === 0, `no page errors${b.errors.length ? `: ${b.errors.join('; ')}` : ''}`);
} finally {
  await b.close();
}
