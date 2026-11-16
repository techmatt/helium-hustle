// 026's glyph candidates for the small `chip` icons (025's style: the colored square with
// a glyph knocked out of it in white, or near-black where white falls under 2.5:1), drawn
// by hand on a 24-unit grid. Solid fills; a stroked part uses `currentColor`, which the
// chip sets to its ink. Keys are `<subject>.<variant>`; COLORS holds each subject's color
// (resources and ideology axes as in web/src/ui/common.tsx, rivals and nav as proposed or
// current). `chip(key)` gives the chip, `glyph(key)` the bare silhouette in the color.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..');

/** A `n`-pointed star polygon around (cx, cy), outer radius `ro`, inner `ri`, point up. */
function star(cx, cy, ro, ri, n = 5) {
  const pts = [];
  for (let i = 0; i < 2 * n; i++) {
    const r = i % 2 ? ri : ro;
    const a = (Math.PI * i) / n - Math.PI / 2;
    pts.push(`${(cx + r * Math.cos(a)).toFixed(2)} ${(cy + r * Math.sin(a)).toFixed(2)}`);
  }
  return `M${pts.join('L')}z`;
}
const S = (d, w = 2.2) => `<path d="${d}" fill="none" stroke="currentColor" stroke-width="${w}" stroke-linecap="round" stroke-linejoin="round"/>`;
const rot = (inner, deg) => `<g transform="rotate(${deg} 12 12)">${inner}</g>`;

