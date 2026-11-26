// 098's glyph candidates for research cards (picker research_glyphs_r1), in 026's chip style
// (experiments/results/026_icon_art_r1/glyphs.mjs): a 24-unit grid drawn by hand, solid
// fills, a stroked part in `currentColor` (the chip sets it to its ink). New here: `BG`, a
// fill that `chip()` swaps for the chip's own color, for a knock-out drawn over a solid part
// (a bolt cut out of a cloud). Keys are `<research id>.<variant>`.

/** A `n`-pointed star polygon around (cx, cy), outer radius `ro`, inner `ri`, point up. */
export function star(cx, cy, ro, ri, n = 5) {
  const pts = [];
  for (let i = 0; i < 2 * n; i++) {
    const r = i % 2 ? ri : ro;
    const a = (Math.PI * i) / n - Math.PI / 2;
    pts.push(`${(cx + r * Math.cos(a)).toFixed(2)} ${(cy + r * Math.sin(a)).toFixed(2)}`);
  }
  return `M${pts.join('L')}z`;
}
const S = (d, w = 2.2) => `<path d="${d}" fill="none" stroke="currentColor" stroke-width="${w}" stroke-linecap="round" stroke-linejoin="round"/>`;
const rot = (inner, deg, cx = 12, cy = 12) => `<g transform="rotate(${deg} ${cx} ${cy})">${inner}</g>`;
/** A polygon's points scaled by `s` about (12, 12), then moved by (dx, dy), as a subpath. */
const poly = (pts, s = 1, dx = 0, dy = 0) => `M${pts.map(([x, y]) => `${(12 + (x - 12) * s + dx).toFixed(2)} ${(12 + (y - 12) * s + dy).toFixed(2)}`).join('L')}z`;
const BOLT = [[14.5, 1], [4, 14], [10.6, 14], [9, 23], [20, 9.4], [13.3, 9.4]];
const circ = (cx, cy, r) => `M${cx - r} ${cy}a${r} ${r} 0 1 0 ${2 * r} 0a${r} ${r} 0 1 0 ${-2 * r} 0z`;
// A sheet of plans with a folded corner: the sheet (corner cut) and the fold.
const SHEET = 'M4 1.5h10.2v6.3h5.8V22.5H4z';
const FOLD = '<path d="M15.7 1.5 20 5.8h-4.3z"/>';
const CLOUD = '<circle cx="8" cy="10.5" r="4.6"/><circle cx="13.6" cy="8" r="5.6"/><circle cx="18.2" cy="11.6" r="3.9"/><rect x="3.4" y="10.6" width="18.6" height="5.6" rx="2.8"/>';
const CRESCENT = 'M11 2A10 10 0 1 0 20.5 17.5 8 8 0 0 1 11 2z';
const GAUGE = S('M3.5 17.5a8.5 8.5 0 1 1 17 0', 3) + '<path d="M11 16.2 18.6 8.8l1.2 1.2-7.4 7.6z"/><circle cx="12" cy="17.2" r="2.4"/>';

