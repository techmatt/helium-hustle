// The run gutter both Programs modes share: one status dot per line (ran, skipped by its
// `if`, failed, not run, locked, unparsed), and on a repeated command line one small dot
// per run of its pass (green done, red failed, gray skipped at its cap, hollow pending,
// filling left to right) up
// to 8 runs, a short bar with "done/total, N failed" past that. The Text mode draws it
// beside its text lines (TextEditor.tsx), the Blocks mode first in each row
// (ProgramLines.tsx). Everything here is the view's.

import { daysPerWake } from '../clock';
import type { LineOutcome, ProgramView, Repeat, View } from '../engine';
import { ifFired, type Store } from '../store';
import { shortText } from './common';

/** A status dot: ran (green), skipped by its `if` or at its cap (gray), won't parse (red), not unlocked
 * this run (amber), failed (a hollow red ring), not run (an empty ring), a note (none). */
type Dot = 'green' | 'gray' | 'red' | 'amber' | 'failed' | 'none' | 'note' | 'blank';

export interface LineStatus {
  dot: Dot;
  /** The notes column's text: why it is red, amber or failed. */
  note: string | null;
  title: string;
  /** An `if` line's days true of the last few. */
  fired: { fired: number; of: number } | null;
  /** A repeated command line's pass: its runs as dots or a bar. */
  repeat: RepeatState | null;
}

/** A repeated line's pass as the gutter draws it. */
interface RepeatState {
  /** The line's repeat count. */
  of: number;
  done: number;
  failed: number;
  /** Bit k: run k (of the first 8) failed. */
  bits: number;
  /** Runs skipped at their cap, and bit k: run k (of the first 8) skipped (gray). */
  skipped: number;
  skips: number;
}

const DOT_OF: Record<LineOutcome, Dot> = {
  ran: 'green',
  done: 'green',
  true: 'green',
  // Didn't run: its output would pass its cap (gray, as the one-gutter ruling's "skipped").
  skipped: 'gray',
  jumped: 'gray',
  false: 'gray',
  failed: 'failed',
};

/** An `else` line's gutter title: it never runs, like a note. */
export const ELSE_TITLE = 'Else: free; its if decides which body runs';

export const DOT_TITLE: Record<Dot, string> = {
  green: 'Ran on the last day',
  gray: 'Skipped by its if on the last day',
  red: "Doesn't parse",
  amber: 'Valid, but not unlocked yet this run',
  failed: 'Failed on the last day',
  none: 'Not run on the last day',
  note: 'A note',
  blank: '',
};

/** A repeated line's pass: the one in progress at the pointer, else its last one, whole
 * (a line the pointer loops straight back to still shows its finished pass); only a
 * partial record at the pointer's start (one a save took mid-pass) shows hollow. */
function repeatOf(p: ProgramView, j: number, of: number): RepeatState {
  const rep: Repeat | null = p.repeats?.[j] ?? null;
  const atStart = p.running > 0 && p.entry === j && p.step === 0;
  if (!rep || (atStart && rep.done < of)) return { of, done: 0, failed: 0, bits: 0, skipped: 0, skips: 0 };
  return { of, done: Math.min(rep.done, of), failed: rep.failed, bits: rep.first_fails, skipped: rep.skipped ?? 0, skips: rep.first_skips ?? 0 };
}

/** A line that never runs: a note, or an `else` (with ELSE_TITLE). */
export function noteStatus(title: string = DOT_TITLE.note): LineStatus {
  return { dot: 'note', note: null, title, fired: null, repeat: null };
}

/** A red (unparsed) or amber (locked) line, and why: the notes column says the reason
 * (amber's as `locked: …`), the title adds it to the dot's; no reason, a bare title. */
export function flaggedStatus(dot: 'red' | 'amber', reason: string | null): LineStatus {
  const note = reason !== null ? (dot === 'amber' ? `locked: ${reason}` : reason) : null;
  return { dot, note, title: `${DOT_TITLE[dot]}${reason !== null ? `: ${reason}` : ''}`, fired: null, repeat: null };
}

/** Program line `j`'s status in the gutter: locked or unparsed as the view says, else
 * what it did on the last day (or over the redraw's days above 10x). */
