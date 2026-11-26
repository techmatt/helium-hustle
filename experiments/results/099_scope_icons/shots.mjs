// 099: scope icons in a browser, before the reveal (scenario late_unlocks), both themes.
//   cd web && npm run build && npx vite preview --port <p> --strictPort
//   HH_URL=http://localhost:<p>/ node ../experiments/results/099_scope_icons/shots.mjs [outDir]
// Research: Regolith Processing (run-only) shows nothing. Projects: the global ones Legacy.
// Story: 1.a Big Launch and 1.b Full Cluster Legacy, 1.c (no reward of its own) nothing.
// After the reveal (Eternal) is the hh-core test scope_icons_show_legacy_until_the_reveal_then_eternal.
import { mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { capture, driver, launch, pageUrl, sleep } from '../../../web/scripts/cdp.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const url = pageUrl('shots', 'node shots.mjs');
const outDir = resolve(process.argv[2] ?? here);
mkdirSync(outDir, { recursive: true });
const b = await launch({ width: 1600, height: 900, profileRoot: resolve(here, '../../../out/099_scope_icons/profile') });
const { evaluate } = b;
const { click, setValue, load, speed, panel, debugMenu, autoStory } = driver(b, { url, settle: 700 });

const failures = [];
const check = (ok, what) => {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) failures.push(what);
};
const shot = async (name) => {
  await sleep(250);
  await capture(b, resolve(outDir, `${name}.png`));
};
const theme = (t) => evaluate(`document.documentElement.dataset.theme = '${t}'`);
const keeps = (sel) =>
  evaluate(`Object.fromEntries([...document.querySelectorAll('${sel}')].map((c) => [c.dataset.id ?? c.dataset.side, c.querySelector('.scope-icon')?.dataset.keep ?? null]))`);
const both = async (name) => {
  await theme('light');
  await shot(`099_${name}_light`);
  await theme('dark');
  await shot(`099_${name}_dark`);
  await theme('light');
};

await load();
await autoStory();
await debugMenu();
await speed(0);
await setValue('#scenario', 'late_unlocks');
await click('#start-scenario');
await autoStory();
await sleep(500);

await panel('research');
const r = await keeps('.card.research');
// A scenario past 1.3 starts with Programming owned (research::grant_flags_due), so its card
// is gone: the hh-core test covers its icon.
check(!('programming' in r), `Programming is owned here, its card gone: ${r.programming}`);
check(r.regolith_processing === null, `a run-only research shows nothing: ${r.regolith_processing}`);
await both('research');

await panel('projects');
const p = await keeps('.card.project');
// Completed ones (Conditional Logic, the Archive) are not shown (096): the fundable globals.
check(p.foundation_grant === 'legacy' && p.chemical_energy === 'legacy', `global projects show Legacy: ${JSON.stringify(p)}`);
const tip = await evaluate(`document.querySelector('.card.project[data-id="foundation_grant"] .scope-icon')?.title ?? null`);
check(tip === 'Legacy: Kept when you retire.', `its hover line: ${tip}`);
await both('projects');

await panel('story');
const s = await keeps('[data-testid="story-side"]');
check(s.qbig_launch === 'legacy' && s.qblueprints_for_everyone === null, `side quests: ${JSON.stringify(s)}`);
await both('story');

check(b.errors.length === 0, `no page errors: ${b.errors.join(' | ')}`);
await b.close();
console.log(failures.length ? `${failures.length} failed` : 'all ok');
process.exit(failures.length ? 1 : 0);
