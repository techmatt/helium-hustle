// The nav grid's cells (Left.tsx): one per tile of `content/nav.json`, in its order, up to the
// last tile shown. A tile not shown yet holds its cell empty, so a panel that opens fills its own
// cell and no other tile moves (109). Tiles past the last shown one take no cell.

/** Each cell's tile, or null for a cell held empty. */
export function navSlots<T extends { visible: boolean }>(tiles: readonly T[]): (T | null)[] {
  let end = tiles.length;
  while (end > 0 && !tiles[end - 1].visible) end--;
  return tiles.slice(0, end).map((t) => (t.visible ? t : null));
}
