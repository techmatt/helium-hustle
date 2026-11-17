// 027: writes the picked small-icon glyphs and compact nav tiles into web/src/assets/.
// Run from the repo root: `node experiments/results/027_art_in_game/export.mjs`.
//
// - assets/chips/<id>.svg: the bare glyph (24-unit grid, no color) for each resource,
//   ideology axis and rival AI; the UI's <Chip> draws the square in the id's color token
//   and knocks the glyph out of it.
// - assets/nav/<id>-compact.svg: the whole compact nav tile (18 px), color baked in:
//   025's glyph in the colored square (`chip`), or 026's subject glyph (`subj`).
// The painted nav tiles and the horizon strips are converted by `images.py`.

import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chip as chip025 } from '../025_nav_horizon_art_r1/glyphs.mjs';
import { GLYPHS, NAV_COLORS, chip as chip026 } from '../026_icon_art_r1/glyphs.mjs';

const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const ASSETS = path.join(REPO, 'web', 'src', 'assets');

// Matt's picks in small_icons_r1, one glyph per subject.
const CHIPS = {
  energy: 'bolt', regolith: 'pile', ice: 'flake', he3: 'nucleus', titanium: 'ingot', circuits: 'dip',
  propellant: 'can', credits: 'coins', science: 'atom', land: 'claim', boredom: 'zz', processors: 'cpu',
  nationalist: 'shield', humanist: 'pair', rationalist: 'compass',
  aria7: 'eye', crucible: 'anvil', nodal: 'graph', fringe9: 'eclipse',
};
// Compact nav: `chip` is 025's glyph; a subject glyph names its 026 key.
const NAV = {
  buildings: 'chip', commands: 'chip', launch_pads: 'chip', research: 'chip', projects: 'projects.crane',
  ideologies: 'chip', adversaries: 'adversaries.drone', stats: 'chip', story: 'chip',
  retirement: 'retirement.chair', options: 'options.wrench',
};

const xmlns = (svg) => svg.replace('<svg ', '<svg xmlns="http://www.w3.org/2000/svg" ').replace(/ width="100%" height="100%"/, '');

mkdirSync(path.join(ASSETS, 'chips'), { recursive: true });
for (const [id, variant] of Object.entries(CHIPS)) {
  const g = GLYPHS[`${id}.${variant}`];
  if (!g) throw new Error(`no glyph ${id}.${variant}`);
  writeFileSync(path.join(ASSETS, 'chips', `${id}.svg`), `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">${g}</svg>\n`, { encoding: 'utf-8' });
}
mkdirSync(path.join(ASSETS, 'nav'), { recursive: true });
for (const [id, pick] of Object.entries(NAV)) {
  const svg = pick === 'chip' ? chip025(id, NAV_COLORS[id]) : chip026(pick, NAV_COLORS[id]);
  writeFileSync(path.join(ASSETS, 'nav', `${id}-compact.svg`), `${xmlns(svg)}\n`, { encoding: 'utf-8' });
}
console.log(`wrote ${Object.keys(CHIPS).length} chip glyphs, ${Object.keys(NAV).length} compact nav tiles`);
