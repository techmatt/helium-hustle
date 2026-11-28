// What the story modal tells, and when: the pages of a main-story transition, a side
// quest's completion, a one-off event's popup and a project completion's, built from the
// engine's view. Pure (type
// imports only, the effect filler passed in), so scripts/unit.mjs runs it under Node; the
// store (`store.ts` `tell`) holds the `Told` record and opens the pages.

import type { Amounts, ProjectDoneView, SideQuestView, StoryView, View } from './engine';

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
  /** A project completion's popup (`text` empty): its card picture, what it gives and
   * what that completion cost. */
  project?: { id: string; rewards: string[]; costs: Amounts };
}

/** An event effect's `{rate}` filled in (`format.ts` `fillRate`). */
export type Fill = (effect: string, rate?: string | null) => string;

/** What this page session has told. */
export interface Told {
  /** Completed quests (by id) and this run's one-off events (`run:id`) already told (or
   * found at a load), so each is told once. */
  stories: Set<string>;
  /** One-off events whose popup this career has shown (by id), saved with the game: with
   * "Hide popups for events you've already seen" on, one already here doesn't pop up again. */
  seenPopups: Set<string>;
  /** Active quests whose opening was told (or found active at a load). */
  opened: Set<string>;
  /** The main-story quest active at the last redraw (a change is a transition). */
  lastQuest: string | null;
}

export function newTold(): Told {
  return { stories: new Set(), seenPopups: new Set(), opened: new Set(), lastQuest: null };
}

/** Forgets the session's story (a new game). */
export function clearTold(t: Told): void {
  t.stories.clear();
  t.seenPopups.clear();
  t.opened.clear();
  t.lastQuest = null;
}

/** A quest's opening paragraphs, its body when it has none. */
export function openingOf(q: { story?: string[]; body: string }): string[] {
  return q.story && q.story.length > 0 ? q.story : q.body ? [q.body] : [];
}

/** A finished chain quest's payoff pages, its reward on the last. */
export function payoffPages(q: StoryView, pages: StoryPage[]): void {
  const at = pages.length;
  for (const t of q.payoff ?? []) pages.push({ quest: q.id, title: q.title, kind: 'payoff', text: t, picture: q.picture ?? null });
  if (pages.length > at && q.reward) pages[pages.length - 1].reward = q.reward;
}

/** A completed side quest's popup: its payoff (its body when it has none), its reward. */
export function sideDonePages(q: StoryView): StoryPage[] {
  const texts = q.payoff && q.payoff.length > 0 ? q.payoff : [q.body];
  const pages: StoryPage[] = texts.map((text) => ({ quest: q.id, title: q.title, kind: 'payoff', text, picture: q.picture ?? null, close: 'OK' }));
  if (q.reward) pages[pages.length - 1].reward = q.reward;
  return pages;
}

/** A one-off event's popup: its story (its body when it has none), its effect. */
export function eventPages(e: StoryView, fill: Fill): StoryPage[] {
  const texts = e.story && e.story.length > 0 ? e.story : [e.body || e.summary];
  const pages: StoryPage[] = texts.map((text) => ({ quest: e.id, title: e.title, kind: 'story', text, picture: e.picture ?? null, close: 'OK' }));
  if (e.effect) pages[pages.length - 1].effect = fill(e.effect, e.rate);
  return pages;
}

/** A project completion's popup: one page, its name, what it gives and what it cost. */
export function projectPages(p: ProjectDoneView): StoryPage[] {
  return [{ quest: p.id, title: p.name, kind: 'payoff', text: '', picture: null, close: 'OK', project: { id: p.id, rewards: p.reward_text, costs: p.costs } }];
}

/** This run's part of a told or Events key: its number, after a timeline's end with the
 * timelines ended before it, as a new timeline starts again at run 1 (134: its run 1 met
 * the first timeline's keys and lost its popups). The first timeline keeps the plain number,
 * so a save's told and read records still match. */
export function runTag(v: Pick<View, 'run' | 'saga'>): string {
  return v.saga.ended > 0 ? `t${v.saga.ended}:${v.run}` : String(v.run);
}

