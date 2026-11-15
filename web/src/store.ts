// The game in play: its state, record, clock and autosave, the latest engine view, and
// UI-only bookkeeping ("new" markers, the retirement summary, notices). Components read
// it through `useStore` and change it only through the functions here. No rules: every
// number shown comes from the engine's view, every change is an engine action.

import { useEffect, useState } from 'preact/hooks';
import baselineConfig from '../../configs/baseline.json?raw';
import { Clock, SPEEDS, type Speed } from './clock';
import { engine, type Action, type LastRun, type Scenario, type Stamped, type StoryView, type View } from './engine';
import * as playlog from './playlog';
import { clearSave, loadSave, writeSave } from './save';

/** wasm's `advance` takes a u32 tick count; larger catch-ups go in chunks. */
const MAX_CHUNK = 1_000_000;
/** Autosave at most this often while ticking (and always on an action or leaving). */
const SAVE_EVERY_MS = 1000;
const CHEATS_KEY = 'hh.cheats';

interface Game {
  state: string;
  seed: string;
  scenario: Scenario | null;
  actions: Stamped[];
  /** Its play log's id (and file name). */
  careerId: string;
}

/** A fingerprint line in the play log at least this often while playing. */
const CHECKPOINT_MS = 10_000;
let lastCheckpoint = 0;

export interface Store {
  view: View;
  speed: Speed;
  notice: string | null;
  /** The finished run's summary, shown until Continue (the clock is paused meanwhile). */
  summary: LastRun | null;
  /** Keys (`kind:id`) revealed during this run and not yet looked at. */
  fresh: Set<string>;
  /** Story texts waiting to be read, oldest first. */
  stories: StoryView[];
  cheatsEnabled: boolean;
  seed: string;
  scenario: Scenario | null;
  actionCount: number;
  configHash: string;
  /** Bumped on every change, so components re-render. */
  version: number;
}

let game: Game;
let lastSave = 0;
let configHash = '';
let speedBeforeSummary: Speed = 1;
/** The last speed other than pause. */
let resumeSpeed: Speed = 1;
let baselineRun = -1;
const baseline = new Set<string>();
const listeners = new Set<() => void>();
export const clock = new Clock(runTicks);

export const store: Store = {
  view: undefined as unknown as View,
  speed: 1,
  notice: null,
  summary: null,
  fresh: new Set(),
  stories: [],
  cheatsEnabled: localStorage.getItem(CHEATS_KEY) === '1',
  seed: '0',
  scenario: null,
  actionCount: 0,
  configHash: '',
  version: 0,
};

/** Re-renders the caller whenever the store changes. */
export function useStore(): Store {
  const [, set] = useState(0);
  useEffect(() => {
    const f = () => set((n) => n + 1);
    listeners.add(f);
    return () => {
      listeners.delete(f);
    };
  }, []);
  return store;
}

function emit(): void {
  store.version++;
  for (const f of listeners) f();
}

// ---- "new" markers -------------------------------------------------------------

/** Every revealed thing a marker can sit on, as `kind:id`. */
function revealed(v: View): string[] {
  return [
    ...v.nav.filter((t) => t.visible).map((t) => `nav:${t.id}`),
    ...v.buildings.filter((b) => b.visible).map((b) => `building:${b.id}`),
    ...v.commands.filter((c) => c.visible).map((c) => `command:${c.id}`),
    ...v.research.filter((r) => r.visible).map((r) => `research:${r.id}`),
    ...v.projects.filter((p) => p.unlocked || p.completed).map((p) => `project:${p.id}`),
    ...v.resources.filter((r) => r.visible).map((r) => `resource:${r.id}`),
  ];
}

/** What was visible at run start (or page load) is the baseline; anything revealed
 * later this run is fresh until looked at. */
function track(v: View, reset: boolean): void {
  if (reset || v.run !== baselineRun) {
    baseline.clear();
    store.fresh.clear();
    baselineRun = v.run;
    for (const k of revealed(v)) baseline.add(k);
    return;
  }
  for (const k of revealed(v)) {
    if (!baseline.has(k)) {
      baseline.add(k);
      store.fresh.add(k);
    }
  }
}

