// Small shared pieces: placeholder icons, bars, collapsible sections, amount lists.

import type { ComponentChildren } from 'preact';
import { useState } from 'preact/hooks';
import type { Amounts, View } from '../engine';
import { num } from '../format';

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

/** A small "new" accent: rendered as a class on the element it marks. */
export function freshClass(fresh: Set<string>, key: string): string {
  return fresh.has(key) ? ' is-new' : '';
}
