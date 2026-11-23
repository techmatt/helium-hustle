// The Programs view: a takeover opened by the Programs nav tile, editing one program in
// two first-class modes on the same lines. Text (TextEditor.tsx) is the program as text;
// Blocks is the block editor (ProgramLines.tsx, with a run gutter, a selected row
// and number fields), live: every edit goes to the engine at once. Both share the All
// instructions panel (a click adds a line in Text, a block in Blocks, below the caret's or
// selected row) and the run gutter (Gutter.tsx). The view remembers the last mode; a new
// player starts in Blocks.

import { useState } from 'preact/hooks';
import { engine } from '../engine';
import { parseProgramText, type Store } from '../store';
import { InstructionPanel } from './Instructions';
import { AddLogic } from './ProgramLogic';
import { blockEnd, canAddElse, insertLine } from './programEdit';
import { editProgram } from './programStore';
import { ProgramLines } from './ProgramLines';
import type { Takeover } from './Takeover';
import { TextEditor } from './TextEditor';

export type ProgramsMode = 'text' | 'blocks';

/** Where the last mode is kept (a UI preference, not game state). */
const MODE_KEY = 'hh-programs-mode';

function savedMode(): ProgramsMode {
  try {
    return localStorage.getItem(MODE_KEY) === 'text' ? 'text' : 'blocks';
  } catch {
    return 'blocks';
  }
}

/** The takeover: opened by the Programs nav tile, on the program the right column shows. */
export function programsTakeover(program: number, setProgram: (i: number) => void): Takeover {
  return {
    id: 'programs',
    label: 'Programs',
    title: 'Programs',
    bleed: true,
    noButton: true,
    shown: () => true,
    render: (s) => <ProgramsView s={s} program={program} setProgram={setProgram} />,
  };
}

function ProgramsView({ s, program, setProgram }: { s: Store; program: number; setProgram: (i: number) => void }) {
  const [mode, setMode] = useState<ProgramsMode>(savedMode);
  const pick = (m: ProgramsMode) => {
    setMode(m);
    try {
      localStorage.setItem(MODE_KEY, m);
    } catch {
      // A private window may refuse storage: the mode still switches.
    }
  };
  const toggle = (
    <div class="mode-toggle" role="group" aria-label="Edit as">
      {(['text', 'blocks'] as const).map((m) => (
        <button type="button" key={m} data-testid={`mode-${m}`} aria-pressed={mode === m} class={mode === m ? 'active' : ''} onClick={() => mode !== m && pick(m)}>
          {m === 'text' ? 'Text' : 'Blocks'}
        </button>
      ))}
    </div>
  );
  return mode === 'text' ? <TextEditor s={s} program={program} setProgram={setProgram} bar={toggle} /> : <BlockEditor s={s} program={program} setProgram={setProgram} bar={toggle} />;
}

function BlockEditor({ s, program, setProgram, bar }: { s: Store; program: number; setProgram: (i: number) => void; bar: preact.ComponentChildren }) {
  const v = s.view;
  const n = v.programs.length;
  const p = n > 0 ? v.programs[Math.min(program, n - 1)] : null;
  // The row the instruction panel adds below: the last until one is clicked.
  const [picked, setPicked] = useState<{ program: number; row: number } | null>(null);
  if (!p) return null;
  const at = Math.min(program, n - 1);
  const entries = p.entries;
  const selected = picked && picked.program === at ? Math.min(picked.row, entries.length - 1) : entries.length - 1;
  // The program as text, one line per block (the engine's form), for the panel and inserts.
  const text = engine.programText(entries).replace(/\n$/, '');
  const insert = (line: string) => {
    // A new `if` goes after the selected row's whole block: applied at once, it must not
    // take the lines below the row into its own body.
    const below = line.startsWith('if ') ? blockEnd(entries, selected) : selected;
    const { lines, last } = insertLine(text === '' ? [] : text.split('\n'), below, line);
    const parsed = parseProgramText(lines.join('\n'));
    // The new row: the inserted line's block (an `if`'s empty body line is dropped: the `if`).
    let row = parsed.lines[last]?.entry ?? null;
    for (let k = last; row === null && k >= 0; k--) row = parsed.lines[k]?.entry ?? null;
    editProgram(at, () => parsed.entries);
    setPicked({ program: at, row: row ?? parsed.entries.length - 1 });
  };
  return (
    <div class="text-editor block-editor" data-testid="block-editor" data-program={at}>
      <div class="te-bar">
        {bar}
        <div class="te-tabs" role="tablist">
          {v.programs.map((q, i) => (
            <button
              type="button"
              role="tab"
              key={i}
              data-testid={`blocks-tab-${i}`}
              aria-selected={i === at}
              class={i === at ? 'active' : ''}
              onClick={() => i !== at && setProgram(i)}
            >
              {q.processors > 0 ? '● ' : ''}Program {i + 1}
            </button>
          ))}
        </div>
      </div>
      <div class="te-main">
        <div class="block-main">
          <ProgramLines s={s} program={at} blocks={{ selected, select: (row) => setPicked({ program: at, row }) }} />
          <AddLogic v={v} canElse={canAddElse(entries)} add={(e) => editProgram(at, (now) => [...now, e])} />
        </div>
        <InstructionPanel s={s} text={text} cursor={Math.max(0, selected)} insert={insert} />
      </div>
    </div>
  );
}