/** Story texts already shown this page session (quests that became active, one-off
 * events that fired), so each shows once. */
const toldStories = new Set<string>();

/** Queues the text of a quest that just became active or an event that just fired. On
 * load nothing is queued; a brand-new game queues its first quest (`intro`). */
function tell(v: View, mode: 'load' | 'intro' | 'play'): void {
  // A quest opens once a career; a one-off event fires again in each run it fires in.
  const items: [string, StoryView][] = [
    ...(v.quest ? [[v.quest.id, v.quest] as [string, StoryView]] : []),
    ...v.event_texts.map((e) => [`${v.run}:${e.id}`, e] as [string, StoryView]),
  ];
  for (const [key, e] of items) {
    if (toldStories.has(key)) continue;
    toldStories.add(key);
    if (mode === 'play' || (mode === 'intro' && e.id === v.quest?.id)) store.stories.push(e);
  }
}

export function dismissStory(): void {
  const s = store.stories.shift();
  if (s) playlog.log('story_read', { id: s.id });
  emit();
}

/** The player opened a panel or a program tab (the play log keeps where attention went). */
export function noteOpened(kind: 'panel' | 'program_tab', id: string | number): void {
  playlog.log(kind, kind === 'panel' ? { panel: id } : { program: id });
}

export function seen(key: string): void {
  if (store.fresh.delete(key)) emit();
}

/** The nav tile each kind of item lives under. */
const PANEL_OF: Record<string, string> = {
  building: 'buildings',
  command: 'commands',
  research: 'research',
  project: 'projects',
};

/** The nav tile has news: it was revealed, or something fresh sits in its panel. */
export function navHasNews(id: string): boolean {
  for (const k of store.fresh) {
    const [kind] = k.split(':');
    if (k === `nav:${id}` || PANEL_OF[kind] === id) return true;
  }
  return false;
}

// ---- the game ------------------------------------------------------------------

function refresh(reset = false, story: 'load' | 'intro' | 'play' = reset ? 'load' : 'play'): void {
  const before = store.view?.run;
  const v = engine.view(game.state);
  store.view = v;
  store.seed = game.seed;
  store.scenario = game.scenario;
  store.actionCount = game.actions.length;
  if (before !== undefined && v.run > before && v.last_run && !reset) {
    // A run just ended: show its summary with the clock stopped; Continue restores the
    // speed it had (paused included).
    store.summary = v.last_run;
    speedBeforeSummary = clock.speed;
    clock.setSpeed(0);
    store.speed = 0;
    playlog.log('retirement', { ended: v.last_run.run, forced: v.last_run.forced, days: v.last_run.days });
    checkpoint();
    save();
  }
  track(v, reset);
  tell(v, story);
  emit();
}

function runTicks(ticks: number): void {
  if (store.summary) return;
  const run = store.view.run;
  while (ticks > 0) {
    const n = Math.min(ticks, MAX_CHUNK);
    game.state = engine.advanceRun(game.state, n);
    ticks -= n;
    // advance_run stops after a forced retirement: show it before playing on.
    if ((JSON.parse(game.state) as { career: { run_number: number } }).career.run_number !== run) break;
  }
  if (performance.now() - lastSave > SAVE_EVERY_MS) save();
  refresh();
}

function save(): void {
  lastSave = performance.now();
  if (lastSave - lastCheckpoint > CHECKPOINT_MS) checkpoint();
  writeSave({
    careerId: game.careerId,
    revision: engine.rulesRevision(),
    configHash,
    state: game.state,
    seed: game.seed,
    scenario: game.scenario,
    actions: game.actions,
    speed: clock.speed,
    summary: store.summary,
    speedBeforeSummary,
    resumeSpeed,
    stories: store.stories,
  });
}

export function setNotice(text: string | null): void {
  store.notice = text;
  emit();
}

/** Logs the state's fingerprint at this tick (what `hh-sim playlog` checks its replay
 * against). */