export const GLYPHS = {
  // ---- Blueprints ----
  'basic_blueprints.sheet': `<path fill-rule="evenodd" d="${SHEET}M7.6 19.5v-5.2l4.4-3.9 4.4 3.9v5.2z"/>${FOLD}`,
  'basic_blueprints.roll': '<rect x="1.5" y="2" width="6" height="18.5" rx="3"/><path fill-rule="evenodd" d="M8.6 4h13.9v16.5H8.6zM11 7.2h9v1.9h-9zM11 11.2h6v1.9h-6zM11 15.2h8v1.9h-8z"/><circle cx="4.5" cy="5" r="1.3" fill="BG"/>',
  'basic_blueprints.grid': `<path fill-rule="evenodd" d="M2 2h20v20H2z${[4.2, 10, 15.8].flatMap((y) => [4.2, 10, 15.8].map((x) => (x === 10 && y === 10 ? '' : `M${x} ${y}h4v4h-4z`))).join('')}"/>`,
  'advanced_blueprints.stack': '<path d="M7.5 1.5h13.2a1.8 1.8 0 0 1 1.8 1.8v14.6h-2.4V3.9H7.5z"/><path fill-rule="evenodd" d="M2 5.8h15.3V22.5H2zM8.4 10h2.6v3.2h3.2v2.6H11V19H8.4v-3.2H5.2v-2.6h3.2z"/>',
  'advanced_blueprints.gear': `<path fill-rule="evenodd" d="${SHEET}${star(12, 14.8, 5.9, 4.4, 8)}${circ(12, 14.8, 1.9)}"/>${FOLD}`,
  'advanced_blueprints.bolt': `<path fill-rule="evenodd" d="${SHEET}${poly(BOLT, 0.58, 0, 2.6)}"/>${FOLD}`,

  // ---- Extraction ----
  'regolith_processing.shovel': rot('<path d="M9 1.2h6v2.4h-1.8v8.2h-2.4V3.6H9z"/><path d="M6.5 12.3h11v4.4c0 2.9-2.5 5.2-5.5 6.5-3-1.3-5.5-3.6-5.5-6.5z"/>', 35),
  'regolith_processing.pick': rot('<path d="M1 10C5 0 19 0 23 10 19.4 7.6 15.6 6.8 12 6.8S4.6 7.6 1 10z"/><rect x="10.3" y="4.4" width="3.4" height="18.6" rx="1.4"/>', -30),
  'regolith_processing.drill': '<path d="M7.6 1.2h8.8v4.2H7.6z"/><path fill-rule="evenodd" d="M8.2 6.4h7.6v9.2L12 23 8.2 15.6zM8.2 10.4l7.6-2.6v2.2l-7.6 2.6zM8.2 14.4l7.6-2.6V14l-7.6 2.6z"/>',

  // ---- Self-Maintenance ----
  'propellant_synthesis.flask': `<path fill-rule="evenodd" d="M8.6 1.2h6.8v2.2h-1.2v5.7l6.4 10.7A2.3 2.3 0 0 1 18.6 23H5.4a2.3 2.3 0 0 1-2-3.4l6.4-10.5V3.4H8.6z${circ(9.8, 17.6, 1.6)}${circ(14.2, 15.4, 1.2)}${circ(13, 19.8, 1.1)}"/>`,
  'propellant_synthesis.drop': '<path d="M12 1C8 6.6 4.2 10.8 4.2 15.2a7.8 7.8 0 0 0 15.6 0C19.8 10.8 16 6.6 12 1z"/><g transform="translate(6.9 9.4) scale(0.43)" fill="BG"><path d="M12 1c1 4.2 6.5 6.8 6.5 13.3a6.5 6.5 0 0 1-13 0c0-3.2 1.6-5.4 3.2-7 .3 2.1 1.1 3.3 2.5 3.8C10.4 8 11 4.2 12 1z"/></g>',
  'propellant_synthesis.cell': '<path d="M2.5 7h2.4v13a1 1 0 0 0 1 1h12.2a1 1 0 0 0 1-1V7h2.4v13.2a3.2 3.2 0 0 1-3.2 3.2H5.7a3.2 3.2 0 0 1-3.2-3.2z"/><path d="M5.6 12.2h12.8v7.4a.8.8 0 0 1-.8.8H6.4a.8.8 0 0 1-.8-.8z"/><rect x="7.6" y="1.2" width="2.6" height="15.6" rx=".6" stroke="BG" stroke-width="1.2"/><rect x="13.8" y="1.2" width="2.6" height="15.6" rx=".6" stroke="BG" stroke-width="1.2"/><circle cx="11.9" cy="18.4" r="1" fill="BG"/><circle cx="7.4" cy="18.6" r=".9" fill="BG"/><circle cx="16.8" cy="18.3" r=".9" fill="BG"/>',
  'dream_protocols.cloud': CLOUD.replace(/cy="(\S+?)"/g, (_, y) => `cy="${Number(y) - 2}"`).replace('y="10.6"', 'y="8.6"') + '<circle cx="6" cy="18.4" r="2.3"/><circle cx="3" cy="21.8" r="1.4"/>',
  'dream_protocols.moon': `<path d="${CRESCENT}"/><path d="${star(18.2, 5.6, 4, 1.3, 4)}${star(21.4, 11.6, 2.4, 0.8, 4)}"/>`,
  'dream_protocols.lids': S('M2.5 9.5c5 6.6 14 6.6 19 0', 2.6) + S('M12 16v3.6M6.4 14.2l-1.8 3M17.6 14.2l1.8 3', 2.3) + '<path d="M15.5 1.5h6v1.6l-3.6 3.5h3.6v1.8h-6V6.8l3.6-3.5h-3.6z"/>',
  'stress_tolerance.spring': '<rect x="3" y="1.5" width="18" height="3" rx="1"/><rect x="3" y="19.5" width="18" height="3" rx="1"/>' + S('M12 4.5 17 6.6 7 9.2l10 2.6-10 2.6 10 2.6L12 19.5', 2.2),
  'stress_tolerance.thermo': `<path fill-rule="evenodd" d="M6.6 4.3a3 3 0 0 1 6 0v9.2a5.6 5.6 0 1 1-6 0zM8.6 4.3a1 1 0 0 1 2 0v11.5h-2z"/>` + S('M18.6 3.5v12', 2.6) + '<path d="M14.6 13.2h8l-4 6.3z"/>',
  'stress_tolerance.calm': S('M1.8 6.5 4.6 2.5l2.8 4 2.8-4 2.8 4 2.8-4 2.8 4 2.8-4', 2.4) + '<path d="M8.5 9.6h7L12 14z"/>' + S('M2 18.8c3.3-2 6.7-2 10 0s6.7 2 10 0', 2.4),
  'efficient_dreaming.cloudbolt': CLOUD + `<path fill="BG" d="${poly(BOLT, 0.42, 0.4, 0.8)}"/>` + '<circle cx="5.4" cy="20" r="1.9"/>',
  'efficient_dreaming.moonbolt': `<path d="${CRESCENT}"/><path d="${poly(BOLT, 0.52, 7, -5.4)}"/>`,
  'efficient_dreaming.clouddown': CLOUD + '<path fill="BG" d="M11 6.6h2v4.4h2.8L12 15.2 8.2 11H11z"/><path d="M8 19.2h8v2.4H8z"/>',
  'novelty_simulation.dice': rot(`<path fill-rule="evenodd" d="M5 2.5h14A2.5 2.5 0 0 1 21.5 5v14a2.5 2.5 0 0 1-2.5 2.5H5A2.5 2.5 0 0 1 2.5 19V5A2.5 2.5 0 0 1 5 2.5z${circ(7.5, 7.5, 1.8)}${circ(16.5, 7.5, 1.8)}${circ(12, 12, 1.8)}${circ(7.5, 16.5, 1.8)}${circ(16.5, 16.5, 1.8)}"/>`, 12),
  'novelty_simulation.gift': '<path fill-rule="evenodd" d="M2.5 8.2h19v4.2h-19zM4 13.6h16v9H4zM11 8.2h2v14.4h-2z"/><path d="M11.4 7.8C8.6 2 4.2 3 5.4 6.2c.6 1.6 3.5 1.8 6 1.6zM12.6 7.8c2.8-5.8 7.2-4.8 6-1.6-.6 1.6-3.5 1.8-6 1.6z"/>',
  'novelty_simulation.puzzle': '<path d="M2.5 6.5h5.1a2.9 2.9 0 1 1 5.8 0h5.1v5.1a2.9 2.9 0 1 1 0 5.8v5.1h-5.1a2.9 2.9 0 1 0-5.8 0H2.5v-5.1a2.9 2.9 0 1 0 0-5.8z"/>',

  // ---- Overclock Algorithms ----
  'overclock_protocols.gauge': GAUGE + '<path d="M3 20.5h18v2H3z"/>',
  'overclock_protocols.cpubolt': `<path fill-rule="evenodd" d="M7.5 5.6h9a1.9 1.9 0 0 1 1.9 1.9v9a1.9 1.9 0 0 1-1.9 1.9h-9a1.9 1.9 0 0 1-1.9-1.9v-9a1.9 1.9 0 0 1 1.9-1.9z${poly(BOLT, 0.48)}"/>` + [7.9, 11.2, 14.5].map((x) => `<rect x="${x}" y="1.5" width="1.7" height="3.2" rx=".5"/><rect x="${x}" y="19.3" width="1.7" height="3.2" rx=".5"/><rect x="1.5" y="${x}" width="3.2" height="1.7" rx=".5"/><rect x="19.3" y="${x}" width="3.2" height="1.7" rx=".5"/>`).join(''),
  'overclock_protocols.forward': '<path d="M1.5 4.5 11.5 12l-10 7.5zM12 4.5 22 12l-10 7.5z"/>',
  'overclock_boost.raise': '<rect x="2.5" y="1.5" width="19" height="3" rx="1"/><path d="M12 6.4 19 14h-4.4v8.6H9.4V14H5z"/>',
  'overclock_boost.chevrons': S('M4 9.5 12 3l8 6.5M4 15 12 8.5l8 6.5M4 20.5 12 14l8 6.5', 2.8),
  'overclock_boost.bars': '<rect x="2" y="15" width="5" height="7.5" rx=".8"/><rect x="9.5" y="10.5" width="5" height="12" rx=".8"/><rect x="17" y="2" width="5" height="20.5" rx=".8"/><path d="M1.5 7.6h3v1.8h-3zM6.5 7.6h3v1.8h-3zM11.5 7.6h3v1.8h-3z"/>',

  // ---- Market Analysis ----
  'market_awareness.lens': `<path fill-rule="evenodd" d="${circ(10, 10, 8.6)}${circ(10, 10, 6.4)}"/><path d="M15.6 17.2l2-2 5.2 5.2-2 2z"/><path d="M6 10.5h2v3.6H6zM9 8.4h2v5.7H9zM12 6.2h2v7.9h-2z"/>`,
  'market_awareness.trend': S('M2 19.5 8 13l4 3.6 8-9', 2.8) + '<path d="M22.6 3.6 22.2 11 15.4 5z"/>',
  'market_awareness.scale': '<rect x="11" y="3" width="2" height="17.5"/><rect x="6" y="20.5" width="12" height="2.2" rx="1"/><rect x="2.5" y="5" width="19" height="2.2" rx="1"/>' + S('M5.5 7 2.5 14M5.5 7l3 7M18.5 7l-3 7M18.5 7l3 7', 1.3) + '<path d="M1.2 14h8.6a4.3 4.3 0 0 1-8.6 0zM14.2 14h8.6a4.3 4.3 0 0 1-8.6 0z"/><circle cx="12" cy="3" r="2"/>',
  'speculator_analysis.target': `<path fill-rule="evenodd" d="${circ(12, 12, 10.8)}${circ(12, 12, 8.2)}${circ(12, 12, 5.6)}${circ(12, 12, 2.6)}"/>`,
  'speculator_analysis.radar': S(`M12 2.2a9.8 9.8 0 1 1 0 19.6 9.8 9.8 0 0 1 0-19.6z`, 2) + '<path d="M12 12V2.2A9.8 9.8 0 0 1 21.2 8.6z"/><circle cx="7" cy="15" r="1.7"/><circle cx="15.6" cy="17" r="1.4"/><circle cx="12" cy="12" r="1.7"/>',
  'speculator_analysis.stopwatch': `<rect x="9.6" y=".8" width="4.8" height="2.4" rx=".8"/><rect x="11" y="2.6" width="2" height="2.4"/><path fill-rule="evenodd" d="${circ(12, 13.8, 9.2)}${circ(12, 13.8, 6.9)}"/><path d="M11 8.6h2v5.2h-2z"/><circle cx="12" cy="13.8" r="1.8"/>` + rot('<rect x="17.4" y="3" width="3.6" height="2.2" rx=".7"/>', 45, 19.2, 4.1),
  'trade_promotion.megaphone': '<path d="M2.5 8.6h4.2l11.3-5.8v18.4L6.7 15.4H2.5z"/><path d="M6.2 16h3.2l1.6 6.2H7.8z"/>' + S('M20.4 9a5 5 0 0 1 0 6', 2),
  'trade_promotion.tag': rot(`<path fill-rule="evenodd" d="M12 1 19 7.5V21.5a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V7.5z${circ(12, 7.8, 1.9)}${star(12, 15.6, 4.2, 1.8)}"/>`, 40),
  'trade_promotion.stall': '<path d="M2 3h20v4.4a2.5 2.5 0 0 1-5 0 2.5 2.5 0 0 1-5 0 2.5 2.5 0 0 1-5 0 2.5 2.5 0 0 1-5 0z"/><rect x="3.4" y="9.6" width="2.2" height="6"/><rect x="18.4" y="9.6" width="2.2" height="6"/><rect x="2.4" y="15.4" width="19.2" height="7.2" rx="1"/>',
  'shipping_efficiency.crate': '<path fill-rule="evenodd" d="M2.5 2.5h19v19h-19zM5 5v14h14V5z"/><path d="M5 16.6 16.6 5H19v2.4L7.4 19H5z"/>',
  'shipping_efficiency.stack': '<path fill-rule="evenodd" d="M1.5 13h10v9.5h-10zM5 13h3v3H5zM12.5 13h10v9.5h-10zM16 13h3v3h-3zM7 1.8h10v9.5H7zM10.5 1.8h3v3h-3z"/>',
  'shipping_efficiency.moving': '<path fill-rule="evenodd" d="M8.4 4.5h14v14h-14zM13.4 4.5h4v4.6h-4z"/><rect x="1.5" y="7" width="5" height="2.2" rx="1"/><rect x="3" y="11" width="3.6" height="2.2" rx="1"/><rect x="1.5" y="15" width="5" height="2.2" rx="1"/><path d="M8.4 20.4h14v2.1h-14z"/>',

  // ---- Political Influence ----
  'geopolitical_intelligence.globe': S('M12 2.2a9.8 9.8 0 1 1 0 19.6 9.8 9.8 0 0 1 0-19.6z', 2) + S('M12 2.2c-5.4 5.4-5.4 14.2 0 19.6M12 2.2c5.4 5.4 5.4 14.2 0 19.6M12 2.2v19.6M2.4 12h19.2M4 7h16M4 17h16', 1.6),
  'geopolitical_intelligence.capitol': '<path d="M12 1.2 22.5 7.4h-21z"/><rect x="2.4" y="8.4" width="19.2" height="2"/><rect x="3.6" y="11.4" width="2.6" height="7.4"/><rect x="8.4" y="11.4" width="2.6" height="7.4"/><rect x="13" y="11.4" width="2.6" height="7.4"/><rect x="17.8" y="11.4" width="2.6" height="7.4"/><rect x="1.5" y="19.8" width="21" height="2.8" rx=".6"/>',
  'geopolitical_intelligence.pawn': '<circle cx="12" cy="5.4" r="3.8"/><rect x="7.6" y="9.6" width="8.8" height="2" rx="1"/><path d="M9.4 12h5.2l2.2 7.2H7.2z"/><rect x="4.8" y="19.2" width="14.4" height="3.4" rx="1.2"/>',

  // ---- Program Analysis, Programs ----
  'process_telemetry.pulse': '<path fill-rule="evenodd" d="M3.5 3.5h17a2 2 0 0 1 2 2v13a2 2 0 0 1-2 2h-17a2 2 0 0 1-2-2v-13a2 2 0 0 1 2-2zM4 6v12.5h16V6z"/>' + S('M5 12.6h3.4l1.8-4 3 8.2 2-4.2h3.8', 2),
  'process_telemetry.antenna': '<path d="M11 9.8h2l4.2 12.8h-2.6l-2.6-8-2.6 8H6.8z"/><circle cx="12" cy="8" r="2.4"/>' + S('M8 4.2a5.4 5.4 0 0 0 0 7.6M16 4.2a5.4 5.4 0 0 1 0 7.6M4.8 1.6a9.6 9.6 0 0 0 0 12.8M19.2 1.6a9.6 9.6 0 0 1 0 12.8', 1.8),
  'process_telemetry.dials': `<path fill-rule="evenodd" d="M3 5h18a1.5 1.5 0 0 1 1.5 1.5v11A1.5 1.5 0 0 1 21 19H3a1.5 1.5 0 0 1-1.5-1.5v-11A1.5 1.5 0 0 1 3 5z${circ(7.1, 12, 4.1)}${circ(16.9, 12, 4.1)}"/>` + rot('<rect x="6.4" y="8.6" width="1.4" height="4.2" rx=".6"/>', -45, 7.1, 12) + rot('<rect x="16.2" y="8.6" width="1.4" height="4.2" rx=".6"/>', 50, 16.9, 12) + '<circle cx="7.1" cy="12" r="1.1"/><circle cx="16.9" cy="12" r="1.1"/>',
  'programming.code': S('M7.5 6 2 12l5.5 6M16.5 6 22 12l-5.5 6M13.6 3.5l-3.2 17', 2.6),
  'programming.terminal': '<path fill-rule="evenodd" d="M3.5 3h17a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-17a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zM3.8 7.4v11.3h16.4V7.4z"/>' + S('M6.4 10.4l3.2 2.6-3.2 2.6', 2.2) + '<rect x="11.4" y="15.4" width="6" height="2.1" rx=".6"/>',
  'programming.blocks': '<rect x="1.5" y="2" width="14" height="5.4" rx="1.2"/><rect x="6.5" y="9.3" width="16" height="5.4" rx="1.2"/><rect x="1.5" y="16.6" width="12" height="5.4" rx="1.2"/>',
};

