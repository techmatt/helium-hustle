// A program edit sent to the engine. Kept apart from programEdit.ts, whose pure list edits
// Node imports for the unit checks (scripts/unit.mjs) without the store.

import type { Entry } from '../engine';
import { act, store } from '../store';

/** Rewrites a program's lines from what the engine holds now (not the last render:
 * two quick clicks must both land). */
export function editProgram(program: number, edit: (entries: Entry[]) => Entry[]): void {
  const now = store.view.programs[program]?.entries ?? [];
  const next = edit(now);
  // set_program restarts the program's pointer and its lines' runs: an edit that changes
  // nothing (a count retyped as "06") must not.
  if (JSON.stringify(next) === JSON.stringify(now)) return;
  act({ type: 'set_program', program, entries: next });
}
