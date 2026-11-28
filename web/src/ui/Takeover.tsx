// The takeover view: a large view (the Cortex now; story mode, the achievements grid, the
// Vault later) that covers the middle and right columns while the left column stays. A
// button in the left column (for the text editor, the program panel's) opens it; that
// button again, Esc or ✕ closes it. The game keeps
// running behind it: the columns it covers stay mounted, only hidden, and a slim status
// strip on top shows the day, boredom, a few key resources and pause.

import type { ComponentChildren } from 'preact';
import { useEffect, useState } from 'preact/hooks';
import { dateText, held } from '../format';
import { store, togglePause, type Store } from '../store';
import { BoredomBar, RateCell } from './common';
import { Chip } from './icons';

/** One takeover view. `shown` says whether its left-column button shows now. */
export type Takeover = {
  id: string;
  /** The left-column button's text. */
  label: string;
  /** The heading over the view. */
  title: ComponentChildren;
  /** The view draws edge to edge: no padding, and it fits itself to the body (no scrolling). */
  bleed?: boolean;
  /** Opened from elsewhere (the program panel's Text editor), with no left-column button. */
  noButton?: boolean;
  shown: (s: Store) => boolean;
  render: (s: Store) => ComponentChildren;
};

const changed = new Set<() => void>();

/** Says a takeover's `shown` may have changed, so the page redraws its buttons. */
export function takeoversChanged(): void {
  for (const f of changed) f();
}

/** Re-renders the caller when `takeoversChanged` is called. */
export function useTakeoversChanged(): void {
  const [, set] = useState(0);
  useEffect(() => {
    const f = () => set((n) => n + 1);
    changed.add(f);
    return () => {
      changed.delete(f);
    };
  }, []);
}

/** The resources the status strip shows, when visible, in this order. */
const STRIP_RESOURCES = ['energy', 'credits', 'he3', 'science'];

/** The left column's buttons, one per takeover shown; a second click closes it. */
export function TakeoverButtons({
  s,
  list,
  open,
  toggle,
}: {
  s: Store;
  list: Takeover[];
  open: string | null;
  toggle: (id: string) => void;
}) {
  const shown = list.filter((t) => !t.noButton && t.shown(s));
  if (shown.length === 0) return null;
  return (
    <div class="takeover-buttons">
      {shown.map((t) => (
        <button
          type="button"
          key={t.id}
          class={`takeover-button${open === t.id ? ' active' : ''}`}
          data-takeover-button={t.id}
          aria-pressed={open === t.id}
          title={open === t.id ? 'Close (Esc)' : `Open ${t.label}`}
          onClick={() => toggle(t.id)}
        >
          {t.label}
        </button>
      ))}
    </div>
  );
}

/** The view itself, over the middle and right columns. Esc closes it. */
export function TakeoverView({ s, t, close, banners }: { s: Store; t: Takeover; close: () => void; banners?: ComponentChildren }) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Escape' || e.ctrlKey || e.altKey || e.metaKey) return;
      // Nothing behind the run summary or a story modal acts: closing would try to save a
      // draft to a run that has ended, or that the modal holds (refused).
      if (store.summary || store.storyPages.length > 0) return;
      e.preventDefault();
      // A field being typed in commits as it would on a click away (its change fires on
      // blur): removed while focused, it would drop what was typed.
      (document.activeElement as HTMLElement | null)?.blur?.();
      close();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [close]);
  return (
    <section class="takeover" data-takeover={t.id} aria-label={t.label}>
      <div class="takeover-head">
        <h2>{t.title}</h2>
        <StatusStrip s={s} />
        <button type="button" class="takeover-close" data-takeover-close={t.id} title="Close (Esc)" onClick={close}>
          ✕ Close
        </button>
      </div>
      {/* The notice, which the hidden middle column would hold. */}
      <div class="takeover-banners">{banners}</div>
      <div class={`takeover-body${t.bleed ? ' takeover-bleed' : ''}`}>{t.render(s)}</div>
    </section>
  );
}

/** Day, boredom, a few key resources and pause, so the game behind stays in view. */
function StatusStrip({ s }: { s: Store }) {
  const v = s.view;
  const paused = s.speed === 0;
  return (
    <div class="takeover-strip" data-testid="takeover-strip">
      <span class="takeover-day">{dateText(v.day, v.days_per_year)}</span>
      <span class="takeover-bored" title="Boredom">
        <span class="status-label">Boredom</span>
        <BoredomBar v={v} />
      </span>
      {STRIP_RESOURCES.flatMap((id) => v.resources.filter((r) => r.id === id && r.visible)).map((r) => (
        <span key={r.id} class="takeover-res" data-strip-res={r.id} title={r.name}>
          <Chip id={r.id} title={r.name} /> {held(r.amount)}
          <span class="muted">
            {' '}
            <RateCell r={r} compact />
          </span>
        </span>
      ))}
      <button
        type="button"
        class="takeover-pause"
        id="takeover-pause"
        aria-pressed={paused}
        title={paused ? 'Resume (space)' : 'Pause (space)'}
        onClick={togglePause}
      >
        {paused ? '▶ Resume' : `❚❚ Pause (${s.speed}x)`}
      </button>
    </div>
  );
}
