// Right column: program tabs (processors, command lines) and Events (Story, Completed,
// Recent Activity).

import type { ComponentChildren } from 'preact';
import { useState } from 'preact/hooks';
import type { Activity, View } from '../engine';
import { num, storyBody } from '../format';
import { daysPerWake } from '../clock';
import { act, eventItems, markRead, store, unreadIn, type EventSection, type Store } from '../store';
import { Quest, ResChip, Section, name } from './common';
import { Chip } from './icons';
import { AddLogic } from './ProgramLogic';
import { Library, type LibraryMode } from './Library';
import { canAddElse } from './programEdit';
import { editProgram } from './programStore';
import { ProgramLines } from './ProgramLines';

export function Right({
  s,
  program,
  setProgram,
}: {
  s: Store;
  program: number;
  setProgram: (i: number) => void;
}) {
  return (
    <aside class="col-right">
      {/* The main-story quest's own slot: nothing else is ever drawn here. */}
      {s.view.quest && (
        <div class="quest-pin" data-testid="quest-pin">
          <div class="quest-pin-label">Main quest</div>
          <Quest v={s.view} />
        </div>
      )}
      <Programs s={s} program={program} setProgram={setProgram} />
      <Events v={s.view} />
    </aside>
  );
}

function Programs({
  s,
  program,
  setProgram,
}: {
  s: Store;
  program: number;
  setProgram: (i: number) => void;
}) {
  const v = s.view;
  const p = v.programs[program];
  const [library, setLibrary] = useState<LibraryMode | null>(null);
  if (!p) return null;
  const entries = p.entries;
  const perRedraw = daysPerWake(s.speed);
  const span = perRedraw > 1;
  const procs = (by: number) => {
    const now = store.view.programs[program]?.processors ?? 0;
    act({ type: 'set_processors', program, processors: Math.max(0, now + by) });
  };
  return (
    <div class="programs" data-testid="programs">
      <div class="prog-tabs" role="tablist">
        {v.programs.map((q, i) => (
          <button
            type="button"
            role="tab"
            key={i}
            data-program={i}
            aria-selected={i === program}
            class={q.processors > 0 ? 'running' : ''}
            onClick={() => setProgram(i)}
          >
            {q.processors > 0 ? '● ' : ''}
            {i + 1}
          </button>
        ))}
      </div>
      <div class="prog-procs">
        <span>
          ⚙ {p.processors} assigned
          {p.running < p.processors && <span class="neg" title="Data Centers short of upkeep: the rest wait until it is paid">, {p.running} running</span>} (
          {v.processors_free} free)
        </span>
        <button
          type="button"
          class="tiny"
          data-testid="proc-minus"
          disabled={p.processors === 0}
          onClick={() => procs(-1)}
        >
          −
        </button>
        <button
          type="button"
          class="tiny"
          data-testid="proc-plus"
          disabled={v.processors_free === 0}
          onClick={() => procs(1)}
        >
          +
        </button>
        <button type="button" class="tiny" title="Clear this program" disabled={entries.length === 0} onClick={() => editProgram(program, () => [])}>
          Reset
        </button>
        <button
          type="button"
          class="tiny"
          data-testid="loadout-save"
          title="Save this program to the library (kept across retirements)"
          aria-pressed={library === 'save'}
          disabled={entries.length === 0}
          onClick={() => setLibrary(library === 'save' ? null : 'save')}
        >
          Save
        </button>
        <button
          type="button"
          class="tiny"
          data-testid="loadout-load"
          title="Load a saved program into this one"
          aria-pressed={library === 'load'}
          onClick={() => setLibrary(library === 'load' ? null : 'load')}
        >
          Load{v.library.length > 0 ? ` (${v.library.length})` : ''}
        </button>
      </div>
      {library && <Library v={v} program={program} mode={library} close={() => setLibrary(null)} />}
      {entries.length === 0 && (
        <p class="muted hint">Empty. Add commands from the Commands panel; each processor runs one step a day.</p>
      )}
      {span && entries.length > 0 && (
        <p class="muted hint" data-testid="lines-span">
          {perRedraw} days per redraw: lines show what ran over them.
        </p>
      )}
      <ProgramLines s={s} program={program} />
      <AddLogic v={v} canElse={canAddElse(entries)} add={(e) => editProgram(program, (now) => [...now, e])} />
    </div>
  );
}

