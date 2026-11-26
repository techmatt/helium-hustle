// 100: project card art and research chips in a browser (scenario everything), both themes.
//   cd web && npm run build && npx vite preview --port <p> --strictPort
//   HH_URL=http://localhost:<p>/ node ../experiments/results/100_wire_project_art_and_research_glyphs/shots.mjs [outDir]
// Checks every shown research card has its chip (in its category's color, before the title)
// and every shown project card its picture (loaded, 96 px).
import { mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('shots', 'node shots.mjs');
const outDir = resolve(process.argv[2] ?? here);
mkdirSync(outDir, { recursive: true });
const b = await launch({ width: 1600, height: 1000, profileRoot: resolve(here, '../../../out/100_wire_project_art_and_research_glyphs/profile') });
const { evaluate } = b;
const { click, setValue, load, speed, panel, debugMenu, autoStory } = driver(b, { url, settle: 700 });

const failures = [];
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) failures.push(what);
};
const shot = async (name) => {
  await sleep(400);
  await capture(b, resolve(outDir, `${name}.png`));
};
const theme = (t) => evaluate(`document.documentElement.dataset.theme = '${t}'`);
const both = async (name) => {
  await theme('light');
  await shot(`100_${name}_light`);
  await theme('dark');
  await shot(`100_${name}_dark`);
  await theme('light');
};

await load();
await autoStory();
await debugMenu();
await speed(0);
await setValue('#scenario', 'everything');
await click('#start-scenario');
await autoStory();
await sleep(500);

await panel('research');
const r = await evaluate(`[...document.querySelectorAll('.card.research')].map((c) => {
  const head = c.querySelector('.card-head');
  const chip = head?.querySelector('svg.chip');
  return { id: c.dataset.id, chip: chip?.dataset.chip ?? null, first: head?.firstElementChild === chip,
    fill: chip?.querySelector('rect')?.getAttribute('fill') ?? null, size: chip?.getAttribute('width') ?? null };
})`);
console.log(JSON.stringify(r));
check(r.length > 0, `research cards shown: ${r.length}`);
check(r.every((x) => x.chip === x.id && x.first && x.size === '22'), `every research card leads its title with its own 22 px chip`);
check(new Set(r.map((x) => x.fill)).size > 1 && !r.some((x) => x.fill === '#999999'), `chips take their category colors: ${[...new Set(r.map((x) => x.fill))].join(' ')}`);
await both('research');

await panel('projects');
const p = await evaluate(`Promise.all([...document.querySelectorAll('.card.project')].map(async (c) => {
  const img = c.querySelector('img.card-art');
  if (img) await img.decode().catch(() => {});
  return { id: c.dataset.id, art: !!img, w: img?.naturalWidth ?? 0, box: img?.getBoundingClientRect().width ?? 0 };
}))`);
console.log(JSON.stringify(p));
check(p.length > 0, `project cards shown: ${p.length}`);
check(p.every((x) => x.art && x.w === 192 && x.box === 96), `every project card shows its 192 px picture in a 96 px box`);
await both('projects');

check(b.errors.length === 0, `no page errors: ${b.errors.join(' | ')}`);
await b.close();
console.log(failures.length ? `${failures.length} failed` : 'all ok');
process.exit(failures.length ? 1 : 0);
