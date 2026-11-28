// Autosave in localStorage, tagged with the rules revision and config hash. A save
// from another revision or config is discarded, never migrated.

import type { LastRun, Scenario, Stamped } from './engine';
import type { Speed } from './clock';
import type { StoryPage } from './story';
import type { SummaryBase } from './store';

export const SAVE_KEY = 'hh.save';

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
  /** A run summary still open (Continue plays the new run at 1x), and the last speed played. */
  summary?: LastRun | null;
  /** What the summarized run started with (the summary's deltas). */
  summaryBase?: SummaryBase | null;
  /** With a summary open, the ended run's last state (its backdrop), so a reload keeps
   * that day behind the summary. */
  backdrop?: string | null;
  resumeSpeed?: Speed;
  /** The one-off events whose popup this career has shown (Events' "Hide popups for
   * events you've already seen"). */
  seenPopups?: string[];
  /** A story modal chain's unread pages (the open one first) and the speed its last
   * page resumes: a reload mid-chain shows them again, the clock held. */
  storyPages?: StoryPage[];
  speedBeforeStory?: Speed;
  /** The play record's recorder (`hh_sim::playrecord::Recorder`), so a reload keeps the
   * log's record whole. */
  record?: string | null;
  /** The page that wrote it: a claim by another tab always changes the save, so the
   * tab playing before hears of it (`storage` fires only on a change). */
  page?: string;
}

/** localStorage, which throws on every touch when the browser blocks site data: a read
 * gives null, a removal does nothing, so the game still plays (unsaved, and saying so). */
export function storageGet(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

export function storageSet(key: string, value: string): void {
  try {
    localStorage.setItem(key, value);
  } catch {}
}

function storageRemove(key: string): void {
  try {
    localStorage.removeItem(key);
  } catch {}
}

/** A save, or why one was dropped (with its career's play log id, to close it). */
type Loaded = { save: Save } | { discarded: string; careerId?: string } | null;

/** The saved game, if it fits this engine and config; otherwise why it was dropped. */
export function loadSave(revision: number, configHash: string): Loaded {
  const raw = storageGet(SAVE_KEY);
  if (raw === null) return null;
  let save: Save | null = null;
  try {
    save = JSON.parse(raw) as Save;
  } catch {}
  if (save === null || typeof save !== 'object' || typeof save.state !== 'string' || !Array.isArray(save.actions)) {
    storageRemove(SAVE_KEY);
    return { discarded: 'The saved game could not be read, so a new game started.' };
  }
  if (save.revision !== revision) {
    storageRemove(SAVE_KEY);
    return {
      discarded: `The saved game was from rules revision ${save.revision} (this is ${revision}), so a new game started.`,
      careerId: save.careerId,
    };
  }
  if (save.configHash !== configHash) {
    storageRemove(SAVE_KEY);
    return {
      discarded: `The saved game used another config (${save.configHash}, this is ${configHash}), so a new game started.`,
      careerId: save.careerId,
    };
  }
  return { save };
}

/** Writes the save; on failure (storage full) returns why, so the page can say so. */
export function writeSave(save: Save): string | null {
  try {
    localStorage.setItem(SAVE_KEY, JSON.stringify(save));
    return null;
  } catch (e) {
    console.warn('autosave failed', e);
    return String(e);
  }
}

export function clearSave(): void {
  storageRemove(SAVE_KEY);
}