/** Events other than the main quest, each section with its unread count. */
function Events({ v }: { v: View }) {
  const keys = eventItems(v);
  const unread = (sec: EventSection) => unreadIn(v, sec);
  const story = unread('story');
  const completed = unread('completed');
  const activity = unread('activity');
  // Newest first.
  const texts = v.event_texts.map((e, i) => ({ e, key: keys.story[i] })).reverse();
  const recent = v.activity.map((a, i) => ({ a, key: keys.activity[i] })).reverse();
  return (
    <div class="events" data-testid="events">
      <h2>Events</h2>
      <Section
        title={`Story (${texts.length})`}
        dark
        testid="events-story"
        unread={story.length}
        onRead={() => markRead(story)}
      >
        {texts.length === 0 && <p class="muted hint">No events this run yet.</p>}
        {texts.map(({ e, key }) => (
          <div key={key} class={`event-item${story.includes(key) ? ' is-unread' : ''}`} data-event={e.id}>
            <div class="event-title">{e.title}</div>
            <div class="muted">{e.summary}</div>
            <div class="event-body">{storyBody(e)}</div>
          </div>
        ))}
      </Section>
      <Section
        title={`Completed (${v.story.length})`}
        dark
        open={false}
        testid="events-completed"
        unread={completed.length}
        onRead={() => markRead(completed)}
      >
        <ul class="plain">
          {v.story.map((e) => (
            <li key={e.id} data-quest={e.id} class={completed.includes(e.id) ? 'is-unread' : ''}>
              ✓ {e.title}
            </li>
          ))}
        </ul>
      </Section>
      <Section
        title={`Recent Activity (${recent.length})`}
        dark
        testid="events-activity"
        unread={activity.length}
        onRead={() => markRead(activity)}
      >
        <ul class="plain activity">
          {recent.map(({ a, key }) => (
            <li key={key} class={`act-${tone(a)}${activity.includes(key) ? ' is-unread' : ''}`} data-kind={a.kind}>
              Day {num(a.day)}: {describe(v, a)}
            </li>
          ))}
        </ul>
      </Section>
    </div>
  );
}

function tone(a: Activity): 'good' | 'bad' | 'info' {
  if (a.kind === 'speculator_burst' || a.kind === 'rival_dump' || a.kind === 'cargo_lost') return 'bad';
  if (a.kind === 'event') return 'info';
  return 'good';
}

function describe(v: View, a: Activity): ComponentChildren {
  switch (a.kind) {
    case 'shipment':
      return (
        <>
          <ResChip v={v} id={a.id} /> {name(v, a.id)} × {num(a.quantity ?? '0')} → <ResChip v={v} id="credits" /> {num(a.amount ?? '0')} credits
        </>
      );
    case 'speculator_burst':
      return (
        <>
          Speculator surge: {num(a.amount ?? '0')} speculators target <ResChip v={v} id={a.id} /> {name(v, a.id)}
        </>
      );
    case 'rival_dump': {
      const r = v.rivals.find((x) => x.id === a.id);
      return (
        <>
          <Chip id={a.id} title={name(v, a.id)} /> {name(v, a.id)} flooded the {r ? <ResChip v={v} id={r.good} /> : null} {r ? name(v, r.good) : '?'} market
        </>
      );
    }
    case 'cargo_lost':
      return (
        <>
          Pad {num(a.amount ?? '0')}: <ResChip v={v} id={a.id} /> {num(a.quantity ?? '0')} {name(v, a.id)} lost switching cargo (stockpile full)
        </>
      );
    case 'quest':
      return `Quest complete: ${name(v, a.id)}`;
    case 'achievement':
      return `Achievement: ${name(v, a.id)}`;
    case 'research':
      return `Researched ${name(v, a.id)}`;
    case 'project':
      return `Project complete: ${name(v, a.id)}`;
    case 'unlock':
      // The one unlock logged so far: the program instructions (`program_logic`).
      return 'Programs can now use Build, Buy Land and verb lines';
    default:
      return name(v, a.id);
  }
}
