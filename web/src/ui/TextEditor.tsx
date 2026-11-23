// The program text editor: a takeover view on one program as plain text (one statement a
// line, `if` bodies indented, `#` notes), which is also its copy and paste import and
// export. The engine parses the text (`parse_program_text`) and writes it
// (`program_text`); this file never parses. The draft re-parses as it is typed, with each
// line's status in the gutter, and is saved (`set_program`) only on Apply or on leaving
// (closing, Esc, a nav tile, another program's tab, the page closing), so a half-typed
// line never runs.

import { useEffect, useLayoutEffect, useRef, useState } from 'preact/hooks';
import { daysPerWake } from '../clock';
import { engine, type Entry, type LineOutcome, type ParsedText, type ProgramView, type View } from '../engine';
import { act, ifFired, parseProgramText, setNotice, store, type Store } from '../store';
import { shortText } from './common';
import type { Takeover } from './Takeover';

/** Wait this long after the last keystroke before re-parsing. */
const PARSE_DEBOUNCE_MS = 80;
/** Re-parse an untouched draft at most this often as the game plays (what is unlocked
 * this run can change). */
const REPARSE_MS = 1000;

/** Drafts a leave couldn't save (the engine or the clock refused: too many lines, the run
 * just ended), by program: the next open starts from it, so leaving never loses the text. */
const unsaved = new Map<number, string>();

/** The takeover entry: opened by the program panel's Text editor button, on `program`. */
export function textEditorTakeover(program: number, setProgram: (i: number) => void): Takeover {
  return {
    id: 'text_editor',
    label: 'Text editor',
    title: 'Text editor',
    bleed: true,
    noButton: true,
    shown: () => true,
    render: (s) => <TextEditor s={s} program={program} setProgram={setProgram} />,
  };
}

function TextEditor({ s, program, setProgram }: { s: Store; program: number; setProgram: (i: number) => void }) {
  const n = s.view.programs.length;
  if (n === 0) return null;
  const p = Math.min(program, n - 1);
  // One draft per program: switching tabs unmounts it, which applies a changed draft.
  return <ProgramDraft key={p} s={s} program={p} setProgram={setProgram} />;
}

/** The program's text as the engine writes it. */
function textOf(entries: Entry[]): string {
  return engine.programText(entries);
}

/** What the draft was last synced to: the program's lines (JSON) and their text. */
interface Base {
  entries: string;
  text: string;
}

/** The engine's parse of one draft text (or why it can't be taken). */
interface Parse {
  text: string;
  result: ParsedText | null;
  error: string | null;
}

function parse(text: string): Parse {
  try {
    return { text, result: parseProgramText(text), error: null };
  } catch (e) {
    return { text, result: null, error: String(e) };
  }
}

/** Whether the draft holds work not in `base`: its text changed, and (once parsed) to
 * other lines. */
function edited(draft: string, base: Base, parsed: Parse | null): boolean {
  if (draft === base.text) return false;
  if (parsed?.text === draft && parsed.result) return JSON.stringify(parsed.result.entries) !== base.entries;
  return true;
}

/** Sends the draft's lines as the program, unless they are its lines already
 * (`set_program` restarts the pointer). Returns the program's lines after, or the error. */
function applyDraft(program: number, text: string): { entries: Entry[] } | { error: string } {
  const r = parse(text);
  if (!r.result) return { error: r.error ?? 'the text could not be read' };
  const now = store.view.programs[program]?.entries ?? [];
  if (JSON.stringify(r.result.entries) !== JSON.stringify(now)) {
    if (!act({ type: 'set_program', program, entries: r.result.entries })) {
      return { error: store.summary ? 'the run ended' : (store.notice ?? 'not applied') };
    }
  }
  return { entries: store.view.programs[program]?.entries ?? [] };
}

