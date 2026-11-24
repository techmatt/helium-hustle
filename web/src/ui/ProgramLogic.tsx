// A program's lines: a command line's row (ProgramLine), the editors of the lines that
// aren't commands (if, build, buy land, verbs; the text editor's notes and unparsed
// lines, shown but not edited here) and the buttons that add them. Every option, value
// and true/false mark is the view's.

import type { ComponentChildren, LiHTMLAttributes } from 'preact';
import { useState } from 'preact/hooks';
import type { Amount, Clause, Cmp, Condition, ConditionView, Entry, LineOutcome, Reading, ShortView, Verb, View } from '../engine';
import { num } from '../format';
import { Bar, ResChip, ShortNote, name, ranPercent, shortText } from './common';
import { otherUnit, sameEntry } from './programEdit';

const CMPS: [Cmp, string][] = [
  ['gt', '>'],
  ['lt', '<'],
  ['ge', '>='],
  ['le', '<='],
];

/** What a line did, in words (tooltips). */
export const OUTCOME_TEXT: Record<LineOutcome, string> = {
  ran: 'ran',
  skipped: 'skipped (output at its cap)',
  failed: 'failed',
  true: 'true',
  false: 'false, its lines skipped',
  jumped: 'skipped with its if',
  done: 'nothing to do (a step)',
};

/** A reading's label: the view's while it is on offer, else built from names (a line can
 * hold a reading not shown yet). */
export function readingLabel(v: View, r: Reading): string {
  const offered = v.readings.find((o) => sameEntry(o.reading, r));
  if (offered) return offered.label;
  switch (r.kind) {
    case 'amount':
    case 'percent':
      return name(v, r.resource);
    case 'free_processors':
      return 'Free processors';
    case 'free_land':
      return 'Free land';
    case 'idle_pads':
      return 'Idle pads';
    case 'energy_per_day':
      return 'Energy per day';
    case 'days_since_purchase':
      return 'Days since a purchase';
  }
}

/** Whether a clause's reading takes the unit toggle: a capped resource (its list entry
 * is a percent), read as a percent or an amount. */
function toggles(v: View, r: Reading): boolean {
  if (r.kind === 'percent') return true;
  return r.kind === 'amount' && v.readings.some((o) => o.reading.kind === 'percent' && sameEntry(o.reading, r));
}

/** A reading's unit, shown after its number: `%` for a fill, none for an amount or a count. */
function unit(r: Reading): string {
  return r.kind === 'percent' ? '%' : '';
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
  return r.kind === 'percent' ? `${num(value)}%` : num(value, 2);
}

/** A condition as text, display names, never ids: "Energy > 50%". */
export function conditionText(v: View, when: Condition): string {
  const one = (c: Clause) => `${readingLabel(v, c.reading)} ${CMPS.find(([k]) => k === c.cmp)?.[1] ?? c.cmp} ${num(c.value, 2)}${unit(c.reading)}`;
  return when.rest ? `${one(when.first)} ${when.rest.join} ${one(when.rest.clause)}` : one(when.first);
}

/** A line as text (tooltips, the notes key). */
export function lineText(v: View, e: Entry): string {
  switch (e.op) {
    case 'command':
      return e.count > 1 ? `${e.count}x ${name(v, e.command)}` : name(v, e.command);
    case 'build':
      return `Build ${name(v, e.building)}${e.up_to != null ? ` up to ${e.up_to}` : ''}`;
    case 'buy_land':
      return 'Buy Land';
    case 'if':
      return `If ${conditionText(v, e.when)}`;
    case 'else':
      return 'Else';
    case 'act':
      return actText(v, e);
    case 'note':
      return `# ${e.text}`;
    case 'unparsed':
      return e.text;
  }
}

/** A press held in a field: where it went down (a click or a drag) and how it ends. */
let held: { x: number; y: number; release: () => void } | null = null;

/** The press is over: its row is draggable again. Safe to call twice. */
function release() {
  held?.release();
  held = null;
}

/**
 * A field inside a draggable row: focusing it, or a click in it, selects its whole text,
 * so typing replaces it; a drag in it selects text as usual and moves no row (the row is
 * undraggable from the press until the button comes up).
 */
