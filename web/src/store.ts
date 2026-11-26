// The game in play: its state, record, clock and autosave, the latest engine view, and
// UI-only bookkeeping ("new" markers, the retirement summary, notices). Components read
// it through `useStore` and change it only through the functions here. No rules: every
// number shown comes from the engine's view, every change is an engine action.

import { useEffect, useState } from 'preact/hooks';
import baselineConfig from '../../configs/baseline.json?raw';
import { Clock, SPEEDS, offeredSpeed, type Speed } from './clock';
import {
  engine,
  type Action,
  type AutopilotStatus,
  type AutopilotTarget,
  type InstructionGroup,
  type LastRun,
  type LinesRun,
  type ParsedText,
  type Scenario,
  type SideQuestView,
  type Stamped,
  type StoryView,
  type View,
} from './engine';
import { fillRate } from './format';
import type { Autopilot } from './wasm/hh_wasm.js';
import * as playlog from './playlog';
import { clearSave, loadSave, SAVE_KEY, storageGet, storageSet, writeSave } from './save';

/** wasm's `advance_run` takes a u32 tick count; larger catch-ups go in chunks. */
const MAX_CHUNK = 1_000_000;
/** What a tab that stood down (another tab plays the career) says when used. */
const ANOTHER_TAB = 'This game is playing in another tab. Reload this tab to play here.';
/** Autosave at most this often while ticking (and always on an action or leaving). */
const SAVE_EVERY_MS = 1000;
const CHEATS_KEY = 'hh.cheats';
/** Events' "Hide popups for events you've already seen" ('0': off; on by default). */
const HIDE_SEEN_POPUPS_KEY = 'hh-hide-seen-popups';

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

/** A run's starting values, as the career view gives them. */
export interface SummaryBase {
  starting_credits: string;
  boredom_resilience: string;
  buy_power_mult: string;
}

