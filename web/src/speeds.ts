// The game speeds the player picks from (the clock runs them: clock.ts). Plain TypeScript,
// so the unit checks load it in Node.

/** Every speed; 200x is the fastest (Matt, docs/rulings/game.md, *Speed*). */
export const SPEEDS = [0, 1, 3, 10, 50, 200] as const;
export type Speed = (typeof SPEEDS)[number];
/** The fastest speed before Accelerated Scheduling (the view's `fast_speeds`) opens 50x and
 * 200x (Matt, 2026-11-26, amendment #3261). */
const SLOW_MAX = 10;

/** The speeds offered: 50x and 200x only once `fast` (the view's `fast_speeds`). */
export function speedsOffered(fast: boolean): readonly Speed[] {
  return fast ? SPEEDS : SPEEDS.filter((s) => s <= SLOW_MAX);
}

/** A speed as offered now: one past the slow speeds drops to 10x until `fast`. */
export function offeredSpeed(s: Speed, fast: boolean): Speed {
  return fast || s <= SLOW_MAX ? s : SLOW_MAX;
}
