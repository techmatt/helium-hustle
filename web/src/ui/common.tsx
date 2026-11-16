// Small shared pieces: placeholder icons, bars, collapsible sections, amount lists.

import type { ComponentChildren } from 'preact';
import { useEffect, useRef, useState } from 'preact/hooks';
import type { Amounts, View } from '../engine';
import { num } from '../format';
import { justMoved } from '../store';

/** Placeholder icon colors (real art is a later round). */
const RESOURCE_COLORS: Record<string, string> = {
  energy: '#f5d000',
  regolith: '#8b5a2b',
  ice: '#aee9ff',
  he3: '#7b7bff',
  titanium: '#c4c4c4',
  circuits: '#35c4a4',
  propellant: '#4aa3f0',
  credits: '#2ecc40',
  science: '#b388ff',
  land: '#4caf50',
  boredom: '#8a8a8a',
  processors: '#d63ad6',
};

const NAV_COLORS: Record<string, string> = {
  buildings: '#4aa3f0',
  commands: '#f0a020',
  launch_pads: '#f08c1e',
  research: '#8a5cf0',
  projects: '#2ecc71',
  ideologies: '#e84c4c',
  adversaries: '#c93030',
  stats: '#66d0d0',
  story: '#f5d000',
  retirement: '#9b4dd6',
  options: '#9aa0a8',
};

/** True axis colors, behind white text (an ideology tag). As text, an axis takes its
 * class (`axis-<id>` in style.css), which has a readable variant per theme. */
export const AXIS_COLORS: Record<string, string> = {
  nationalist: '#c62828',
  humanist: '#2e7d32',
  rationalist: '#1565c0',
};

export function color(id: string): string {
  return RESOURCE_COLORS[id] ?? NAV_COLORS[id] ?? '#999';
}

export function Square({ id, size = 12 }: { id: string; size?: number }) {
  return <span class="sq" style={{ background: color(id), width: `${size}px`, height: `${size}px` }} />;
}

/** A fill bar; `frac` is clamped to 0..1. */
export function Bar({ frac, tone = 'green', label }: { frac: number; tone?: string; label?: string }) {
  const f = Math.max(0, Math.min(1, Number.isFinite(frac) ? frac : 0));
  return (
    <span class={`bar bar-${tone}`}>
      <span class="fill" style={{ width: `${f * 100}%` }} />
      {label !== undefined && <span class="bar-label">{label}</span>}
    </span>
  );
}

/** A header that folds its body; open by default. */
export function Section({
  title,
  children,
  open: initial = true,
  dark = false,
  testid,
}: {
  title: ComponentChildren;
  children: ComponentChildren;
  open?: boolean;
  dark?: boolean;
  testid?: string;
}) {
  const [open, setOpen] = useState(initial);
  return (
    <section class={`fold ${dark ? 'fold-dark' : ''}`} data-testid={testid}>
      <button type="button" class="fold-head" aria-expanded={open} onClick={() => setOpen(!open)}>
        <span class="caret">{open ? '▼' : '▶'}</span> {title}
      </button>
      {open && <div class="fold-body">{children}</div>}
    </section>
  );
}

/** Whether the nav tile (and so the panel) `id` shows. */
export function navShown(v: View, id: string): boolean {
  return v.nav.some((t) => t.id === id && t.visible);
}

export function name(v: View, id: string): string {
  return v.names[id] ?? id;
}

/** Lines like "+5 Energy/day" (`sign` 1) or "−2 Energy/day" (`sign` −1). */
export function FlowLines({ v, amounts, sign, suffix = '/day' }: { v: View; amounts: Amounts; sign: 1 | -1; suffix?: string }) {
  return (
    <>
      {Object.entries(amounts).map(([res, amt]) => (
        <div class={`flow ${sign < 0 ? 'neg' : 'pos'}`} key={res}>
          <Square id={res} /> {`${sign < 0 ? '−' : '+'}${num(amt)} ${name(v, res)}${suffix}`}
        </div>
      ))}
    </>
  );
}

/** A cost table: resource, amount; red where short. */
export function CostRows({ v, cost, short }: { v: View; cost: Amounts; short: string[] }) {
  return (
    <table class="costs">
      <tbody>
        {Object.entries(cost).map(([res, amt]) => (
          <tr key={res} class={short.includes(res) ? 'short' : ''}>
            <td>
              <Square id={res} /> {name(v, res)}
            </td>
            <td class="num">{num(amt)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

/** The card's own controls, which keep their jobs inside a clickable card. */
const CONTROLS = 'button, a, input, select, textarea, label';

/** Props that make a card the button for its action (building and research cards): a click
 * anywhere but its own controls calls `buy(times)` (Shift: `shiftTimes`), as do Enter and
 * space when the card has keyboard focus. `buy` must read the engine's view at call time,
 * not a render's, so two quick clicks count twice and never more. */
export function cardButton(buy: (times: number) => void, shiftTimes = 1) {
  return {
    role: 'button' as const,
    tabIndex: 0,
    onClick: (e: MouseEvent) => {
      if ((e.target as HTMLElement).closest(CONTROLS)) return;
      // The second click of a double-click on a story's OK or the summary's Continue lands
      // on whatever slid under the pointer: not a buy. (Pointer clicks only: a click with
      // no pointer behind it, detail 0, was aimed.)
      if (e.detail > 0 && justMoved()) return;
      buy(e.shiftKey ? shiftTimes : 1);
    },
    // A mouse press neither focuses the card nor selects text: focus would turn the next
    // space (the pause key) into a second buy, and a double- or Shift-click would select
    // text (its clicks still buy). Tab still focuses it for the keyboard.
    onMouseDown: (e: MouseEvent) => {
      if (!(e.target as HTMLElement).closest(CONTROLS)) e.preventDefault();
    },
    onKeyDown: (e: KeyboardEvent) => {
      if (e.target === e.currentTarget && (e.key === 'Enter' || e.key === ' ')) {
        e.preventDefault();
        e.stopPropagation(); // the space key is also pause
        buy(e.shiftKey ? shiftTimes : 1);
      }
    },
  };
}

/** A short hint on a card that clears itself (why a grey card buys nothing). */
export function useHint(): [string | null, (text: string) => void] {
  const [hint, setHint] = useState<string | null>(null);
  const timer = useRef<number | undefined>(undefined);
  useEffect(() => () => clearTimeout(timer.current), []);
  const show = (text: string) => {
    setHint(text);
    clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setHint(null), 1800);
  };
  return [hint, show];
}

/** A small "new" accent: rendered as a class on the element it marks. */
export function freshClass(fresh: Set<string>, key: string): string {
  return fresh.has(key) ? ' is-new' : '';
}
