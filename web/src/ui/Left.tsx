// Left column: title, nav grid, speed controls, Resources and Ideology, the horizon.

import { Fragment, type ComponentChildren } from 'preact';
import { DEBUG_SPEEDS, SPEEDS } from '../clock';
import { capitalize, num, perDay } from '../format';
import { navHasNews, seen, setSpeed, type Store } from '../store';
import { useEffect, useState } from 'preact/hooks';
import { Breakdown, CappedRate, Section, ShortRate, navShown, risingIsBad } from './common';
import { Chip, Horizon, NavIcon } from './icons';

/** The Resources panel's first group, in this order; every other resource follows in a
 * second group, in the engine's order. Display order only. */
const FIRST_GROUP = ['energy', 'credits', 'land', 'boredom', 'processors'];

function resourceGroups<R extends { id: string }>(shown: R[]): R[][] {
  const first = FIRST_GROUP.flatMap((id) => shown.filter((r) => r.id === id));
  const rest = shown.filter((r) => !FIRST_GROUP.includes(r.id));
  return [first, rest].filter((g) => g.length > 0);
}

/** Below this window height Ideology starts folded, so a late career's column fits
 * without scrolling (at 768 px it was about one row over); a click opens it. */
const IDEOLOGY_OPEN_MIN_HEIGHT = 800;

