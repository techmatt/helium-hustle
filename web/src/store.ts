// The game in play: its state, record, clock and autosave, the latest engine view, and
// UI-only bookkeeping ("new" markers, the retirement summary, notices). Components read
// it through `useStore` and change it only through the functions here. No rules: every
// number shown comes from the engine's view, every change is an engine action.

import { useEffect, useState } from 'preact/hooks';
import baselineConfig from '../../configs/baseline.json?raw';
import { Clock, SPEEDS, type Speed } from './clock';
import { engine, type Action, type LastRun, type LinesRun, type Scenario, type Stamped, type StoryView, type View } from './engine';
import * as playlog from './playlog';
import { clearSave, loadSave, SAVE_KEY, storageGet, storageSet, writeSave } from './save';

/** wasm's `advance_run` takes a u32 tick count; larger catch-ups go in chunks. */
const MAX_CHUNK = 1_000_000;
/** What a tab that stood down (another tab plays the career) says when used. */
const ANOTHER_TAB = 'This game is playing in another tab. Reload this tab to play here.';
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
  /** The program lines run over the days since the last redraw (null after an action
   * or a load): above 10x one redraw covers several days. */
  lines: LinesRun | null;
  /** Bumped on every change, so components re-render. */
  version: number;
}

let game: Game;
/** The page's session is under way (start() logged its session_start). */
let sessionOpen = false;
/** Another tab took this career over: this page stops playing, saving and logging. */
let stoodDown = false;
let lastSave = 0;
let saveFailed = false;
let configHash = '';
let speedBeforeSummary: Speed = 1;
/** The last speed other than pause. */
let resumeSpeed: Speed = 1;
let baselineRun = -1;
/** The lines run by the ticks just played, for the next refresh. */
let ranLines: LinesRun | null = null;
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
  cheatsEnabled: storageGet(CHEATS_KEY) === '1',
  seed: '0',
  scenario: null,
  actionCount: 0,
  configHash: '',
  lines: null,
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
  const now = new Set(revealed(v));
  for (const k of now) {
    if (!baseline.has(k)) {
      baseline.add(k);
      store.fresh.add(k);
    }
  }
  // Hidden again before it was looked at (a research card whose condition lapsed): its
  // marker goes, and a later reveal counts as new again.
  for (const k of store.fresh) {
    if (!now.has(k)) {
      store.fresh.delete(k);
      baseline.delete(k);
    }
  }
}

/** Quest texts already shown this page session, so each shows once. */
const toldStories = new Set<string>();

/** Queues the text of a main-story quest that just became active. On load nothing is
 * queued; a brand-new game queues its first quest (`intro`). Other events (boredom beats,
 * side story) are never queued here: they land in the right column's Events sections as
 * unread items (`eventItems`), so nothing stands in front of the main quest. */
function tell(v: View, mode: 'load' | 'intro' | 'play'): void {
  // A quest opens once a career. In play, one that opened and completed between two
  // redraws (the engine closes quests in a chain within one day) is told too, in order;
  // at a load or a new game's start, what is already completed was told before.
  for (const q of v.story) {
    if (toldStories.has(q.id)) continue;
    toldStories.add(q.id);
    if (mode === 'play') store.stories.push(q);
  }
  const q = v.quest;
  if (!q || toldStories.has(q.id)) return;
  toldStories.add(q.id);
  if (mode !== 'load') store.stories.push(q);
}

// ---- unread events -------------------------------------------------------------------

/** The right column's Events sections that count unread items. */
export type EventSection = 'story' | 'completed' | 'activity';

/** Each Events section's items now, as keys stable while the item stays listed: this
 * run's event texts, the completed quests, this run's activity feed (a repeated line
 * numbered by its occurrence). */
export function eventItems(v: View): Record<EventSection, string[]> {
  const n = new Map<string, number>();
  return {
    story: v.event_texts.map((e) => `${v.run}:${e.id}`),
    completed: v.story.map((e) => e.id),
    activity: v.activity.map((a) => {
      const base = `${v.run}:${a.day}:${a.kind}:${a.id}`;
      const i = n.get(base) ?? 0;
      n.set(base, i + 1);
      return `${base}:${i}`;
    }),
  };
}

/** Events items already read (UI only, like the "new" markers): what is listed when the
 * page loads counts as read; anything later is unread until its section has been open
 * and on screen for a moment (`markRead`). */
