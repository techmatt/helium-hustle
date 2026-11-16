// Right column: program tabs (processors, command lines) and Events (Story, Completed,
// Recent Activity).

import { useState } from 'preact/hooks';
import type { Activity, Entry, View } from '../engine';
import { num } from '../format';
import { daysPerWake } from '../clock';
import { act, store, type Store } from '../store';
import { Bar, Section, name } from './common';

export function Right({ s, program, setProgram }: { s: Store; program: number; setProgram: (i: number) => void }) {
  return (
    <aside class="col-right">
      <Programs s={s} program={program} setProgram={setProgram} />
      <Events v={s.view} />
    </aside>
  );
}

/** Rewrites a program's lines from what the engine holds now (not the last render:
 * two quick clicks must both land). */
export function editProgram(program: number, edit: (entries: Entry[]) => Entry[]): void {
  const now = store.view.programs[program]?.entries ?? [];
  act({ type: 'set_program', program, entries: edit(now) });
}

function Programs({ s, program, setProgram }: { s: Store; program: number; setProgram: (i: number) => void }) {
  const v = s.view;
  const p = v.programs[program];
  const [drag, setDrag] = useState<number | null>(null);
  if (!p) return null;
  const entries = p.entries;
  // Above 10x one redraw covers several days: the pointer and "ran today" of the last
  // of them would hop around the queue (even backward), so each line instead shows what
  // it did over those days, with no pointer.
  const perRedraw = daysPerWake(s.speed);
  const span = perRedraw > 1;
  const spanOutcomes = s.lines?.lines[program];
  const move = (from: number, to: number) => {
    if (from === to) return;
    editProgram(program, (now) => {
      const next = [...now];
      const [e] = next.splice(from, 1);
      next.splice(to, 0, e);
      return next;
    });
  };
  const procs = (by: number) => {
    const now = store.view.programs[program]?.processors ?? 0;
    act({ type: 'set_processors', program, processors: Math.max(0, now + by) });
  };
  const bump = (i: number, by: number) =>
    editProgram(program, (now) => now.map((x, j) => (j === i ? { ...x, count: Math.max(1, x.count + by) } : x)));
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
          ⚙ {p.processors} assigned ({v.processors_free} free)
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
      </div>
      {entries.length === 0 && (
        <p class="muted hint">Empty. Add commands from the Commands panel; each processor runs one step a day.</p>
      )}
      {span && entries.length > 0 && (
        <p class="muted hint" data-testid="lines-span">
          {perRedraw} days per redraw: lines show what ran over them.
        </p>
      )}
      <ol class="lines">
        {entries.map((e, i) => {
          const outcome = span ? (spanOutcomes?.[i] ?? null) : p.outcomes[i];
          // No processor on the program: no line runs next, so none is framed.
          const current = !span && p.processors > 0 && i === p.entry;
          // Part way through its runs: that share; otherwise full once it ran today.
          const frac = current && p.step > 0 ? p.step / e.count : outcome ? 1 : 0;
          return (
            <li
              key={`${i}-${e.command}`}
              class={`line ${outcome ? `out-${outcome}` : ''} ${current ? 'current' : ''}`}
              data-line={i}
              data-command={e.command}
              draggable
              onDragStart={(ev) => {
                setDrag(i);
                ev.dataTransfer?.setData('text/plain', String(i));
              }}
              onDragOver={(ev) => ev.preventDefault()}
              onDrop={(ev) => {
                ev.preventDefault();
                if (drag !== null) move(drag, i);
                setDrag(null);
              }}
              // The name can be cut short at narrow widths: the tooltip carries it whole.
              title={`${name(v, e.command)} (x${e.count}): ${
                span
                  ? outcome
                    ? `last run since the last redraw: ${outcome}`
                    : 'not run since the last redraw'
                  : outcome
                    ? `last run today: ${outcome}`
                    : 'not run today'
              }`}
            >
              <span class="grip" aria-hidden>
                ⠿
              </span>
              {/* The current line is marked by its frame (style.css `.line.current`), never by
                  text in the name, so no line's text moves as the pointer walks. */}
              <span class="line-name">
                {name(v, e.command)} (x{e.count})
              </span>
              <Bar frac={frac} tone={outcome === 'failed' ? 'red' : outcome === 'skipped' ? 'gray' : 'green'} />
              <button type="button" class="tiny" disabled={e.count <= 1} onClick={() => bump(i, -1)}>
                −
              </button>
              <button type="button" class="tiny" onClick={() => bump(i, 1)}>
                +
              </button>
              <button type="button" class="tiny" title="Remove" onClick={() => editProgram(program, (now) => now.filter((_, j) => j !== i))}>
                ×
              </button>
            </li>
          );
        })}
      </ol>
    </div>
  );
}

