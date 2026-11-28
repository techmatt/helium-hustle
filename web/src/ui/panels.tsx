// The middle column's other panels: Commands, Launch Pads, Research, Ideologies,
// Adversaries, Stats and Retirement (Projects: Projects.tsx; Story and Achievements:
// Story.tsx). Each shows what the view has.

import type { ComponentChildren } from 'preact';
import { useState } from 'preact/hooks';
import type { CommandView, Fx, ResearchView, View } from '../engine';
import { boredomText, capitalize, commandTip, dateText, held, mult, num, perDay, signed, slotLetter } from '../format';
import { act, justMoved, seen, store, type Store } from '../store';
import { AxisMark, Bar, Breakdown, FlowLines, ResChip, Section, cardButton, freshClass, modText, name, navShown, numBelow, useHideCompleted, useHint } from './common';
import { emblemArt } from './cardArt';
import { Chip, ResearchChip, ScopeIcon } from './icons';
import { editProgram } from './programStore';

// ---- Commands ------------------------------------------------------------------

export function Commands({ s, program }: { s: Store; program: number }) {
  const v = s.view;
  const add = (id: string) => editProgram(program, (now) => [...now, { op: 'command', command: id, count: 1 }]);
  // Per-target commands fold into one card per verb, its target picked from a dropdown.
  const [picked, setPicked] = useState<Record<string, string>>({});
  const shown = v.commands.filter((c) => c.visible);
  const cards = shown.filter((c, i) => !c.verb || shown.findIndex((x) => x.verb === c.verb) === i);
  return (
    <div class="panel" data-panel="commands">
      <h2>Commands</h2>
      <p class="muted">
        Commands run only from programs (right): each assigned processor runs one step a day. Adding goes to
        Program {slotLetter(program)}.
      </p>
      <div class="cards">
        {cards
          .map((first) => {
            const group = first.verb ? shown.filter((x) => x.verb === first.verb) : [first];
            const c = group.find((x) => x.id === picked[first.verb ?? '']) ?? first;
            // The card is new while any of its commands is; hovering it sees them all.
            const keys = group.map((x) => `command:${x.id}`);
            const fresh = keys.map((k) => freshClass(s.fresh, k)).find((f) => f !== '') ?? '';
            return (
              <div key={first.verb ?? c.id} class={`card command${fresh}`} data-id={c.id} onMouseEnter={() => keys.forEach((k) => seen(k))}>
                <div class="card-head">
                  {first.verb && group.length > 1 ? (
                    <h3>
                      {first.verb}{' '}
                      <select
                        aria-label={`${first.verb} target`}
                        data-testid={`target-${first.verb}`}
                        value={c.id}
                        onChange={(ev) => {
                          const id = ev.currentTarget.value;
                          setPicked((p) => ({ ...p, [first.verb ?? '']: id }));
                        }}
                      >
                        {group.map((x) => (
                          <option key={x.id} value={x.id}>
                            {x.target ?? x.name}
                          </option>
                        ))}
                      </select>
                    </h3>
                  ) : (
                    <h3>{c.name}</h3>
                  )}
                </div>
                <CommandBody v={v} c={c} />
                <div class="card-actions">
                  <button
                    type="button"
                    data-testid={`add-${c.id}`}
                    disabled={!c.unlocked}
                    title={commandTip(c, v) || undefined}
                    // A double-click's second click on a modal's last button that lands
                    // here is not an add (as a card's buy, #31).
                    onClick={(e) => (e.detail > 0 && justMoved() ? undefined : add(c.id))}
                  >
                    {c.unlocked ? 'Add to Program' : 'Locked'}
                  </button>
                </div>
              </div>
            );
          })}
      </div>
    </div>
  );
}

/** A command card's description and flows (the Commands panel's, the text editor's hover). */
export function CommandBody({ v, c }: { v: View; c: CommandView }) {
  return (
    <>
      <p class="desc">{c.description}</p>
      <div class="effects">
        <FlowLines v={v} amounts={c.costs} sign={-1} suffix="" />
        <FlowLines v={v} amounts={c.produces} sign={1} suffix="" />
        {/* Signed: a reducer (Dream, Theorize) reads "−2 Boredom", good news, with the icon of
            the axis that scales it. */}
        {boredomText(c.boredom) && (
          <div class={`flow ${Number(c.boredom) > 0 ? 'neg' : 'pos'}`} data-testid="command-boredom">
            <ResChip v={v} id="boredom" /> {boredomText(c.boredom)} Boredom
            {c.ideology && (
              <>
                {' '}
                <AxisMark v={v} axis={c.ideology} />
              </>
            )}
          </div>
        )}
      </div>
    </>
  );
}