const readItems = new Set<string>();

function baselineRead(v: View): void {
  readItems.clear();
  for (const keys of Object.values(eventItems(v))) for (const k of keys) readItems.add(k);
}

export function unreadIn(v: View, section: EventSection): string[] {
  return eventItems(v)[section].filter((k) => !readItems.has(k));
}

/** Marks `keys` (a section's unread items when it was seen) read. */
export function markRead(keys: string[]): void {
  let changed = false;
  for (const k of keys) {
    if (!readItems.has(k)) {
      readItems.add(k);
      changed = true;
    }
  }
  // A long career's feed keys pile up: keep only what is still listed.
  if (readItems.size > 2000) {
    const now = new Set(Object.values(eventItems(store.view)).flat());
    for (const k of readItems) if (!now.has(k)) readItems.delete(k);
  }
  if (changed) emit();
}

/** When the screen last moved under the pointer by the player's own click (a story text
 * dismissed, the run summary closed): what slides under the cursor is not what they
 * clicked, so a building card ignores clicks for a moment after (`justMoved`). */
let movedAt = -Infinity;
const MOVED_MS = 500;
export function justMoved(): boolean {
  return performance.now() - movedAt < MOVED_MS;
}

export function dismissStory(): void {
  movedAt = performance.now();
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
  // Only the redraw after ticks has lines run; after an action or a load the queues may differ.
  store.lines = ranLines;
  ranLines = null;
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
  if (reset) baselineRead(v);
  tell(v, story);
  emit();
}

function runTicks(ticks: number): void {
  if (store.summary) return;
  const run = store.view.run;
  let lines: LinesRun | null = null;
  while (ticks > 0) {
    const n = Math.min(ticks, MAX_CHUNK);
    const r = engine.advanceRun(game.state, n);
    game.state = r.state;
    lines = r.lines; // one chunk in practice (MAX_CHUNK); a later one supersedes
    ticks -= n;
    // advance_run stops after a forced retirement: show it before playing on.
    if (head().career.run_number !== run) break;
  }
  if (performance.now() - lastSave > SAVE_EVERY_MS) save();
  ranLines = lines;
  refresh();
}

function save(): void {
  if (stoodDown) return;
  lastSave = performance.now();
  if (lastSave - lastCheckpoint > CHECKPOINT_MS) checkpoint();
  const failed = writeSave({
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
    page: playlog.pageId,
  });
  // Said until a save lands again (an action's success clears the notice line, so a save
  // that fails again says it again): the game is no longer being kept.
  if (failed && (!saveFailed || store.notice === null)) store.notice = `Autosave failed (${failed}): progress since is not kept in this browser.`;
  saveFailed = failed !== null;
}

export function setNotice(text: string | null): void {
  store.notice = text;
  emit();
}

/** Logs the state's fingerprint at this tick (what `hh-sim playlog` checks its replay
 * against). */
/** The few fields of the game state the page reads itself (where the state stands, its
 * config). Cheaper than a view, which the engine builds by playing a day on a copy. */
interface StateHead {
  tick: number;
  config: unknown;
  career: { run_number: number };
  run: { day: number };
}

function head(): StateHead {
  return JSON.parse(game.state) as StateHead;
}

function checkpoint(kind = 'checkpoint'): void {
  lastCheckpoint = performance.now();
  // Where the state itself stands (the view may lag a wake's ticks behind).
  const g = head();
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
    config: head().config,
    seed: game.seed,
    scenario: game.scenario,
    prior_actions: prior,
  });
  return id;
}

/** A new career (`why`: what started it). The career before, if any, closes its log. */
export function newGame(seed: string, scenario: Scenario | null, why = 'new game'): void {
  if (stoodDown) throw new Error(ANOTHER_TAB);
  // The new state first: a seed or scenario the engine refuses throws here, and the
  // career in play goes on untouched (its log not closed).
  const state = engine.newGame(baselineConfig, seed, scenario?.name ?? '');
  // The engine's own reading of the seed ("+5", " 5" -> "5"): the career id (a file
  // name) and the record carry that.
  seed = String((JSON.parse(state) as { seed: number | string }).seed ?? seed);
  if (game) {
    checkpoint();
    playlog.log('career_end', { reason: why });
  }
  game = { state, seed, scenario, actions: [], careerId: '' };
  store.view = undefined as unknown as View;
  game.careerId = openLog([]);
  playlog.log('new_game', { why, seed, scenario: scenario?.name ?? null });
  // A career begun mid-visit starts its own session (a page load logs its own).
  if (sessionOpen) playlog.log('session_start', { how: 'new', width: window.innerWidth, height: window.innerHeight, agent: navigator.userAgent });
  store.summary = null;
  store.stories = [];
  toldStories.clear();
  save();
  refresh(true, 'intro');
}