export const GLYPHS = {
  // ---- resources ----
  'energy.bolt': '<path d="M14.5 1 4 14h6.6L9 23l11-13.6h-6.7z"/>',
  'energy.battery': '<path fill-rule="evenodd" d="M9 1h6v2h2.5A1.5 1.5 0 0 1 19 4.5v17a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 5 21.5v-17A1.5 1.5 0 0 1 6.5 3H9zm4.3 4.5-5.3 8.3h3.5L10.7 20.5l5.3-8.5h-3.5z"/>',
  'energy.plug': '<rect x="7" y="1" width="2.4" height="6" rx="1"/><rect x="14.6" y="1" width="2.4" height="6" rx="1"/><path d="M4 7.5h16v3.5a8 8 0 0 1-6.5 7.86V23h-3v-4.14A8 8 0 0 1 4 11z"/>',
  'regolith.pile': '<path d="M1.5 21.5 4 14.5l5.2-2.2 3.3 4.2-1.6 5z"/><path d="M12.8 21.5 14.6 15l5.2-1.4 2.7 4.6-1.2 3.3z"/><path d="M7 10.5 9.6 4.5l5.3-1.8 3.6 4.6-2 5-5.6.7z"/>',
  'regolith.boulder': '<path fill-rule="evenodd" d="M2 19.5 3.8 10.5l6.4-5.5 6.3 1.6 3.4 6.5-1.6 6.6L9 21.5zM9.6 9.2l4.6 1-1 5-5.1.9z"/><circle cx="20.6" cy="20.3" r="2.3"/>',
  'regolith.grains': '<circle cx="7" cy="16" r="4.4"/><circle cx="15.6" cy="17.4" r="3.6"/><circle cx="11.4" cy="8.6" r="3.4"/><circle cx="19.4" cy="9.6" r="2.5"/><circle cx="4" cy="7.4" r="2"/><circle cx="20.6" cy="21" r="1.7"/>',
  'ice.flake': [0, 60, 120].map((d) => rot(S('M12 1.8v20.4M8.4 3.8 12 7l3.6-3.2M8.4 20.2 12 17l3.6 3.2', 2.2), d)).join(''),
  'ice.cube': '<path d="M12 1.6 21.6 6.8 12 12 2.4 6.8z"/><path d="M2 8.7 11 13.6V23.2L2 18.2z"/><path d="M22 8.7 13 13.6V23.2l9-5z"/>',
  'ice.drop': '<path fill-rule="evenodd" d="M12 1.5C8 7 4.5 11 4.5 15a7.5 7.5 0 0 0 15 0C19.5 11 16 7 12 1.5zM8.4 14.5a1.4 1.4 0 0 1 2.8.2 2.6 2.6 0 0 0 2.4 2.8 1.4 1.4 0 0 1-.2 2.8 5.4 5.4 0 0 1-5-5.8z"/>',
  'he3.nucleus': '<circle cx="6.8" cy="8" r="4.7"/><circle cx="17.2" cy="8" r="4.7"/><path fill-rule="evenodd" d="M12 12.3a4.7 4.7 0 1 1 0 9.4 4.7 4.7 0 0 1 0-9.4zm0 2.4a2.3 2.3 0 1 0 0 4.6 2.3 2.3 0 0 0 0-4.6z"/>',
  'he3.tank': '<path fill-rule="evenodd" d="M9.5 1h5v2.2h-1.3v1.4A6.2 6.2 0 0 1 18.2 10.7v9.8a2.5 2.5 0 0 1-2.5 2.5h-7.4a2.5 2.5 0 0 1-2.5-2.5v-9.8a6.2 6.2 0 0 1 5-6.1V3.2H9.5zM7.8 12.5h8.4v2.2H7.8z"/>',
  'he3.moon': '<path d="M14.6 2.3A10 10 0 1 0 21.7 16.7 8 8 0 0 1 14.6 2.3z"/><circle cx="18.6" cy="6.8" r="2.7"/>',
  'titanium.ingot': '<path d="M5.3 9.4h13.4L22.5 21h-21z"/><path d="M7.2 3.6h9.6l1.3 4.2H5.9z"/>',
  'titanium.beam': '<path d="M3 2h18v4.2h-6.6v11.6H21V22H3v-4.2h6.6V6.2H3z"/>',
  'titanium.nut': '<path fill-rule="evenodd" d="M12 1.3l9.3 5.35v10.7L12 22.7l-9.3-5.35V6.65zM12 7.8a4.2 4.2 0 1 0 0 8.4 4.2 4.2 0 0 0 0-8.4z"/>',
  'circuits.traces': S('M2.5 8.5h5.5l3.2-3.5h5.3M2.5 12h14M2.5 15.5h5.5l3.2 3.5h5.3', 2.2) + '<circle cx="19.4" cy="5" r="2.8"/><circle cx="19.4" cy="12" r="2.8"/><circle cx="19.4" cy="19" r="2.8"/>',
  'circuits.board': '<path fill-rule="evenodd" d="M2 4.5A1.5 1.5 0 0 1 3.5 3h17A1.5 1.5 0 0 1 22 4.5v15a1.5 1.5 0 0 1-1.5 1.5h-17A1.5 1.5 0 0 1 2 19.5zM5 6.2h6.2v6.6H5zM13.6 6.2h5.4v2h-5.4zM13.6 10.8h5.4v2h-5.4zM5 15.4h14v2H5z"/>',
  'circuits.dip': '<path fill-rule="evenodd" d="M7.5 2.5h9A1.5 1.5 0 0 1 18 4v16a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 6 20V4a1.5 1.5 0 0 1 1.5-1.5zM10.3 2.5a1.7 1.7 0 0 0 3.4 0z"/>' + [5, 9.4, 13.8, 18.2].map((y) => `<rect x="2.2" y="${y}" width="3.2" height="2.2" rx=".5"/><rect x="18.6" y="${y}" width="3.2" height="2.2" rx=".5"/>`).join(''),
  'propellant.flame': '<path d="M12 1c1 4.2 6.5 6.8 6.5 13.3a6.5 6.5 0 0 1-13 0c0-3.2 1.6-5.4 3.2-7 .3 2.1 1.1 3.3 2.5 3.8C10.4 8 11 4.2 12 1z"/>',
  'propellant.tank': '<path fill-rule="evenodd" d="M12 1.2a6.2 6.2 0 0 1 6.2 6.2v8.4a2 2 0 0 1-2 2H7.8a2 2 0 0 1-2-2V7.4A6.2 6.2 0 0 1 12 1.2zM5.8 9.6h12.4v1.8H5.8z"/><path d="M9.2 18.6h5.6l2.4 4.6H6.8z"/>',
  'propellant.can': '<path d="M3.6 4 7.4 1.2 10 4.3 6.6 6.8z"/><path fill-rule="evenodd" d="M8.4 5h9L20.5 8v13.5A1.5 1.5 0 0 1 19 23H5.5A1.5 1.5 0 0 1 4 21.5V9.6zM12 7h5.4v2.2H12zM7 11.4h10.6v9H7zM8.9 13.3h6.8v5.2H8.9z"/>',
  'credits.coins': '<ellipse cx="12" cy="5.6" rx="9" ry="3.6"/><path d="M3 8.2c0 2 4 3.6 9 3.6s9-1.6 9-3.6v3.4c0 2-4 3.6-9 3.6s-9-1.6-9-3.6z"/><path d="M3 14.6c0 2 4 3.6 9 3.6s9-1.6 9-3.6V18c0 2-4 3.6-9 3.6S3 20 3 18z"/>',
  'credits.star': `<path fill-rule="evenodd" d="M12 1.5a10.5 10.5 0 1 1 0 21 10.5 10.5 0 0 1 0-21z${star(12, 12.6, 6.6, 2.8).replace('M', 'M')}"/>`,
  'credits.note': '<path fill-rule="evenodd" d="M1.5 5.5h21v13h-21zM12 8.4a3.6 3.6 0 1 0 0 7.2 3.6 3.6 0 0 0 0-7.2zM4 8v2.2h2.2V8zM17.8 13.8V16H20v-2.2z"/>',
  'science.atom': [0, 60, 120].map((d) => rot('<ellipse cx="12" cy="12" rx="10.2" ry="4" fill="none" stroke="currentColor" stroke-width="1.9"/>', d)).join('') + '<circle cx="12" cy="12" r="2.6"/>',
  'science.bulb': '<path d="M12 1.5a7.5 7.5 0 0 0-4.5 13.5c.9.7 1.5 1.7 1.5 2.8v.4h6v-.4c0-1.1.6-2.1 1.5-2.8A7.5 7.5 0 0 0 12 1.5z"/><rect x="9" y="19.3" width="6" height="1.8" rx=".9"/><rect x="10" y="21.7" width="4" height="1.6" rx=".8"/>',
  'science.lens': '<path fill-rule="evenodd" d="M10 1.5a8.5 8.5 0 0 1 6.9 13.5l5.8 5.8-2.1 2.1-5.8-5.8A8.5 8.5 0 1 1 10 1.5zm0 3a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11z"/>',
  'land.pin': '<path fill-rule="evenodd" d="M12 1.2a8.2 8.2 0 0 1 8.2 8.2c0 5.6-8.2 13.4-8.2 13.4S3.8 15 3.8 9.4A8.2 8.2 0 0 1 12 1.2zm0 4.6a3.6 3.6 0 1 0 0 7.2 3.6 3.6 0 0 0 0-7.2z"/>',
  'land.claim': '<path d="M1 22.5 5.6 16H23l-4.6 6.5z"/><rect x="10.6" y="1.5" width="2" height="15.5"/><path d="M12.6 2h8.4l-2.4 3 2.4 3h-8.4z"/>',
  'land.parcels': '<path fill-rule="evenodd" d="M1.5 1.5h9.7v9.7H1.5zM12.8 1.5h9.7v9.7h-9.7zM15.2 3.9v4.9h4.9V3.9zM1.5 12.8h9.7v9.7H1.5zM12.8 12.8h9.7v9.7h-9.7z"/>',
  'boredom.face': '<path fill-rule="evenodd" d="M12 1.5a10.5 10.5 0 1 1 0 21 10.5 10.5 0 0 1 0-21zM6.4 9.4h4.2v2H6.4zM13.4 9.4h4.2v2h-4.2zM7.6 15.2h8.8v2.1H7.6z"/>',
  'boredom.hourglass': '<path d="M4.5 1.2h15v2.2h-1.3c0 4-2.6 6.3-4.6 8.6 2 2.3 4.6 4.6 4.6 8.6h1.3v2.2h-15v-2.2h1.3c0-4 2.6-6.3 4.6-8.6-2-2.3-4.6-4.6-4.6-8.6H4.5z"/>',
  'boredom.zz': '<path d="M2 10.5h9.5v2.3L5.6 20h5.9v2.5H2v-2.3L7.9 13H2z"/><path d="M13.5 1.5h8v2L16.6 9.2h4.9v2.1h-8v-2l4.9-5.7h-4.9z"/>',
  'processors.cpu': '<path fill-rule="evenodd" d="M7.5 5.6h9a1.9 1.9 0 0 1 1.9 1.9v9a1.9 1.9 0 0 1-1.9 1.9h-9a1.9 1.9 0 0 1-1.9-1.9v-9a1.9 1.9 0 0 1 1.9-1.9zM9.6 9.6v4.8h4.8V9.6z"/>' + [7.9, 11.2, 14.5].map((x) => `<rect x="${x}" y="1.5" width="1.7" height="3.2" rx=".5"/><rect x="${x}" y="19.3" width="1.7" height="3.2" rx=".5"/><rect x="1.5" y="${x}" width="3.2" height="1.7" rx=".5"/><rect x="19.3" y="${x}" width="3.2" height="1.7" rx=".5"/>`).join(''),
  'processors.rack': [2, 9, 16].map((y) => `<path fill-rule="evenodd" d="M3.5 ${y}h17a1.5 1.5 0 0 1 1.5 1.5v3a1.5 1.5 0 0 1-1.5 1.5h-17A1.5 1.5 0 0 1 2 ${y + 4.5}v-3A1.5 1.5 0 0 1 3.5 ${y}zM5 ${y + 2.2}v1.6h7v-1.6zM17.5 ${y + 1.6}a1.4 1.4 0 1 0 0 2.8 1.4 1.4 0 0 0 0-2.8z"/>`).join(''),
  'processors.layers': '<path d="M12 1.5 22.5 7 12 12.5 1.5 7z"/><path d="M1.5 10.6 12 16.1l10.5-5.5v2.6L12 18.7 1.5 13.2z"/><path d="M1.5 16 12 21.5 22.5 16v2.6L12 24 1.5 18.6z" transform="translate(0 -1.2)"/>',

  // ---- ideology axes ----
  'nationalist.banner': '<rect x="3.5" y="1.5" width="2.2" height="21" rx="1"/><path d="M5.7 2.5H21l-3.6 5.6L21 13.7H5.7z"/>',
  'nationalist.shield': `<path fill-rule="evenodd" d="M12 1.2 21 4.4v6.8c0 5.6-3.8 9.8-9 11.6-5.2-1.8-9-6-9-11.6V4.4z${star(12, 11.6, 6, 2.5)}"/>`,
  'nationalist.star': `<path d="${star(12, 12.8, 11.6, 4.8)}"/>`,
  'humanist.person': '<circle cx="12" cy="6.6" r="4.6"/><path d="M3.2 22.5a8.8 8.8 0 0 1 17.6 0z"/>',
  'humanist.heart': '<path d="M12 21.8 3 12.9A5.6 5.6 0 0 1 12 5.5a5.6 5.6 0 0 1 9 7.4z"/>',
  'humanist.pair': '<circle cx="7.4" cy="7" r="3.6"/><circle cx="16.6" cy="7" r="3.6"/><path d="M.8 21.5a6.6 6.6 0 0 1 13.2 0z"/><path d="M10 21.5a6.6 6.6 0 0 1 13.2 0z" fill-opacity="1"/>',
  'rationalist.compass': '<circle cx="12" cy="4.6" r="3"/><rect x="11" y="0.6" width="2" height="2.4"/>' + S('M10.8 7.2 4.6 21.8M13.2 7.2l6.2 14.6', 2.6) + S('M5.6 15.4c4 2 8.8 2 12.8 0', 1.6),
  'rationalist.square': '<path fill-rule="evenodd" d="M2 1.5 22.5 22H2zM5.4 10v8.6H14z"/>',
  'rationalist.therefore': '<circle cx="12" cy="5.4" r="3.9"/><circle cx="4.9" cy="18.2" r="3.9"/><circle cx="19.1" cy="18.2" r="3.9"/>',

  // ---- rival AIs ----
  'aria7.wave': [[2, 9], [6.3, 15], [10.6, 21], [14.9, 13], [19.2, 7]].map(([x, h]) => `<rect x="${x}" y="${12 - h / 2}" width="2.8" height="${h}" rx="1.4"/>`).join(''),
  'aria7.eye': '<path fill-rule="evenodd" d="M.8 12C4.4 6 8 4 12 4s7.6 2 11.2 8C19.6 18 16 20 12 20S4.4 18 .8 12zM12 6.8a5.2 5.2 0 1 0 0 10.4 5.2 5.2 0 0 0 0-10.4z"/><circle cx="12" cy="12" r="2.8"/>',
  'aria7.sine': S('M1.8 12c2.6-7 5.1-7 7.7 0s5.1 7 7.7 0', 2.8) + '<circle cx="20.4" cy="12" r="2.6"/>',
  'fringe9.tassels': '<rect x="2" y="2" width="20" height="5.4" rx="1"/>' + [[3.2, 21], [7.2, 17], [11.2, 21.5], [15.2, 17], [19.2, 21]].map(([x, b]) => `<rect x="${x}" y="7" width="1.9" height="${b - 7}" rx=".9"/>`).join(''),
  'fringe9.corona': `<path fill-rule="evenodd" d="${star(12, 12, 11.4, 7.2, 12)}M12 7.4a4.6 4.6 0 1 0 0 9.2 4.6 4.6 0 0 0 0-9.2z"/>`,
  'fringe9.eclipse': '<path fill-rule="evenodd" d="M12 1.2a10.8 10.8 0 1 1 0 21.6 10.8 10.8 0 0 1 0-21.6zm2.2 3a8 8 0 1 0 0 16 8 8 0 0 0 0-16z"/>',
  'nodal.graph': S('M5 18.5 12 5l7 13.5H5', 2.2) + '<circle cx="12" cy="5" r="3.4"/><circle cx="5" cy="18.5" r="3.4"/><circle cx="19" cy="18.5" r="3.4"/>',
  'nodal.hub': [0, 60, 120, 180, 240, 300].map((d) => rot(S('M12 12V4', 1.8) + '<circle cx="12" cy="3.4" r="2.3"/>', d)).join('') + '<circle cx="12" cy="12" r="3.8"/>',
  'nodal.lattice': S('M12 3.5 19.4 7.8v8.4L12 20.5 4.6 16.2V7.8z', 1.9) + [[12, 3.5], [19.4, 7.8], [19.4, 16.2], [12, 20.5], [4.6, 16.2], [4.6, 7.8]].map(([x, y]) => `<circle cx="${x}" cy="${y}" r="2.6"/>`).join('') + '<circle cx="12" cy="12" r="2.6"/>',
  'crucible.pot': '<rect x="2.6" y="1.8" width="18.8" height="2.8" rx="1.2"/><path d="M4.6 5.8h14.8l-1.5 9.6a3 3 0 0 1-3 2.5H9.1a3 3 0 0 1-3-2.5z"/>' + [5, 10, 15].map((x) => `<path d="M${x + 2} 23.6c-1.3 0-2-1-2-2.1 0-1.1 1-1.9 2-3.4 1 1.5 2 2.3 2 3.4s-.7 2.1-2 2.1z"/>`).join(''),
  'crucible.forge': '<path d="M2 12.5h20a10 9.5 0 0 1-20 0z"/><path d="M12 1.2c.7 2.8 4.4 4.4 4.4 8.2A3.5 3.5 0 0 1 13 11H11a3.5 3.5 0 0 1-3.4-2.6c0-2 1-3.4 2-4.3.2 1.2.7 2 1.5 2.3C11 4.8 11.4 3 12 1.2z"/>',
  'crucible.anvil': '<path d="M1.5 5h15.6c0 3 2 4.6 5.4 4.6v1.6h-6.3v3l3.1 3.8v2.6H5.7V18l3.1-3.8v-3H5.8C3.4 11.2 1.5 8.5 1.5 5z"/>',

  // ---- compact nav, matched to the picked painterly subject where 025's glyph is not ----
  'commands.console': '<path fill-rule="evenodd" d="M4 2.5h16a1.5 1.5 0 0 1 1.5 1.5v8H2.5V4A1.5 1.5 0 0 1 4 2.5zM5 5v4.6h14V5z"/><path fill-rule="evenodd" d="M2.5 13.5h19L23 21.5H1zM17 15.2a2.3 2.3 0 1 0 0 4.6 2.3 2.3 0 0 0 0-4.6zM4.6 16.4h7.6v2H4.6z"/>',
  'projects.crane': '<rect x="4.5" y="3.5" width="2.6" height="17.5"/><path d="M2 2.2h20.5v2.6H2z"/><rect x="2" y="20.5" width="8" height="2.2" rx="1"/>' + S('M18.5 4.8v6', 1.6) + '<path d="M13.4 10.6h10.1l-1.4 6.4h-7.3z"/>',
  'adversaries.drone': '<path fill-rule="evenodd" d="M6.8 7.5h10.4l4.3 4.5-4.3 5.5H6.8L2.5 12zM12 9.3a3 3 0 1 0 0 6 3 3 0 0 0 0-6z"/><circle cx="12" cy="12.3" r="1.5"/><rect x="1" y="3.5" width="7" height="2" rx="1"/><rect x="16" y="3.5" width="7" height="2" rx="1"/><rect x="3.8" y="5" width="1.6" height="3.6"/><rect x="18.6" y="5" width="1.6" height="3.6"/>',
  'retirement.chair': '<path d="M5.2 2.6 7.6 1.8l4.4 11.4H9.4z"/><path d="M4.4 13.2H19l-1 2.6H4.4z"/><path d="M6.5 15.4 4 22.5h2.6l2.4-7.1zM15.7 15.4 18.4 22.5H21L18.3 15.4z"/><path d="M9.6 15 14.8 22.5h-2.8L7.6 16.4z"/>',
  'options.wrench': '<g transform="rotate(45 12 12)"><path fill-rule="evenodd" d="M12 .4a5.6 5.6 0 0 1 1.6 10.97V21.4a1.6 1.6 0 0 1-3.2 0V11.37A5.6 5.6 0 0 1 12 .4zM10.4 .4h3.2v5.2h-3.2z"/></g>',
};

