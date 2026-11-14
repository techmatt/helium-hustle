// Autosave in localStorage, tagged with the rules revision and config hash. A save
// from another revision or config is discarded, never migrated.

import type { Scenario, Stamped } from './engine';
import type { Speed } from './clock';

const KEY = 'hh.save';

export interface Save {
  revision: number;
  configHash: string;
  /** The state JSON as the engine produced it. */
  state: string;
  /** The game's record so far: its seed and scenario are in `state`/here. */
  seed: string;
  scenario: Scenario | null;
  actions: Stamped[];
  speed: Speed;
}

export type Loaded = { save: Save } | { discarded: string } | null;

/** The saved game, if it fits this engine and config; otherwise why it was dropped. */
export function loadSave(revision: number, configHash: string): Loaded {
  const raw = localStorage.getItem(KEY);
  if (raw === null) return null;
  let save: Save | null = null;
  try {
    save = JSON.parse(raw) as Save;
  } catch {}
  if (save === null || typeof save !== 'object' || typeof save.state !== 'string' || !Array.isArray(save.actions)) {
    localStorage.removeItem(KEY);
    return { discarded: 'The saved game could not be read, so a new game started.' };
  }
  if (save.revision !== revision) {
    localStorage.removeItem(KEY);
    return { discarded: `The saved game was from rules revision ${save.revision} (this is ${revision}), so a new game started.` };
  }
  if (save.configHash !== configHash) {
    localStorage.removeItem(KEY);
    return { discarded: `The saved game used another config (${save.configHash}, this is ${configHash}), so a new game started.` };
  }
  return { save };
}

export function writeSave(save: Save): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(save));
  } catch (e) {
    console.warn('autosave failed', e);
  }
}

export function clearSave(): void {
  localStorage.removeItem(KEY);
}