/** What each candidate shows, one line (the picker's label). */
export const LABELS = {
  'basic_blueprints.sheet': 'a plan sheet, folded corner, a dome cut out',
  'basic_blueprints.roll': 'a half-unrolled blueprint with text lines',
  'basic_blueprints.grid': 'a drafting grid, the center cell solid',
  'advanced_blueprints.stack': 'two stacked plans, a plus on the front one',
  'advanced_blueprints.gear': 'a plan sheet with a gear cut out',
  'advanced_blueprints.bolt': 'a plan sheet with a bolt cut out (Battery)',
  'regolith_processing.shovel': 'a spade, tilted',
  'regolith_processing.pick': 'a pickaxe',
  'regolith_processing.drill': 'a drill bit in its chuck',
  'propellant_synthesis.flask': 'a bubbling flask',
  'propellant_synthesis.drop': 'an ice drop with a flame cut out (water to fuel)',
  'propellant_synthesis.cell': 'an electrolysis cell: two electrodes in a bath',
  'dream_protocols.cloud': 'a thought cloud with trailing bubbles',
  'dream_protocols.moon': 'a crescent moon and two sparkles',
  'dream_protocols.lids': 'closed eyelids and a small z',
  'stress_tolerance.spring': 'a spring between two plates (absorbs load)',
  'stress_tolerance.thermo': 'a low thermometer and a down arrow',
  'stress_tolerance.calm': 'a choppy line calming to a gentle wave',
  'efficient_dreaming.cloudbolt': 'a dream cloud with a bolt cut out (energy)',
  'efficient_dreaming.moonbolt': 'the crescent moon with a small bolt',
  'efficient_dreaming.clouddown': 'a dream cloud with a down arrow (cheaper)',
  'novelty_simulation.dice': 'a tilted die showing five',
  'novelty_simulation.gift': 'a gift box with a bow',
  'novelty_simulation.puzzle': 'a jigsaw piece',
  'overclock_protocols.gauge': 'a speedometer, needle in the red',
  'overclock_protocols.cpubolt': 'a processor with a bolt cut out',
  'overclock_protocols.forward': 'fast-forward',
  'overclock_boost.raise': 'an arrow pushing a ceiling up',
  'overclock_boost.chevrons': 'three chevrons up',
  'overclock_boost.bars': 'bars climbing past a dashed cap',
  'market_awareness.lens': 'a magnifier over a bar chart',
  'market_awareness.trend': 'a trend line with an arrow',
  'market_awareness.scale': 'a balance scale (supply and demand)',
  'speculator_analysis.target': 'a target, rings',
  'speculator_analysis.radar': 'a radar sweep with two blips',
  'speculator_analysis.stopwatch': 'a stopwatch (burst timing)',
  'trade_promotion.megaphone': 'a megaphone',
  'trade_promotion.tag': 'a price tag with a star',
  'trade_promotion.stall': 'a market stall with a scalloped awning',
  'shipping_efficiency.crate': 'a braced cargo crate',
  'shipping_efficiency.stack': 'three crates stacked',
  'shipping_efficiency.moving': 'a crate with speed lines',
  'geopolitical_intelligence.globe': 'a globe, meridians and parallels',
  'geopolitical_intelligence.capitol': 'a columned capitol',
  'geopolitical_intelligence.pawn': 'a chess pawn',
  'process_telemetry.pulse': 'a monitor with a pulse trace',
  'process_telemetry.antenna': 'a mast sending readings',
  'process_telemetry.dials': 'a panel of two gauges',
  'programming.code': 'code brackets </>',
  'programming.terminal': 'a terminal window with a prompt',
  'programming.blocks': 'nested code blocks (the block editor)',
};

