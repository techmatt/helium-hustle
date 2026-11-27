// The Projects panel: each unlocked project's card, its funding steps (held buttons
// repeat) and progress. Shows what the view has.

import type { ComponentChildren } from 'preact';
import { useEffect, useState } from 'preact/hooks';
import type { ProjectView, View } from '../engine';
import { num } from '../format';
import { act, justMoved, seen, store, type Store } from '../store';
import { Bar, ResChip, Section, freshClass, name, numBelow, useHideCompleted } from './common';
import { projectArt } from './cardArt';
import { ScopeIcon } from './icons';

export function Projects({ s }: { s: Store }) {
  const v = s.view;
  // Timelines go unnamed until the swarm's reveal (1.11 starts the calendar): nothing
  // upcoming is telegraphed (ui.md).
  const revealed = v.saga.timeline !== null;
  const scopes: [ProjectView['scope'], string, string][] = [
    ['run', 'Strategic (this run)', 'This run only: progress resets on retirement.'],
    revealed
      ? ['timeline', 'Global (this timeline)', 'Progress and completion carry across retirements until the timeline ends; funding rates reset, so set them again each run.']
      : ['timeline', 'Global', 'Progress and completion carry across retirements; funding rates reset, so set them again each run.'],
    ['forever', 'Forever', 'Progress and completion are kept for good.'],
  ];
  // Finished one-offs show as done cards unless hidden (the default).
  const hide = useHideCompleted('hh.hideCompleted.projects', true);
  return (
    <div class="panel" data-panel="projects">
      <div class="panel-head">
        <h2>Projects</h2>
        {hide.box}
      </div>
      <p class="muted">Fund a project by setting a daily rate per resource; it drains that much each day until done.</p>
      {!v.projects.some((p) => p.unlocked) && <p class="muted hint">No project to fund right now.</p>}
      {scopes.map(([scope, title, note]) => {
        // What can be funded now, and finished one-offs unless hidden; never a locked card (ui.md).
        const items = v.projects.filter((p) => p.scope === scope && (p.unlocked || p.completed) && hide.shows(p.id, p.completed && !p.unlocked));
        if (items.length === 0) return null;
        return (
          <Section key={scope} title={`${title} (${items.length})`} dark>
            <p class="muted small">{note}</p>
            <div class="cards wide">
              {items.map((p) => (
                <Project key={p.id} v={v} p={p} fresh={s.fresh} />
              ))}
            </div>
          </Section>
        );
      })}
    </div>
  );
}

// Holding a funding −/+ keeps stepping: one step on press, repeating after HOLD_DELAY_MS
// every HOLD_EVERY_MS; held past HOLD_FAST_MS each repeat is 5 steps, past HOLD_FASTER_MS 10.
const HOLD_DELAY_MS = 400;
const HOLD_EVERY_MS = 100;
const HOLD_FAST_MS = 1500;
const HOLD_FASTER_MS = 3000;

/** A button that steps once on press and keeps stepping while held. `step(n)` takes `n`
 *  steps and returns false at its bound (0 or the cap), which ends the hold. */
function HoldButton({ disabled, step, children }: { disabled: boolean; step: (n: number) => boolean; children: ComponentChildren }) {
  const [timer] = useState<{ id?: number; t0: number }>({ t0: 0 });
  const stop = () => {
    if (timer.id !== undefined) window.clearTimeout(timer.id);
    timer.id = undefined;
  };
  const tick = () => {
    const held = performance.now() - timer.t0;
    const n = held >= HOLD_FASTER_MS ? 10 : held >= HOLD_FAST_MS ? 5 : 1;
    timer.id = step(n) ? window.setTimeout(tick, HOLD_EVERY_MS) : undefined;
  };
  const down = (e: PointerEvent) => {
    // Not a press landing just after a modal moved (App's click guard sees only clicks).
    if (e.button !== 0 || justMoved()) return;
    stop();
    // Captured, the press stays the button's when a step moves it (a card's line coming
    // or a row of cards reflowing): it ends on release, not because the button slid away
    // under a still pointer (122: a hold from 0 gave one step).
    (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId);
    timer.t0 = performance.now();
    if (step(1)) timer.id = window.setTimeout(tick, HOLD_DELAY_MS);
  };
  useEffect(() => stop, []);
  return (
    <button
      type="button"
      class="tiny"
      disabled={disabled}
      onPointerDown={down}
      onPointerUp={stop}
      onPointerCancel={stop}
      onLostPointerCapture={stop}
      // A click no pointer press began (keyboard, assistive tech, a script's click()) steps
      // once; a mouse or touch click's step already came on its press.
      onClick={(e) => e.detail === 0 && step(1)}
    >
      {children}
    </button>
  );
}