const fieldSelect = {
  // Focus from the keyboard (Tab) selects at once; a press waits for its mouseup, so a
  // drag selects what it covers.
  onFocus: (ev: FocusEvent) => held === null && (ev.currentTarget as HTMLInputElement).select(),
  onMouseDown: (ev: MouseEvent) => {
    if (ev.button !== 0) return;
    release();
    const el = ev.currentTarget as HTMLInputElement;
    // A press on selected text would drag it off as text: collapse it first, so the press
    // places the caret and a drag selects from there.
    if (document.activeElement === el && el.selectionStart !== el.selectionEnd) el.setSelectionRange(el.selectionEnd, el.selectionEnd);
    const row = el.closest<HTMLElement>('[draggable="true"]');
    if (row) row.draggable = false;
    // The row's `draggable` is set once by Preact (an unchanged prop isn't set again): it
    // comes back when the button comes up anywhere (after the field's own mouseup), or the
    // window loses the press (a context menu, a switch away).
    const end = (e: Event) => (e instanceof MouseEvent && e.button !== 0 ? undefined : release());
    held = {
      x: ev.clientX,
      y: ev.clientY,
      release: () => {
        if (row) row.draggable = true;
        window.removeEventListener('mouseup', end);
        window.removeEventListener('blur', end);
      },
    };
    window.addEventListener('mouseup', end);
    window.addEventListener('blur', end);
  },
  onMouseUp: (ev: MouseEvent) => {
    if (ev.button !== 0) return;
    const at = held;
    release();
    // A click (no drag): the whole text, not a caret where it landed.
    if (at && Math.abs(ev.clientX - at.x) < 4 && Math.abs(ev.clientY - at.y) < 4) {
      ev.preventDefault();
      (ev.currentTarget as HTMLInputElement).select();
    }
  },
};

/** A number field that keeps what is being typed through redraws (the game redraws
 * every day) and sends it when the field is left or Enter is pressed: a rewrite restarts
 * the program. `commit` checks the text and refuses what it can't use, which then reverts.
 * A text input (`inputmode` numeric), not `type="number"`: a number input's selection
 * can't be set, so a press on its selected text would drag the text rather than select. */
export function NumberField({ value, commit, ...rest }: { value: string; commit: (text: string) => void } & Record<string, unknown>) {
  const [draft, setDraft] = useState<string | null>(null);
  // Redraws after every change, so refused text goes back to the value at once.
  const [, redraw] = useState(0);
  return (
    <input
      type="text"
      inputMode="decimal"
      autoComplete="off"
      {...fieldSelect}
      {...rest}
      value={draft ?? value}
      onInput={(ev) => setDraft(ev.currentTarget.value)}
      onChange={(ev) => {
        const text = ev.currentTarget.value.trim();
        setDraft(null);
        redraw((n) => n + 1);
        commit(text);
      }}
      onBlur={() => setDraft(null)}
    />
  );
}

