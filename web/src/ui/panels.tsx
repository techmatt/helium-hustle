// The middle column's other panels: Commands, Launch Pads, Research, Projects,
// Ideologies, Adversaries, Stats, Story and Retirement. Each shows what the view has.

import { useState } from 'preact/hooks';
import type { Amounts, Fx, ProjectView, ResearchView, View } from '../engine';
import { capitalize, mult, num, perDay } from '../format';
import { act, seen, store, type Store } from '../store';
import { Bar, FlowLines, Section, Square, cardButton, freshClass, name, useHint } from './common';
import { Quest, editProgram } from './Right';

// ---- Commands ------------------------------------------------------------------

export function Commands({ s, program }: { s: Store; program: number }) {
  const v = s.view;
  const add = (id: string) => editProgram(program, (now) => [...now, { command: id, count: 1 }]);
  return (
    <div class="panel" data-panel="commands">
      <h2>Commands</h2>
      <p class="muted">
        Commands run from programs (right): each assigned processor runs one step a day. Adding goes to
        Program {program + 1}.
      </p>
      <div class="cards">
        {v.commands
          .filter((c) => c.visible)
          .map((c) => {
            const key = `command:${c.id}`;
            return (
              <div key={c.id} class={`card command${freshClass(s.fresh, key)}`} data-id={c.id} onMouseEnter={() => seen(key)}>
                <div class="card-head">
                  <h3>{c.name}</h3>
                </div>
                <p class="desc">{c.description}</p>
                <div class="effects">
                  <FlowLines v={v} amounts={c.costs} sign={-1} suffix="" />
                  <FlowLines v={v} amounts={c.produces} sign={1} suffix="" />
                  {Number(c.boredom) > 0 && (
                    <div class="flow neg">
                      <Square id="boredom" /> +{num(c.boredom, 2)} Boredom
                    </div>
                  )}
                </div>
                <div class="card-actions">
                  <button type="button" data-testid={`add-${c.id}`} disabled={!c.unlocked} onClick={() => add(c.id)}>
                    {c.unlocked ? `Add to Program ${program + 1}` : 'Locked'}
                  </button>
                </div>
              </div>
            );
          })}
      </div>
    </div>
  );
}

// ---- Launch Pads -----------------------------------------------------------------

