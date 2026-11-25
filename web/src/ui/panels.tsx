// The middle column's other panels: Commands, Launch Pads, Research, Projects,
// Ideologies, Adversaries, Stats and Retirement (Story and Achievements: Story.tsx). Each
// shows what the view has.

import type { ComponentChildren } from 'preact';
import { useState } from 'preact/hooks';
import type { CommandView, Fx, ProjectView, ResearchView, View } from '../engine';
import { capitalize, mult, num, perDay } from '../format';
import { act, seen, store, type Store } from '../store';
import { Bar, Breakdown, FlowLines, ResChip, Section, cardButton, freshClass, name, navShown, numBelow, useHint } from './common';
import { Chip } from './icons';
import { editProgram } from './programStore';

// ---- Commands ------------------------------------------------------------------

export function Commands({ s, program }: { s: Store; program: number }) {
  const v = s.view;
  const add = (id: string) => editProgram(program, (now) => [...now, { op: 'command', command: id, count: 1 }]);
  // Per-target commands fold into one card per verb, its target picked from a dropdown.
  const [picked, setPicked] = useState<Record<string, string>>({});
  const shown = v.commands.filter((c) => c.visible);
  const cards = shown.filter((c, i) => !c.verb || shown.findIndex((x) => x.verb === c.verb) === i);
  // Add to Program shows once Programming is bought (the Programs nav tile shows).
  const programming = navShown(v, 'programs');
  return (
    <div class="panel" data-panel="commands">
      <h2>Commands</h2>
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
                    data-testid={`run-${c.id}`}
                    title="Run it once, now"
                    disabled={!c.unlocked}
                    onClick={() => act({ type: 'run_command', command: c.id })}
                  >
                    {c.unlocked ? 'Run' : 'Locked'}
                  </button>
                  {programming && c.unlocked && (
                    <button type="button" data-testid={`add-${c.id}`} onClick={() => add(c.id)}>
                      Add to Program
                    </button>
                  )}
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
        {Number(c.boredom) > 0 && (
          <div class="flow neg">
            <ResChip v={v} id="boredom" /> +{num(c.boredom, 2)} Boredom
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
                  <ResChip v={v} id={m.good} /> {m.name}
                </td>
                <td>{m.demand !== null ? `${num(Number(m.demand) * 100, 0)}% (${m.tier})` : m.tier}</td>
                {/* Under 10, a decimal: a pool of 0.4 still draws Disrupt Speculators first. */}
                <td>{m.speculators !== null ? num(m.speculators, Number(m.speculators) < 10 ? 1 : 0) : '—'}</td>
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
            const label = { idle: p.unpaid ? 'idle: upkeep unpaid' : 'inactive', paused: 'Paused', nocargo: 'No cargo: pick one', ready: 'Ready', blocked: 'Full: no fuel', cooling: `Full: ready in ${p.cooldown} day${p.cooldown === 1 ? '' : 's'}`, loading: 'loading' }[state];
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
  const shown = v.research.filter((r) => r.visible || r.done);
  const cats = [...new Set(v.research.map((r) => r.category))];
  const sci = v.resources.find((r) => r.id === 'science');
  return (
    <div class="panel" data-panel="research">
      <h2>Research</h2>
      <p class="muted">
        <ResChip v={v} id="science" /> {num(sci?.amount ?? '0')} Science ({perDay(sci?.per_day ?? '0')}). Research resets each
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
export function ResearchCard({ v, r, fresh }: { v: View; r: ResearchView; fresh: Set<string> }) {
  const key = `research:${r.id}`;
  const [hint, showHint] = useHint();
  const needs = r.requires && !v.research.find((x) => x.id === r.requires)?.done ? r.requires : null;
  // Reads the engine's view now, not this render's: a double-click researches once.
  const research = () => {
    const now = store.view.research.find((x) => x.id === r.id);
    if (!now || now.done) return;
    if (!now.affordable) {
      const req = now.requires && !store.view.research.find((x) => x.id === now.requires)?.done ? now.requires : null;
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
        <h3>{r.name}</h3>
        {r.done && <span class="ok">✓ Done</span>}
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

// ---- Projects ----------------------------------------------------------------------

export function Projects({ s }: { s: Store }) {
  const v = s.view;
  const scopes: [ProjectView['scope'], string, string][] = [
    ['run', 'Strategic (this run)', 'This run only: progress resets on retirement.'],
    ['timeline', 'Global (this timeline)', 'Progress and completion carry across retirements until the timeline ends; funding rates reset, so set them again each run.'],
    ['forever', 'Forever', 'Progress and completion are kept for good.'],
  ];
  return (
    <div class="panel" data-panel="projects">
      <h2>Projects</h2>
      <p class="muted">Fund a project by setting a daily rate per resource; it drains that much each day until done.</p>
      {scopes.map(([scope, title, note]) => {
        const items = v.projects.filter((p) => p.scope === scope);
        if (items.length === 0) return null;
        return (
          <Section key={scope} title={`${title} (${items.length})`} dark>
            <p class="muted small">{note}</p>
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

export function Project({ v, p, fresh }: { v: View; p: ProjectView; fresh: Set<string> }) {
  const key = `project:${p.id}`;
  const step = (res: string, by: number) => {
    // From the engine's rate now, not this render's: quick clicks must each count.
    const cur = store.view.projects.find((x) => x.id === p.id) ?? p;
    const now = Number(cur.rates[res] ?? '0');
    const next = Math.max(0, Math.min(Number(p.max_rates[res] ?? '0'), now + by));
    // Six places, as the engine reads a number: float sums of a max like 1.666667 carry
    // more (0.6666669999999999), which it refuses.
    act({ type: 'set_project_rate', project: p.id, resource: res, rate: next.toFixed(6) });
  };
  const locked = !p.unlocked && !p.completed;
  const funded = p.unlocked && Object.values(p.rates).some((r) => Number(r) > 0);
  const state = p.completed && !p.unlocked ? ' done' : locked ? ' locked' : '';
  return (
    <div class={`card project${state}${freshClass(fresh, key)}`} data-id={p.id} onMouseEnter={() => seen(key)}>
      <div class="card-head">
        <h3>{p.name}</h3>
        {p.completed && !p.repeatable && <span class="ok">✓ Complete</span>}
      </div>
      <p class="desc">{p.description}</p>
      {locked && <p class="muted small">Locked: {p.unlock_text}</p>}
      {p.reward_text.map((t, i) => (
        <p class="small" key={i}>
          Reward: {t}
        </p>
      ))}
      {p.repeatable && p.completions > 0 && (
        <p class="ok small">
          Completed {p.completions} {p.completions === 1 ? 'time' : 'times'}
        </p>
      )}
      {funded && (
        <p class="muted small">
          {p.days_to_done !== null
            ? `Done in ${p.days_to_done} ${p.days_to_done === 1 ? 'day' : 'days'}`
            : 'Not every cost is funded'}
        </p>
      )}
      {p.unlocked &&
        Object.entries(p.costs).map(([res, cost]) => {
          const got = p.progress[res] ?? '0';
          const rate = p.rates[res] ?? '0';
          const max = p.max_rates[res] ?? '0';
          return (
            <div class="proj-row" key={res}>
              <span class="proj-res">
                <ResChip v={v} id={res} /> {name(v, res)}
              </span>
              <Bar frac={Number(got) / Number(cost)} label={`${numBelow(got, cost)} / ${num(cost)}`} />
              <span class="proj-rate">{num(rate)}/day</span>
              <button type="button" class="tiny" disabled={Number(rate) <= 0} onClick={() => step(res, -1)}>
                −
              </button>
              <button type="button" class="tiny" disabled={Number(rate) >= Number(max)} onClick={() => step(res, 1)}>
                +
              </button>
              <button type="button" class="tiny" title={`Fund at the most (${num(max)}/day)`} onClick={() => step(res, Number(max))}>
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
        discounts, among them the price of buildings tagged with it; rank 5 is a major rank with its own change.
        Scores carry into the next run, halving every year of the years away.
      </p>
      {v.ideology.map((a) => {
        const lo = Number(a.rank_score);
        const hi = Number(a.next_rank_score);
        const sc = Math.abs(Number(a.score));
        // Below rank 1 either way the rank is 0: the score's sign says which way it heads.
        const down = Number(a.score) < 0;
        return (
          <div class="axis" key={a.axis} data-axis={a.axis}>
            <h3 class={`axis-${a.axis}`}>
              <Chip id={a.axis} title={capitalize(a.axis)} size={18} /> {capitalize(a.axis)}
            </h3>
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
      <Section title="Attention">
        <p class="muted">Selling draws speculators; claiming land draws the rival AIs. Each comes in once its Attention reaches its threshold, and Attention fades day by day.</p>
        <table class="grid-table">
          <tbody>
            {v.adversaries.map((a) => (
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
      <Section title="Resource breakdown" testid="stats-breakdown">
        <p class="muted hint">
          Per day, by source (+) and sink (−): the last day, and a rolling average over about {v.flow_average_days} days. Each day
          Solar Panels produce, programs are paid next, and buildings with upkeep share what is left.
        </p>
        <div class="breakdown-grid">
          {v.breakdown.map((b) => (
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
          <Rows rows={v.overclocks.map((o, i) => [`Overclock ${i + 1}: ${o.target}`, `+${num(Number(o.bonus) * 100)}% through day ${o.until_day - 1}`])} />
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

function delta(now: Fx, next: Fx, fmt: (x: Fx) => string): string {
  return Number(now) === Number(next) ? fmt(next) : `${fmt(now)} → ${fmt(next)}`;
}

export function NextRunRows({ v }: { v: View }) {
  const c = v.career;
  const n = v.next_run;
  const rows: [ComponentChildren, string][] = [
    ['Starting credits', delta(c.starting_credits, n.starting_credits, (x) => num(x))],
    ['Boredom growth (resilience)', delta(c.boredom_resilience, n.boredom_resilience, mult)],
  ];
  if (Number(n.buy_power_mult) !== 1) rows.push(['Buy Power scaling', delta(c.buy_power_mult, n.buy_power_mult, mult)]);
  for (const a of n.ideology) {
    if (Number(a.score) > 0) rows.push([
      <>
        <Chip id={a.axis} title={capitalize(a.axis)} /> {capitalize(a.axis)} carried
      </>,
      `score ${num(a.score)} (rank ${a.rank})`,
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
        Retiring ends this run: buildings, resources, research, Strategic projects and every project's funding rates reset. The career keeps its
        bests, achievements, global projects, quests, saved programs, ideology (fading through the years away), adversaries and what you have
        seen. Boredom at its cap retires you anyway.
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
