// The play log: everything that happens in a career, as JSON lines, sent to the dev
// server's `/__playlog` endpoint (vite.config.ts), which appends them to one file per
// career on disk (web/README.md, *Play logs*; `hh-sim playlog` replays and measures it).
//
// The first line is the header: format, career id, rules revision, config hash and the
// config itself, seed, scenario, and any actions taken before the log began; the server
// stamps it with the repo commit. Every other line has `t` (wall-clock ms), `kind`,
// `tick`, `run`, `day` and `speed`. `action` lines are the game's record (the action and
// the tick it applied at), so the log replays the career exactly; `checkpoint` lines
// carry the state's fingerprint to check the replay against.

import type { Scenario, Stamped } from './engine';

export const FORMAT = 'hh-playlog/1';
const ENDPOINT = '/__playlog';
/** Send queued lines at most this often (and when the page is hidden or closed). */
const FLUSH_MS = 2000;
/** Lines kept while the server can't be reached (oldest dropped past this). */
const MAX_QUEUED = 50_000;

export interface Where {
  tick: number;
  run: number;
  day: number;
  speed: number;
}

export interface Header {
  career: string;
  rules_revision: number;
  config_hash: string;
  engine: string;
  config: unknown;
  seed: string;
  scenario: Scenario | null;
  /** Actions taken before this log began (a career joined mid-way). */
  prior_actions: Stamped[];
}

type Line = Record<string, unknown>;

let career: string | null = null;
let queue: { career: string; line: string }[] = [];
let sending = false;
/** `on`: the server took the last lines; `off`: there is no log server (a static build). */
export let status: 'on' | 'off' | 'pending' = 'pending';
export let logDir = '';
let where: () => Where = () => ({ tick: 0, run: 0, day: 0, speed: 0 });

/** A career id, also its file name: start time (local), a random tag (two careers can
 * start in the same second) and the seed. */
export function newCareerId(seed: string): string {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, '0');
  const stamp = `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}_${p(d.getHours())}-${p(d.getMinutes())}-${p(d.getSeconds())}`;
  const tag = Array.from(crypto.getRandomValues(new Uint8Array(3)), (b) => b.toString(16).padStart(2, '0')).join('');
  // A browser under automation (the e2e) is no player: the server files its logs apart.
  return `${navigator.webdriver ? 'bot_' : ''}${stamp}_${tag}_seed-${seed}`;
}

/** Where the game stands now, for every line. */
export function locate(f: () => Where): void {
  where = f;
}

export function currentCareer(): string | null {
  return career;
}

/** Starts (or, for a saved career, resumes) logging to `id`. A new log gets a header. */
export function open(id: string, header: Header | null): void {
  career = id;
  if (header) push({ kind: 'header', format: FORMAT, ...header, t: Date.now() }, id);
}

/** Appends a line to a career's log (the current one unless `to` is given). */
export function log(kind: string, extra: Line = {}, to: string | null = career): void {
  if (!to) return;
  const w = where();
  push({ t: Date.now(), kind, tick: w.tick, run: w.run, day: w.day, speed: w.speed, ...extra }, to);
}

/** Every line's `id` (`<page load>:<n>`): a line can reach the server twice (a send
 * the page saw fail but the server took, then sent again), and readers drop repeats. */
const pageId = Array.from(crypto.getRandomValues(new Uint8Array(4)), (b) => b.toString(16).padStart(2, '0')).join('');
let seq = 0;

function push(line: Line, to: string): void {
  line.id = `${pageId}:${seq++}`;
  queue.push({ career: to, line: JSON.stringify(line) });
  if (queue.length > MAX_QUEUED) queue.splice(0, queue.length - MAX_QUEUED);
}

/** The queued lines, grouped by career in order. */
function batches(lines: { career: string; line: string }[]): { career: string; lines: string[] }[] {
  const out: { career: string; lines: string[] }[] = [];
  for (const l of lines) {
    const last = out[out.length - 1];
    if (last && last.career === l.career) last.lines.push(l.line);
    else out.push({ career: l.career, lines: [l.line] });
  }
  return out;
}

/** Back on the queue, ahead of anything queued since: a batch the server didn't take. */
function requeue(b: { career: string; lines: string[] }): void {
  queue = b.lines.map((line) => ({ career: b.career, line })).concat(queue);
}

/** Sends queued lines, one request at a time, so the file keeps their order. Only a
 * batch the server didn't take is sent again (never one it appended). `keepalive` lets
 * the send finish if the page is closing (small batches only). */
export async function flush(keepalive = false): Promise<void> {
  if (sending || queue.length === 0 || status === 'off') return;
  sending = true;
  const todo = batches(queue);
  queue = [];
  try {
    while (todo.length > 0) {
      const b = todo[0];
      const body = JSON.stringify(b);
      let r: Response;
      try {
        r = await fetch(ENDPOINT, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body,
          keepalive: keepalive && body.length < 60_000,
        });
      } catch {
        break; // unreachable now: this batch and the rest wait for the next try
      }
      if (r.status === 404) {
        status = 'off';
        todo.length = 0;
        break;
      }
      todo.shift();
      if (r.ok) status = 'on';
      else console.warn(`play log: the server refused a batch (${r.status}); dropped`);
    }
  } finally {
    for (const b of todo.reverse()) requeue(b);
    sending = false;
  }
}

/** On leaving the page: whatever is queued goes in beacons (they outlive the page; a
 * send still in flight may land after them, so readers order lines by tick and time). */
export function flushOnLeave(): void {
  if (queue.length === 0 || status === 'off') return;
  const todo = batches(queue);
  queue = [];
  for (const b of todo) {
    const ok = navigator.sendBeacon?.(ENDPOINT, new Blob([JSON.stringify(b)], { type: 'application/json' }));
    if (!ok) queue = queue.concat(b.lines.map((line) => ({ career: b.career, line })));
  }
}

/** Asks the server where logs go (shown in Options), and starts the flush timer. */
export async function start(): Promise<void> {
  try {
    const r = await fetch(ENDPOINT);
    if (r.status === 404) status = 'off';
    else if (r.ok) logDir = ((await r.json()) as { dir: string }).dir;
  } catch {
    status = 'off';
  }
  window.setInterval(() => void flush(), FLUSH_MS);
}
