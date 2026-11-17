// Small shared pieces: resource chips, bars, collapsible sections, amount lists.

import type { ComponentChildren } from 'preact';
import { useEffect, useRef, useState } from 'preact/hooks';
import type { Amounts, Fx, View } from '../engine';
import { num, perDay } from '../format';
import { justMoved } from '../store';
import { Chip } from './icons';

/** A resource's chip, its name as the tooltip. */
export function ResChip({ v, id, size }: { v: View; id: string; size?: number }) {
  return <Chip id={id} title={name(v, id)} size={size} />;
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

/** How long an open section must be on screen before its unread items count as read. */
const READ_MS = 1000;

/** A header that folds its body; open by default. With `unread`, the header carries an
 * unread count (like unread mail), and `onRead` runs once the section has been open and on
 * screen, with the page visible, for about a second; a folded section keeps its count. */
export function Section({
  title,
  children,
  open: initial = true,
  dark = false,
  testid,
  unread = 0,
  onRead,
}: {
  title: ComponentChildren;
  children: ComponentChildren;
  open?: boolean;
  dark?: boolean;
  testid?: string;
  unread?: number;
  onRead?: () => void;
}) {
  const [open, setOpen] = useState(initial);
  const body = useRef<HTMLDivElement>(null);
  const [onScreen, setOnScreen] = useState(false);
  const read = useRef(onRead);
  read.current = onRead;
  const tracks = onRead !== undefined;
  useEffect(() => {
    const el = body.current;
    if (!open || !tracks || !el) {
      setOnScreen(false);
      return;
    }
    const io = new IntersectionObserver(([e]) => setOnScreen(e.isIntersecting));
    io.observe(el);
    return () => io.disconnect();
  }, [open, tracks]);
  const pending = open && onScreen && unread > 0;
  useEffect(() => {
    if (!pending) return;
    // Seen time counts only while the page is visible.
    let seenMs = 0;
    const step = 250;
    const t = window.setInterval(() => {
      if (document.visibilityState === 'visible') seenMs += step;
      if (seenMs >= READ_MS) {
        clearInterval(t);
        read.current?.();
      }
    }, step);
    return () => clearInterval(t);
  }, [pending]);
  return (
    <section class={`fold ${dark ? 'fold-dark' : ''}`} data-testid={testid} data-unread={tracks ? unread : undefined}>
      <button type="button" class="fold-head" aria-expanded={open} onClick={() => setOpen(!open)}>
        <span class="caret">{open ? '▼' : '▶'}</span> {title}
        {unread > 0 && (
          <span class="unread" title={`${unread} unread`}>
            {unread}
          </span>
        )}
      </button>
      {open && (
        <div class="fold-body" ref={body}>
          {children}
        </div>
      )}
    </section>
  );
}

/** A resource's net rate while it sits at its cap: muted, marked capped (the surplus the
 * cap throws away). `compact` (the narrow Resources column, where several can be capped
 * at once): "+6" and a small "cap" tag, no wider than an ordinary "+0.2/day". */
export function CappedRate({ net, compact = false, spare = false }: { net: Fx; compact?: boolean; spare?: boolean }) {
  const rate = perDay(net);
  return (
    <span class={`capped-rate${compact ? ' compact' : ''}${spare ? ' spare' : ''}`} title={`At its cap: ${rate} is lost each day`} data-capped>
      {compact ? rate.replace('/day', '') : rate} <span class="cap-mark">{compact ? 'cap' : '(capped)'}</span>
    </span>
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
          <ResChip v={v} id={res} /> {`${sign < 0 ? '−' : '+'}${num(amt)} ${name(v, res)}${suffix}`}
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
              <ResChip v={v} id={res} /> {name(v, res)}
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
