// A program's lines: a command line's row (ProgramLine), the editors of the lines that
// aren't commands (if, wait until, build, buy land) and the buttons that add them. Every
// option, value and true/false mark is the view's.

import type { ComponentChildren, LiHTMLAttributes } from 'preact';
import { useState } from 'preact/hooks';
import type { Amount, Clause, Cmp, Condition, ConditionView, Entry, LineOutcome, Reading, ShortView, Verb, View } from '../engine';
import { num } from '../format';
import { Bar, ResChip, ShortNote, name, ranPercent, shortText } from './common';
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
  // The view offers `every` at one N; a line holds its own.
  if (r.kind === 'every') return `Every ${r.days} day${r.days === 1 ? '' : 's'}`;
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
      return `Ready pads ${r.good ? name(v, r.good) : '(all)'}`;
    case 'cargo':
      return `Cargo of pads ${r.good ? name(v, r.good) : '(all)'}`;
    case 'next_cost':
      return `Next cost of ${name(v, r.building)}`;
    case 'land_price':
      return 'Land price';
    case 'days_to_cap':
      return `Days to cap of ${name(v, r.resource)}`;
    case 'days_since_launch':
      return 'Days since last launch';
  }
}

/** The verb's display label: the view's. */
function verbLabel(v: View, verb: Verb): string {
  return v.verbs.find((x) => x.verb === verb)?.label ?? verb;
}

/** An `act` line as text: `<verb> <target> [amount]`, display names. */
function actText(v: View, e: Extract<Entry, { op: 'act' }>): string {
  const spec = v.verbs.find((x) => x.verb === e.verb);
  const target = e.target ? spec?.targets?.find((t) => t.value === e.target)?.label ?? name(v, e.target) : spec?.target_all ? 'all' : '';
  const tab = e.tab != null ? (e.verb === 'load_program' ? `into Program ${e.tab + 1}` : `Program ${e.tab + 1}`) : '';
  const amount = e.amount == null ? '' : e.amount === 'all' ? 'all' : e.verb === 'set_pads' ? `up to ${e.amount.n}` : String(e.amount.n);
  return [verbLabel(v, e.verb), target, tab, amount].filter((s) => s !== '').join(' ');
}

function shown(r: Reading, value: string | null): string {
  if (value === null) return '?';
  return r.kind === 'percent' ? `${num(value)}%` : r.kind === 'rate' ? `${Number(value) > 0 ? '+' : ''}${num(value)}/day` : num(value, 2);
}

