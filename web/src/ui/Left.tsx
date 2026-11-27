// Left column: title, nav grid, speed controls, Resources and Ideology, and at the foot
// the active main quest's picture (the lunar-base horizon when it has none).

import { Fragment, type ComponentChildren } from 'preact';
import { speedsOffered } from '../clock';
import { capitalize, num, perDay } from '../format';
import { navHasNews, seen, setSpeed, type Store } from '../store';
import { useEffect, useRef, useState } from 'preact/hooks';
import { Breakdown, RateCell, Section, isCapped, modText, navShown, risingIsBad } from './common';
import { QuestPicture, questPicture } from './art';
import { Chip, Horizon, NavIcon } from './icons';
import { navSlots } from './navSlots';
import { FIRST_GROUP } from './resourceOrder';
import { CalendarStrip } from './Saga';
import { TakeoverButtons, type Takeover } from './Takeover';

/** The Resources panel's two groups: `FIRST_GROUP` (resourceOrder.ts, the order every card's
 * resource list follows), then every other resource in the engine's order. */
function resourceGroups<R extends { id: string }>(shown: R[]): R[][] {
  const first = FIRST_GROUP.flatMap((id) => shown.filter((r) => r.id === id));
  const rest = shown.filter((r) => !FIRST_GROUP.includes(r.id));
  return [first, rest].filter((g) => g.length > 0);
}

/** Below this window height Ideology starts folded, so a late career's column fits
 * without scrolling (at 768 px it was about one row over); a click opens it. */
const IDEOLOGY_OPEN_MIN_HEIGHT = 800;

export function Left({
  s,
  panel,
  open,
  takeovers,
  takeover,
  toggleTakeover,
}: {
  s: Store;
  /** The panel shown, or null while a takeover covers it. */
  panel: string | null;
  open: (id: string) => void;
  takeovers: Takeover[];
  takeover: string | null;
  toggleTakeover: (id: string) => void;
}) {
  const v = s.view;
  // One layout all game (109): two columns, the open tiles packed in content order, no blank
  // cells; a panel that opens takes its place and the utility tiles after it move down.
  const slots = navSlots(v.nav);
  // A tile that turns visible while the column is up (an unlock, a later run reopening a
  // per-run panel) fades in; the tiles shown when it first draws don't. `fading` keeps the
  // class until the tile hides again, so a later render never cuts the fade short.
  const lastShown = useRef<Set<string> | null>(null);
  const fading = useRef(new Set<string>());
  const shown = new Set(v.nav.filter((t) => t.visible).map((t) => t.id));
  if (lastShown.current !== null) for (const id of shown) if (!lastShown.current.has(id)) fading.current.add(id);
  for (const id of fading.current) if (!shown.has(id)) fading.current.delete(id);
  lastShown.current = shown;
  const padReady = v.pads.some((p) => p.ready && !p.paused);
  // A working pad with No cargo loads nothing until the player picks one.
  const padNoCargo = v.pads.some((p) => p.active && p.good === null);
  // The one resource row expanded to its breakdown, if any.
  const [expanded, setExpanded] = useState<string | null>(null);
  // A resource hidden (a new run) closes its breakdown, so it never reopens by itself.
  const expandedShown = v.resources.some((r) => r.visible && r.id === expanded);
  useEffect(() => {
    if (expanded !== null && !expandedShown) setExpanded(null);
  }, [expanded, expandedShown]);
  return (
    <aside class="col-left">
      {/* split-glow (077's pick): HELIUM in ink, HUSTLE in glowing amber, a glowing rule. */}
      <h1 class="title split-glow">
        <span class="lt-a">Helium</span> <span class="lt-b">Hustle</span>
      </h1>
      <CalendarStrip v={v} />
      <nav class="nav-grid" aria-label="Panels">
        {slots.map((t) => (
          <button
            type="button"
            key={t.id}
            class={`nav-tile${panel === t.id ? ' active' : ''}${fading.current.has(t.id) ? ' fresh' : ''}`}
            data-nav={t.id}
            onClick={() => {
              seen(`nav:${t.id}`);
              open(t.id);
            }}
          >
            <NavIcon id={t.id} />
            <span>{t.name}</span>
            {navHasNews(t.id) && <span class="dot dot-new" title="Something new" />}
            {t.id === 'launch_pads' && padReady && <span class="dot dot-ready" data-testid="pads-ready" title="A pad is ready to launch" />}
            {t.id === 'launch_pads' && padNoCargo && <span class="dot dot-cargo" data-testid="pads-no-cargo" title="A pad has No cargo: pick one" />}
          </button>
        ))}
      </nav>
      <TakeoverButtons s={s} list={takeovers} open={takeover} toggle={toggleTakeover} />
      <div class="speed">
        <div class="label">Game Speed</div>
        <div class="speeds" id="speeds" role="group" aria-label="Speed">
          {speedsOffered(v.fast_speeds).map((sp) => (
            <button
              type="button"
              key={sp}
              data-speed={sp}
              aria-pressed={s.speed === sp}
              title={sp === 0 ? 'Pause (space)' : `${sp} days per second`}
              onClick={() => setSpeed(sp)}
            >
              {sp === 0 ? '❚❚' : `${sp}x`}
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
                // The rate is the engine's (tomorrow's). At its
                // cap it reads the surplus the cap throws away, muted and marked; capped
                // Energy's surplus is spare power, good news: it shows green. Short, it reads
                // the demand left unmet, marked.
                const capped = isCapped(r);
                // The plain rate's text (null when capped or short), or the text that replaces it.
                let text: string | null = capped || Number(r.unmet) > 0 ? null : perDay(r.per_day);
                if (r.id === 'processors') {
                  shown = `${v.processors - v.processors_free} / ${v.processors}`;
                  // Data Centers short of upkeep: fewer run than are assigned.
                  text = v.processors_running < v.processors - v.processors_free ? `${v.processors_running} running` : `${v.processors_free} free`;
                }
                if (r.id === 'land') text = 'free';
                // The next day retires the run: its rates (all 0) say nothing.
                else if (v.retires_next && r.id !== 'processors') text = '—';
                const rate: ComponentChildren = text ?? <RateCell r={r} compact />;
                // Boredom rising is bad news: its rate reads the other way round.
                // A rate too small to show reads "0/day", uncolored.
                const rateNum = capped || text === '0/day' ? 0 : Number(r.per_day) * (risingIsBad(r.id) ? -1 : 1);
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
                  <td>
                    Rank {a.rank} <span class="muted" data-testid={`left-axis-mod-${a.axis}`}>{modText(a.modifier)}</span>
                  </td>
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
      {/* The active main quest's picture; the lunar-base horizon when it has none. */}
      {questPicture(v.quest?.picture) ? (
        <div class="horizon quest-scene" data-quest={v.quest?.id} aria-hidden="true">
          <QuestPicture stem={v.quest?.picture} at="left" />
        </div>
      ) : (
        <Horizon v={v} />
      )}
    </aside>
  );
}