/** A fresh career on a random seed. */
export function newCareer(): void {
  if (stoodDown) {
    setNotice(ANOTHER_TAB);
    return;
  }
  const words = crypto.getRandomValues(new Uint32Array(2));
  const seed = ((BigInt(words[0]) << 32n) | BigInt(words[1])).toString();
  newGame(seed, null, 'new career');
  setNotice(null);
}

/** Applies an engine action between ticks; returns whether it applied. */
export function act(action: Action): boolean {
  if (stoodDown) {
    setNotice(ANOTHER_TAB);
    return false;
  }
  // The run summary covers the screen: a key on a card still focused behind it must not
  // play into the new run unseen.
  if (store.summary) return false;
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
    const tick = head().tick;
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
  // Settle the ticks owed at the old speed first: they may end the run, and then the
  // summary holds the clock stopped.
  if (stoodDown) return;
  clock.wake();
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
  if (stoodDown) return;
  movedAt = performance.now();
  store.summary = null;
  playlog.log('continue');
  setSpeed(speedBeforeSummary);
}

export function setCheatsEnabled(on: boolean): void {
  playlog.log('debug', { what: 'cheats', on });
  store.cheatsEnabled = on;
  storageSet(CHEATS_KEY, on ? '1' : '0');
  emit();
}

/** Jumps ahead: a marked cheat action, then the days played as usual (stopping at a
 * retirement), so replay stays exact. */
export function jumpAhead(days: number): void {
  if (!act({ type: 'cheat', cheat: { kind: 'jump_ahead', days } })) return;
  clock.wake();
  runTicks(days);
  // The wall time the jump took to compute is no play time.
  clock.forgetTimeAway();
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
    config: head().config,
    seed: game.seed,
    scenario: game.scenario,
    actions: game.actions,
    ticks: head().tick,
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

/** Another tab claimed the career: stop at once, without playing the ticks owed (the
 * other tab goes on from the save it loaded) or writing anything more but a session end. */
function standDown(): void {
  if (stoodDown) return;
  stoodDown = true;
  clock.stop();
  clock.speed = 0;
  store.speed = 0;
  playlog.log('session_end', { reason: 'another tab took over' });
  playlog.close();
  setNotice('This game is now playing in another tab. Reload this tab to play here instead.');
}

/** The page that wrote the save now, if it says. */
function readSavePage(): string | undefined {
  try {
    return (JSON.parse(storageGet(SAVE_KEY) ?? 'null') as { page?: string } | null)?.page;
  } catch {
    return undefined;
  }
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
  sessionOpen = true;
  // Claims the career: a tab already playing it stands down (below, in that tab).
  save();
  clock.start();
  // One page plays a career at a time: when another tab of this game saves (`storage`
  // fires only in the other pages), this one stands down rather than overwrite it.
  window.addEventListener('storage', (e) => {
    if (e.key === SAVE_KEY) standDown();
  });
  // Leaving the page pauses the game (no offline progress): save what was played.
  window.addEventListener('pagehide', () => {
    if (stoodDown) return;
    clock.wake();
    checkpoint();
    playlog.log('session_end');
    save();
    playlog.flushOnLeave();
  });
  // A page restored from the back/forward cache resumes where it left off.
  window.addEventListener('pageshow', (e) => {
    if (e.persisted && !stoodDown) {
      // A cached page hears no storage events: another tab may have claimed the career.
      if (readSavePage() !== playlog.pageId) {
        standDown();
        return;
      }
      clock.forgetTimeAway();
      playlog.log('session_start', { how: 'restored' });
    }
  });
  document.addEventListener('visibilitychange', () => {
    if (stoodDown) return;
    if (document.visibilityState === 'hidden') {
      checkpoint();
      playlog.log('hidden');
      save();
      void playlog.flush(true);
    } else playlog.log('visible');
  });
}