/** A condition as text: display names, never ids. */
export function conditionText(v: View, when: Condition): string {
  const one = (c: Clause) =>
    c.reading.kind === 'every'
      ? readingLabel(v, c.reading)
      : `${readingLabel(v, c.reading)} ${CMPS.find(([k]) => k === c.cmp)?.[1] ?? c.cmp} ${c.against ? `${readingLabel(v, c.against)} × ` : ''}${num(c.value, 2)}`;
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
    case 'act':
      return actText(v, e);
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

/** The right side's optional reading: `<number>` alone, or `<reading> × <number>`. */
function AgainstEdit({ v, c, set }: { v: View; c: Clause; set: (c: Clause) => void }) {
  const against = c.against ?? null;
  const offered = v.readings.filter((o) => o.reading.kind !== 'every');
  const options = !against || offered.some((o) => sameReading(o.reading, against)) ? offered : [{ reading: against, label: readingLabel(v, against), value: null }, ...offered];
  const at = against ? options.findIndex((o) => sameReading(o.reading, against)) : -1;
  return (
    <select
      aria-label="Times reading"
      title="Compare with another reading times the number"
      value={String(at)}
      onChange={(ev) => {
        const i = Number(ev.currentTarget.value);
        set({ ...c, against: i < 0 ? null : options[i].reading });
      }}
    >
      <option value="-1">number</option>
      {options.map((o, i) => (
        <option key={i} value={String(i)}>
          {o.label} ×
        </option>
      ))}
    </select>
  );
}

function ClauseEdit({ v, c, now, set }: { v: View; c: Clause; now?: { value: string | null; holds: boolean }; set: (c: Clause) => void }) {
  const options = v.readings.some((o) => sameReading(o.reading, c.reading))
    ? v.readings
    : [{ reading: c.reading, label: readingLabel(v, c.reading), value: null }, ...v.readings];
  const at = options.findIndex((o) => sameReading(o.reading, c.reading));
  return (
    <span class="clause">
      <select
        aria-label="Reading"
        value={String(at)}
        onChange={(ev) => {
          const reading = options[Number(ev.currentTarget.value)].reading;
          // `every N days` is the clause `= 1`, with no right-side reading.
          set(reading.kind === 'every' ? { reading, cmp: 'eq', value: '1', against: null } : { ...c, reading });
        }}
      >
        {options.map((o, i) => (
          <option key={i} value={String(i)}>
            {o.label}
          </option>
        ))}
      </select>
      {c.reading.kind === 'every' ? (
        // `every N days` is true on its days: the clause is `= 1`, N its field.
        <NumberField
          min="1"
          step="1"
          aria-label="Days"
          value={String(c.reading.days)}
          commit={(text) => {
            if (/^\d{1,6}$/.test(text) && Number(text) >= 1) set({ ...c, reading: { kind: 'every', days: Number(text) }, cmp: 'eq', value: '1', against: null });
          }}
        />
      ) : (
        <>
          <select aria-label="Comparison" value={c.cmp} onChange={(ev) => set({ ...c, cmp: ev.currentTarget.value as Cmp })}>
            {CMPS.map(([k, sign]) => (
              <option key={k} value={k}>
                {sign}
              </option>
            ))}
          </select>
          <AgainstEdit v={v} c={c} set={set} />
          <NumberField
            step="any"
            aria-label="Number"
            // The engine's "500.000000" as 500.
            value={String(Number(c.value))}
            commit={(text) => {
              if (/^-?\d{1,12}(\.\d{1,6})?$/.test(text)) set({ ...c, value: text });
            }}
          />
        </>
      )}
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

/**
 * One command line's row: its chips and name, progress bar, count −/+ and why it failed.
 * The caller (Right.tsx `Programs`) gives what it shares with the logic lines: the row's
 * class, drag handlers, tooltip tail (`ranText`) and indent button. `noted` holds the lines
 * that have shown a note since the program's lines last changed: such a line keeps the
 * note's row, so a note coming and going day to day moves no line below it.
 */
export function ProgramLine(props: {
  v: View;
  e: Extract<Entry, { op: 'command' }>;
  i: number;
  outcome: LineOutcome | null;
  /** Framed: the next line to run. */
  current: boolean;
  /** Runs done of the current line. */
  step: number;
  short: ShortView | null;
  ranText: string;
  rowClass: string;
  drag: LiHTMLAttributes<HTMLLIElement>;
  indent: ComponentChildren;
  noted: Set<number>;
  bump: (by: number) => void;
  remove: () => void;
}) {
  const { v, e, i, outcome, current, step, short, ranText, rowClass, drag, indent, noted, bump, remove } = props;
  const frac = current && step > 0 ? step / e.count : outcome ? 1 : 0;
  // A failed line says why (today's last run of it); the tooltip has it in full.
  // A conversion that ran part way says so too, as a starved building does.
  const why = outcome === 'failed' || outcome === 'ran' ? short : null;
  const command = v.commands.find((c) => c.id === e.command);
  // A command locked again (used in an earlier run) fails with no input short.
  const locked = outcome === 'failed' && !why && command?.unlocked === false;
  if (why || locked) noted.add(i);
  return (
    <li
      class={rowClass}
      data-line={i}
      data-command={e.command}
      {...drag}
      // The name can be cut short at narrow widths: the tooltip carries it whole.
      title={`${lineText(v, e)}: ${ranText}${why ? `. ${shortText(v, why)}` : locked ? '. Locked this run.' : ''}`}
    >
      <span class="grip" aria-hidden>
        ⠿
      </span>
      {/* The current line is marked by its frame (style.css `.line.current`), never by
          text in the name, so no line's text moves as the pointer walks. */}
      <span class="line-name">
        {Object.keys(command?.produces ?? {}).map((res) => (
          <ResChip key={res} v={v} id={res} />
        ))}{' '}
        {name(v, e.command)} (x{e.count})
      </span>
      <Bar frac={frac} tone={outcome === 'failed' ? 'red' : outcome === 'skipped' ? 'gray' : 'green'} />
      <button type="button" class="tiny" disabled={e.count <= 1} onClick={() => bump(-1)}>
        −
      </button>
      <button type="button" class="tiny" onClick={() => bump(1)}>
        +
      </button>
      <span class="line-buttons">
        {indent}
        <button type="button" class="tiny" title="Remove" onClick={remove}>
          ×
        </button>
      </span>
      {!(why || locked) && noted.has(i) && (
        <span class="line-why" aria-hidden="true">
          {' '}
        </span>
      )}
      {(why || locked) && (
        <span class="line-why" data-line-why>
          {why ? (
            <>
              {Number(why.ran) > 0 ? `ran at ${ranPercent(why.ran)} · ` : ''}
              <ShortNote v={v} s={why} />
            </>
          ) : (
            'locked this run'
          )}
        </span>
      )}
    </li>
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
    case 'act':
      return <ActEdit v={v} e={e} set={set} />;
    case 'command':
      return null;
  }
}

/** A fresh `act` line for `verb`: its first target (or all) and its default amount. */
function newAct(v: View, verb: Verb): Extract<Entry, { op: 'act' }> {
  const spec = v.verbs.find((x) => x.verb === verb);
  const target = spec?.targets ? (spec.target_all ? null : spec.targets[0]?.value ?? null) : null;
  const amount: Amount | null = spec?.amount === 'up_to' ? { n: 1 } : spec?.amount === 'n_or_all' ? 'all' : null;
  return { op: 'act', verb, target, tab: spec?.tab ? 0 : null, amount };
}

/** The one editor of every verb: `<verb> <target> [tab] [amount]`, each slot a dropdown or a
 * number as the view's verb list says. */
function ActEdit({ v, e, set }: { v: View; e: Extract<Entry, { op: 'act' }>; set: (e: Entry) => void }) {
  const spec = v.verbs.find((x) => x.verb === e.verb);
  const targets = spec?.targets ?? null;
  const known = !e.target || targets?.some((t) => t.value === e.target);
  const options = targets && !known && e.target ? [{ value: e.target, label: name(v, e.target) }, ...targets] : targets;
  const amountText = e.amount == null ? '' : e.amount === 'all' ? 'all' : String(e.amount.n);
  return (
    <span class="line-logic" data-verb={e.verb}>
      <select aria-label="Verb" value={e.verb} onChange={(ev) => set({ ...newAct(v, ev.currentTarget.value as Verb), indented: e.indented })}>
        {v.verbs.map((x) => (
          <option key={x.verb} value={x.verb}>
            {x.label}
          </option>
        ))}
      </select>
      {options && (
        <select aria-label="Target" value={e.target ?? ''} onChange={(ev) => set({ ...e, target: ev.currentTarget.value === '' ? null : ev.currentTarget.value })}>
          {spec?.target_all && <option value="">all</option>}
          {options.map((t) => (
            <option key={t.value} value={t.value}>
              {t.label}
            </option>
          ))}
        </select>
      )}
      {spec?.tab && (
        <label>
          {e.verb === 'load_program' ? 'into' : ''}
          <select aria-label="Program" value={String(e.tab ?? 0)} onChange={(ev) => set({ ...e, tab: Number(ev.currentTarget.value) })}>
            {v.programs.map((_, i) => (
              <option key={i} value={String(i)}>
                Program {i + 1}
              </option>
            ))}
          </select>
        </label>
      )}
      {spec && spec.amount !== 'none' && (
        <label class="up-to">
          {spec.amount === 'up_to' ? 'up to' : ''}
          {/* A draft survives the daily redraws (NumberField); refused text reverts. */}
          <NumberField
            type="text"
            size={4}
            aria-label="Amount"
            placeholder={spec.amount === 'optional' ? '1' : ''}
            title={spec.amount === 'up_to' ? 'A whole number from 1' : 'A whole number, or all'}
            value={amountText}
            commit={(raw) => {
              const text = raw.toLowerCase();
              const least = e.verb === 'assign_processors' ? 0 : 1;
              if (text === '' && spec.amount === 'optional') set({ ...e, amount: null });
              else if (text === 'all' && spec.amount !== 'up_to') set({ ...e, amount: 'all' });
              else if (/^\d{1,9}$/.test(text) && Number(text) >= least) set({ ...e, amount: { n: Number(text) } });
            }}
          />
        </label>
      )}
    </span>
  );
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
      <button
        type="button"
        class="tiny"
        data-testid="add-act"
        disabled={v.verbs.length === 0}
        title="A verb line: launch, load or set pads, pause, resume, research, load a program, assign processors, retire"
        onClick={() => v.verbs[0] && add(newAct(v, v.verbs[0].verb))}
      >
        Verb
      </button>
    </div>
  );
}