// ---- Launch Pads -----------------------------------------------------------------

export function LaunchPads({ s }: { s: Store }) {
  const v = s.view;
  const goods = v.loading_priority;
  // Nothing upcoming is telegraphed: the column waits for speculators' first arrival this run.
  const specs = speculatorsSeen(v);
  const propellant = v.resources.find((r) => r.id === 'propellant')?.amount ?? '0';
  const swap = (i: number, j: number) => {
    const next = [...store.view.loading_priority];
    [next[i], next[j]] = [next[j], next[i]];
    act({ type: 'set_loading_priority', goods: next });
  };
  return (
    <div class="panel" data-panel="launch_pads">
      <h2>Launch Pads</h2>
      <Section title="Markets">
        <table class="grid-table">
          <thead>
            <tr>
              <th>Good</th>
              <th>Demand</th>
              {specs && <th>Speculators</th>}
            </tr>
          </thead>
          <tbody>
            {v.markets.map((m) => (
              <tr key={m.good}>
                <td>
                  <ResChip v={v} id={m.good} /> {m.name}
                </td>
                <td>{m.demand !== null ? `${num(Number(m.demand) * 100, 0)}% (${m.tier})` : m.tier}</td>
                {/* Under 10, a decimal: a pool of 0.4 still draws Disrupt Speculators first. */}
                {specs && <td>{m.speculators !== null ? num(m.speculators, Number(m.speculators) < 10 ? 1 : 0) : '—'}</td>}
              </tr>
            ))}
          </tbody>
        </table>
        {v.markets.some((m) => m.demand === null) && <p class="muted">Exact demand shows once Market Awareness is researched.</p>}
      </Section>
      <Section title="Loading priority">
        <ol class="priority">
          {goods.map((g, i) => (
            <li key={g}>
              <ResChip v={v} id={g} /> {name(v, g)}
              <button type="button" class="tiny" disabled={i === 0} onClick={() => swap(i, i - 1)}>
                ▲
              </button>
              <button type="button" class="tiny" disabled={i === goods.length - 1} onClick={() => swap(i, i + 1)}>
                ▼
              </button>
            </li>
          ))}
        </ol>
      </Section>
      <Section title={`Pads (${v.pads.length})`}>
        {v.pads.length === 0 && <p class="muted">No Launch Pads owned this run.</p>}
        <div class="cards">
          {v.pads.map((p, i) => {
            // The head's tint: green Ready, yellow paused (a paused full pad reads Paused),
            // red full but short of fuel, yellow No cargo (Load skips it), neutral loading or idle.
            const full = Number(p.cargo) >= Number(p.capacity);
            const ready = p.ready && !p.paused;
            // Full and fueled but still cooling down from its last launch: not loading.
            const state = !p.active
              ? 'idle'
              : p.paused
                ? 'paused'
                : p.good === null
                  ? 'nocargo'
                  : ready
                  ? 'ready'
                  : full && !p.fueled
                    ? 'blocked'
                    : full && p.cooldown > 0
                      ? 'cooling'
                      : 'loading';
            const label = { idle: p.unpaid ? 'idle: upkeep unpaid' : 'inactive', paused: 'Paused', nocargo: 'No cargo: pick one', ready: 'Ready', blocked: Number(propellant) > 0 ? 'Full: insufficient fuel' : 'Full: no fuel', cooling: `Full: ready in ${p.cooldown} day${p.cooldown === 1 ? '' : 's'}`, loading: 'loading' }[state];
            return (
            <div
              class={`card pad pad-${state}${p.active ? '' : ' idle'}${ready ? ' ready' : ''}`}
              key={i}
              data-pad={i}
              data-cargo={p.cargo}
              data-ready={ready}
              data-pad-state={state}
            >
              <div class="card-head">
                <h3>Pad {i + 1}</h3>
                <span class={`pad-status${ready ? ' pad-ready' : p.unpaid || state === 'nocargo' ? ' neg' : state === 'idle' || state === 'loading' || state === 'cooling' ? ' muted' : ''}`}>{label}</span>
              </div>
              <label
                title={
                  Number(p.cargo) > 0 && p.good !== null
                    ? `Switching cargo returns ${num(p.switch_returns)} of the ${num(p.cargo)} ${name(v, p.good)} loaded to the stockpile${
                        Number(p.switch_returns) < Number(p.cargo) ? ' (the rest is lost: stockpile full)' : ''
                      }`
                    : undefined
                }
              >
                Cargo {p.good !== null && <ResChip v={v} id={p.good} />}{' '}
                <select
                  value={p.good ?? ''}
                  onChange={(e) => {
                    const good = (e.target as HTMLSelectElement).value;
                    act({ type: 'set_pad_good', pad: i, good: good === '' ? null : good });
                  }}
                >
                  <option value="">No cargo</option>
                  {/* The goods the engine allows now; a pad's current good outside them stays shown. */}
                  {(p.good !== null && !v.pad_goods.includes(p.good) ? [...v.pad_goods, p.good] : v.pad_goods).map((g) => (
                    <option key={g} value={g}>
                      {name(v, g)}
                    </option>
                  ))}
                </select>
              </label>
              {/* Short of full, the cargo is rounded down: 99.96 must not read 100 / 100. */}
              <Bar
                frac={Number(p.cargo) / Number(p.capacity)}
                label={`${numBelow(p.cargo, p.capacity)} / ${num(p.capacity)}`}
              />
              {/* The shared Propellant pool against one launch's burn. */}
              <div class={`pad-fuel ${p.fueled ? 'pos' : 'neg'}`} data-testid={`fuel-${i}`} data-fueled={p.fueled}>
                {/* Short, the pool is rounded down: 19.96 must not read as 20 / 20. */}
                <ResChip v={v} id="propellant" /> Fuel {num(p.fueled ? propellant : Math.floor(Number(propellant) * 10 + 1e-6) / 10)} / {num(p.fuel)} Propellant
              </div>
              <div class="muted small">
                {p.good === null
                  ? 'No cargo: pick a good for Load Launch Pads to load'
                  : `Each Load Launch Pads run loads up to ${num(p.load)} ${name(v, p.good)}`}{' '}
                · a launch burns the fuel from the shared pool
                {p.cooldown > 0 ? ` · cooling down ${p.cooldown} day${p.cooldown === 1 ? '' : 's'}` : ''}
              </div>
              <div>
                Would pay <ResChip v={v} id="credits" /> {num(p.payout)} credits now
              </div>
              <div class="card-actions">
                <button
                  type="button"
                  class={ready ? 'go' : undefined}
                  data-testid={`launch-${i}`}
                  disabled={!p.can_launch}
                  onClick={() => act({ type: 'launch', pad: i })}>
                  Launch
                </button>
                <button
                  type="button"
                  class="tiny"
                  title={p.paused ? undefined : 'A paused pad neither loads nor launches through Launch Full Pads; Launch still works'}
                  onClick={() => act({ type: 'set_pad_paused', pad: i, paused: !p.paused })}
                >
                  {p.paused ? 'Resume loading' : 'Pause loading'}
                </button>
              </div>
            </div>
            );
          })}
        </div>
      </Section>
    </div>
  );
}