/** A project completion's key in the told record and the Events list: the run, the
 * project and which completion it was. */
export function projectKey(v: Pick<View, 'run' | 'saga'>, p: ProjectDoneView): string {
  return `${runTag(v)}:project:${p.id}:${p.completion}`;
}

/** A finished chain quest's modal again (Story's Replay): its opening, then its payoff. */
export function replayPages(q: StoryView): StoryPage[] {
  const pages: StoryPage[] = openingOf(q).map((text) => ({ quest: q.id, title: q.title, kind: 'story' as const, text, picture: q.picture ?? null }));
  if (pages.length > 0 && q.goal) pages[pages.length - 1].task = q.goal;
  payoffPages(q, pages);
  return pages;
}

/** An open side quest's popup: its story with its goal and reward. */
export function sideQuestPages(q: SideQuestView): StoryPage[] {
  const pages: StoryPage[] = [{ quest: q.id, title: q.title, kind: 'story', text: q.body, picture: q.picture ?? null, close: 'OK' }];
  if (q.goal) pages[0].task = q.goal;
  if (q.reward) pages[0].reward = q.reward;
  return pages;
}

/** The pages `v` tells, recording what it told in `t`. A main-story transition (the
 * active quest changed from X to Y) gives each chain quest completed since, in chain order
 * (one that was never shown active first gets its opening), its payoff, then Y's opening
 * with its task on the last page. On load nothing is given; a brand-new game (run 1, day 0,
 * nothing completed) gives only its first quest's opening (`intro`). A completed side quest
 * gives its payoff and reward. Every one-off event fired gives its popup, unless
 * `hideSeen` is on and this career has shown it before; every project completion its own. Under the autopilot (`pilot`)
 * nothing is given and no popup counts as seen; the told record still moves on. */
export function tellPages(v: View, mode: 'load' | 'intro' | 'play', t: Told, opts: { pilot: boolean; hideSeen: boolean; fill: Fill }): StoryPage[] {
  const pages: StoryPage[] = [];
  const quest = v.quest;
  for (const q of v.story) {
    if (t.stories.has(q.id)) continue;
    t.stories.add(q.id);
    if (mode !== 'play') continue;
    if (!q.chain) {
      pages.push(...sideDonePages(q));
      continue;
    }
    if (q.id !== t.lastQuest) for (const text of openingOf(q)) pages.push({ quest: q.id, title: q.title, kind: 'story', text, picture: q.picture ?? null });
    payoffPages(q, pages);
  }
  for (const e of v.event_texts) {
    const key = `${runTag(v)}:${e.id}`;
    if (t.stories.has(key)) continue;
    t.stories.add(key);
    // Seen: popped up now. Not what the autopilot played past, a scenario started with or
    // a load found: the player saw no popup.
    if (mode !== 'play' || opts.pilot) continue;
    const seenBefore = t.seenPopups.has(e.id);
    t.seenPopups.add(e.id);
    if (seenBefore && opts.hideSeen) continue;
    pages.push(...eventPages(e, opts.fill));
  }
  // A project completion pops up as a quest completion does: told once, never on a load,
  // never hidden as already seen (each completion is new).
  for (const p of v.project_done) {
    const key = projectKey(v, p);
    if (t.stories.has(key)) continue;
    t.stories.add(key);
    if (mode === 'play') pages.push(...projectPages(p));
  }
  const fresh = mode === 'intro' && v.run === 1 && v.day === 0 && v.completed_quests.length === 0;
  if (quest && quest.id !== t.lastQuest && !t.opened.has(quest.id)) {
    t.opened.add(quest.id);
    if (mode === 'play' || fresh) {
      for (const text of openingOf(quest)) pages.push({ quest: quest.id, title: quest.title, kind: 'story', text, picture: quest.picture });
      if (pages.length > 0) Object.assign(pages[pages.length - 1], { task: quest.goal });
    }
  }
  t.lastQuest = quest?.id ?? null;
  return opts.pilot ? [] : pages;
}
