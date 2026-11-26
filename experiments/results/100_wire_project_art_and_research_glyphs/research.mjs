// `node experiments/results/100_wire_project_art_and_research_glyphs/research.mjs` from the
// repo root: ships Matt's research glyph picks (picker research_glyphs_r1) as
// web/src/assets/research/<research id>.svg, one 24-unit glyph each from 098's glyphs.mjs
// (committed beside this script). The `BG` fill token is kept: icons.tsx's ResearchChip
// swaps it for the chip's category color. Every pick is checked before anything is written.
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { GLYPHS } from './098_research_glyphs_r1/glyphs.mjs';

const REPO = process.cwd();
const OUT = path.join(REPO, 'web', 'src', 'assets', 'research');

const PICKS = {
  basic_blueprints: 'roll',
  advanced_blueprints: 'stack',
  regolith_processing: 'drill',
  propellant_synthesis: 'flask',
  dream_protocols: 'moon',
  stress_tolerance: 'spring',
  efficient_dreaming: 'cloudbolt',
  overclock_protocols: 'gauge',
  overclock_boost: 'chevrons',
  market_awareness: 'lens',
  speculator_analysis: 'radar',
  trade_promotion: 'megaphone',
  shipping_efficiency: 'moving',
  geopolitical_intelligence: 'capitol',
  novelty_simulation: 'dice',
  process_telemetry: 'pulse',
  programming: 'terminal',
};

const research = JSON.parse(await readFile(path.join(REPO, 'content', 'research.json'), 'utf8'));
const ids = new Set(research.map((r) => r.id));
const files = [];
for (const [id, key] of Object.entries(PICKS)) {
  if (!ids.has(id)) throw new Error(`research: no research ${id} in content/research.json`);
  const g = GLYPHS[`${id}.${key}`];
  if (!g) throw new Error(`research: no glyph ${id}.${key}`);
  files.push([id, `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">${g}</svg>\n`]);
}
const missing = [...ids].filter((id) => !PICKS[id]);
if (missing.length) console.log(`research: no pick for ${missing.join(', ')} (their cards show no chip)`);
await mkdir(OUT, { recursive: true });
for (const [id, svg] of files) await writeFile(path.join(OUT, `${id}.svg`), svg, 'utf-8');
console.log(`research: ${files.length} glyphs`);
