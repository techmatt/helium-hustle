// The wall clock, which the UI owns (the engine only counts ticks). At 1x one tick (one
// in-game day) passes per second. On each timer wake the clock turns the wall time
// since the last wake into ticks at the current speed and hands them over at once, so
// a throttled background tab catches up on its next wake. Nothing runs while the page
// is closed: there is no offline progress.

export const SPEEDS = [0, 1, 3, 10, 50, 200] as const;
export type Speed = (typeof SPEEDS)[number];

/** Ticks per second at 1x. */
export const TICKS_PER_SECOND = 1;
/** How often the clock asks to wake (browsers stretch this in background tabs). */
const WAKE_MS = 100;

export class Clock {
  speed: Speed = 1;
  private last = performance.now();
  /** Fractional ticks carried to the next wake. */
  private carry = 0;
  private timer: number | undefined;

  constructor(private readonly run: (ticks: number) => void) {}

  start(): void {
    this.last = performance.now();
    this.timer = window.setInterval(() => this.wake(), WAKE_MS);
  }

  stop(): void {
    window.clearInterval(this.timer);
  }

  /** Forgets the time since the last wake: for a page restored from the back/forward
   * cache, whose time away is offline time and must not run. */
  forgetTimeAway(): void {
    this.last = performance.now();
    this.carry = 0;
  }

  setSpeed(s: Speed): void {
    this.wake(); // settle the time at the old speed first
    this.speed = s;
    if (s === 0) this.carry = 0;
  }

  /** Runs the ticks owed for the wall time since the last wake. */
  wake(): void {
    const now = performance.now();
    const seconds = Math.max(0, now - this.last) / 1000;
    this.last = now;
    this.carry += seconds * this.speed * TICKS_PER_SECOND;
    const ticks = Math.floor(this.carry);
    if (ticks > 0) {
      this.carry -= ticks;
      this.run(ticks);
    }
  }
}