export function entryStatus(v: View, s: Store, program: number, j: number): LineStatus {
  const p = v.programs[program];
  const e = p.entries[j];
  if (e.op === 'note') return noteStatus();
  const error = e.op === 'unparsed' ? (p.errors?.[j] ?? "doesn't parse") : null;
  if (error !== null) return flaggedStatus('red', error);
  // Amber as the Text mode's parse says it (an `else` under a locked `if` too).
  const locked = p.locked?.[j] ?? null;
  if (locked !== null) return flaggedStatus('amber', locked);
  // An `else` line never runs (its `if` decides): free, like a note.
  if (e.op === 'else') return noteStatus(ELSE_TITLE);
  const span = daysPerWake(s.speed) > 1;
  const outcomes = span ? (s.lines?.lines[program] ?? []) : p.outcomes;
  const outcome = outcomes[j] ?? null;
  const dot: Dot = outcome ? DOT_OF[outcome] : 'none';
  const short = dot === 'failed' ? (p.shorts[j] ?? null) : null;
  const note =
    dot === 'failed' ? (short ? `failed: ${shortText(v, short)}` : 'failed') : outcome === 'skipped' ? 'skipped: its output would pass its cap' : null;
  const fired = e.op === 'if' ? ifFired(program, j) : null;
  const repeat = e.op === 'command' && e.count > 1 ? repeatOf(p, j, e.count) : null;
  return { dot, note, title: note ?? DOT_TITLE[dot], fired, repeat };
}

/** Most runs drawn as dots; past it, a bar. */
const MAX_DOTS = 8;

/** One line's gutter: its status dot, or its runs on a repeated line. */
export function GutterMark({ st, line }: { st: LineStatus | null; line: number }) {
  const r = st?.repeat;
  // A red or amber line shows its status, never its runs.
  if (r && st && st.dot !== 'red' && st.dot !== 'amber') {
    // Not run on the last day (jumped with its `if`, or not reached): the last pass, muted.
    // A gray with a note (skipped at its cap) was reached today: its pass is current.
    const stale = (st.dot === 'gray' && st.note === null) || st.dot === 'none' ? ' rep-stale' : '';
    const title = `${r.done}/${r.of}${r.failed > 0 ? `, ${r.failed} failed` : ''}${r.skipped > 0 ? `, ${r.skipped} skipped at the cap` : ''}${stale ? ' (an earlier pass)' : ''}`;
    if (r.of <= MAX_DOTS) {
      return (
        <span class={`rep-dots${stale}`} data-line={line} data-line-status={st.dot} data-repeat={`${r.done}/${r.of}`} data-repeat-failed={r.failed} data-repeat-skipped={r.skipped} title={title}>
          {Array.from({ length: r.of }, (_, k) => (
            <span key={k} class={`rep-dot ${k >= r.done ? 'rep-pending' : r.bits & (1 << k) ? 'rep-failed' : r.skips & (1 << k) ? 'rep-skipped' : 'rep-done'}`} />
          ))}
        </span>
      );
    }
    const failed = Math.min(r.failed, r.done);
    const skipped = Math.min(r.skipped, r.done - failed);
    const ok = r.done - failed - skipped;
    return (
      <span class={`rep-bar${stale}`} data-line={line} data-line-status={st.dot} data-repeat={`${r.done}/${r.of}`} data-repeat-failed={r.failed} data-repeat-skipped={r.skipped} title={title}>
        <span class="rep-bar-ok" style={{ width: `${(100 * ok) / r.of}%` }} />
        <span class="rep-bar-skipped" style={{ width: `${(100 * skipped) / r.of}%` }} />
        <span class="rep-bar-failed" style={{ width: `${(100 * failed) / r.of}%` }} />
      </span>
    );
  }
  return (
    <span
      class={`te-dot te-${st?.dot ?? 'blank'}`}
      data-line={line}
      data-line-status={st?.dot ?? 'blank'}
      data-fired={st?.fired ? `${st.fired.fired}/${st.fired.of}` : undefined}
      title={st?.title}
    />
  );
}

/** The Blocks mode's gutter cell for program line `j`. */
export function RunGutter({ s, program, j }: { s: Store; program: number; j: number }) {
  const st = entryStatus(s.view, s, program, j);
  return (
    <span class="block-gutter" aria-hidden="true">
      <GutterMark st={st} line={j} />
    </span>
  );
}
