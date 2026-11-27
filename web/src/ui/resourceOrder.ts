// The Resources panel's order, the one every resource list on a card follows (cost rows,
// flow lines, project costs, command tooltips). Display order only: the panel's first group
// in its fixed order, then every other resource in the engine's order (the view's
// `resources`), then anything not a resource, as given.

/** The Resources panel's first group, in this order; every other resource follows in a
 * second group, in the engine's order. */
export const FIRST_GROUP = ['energy', 'credits', 'land', 'boredom', 'processors'];

/** A rank per resource id, lowest first, from the view's resources in engine order. */
export function resourceRanks(resources: readonly { id: string }[]): Map<string, number> {
  const rank = new Map<string, number>();
  FIRST_GROUP.forEach((id, i) => rank.set(id, i));
  for (const r of resources) if (!rank.has(r.id)) rank.set(r.id, rank.size);
  return rank;
}

/** `items` sorted into the panel's order by `key` (stable: ties and unknown ids keep their
 * order, unknowns last). */
export function inPanelOrder<T>(resources: readonly { id: string }[], items: readonly T[], key: (t: T) => string): T[] {
  const rank = resourceRanks(resources);
  const at = (t: T) => rank.get(key(t)) ?? Number.MAX_SAFE_INTEGER;
  return items
    .map((t, i) => [t, i] as const)
    .sort(([a, i], [b, j]) => at(a) - at(b) || i - j)
    .map(([t]) => t);
}

/** A resource-keyed record's entries in the panel's order. */
export function panelEntries<V>(resources: readonly { id: string }[], amounts: Record<string, V>): [string, V][] {
  return inPanelOrder(resources, Object.entries(amounts), ([id]) => id);
}