// ---- Research ----------------------------------------------------------------------

export function Research({ s }: { s: Store }) {
  const v = s.view;
  const hide = useHideCompleted('hh.hideCompleted.research', false);
  const shown = v.research.filter((r) => (r.visible || r.done) && hide.shows(r.id, r.done));
  const cats = [...new Set(v.research.map((r) => r.category))];
  const sci = v.resources.find((r) => r.id === 'science');
  return (
    <div class="panel" data-panel="research">
      <div class="panel-head">
        <h2>Research</h2>
        {hide.box}
      </div>
      <p class="muted">
        <ResChip v={v} id="science" /> {held(sci?.amount ?? '0')} Science ({perDay(sci?.per_day ?? '0')}). Research resets each
        run, but for items marked as kept.
      </p>
      {shown.length === 0 && <p class="muted">Nothing to research yet.</p>}
      {cats.map((cat) => {
        const items = shown.filter((r) => r.category === cat);
        if (items.length === 0) return null;
        return (
          <Section key={cat} title={cat} dark>
            <div class="cards">
              {items.map((r) => (
                <ResearchCard key={r.id} v={v} r={r} fresh={s.fresh} />
              ))}
            </div>
          </Section>
        );
      })}
    </div>
  );
}

/** The research `r` requires, while it is not done (else null). */
const unmetRequire = (v: View, r: ResearchView): string | null =>
  r.requires && !v.research.find((x) => x.id === r.requires)?.done ? r.requires : null;