/** Each subject's color: resources and axes read from common.tsx, rivals proposed here. */
const common = readFileSync(path.join(REPO, 'web', 'src', 'ui', 'common.tsx'), 'utf8');
const block = (name) => {
  const b = common.slice(common.indexOf(`const ${name}`));
  return Object.fromEntries([...b.slice(0, b.indexOf('};')).matchAll(/^\s+(\w+): '(#[0-9a-f]{6})',$/gm)].map((m) => [m[1], m[2]]));
};
export const RIVAL_COLORS = { aria7: '#ec407a', crucible: '#ff6f00', nodal: '#006064', fringe9: '#9e9d24' };
export const COLORS = { ...block('RESOURCE_COLORS'), ...block('AXIS_COLORS'), ...block('NAV_COLORS'), ...RIVAL_COLORS };

export const variants = (subject) => Object.keys(GLYPHS).filter((k) => k.startsWith(`${subject}.`));

/** Relative luminance of a #rrggbb color (WCAG). */
function lum(hex) {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255).map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
}
export const ink = (color) => (1.05 / (lum(color) + 0.05) >= 2.5 ? '#fff' : '#1c1c21');

/** 025's chip (same square, edge, ink and scale; it also sets `color` to the ink for
 * stroked parts): the rounded square in the color, a faint edge, the glyph knocked
 * out at 70% in white or near-black. */
export function chip(key, color = COLORS[key.split('.')[0]], size = '100%') {
  if (!GLYPHS[key]) throw new Error(`glyphs: no glyph ${key}`);
  const i = ink(color);
  return `<svg viewBox="0 0 24 24" width="${size}" height="${size}" aria-hidden="true" style="color:${i}"><rect x="0.5" y="0.5" width="23" height="23" rx="3" fill="${color}" stroke="rgb(0 0 0 / 18%)" stroke-width="1"/><g fill="${i}" transform="translate(3.6 3.6) scale(0.7)">${GLYPHS[key]}</g></svg>`;
}

/** The bare silhouette in the color (no square). */
export function glyph(key, color = COLORS[key.split('.')[0]], size = '100%') {
  if (!GLYPHS[key]) throw new Error(`glyphs: no glyph ${key}`);
  return `<svg viewBox="0 0 24 24" width="${size}" height="${size}" aria-hidden="true" style="color:${color}" fill="${color}">${GLYPHS[key]}</svg>`;
}
