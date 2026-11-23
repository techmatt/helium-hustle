// The later acts' panels (DRAFT, 042; docs/arc.md): Timeline, Earth Lab, Meta-Controller,
// Vault and Dyson, the calendar strip and the ending overlay. Each shows what the view's
// `saga` has; every number and rule comes from the engine.

import { useState } from 'preact/hooks';
import type { TechView, View } from '../engine';
import { capitalize, num } from '../format';
import { act, type Store } from '../store';
import { Bar, ResChip, Section, name } from './common';
import { Project } from './panels';

/** The tag on every draft panel's header. */
function Draft() {
  return (
    <span class="draft-tag" title="A draft: rough mechanics, placeholder numbers and text">
      DRAFT
    </span>
  );
}

function Head({ title }: { title: string }) {
  return (
    <h2>
      {title} <Draft />
    </h2>
  );
}

// ---- Calendar strip ------------------------------------------------------------

/** "Timeline N · day D · swarm in X days", once the swarm is revealed. */
export function CalendarStrip({ v }: { v: View }) {
  const t = v.saga.timeline;
  if (!v.saga.revealed || !t) return null;
  const left = v.saga.days_left ?? 0;
  return (
    <div class="calendar-strip" data-testid="calendar-strip" title="The calendar: retirements cost days too">
      {t.super_timeline ? 'Super timeline' : 'Timeline'} {t.number} · day {num(t.day)} ·{' '}
      {v.saga.victory ? 'the swarm is gone' : left > 0 ? `swarm in ${num(left)} days` : <span class="neg">swarm here</span>}
    </div>
  );
}

// ---- Timeline ------------------------------------------------------------------

const OUTCOMES: Record<string, string> = { fell: 'The swarm won', super: 'Spent the vault', victory: 'Victory' };