/** A research card is its Research button, like a building card (`cardButton`). */
export function ResearchCard({ v, r, fresh }: { v: View; r: ResearchView; fresh: Set<string> }) {
  const key = `research:${r.id}`;
  const [hint, showHint] = useHint();
  const needs = unmetRequire(v, r);
  // Reads the engine's view now, not this render's: a double-click researches once.
  const research = () => {
    const now = store.view.research.find((x) => x.id === r.id);
    if (!now || now.done) return;
    if (!now.affordable) {
      const req = unmetRequire(store.view, now);
      // How much more, never "0 more" (199.96 held shows as 200 beside a 200 price).
      const sci = Number(store.view.resources.find((x) => x.id === 'science')?.amount ?? 0);
      const more = Number(now.cost) - sci;
      // Rounded up, so the gap never reads smaller than it is (whole from 100 up, as num shows).
      const up = more >= 100 ? Math.ceil(more) : Math.ceil(more * 10) / 10;
      const short = more > 0 ? ` (${more < 0.1 ? 'under 0.1' : num(up)} more)` : '';
      showHint(req ? `Needs ${name(store.view, req)} first.` : `Short of Science${short}.`);
      return;
    }
    act({ type: 'research', research: r.id });
  };
  return (
    <div
      class={`card research ${r.done ? 'done' : r.affordable ? 'can-buy' : 'cannot-buy'}${freshClass(fresh, key)}`}
      data-id={r.id}
      data-testid={`research-${r.id}`}
      aria-disabled={!r.affordable}
      title={r.affordable ? 'Click to research' : undefined}
      onMouseEnter={() => seen(key)}
      {...(r.done ? {} : cardButton(research))}
    >
      <div class="card-head">
        <ResearchChip id={r.id} category={r.category} title={r.category} />
        <h3>{r.name}</h3>
        {r.done && <span class="ok">✓ Done</span>}
        <ScopeIcon keep={r.keep} />
      </div>
      <p class="desc">{r.description}</p>
      {needs && !r.done && <p class="muted small">Needs {name(v, needs)} first.</p>}
      {!r.done && (
        <div class="card-actions">
          <span>
            <ResChip v={v} id="science" /> {num(r.cost)} Science
          </span>
          <span class={hint ? 'buy-hint short' : 'buy-hint muted'}>{hint ?? ''}</span>
        </div>
      )}
    </div>
  );
}

// ---- Ideologies --------------------------------------------------------------------

export function Ideologies({ s }: { s: Store }) {
  const v = s.view;
  return (
    <div class="panel" data-panel="ideologies">
      <h2>Ideologies</h2>
      <p class="muted">
        Fund commands push one axis up and the other two down; using an axis's boredom tool nudges it up a little.
        Each whole rank raises the axis's modifier ×m (1.1 a rank): its tagged buildings cost ÷m and its boredom
        tool works ×m. Rank 5 is a major rank: Nationalist and Humanist get their own change there.
        Rank 5 on any axis completes Friends in High Places and opens that faction's global project.
        Scores carry into the next run, fading over the years away.
      </p>
      {v.ideology.map((a) => {
        const lo = Number(a.rank_score);
        const hi = Number(a.next_rank_score);
        const sc = Math.abs(Number(a.score));
        // Below rank 1 either way the rank is 0: the score's sign says which way it heads.
        const down = Number(a.score) < 0;
        // Each axis a bold card (117): its emblem fills the left edge (a 160 px slot: the
        // axis's painted emblem, 119, or its chip with none), its name centered in its color,
        // a large rank numeral, and a full-width rank bar.
        const emblem = emblemArt(a.axis);
        return (
          <div class={`axis axis-tone-${a.axis}`} key={a.axis} data-axis={a.axis}>
            <div class="axis-emblem" data-testid={`axis-emblem-${a.axis}`}>
              {emblem ? (
                <img class="axis-emblem-art" src={emblem} alt={capitalize(a.axis)} title={capitalize(a.axis)} />
              ) : (
                <Chip id={a.axis} title={capitalize(a.axis)} size={112} />
              )}
            </div>
            <div class="axis-body">
              <h3 class={`axis-name axis-${a.axis}`}>{capitalize(a.axis)}</h3>
              <div class="axis-rank">
                <span class="axis-rank-label">Rank</span>
                <span class={`axis-rank-num axis-${a.axis}`}>{signed(a.rank)}</span>
                <span class="axis-mod" data-testid={`axis-mod-${a.axis}`} title="This axis's modifier now: its tagged buildings cost ÷ this, its boredom tool works × this">
                  {modText(a.modifier)}
                </span>
              </div>
              <Bar
                frac={(sc - lo) / (hi - lo)}
                tone={down ? 'red' : 'green'}
                label={`score ${signed(a.score)} · next rank at ${down ? '−' : ''}${num(a.next_rank_score)}`}
              />
              <div class="axis-best muted">Career best rank: {v.career.max_ideology_ranks[a.axis] ?? 0}</div>
            </div>
          </div>
        );
      })}
    </div>
  );
}