export function Project({ v, p, fresh }: { v: View; p: ProjectView; fresh: Set<string> }) {
  const key = `project:${p.id}`;
  // Moves the rate by `by` (clamped to 0..max); false once it sits at that bound.
  const step = (res: string, by: number): boolean => {
    // From the engine's rate now, not this render's: quick clicks and held repeats must each count.
    const cur = store.view.projects.find((x) => x.id === p.id) ?? p;
    const now = Number(cur.rates[res] ?? '0');
    const max = Number(cur.max_rates[res] ?? '0');
    const next = Math.max(0, Math.min(max, now + by));
    // Six places, as the engine reads a number: float sums of a max like 1.666667 carry
    // more (0.6666669999999999), which it refuses.
    if (next !== now && !act({ type: 'set_project_rate', project: p.id, resource: res, rate: next.toFixed(6) })) return false;
    return next > 0 && next < max;
  };
  const locked = !p.unlocked && !p.completed;
  const funded = p.unlocked && Object.values(p.rates).some((r) => Number(r) > 0);
  const state = p.completed && !p.unlocked ? ' done' : locked ? ' locked' : '';
  const art = projectArt(p.id);
  return (
    <div class={`card project${state}${freshClass(fresh, key)}`} data-id={p.id} onMouseEnter={() => seen(key)}>
      <div class="card-head">
        <h3>{p.name}</h3>
        {p.completed && !p.repeatable && <span class="ok">✓ Complete</span>}
        <ScopeIcon keep={p.keep} />
      </div>
      {/* The picture sits under the title beside the text, as on a building card; the
          funding rows run the card's full width below. */}
      <div class="card-body">
        {art && <img class="card-art" src={art} alt="" width={96} height={96} draggable={false} />}
        <div class="card-text">
          <p class="desc">{p.description}</p>
          {locked && <p class="muted small">Locked: {p.unlock_text}</p>}
          {p.reward_text.map((t, i) => (
            <p class="small" key={i}>
              Reward: {t}
            </p>
          ))}
          {p.repeatable && p.completions > 0 && (
            <p class="ok small">
              Completed {p.completions} {p.completions === 1 ? 'time' : 'times'}
            </p>
          )}
          {/* Always there on a card that can be funded, so the first step doesn't grow the
              card and move its rows (and the button being pressed) down. */}
          {p.unlocked && (
            <p class="muted small" data-testid="project-status">
              {!funded
                ? 'Not funded'
                : p.days_to_done !== null
                ? `Done in ${p.days_to_done} ${p.days_to_done === 1 ? 'day' : 'days'}`
                : Object.keys(p.costs).every((res) => Number(p.rates[res] ?? '0') > 0 || Number(p.progress[res] ?? '0') >= Number(p.costs[res]))
                  ? 'Stalled: the stock is supplying nothing of a cost'
                  : 'Not every cost is funded'}
            </p>
          )}
        </div>
      </div>
      {p.unlocked &&
        Object.entries(p.costs).map(([res, cost]) => {
          const got = p.progress[res] ?? '0';
          const rate = p.rates[res] ?? '0';
          const max = p.max_rates[res] ?? '0';
          const actual = p.actual_rates[res] ?? rate;
          const short = Number(rate) > 0 && Number(actual) < Number(rate);
          return (
            <div class="proj-row" key={res}>
              <span class="proj-res">
                <ResChip v={v} id={res} /> {name(v, res)}
              </span>
              <Bar frac={Number(got) / Number(cost)} label={`${numBelow(got, cost)} / ${num(cost)}`} />
              {short ? (
                <span class="proj-rate" data-short title={`The stock covered only ${num(actual)} of the ${num(rate)} a day set`}>
                  <span class="neg">{num(actual)}</span> / {num(rate)} per day
                </span>
              ) : (
                <span class={`proj-rate${Number(rate) > 0 ? ' ok' : ''}`}>{num(rate)}/day</span>
              )}
              <HoldButton disabled={Number(rate) <= 0} step={(n) => step(res, -n)}>
                −
              </HoldButton>
              <HoldButton disabled={Number(rate) >= Number(max)} step={(n) => step(res, n)}>
                +
              </HoldButton>
              <button type="button" class="tiny" title={`Fund at the most (${num(max)}/day)`} onClick={() => step(res, Number(max))}>
                Max
              </button>
            </div>
          );
        })}
    </div>
  );
}
