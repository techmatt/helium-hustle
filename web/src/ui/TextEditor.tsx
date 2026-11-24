// The program text editor: a takeover view on one program as plain text (one statement a
// line, `if` bodies indented, `#` notes), which is also its copy and paste import and
// export. The engine parses the text (`parse_program_text`) and writes it
// (`program_text`); this file never parses. The draft re-parses as it is typed, with each
// line's status in the gutter, and is saved (`set_program`) only on Apply or on leaving
// (closing, Esc, a nav tile, another program's tab, the page closing), so a half-typed
// line never runs.

import type { ComponentChildren } from 'preact';
import { useEffect, useLayoutEffect, useRef, useState } from 'preact/hooks';
import { engine, type Entry, type ParsedText, type ProgramView, type View } from '../engine';
import { act, parseProgramText, setNotice, store, type Store } from '../store';
import { DOT_TITLE, ELSE_TITLE, GutterMark, entryStatus, type Dot, type LineStatus } from './Gutter';
import { InstructionPanel } from './Instructions';
import { insertLine } from './programEdit';

/** Wait this long after the last keystroke before re-parsing. */
const PARSE_DEBOUNCE_MS = 80;
/** Re-parse an untouched draft at most this often as the game plays (what is unlocked
 * this run can change). */
const REPARSE_MS = 1000;

/** Drafts a leave couldn't save (the engine or the clock refused: too many lines, the run
 * just ended), by program: the next open starts from it, so leaving never loses the text. */
const unsaved = new Map<number, string>();

/** The Programs view's Text mode on `program` (Programs.tsx); `bar` is drawn in its top bar
 * (the Text | Blocks toggle). */
export function TextEditor({ s, program, setProgram, bar }: { s: Store; program: number; setProgram: (i: number) => void; bar: ComponentChildren }) {
  const n = s.view.programs.length;
  if (n === 0) return null;
  const p = Math.min(program, n - 1);
  // One draft per program: switching tabs unmounts it, which applies a changed draft.
  return <ProgramDraft key={p} s={s} program={p} setProgram={setProgram} bar={bar} />;
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

function ProgramDraft({ s, program, setProgram, bar }: { s: Store; program: number; setProgram: (i: number) => void; bar: ComponentChildren }) {
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
  // The caret's text line (the instruction panel inserts below it), and where an insert
  // puts the caret once the new draft is drawn.
  // Until the text area is clicked: the last line, so the panel appends.
  const [cursor, setCursor] = useState(() => draft.split('\n').length - 1);
  const caretAfter = useRef<number | null>(null);
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
          setNotice(`Programs: Program ${program + 1} not saved (${r.error}); the text is kept there.`);
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

  const noteCursor = () => {
    const a = area.current;
    if (a) setCursor(a.value.slice(0, a.selectionStart).split('\n').length - 1);
  };

  // An instruction from the panel: a new line below the caret's line with its indent (in
  // the body under an `if` or `else` line; an `if` or `else` itself at the top level, with
  // an empty body line under it), or on the caret's line when that is empty; the caret
  // moves to the new line (the body line for a block).
  const insert = (line: string) => {
    const a = area.current;
    const { lines, last } = insertLine((a ? a.value : draft).split('\n'), cursor, line);
    caretAfter.current = lines.slice(0, last + 1).join('\n').length;
    setApplyError(null);
    setDraft(lines.join('\n'));
    setCursor(last);
  };
  useLayoutEffect(() => {
    const a = area.current;
    const c = caretAfter.current;
    if (!a || c === null) return;
    caretAfter.current = null;
    // The caret first: focusing notes the caret's line.
    a.setSelectionRange(c, c);
    a.focus();
  }, [draft]);

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
        {bar}
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
      <div class="te-main">
      <div class="te-editor">
        <div class="te-gutter" ref={gutter} aria-hidden="true">
          <div class="te-col">
            {textLines.map((_, k) => {
              const st = statuses[k];
              return (
                <div key={k} class="te-row">
                  <GutterMark st={st ?? null} line={k} />
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
            noteCursor();
          }}
          onKeyUp={noteCursor}
          onMouseUp={noteCursor}
          onSelect={noteCursor}
          onFocus={noteCursor}
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
      {/* The debounced parse's text: the engine isn't asked on every keystroke. */}
      <InstructionPanel s={s} text={parsed?.text ?? draft} cursor={cursor} insert={insert} />
      </div>
    </div>
  );
}

/** Each text line's status: red and amber from the parse; an ok line takes the outcome of
 * the program line it matches (the draft's lines may differ from the program's). */
function lineStatuses(v: View, s: Store, program: number, p: ProgramView, r: ParsedText, count: number): (LineStatus | null)[] {
  const map = matchLines(r.entries, p.entries);
  const out: (LineStatus | null)[] = [];
  for (let k = 0; k < count; k++) {
    const l = r.lines[k];
    if (!l || l.status === 'blank') {
      out.push(null);
      continue;
    }
    // An `else` line never runs (its `if` decides): free, like a note.
    if (l.status === 'note' || (l.status === 'ok' && l.entry !== null && r.entries[l.entry]?.op === 'else')) {
      const title = l.status === 'note' ? DOT_TITLE.note : ELSE_TITLE;
      out.push({ dot: 'note', note: null, title, fired: null, repeat: null });
      continue;
    }
    if (l.status === 'red' || l.status === 'amber') {
      const dot: Dot = l.status;
      const note = l.reason ? (dot === 'amber' ? `locked: ${l.reason}` : l.reason) : null;
      out.push({ dot, note, title: `${DOT_TITLE[dot]}${l.reason ? `: ${l.reason}` : ''}`, fired: null, repeat: null });
      continue;
    }
    // The program line it is (the Blocks mode's status for it), or a new line not run yet.
    const j = l.entry === null ? null : map[l.entry];
    out.push(j === null || j === undefined ? { dot: 'none', note: null, title: DOT_TITLE.none, fired: null, repeat: null } : entryStatus(v, s, program, j));
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
