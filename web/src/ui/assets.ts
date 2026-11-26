// Readers for Vite's import.meta.glob maps of shipped assets (src/assets/), shared by
// art.tsx, cardArt.ts and icons.tsx.

/** A glob map keyed by file stem: `../assets/quests/scales.webp` -> `scales`. */
export const byName = <T,>(files: Record<string, T>, ext: string): Record<string, T> =>
  Object.fromEntries(Object.entries(files).map(([f, x]) => [f.slice(f.lastIndexOf('/') + 1, -ext.length), x]));

/** Each SVG file's inside (what lies between its `<svg ...>` and `</svg>`), by stem. */
export const svgInner = (files: Record<string, string>): Record<string, string> =>
  Object.fromEntries(
    Object.entries(byName(files, '.svg')).map(([id, svg]) => [id, svg.slice(svg.indexOf('>') + 1, svg.lastIndexOf('</svg>'))]),
  );
