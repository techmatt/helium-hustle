// Small shared pieces: resource chips, bars, collapsible sections, amount lists.

import type { ComponentChildren } from 'preact';
import { useEffect, useRef, useState } from 'preact/hooks';
import type { Amounts, FlowRow, Fx, ResourceView, ShortView, View } from '../engine';
import { num, perDay } from '../format';
import { justMoved } from '../store';
import { Chip } from './icons';

/** A resource's chip, its name as the tooltip. */
export function ResChip({ v, id, size }: { v: View; id: string; size?: number }) {
  return <Chip id={id} title={name(v, id)} size={size} />;
}

/** Whether a resource going up is bad news (Boredom): its rates read the other way round. */
export function risingIsBad(id: string): boolean {
  return id === 'boredom';
}

/** The main-story quest: title, summary, and its progress or sub-objectives. */
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
export function CappedRate({ net, lost, compact = false, spare = false }: { net: Fx; lost: Fx; compact?: boolean; spare?: boolean }) {
  const rate = perDay(net);
  const amt = num(lost);
  const title = spare ? `At its cap: ${amt} a day of spare power goes unused` : `At its cap: ${amt} a day is lost`;
  // (`lost` is the engine's overflow, also when the cap is reached within the day.)
  return (
    <span class={`capped-rate${compact ? ' compact' : ''}${spare ? ' spare' : ''}`} title={title} data-capped>
      {compact ? rate.replace('/day', '') : rate} <span class="cap-mark">{compact ? 'cap' : '(capped)'}</span>
    </span>
  );
}

/** A rate below the stock's change: demand the day leaves unmet (a brownout reads its
 * shortfall, not 0). The actual change is in the tooltip. */
export function ShortRate({ rate, unmet, change, compact = false }: { rate: Fx; unmet: Fx; change: Fx; compact?: boolean }) {
  const shown = perDay(rate);
  const title = `Short: ${num(unmet)} a day of demand goes unmet; the stock changes ${perDay(change)}`;
  return (
    <span class={`short-rate${compact ? ' compact' : ''}`} title={title} data-short>
      {compact ? shown.replace('/day', '') : shown} <span class="cap-mark">{compact ? 'short' : '(short)'}</span>
    </span>
  );
}

/** Whether a resource's rate reads as capped: production thrown away at its cap (never
 * boredom, whose cap ends the run). */
export function isCapped(r: ResourceView): boolean {
  return r.id !== 'boredom' && Number(r.overflow) > 0;
}

/** A resource's engine rate as shown in Resources and the status bar: capped (the
 * surplus, muted; Energy's green as spare power), short (demand left unmet) or plain. */
export function RateCell({ r, compact = false }: { r: ResourceView; compact?: boolean }) {
  if (isCapped(r)) return <CappedRate net={r.per_day} lost={r.overflow} compact={compact} spare={r.id === 'energy'} />;
  if (Number(r.unmet) > 0) return <ShortRate rate={r.per_day} unmet={r.unmet} change={r.change_per_day} compact={compact} />;
  return <>{perDay(r.per_day)}</>;
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

/** What a short source had, never shown equal to what it needed ("needs 150, had 150"
 * for 149.6): floored to two decimals then, or "under" the need. */
export function hadText(s: ShortView): string {
  return numBelow(s.had, s.need);
}

/** `num(x)`, but an `x` under `limit` never reads equal to it ("2,000 / 2,000" for
 * 1,999.6): floored to two decimals then, or "under" the limit. */
export function numBelow(x: Fx | number, limit: Fx | number): string {
  const shown = num(limit);
  if (Number(x) >= Number(limit) || num(x) !== shown) return num(x);
  // Past 10,000 the suffix form stays: "under 20K", not "19,996 / 20K".
  if (Math.abs(Number(limit)) >= 10_000) return `under ${shown}`;
  const h = (Math.floor(Number(x) * 100) / 100).toLocaleString('en-US', { maximumFractionDigits: 2 });
  return h !== shown ? h : `under ${shown}`;
}

/** A shortfall's short form, "needs ⚡2, had 0" (the chip for the resource). */
export function ShortNote({ v, s }: { v: View; s: ShortView }) {
  return (
    <span class="short-note">
      needs <ResChip v={v} id={s.resource} />
      {num(s.need)}, had {hadText(s)}
    </span>
  );
}

/** The share a short building or line ran at, floored (99.6% must not read 100%), the
 * same everywhere it shows: card status, tooltip, breakdown. */
export function ranPercent(ran: Fx): string {
  const p = Math.floor(Number(ran) * 100 + 1e-9);
  return p < 1 ? '<1%' : `${p}%`;
}

/** A shortfall in full words, for tooltips. */
export function shortText(v: View, s: ShortView): string {
  const ran = Number(s.ran) > 0 ? `ran at ${ranPercent(s.ran)}` : 'did not run';
  return `${s.label} ${ran}: needed ${num(s.need)} ${name(v, s.resource)}, had ${hadText(s)}`;
}

/** A signed amount for a breakdown cell ("+5", "−2"; blank for none). */
function signed(x: number): string {
  const shown = num(Math.abs(x));
  return shown === '0' ? '' : `${x < 0 ? '−' : '+'}${shown}`;
}

/** A resource's breakdown: each source (+) and sink (−) per day, net per source, the last
 * day's and (with `avg`) the rolling average's; then who went short of it. */
export function Breakdown({ v, id, avg = false }: { v: View; id: string; avg?: boolean }) {
  const b = v.breakdown.find((x) => x.resource === id);
  const shows = (r: FlowRow) =>
    !!signed(Number(r.produced) - Number(r.consumed)) || (avg && !!signed(Number(r.avg_produced) - Number(r.avg_consumed)));
  if (!b || (!b.rows.some(shows) && b.shorts.length === 0)) return <p class="muted breakdown-empty">Nothing moved it the last day.</p>;
  return (
    <div class="breakdown" data-breakdown={id}>
      {/* Only a shortfall to show (nothing moved): no empty table under its header. */}
      {b.rows.some(shows) && (
      <table class="breakdown-table">
        <thead>
          <tr>
            <th />
            <th class="num">Last day</th>
            {avg && <th class="num">Avg {v.flow_average_days}d</th>}
          </tr>
        </thead>
        <tbody>
          {b.rows.map((r) => {
            const day = Number(r.produced) - Number(r.consumed);
            const mean = Number(r.avg_produced) - Number(r.avg_consumed);
            // A row whose every shown cell rounds to nothing says nothing: left out.
            if (!shows(r)) return null;
            // Each cell its own color: a source's average can run the other way. Boredom
            // rising is bad news, as its Resources rate reads.
            const tone = (x: number) => ((risingIsBad(id) ? -x : x) > 0 ? 'pos' : 'neg');
            return (
              <tr key={r.source} data-source={r.source} class={r.source === 'cap' ? 'lost' : ''}>
                <td>{r.label}</td>
                <td class={`num ${tone(day)}`}>{signed(day)}</td>
                {avg && <td class={`num ${tone(mean)}`}>{signed(mean)}</td>}
              </tr>
            );
          })}
        </tbody>
      </table>
      )}
      {b.shorts.length > 0 && (
        <ul class="plain breakdown-shorts">
          {b.shorts.map((s) => (
            <li key={s.source} title={shortText(v, s)} data-short={s.source}>
              <span class="neg">Short:</span> {s.label}
              {Number(s.ran) > 0 ? ` at ${ranPercent(s.ran)}` : ''} · <ShortNote v={v} s={s} />
            </li>
          ))}
        </ul>
      )}
    </div>
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