/** One clause: `<reading> <op> <number>[%]`, `op` one of `>`, `<`, `>=`, `<=`. */
function ClauseEdit({ v, c, now, set }: { v: View; c: Clause; now?: { value: string | null; holds: boolean }; set: (c: Clause) => void }) {
  const options = v.readings.some((o) => sameEntry(o.reading, c.reading))
    ? v.readings
    : [{ reading: c.reading, label: readingLabel(v, c.reading), value: null }, ...v.readings];
  const at = options.findIndex((o) => sameEntry(o.reading, c.reading));
  return (
    <span class="clause">
      <select
        aria-label="Reading"
        // A long reading ("Days since a purchase") is cut at narrow widths.
        title={options[at]?.label}
        value={String(at)}
        onChange={(ev) => set({ ...c, reading: options[Number(ev.currentTarget.value)].reading })}
      >
        {options.map((o, i) => (
          <option key={i} value={String(i)}>
            {o.label}
          </option>
        ))}
      </select>
      <select aria-label="Comparison" value={c.cmp} onChange={(ev) => set({ ...c, cmp: ev.currentTarget.value as Cmp })}>
        {CMPS.map(([k, word]) => (
          <option key={k} value={k}>
            {word}
          </option>
        ))}
      </select>
      <NumberField
        aria-label="Number"
        title={c.reading.kind === 'percent' ? 'A percent of its cap' : c.reading.kind === 'energy_per_day' ? 'Energy a day (negative: a loss)' : 'An amount'}
        // The engine's "500.000000" as 500.
        value={String(Number(c.value))}
        commit={(text) => {
          // The same number written another way ("06", "500.0") is no edit: a rewrite
          // restarts the program.
          if (/^-?\d{1,12}(\.\d{1,6})?$/.test(text) && Number(text) !== Number(c.value)) set({ ...c, value: text });
        }}
      />
      {toggles(v, c.reading) ? (
        <button
          type="button"
          class="unit unit-toggle"
          data-unit-toggle
          aria-label="Unit"
          title={c.reading.kind === 'percent' ? 'A percent of its cap: click for an amount' : 'An amount: click for a percent of its cap'}
          onClick={() => set({ ...c, reading: otherUnit(c.reading) })}
        >
          {c.reading.kind === 'percent' ? '%' : 'amt'}
        </button>
      ) : (
        unit(c.reading) && (
          <span class="unit" data-unit>
            {unit(c.reading)}
          </span>
        )
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
 * The caller (ProgramLines.tsx) gives what it shares with the logic lines: the row's
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
  /** Why the line isn't unlocked this run (the view's `locked`), if so. */
  locked: string | null;
  ranText: string;
  rowClass: string;
  drag: LiHTMLAttributes<HTMLLIElement>;
  indent: ComponentChildren;
  noted: Set<number>;
  /** Blocks mode: the run gutter, first in the row. */
  gutter?: ComponentChildren;
  /** Blocks mode: the repeat count is a number field, first (`4x sell cloud compute`). */
  setCount?: (n: number) => void;
  bump: (by: number) => void;
  remove: () => void;
}) {
  const { v, e, i, outcome, current, step, short, ranText, rowClass, drag, indent, noted, gutter, setCount, bump, remove } = props;
  const frac = current && step > 0 ? step / e.count : outcome ? 1 : 0;
  // A failed line says why (today's last run of it); the tooltip has it in full.
  // A conversion that ran part way says so too, as a starved building does.
  const why = outcome === 'failed' || outcome === 'ran' ? short : null;
  const command = v.commands.find((c) => c.id === e.command);
  // A command locked again (used in an earlier run): the view's reason, or, failing
  // that, a failure with no input short.
  const lockedWhy = props.locked;
  const locked = lockedWhy !== null || (outcome === 'failed' && !why && command?.unlocked === false);
  if (why || locked) noted.add(i);
  return (
    <li
      class={rowClass}
      data-line={i}
      data-command={e.command}
      {...drag}
      // The name can be cut short at narrow widths: the tooltip carries it whole.
      title={`${lineText(v, e)}: ${ranText}${why ? `. ${shortText(v, why)}` : locked ? `. Locked this run${lockedWhy ? `: ${lockedWhy}` : ''}.` : ''}`}
    >
      {gutter}
      <span class="grip" aria-hidden>
        ⠿
      </span>
      {setCount && (
        <label class="repeat-count" title="Runs of this line in a row (its repeat count)">
          <NumberField
            inputMode="numeric"
            aria-label="Repeat count"
            data-testid="repeat-count"
            value={String(e.count)}
            commit={(text) => {
              if (/^\d{1,9}$/.test(text) && Number(text) >= 1) setCount(Number(text));
            }}
          />
          x
        </label>
      )}
      {/* The current line is marked by its frame (style.css `.line.current`), never by
          text in the name, so no line's text moves as the pointer walks. */}
      {/* Cut short in a narrow right column ("Sell Cloud Compute (x…"): the tooltip has it. */}
      <span class="line-name" title={setCount ? name(v, e.command) : `${name(v, e.command)} (x${e.count})`}>
        {Object.keys(command?.produces ?? {}).map((res) => (
          <ResChip key={res} v={v} id={res} />
        ))}{' '}
        {setCount ? name(v, e.command) : `${name(v, e.command)} (x${e.count})`}
      </span>
      <Bar frac={frac} tone={outcome === 'failed' ? 'red' : outcome === 'skipped' ? 'gray' : 'green'} />
      {!setCount && (
        <>
          <button type="button" class="tiny" disabled={e.count <= 1} onClick={() => bump(-1)}>
            −
          </button>
          <button type="button" class="tiny" onClick={() => bump(1)}>
            +
          </button>
        </>
      )}
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
            `locked this run${lockedWhy ? `: ${lockedWhy}` : ''}`
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
      return (
        <span class="line-logic" data-holds={now ? String(now.holds) : undefined}>
          <b>If</b>
          <ConditionEdit v={v} when={e.when} now={now} set={(when) => set({ ...e, when })} />
        </span>
      );
    case 'else':
      return (
        <span class="line-logic">
          <b>Else</b>
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
              inputMode="numeric"
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
    case 'note':
      // A `#` comment: shown, never run; its text edited in place.
      return (
        <span class="line-logic line-note" data-note>
          #
          <NoteField text={e.text} commit={(text) => set({ ...e, text })} />
        </span>
      );
    case 'unparsed':
      // A text-editor line that doesn't parse, kept as typed until fixed there.
      return (
        <span class="line-logic line-unparsed" data-unparsed>
          <b>unparsed:</b> <code>{e.text}</code>
        </span>
      );
    case 'command':
      return null;
  }
}

/** A note's text: kept through the daily redraws, sent when left or on Enter. */
function NoteField({ text, commit }: { text: string; commit: (text: string) => void }) {
  const [draft, setDraft] = useState<string | null>(null);
  return (
    <input
      type="text"
      class="note-field"
      aria-label="Note"
      placeholder="a note"
      value={draft ?? text}
      onInput={(ev) => setDraft(ev.currentTarget.value)}
      onChange={(ev) => {
        setDraft(null);
        if (ev.currentTarget.value !== text) commit(ev.currentTarget.value);
      }}
    />
  );
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
        {/* A verb with nothing to name yet (Load with an empty library, Fund with no
            open project) would make a line the engine refuses: offered, not pickable. */}
        {v.verbs.map((x) => {
          const none = !!x.targets && x.targets.length === 0 && !x.target_all && x.verb !== e.verb;
          return (
            <option key={x.verb} value={x.verb} disabled={none}>
              {x.label}
              {none ? ' (nothing to name yet)' : ''}
            </option>
          );
        })}
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
            inputMode="text"
            size={4}
            aria-label="Amount"
            placeholder={spec.amount === 'optional' ? '1' : ''}
            title={spec.amount === 'up_to' ? 'A whole number from 1' : 'A whole number, or all'}
            value={amountText}
            commit={(raw) => {
              const text = raw.toLowerCase();
              const least = e.verb === 'assign_processors' || e.verb === 'fund' ? 0 : 1;
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

/** The buttons adding non-command lines, once they unlock (nothing before). */
export function AddLogic({ v, add, canElse }: { v: View; add: (e: Entry) => void; canElse: boolean }) {
  if (!v.program_logic) return null;
  const first = v.readings[0]?.reading;
  const when: Condition | null = first ? { first: { reading: first, cmp: 'gt', value: '0' }, rest: null } : null;
  const building = v.buildings.find((b) => b.visible)?.id;
  return (
    <div class="add-logic" data-testid="logic-add">
      <span class="muted">Add:</span>
      <button
        type="button"
        class="tiny"
        disabled={!when || !v.if_open}
        title={v.if_open ? 'The indented lines under it run only when it is true' : 'Opens with the Conditional Logic global project'}
        onClick={() => when && add({ op: 'if', when })}
      >
        If
      </button>
      <button
        type="button"
        class="tiny"
        data-testid="add-else"
        disabled={!v.if_open || !canElse}
        title={
          !v.if_open
            ? 'Opens with If'
            : canElse
              ? 'Its indented lines run when the if before it is false'
              : 'Follows an if (or its body) at the end of the program, one per if'
        }
        onClick={() => add({ op: 'else' })}
      >
        Else
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