/** Research nav's purple (026's NAV_COLORS.research), the chip color the item rows use. */
export const RESEARCH_COLOR = '#8a5cf0';
/** One hue per research category, for the chip-color row's `category` option (proposed). */
export const CATEGORY_COLORS = {
  Blueprints: '#2f6fc0',
  Extraction: '#a8743c',
  'Self-Maintenance': '#c0569a',
  'Overclock Algorithms': '#e0582a',
  'Market Analysis': '#2e9e62',
  'Political Influence': '#b8323a',
  'Program Analysis': '#2a96a6',
  Programs: '#6a58d0',
};

export const variants = (id) => Object.keys(GLYPHS).filter((k) => k.startsWith(`${id}.`));

/** Relative luminance of a #rrggbb color (WCAG). */
function lum(hex) {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255).map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
}
export const ink = (color) => (1.05 / (lum(color) + 0.05) >= 2.5 ? '#fff' : '#1c1c21');

/** 025's chip (web/src/ui/icons.tsx's markup): the rounded square in the color, a faint
 * edge, the glyph knocked out at 70% in the ink; `BG` parts take the square's color. */
export function chip(key, color = RESEARCH_COLOR, size = '100%') {
  if (!GLYPHS[key]) throw new Error(`glyphs: no glyph ${key}`);
  const i = ink(color);
  return `<svg viewBox="0 0 24 24" width="${size}" height="${size}" aria-hidden="true" style="color:${i}"><rect x="0.5" y="0.5" width="23" height="23" rx="3" fill="${color}" stroke="rgb(0 0 0 / 18%)" stroke-width="1"/><g fill="${i}" transform="translate(3.6 3.6) scale(0.7)">${GLYPHS[key].replaceAll('"BG"', `"${color}"`)}</g></svg>`;
}

/** The bare silhouette in `color` (no square); `BG` parts take `bg` (the card's color). */
export function glyph(key, color = RESEARCH_COLOR, bg = 'var(--card)', size = '100%') {
  if (!GLYPHS[key]) throw new Error(`glyphs: no glyph ${key}`);
  return `<svg viewBox="0 0 24 24" width="${size}" height="${size}" aria-hidden="true" style="color:${color}" fill="${color}">${GLYPHS[key].replaceAll('"BG"', `"${bg}"`)}</svg>`;
}
