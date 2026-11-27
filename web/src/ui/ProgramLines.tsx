// A program's lines as live blocks: the right column's list (Right.tsx) and the Programs
// view's Blocks mode (Programs.tsx, with a run gutter, a selected row and a repeat-count
// field). Each row's editors are ProgramLogic.tsx's; every edit goes to the engine at once.

import { useRef, useState } from 'preact/hooks';
import type { Entry } from '../engine';
import { daysPerWake } from '../clock';
import type { Store } from '../store';
import { ShortNote, shortText } from './common';
import { RunGutter } from './Gutter';
import { LogicBody, OUTCOME_TEXT, ProgramLine, lineText } from './ProgramLogic';
import { canIndent, moveLines, removeLine, setIndent } from './programEdit';
import { editProgram } from './programStore';

/** The Blocks mode's extras over the right column's list: a selected row (the instruction
 * panel adds below it), the run gutter and a number field for repeat counts. */
interface BlockMode {
  selected: number;
  select: (i: number) => void;
}

/** A program's lines as blocks, live: the right column's list, and the Programs view's
 * Blocks mode (with `blocks`). Edits go to the engine at once. */
export function ProgramLines({ s, program, blocks }: { s: Store; program: number; blocks?: BlockMode }) {
  const v = s.view;
  const p = v.programs[program];
  const [drag, setDrag] = useState<number | null>(null);
  // Lines that have shown a note ("needs …") since this program's lines last changed keep
  // the note's row, so a note coming and going day to day moves no line below it.
  const noted = useRef<{ key: string; lines: Set<number> }>({ key: '', lines: new Set() });
  if (!p) return null;
  const entries = p.entries;
  const notedKey = `${v.run}:${program}:${JSON.stringify(entries)}`;
  if (noted.current.key !== notedKey) noted.current = { key: notedKey, lines: new Set() };
  // Above 10x one redraw covers several days: the pointer and "ran today" of the last
  // of them would hop around the queue (even backward), so each line instead shows what
  // it did over those days, with no pointer.
  const span = daysPerWake(s.speed) > 1;
  const spanOutcomes = s.lines?.lines[program];
  const move = (from: number, to: number) => {
    if (from === to) return;
    editProgram(program, (now) => moveLines(now, from, to));
  };
  const setLine = (i: number, e: Entry) => editProgram(program, (now) => now.map((x, j) => (j === i ? e : x)));
  const remove = (i: number) => editProgram(program, (now) => removeLine(now, i));
  const bump = (i: number, by: number) =>
    editProgram(program, (now) => now.map((x, j) => (j === i && x.op === 'command' ? { ...x, count: Math.max(1, x.count + by) } : x)));
  const setCount = (i: number, n: number) =>
    editProgram(program, (now) => now.map((x, j) => (j === i && x.op === 'command' ? { ...x, count: Math.max(1, n) } : x)));
  return (
      <ol class={`lines${blocks ? ' block-lines' : ''}`} data-testid={blocks ? 'block-lines' : undefined}>
        {entries.map((e, i) => {
          const outcome = span ? (spanOutcomes?.[i] ?? null) : p.outcomes[i];
          // No processor running the program: no line runs next, so none is framed.
          const current = !span && p.running > 0 && i === p.entry;
          // Valid but not unlocked this run: kept, dimmed, skipped by the engine.
          const locked = p.locked?.[i] ?? null;
          // Part way through its runs: that share; otherwise full once it ran today.
          const sel = blocks?.selected === i;
          const rowClass = `line ${outcome ? `out-${outcome}` : ''} ${current ? 'current' : ''} ${e.indented ? 'indented' : ''} ${locked ? 'is-locked' : ''}${sel ? ' selected' : ''}`;
          const gutter = blocks && <RunGutter s={s} program={program} j={i} />;
          const dragProps = {
            draggable: true,
            onDragStart: (ev: DragEvent) => {
              // Only the row itself: text dragged out of a field in it moves no line.
              if (ev.target !== ev.currentTarget) return;
              setDrag(i);
              ev.dataTransfer?.setData('text/plain', String(i));
            },
            // A drag let go off the list (or cancelled) is over: a later drop of anything
            // else on a line must not move the line it started from.
            onDragEnd: () => setDrag(null),
            onDragOver: (ev: DragEvent) => ev.preventDefault(),
            onDrop: (ev: DragEvent) => {
              ev.preventDefault();
              if (drag !== null) move(drag, i);
              setDrag(null);
            },
            // Blocks mode: a click picks the row the instruction panel adds below.
            onClick: blocks ? () => blocks.select(i) : undefined,
            'data-selected': sel ? 'true' : undefined,
          };
          const ranText = span
            ? outcome
              ? `last run since the last redraw: ${OUTCOME_TEXT[outcome]}`
              : 'not run since the last redraw'
            : outcome
              ? `last run today: ${OUTCOME_TEXT[outcome]}`
              : 'not run today';
          // In or out of the `if` or `else` above (a drag onto a body line does the same).
          const indent = e.op !== 'if' && e.op !== 'else' && (e.indented || canIndent(entries, i)) && (
            <button
              type="button"
              class="tiny"
              data-testid={e.indented ? 'line-outdent' : 'line-indent'}
              title={e.indented ? 'Move out of the block' : 'Move into the block before it'}
              onClick={() => editProgram(program, (now) => setIndent(now, i, !now[i]?.indented))}
            >
              {e.indented ? '⇤' : '⇥'}
            </button>
          );
          if (e.op !== 'command') {
            const why = outcome === 'failed' ? (p.shorts[i] ?? null) : null;
            // A note, an unparsed line (from the text editor) or an else never runs: no outcome.
            const kept = e.op === 'note' ? 'note-line' : e.op === 'unparsed' ? 'unparsed-line' : e.op === 'else' ? 'else-line' : '';
            const error = e.op === 'unparsed' ? (p.errors?.[i] ?? null) : null;
            const tail = error ? `: doesn't parse: ${error}` : locked ? `: locked this run: ${locked}` : kept ? '' : `: ${ranText}${why ? `. ${shortText(v, why)}` : ''}`;
            return (
              <li key={`${i}-${e.op}`} class={`${rowClass} logic ${kept}`} data-line={i} data-op={e.op} {...dragProps} title={`${lineText(v, e)}${tail}`}>
                {gutter}
                <span class="grip" aria-hidden>
                  ⠿
                </span>
                <LogicBody v={v} e={e} now={p.conditions[i] ?? null} set={(x) => setLine(i, x)} />
                <span class="line-buttons">
                  {indent}
                  <button type="button" class="tiny" title="Remove" onClick={() => remove(i)}>
                    ×
                  </button>
                </span>
                {error !== null ? (
                  <span class="line-why" data-line-why>
                    {error}
                  </span>
                ) : locked !== null ? (
                  <span class="line-why line-locked" data-line-locked>
                    locked this run: {locked}
                  </span>
                ) : (
                  why && (
                    <span class="line-why" data-line-why>
                      <ShortNote v={v} s={why} />
                    </span>
                  )
                )}
              </li>
            );
          }
          return (
            <ProgramLine
              key={`${i}-${e.command}`}
              v={v}
              e={e}
              i={i}
              outcome={outcome}
              current={current}
              step={p.step}
              short={p.shorts[i] ?? null}
              locked={locked}
              ranText={ranText}
              rowClass={rowClass}
              drag={dragProps}
              indent={indent}
              noted={noted.current.lines}
              gutter={gutter}
              setCount={blocks ? (n) => setCount(i, n) : undefined}
              bump={(by) => bump(i, by)}
              remove={() => remove(i)}
            />
          );
        })}
      </ol>
  );
}
