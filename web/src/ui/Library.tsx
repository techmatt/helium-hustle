// The loadout library under a program's toolbar: save the program (as new, or over a
// slot) and load, rename or delete saved ones. The library is career state: it outlives
// retirement, and loading is how a program comes back in a later run. Every change is an
// engine action; the engine checks names and slots.

import { useState } from 'preact/hooks';
import type { View } from '../engine';
import { act } from '../store';
import { lineText } from './ProgramLogic';

export type LibraryMode = 'save' | 'load';

export function Library({ v, program, mode, close }: { v: View; program: number; mode: LibraryMode; close: () => void }) {
  const [name, setName] = useState(`Program ${program + 1}, run ${v.run}`);
  const [renaming, setRenaming] = useState<number | null>(null);
  const [newName, setNewName] = useState('');
  const lib = v.library;
  const full = lib.length >= v.library_slots;
  const okName = name.trim().length > 0;
  const save = (slot: number | null) => {
    act({ type: 'save_loadout', program, slot, name: slot === null ? name : (lib[slot]?.name ?? name) });
    close();
  };
  return (
    <div class="library" data-testid="library" data-mode={mode}>
      {mode === 'save' && (
        <div class="library-save">
          <input
            type="text"
            data-testid="loadout-name"
            value={name}
            maxLength={v.loadout_name_max}
            onInput={(e) => setName((e.target as HTMLInputElement).value)}
            onKeyDown={(e) => e.key === 'Enter' && okName && !full && save(null)}
          />
          <button type="button" class="tiny" data-testid="loadout-save-new" disabled={!okName || full} onClick={() => save(null)}>
            Save as new
          </button>
          {full && <span class="muted"> Library full ({v.library_slots}): overwrite one below.</span>}
        </div>
      )}
      {lib.length === 0 ? (
        <p class="muted hint">No saved programs yet. Saved programs are kept across retirements.</p>
      ) : (
        <ul class="library-list">
          {lib.map((l, i) => (
            <li key={i} data-loadout={i} title={l.entries.map((e) => (e.indented ? '  ' : '') + lineText(v, e)).join('\n')}>
              {renaming === i ? (
                <input
                  type="text"
                  value={newName}
                  maxLength={v.loadout_name_max}
                  onInput={(e) => setNewName((e.target as HTMLInputElement).value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && newName.trim()) {
                      act({ type: 'rename_loadout', loadout: i, name: newName });
                      setRenaming(null);
                    } else if (e.key === 'Escape') setRenaming(null);
                  }}
                  onBlur={() => setRenaming(null)}
                  autoFocus
                />
              ) : (
                <span class="loadout-name">{l.name}</span>
              )}
              <span class="muted">
                {' '}
                {l.entries.length} line{l.entries.length === 1 ? '' : 's'}
              </span>
              <span class="loadout-actions">
                {mode === 'save' ? (
                  <button type="button" class="tiny" data-testid={`loadout-overwrite-${i}`} onClick={() => save(i)}>
                    Overwrite
                  </button>
                ) : (
                  <button
                    type="button"
                    class="tiny"
                    data-testid={`loadout-load-${i}`}
                    title={`Replace Program ${program + 1}'s lines with these`}
                    onClick={() => {
                      act({ type: 'load_loadout', loadout: i, program });
                      close();
                    }}
                  >
                    Load
                  </button>
                )}
                <button
                  type="button"
                  class="tiny"
                  onClick={() => {
                    setNewName(l.name);
                    setRenaming(i);
                  }}
                >
                  Rename
                </button>
                <button type="button" class="tiny" data-testid={`loadout-delete-${i}`} onClick={() => act({ type: 'delete_loadout', loadout: i })}>
                  Delete
                </button>
              </span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