// ---- Adversaries -------------------------------------------------------------------

/** Speculators have been in some market this run (the view sends `null` until then). */
function speculatorsSeen(v: View): boolean {
  return v.markets.some((m) => m.speculators !== null);
}

export function Adversaries({ s }: { s: Store }) {
  const v = s.view;
  // Only those that have arrived in the career: nothing upcoming is telegraphed (ui.md).
  const seen = v.adversaries.filter((a) => a.seen);
  const rivals = seen.some((a) => a.id === 'rivals');
  return (
    <div class="panel" data-panel="adversaries">
      <h2>Adversaries</h2>
      <Section title="Attention">
        <p class="muted">
          {rivals ? 'Selling draws speculators; claiming land draws the rival AIs. Each comes' : 'Selling draws speculators. They come'} in once
          {rivals ? ' its' : ' their'} Attention reaches its threshold, and Attention fades day by day.
        </p>
        <table class="grid-table">
          <tbody>
            {seen.map((a) => (
              <tr key={a.id} data-testid={`attention-${a.id}`}>
                <td>{a.id === 'rivals' ? 'Rival AIs' : 'Speculators'}</td>
                <td>
                  {num(a.attention, 0)} / {num(a.threshold, 0)}
                </td>
                <td>{a.present ? 'present' : 'away'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Section>
      {rivals && (
        <Section title="Rival AIs">
          <p class="muted">Rivals dump goods on Earth's markets from time to time, pushing that good's demand down.</p>
          <table class="grid-table">
            <thead>
              <tr>
                <th>Rival</th>
                <th>Market</th>
                <th title="Among the recent activity kept (the last lines of Recent Activity)">Recent dumps</th>
              </tr>
            </thead>
            <tbody>
              {v.rivals.map((r) => (
                <tr key={r.id}>
                  <td>
                    <Chip id={r.id} title={r.name} /> {r.name}
                  </td>
                  <td>
                    <ResChip v={v} id={r.good} /> {name(v, r.good)}
                  </td>
                  <td>{r.dumps_seen}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </Section>
      )}
      {speculatorsSeen(v) && (
        <Section title="Speculators">
          <p class="muted">Speculators gather on goods that earn; they suppress demand until they drift away or are disrupted.</p>
          <table class="grid-table">
            <tbody>
              {v.markets.map((m) => (
                <tr key={m.good}>
                  <td>
                    <ResChip v={v} id={m.good} /> {m.name}
                  </td>
                  <td>{m.speculators !== null ? `${num(m.speculators, Number(m.speculators) < 10 ? 1 : 0)} speculators` : 'none seen'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </Section>
      )}
    </div>
  );
}

// ---- Stats -------------------------------------------------------------------------

function Rows({ rows }: { rows: [ComponentChildren, string | number][] }) {
  return (
    <table class="grid-table kv">
      <tbody>
        {rows.map(([k, val], i) => (
          <tr key={i}>
            <td>{k}</td>
            <td class="num">{val}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

/** Stats' Boredom section: this run's boredom by source (the view's, signed, totalling the
 * current boredom), then the per-day sources, last day and rolling average. */
function BoredomStats({ v }: { v: View }) {
  const b = v.boredom;
  // Boredom rising is bad news: a source adding it reads red, a reducer green.
  const tone = (x: number) => (x > 0 ? 'neg' : x < 0 ? 'pos' : '');
  const signed = (x: number) => `${x < 0 ? '−' : x > 0 ? '+' : ''}${num(Math.abs(x), 2)}`;
  return (
    <Section title="Boredom" testid="stats-boredom">
      <p class="muted hint">
        What drives boredom and by how much: this run's total by source (reducers negative), then per day, the last day and a rolling average
        over about {v.flow_average_days} days.
        {Number(b.escalation) !== 1 && ` Past the last phase it escalates each year: growth is now ${mult(b.escalation)}.`}
      </p>
      <div class="breakdown-grid">
        <div class="breakdown-card" data-testid="boredom-run-sources" data-res="boredom">
          <h4>
            <ResChip v={v} id="boredom" /> This run
          </h4>
          {b.sources.length === 0 ? (
            <p class="muted breakdown-empty">No boredom yet this run.</p>
          ) : (
            <table class="breakdown-table">
              <tbody>
                {b.sources.map(([k, label, x]) => (
                  <tr key={k} data-source={k}>
                    <td>{label}</td>
                    <td class={`num ${tone(Number(x))}`}>{signed(Number(x))}</td>
                  </tr>
                ))}
              </tbody>
              <tfoot>
                <tr class="total" data-source="total">
                  <td>Total (current boredom)</td>
                  <td class="num">{num(b.amount, 2)}</td>
                </tr>
              </tfoot>
            </table>
          )}
        </div>
        <div class="breakdown-card" data-testid="boredom-per-day">
          <h4>
            <ResChip v={v} id="boredom" /> Per day
          </h4>
          <Breakdown v={v} id="boredom" avg />
        </div>
      </div>
    </Section>
  );
}

export function Stats({ s }: { s: Store }) {
  const v = s.view;
  const c = v.career;
  const mods = Object.entries(v.modifiers).filter(([, x]) => Number(x) !== 1);
  return (
    <div class="panel" data-panel="stats">
      <h2>Stats</h2>
      <Section title={`This run (run ${v.run}, ${dateText(v.day, v.days_per_year)})`}>
        <Rows
          rows={[
            ['Credits earned', num(v.run_stats.credits_earned)],
            ['Shipments', v.run_stats.shipments],
            ['Buildings built', v.run_stats.buildings_built],
            ['Research completed', v.run_stats.research_completed],
            ['Peak energy production', perDay(v.run_stats.peak_energy)],
            ['Base boredom growth (before buildings’ relief and commands)', `${perDay(v.boredom.rate, 2)} (phase ${v.boredom.phase} of ${v.boredom.phases})`],
          ]}
        />
      </Section>
      <BoredomStats v={v} />
      <Section title="Resource breakdown" testid="stats-breakdown">
        <p class="muted hint">
          Per day, by source (+) and sink (−): the last day, and a rolling average over about {v.flow_average_days} days. Each day
          Solar Panels produce, programs are paid next, and buildings with upkeep share what is left.
        </p>
        <div class="breakdown-grid">
          {/* Boredom's table is in the Boredom section above. */}
          {v.breakdown.filter((b) => b.resource !== 'boredom').map((b) => (
            <div class="breakdown-card" key={b.resource}>
              <h4>
                <ResChip v={v} id={b.resource} /> {name(v, b.resource)}
              </h4>
              <Breakdown v={v} id={b.resource} avg />
            </div>
          ))}
        </div>
      </Section>
      <Section title="Career">
        <Rows
          rows={[
            ['Retirements', c.retirements],
            ['Days played (finished runs)', num(c.lifetime_days)],
            ['Credits earned (finished runs)', num(c.lifetime_credits_earned)],
            ['Shipments (finished runs)', c.lifetime_shipments],
            ['Buildings built (finished runs)', c.lifetime_buildings_built],
            ['Research completed (finished runs)', c.lifetime_research_completed],
            ['Longest run (finished runs)', `${num(c.best_run_days)} days`],
            ['Best run credits (finished runs)', num(c.best_run_credits)],
            ['Best run shipments (finished runs)', c.best_run_shipments],
            ['Peak energy production (finished runs)', perDay(c.peak_power)],
          ]}
        />
      </Section>
      <Section title="Active modifiers">
        {mods.length === 0 ? (
          <p class="muted">None yet: achievements, projects and career bests add them.</p>
        ) : (
          <Rows rows={mods.map(([k, x]) => [k.replace(/_/g, ' '), mult(x)])} />
        )}
        {v.overclocks.length > 0 && (
          <Rows rows={v.overclocks.map((o, i) => [`Overclock ${i + 1}: ${o.target}`, `+${num(Number(o.bonus) * 100)}% through ${dateText(o.until_day - 1, v.days_per_year)}`])} />
        )}
        <h4>Project rewards</h4>
        {v.live_rewards.length === 0 ? (
          <p class="muted">None yet</p>
        ) : (
          <ul class="plain" data-testid="live-rewards">
            {v.live_rewards.map((t, i) => (
              <li key={i}>{t}</li>
            ))}
          </ul>
        )}
      </Section>
    </div>
  );
}

// ---- Retirement --------------------------------------------------------------------

function changeText(now: Fx, next: Fx, fmt: (x: Fx) => string): string {
  return Number(now) === Number(next) ? fmt(next) : `${fmt(now)} → ${fmt(next)}`;
}

export function NextRunRows({ v }: { v: View }) {
  const c = v.career;
  const n = v.next_run;
  const rows: [ComponentChildren, string][] = [
    ['Starting credits', changeText(c.starting_credits, n.starting_credits, (x) => num(x))],
    ['Boredom growth (resilience)', changeText(c.boredom_resilience, n.boredom_resilience, mult)],
  ];
  if (Number(n.buy_power_mult) !== 1) rows.push(['Buy Power scaling', changeText(c.buy_power_mult, n.buy_power_mult, mult)]);
  // A carried score either way once Ideologies is open: a negative one costs (m below 1;
  // 134: only scores above 0 were listed, and before the panel opened too).
  for (const a of navShown(v, 'ideologies') ? n.ideology : []) {
    if (signed(a.score) !== '0') rows.push([
      <>
        <Chip id={a.axis} title={capitalize(a.axis)} /> {capitalize(a.axis)} carried
      </>,
      `score ${signed(a.score)} (rank ${signed(a.rank)})`,
    ]);
  }
  return <Rows rows={rows} />;
}

export function Retirement({ s }: { s: Store }) {
  const v = s.view;
  // The run the confirm was opened in: a run that ended meanwhile (boredom) drops it, so
  // a stale "Yes, retire" can't end the next run on its first day.
  const [confirmRun, setConfirmRun] = useState<number | null>(null);
  const confirming = confirmRun === v.run;
  const setConfirming = (on: boolean) => setConfirmRun(on ? v.run : null);
  return (
    <div class="panel" data-panel="retirement">
      <h2>Retirement</h2>
      <p class="muted">
        Retiring ends this run: buildings, resources, research (but for items marked as kept), Strategic projects and every project's funding rates reset. The career keeps its
        bests, achievements, global projects, quests, saved programs, {navShown(v, 'programs') ? 'Programming, ' : ''}ideology (fading through the years away), adversaries and what you have
        seen. Boredom at its cap retires you anyway.
      </p>
      <Section title={`This run (${dateText(v.day, v.days_per_year)})`}>
        <Rows
          rows={[
            ['Credits earned', num(v.run_stats.credits_earned)],
            ['Shipments completed', v.run_stats.shipments],
            ['Buildings built', v.run_stats.buildings_built],
            ['Peak energy production', perDay(v.run_stats.peak_energy)],
            ['Research completed', v.run_stats.research_completed],
            ['Run length', `${num(v.day)} days`],
          ]}
        />
      </Section>
      <Section title="Next run bonuses (if you retire now)">
        <NextRunRows v={v} />
      </Section>
      {!confirming ? (
        <button type="button" class="big" data-testid="retire" disabled={!v.can_retire} onClick={() => setConfirming(true)}>
          Retire Now
        </button>
      ) : (
        <div class="confirm">
          <span>Retire and start run {v.run + 1}?</span>
          <button
            type="button"
            class="big danger"
            data-testid="retire-confirm"
            disabled={!v.can_retire}
            onClick={() => {
              setConfirming(false);
              act({ type: 'retire' });
            }}
          >
            Yes, retire
          </button>
          <button type="button" onClick={() => setConfirming(false)}>
            Cancel
          </button>
        </div>
      )}
    </div>
  );
}
