// A program's lines that aren't commands (if, wait until, build, buy land): their editors
// and the buttons that add them. Every option, value and true/false mark is the view's.

import { useState } from 'preact/hooks';
import type { Clause, Cmp, Condition, ConditionView, Entry, LineOutcome, Reading, View } from '../engine';
import { num } from '../format';
import { name } from './common';
import { sameReading } from './programEdit';

const CMPS: [Cmp, string][] = [
  ['lt', '<'],
  ['le', '≤'],
  ['gt', '>'],
  ['ge', '≥'],
  ['eq', '='],
];

/** What a line did, in words (tooltips). */
export const OUTCOME_TEXT: Record<LineOutcome, string> = {
  ran: 'ran',
  skipped: 'skipped (output at its cap)',
  failed: 'failed',
  true: 'true',
  false: 'false, its lines skipped',
  jumped: 'skipped with its if',
  waiting: 'waiting',
  done: 'done, passed over',
};

/** A reading's label: the view's while it is on offer, else built from names. */
export function readingLabel(v: View, r: Reading): string {
  const offered = v.readings.find((o) => sameReading(o.reading, r));
  if (offered) return offered.label;
  switch (r.kind) {
    case 'amount':
      return name(v, r.resource);
    case 'percent':
      return `${name(v, r.resource)} %`;
    case 'rate':
      return `${name(v, r.resource)} rate`;
    case 'count':
      return `${name(v, r.building)} count`;
    case 'demand':
      return `${name(v, r.good)} demand`;
    case 'day':
      return 'Day';
    case 'pads_ready':
      return 'Pads ready';
  }
}

function shown(r: Reading, value: string | null): string {
  if (value === null) return '?';
  return r.kind === 'percent' ? `${num(value)}%` : r.kind === 'rate' ? `${Number(value) > 0 ? '+' : ''}${num(value)}/day` : num(value, 2);
}

/** A condition as text: display names, never ids. */
export function conditionText(v: View, when: Condition): string {
  const one = (c: Clause) => `${readingLabel(v, c.reading)} ${CMPS.find(([k]) => k === c.cmp)?.[1] ?? c.cmp} ${num(c.value, 2)}`;
  return when.rest ? `${one(when.first)} ${when.rest.join} ${one(when.rest.clause)}` : one(when.first);
}

/** A line as text (tooltips, the notes key). */
export function lineText(v: View, e: Entry): string {
  switch (e.op) {
    case 'command':
      return `${name(v, e.command)} (x${e.count})`;
    case 'build':
      return `Build ${name(v, e.building)}${e.up_to != null ? ` up to ${e.up_to}` : ''}`;
    case 'buy_land':
      return 'Buy Land';
    case 'if':
      return `If ${conditionText(v, e.when)}`;
    case 'wait_until':
      return `Wait until ${conditionText(v, e.when)}`;
  }
}

/** A number field that keeps what is being typed through redraws (the game redraws
 * every day) and sends it when the field is left or Enter is pressed: a rewrite restarts
 * the program. `commit` returns false to refuse the text, which then reverts. */
function NumberField({ value, commit, ...rest }: { value: string; commit: (text: string) => void } & Record<string, unknown>) {
  const [draft, setDraft] = useState<string | null>(null);
  // Redraws after every change, so refused text goes back to the value at once.
  const [, redraw] = useState(0);
  return (
    <input
      type="number"
      {...rest}
      value={draft ?? value}
      onInput={(ev) => setDraft(ev.currentTarget.value)}
      onChange={(ev) => {
        const text = ev.currentTarget.value.trim();
        setDraft(null);
        redraw((n) => n + 1);
        // Text a number field can't parse ("1e", "-") reads as empty: not a commit.
        if (!ev.currentTarget.validity.badInput) commit(text);
      }}
      onBlur={() => setDraft(null)}
    />
  );
}

function ClauseEdit({ v, c, now, set }: { v: View; c: Clause; now?: { value: string | null; holds: boolean }; set: (c: Clause) => void }) {
  const options = v.readings.some((o) => sameReading(o.reading, c.reading))
    ? v.readings
    : [{ reading: c.reading, label: readingLabel(v, c.reading), value: null }, ...v.readings];
  const at = options.findIndex((o) => sameReading(o.reading, c.reading));
  return (
    <span class="clause">
      <select aria-label="Reading" value={String(at)} onChange={(ev) => set({ ...c, reading: options[Number(ev.currentTarget.value)].reading })}>
        {options.map((o, i) => (
          <option key={i} value={String(i)}>
            {o.label}
          </option>
        ))}
      </select>
      <select aria-label="Comparison" value={c.cmp} onChange={(ev) => set({ ...c, cmp: ev.currentTarget.value as Cmp })}>
        {CMPS.map(([k, sign]) => (
          <option key={k} value={k}>
            {sign}
          </option>
        ))}
      </select>
      <NumberField
        step="any"
        aria-label="Number"
        // The engine's "500.000000" as 500.
        value={String(Number(c.value))}
        commit={(text) => {
          if (/^-?\d{1,12}(\.\d{1,6})?$/.test(text)) set({ ...c, value: text });
        }}
      />
      {now && (
        <span class={`cond-now ${now.holds ? 'is-true' : 'is-false'}`} data-cond-now title={`${readingLabel(v, c.reading)} now`}>
          {shown(c.reading, now.value)} {now.holds ? '✓' : '✗'}
        </span>
      )}
    </span>
  );
}