function Events({ v }: { v: View }) {
  const q = v.quest;
  const recent = [...v.activity].reverse();
  return (
    <div class="events" data-testid="events">
      <h2>Events</h2>
      <Section title={`Story${q ? ' (1)' : ''}`} dark testid="events-story">
        {q && <Quest v={v} />}
      </Section>
      <Section title={`Completed (${v.story.length})`} dark open={false} testid="events-completed">
        <ul class="plain">
          {v.story.map((e) => (
            <li key={e.id} data-quest={e.id}>
              ✓ {e.title}
            </li>
          ))}
        </ul>
      </Section>
      <Section title={`Recent Activity (${recent.length})`} dark testid="events-activity">
        <ul class="plain activity">
          {recent.map((a, i) => (
            <li key={`${a.day}-${a.kind}-${a.id}-${i}`} class={`act-${tone(a)}`} data-kind={a.kind}>
              Day {num(a.day)}: {describe(v, a)}
            </li>
          ))}
        </ul>
      </Section>
    </div>
  );
}

export function Quest({ v }: { v: View }) {
  const q = v.quest;
  if (!q) return null;
  const subs = q.sub_objectives;
  const done = subs.filter((x) => x.done).length;
  return (
    <div class="quest" data-testid="quest" data-quest={q.id}>
      <div class="quest-title">
        {q.title}
        {subs.length > 0 && ` [${done}/${subs.length}]`}
      </div>
      <div class="quest-summary">{q.summary}</div>
      {q.progress && subs.length === 0 && (
        <Bar frac={Number(q.progress.now) / Number(q.progress.target)} label={`${num(q.progress.now)} / ${num(q.progress.target)}`} />
      )}
      {subs.map((x) => (
        <div key={x.id} class={`sub ${x.done ? 'done' : ''}`}>
          {x.done ? '✓' : '○'} {x.label}
          {!x.done && x.progress && (
            <span class="muted">
              {' '}
              {num(x.progress.now)} / {num(x.progress.target)}
            </span>
          )}
        </div>
      ))}
    </div>
  );
}

function tone(a: Activity): 'good' | 'bad' | 'info' {
  if (a.kind === 'speculator_burst' || a.kind === 'rival_dump') return 'bad';
  if (a.kind === 'event') return 'info';
  return 'good';
}

function describe(v: View, a: Activity): string {
  switch (a.kind) {
    case 'shipment':
      return `${name(v, a.id)} × ${num(a.quantity ?? '0')} → ${num(a.amount ?? '0')} credits`;
    case 'speculator_burst':
      return `Speculator surge: ${num(a.amount ?? '0')} speculators target ${name(v, a.id)}`;
    case 'rival_dump': {
      const r = v.rivals.find((x) => x.id === a.id);
      return `${name(v, a.id)} flooded the ${r ? name(v, r.good) : '?'} market`;
    }
    case 'quest':
      return `Quest complete: ${name(v, a.id)}`;
    case 'achievement':
      return `Achievement: ${name(v, a.id)}`;
    case 'research':
      return `Researched ${name(v, a.id)}`;
    case 'project':
      return `Project complete: ${name(v, a.id)}`;
    default:
      return name(v, a.id);
  }
}