export function Left({ s, panel, open }: { s: Store; panel: string; open: (id: string) => void }) {
  const v = s.view;
  const tiles = v.nav.filter((t) => t.visible);
  // Past nine tiles they turn compact (icon beside the name), so a late career's column
  // still fits a 1280 px window without scrolling.
  const compact = tiles.length > 9;
  const padReady = v.pads.some((p) => p.ready && !p.paused);
  // The one resource row expanded to its breakdown, if any.
  const [expanded, setExpanded] = useState<string | null>(null);
  // A resource hidden (a new run) closes its breakdown, so it never reopens by itself.
  const expandedShown = v.resources.some((r) => r.visible && r.id === expanded);
  useEffect(() => {
    if (expanded !== null && !expandedShown) setExpanded(null);
  }, [expanded, expandedShown]);
  return (
    <aside class="col-left">
      <h1 class="title">Helium Hustle</h1>
      <nav class={`nav-grid${compact ? ' compact' : ''}`} aria-label="Panels">
        {tiles.map((t) => (
          <button
            type="button"
            key={t.id}
            class={`nav-tile${panel === t.id ? ' active' : ''}`}
            data-nav={t.id}
            onClick={() => {
              seen(`nav:${t.id}`);
              open(t.id);
            }}
          >
            <NavIcon id={t.id} compact={compact} />
            <span>{t.name}</span>
            {navHasNews(t.id) && <span class="dot" title="Something new" />}
            {t.id === 'launch_pads' && padReady && <span class="dot dot-ready" data-testid="pads-ready" title="A pad is ready to launch" />}
          </button>
        ))}
      </nav>
      <div class="speed">
        <div class="label">Game Speed</div>
        <div class="speeds" id="speeds" role="group" aria-label="Speed">
          {[...SPEEDS, ...(s.cheatsEnabled ? DEBUG_SPEEDS : [])].map((sp) => (
            <button
              type="button"
              key={sp}
              data-speed={sp}
              aria-pressed={s.speed === sp}
              title={sp === 0 ? 'Pause (space)' : `${sp} days per second${sp > 200 ? ' (debug)' : ''}`}
              onClick={() => setSpeed(sp)}
            >
              {sp === 0 ? '❚❚' : sp >= 1000 ? `${sp / 1000}k` : `${sp}x`}
            </button>
          ))}
        </div>
      </div>
      <Section title="Resources" testid="resources">
        <table class="res-table">
          {resourceGroups(v.resources.filter((r) => r.visible)).map((group, gi) => (
          <tbody key={gi} data-res-group={gi}>
            {group.map((r) => {
                const amt = Number(r.amount);
                const cap = r.cap === null ? null : Number(r.cap);
                const full = cap !== null && amt >= cap;
                const empty = amt <= 0;
                let shown = r.cap === null ? num(r.amount) : `${num(r.amount)} / ${num(r.cap)}`;
                // The rate is the engine's (a program's `rate` reading, for tomorrow). At its
                // cap it reads the surplus the cap throws away, muted and marked; capped
                // Energy's surplus is spare power, good news: it shows green. Short, it reads
                // the demand left unmet, marked.
                const capped = r.id !== 'boredom' && Number(r.overflow) > 0;
                let rate: ComponentChildren = capped ? (
                  <CappedRate net={r.per_day} lost={r.overflow} compact spare={r.id === 'energy'} />
                ) : Number(r.unmet) > 0 ? (
                  <ShortRate rate={r.per_day} unmet={r.unmet} change={r.change_per_day} compact />
                ) : (
                  perDay(r.per_day)
                );
                if (r.id === 'processors') {
                  shown = `${v.processors - v.processors_free} / ${v.processors}`;
                  // Data Centers short of upkeep: fewer run than are assigned.
                  rate = v.processors_running < v.processors - v.processors_free ? `${v.processors_running} running` : `${v.processors_free} free`;
                }
                if (r.id === 'land') rate = 'free';
                // The next day retires the run: its rates (all 0) say nothing.
                else if (v.retires_next && r.id !== 'processors') rate = '—';
                // Boredom rising is bad news: its rate reads the other way round.
                // A rate too small to show reads "0/day", uncolored.
                const rateNum = capped || rate === '0/day' ? 0 : Number(r.per_day) * (risingIsBad(r.id) ? -1 : 1);
                const isOpen = expanded === r.id;
                // Processors are counted from buildings, not made or spent: no breakdown.
                const opens = r.id !== 'processors';
                return (
                  <Fragment key={r.id}>
                  <tr
                    data-res={r.id}
                    data-amount={r.amount}
                    class={`res-row${opens ? '' : ' no-breakdown'}${isOpen ? ' open' : ''}${s.fresh.has(`resource:${r.id}`) ? ' is-new' : ''}`}
                    title={!opens ? 'Processors come from buildings; each runs a program' : isOpen ? 'Click to close' : 'Click for where it comes from and goes'}
                    aria-expanded={opens ? isOpen : undefined}
                    onMouseEnter={() => seen(`resource:${r.id}`)}
                    onClick={() => opens && setExpanded(isOpen ? null : r.id)}
                  >
                    <td>
                      {/* A long name may wrap when the column is tight, but only between
                          words ("Helium-3" stays whole), and the swatch stays with the
                          first word. */}
                      {r.name.split(' ').map((w, i) => (
                        <Fragment key={i}>
                          {i > 0 && ' '}
                          <span class="word">
                            {i === 0 && (
                              <>
                                <Chip id={r.id} title={r.name} />{' '}
                              </>
                            )}
                            {w}
                          </span>
                        </Fragment>
                      ))}
                    </td>
                    <td class={`num ${r.id === 'boredom' ? '' : full ? 'at-cap' : empty ? 'empty' : ''}`}>{shown}</td>
                    <td class={`num rate ${r.id === 'processors' || r.id === 'land' ? '' : rateNum > 0 ? 'pos' : rateNum < 0 ? 'neg' : ''}`}>{rate}</td>
                  </tr>
                  {isOpen && (
                    <tr class="res-breakdown" data-res-breakdown={r.id}>
                      <td colSpan={3}>
                        <Breakdown v={v} id={r.id} />
                      </td>
                    </tr>
                  )}
                  </Fragment>
                );
              })}
          </tbody>
          ))}
        </table>
      </Section>
      {navShown(v, 'ideologies') && (
        <Section title="Ideology" testid="ideology-summary" open={window.innerHeight >= IDEOLOGY_OPEN_MIN_HEIGHT}>
          <table class="res-table">
            <tbody>
              {v.ideology.map((a) => (
                <tr key={a.axis}>
                  <td class={`axis-${a.axis}`}>
                    <Chip id={a.axis} title={capitalize(a.axis)} /> {capitalize(a.axis)}
                  </td>
                  <td>Rank {a.rank}</td>
                  <td class="num muted">
                    {Number(a.score) < 0 ? '−' : ''}
                    {num(Math.abs(Number(a.score)))} / {Number(a.score) < 0 ? '−' : ''}
                    {num(a.next_rank_score)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </Section>
      )}
      <Horizon v={v} />
    </aside>
  );
}