function ConditionEdit({ v, when, now, set }: { v: View; when: Condition; now: ConditionView | null; set: (c: Condition) => void }) {
  const rest = when.rest;
  return (
    <>
      <ClauseEdit v={v} c={when.first} now={now?.clauses[0]} set={(first) => set({ ...when, first })} />
      <select
        aria-label="Join"
        class="join"
        value={rest?.join ?? ''}
        onChange={(ev) => {
          const join = ev.currentTarget.value as 'and' | 'or' | '';
          set({ ...when, rest: join === '' ? null : { join, clause: rest?.clause ?? when.first } });
        }}
      >
        <option value="">…</option>
        <option value="and">and</option>
        <option value="or">or</option>
      </select>
      {rest && <ClauseEdit v={v} c={rest.clause} now={now?.clauses[1]} set={(clause) => set({ ...when, rest: { ...rest, clause } })} />}
    </>
  );
}

/** The editor of one non-command line (the row's grip and buttons are the caller's). */
export function LogicBody({ v, e, now, set }: { v: View; e: Entry; now: ConditionView | null; set: (e: Entry) => void }) {
  switch (e.op) {
    case 'if':
    case 'wait_until':
      return (
        <span class="line-logic" data-holds={now ? String(now.holds) : undefined}>
          <b>{e.op === 'if' ? 'If' : 'Wait until'}</b>
          <ConditionEdit v={v} when={e.when} now={now} set={(when) => set({ ...e, when })} />
        </span>
      );
    case 'build': {
      const options = v.buildings.filter((b) => b.visible || b.id === e.building);
      const owned = v.buildings.find((b) => b.id === e.building)?.owned ?? 0;
      return (
        <span class="line-logic">
          <b>Build</b>
          <select aria-label="Building" value={e.building} onChange={(ev) => set({ ...e, building: ev.currentTarget.value })}>
            {options.map((b) => (
              <option key={b.id} value={b.id}>
                {b.name}
              </option>
            ))}
          </select>
          <label class="up-to">
            up to
            <NumberField
              min="1"
              step="1"
              placeholder="any"
              aria-label="Up to"
              title="Stop once this many are owned (empty: build one every pass)"
              value={e.up_to == null ? '' : String(e.up_to)}
              commit={(text) => {
                // Empty: no limit. Anything but a whole number from 1 reverts.
                if (text === '') set({ ...e, up_to: null });
                else if (/^\d{1,9}$/.test(text) && Number(text) >= 1) set({ ...e, up_to: Number(text) });
              }}
            />
          </label>
          <span class="muted" data-owned>
            have {owned}
          </span>
        </span>
      );
    }
    case 'buy_land':
      return (
        <span class="line-logic">
          <b>Buy Land</b>
          <span class="muted">
            {num(v.land_price)} {name(v, 'credits')}
          </span>
        </span>
      );
    case 'command':
      return null;
  }
}

/** The buttons adding non-command lines, or how they unlock. */
export function AddLogic({ v, add }: { v: View; add: (e: Entry) => void }) {
  if (!v.program_logic) {
    return (
      <p class="muted hint" data-testid="logic-locked">
        If, Wait until, Build and Buy Land lines unlock for good the first day {v.logic_unlock_processors} processors work.
      </p>
    );
  }
  const first = v.readings[0]?.reading;
  const when: Condition | null = first ? { first: { reading: first, cmp: 'gt', value: '0' }, rest: null } : null;
  const building = v.buildings.find((b) => b.visible)?.id;
  return (
    <div class="add-logic" data-testid="logic-add">
      <span class="muted">Add:</span>
      <button type="button" class="tiny" disabled={!when} title="The indented lines under it run only when it is true" onClick={() => when && add({ op: 'if', when })}>
        If
      </button>
      <button type="button" class="tiny" disabled={!when} title="Holds the program here until it is true" onClick={() => when && add({ op: 'wait_until', when })}>
        Wait until
      </button>
      <button type="button" class="tiny" disabled={!building} title="Buys one building a step" onClick={() => building && add({ op: 'build', building, up_to: null })}>
        Build
      </button>
      <button type="button" class="tiny" title="Buys land" onClick={() => add({ op: 'buy_land' })}>
        Buy Land
      </button>
    </div>
  );
}