function checkpoint(kind = 'checkpoint'): void {
  lastCheckpoint = performance.now();
  // Where the state itself stands (the view may lag a wake's ticks behind).
  const g = JSON.parse(game.state) as { tick: number; career: { run_number: number }; run: { day: number } };
  playlog.log(kind, { tick: g.tick, run: g.career.run_number, day: g.run.day, fingerprint: engine.fingerprint(game.state) });
}

/** Starts a play log for the game in hand; `prior` are its actions taken before. */
function openLog(prior: Stamped[]): string {
  const id = playlog.newCareerId(game.seed);
  playlog.open(id, {
    career: id,
    rules_revision: engine.rulesRevision(),
    config_hash: configHash,
    engine: engine.version(),
    config: JSON.parse(game.state).config,
    seed: game.seed,
    scenario: game.scenario,
    prior_actions: prior,
  });
  return id;
}

/** A new career (`why`: what started it). The career before, if any, closes its log. */
export function newGame(seed: string, scenario: Scenario | null, why = 'new game'): void {
  if (game) {
    checkpoint();
    playlog.log('career_end', { reason: why });
  }
  game = { state: engine.newGame(baselineConfig, seed, scenario?.name ?? ''), seed, scenario, actions: [], careerId: '' };
  store.view = undefined as unknown as View;
  game.careerId = openLog([]);
  playlog.log('new_game', { why, seed, scenario: scenario?.name ?? null });
  store.summary = null;
  store.stories = [];
  toldStories.clear();
  save();
  refresh(true, 'intro');
}

/** A fresh career on a random seed. */
export function newCareer(): void {
  const words = crypto.getRandomValues(new Uint32Array(2));
  const seed = ((BigInt(words[0]) << 32n) | BigInt(words[1])).toString();
  newGame(seed, null, 'new career');
  setNotice(null);
}

/** Applies an engine action between ticks; returns whether it applied. */
export function act(action: Action): boolean {
  const run = store.view.run;
  clock.wake(); // settle the ticks owed so the action lands between the right ticks
  if (store.view.run !== run) {
    // Those ticks ended the run: the click was meant for the run that is gone.
    playlog.log('refused', { action, reason: 'the run ended first' });
    setNotice('The run ended just before that: nothing was done.');
    return false;
  }
  let ok = true;
  try {
    game.state = engine.apply(game.state, action);
    const tick = engine.view(game.state).tick;
    game.actions.push({ tick, action });
    playlog.log('action', { tick, action });
    store.notice = null;
  } catch (e) {
    playlog.log('rejected', { action, error: String(e) });
    store.notice = String(e);
    ok = false;
  }
  save();
  refresh();
  return ok;
}

/** Applies actions in order, stopping at the first that fails. */
export function actAll(actions: Action[]): void {
  for (const a of actions) if (!act(a)) break;
}

export function setSpeed(s: Speed): void {
  if (store.summary) return;
  clock.setSpeed(s);
  // `speed` on a line is the speed from then on.
  if (s !== store.speed) playlog.log('speed', { from: store.speed, speed: s });
  store.speed = s;
  if (s !== 0) resumeSpeed = s;
  save();
  emit();
}

/** Pauses, or resumes at the last speed played (the space key). */
export function togglePause(): void {
  setSpeed(store.speed === 0 ? resumeSpeed : 0);
}

/** Closes the retirement summary and plays on at the speed before it. */
export function continueAfterRetirement(): void {
  store.summary = null;
  playlog.log('continue');
  setSpeed(speedBeforeSummary);
}

export function setCheatsEnabled(on: boolean): void {
  playlog.log('debug', { what: 'cheats', on });
  store.cheatsEnabled = on;
  localStorage.setItem(CHEATS_KEY, on ? '1' : '0');
  emit();
}

/** Jumps ahead: a marked cheat action, then the days played as usual (stopping at a
 * retirement), so replay stays exact. */
export function jumpAhead(days: number): void {
  if (!act({ type: 'cheat', cheat: { kind: 'jump_ahead', days } })) return;
  clock.wake();
  runTicks(days);
  // Pins the jump in the play log: its days took no real time.
  checkpoint('jump_end');
  save();
}

