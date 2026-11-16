// Left column: title, nav grid, speed controls, Resources and Ideology, the horizon.

import { Fragment } from 'preact';
import { SPEEDS } from '../clock';
import { capitalize, num, perDay } from '../format';
import { navHasNews, seen, setSpeed, type Store } from '../store';
import { Horizon } from './art';
import { Section, Square } from './common';

export function Left({ s, panel, open }: { s: Store; panel: string; open: (id: string) => void }) {
  const v = s.view;
  const tiles = v.nav.filter((t) => t.visible);
  return (
    <aside class="col-left">
      <h1 class="title">Helium Hustle</h1>
      <nav class="nav-grid" aria-label="Panels">
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
            <Square id={t.id} size={34} />
            <span>{t.name}</span>
            {navHasNews(t.id) && <span class="dot" title="Something new" />}
          </button>
        ))}
      </nav>
      <div class="speed">
        <div class="label">Game Speed</div>
        <div class="speeds" id="speeds" role="group" aria-label="Speed">
          {SPEEDS.map((sp) => (
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
          <tbody>
            {v.resources
              .filter((r) => r.visible)
              .map((r) => {
                const amt = Number(r.amount);
                const cap = r.cap === null ? null : Number(r.cap);
                const full = cap !== null && amt >= cap;
                const empty = amt <= 0;
                let shown = r.cap === null ? num(r.amount) : `${num(r.amount)} / ${num(r.cap)}`;
                let rate = perDay(r.per_day);
                if (r.id === 'processors') {
                  shown = `${v.processors - v.processors_free} / ${v.processors}`;
                  rate = `${v.processors_free} free`;
                }
                if (r.id === 'land') rate = 'free';
                // Boredom rising is bad news: its rate reads the other way round.
                const rateNum = Number(r.per_day) * (r.id === 'boredom' ? -1 : 1);
                return (
                  <tr key={r.id} data-res={r.id} data-amount={r.amount} class={s.fresh.has(`resource:${r.id}`) ? 'is-new' : ''} onMouseEnter={() => seen(`resource:${r.id}`)}>
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
                                <Square id={r.id} />{' '}
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
                );
              })}
          </tbody>
        </table>
      </Section>
      {v.nav.some((t) => t.id === 'ideologies' && t.visible) && (
        <Section title="Ideology" testid="ideology-summary">
          <table class="res-table">
            <tbody>
              {v.ideology.map((a) => (
                <tr key={a.axis}>
                  <td class={`axis-${a.axis}`}>{capitalize(a.axis)}</td>
                  <td>Rank {a.rank}</td>
                  <td class="num muted">
                    {Number(a.score) < 0 ? '−' : ''}
                    {num(Math.abs(Number(a.score)))} / {num(a.next_rank_score)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </Section>
      )}
      <Horizon />
    </aside>
  );
}