function ProgramDraft({ s, program, setProgram }: { s: Store; program: number; setProgram: (i: number) => void }) {
  const v = s.view;
  const p = v.programs[program];
  const currentJson = JSON.stringify(p.entries);
  const [base, setBase] = useState<Base>(() => ({ entries: currentJson, text: textOf(p.entries) }));
  // The draft lives here, not in the store: the game redraws every day, the text and caret stay.
  const [draft, setDraft] = useState(() => {
    const kept = unsaved.get(program);
    unsaved.delete(program);
    return kept ?? base.text;
  });
  const [parsed, setParsed] = useState<Parse | null>(null);
  const [applyError, setApplyError] = useState<string | null>(null);
  const area = useRef<HTMLTextAreaElement>(null);
  const gutter = useRef<HTMLDivElement>(null);
  const notes = useRef<HTMLDivElement>(null);
  const lastParse = useRef(0);
  // The latest of each, for the unmount and page-close handlers.
  const latest = useRef({ draft, base, parsed });
  latest.current = { draft, base, parsed };

  const reparse = (text: string) => {
    lastParse.current = performance.now();
    setParsed(parse(text));
  };

  // Parse on input, debounced (the first parse at once).
  useEffect(() => {
    if (parsed === null) {
      reparse(draft);
      return;
    }
    const t = window.setTimeout(() => reparse(draft), PARSE_DEBOUNCE_MS);
    return () => window.clearTimeout(t);
  }, [draft]);

  // As the game plays, re-parse now and then: a line may unlock (or lock, a new run).
  useEffect(() => {
    if (performance.now() - lastParse.current > REPARSE_MS) reparse(latest.current.draft);
  }, [s.version]);

  // The program changed from outside (a load_program line, a loaded save): an untouched
  // draft follows it; an edited one is kept, and Apply replaces the program with it.
  useEffect(() => {
    if (currentJson === base.entries) return;
    if (edited(draft, base, parsed)) return;
    const text = textOf(p.entries);
    setBase({ entries: currentJson, text });
    setDraft(text);
  }, [currentJson]);

  // Leaving saves: closing the view (✕, Esc, a nav tile), another program's tab, or the
  // page closing.
  useEffect(() => {
    const leave = () => {
      const l = latest.current;
      if (edited(l.draft, l.base, l.parsed)) {
        const r = applyDraft(program, l.draft);
        // Shown in the notice line once the view is gone.
        if ('error' in r) {
          unsaved.set(program, l.draft);
          setNotice(`Text editor: Program ${program + 1} not saved (${r.error}); the text is kept there.`);
        } else latest.current.base = { entries: JSON.stringify(r.entries), text: l.draft };
      }
    };
    window.addEventListener('pagehide', leave);
    return () => {
      window.removeEventListener('pagehide', leave);
      leave();
    };
  }, []);

  const apply = () => {
    const r = applyDraft(program, draft);
    if ('error' in r) {
      setApplyError(r.error);
      return;
    }
    setApplyError(null);
    const json = JSON.stringify(r.entries);
    // The engine may hold the lines a little differently (an indent it dropped): show its text then.
    const text = parsed?.text === draft && parsed.result && JSON.stringify(parsed.result.entries) === json ? draft : textOf(r.entries);
    setBase({ entries: json, text });
    if (text !== draft) setDraft(text);
  };

  // The gutter and notes scroll with the text area.
  const sync = () => {
    const a = area.current;
    if (!a) return;
    if (gutter.current) gutter.current.scrollTop = a.scrollTop;
    if (notes.current) notes.current.scrollTop = a.scrollTop;
  };
  useLayoutEffect(sync);

  const dirty = edited(draft, base, parsed);
  const textLines = draft.split('\n');
  const result = parsed?.result ?? null;
  const statuses = result ? lineStatuses(v, s, program, p, result, textLines.length) : [];
  const red = result?.lines.filter((l) => l.status === 'red').length ?? 0;
  const amber = result?.lines.filter((l) => l.status === 'amber').length ?? 0;
  const outside = currentJson !== base.entries && dirty;

  return (
    <div class="text-editor" data-testid="text-editor" data-program={program}>
      <div class="te-bar">
        <div class="te-tabs" role="tablist">
          {v.programs.map((q, i) => (
            <button
              type="button"
              role="tab"
              key={i}
              data-testid={`text-editor-tab-${i}`}
              aria-selected={i === program}
              class={i === program ? 'active' : ''}
              title={i === program ? undefined : 'Switch program (saves this one first)'}
              onClick={() => i !== program && setProgram(i)}
            >
              {q.processors > 0 ? '● ' : ''}Program {i + 1}
            </button>
          ))}
        </div>
        <button
          type="button"
          class="te-apply"
          data-testid="text-editor-apply"
          disabled={!dirty}
          title={dirty ? 'Save the text as this program (leaving the view saves too)' : 'Nothing changed'}
          onClick={apply}
        >
          Apply
        </button>
      </div>
      <p class="te-status" data-testid="text-editor-status">
        {applyError ? (
          <span class="neg">Not applied: {applyError}</span>
        ) : parsed?.error ? (
          <span class="neg">{parsed.error}</span>
        ) : (
          <>
            {red > 0 && <span class="neg">{red} {red === 1 ? 'line' : 'lines'} unparsed</span>}
            {red > 0 && amber > 0 && ', '}
            {amber > 0 && <span class="te-amber-text">{amber} locked this run</span>}
            {(red > 0 || amber > 0) && ' · '}
            <span class="muted">
              {outside
                ? 'The program changed outside the editor: Apply replaces it with this text.'
                : dirty
                  ? 'Unsaved: Apply, or leave the view, to save.'
                  : 'Saved.'}
            </span>
          </>
        )}
      </p>
      <p class="te-help muted small">
        One line per statement, the words the block editor shows; the lines of an <code>if</code> indented under it; <code>#</code> starts a
        note. Copy and paste the text to share a program.
      </p>
      <div class="te-editor">
        <div class="te-gutter" ref={gutter} aria-hidden="true">
          <div class="te-col">
            {textLines.map((_, k) => {
              const st = statuses[k];
              return (
                <div key={k} class="te-row">
                  <span
                    class={`te-dot te-${st?.dot ?? 'blank'}`}
                    data-line={k}
                    data-line-status={st?.dot ?? 'blank'}
                    data-fired={st?.fired ? `${st.fired.fired}/${st.fired.of}` : undefined}
                    title={st?.title}
                  />
                </div>
              );
            })}
          </div>
        </div>
        <textarea
          ref={area}
          class="te-area"
          data-testid="text-editor-area"
          aria-label={`Program ${program + 1} as text`}
          wrap="off"
          spellcheck={false}
          autocomplete="off"
          value={draft}
          onInput={(ev) => {
            setApplyError(null);
            setDraft(ev.currentTarget.value);
          }}
          onScroll={sync}
          onKeyDown={(ev) => {
            // Tab indents (two spaces) rather than leaving the field; Esc still closes the view.
            if (ev.key === 'Tab' && !ev.shiftKey && !ev.ctrlKey && !ev.altKey && !ev.metaKey) {
              ev.preventDefault();
              // insertText keeps the browser's undo; it fires the input event itself.
              if (!document.execCommand('insertText', false, '  ')) {
                const a = ev.currentTarget;
                a.setRangeText('  ', a.selectionStart, a.selectionEnd, 'end');
                setDraft(a.value);
              }
            }
          }}
        />
        <div class="te-notes" ref={notes}>
          <div class="te-col">
            {textLines.map((_, k) => {
              const st = statuses[k];
              return (
                <div key={k} class={`te-row te-note te-note-${st?.dot ?? 'blank'}`} title={st?.note ?? undefined}>
                  {st?.note}
                  {st?.fired && (
                    <span class="te-fired" data-fired={`${st.fired.fired}/${st.fired.of}`}>
                      {st.note ? ' · ' : ''}fired {st.fired.fired}/{st.fired.of}
                    </span>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}

/** A gutter dot: ran (green), skipped by its `if` (grey), won't parse (red), not unlocked
 * this run (amber), failed (a hollow red ring), not run (an empty ring), a note (none). */
type Dot = 'green' | 'grey' | 'red' | 'amber' | 'failed' | 'none' | 'note' | 'blank';

interface LineStatus {
  dot: Dot;
  /** The notes column's text: why it is red, amber or failed. */
  note: string | null;
  title: string;
  /** An `if` line's days true of the last few. */
  fired: { fired: number; of: number } | null;
}

const DOT_OF: Record<LineOutcome, Dot> = {
  ran: 'green',
  done: 'green',
  true: 'green',
  skipped: 'green',
  jumped: 'grey',
  false: 'grey',
  failed: 'failed',
};

const DOT_TITLE: Record<Dot, string> = {
  green: 'Ran on the last day',
  grey: 'Skipped by its if on the last day',
  red: "Doesn't parse",
  amber: 'Valid, but not unlocked yet this run',
  failed: 'Failed on the last day',
  none: 'Not run on the last day',
  note: 'A note',
  blank: '',
};

/** Each text line's status: red and amber from the parse; an ok line takes the outcome of
 * the program line it matches (the draft's lines may differ from the program's). */
function lineStatuses(v: View, s: Store, program: number, p: ProgramView, r: ParsedText, count: number): (LineStatus | null)[] {
  const map = matchLines(r.entries, p.entries);
  // Above 10x one redraw covers several days: each line's outcome over them, as the block editor.
  const span = daysPerWake(s.speed) > 1;
  const outcomes = span ? (s.lines?.lines[program] ?? []) : p.outcomes;
  const out: (LineStatus | null)[] = [];
  for (let k = 0; k < count; k++) {
    const l = r.lines[k];
    if (!l || l.status === 'blank') {
      out.push(null);
      continue;
    }
    if (l.status === 'note') {
      out.push({ dot: 'note', note: null, title: DOT_TITLE.note, fired: null });
      continue;
    }
    if (l.status === 'red' || l.status === 'amber') {
      const dot: Dot = l.status;
      const note = l.reason ? (dot === 'amber' ? `locked: ${l.reason}` : l.reason) : null;
      out.push({ dot, note, title: `${DOT_TITLE[dot]}${l.reason ? `: ${l.reason}` : ''}`, fired: null });
      continue;
    }
    const j = l.entry === null ? null : map[l.entry];
    const outcome = j === null || j === undefined ? null : (outcomes[j] ?? null);
    const dot: Dot = outcome ? DOT_OF[outcome] : 'none';
    const short = dot === 'failed' && j !== null && j !== undefined ? (p.shorts[j] ?? null) : null;
    const note = dot === 'failed' ? (short ? `failed: ${shortText(v, short)}` : 'failed') : null;
    const isIf = l.entry !== null && r.entries[l.entry]?.op === 'if';
    const fired = isIf && j !== null && j !== undefined ? ifFired(program, j) : null;
    out.push({ dot, note, title: note ?? DOT_TITLE[dot], fired });
  }
  return out;
}

/** For each draft entry, the program line it is (null: new or changed): by index when the
 * lists are equal, else a longest common subsequence over the entries' JSON. */
function matchLines(draft: Entry[], prog: Entry[]): (number | null)[] {
  const a = draft.map((e) => JSON.stringify(e));
  const b = prog.map((e) => JSON.stringify(e));
  if (a.length === b.length && a.every((x, i) => x === b[i])) return a.map((_, i) => i);
  const n = a.length;
  const m = b.length;
  // len[i][j]: the LCS of a[i..] and b[j..].
  const len: number[][] = Array.from({ length: n + 1 }, () => new Array<number>(m + 1).fill(0));
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      len[i][j] = a[i] === b[j] ? len[i + 1][j + 1] + 1 : Math.max(len[i + 1][j], len[i][j + 1]);
    }
  }
  const out: (number | null)[] = new Array(n).fill(null);
  let i = 0;
  let j = 0;
  while (i < n && j < m) {
    if (a[i] === b[j]) {
      out[i] = j;
      i++;
      j++;
    } else if (len[i + 1][j] >= len[i][j + 1]) i++;
    else j++;
  }
  return out;
}
