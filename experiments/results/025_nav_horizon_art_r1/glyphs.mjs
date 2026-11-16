// The flat glyph candidates: one solid silhouette per nav tile on a 24-unit grid, filled
// (no strokes, no outline, no inner ink), so at 18-34 px it reads as one shape in the
// tile's color. `svg(id, fill)` gives the glyph alone; `chip(id, color)` gives the tile's
// current colored square with the glyph knocked out of it in white.

const G = {
  // A domed habitat on a block, a door cut out.
  buildings: '<path d="M3 21V12h4.2A6 6 0 0 1 12 6.6a6 6 0 0 1 4.8 5.4H21v9h-7v-4h-4v4z"/><rect x="11.2" y="2" width="1.6" height="5"/>',
  // A terminal window with a prompt chevron and cursor.
  commands: '<path fill-rule="evenodd" d="M2 5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2zm3.5 4.2 1.4-1.4 4.6 4.6-4.6 4.6-1.4-1.4 3.2-3.2zM12 15h6v2h-6z"/>',
  // A rocket with fins over a pad bar.
  launch_pads: '<path d="M12 1.5c2.6 2.2 3.6 5.3 3.6 8.5v6H8.4v-6c0-3.2 1-6.3 3.6-8.5z"/><path d="M8.4 11.5 5.5 15v3h2.9zM15.6 11.5l2.9 3.5v3h-2.9z"/><path d="M10 17h4l-.8 2.6h-2.4z"/><rect x="3" y="20.5" width="18" height="2" rx="1"/>',
  // A round-bottomed flask.
  research: '<path d="M9 2h6v2h-1v5.2a7.5 7.5 0 1 1-4 0V4H9z"/>',
  // A pyramid of three stacked crates: something big being built.
  projects: '<rect x="2" y="13" width="9.4" height="9" rx="1"/><rect x="12.6" y="13" width="9.4" height="9" rx="1"/><rect x="7.3" y="2.8" width="9.4" height="9" rx="1"/>',
  // A planted flag, Apollo style, its banner held out by a top bar.
  ideologies: '<rect x="4" y="2" width="2" height="20"/><path d="M6 3h15v11H6z"/><rect x="2" y="21" width="6" height="2" rx="1"/>',
  // A crosshair: ring, cross and center dot.
  adversaries: '<path fill-rule="evenodd" d="M11 1h2v3.1A8 8 0 0 1 19.9 11H23v2h-3.1A8 8 0 0 1 13 19.9V23h-2v-3.1A8 8 0 0 1 4.1 13H1v-2h3.1A8 8 0 0 1 11 4.1zm1 5a6 6 0 1 0 0 12 6 6 0 0 0 0-12z"/><circle cx="12" cy="12" r="2.4"/>',
  // Four rising bars on a baseline.
  stats: '<rect x="3" y="14" width="3.6" height="7"/><rect x="8" y="10" width="3.6" height="11"/><rect x="13" y="6" width="3.6" height="15"/><rect x="18" y="2.5" width="3.6" height="18.5"/><rect x="2" y="21" width="20.5" height="1.6"/>',
  // An open book.
  story: '<path d="M11 6.2C8.6 4.6 5.4 4 2 4.3V19c3.4-.3 6.6.3 9 1.9zM13 6.2c2.4-1.6 5.6-2.2 9-1.9V19c-3.4-.3-6.6.3-9 1.9z"/>',
  // A sun setting on the horizon, five rays.
  retirement: '<path d="M5 17a7 7 0 0 1 14 0z"/><rect x="1" y="18.5" width="22" height="2.2" rx="1.1"/><rect x="11" y="2.5" width="2" height="4.5" rx="1"/><rect x="11" y="2.5" width="2" height="4.5" rx="1" transform="rotate(-50 12 17)"/><rect x="11" y="2.5" width="2" height="4.5" rx="1" transform="rotate(50 12 17)"/><rect x="11" y="4" width="2" height="3.5" rx="1" transform="rotate(-82 12 17)"/><rect x="11" y="4" width="2" height="3.5" rx="1" transform="rotate(82 12 17)"/>',
  // A gear with a hub hole.
  options: '<path fill-rule="evenodd" d="M10.3 1.5h3.4l.6 3a7.6 7.6 0 0 1 2.1 1.2l2.9-1 1.7 2.9-2.3 2a7.6 7.6 0 0 1 0 2.4l2.3 2-1.7 2.9-2.9-1a7.6 7.6 0 0 1-2.1 1.2l-.6 3h-3.4l-.6-3a7.6 7.6 0 0 1-2.1-1.2l-2.9 1-1.7-2.9 2.3-2a7.6 7.6 0 0 1 0-2.4l-2.3-2 1.7-2.9 2.9 1a7.6 7.6 0 0 1 2.1-1.2zM12 8.6a3.4 3.4 0 1 0 0 6.8 3.4 3.4 0 0 0 0-6.8z"/>',
};

export const GLYPHS = Object.keys(G);

export function svg(id, fill, size = '100%') {
  if (!G[id]) throw new Error(`glyphs: no glyph for ${id}`);
  return `<svg viewBox="0 0 24 24" width="${size}" height="${size}" fill="${fill}" aria-hidden="true">${G[id]}</svg>`;
}

/** Relative luminance of a #rrggbb color (WCAG). */
function lum(hex) {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255).map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
}

/** The current square in the tile color, the glyph knocked out of it at 70%: white, or
 * near-black where white falls under 2.5:1 contrast (Story's yellow, Stats' cyan,
 * Projects' green, the oranges). */
export function chip(id, color, size = '100%') {
  if (!G[id]) throw new Error(`glyphs: no glyph for ${id}`);
  const ink = 1.05 / (lum(color) + 0.05) >= 2.5 ? '#fff' : '#1c1c21';
  return `<svg viewBox="0 0 24 24" width="${size}" height="${size}" aria-hidden="true"><rect x="0.5" y="0.5" width="23" height="23" rx="3" fill="${color}" stroke="rgb(0 0 0 / 18%)" stroke-width="1"/><g fill="${ink}" transform="translate(3.6 3.6) scale(0.7)">${G[id]}</g></svg>`;
}