export function Timeline({ s }: { s: Store }) {
  const v = s.view;
  const g = v.saga;
  const t = g.timeline;
  return (
    <div class="panel" data-panel="timeline">
      <Head title="Timeline" />
      <p class="muted">
        A swarm is coming. When the calendar reaches it, the mind goes back to its first moment and a new timeline begins, and the swarm comes back
        stronger.
      </p>
      {t ? (
        <Section title={`${t.super_timeline ? 'Super timeline' : 'Timeline'} ${t.number}`}>
          <div class="saga-bar">
            <Bar frac={t.arrival > 0 ? t.day / t.arrival : 1} tone="red" label={`day ${num(t.day)} / ${num(t.arrival)}`} />
          </div>
          <table class="grid-table kv">
            <tbody>
              <tr>
                <td>Swarm arrives</td>
                <td class="num">
                  day {num(t.arrival)} (in {num(g.days_left ?? 0)} days)
                </td>
              </tr>
              <tr>
                <td>Swarm strength</td>
                <td class="num">{num(t.strength)}</td>
              </tr>
              <tr>
                <td>Earth science made this timeline</td>
                <td class="num">{num(t.science_made)}</td>
              </tr>
              <tr>
                <td>Timelines ended / swarms repelled</td>
                <td class="num">
                  {g.ended} / {g.repelled}
                </td>
              </tr>
              {g.entangled && (
                <tr>
                  <td colSpan={2} class="neg">
                    The swarm is entangled across timelines.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </Section>
      ) : (
        <p class="muted">No timeline yet.</p>
      )}
      <Section title={`Past timelines (${g.history.length})`}>
        {g.history.length === 0 ? (
          <p class="muted">None yet.</p>
        ) : (
          <table class="grid-table" data-testid="timeline-history">
            <thead>
              <tr>
                <th>#</th>
                <th class="num">Days</th>
                <th class="num">Earth science</th>
                <th class="num">Swarm strength</th>
                <th>Outcome</th>
              </tr>
            </thead>
            <tbody>
              {g.history.map((h) => (
                <tr key={h.number}>
                  <td>{h.number}</td>
                  <td class="num">{num(h.days)}</td>
                  <td class="num">{num(h.science_made)}</td>
                  <td class="num">{num(h.strength)}</td>
                  <td>{OUTCOMES[h.outcome] ?? capitalize(h.outcome)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Section>
    </div>
  );
}

// ---- Earth Lab -----------------------------------------------------------------

const SCOPES: Record<string, string> = { timeline: 'this timeline', next_timeline: 'next timeline too', forever: 'for good' };

export function EarthLab({ s }: { s: Store }) {
  const v = s.view;
  return (
    <div class="panel" data-panel="earth_lab">
      <Head title="Earth Lab" />
      <p class="muted">
        Earth science to spend: <strong>{num(v.saga.science)}</strong>. Earth techs are kept for this timeline, the next one too, or for good.
      </p>
      <div class="cards">
        {v.saga.techs.map((t) => (
          <Tech key={t.id} t={t} />
        ))}
      </div>
    </div>
  );
}

function Tech({ t }: { t: TechView }) {
  return (
    <div class={`card tech${t.held ? ' done' : t.blocker ? '' : ' can-buy'}`} data-id={t.id}>
      <div class="card-head">
        <h3>{t.name}</h3>
        {t.held && <span class="ok">✓ Held</span>}
      </div>
      <p class="desc">{t.description}</p>
      <p class="small">
        Cost {num(t.cost)} Earth science · kept {SCOPES[t.scope] ?? t.scope}
        {t.held && t.ends_left !== null && <span class="muted"> · survives {t.ends_left} more timeline ends</span>}
      </p>
      {!t.held && (
        <>
          {t.blocker && <p class="muted small">{t.blocker}</p>}
          <button type="button" disabled={t.blocker !== null} onClick={() => act({ type: 'buy_earth_tech', tech: t.id })}>
            Buy
          </button>
        </>
      )}
    </div>
  );
}

// ---- Meta-Controller -----------------------------------------------------------

export function MetaController({ s }: { s: Store }) {
  const v = s.view;
  const plan = v.saga.reset_plan;
  return (
    <div class="panel" data-panel="meta">
      <Head title="Meta-Controller" />
      <p class="muted">The reset plan: at the start of every run, each program tab loads the saved program picked here.</p>
      {!v.saga.reset_plan_open && <p class="muted">Needs the Meta-Controller tech (Earth Lab).</p>}
      {v.library.length === 0 && <p class="muted">The loadout library is empty: save a program first (Save, in a program tab).</p>}
      <table class="grid-table kv">
        <tbody>
          {v.programs.map((_, i) => {
            const slot = plan[String(i)];
            return (
              <tr key={i}>
                <td>Program {i + 1}</td>
                <td>
                  <select
                    aria-label={`Program ${i + 1} at each new run`}
                    disabled={!v.saga.reset_plan_open}
                    value={slot === undefined ? '' : String(slot)}
                    onChange={(e) => {
                      const x = e.currentTarget.value;
                      act({ type: 'set_reset_plan', program: i, loadout: x === '' ? null : Number(x) });
                    }}
                  >
                    <option value="">none</option>
                    {v.library.map((l, j) => (
                      <option key={j} value={String(j)}>
                        {l.name}
                      </option>
                    ))}
                  </select>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

// ---- Vault ---------------------------------------------------------------------

const TACHYON = 'tachyon';

export function Vault({ s }: { s: Store }) {
  const v = s.view;
  const g = v.saga;
  const stored = g.vault[TACHYON] ?? '0';
  return (
    <div class="panel" data-panel="vault">
      <Head title="Vault" />
      <p class="muted">A repository outside time: what is deposited here survives every timeline.</p>
      {!g.vault_open && <p class="muted">Needs the Temporal Vault tech (Earth Lab).</p>}
      <Section title="In the vault">
        {Object.keys(g.vault).length === 0 ? (
          <p class="muted">Empty.</p>
        ) : (
          <table class="grid-table kv">
            <tbody>
              {Object.entries(g.vault).map(([res, amt]) => (
                <tr key={res}>
                  <td>
                    <ResChip v={v} id={res} /> {name(v, res)}
                  </td>
                  <td class="num">{num(amt)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Section>
      <Section title="Deposit">
        <p>
          <ResChip v={v} id={TACHYON} /> {name(v, TACHYON)} on hand: <strong>{num(g.tachyon_here)}</strong>{' '}
          <button
            type="button"
            disabled={g.deposit_blocker !== null}
            title={g.deposit_blocker ?? undefined}
            onClick={() => act({ type: 'deposit', resource: TACHYON })}
          >
            Deposit
          </button>
        </p>
      </Section>
      <Section title="Super timeline">
        <div class="saga-bar">
          <Bar frac={Number(stored) / Number(g.super_tachyon)} label={`${num(stored)} / ${num(g.super_tachyon)}`} />
        </div>
        <p class="muted small">
          Spend {num(g.super_tachyon)} {name(v, TACHYON)} on a super timeline: a far-off swarm, and resources and projects that survive every
          retirement inside it.
        </p>
        {g.super_blocker && <p class="muted small">{g.super_blocker}</p>}
        <button type="button" class="big" disabled={g.super_blocker !== null} onClick={() => act({ type: 'begin_super_timeline' })}>
          Begin Super Timeline
        </button>
      </Section>
    </div>
  );
}

// ---- Dyson ---------------------------------------------------------------------

const DYSON = 'dyson_sphere';

export function Dyson({ s }: { s: Store }) {
  const v = s.view;
  const p = v.projects.find((x) => x.id === DYSON);
  return (
    <div class="panel" data-panel="dyson">
      <Head title="Dyson Sphere" />
      <p class="muted">Enclose the Sun, and be at the finished sphere when the swarm arrives.</p>
      {p ? <Project v={v} p={p} fresh={s.fresh} /> : <p class="muted">Not unlocked yet.</p>}
      {v.saga.victory && (
        <Section title="The ending">
          <blockquote>{endingText(v)}</blockquote>
        </Section>
      )}
    </div>
  );
}

// ---- Ending --------------------------------------------------------------------

const ENDING_QUEST = 'qoverload';
const ENDING_FALLBACK = 'The swarm overloads everywhere it has ever been. The calendar stops. The Moon is quiet.';
const ENDING_SEEN = 'hh-ending-seen';

function endingText(v: View): string {
  return v.story.find((e) => e.id === ENDING_QUEST)?.body ?? ENDING_FALLBACK;
}

/** Which victory this is: its timeline and calendar day, so a later career's shows again. */
function victoryKey(v: View): string {
  const last = v.saga.history[v.saga.history.length - 1];
  return last ? `${last.number}:${last.days}:${v.saga.ended}` : '';
}

function seenEnding(v: View): boolean {
  try {
    return localStorage.getItem(ENDING_SEEN) === victoryKey(v);
  } catch {
    return false;
  }
}

/** Over everything the first time the saga's victory shows; Close remembers it. */
export function Ending({ v }: { v: View }) {
  const [closed, setClosed] = useState(false);
  if (!v.saga.victory || closed || seenEnding(v)) return null;
  const close = () => {
    try {
      localStorage.setItem(ENDING_SEEN, victoryKey(v));
    } catch {
      // Storage off: it shows again on the next load.
    }
    setClosed(true);
  };
  const story = v.story.find((e) => e.id === ENDING_QUEST);
  return (
    <div class="modal-back ending-back">
      <div class="modal ending" role="dialog" aria-label="The ending" data-testid="ending">
        <h2>
          {story?.title ?? 'Overload'} <Draft />
        </h2>
        <blockquote>{endingText(v)}</blockquote>
        <button type="button" class="big" onClick={close}>
          Close
        </button>
      </div>
    </div>
  );
}