export function LaunchPads({ s }: { s: Store }) {
  const v = s.view;
  const goods = v.loading_priority;
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
              <th>Speculators</th>
            </tr>
          </thead>
          <tbody>
            {v.markets.map((m) => (
              <tr key={m.good}>
                <td>
                  <Square id={m.good} /> {m.name}
                </td>
                <td>{m.demand !== null ? `${num(Number(m.demand) * 100, 0)}% (${m.tier})` : m.tier}</td>
                <td>{m.speculators !== null ? num(m.speculators, 0) : '—'}</td>
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
              <Square id={g} /> {name(v, g)}
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
          {v.pads.map((p, i) => (
            <div
              class={`card pad${p.active ? '' : ' idle'}${p.ready ? ' ready' : ''}`}
              key={i}
              data-pad={i}
              data-cargo={p.cargo}
              data-ready={p.ready}
            >
              <div class="card-head">
                <h3>Pad {i + 1}</h3>
                {p.ready ? (
                  <span class="pad-status pad-ready">Ready</span>
                ) : (
                  <span class="pad-status muted">{p.active ? (p.paused ? 'paused' : 'loading') : 'inactive'}</span>
                )}
              </div>
              <label>
                Cargo{' '}
                <select
                  value={p.good}
                  onChange={(e) => act({ type: 'set_pad_good', pad: i, good: (e.target as HTMLSelectElement).value })}
                >
                  {goods.map((g) => (
                    <option key={g} value={g}>
                      {name(v, g)}
                    </option>
                  ))}
                </select>
              </label>
              <Bar frac={Number(p.cargo) / Number(p.capacity)} label={`${num(p.cargo)} / ${num(p.capacity)}`} />
              {/* The shared Propellant pool against one launch's burn. */}
              <div class={`pad-fuel ${p.fueled ? 'pos' : 'neg'}`} data-testid={`fuel-${i}`} data-fueled={p.fueled}>
                {/* Short, the pool is rounded down: 19.96 must not read as 20 / 20. */}
                Fuel {num(p.fueled ? propellant : Math.floor(Number(propellant) * 10 + 1e-6) / 10)} / {num(p.fuel)} Propellant
              </div>
              <div class="muted small">
                Each Load Launch Pads run loads up to {num(p.load)} {name(v, p.good)} · a launch burns the fuel from the shared pool
                {p.cooldown > 0 ? ` · cooling down ${p.cooldown} days` : ''}
              </div>
              <div>Would pay {num(p.payout)} credits now</div>
              <div class="card-actions">
                <button
                  type="button"
                  class={p.ready ? 'go' : undefined}
                  data-testid={`launch-${i}`}
                  disabled={!p.can_launch}
                  onClick={() => act({ type: 'launch', pad: i })}>
                  Launch
                </button>
                <button type="button" class="tiny" onClick={() => act({ type: 'set_pad_paused', pad: i, paused: !p.paused })}>
                  {p.paused ? 'Resume loading' : 'Pause loading'}
                </button>
              </div>
            </div>
          ))}
        </div>
      </Section>
    </div>
  );
}

// ---- Research ----------------------------------------------------------------------

export function Research({ s }: { s: Store }) {
  const v = s.view;
  const shown = v.research.filter((r) => r.visible || r.done);
  const cats = [...new Set(v.research.map((r) => r.category))];
  const sci = v.resources.find((r) => r.id === 'science');
  return (
    <div class="panel" data-panel="research">
      <h2>Research</h2>
      <p class="muted">
        <Square id="science" /> {num(sci?.amount ?? '0')} Science ({perDay(sci?.per_day ?? '0')}). Research resets each
        run.
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

/** A research card is its Research button, like a building card (`cardButton`). */
function ResearchCard({ v, r, fresh }: { v: View; r: ResearchView; fresh: Set<string> }) {
  const key = `research:${r.id}`;
  const [hint, showHint] = useHint();
  const needs = r.requires && !v.research.find((x) => x.id === r.requires)?.done ? r.requires : null;
  // Reads the engine's view now, not this render's: a double-click researches once.
  const research = () => {
    const now = store.view.research.find((x) => x.id === r.id);
    if (!now || now.done) return;
    if (!now.affordable) {
      const req = now.requires && !store.view.research.find((x) => x.id === now.requires)?.done ? now.requires : null;
      showHint(req ? `Needs ${name(store.view, req)} first.` : 'Short of Science.');
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
        <h3>{r.name}</h3>
        {r.done && <span class="ok">✓ Done</span>}
      </div>
      <p class="desc">{r.description}</p>
      {needs && !r.done && <p class="muted small">Needs {name(v, needs)} first.</p>}
      {!r.done && (
        <div class="card-actions">
          <span>
            <Square id="science" /> {num(r.cost)} Science
          </span>
          <span class={hint ? 'buy-hint short' : 'buy-hint muted'}>{hint ?? ''}</span>
        </div>
      )}
    </div>
  );
}

// ---- Projects ----------------------------------------------------------------------

export function Projects({ s }: { s: Store }) {
  const v = s.view;
  const tiers: [string, string, string][] = [
    ['persistent', 'Long-Term', 'Progress and completion carry across retirements.'],
    ['personal', 'Strategic', 'This run only: progress resets on retirement.'],
  ];
  return (
    <div class="panel" data-panel="projects">
      <h2>Projects</h2>
      <p class="muted">Fund a project by setting a daily rate per resource; it drains that much each day until done.</p>
      {tiers.map(([tier, title, note]) => {
        const items = v.projects.filter((p) => p.tier === tier && (p.unlocked || p.completed));
        return (
          <Section key={tier} title={`${title} (${items.length})`} dark>
            <p class="muted small">{note}</p>
            {items.length === 0 && <p class="muted">None unlocked yet.</p>}
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

function Project({ v, p, fresh }: { v: View; p: ProjectView; fresh: Set<string> }) {
  const key = `project:${p.id}`;
  const step = (res: string, by: number) => {
    // From the engine's rate now, not this render's: quick clicks must each count.
    const cur = store.view.projects.find((x) => x.id === p.id) ?? p;
    const now = Number(cur.rates[res] ?? '0');
    const next = Math.max(0, Math.min(Number(p.max_rate), now + by));
    act({ type: 'set_project_rate', project: p.id, resource: res, rate: String(next) });
  };
  return (
    <div class={`card project${p.completed ? ' done' : ''}${freshClass(fresh, key)}`} data-id={p.id} onMouseEnter={() => seen(key)}>
      <div class="card-head">
        <h3>{p.name}</h3>
        {p.completed && <span class="ok">✓ Complete</span>}
      </div>
      <p class="desc">{p.description}</p>
      {!p.completed &&
        Object.entries(p.costs).map(([res, cost]) => {
          const got = p.progress[res] ?? '0';
          const rate = p.rates[res] ?? '0';
          return (
            <div class="proj-row" key={res}>
              <span class="proj-res">
                <Square id={res} /> {name(v, res)}
              </span>
              <Bar frac={Number(got) / Number(cost)} label={`${num(got)} / ${num(cost)}`} />
              <span class="proj-rate">{num(rate)}/day</span>
              <button type="button" class="tiny" disabled={Number(rate) <= 0} onClick={() => step(res, -1)}>
                −
              </button>
              <button type="button" class="tiny" disabled={Number(rate) >= Number(p.max_rate)} onClick={() => step(res, 1)}>
                +
              </button>
              <button type="button" class="tiny" title={`Fund at the most (${num(p.max_rate)}/day)`} onClick={() => step(res, Number(p.max_rate))}>
                Max
              </button>
            </div>
          );
        })}
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
        Fund commands push one axis up and the other two down. Each whole rank multiplies that axis's bonuses and
        discounts, among them the price of buildings tagged with it. Scores reset each run to a head start from the
        career's best.
      </p>
      {v.ideology.map((a) => {
        const lo = Number(a.rank_score);
        const hi = Number(a.next_rank_score);
        const sc = Math.abs(Number(a.score));
        // Below rank 1 either way the rank is 0: the score's sign says which way it heads.
        const down = Number(a.score) < 0;
        return (
          <div class="axis" key={a.axis} data-axis={a.axis}>
            <h3 class={`axis-${a.axis}`}>{capitalize(a.axis)}</h3>
            <div>
              Rank <strong>{a.rank}</strong> · score {num(a.score)} · next rank at {down ? '−' : ''}
              {num(a.next_rank_score)}
            </div>
            <Bar frac={(sc - lo) / (hi - lo)} tone={down ? 'red' : 'green'} />
            <div class="muted small">Career best rank: {v.career.max_ideology_ranks[a.axis] ?? 0}</div>
          </div>
        );
      })}
    </div>
  );
}

// ---- Adversaries -------------------------------------------------------------------

export function Adversaries({ s }: { s: Store }) {
  const v = s.view;
  return (
    <div class="panel" data-panel="adversaries">
      <h2>Adversaries</h2>
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
                <td>{r.name}</td>
                <td>
                  <Square id={r.good} /> {name(v, r.good)}
                </td>
                <td>{r.dumps_seen}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Section>
      <Section title="Speculators">
        <p class="muted">Speculators gather on goods that earn; they suppress demand until they drift away or are disrupted.</p>
        <table class="grid-table">
          <tbody>
            {v.markets.map((m) => (
              <tr key={m.good}>
                <td>
                  <Square id={m.good} /> {m.name}
                </td>
                <td>{m.speculators !== null ? `${num(m.speculators, 0)} speculators` : 'none seen'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Section>
    </div>
  );
}

// ---- Stats -------------------------------------------------------------------------

function Rows({ rows }: { rows: [string, string | number][] }) {
  return (
    <table class="grid-table kv">
      <tbody>
        {rows.map(([k, val]) => (
          <tr key={k}>
            <td>{k}</td>
            <td class="num">{val}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function amountRows(v: View, a: Amounts): [string, string][] {
  return Object.entries(a).map(([k, x]) => [name(v, k), num(x)]);
}

export function Stats({ s }: { s: Store }) {
  const v = s.view;
  const c = v.career;
  const mods = Object.entries(v.modifiers).filter(([, x]) => Number(x) !== 1);
  return (
    <div class="panel" data-panel="stats">
      <h2>Stats</h2>
      <Section title={`This run (run ${v.run}, day ${num(v.day)})`}>
        <Rows
          rows={[
            ['Credits earned', num(v.run_stats.credits_earned)],
            ['Shipments', v.run_stats.shipments],
            ['Buildings built', v.run_stats.buildings_built],
            ['Research completed', v.run_stats.research_completed],
            ['Peak energy production', perDay(v.run_stats.peak_energy)],
            ['Base boredom growth (before commands and Dream)', `${perDay(v.boredom.rate, 2)} (phase ${v.boredom.phase} of ${v.boredom.phases})`],
          ]}
        />
      </Section>
      <Section title="Produced this run">
        <Rows rows={amountRows(v, v.run_stats.produced)} />
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
            ['Longest run', `${num(c.best_run_days)} days`],
            ['Best run credits', num(c.best_run_credits)],
            ['Best run shipments', c.best_run_shipments],
            ['Peak energy production', perDay(c.peak_power)],
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
          <Rows rows={v.overclocks.map((o, i) => [`Overclock ${i + 1}: ${o.target}`, `+${num(Number(o.bonus) * 100)}% until day ${o.until_day}`])} />
        )}
      </Section>
    </div>
  );
}

// ---- Story -------------------------------------------------------------------------

export function Story({ s }: { s: Store }) {
  const v = s.view;
  const [openId, setOpen] = useState<string | null>(null);
  const cats = [...new Set(v.achievement_list.map((a) => a.category))];
  const earned = v.achievement_list.filter((a) => a.earned).length;
  return (
    <div class="panel" data-panel="story">
      <h2>Story</h2>
      <Section title="Primary objectives">
        <ul class="plain story-list">
          {v.story.map((e) => (
            <li key={e.id}>
              <button type="button" class="link" onClick={() => setOpen(openId === e.id ? null : e.id)}>
                ✓ {e.title}
              </button>{' '}
              <span class="muted">{e.summary}</span>
              {openId === e.id && <blockquote>{e.body}</blockquote>}
            </li>
          ))}
        </ul>
        {v.quest && (
          <div class="active-quest">
            <h3>Active</h3>
            <Quest v={v} />
            <blockquote>{v.quest.body}</blockquote>
          </div>
        )}
      </Section>
      <Section title={`Achievements (${earned}/${v.achievement_list.length})`}>
        {cats.map((cat) => {
          const list = v.achievement_list.filter((a) => a.category === cat);
          return (
            <div key={cat}>
              <h3>
                {capitalize(cat)} ({list.filter((a) => a.earned).length}/{list.length})
              </h3>
              <ul class="plain">
                {list.map((a) => (
                  <li key={a.id} class={a.earned ? 'ok' : ''}>
                    {a.earned ? '✓' : '○'} <strong>{a.name}</strong>: {a.description}{' '}
                    <span class="muted">Reward: {a.reward}</span>
                  </li>
                ))}
              </ul>
            </div>
          );
        })}
      </Section>
      <Section title={`Events this run (${v.event_texts.length})`}>
        {v.event_texts.length === 0 && <p class="muted">None yet.</p>}
        {v.event_texts.map((e) => (
          <div key={e.id} class="event-text">
            <strong>{e.title}</strong> <span class="muted">{e.summary}</span>
            <blockquote>{e.body}</blockquote>
          </div>
        ))}
      </Section>
    </div>
  );
}

// ---- Retirement --------------------------------------------------------------------

function delta(now: Fx, next: Fx, fmt: (x: Fx) => string): string {
  return Number(now) === Number(next) ? fmt(next) : `${fmt(now)} → ${fmt(next)}`;
}

export function NextRunRows({ v }: { v: View }) {
  const c = v.career;
  const n = v.next_run;
  const rows: [string, string][] = [
    ['Starting credits', delta(c.starting_credits, n.starting_credits, (x) => num(x))],
    ['Boredom growth (resilience)', delta(c.boredom_resilience, n.boredom_resilience, mult)],
  ];
  if (Number(n.buy_power_mult) !== 1) rows.push(['Buy Power scaling', delta(c.buy_power_mult, n.buy_power_mult, mult)]);
  for (const a of n.ideology) {
    if (Number(a.score) > 0) rows.push([`${capitalize(a.axis)} head start`, `score ${num(a.score)} (rank ${a.rank})`]);
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
        Retiring ends this run: buildings, resources, research and Strategic projects reset. The career keeps its
        bests, achievements, Long-Term projects, quests and what you have seen. Boredom at its cap retires you anyway.
      </p>
      <Section title={`This run (day ${num(v.day)})`}>
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
