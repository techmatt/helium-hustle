// The nav grid's tiles (Left.tsx): the open ones, packed from the top in `content/nav.json`'s
// order with no reserved cells (Matt, 2026-11-26). The utility tiles (Stats, Story,
// Achievements, Options) are that file's last rows, so they follow the last open play tile and
// move down as panels open.

/** The tiles shown, in order. */
export function navSlots<T extends { visible: boolean }>(tiles: readonly T[]): T[] {
  return tiles.filter((t) => t.visible);
}