export function record() {
  return {
    format: 'hh-record/1',
    rules_revision: engine.rulesRevision(),
    // The config the game plays under, as embedded in its state (not today's file,
    // whose name may differ under the same hash).
    config: JSON.parse(game.state).config,
    seed: game.seed,
    scenario: game.scenario,
    actions: game.actions,
    ticks: engine.view(game.state).tick,
    fingerprint: engine.fingerprint(game.state),
  };
}

export function checkReplay(): void {
  try {
    const r = engine.replay(record());
    const ok = r.fingerprint === engine.fingerprint(game.state) && r.warnings.length === 0;
    playlog.log('debug', { what: 'replay_check', ok });
    setNotice(ok ? `Replay ok: ${r.fingerprint}` : `Replay MISMATCH: ${r.fingerprint} ${r.warnings.join('; ')}`);
  } catch (e) {
    setNotice(`Replay failed: ${e}`);
  }
}

export function fingerprint(): string {
  return engine.fingerprint(game.state);
}

/** Loads the save (or starts a new game) and starts the clock. */
export function start(): void {
  configHash = engine.resolveConfig(baselineConfig).hash;
  store.configHash = configHash;
  void playlog.start();
  playlog.locate(() => ({
    tick: store.view?.tick ?? 0,
    run: store.view?.run ?? 0,
    day: store.view?.day ?? 0,
    speed: store.speed,
  }));
  const loaded = loadSave(engine.rulesRevision(), configHash);
  let how = 'new';
  if (loaded && 'save' in loaded) {
    try {
      const s = loaded.save;
      engine.view(s.state); // throws if the engine can't read it
      game = { state: s.state, seed: s.seed, scenario: s.scenario, actions: s.actions, careerId: s.careerId ?? '' };
      // A save from before play logs: its log starts now, carrying the actions so far.
      if (!game.careerId) game.careerId = openLog(game.actions);
      else playlog.open(game.careerId, null);
      how = 'save';
      const speed = SPEEDS.includes(s.speed) ? s.speed : 1;
      clock.speed = speed;
      store.speed = speed;
      if (s.resumeSpeed && SPEEDS.includes(s.resumeSpeed)) resumeSpeed = s.resumeSpeed;
      if (s.speedBeforeSummary !== undefined && SPEEDS.includes(s.speedBeforeSummary)) speedBeforeSummary = s.speedBeforeSummary;
      refresh(true);
      // A run summary left open is still open, and unread story texts still wait.
      store.summary = s.summary ?? null;
      store.stories = Array.isArray(s.stories) ? s.stories : [];
    } catch (e) {
      clearSave();
      // `offline`: written at a later visit, so it has no place in the career's time.
      if (loaded.save.careerId) playlog.log('career_end', { reason: `save unreadable: ${e}`, offline: true }, loaded.save.careerId);
      newGame('0', null, 'save unreadable');
      setNotice(`The saved game could not be loaded (${e}), so a new game started.`);
      how = 'save unreadable';
    }
  } else {
    // A discarded save's career keeps its log, closed.
    if (loaded?.careerId) playlog.log('career_end', { reason: loaded.discarded, offline: true }, loaded.careerId);
    newGame('0', null, loaded ? 'save discarded' : 'first visit');
    if (loaded) setNotice(loaded.discarded);
    if (loaded) how = 'save discarded';
  }
  playlog.log('session_start', {
    how,
    width: window.innerWidth,
    height: window.innerHeight,
    agent: navigator.userAgent,
  });
  checkpoint();
  clock.start();
  // Leaving the page pauses the game (no offline progress): save what was played.
  window.addEventListener('pagehide', () => {
    clock.wake();
    checkpoint();
    playlog.log('session_end');
    save();
    playlog.flushOnLeave();
  });
  // A page restored from the back/forward cache resumes where it left off.
  window.addEventListener('pageshow', (e) => {
    if (e.persisted) {
      clock.forgetTimeAway();
      playlog.log('session_start', { how: 'restored' });
    }
  });
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'hidden') {
      checkpoint();
      playlog.log('hidden');
      save();
      void playlog.flush(true);
    } else playlog.log('visible');
  });
}
