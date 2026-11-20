// A program edit sent to the engine. Kept apart from programEdit.ts, whose pure list edits
// Node imports for the unit checks (scripts/unit.mjs) without the store.

import type { Entry } from '../engine';
import { act, store } from '../store';

/** Rewrites a program's lines from what the engine holds now (not the last render:
 * two quick clicks must both land). */
export function editProgram(program: number, edit: (entries: Entry[]) => Entry[]): void {
  const now = store.view.programs[program]?.entries ?? [];
  act({ type: 'set_program', program, entries: edit(now) });
}
