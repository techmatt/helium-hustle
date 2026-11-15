// Autosave in localStorage, tagged with the rules revision and config hash. A save
// from another revision or config is discarded, never migrated.

import type { LastRun, Scenario, Stamped, StoryView } from './engine';
import type { Speed } from './clock';

const KEY = 'hh.save';

export interface Save {
  /** The career's play log (src/playlog.ts). */
  careerId?: string;
  revision: number;
  configHash: string;
  /** The state JSON as the engine produced it. */
  state: string;
  /** The game's record so far: its seed and scenario are in `state`/here. */
  seed: string;
  scenario: Scenario | null;
  actions: Stamped[];
  speed: Speed;
  /** A run summary still open, the speed Continue restores, and the last speed played. */
  summary?: LastRun | null;
  speedBeforeSummary?: Speed;
  resumeSpeed?: Speed;
  /** Story texts not yet read. */
  stories?: StoryView[];
}

/** A save, or why one was dropped (with its career's play log id, to close it). */
export type Loaded = { save: Save } | { discarded: string; careerId?: string } | null;

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
    return {
      discarded: `The saved game was from rules revision ${save.revision} (this is ${revision}), so a new game started.`,
      careerId: save.careerId,
    };
  }
  if (save.configHash !== configHash) {
    localStorage.removeItem(KEY);
    return {
      discarded: `The saved game used another config (${save.configHash}, this is ${configHash}), so a new game started.`,
      careerId: save.careerId,
    };
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