export interface Store {
  view: View;
  speed: Speed;
  notice: string | null;
  /** The finished run's summary, shown until Continue (the clock is paused meanwhile). */
  summary: LastRun | null;
  /** The ended run's view, shown behind its summary until Continue (null after a reload:
   * the new run shows). */
  backdrop: View | null;
  /** What the ended run started with (the summary's deltas; null: not known). */
  summaryBase: SummaryBase | null;
  /** Keys (`kind:id`) revealed during this run and not yet looked at. */
  fresh: Set<string>;
  /** The popup's pages still to read (a main-story transition, a side quest's completion,
   * a one-off event), the open one first (the clock is paused while any are left). */
  storyPages: StoryPage[];
  /** "Hide popups for events you've already seen" (Events' checkbox; per browser). */
  hideSeenPopups: boolean;
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
/** The speed the story modal's last page resumes. */
let speedBeforeStory: Speed = 1;
/** The last speed other than pause. */
let resumeSpeed: Speed = 1;
let baselineRun = -1;
/** The ended run's last state, when the ticks or action just played retired it, for the
 * next refresh (the summary's backdrop). */
let endedState: string | null = null;
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
  backdrop: null,
  summaryBase: null,
  fresh: new Set(),
  storyPages: [],
  hideSeenPopups: storageGet(HIDE_SEEN_POPUPS_KEY) !== '0',
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

/** One page of the main-story modal: one paragraph. */
export interface StoryPage {
  /** The quest it belongs to. */
  quest: string;
  title: string;
  /** `story`: the quest's opening text; `payoff`: its completion text. */
  kind: 'story' | 'payoff';
  text: string;
  /** The next quest's one-line task (its goal), on a transition's last page (mechanics
   * text, shown apart from the paragraph). */
  task?: string;
  /** The finished quest's reward, on its last payoff page (mechanics text). */
  reward?: string;
  /** A one-off event's effect, on its last page (mechanics text). */
  effect?: string;
  /** Its quest's picture's file stem (above the paragraph), null when it has none. */
  picture: string | null;
  /** The last page's button ("Begin" when absent: a main-story transition). */
  close?: string;
}

/** Completed quests (by id) and this run's one-off events (`run:id`) already told (or
 * found at a load) this page session, so each is told once. */
const toldStories = new Set<string>();
/** One-off events whose popup this career has shown (by id), saved with the game: with
 * `hideSeenPopups` on, one already here doesn't pop up again. */
const seenPopups = new Set<string>();
/** Set while a save from before `seenPopups` was kept loads: the run's events listed then
 * count as seen. */
let seedSeenAtLoad = false;
/** Active quests whose opening was told (or found active at a load). */
const openedQuests = new Set<string>();
/** The main-story quest active at the last redraw (a change is a transition). */
let lastQuest: string | null = null;

/** A quest's opening paragraphs, its body when it has none. */
function openingOf(q: { story?: string[]; body: string }): string[] {
  return q.story && q.story.length > 0 ? q.story : q.body ? [q.body] : [];
}

/** A finished chain quest's payoff pages, its reward on the last. */
function payoffPages(q: StoryView, pages: StoryPage[]): void {
  const at = pages.length;
  for (const t of q.payoff ?? []) pages.push({ quest: q.id, title: q.title, kind: 'payoff', text: t, picture: q.picture ?? null });
  if (pages.length > at && q.reward) pages[pages.length - 1].reward = q.reward;
}

/** A completed side quest's popup: its payoff (its body when it has none), its reward. */
function sideDonePages(q: StoryView): StoryPage[] {
  const texts = q.payoff && q.payoff.length > 0 ? q.payoff : [q.body];
  const pages: StoryPage[] = texts.map((text) => ({ quest: q.id, title: q.title, kind: 'payoff', text, picture: q.picture ?? null, close: 'OK' }));
  if (q.reward) pages[pages.length - 1].reward = q.reward;
  return pages;
}

/** A one-off event's popup: its story (its body when it has none), its effect. */
function eventPages(e: StoryView): StoryPage[] {
  const texts = e.story && e.story.length > 0 ? e.story : [e.body || e.summary];
  const pages: StoryPage[] = texts.map((text) => ({ quest: e.id, title: e.title, kind: 'story', text, picture: e.picture ?? null, close: 'OK' }));
  if (e.effect) pages[pages.length - 1].effect = fillRate(e.effect, e.rate);
  return pages;
}

/** Tells the story. A main-story transition (the active quest changed from X to Y)
 * queues the modal's pages: each chain quest completed since, in chain order (one that
 * was never shown active first gets its opening), its payoff, then Y's opening with its
 * task on the last page. On load nothing is queued; a brand-new game (run 1, day 0,
 * nothing completed) queues only its first quest's opening (`intro`). A completed side
 * quest pops up its payoff and reward. Every one-off event fired (Propellant discovery,
 * Burnout, each boredom phase) pops up too, unless `hideSeenPopups` is on and this
 * career has shown its popup before; either way it lands in the right column's Events
 * list (`eventItems`). */
function tell(v: View, mode: 'load' | 'intro' | 'play'): void {
  const pages: StoryPage[] = [];
  const quest = v.quest;
  for (const q of v.story) {
    if (toldStories.has(q.id)) continue;
    toldStories.add(q.id);
    if (mode !== 'play') continue;
    if (!q.chain) {
      pages.push(...sideDonePages(q));
      continue;
    }
    if (q.id !== lastQuest) for (const t of openingOf(q)) pages.push({ quest: q.id, title: q.title, kind: 'story', text: t, picture: q.picture ?? null });
    payoffPages(q, pages);
  }
  for (const e of v.event_texts) {
    const key = `${v.run}:${e.id}`;
    if (toldStories.has(key)) continue;
    toldStories.add(key);
    // Seen: popped up now, or listed when a save from before the set was kept loads. Not
    // what the autopilot played past or a scenario started with: the player saw no popup.
    if (mode !== 'play') {
      if (seedSeenAtLoad) seenPopups.add(e.id);
      continue;
    }
    if (pilot) continue;
    const seenBefore = seenPopups.has(e.id);
    seenPopups.add(e.id);
    if (seenBefore && store.hideSeenPopups) continue;
    pages.push(...eventPages(e));
  }
  const fresh = mode === 'intro' && v.run === 1 && v.day === 0 && v.completed_quests.length === 0;
  if (quest && quest.id !== lastQuest && !openedQuests.has(quest.id)) {
    openedQuests.add(quest.id);
    if (mode === 'play' || fresh) {
      for (const t of openingOf(quest)) pages.push({ quest: quest.id, title: quest.title, kind: 'story', text: t, picture: quest.picture });
      if (pages.length > 0) Object.assign(pages[pages.length - 1], { task: quest.goal });
    }
  }
  lastQuest = quest?.id ?? null;
  if (pages.length > 0 && !pilot) openStory(pages);
}

/** Queues modal pages, pausing the clock (the run summary, if open, already holds it). */
function openStory(pages: StoryPage[]): void {
  const first = store.storyPages.length === 0 && !store.summary;
  // The modal comes up under a pointer that was aiming at something else: a click just
  // after (a card's double-click, a quick next click) must not read its first page away.
  if (first) movedAt = performance.now();
  // Queued before the clock stops: setSpeed's settling wake then finds the gate shut
  // (runTicks plays nothing while pages wait), so no day passes the transition unseen.
  store.storyPages.push(...pages);
  if (first) speedBeforeStory = holdClock();
}

/** Stops the clock for a modal (a story chain, the run summary) and returns the speed it
 * had. Call it once the modal's gate is set: setSpeed's settling wake then plays nothing. */
function holdClock(): Speed {
  const was = clock.speed;
  clock.setSpeed(0);
  store.speed = 0;
  return was;
}

/** Story's finished-beat click: that quest's modal chain again (its opening, then its
 * payoff), the clock stopped as for a transition. Nothing while another modal or the
 * autopilot holds the screen. */
export function replayStory(q: StoryView): void {
  const pages: StoryPage[] = openingOf(q).map((text) => ({ quest: q.id, title: q.title, kind: 'story' as const, text, picture: q.picture ?? null }));
  if (pages.length > 0 && q.goal) pages[pages.length - 1].task = q.goal;
  payoffPages(q, pages);
  showPages(pages);
}

/** Opens `pages` as a popup on the player's click, the clock stopped as for a transition.
 * Nothing while another modal or the autopilot holds the screen. */
function showPages(pages: StoryPage[]): void {
  if (stoodDown || store.summary || store.storyPages.length > 0 || pilotBusy() || pages.length === 0) return;
  movedAt = performance.now();
  openStory(pages);
  emit();
}

/** An Events entry's click: that one-off event's popup again. */
export function replayEvent(e: StoryView): void {
  showPages(eventPages(e));
}

/** An open side quest's click (the right column): its popup, its story with its goal and
 * reward. */
export function showSideQuest(q: SideQuestView): void {
  const pages: StoryPage[] = [{ quest: q.id, title: q.title, kind: 'story', text: q.body, picture: q.picture ?? null, close: 'OK' }];
  if (q.goal) pages[0].task = q.goal;
  if (q.reward) pages[0].reward = q.reward;
  showPages(pages);
}

/** Events' "Hide popups for events you've already seen" (remembered per browser). */
export function setHideSeenPopups(on: boolean): void {
  store.hideSeenPopups = on;
  storageSet(HIDE_SEEN_POPUPS_KEY, on ? '1' : '0');
  emit();
}

/** The story modal's button: the next page, or after the last, play on at the speed
 * before it. */
export function nextStoryPage(): void {
  if (stoodDown) return;
  movedAt = performance.now();
  const p = store.storyPages.shift();
  if (p && store.storyPages.length === 0) {
    playlog.log('story_read', { id: p.quest });
    setSpeed(speedBeforeStory);
    return;
  }
  emit();
}

// ---- unread events -------------------------------------------------------------------

/** The right column's sections that count unread items. */
export type EventSection = 'events' | 'activity';

/** This run's one-off events the Events list shows (a superseded boredom event drops
 * out), oldest first. */
export function listedEvents(v: View): StoryView[] {
  return v.event_texts.filter((e) => !e.superseded);
}

/** Each section's items now, as keys stable while the item stays listed: this run's
 * listed events, this run's activity feed (a repeated line numbered by its occurrence). */
export function eventItems(v: View): Record<EventSection, string[]> {
  const n = new Map<string, number>();
  return {
    events: listedEvents(v).map((e) => `${v.run}:${e.id}`),
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
 * dismissed, a story page turned, the run summary closed) or a story modal opened: what
 * slides under the cursor is not what they clicked, so a building card and the story
 * modal's button ignore clicks for a moment after (`justMoved`). */
let movedAt = -Infinity;
const MOVED_MS = 500;
export function justMoved(): boolean {
  return performance.now() - movedAt < MOVED_MS;
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
  if (ranLines) noteIfTrue(v, ranLines);
  ranLines = null;
  store.seed = game.seed;
  store.scenario = game.scenario;
  store.actionCount = game.actions.length;
  let justEnded: View | null = null;
  if (before !== undefined && v.run > before && v.last_run && !reset) {
    // A run just ended: show its summary with the clock stopped; Continue restores the
    // speed it had (paused included).
    store.summary = v.last_run;
    store.backdrop = endedState ? engine.view(endedState) : null;
    justEnded = store.backdrop;
    const c = store.backdrop?.career;
    store.summaryBase = c
      ? { starting_credits: c.starting_credits, boredom_resilience: c.boredom_resilience, buy_power_mult: c.buy_power_mult }
      : null;
    // An open story modal already stopped the clock: Continue resumes what it held.
    const was = holdClock();
    speedBeforeSummary = store.storyPages.length > 0 ? speedBeforeStory : was;
    playlog.log('retirement', { ended: v.last_run.run, forced: v.last_run.forced, days: v.last_run.days });
    checkpoint();
    save();
  }
  // The ended run's last day first: an event that fired in the wake that retired the run
  // is in its view, not the new run's, and still pops up (behind the summary).
  const ended = justEnded;
  endedState = null;
  track(v, reset);
  if (reset) baselineRead(v);
  if (ended) tell(ended, story);
  tell(v, story);
  emit();
}

// ---- `if` history (the text editor's "fired k/10") --------------------------------------

/** Per program: the days each `if` line was true, as bits (bit 0: the last day played),
 * over the `observed` days played since its lines last changed (`key`). Display only, not
 * saved: a reload starts it again. */
interface IfHistory {
  key: string;
  bits: number[];
  observed: number;
}
const ifHistory: IfHistory[] = [];

/** Folds a span's `if_true` bits into each program's history; a program whose lines
 * changed starts over. */
function noteIfTrue(v: View, ran: LinesRun): void {
  const days = ran.days;
  if (!ran.if_true || days <= 0) return;
  v.programs.forEach((p, i) => {
    const key = JSON.stringify(p.entries);
    let h = ifHistory[i];
    if (!h || h.key !== key) h = ifHistory[i] = { key, bits: [], observed: 0 };
    const now = ran.if_true[i] ?? [];
    h.bits = p.entries.map((_, j) => {
      const span = (now[j] ?? 0) >>> 0;
      // A shift by 32 or more is a shift by its low 5 bits in JS: the old bits are all out.
      return days >= 32 ? span : (((h.bits[j] ?? 0) << days) | span) >>> 0;
    });
    h.observed = Math.min(h.observed + days, 1_000_000);
  });
}

/** How many of a line's last `min(10, observed)` days its `if` was true (null: none
 * observed since the program's lines last changed). */
export function ifFired(program: number, line: number): { fired: number; of: number } | null {
  const h = ifHistory[program];
  const p = store.view.programs[program];
  if (!h || !p || h.observed === 0 || h.key !== JSON.stringify(p.entries)) return null;
  const of = Math.min(10, h.observed);
  let bits = (h.bits[line] ?? 0) & ((1 << of) - 1);
  let fired = 0;
  while (bits) {
    fired += bits & 1;
    bits >>>= 1;
  }
  return { fired, of };
}

/** The engine's parse of a text-editor draft, against the game in play (what is
 * unlocked this run). Throws the engine's error. */
export function parseProgramText(text: string): ParsedText {
  return engine.parseProgramText(game.state, text);
}

/** The text editor's instruction panel against the game in play. */
export function programInstructions(text: string, cursor: number): InstructionGroup[] {
  return engine.programInstructions(game.state, text, cursor);
}

/** Plays `ticks` days, stopping after a forced retirement, and (the clock's wakes, not a
 * jump: `toQuest`) after a quest change, so its story modal holds the clock on the day it
 * came; the wake's other days are not played. */
function runTicks(ticks: number, toQuest = true): void {
  if (store.summary || store.storyPages.length > 0) return;
  const run = store.view.run;
  let lines: LinesRun | null = null;
  while (ticks > 0) {
    const n = Math.min(ticks, MAX_CHUNK);
    const pre = game.state;
    const r = engine.advanceRun(pre, n);
    game.state = r.state;
    lines = r.lines; // one chunk in practice (MAX_CHUNK); a later one supersedes
    ticks -= r.ran;
    // The run ended on this chunk's last tick (nothing changed before it, or the chunk
    // would have stopped there): its last day's state, again, for the summary's backdrop.
    if (r.stopped && head().career.run_number !== run) endedState = r.ran > 1 ? engine.advanceRun(pre, r.ran - 1).state : pre;
    if (r.stopped && (toQuest || head().career.run_number !== run)) break;
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
    // An open story modal is saved with the speed its last page resumes (the clock is
    // held meanwhile), so a reload mid-chain shows its unread pages again.
    speed: clock.speed,
    summary: store.summary,
    speedBeforeSummary,
    summaryBase: store.summaryBase,
    resumeSpeed,
    storyPages: store.storyPages,
    seenPopups: [...seenPopups],
    speedBeforeStory,
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
  abortAutopilot();
  if (game) {
    checkpoint();
    playlog.log('career_end', { reason: why });
  }
  game = { state, seed, scenario, actions: [], careerId: '' };
  // The `if` history was the old game's.
  ifHistory.length = 0;
  store.view = undefined as unknown as View;
  game.careerId = openLog([]);
  playlog.log('new_game', { why, seed, scenario: scenario?.name ?? null });
  // A career begun mid-visit starts its own session (a page load logs its own).
  if (sessionOpen) playlog.log('session_start', { how: 'new', width: window.innerWidth, height: window.innerHeight, agent: navigator.userAgent });
  store.summary = null;
  store.backdrop = null;
  store.summaryBase = null;
  store.storyPages = [];
  toldStories.clear();
  seenPopups.clear();
  openedQuests.clear();
  lastQuest = null;
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
  // play into the new run unseen. So does the story modal.
  if (store.summary || store.storyPages.length > 0 || pilotBusy()) return false;
  const run = store.view.run;
  clock.wake(); // settle the ticks owed so the action lands between the right ticks
  if (store.view.run !== run) {
    // Those ticks ended the run: the click was meant for the run that is gone.
    playlog.log('refused', { action, reason: 'the run ended first' });
    setNotice('The run ended just before that: nothing was done.');
    return false;
  }
  let ok = true;
  const before = game.state;
  try {
    game.state = engine.apply(game.state, action);
    // A retirement: the run's last state backs its summary.
    if (head().career.run_number !== run) endedState = before;
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

export function setSpeed(want: Speed): void {
  // 50x and 200x wait for Accelerated Scheduling (the view's `fast_speeds`).
  const s = offeredSpeed(want, store.view?.fast_speeds ?? false);
  // Settle the ticks owed at the old speed first: they may end the run, and then the
  // summary holds the clock stopped.
  if (stoodDown) return;
  if (s !== 0 && pilotBusy()) return;
  clock.wake();
  // The summary and the story modal hold the clock stopped until closed.
  if (store.summary || store.storyPages.length > 0) return;
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
  store.backdrop = null;
  store.summaryBase = null;
  playlog.log('continue');
  // A story chain that came with the retirement shows next, the clock still stopped;
  // its last page resumes the speed before the summary.
  if (store.storyPages.length > 0) {
    speedBeforeStory = speedBeforeSummary;
    emit();
    return;
  }
  setSpeed(speedBeforeSummary);
}

export function setCheatsEnabled(on: boolean): void {
  playlog.log('debug', { what: 'cheats', on });
  store.cheatsEnabled = on;
  storageSet(CHEATS_KEY, on ? '1' : '0');
  emit();
}

/** Jumps ahead: a marked cheat action, then the days played as usual (stopping at a
 * retirement, not at a quest change: its modal shows after), so replay stays exact. */
export function jumpAhead(days: number): void {
  if (!act({ type: 'cheat', cheat: { kind: 'jump_ahead', days } })) return;
  clock.wake();
  runTicks(days, false);
  // The wall time the jump took to compute is no play time.
  clock.forgetTimeAway();
  // Pins the jump in the play log: its days took no real time.
  checkpoint('jump_end');
  save();
}

/** A running autopilot (debug accelerator), if any. */
let pilot: { ap: Autopilot; stop: boolean } | null = null;
/** Ticks the autopilot plays between redraws. */
const PILOT_CHUNK = 2000;

/** The autopilot's progress line (null while none ran this visit). */
export const pilotStatus: { text: string | null; running: boolean } = { text: null, running: false };

/** Hands the game to a sim player (the debug autopilot, docs/simulator.md *Accelerators*):
 * it plays on, honestly and with the boost when given, until `target` is reached, the
 * budget is spent or Stop. Its actions (the boost and autopilot cheats first) join the
 * record and the play log as any others, so replay stays exact; the jump's days take no
 * play time in the log (`jump_end` closes them, as for jump ahead). */
export function runAutopilot(target: AutopilotTarget, agent: string, boost: string, budget: number): void {
  if (pilot || stoodDown || store.summary || store.storyPages.length > 0) return;
  setSpeed(0);
  const career = game.careerId;
  const met = pilotMet(career);
  let ap: Autopilot;
  try {
    ap = engine.autopilot(game.state, target, agent, boost, budget, met);
  } catch (e) {
    setNotice(`Autopilot: ${e}`);
    return;
  }
  const mine = { ap, stop: false };
  pilot = mine;
  pilotStatus.running = true;
  playlog.log('debug', { what: 'autopilot', target, agent, boost, budget });
  const startedAt = performance.now();
  const take = () => {
    const acts = JSON.parse(ap.take_actions()) as Stamped[];
    for (const a of acts) {
      game.actions.push(a);
      playlog.log('action', { tick: a.tick, action: a.action });
    }
    game.state = ap.state();
  };
  const finish = (text: string) => {
    pilot = null;
    pilotStatus.running = false;
    pilotStatus.text = text;
    freePilot(ap);
    clock.forgetTimeAway();
    checkpoint('jump_end');
    save();
    refresh(true);
  };
  const chunk = () => {
    // Aborted (a new game, another tab): the career it played is gone; nothing is taken.
    if (pilot !== mine) return;
    let st: AutopilotStatus;
    try {
      st = JSON.parse(ap.step(PILOT_CHUNK)) as AutopilotStatus;
      take();
    } catch (e) {
      finish(`Autopilot failed: ${e}`);
      return;
    }
    for (const h of st.hits) if (!met.includes(h.id)) met.push(h.id);
    storageSet(PILOT_MET_KEY, JSON.stringify({ career, met }));
    const secs = ((performance.now() - startedAt) / 1000).toFixed(1);
    const last = st.hits.length ? st.hits[st.hits.length - 1].name : 'nothing yet';
    const text =
      `${st.reached ? 'Reached' : st.gave_up ? 'Gave up' : mine.stop ? 'Stopped' : 'Playing'}: ` +
      `${st.ticks_played} days in ${secs} s, run ${st.run}; ${st.hits.length} steps met (last: ${last}); ` +
      `${st.left.length} left${st.left.length && st.left.length <= 4 ? ` (${st.left.join('; ')})` : ''}`;
    if (st.reached || st.gave_up || mine.stop) return finish(text);
    pilotStatus.text = text;
    // Each chunk saved: the save, the play log and `hh.pilotMet` stay in step if the
    // page dies or another tab takes the career mid-run.
    save();
    refresh(true);
    window.setTimeout(chunk, 0);
  };
  window.setTimeout(chunk, 0);
}

export function stopAutopilot(): void {
  if (pilot) pilot.stop = true;
}

/** Drops a running autopilot without taking its game (the career it played is gone). */
function abortAutopilot(): void {
  if (!pilot) return;
  const ap = pilot.ap;
  pilot = null;
  pilotStatus.running = false;
  pilotStatus.text = 'Autopilot dropped: the career changed.';
  freePilot(ap);
}

/** Frees the wasm object; after a panic mid-step it may still count as borrowed and
 * refuse (its memory is then lost, a debug run's worth). */
function freePilot(ap: Autopilot): void {
  try {
    ap.free();
  } catch {
    // ignored: see above
  }
}

/** The steps the autopilot has met in this career (`hh.pilotMet`): a milestone met in an
 * earlier run (a run's building count, a credit rate) stays met for the next *Next
 * milestone*. */
const PILOT_MET_KEY = 'hh.pilotMet';
function pilotMet(career: string): string[] {
  try {
    const v = JSON.parse(storageGet(PILOT_MET_KEY) ?? 'null') as { career?: string; met?: string[] } | null;
    return v?.career === career && Array.isArray(v.met) ? v.met : [];
  } catch {
    return [];
  }
}

/** While the autopilot plays, the game is its: a hand action, a speed or a jump would
 * land on a state the next chunk replaces. */
function pilotBusy(): boolean {
  if (!pilot) return false;
  setNotice('The autopilot is playing: Stop it first.');
  return true;
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
  abortAutopilot();
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
      const fast = engine.view(s.state).fast_speeds; // throws if the engine can't read it
      game ={ state: s.state, seed: s.seed, scenario: s.scenario, actions: s.actions, careerId: s.careerId ?? '' };
      // A save from before play logs: its log starts now, carrying the actions so far.
      if (!game.careerId) game.careerId = openLog(game.actions);
      else playlog.open(game.careerId, null);
      how = 'save';
      // A speed no longer offered (an older save's 2000x or 10000x debug speed) reloads as 1x;
      // 50x or 200x without Accelerated Scheduling as 10x.
      const normal = (x: number): x is Speed => (SPEEDS as readonly number[]).includes(x);
      const offered = (x: Speed) => offeredSpeed(x, fast);
      const speed = offered(normal(s.speed) ? s.speed : 1);
      clock.speed = speed;
      store.speed = speed;
      if (s.resumeSpeed && normal(s.resumeSpeed)) resumeSpeed = offered(s.resumeSpeed);
      if (s.speedBeforeSummary !== undefined && normal(s.speedBeforeSummary)) speedBeforeSummary = offered(s.speedBeforeSummary);
      // A story chain left mid-read shows its unread pages again, the clock held; its
      // last page resumes the speed saved with it.
      if (Array.isArray(s.storyPages) && s.storyPages.length > 0) {
        store.storyPages = s.storyPages;
        speedBeforeStory = s.speedBeforeStory !== undefined && normal(s.speedBeforeStory) ? offered(s.speedBeforeStory) : 1;
        clock.speed = 0;
        store.speed = 0;
      }
      // The events whose popup this career has shown (an older save's: those of the run
      // in play, which the load adds).
      seenPopups.clear();
      if (Array.isArray(s.seenPopups)) for (const id of s.seenPopups) if (typeof id === 'string') seenPopups.add(id);
      seedSeenAtLoad = !Array.isArray(s.seenPopups);
      refresh(true);
      seedSeenAtLoad = false;
      // A run summary left open is still open.
      store.summary = s.summary ?? null;
      store.summaryBase = (store.summary && s.summaryBase) || null;
      // An older save's unread side-quest banners come up as their completion popups.
      const banners = (Array.isArray(s.stories) ? s.stories : []).flatMap((x: StoryView) => {
        const q = store.view.story.find((y) => y.id === x.id);
        return q && !q.chain ? sideDonePages(q) : [];
      });
      if (banners.length > 0) openStory(banners);
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
