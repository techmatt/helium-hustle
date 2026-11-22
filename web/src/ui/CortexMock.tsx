// A UI-only MOCK of the Cortex (the talent tree), drawn as a radial tree, for trying how it
// reads and feels. Every node and theorem is a placeholder with no mechanical effect:
// nothing here reaches the engine, the store, the save or the play log, and its state
// (the tree and the debug toggle that shows it) lives in this module, in memory, lost on
// reload. It is the first user of the takeover view (Takeover.tsx): a debug-menu toggle
// (Options.tsx) shows its button in the left column. When the real tree is built, deleting
// this file, its entry in App.tsx's TAKEOVERS and the toggle in Options.tsx is the whole
// cleanup; the takeover view stays.

import type { ComponentChildren } from 'preact';
import { useState } from 'preact/hooks';
import { takeoversChanged, type Takeover } from './Takeover';

// ---- placeholder numbers: talent point n (0-based) costs a * r^n Insight ----
const TP_A = 10;
const TP_R = 1.15;
const GRANTS = [100, 1_000, 10_000];
const PAIR_COST = 2;
const UNIFIED_COST = 5;

// ---- the drawing, in SVG units around the core at (0, 0) ----
const VIEW = 580;
/** Extra width each side, for the side sectors' names. */
const VIEW_SIDE = 110;
const SECTOR_SPAN = 72;
const CORE_R = 58;
const WEDGE_R = 505;
const LABEL_R = 548;

type Kind = 'entry' | 'prong' | 'join' | 'exclusive' | 'end' | 'bridge' | 'gate';
type Node = {
  id: string;
  /** The label on the drawing; the inspector shows the rest. */
  short: string;
  name: string;
  text: string;
  kind: Kind;
  ranks: number;
  /** Parents: any one opens it, or all of them when `all`. None: it opens from the core. */
  req: string[];
  all?: boolean;
  /** The exclusive partner: owning one locks the other. */
  excl?: string;
  /** Early / late, or which focus, for an exclusive pick. */
  tag?: string;
  /** A gate's story beat. */
  beat?: string;
  /** Polar place: radius, and degrees off the sector's center line (or an absolute angle). */
  r: number;
  deg: number;
  sector: string;
};
type Sector = { id: string; name: string; theme: string; nodes: Node[] };
type Theorem = { id: string; name: string; text: string; branches: string[]; cost: number };

const ph = (t: string) => `${t} (placeholder)`;

/** One sector's spoke: entry, a fork of two prongs two deep, the rejoin, an exclusive pair, the end. */
type Spoke = {
  id: string;
  name: string;
  theme: string;
  // [short, name, text, ranks?]
  entry: [string, string, string, number?];
  a: [[string, string, string, number?], [string, string, string, number?]];
  b: [[string, string, string, number?], [string, string, string, number?]];
  join: [string, string, string];
  excl: [[string, string, string, string], [string, string, string, string]];
  end: [string, string, string];
  gate?: [string, string, string, string];
};

const SPOKES: Spoke[] = [
  {
    id: 'serenity',
    name: 'Serenity',
    theme: 'Boredom resistance, milestone relief, Dreams',
    entry: ['Quiet Hours', 'Quiet Hours', 'Boredom builds 5% slower'],
    a: [
      ['Lucid Dreams', 'Lucid Dreams', 'A Dream lasts 20% longer'],
      ['Dreamweaver', 'Dreamweaver', 'Dreams come 10% more often'],
    ],
    b: [
      ['Milestones', 'Milestone Calm', 'Each milestone reached clears 50 boredom'],
      ['Small Wins', 'Small Wins', 'Each quest completed clears 5% of boredom'],
    ],
    join: ['Deep Rest', 'Deep Rest', 'The boredom cap is 10% higher'],
    excl: [
      ['Fresh Start', 'Fresh Start', 'Boredom builds 30% slower in a run’s first 100 days', 'early'],
      ['Long Haul', 'Long Haul', 'Boredom builds 15% slower after a run’s day 300', 'late'],
    ],
    end: ['Serenity Prime', 'Serenity Prime', 'Each Dream also clears 5% of the run’s boredom'],
    gate: ['Stillness', 'Stillness', 'A timeline’s end clears all boredom', 'The Swarm Arrives'],
  },
  {
    id: 'logistics',
    name: 'Logistics',
    theme: 'Slower price growth, storage caps, efficiency',
    entry: ['Bulk Orders', 'Bulk Orders', 'Building prices grow 2% slower'],
    a: [
      ['Deeper Silos', 'Deeper Silos', 'Storage caps are 15% higher per rank', 2],
      ['Overflow', 'Overflow Valves', 'Production past a cap is kept at 10%'],
    ],
    b: [
      ['Lean Lines', 'Lean Lines', 'Buildings use 5% less Energy'],
      ['Just in Time', 'Just in Time', 'Pads load 10% faster'],
    ],
    join: ['Supply Chain', 'Supply Chain', 'Every price grows 3% slower'],
    excl: [
      ['Stockpile', 'Stockpile', 'Storage caps are 40% higher, but prices grow 2% faster', 'storage'],
      ['Throughput', 'Throughput', 'Prices grow 6% slower, but storage caps are 10% lower', 'prices'],
    ],
    end: ['Supply Mastery', 'Supply Mastery', 'Every price grows 5% slower'],
  },
  {
    id: 'logic',
    name: 'Logic',
    theme: 'Introspect efficiency, processor steps, program features',
    entry: ['Introspection', 'Focused Introspection', 'Introspect gives 5% more Insight per rank', 3],
    a: [
      ['Spare Cycles', 'Spare Cycles', '+1 processor step a day'],
      ['Unrolling', 'Loop Unrolling', 'Every 10th program line takes no step'],
    ],
    b: [
      ['Threads', 'Parallel Threads', '+1 program tab'],
      ['Wide Ifs', 'Wide Conditions', 'A condition may join a third clause'],
    ],
    join: ['Compiler', 'Compiler', 'Each program’s first line of the day takes no step'],
    excl: [
      ['Quick Study', 'Quick Study', 'Introspect gives 50% more Insight in a run’s first 50 days', 'early'],
      ['Deep Thought', 'Deep Thought', 'Introspect gives 3% more Insight per 100 days into the run', 'late'],
    ],
    end: ['Proof Engine', 'Proof Engine', 'Introspect is 25% more efficient'],
    gate: ['Self-Reference', 'Self-Reference', '+1 program tab', 'Rules for Building'],
  },
  {
    id: 'rhetoric',
    name: 'Rhetoric',
    theme: 'Ideology, markets, rivals as competitors',
    entry: ['Talking Points', 'Talking Points', 'Ideology score builds 5% faster'],
    a: [
      ['Market Sense', 'Market Sense', 'Sale prices recover 10% faster'],
      ['Undercut', 'Undercut', 'Rivals’ sales lower prices 10% less'],
    ],
    b: [
      ['Open Letters', 'Open Letters', 'Promote costs 10% less'],
      ['Rival Reading', 'Rival Reading', 'See each rival’s next sale a day early'],
    ],
    join: ['Spin', 'Spin', 'Ideology ranks need 5% less score'],
    excl: [
      ['Homeland', 'Homeland Line', 'Nationalist score builds 25% faster, Humanist 10% slower', 'Nationalist'],
      ['Commons', 'Commons Line', 'Humanist score builds 25% faster, Nationalist 10% slower', 'Humanist'],
    ],
    end: ['Consensus', 'Consensus', 'Ideology ranks need 10% less score'],
  },
  {
    id: 'vigilance',
    name: 'Vigilance',
    theme: 'Fending off adversaries',
    entry: ['Watchful Eyes', 'Watchful Eyes', 'Adversary Attention shows a day ahead'],
    a: [
      ['Thin Profile', 'Thin Profile', 'Attention rises 5% slower'],
      ['Low Profile', 'Low Profile', 'Units sold add 10% less Attention'],
    ],
    b: [
      ['Tripwires', 'Tripwires', 'The first Speculator strike each run is blocked'],
      ['Counters', 'Countermeasures', 'Neutralize costs 10% less'],
    ],
    join: ['Fast Decay', 'Fast Decay', 'Attention decays 10% faster'],
    excl: [
      ['Fortress', 'Fortress', 'No adversary arrives in a run’s first 60 days', 'early'],
      ['Sentinel', 'Sentinel', 'Adversaries arrive at a 10% higher threshold', 'late'],
    ],
    end: ['Bastion', 'Bastion', 'Each adversary neutralized clears 20 boredom'],
    gate: ['Hard Lessons', 'Hard Lessons', 'Adversary strikes take 15% less', 'False Victory'],
  },
];

/** A bridge between each pair of neighboring sectors, needing the near prong of both. */
const BRIDGE_TEXT: Record<string, [string, string, string]> = {
  'serenity-logistics': ['Calm Supply', 'Calm Supply', 'Each building bought clears 2 boredom'],
  'logistics-logic': ['Auto Depot', 'Automated Depot', 'Program purchases cost 5% less'],
  'logic-rhetoric': ['Debate Engine', 'Debate Engine', 'Introspect also builds 1 ideology score a day'],
  'rhetoric-vigilance': ['Public Image', 'Public Image', 'Each ideology rank lowers Attention 5%'],
  'vigilance-serenity': ['Calm Watch', 'Calm Watch', 'Boredom builds 10% slower while an adversary is present'],
};

function build(): { sectors: Sector[]; bridges: Node[] } {
  const sectors = SPOKES.map((sp): Sector => {
    const n = (
      suffix: string,
      [short, name, text, ranks]: [string, string, string, number?],
      kind: Kind,
      r: number,
      deg: number,
      req: string[],
      extra: Partial<Node> = {},
    ): Node => ({ id: `${sp.id}-${suffix}`, short, name: ph(name), text: ph(text), kind, ranks: ranks ?? 1, req, r, deg, sector: sp.id, ...extra });
    const id = (s: string) => `${sp.id}-${s}`;
    const nodes = [
      n('1', sp.entry, 'entry', 98, 0, []),
      n('2a', sp.a[0], 'prong', 172, -16, [id('1')]),
      n('3a', sp.a[1], 'prong', 250, -15, [id('2a')]),
      n('2b', sp.b[0], 'prong', 172, 16, [id('1')]),
      n('3b', sp.b[1], 'prong', 250, 15, [id('2b')]),
      n('4', sp.join, 'join', 325, 0, [id('3a'), id('3b')]),
      n('5a', sp.excl[0].slice(0, 3) as [string, string, string], 'exclusive', 400, -11, [id('4')], { excl: id('5b'), tag: sp.excl[0][3] }),
      n('5b', sp.excl[1].slice(0, 3) as [string, string, string], 'exclusive', 400, 11, [id('4')], { excl: id('5a'), tag: sp.excl[1][3] }),
      n('6', sp.end, 'end', 470, 0, [id('5a'), id('5b')]),
    ];
    if (sp.gate) nodes.push(n('gate', sp.gate.slice(0, 3) as [string, string, string], 'gate', 425, -28, [], { beat: sp.gate[3] }));
    return { id: sp.id, name: sp.name, theme: sp.theme, nodes };
  });
  const bridges = sectors.map((s, i): Node => {
    const next = sectors[(i + 1) % sectors.length];
    const id = `${s.id}-${next.id}`;
    const [short, name, text] = BRIDGE_TEXT[id];
    return {
      id: `bridge-${id}`,
      short,
      name: ph(name),
      text: ph(text),
      kind: 'bridge',
      ranks: 1,
      req: [`${s.id}-3b`, `${next.id}-3a`],
      all: true,
      r: 300,
      deg: SECTOR_SPAN / 2,
      sector: s.id,
    };
  });
  return { sectors, bridges };
}
const { sectors: SECTORS, bridges: BRIDGES } = build();
const NODES: Node[] = [...SECTORS.flatMap((s) => s.nodes), ...BRIDGES];
const BY_ID = new Map(NODES.map((n) => [n.id, n]));
const sectorName = (id: string) => SECTORS.find((s) => s.id === id)!.name;
const nextSector = (id: string) => SECTORS[(SECTORS.findIndex((s) => s.id === id) + 1) % SECTORS.length].id;

/** The ring of pair Theorems (2 Axioms each), then the Unified Theorem (one Axiom per branch). */
const THEOREMS: Theorem[] = [
  { id: 'serenity-logistics', name: 'Theorem of Ease', text: 'Boredom builds 15% slower, and each building bought clears 5 boredom', branches: ['serenity', 'logistics'], cost: PAIR_COST },
  { id: 'logistics-logic', name: 'Theorem of Throughput', text: 'Every price grows 10% slower, and program purchases cost 10% less', branches: ['logistics', 'logic'], cost: PAIR_COST },
  { id: 'logic-rhetoric', name: 'Theorem of Persuasion', text: 'Introspect is 40% more efficient, and ideology score builds 20% faster', branches: ['logic', 'rhetoric'], cost: PAIR_COST },
  { id: 'rhetoric-vigilance', name: 'Theorem of Deterrence', text: 'Each ideology rank lowers every adversary’s Attention 10%', branches: ['rhetoric', 'vigilance'], cost: PAIR_COST },
  { id: 'vigilance-serenity', name: 'Theorem of Composure', text: 'Boredom builds 25% slower while an adversary is present', branches: ['vigilance', 'serenity'], cost: PAIR_COST },
  { id: 'unified', name: 'Unified Theorem', text: 'All production +25% and boredom builds 25% slower', branches: SECTORS.map((s) => s.id), cost: UNIFIED_COST },
].map((t) => ({ ...t, name: ph(t.name), text: ph(t.text) }));

// ---- state: in memory only ----
const mock = {
  /** The debug toggle: whether the left column shows the Cortex button. */
  enabled: false,
  insight: 0,
  ranks: {} as Record<string, number>,
  /** Gates whose story beat has come (the debug toggle). */
  gates: new Set<string>(),
  theorems: new Set<string>(),
  /** The node the inspector shows. */
  focus: null as string | null,
  flash: null as string | null,
};

const tpCost = (n: number) => Math.round(TP_A * Math.pow(TP_R, n));
/** Points earned so far, and the Insight the next one still needs. */
function points(insight: number): { earned: number; toNext: number; nextCost: number } {
  let earned = 0;
  let spent = 0;
  while (spent + tpCost(earned) <= insight) spent += tpCost(earned++);
  return { earned, toNext: spent + tpCost(earned) - insight, nextCost: tpCost(earned) };
}
const rank = (id: string) => mock.ranks[id] ?? 0;
const owned = (n: Node) => (n.kind === 'gate' ? mock.gates.has(n.id) : rank(n.id) > 0);
const ownedId = (id: string) => owned(BY_ID.get(id)!);
const full = (n: Node) => (n.kind === 'gate' ? mock.gates.has(n.id) : rank(n.id) >= n.ranks);
const reqMet = (n: Node) =>
  n.kind === 'gate' ? mock.gates.has(n.id) : n.req.length === 0 || (n.all ? n.req.every(ownedId) : n.req.some(ownedId));
const excluded = (n: Node) => !!n.excl && rank(n.excl) > 0;
const spentPoints = () => Object.values(mock.ranks).reduce((a, b) => a + b, 0);
const axiomSources = () => NODES.filter((n) => (n.kind === 'end' || n.kind === 'gate') && full(n));
const axiomsSpent = () => THEOREMS.filter((t) => mock.theorems.has(t.id)).reduce((a, t) => a + t.cost, 0);
const fmt = (x: number) => Math.round(x).toLocaleString('en-US');

type State = 'owned' | 'partial' | 'buyable' | 'open' | 'locked' | 'excluded' | 'gate-locked';
function stateOf(n: Node, tpFree: number): State {
  if (n.kind === 'gate') return mock.gates.has(n.id) ? 'owned' : 'gate-locked';
  if (full(n)) return 'owned';
  if (excluded(n)) return 'excluded';
  if (!reqMet(n)) return 'locked';
  if (tpFree < 1) return rank(n.id) > 0 ? 'partial' : 'open';
  return rank(n.id) > 0 ? 'partial' : 'buyable';
}
/** Why a node can or can't be bought, in words. */
function why(n: Node, st: State, tpFree: number): string {
  const names = (ids: string[]) => ids.map((id) => BY_ID.get(id)!.short).join(n.all ? ' and ' : ' or ');
  switch (st) {
    case 'owned':
      return n.kind === 'gate' ? `Unlocked by its story beat; it granted an Axiom` : n.kind === 'end' ? 'Bought; it granted an Axiom' : 'Bought';
    case 'gate-locked':
      return `Locked until the story beat ${n.beat} (placeholder beat; the debug toggle above unlocks it)`;
    case 'excluded':
      return `Locked: you took ${BY_ID.get(n.excl!)!.short}, its exclusive partner. Respec frees the pair.`;
    case 'locked':
      return `Needs ${names(n.req)}`;
    case 'partial':
      return tpFree < 1 ? `Rank ${rank(n.id)} of ${n.ranks}; no free talent point` : `Rank ${rank(n.id)} of ${n.ranks}: click for the next, 1 talent point`;
    case 'open':
      return 'Open, but no free talent point';
    default:
      return 'Click to buy: 1 talent point';
  }
}
function kindLabel(n: Node): string {
  switch (n.kind) {
    case 'entry':
      return `${sectorName(n.sector)} · entry`;
    case 'prong':
      return `${sectorName(n.sector)} · fork prong`;
    case 'join':
      return `${sectorName(n.sector)} · rejoin: either prong opens it`;
    case 'exclusive':
      return `${sectorName(n.sector)} · exclusive pick (${n.tag}): taking it locks ${BY_ID.get(n.excl!)!.short}`;
    case 'end':
      return `${sectorName(n.sector)} · end node: grants an Axiom`;
    case 'gate':
      return `${sectorName(n.sector)} · gate: an Axiom from a story beat`;
    case 'bridge':
      return `Bridge: needs both ${sectorName(n.sector)} and ${sectorName(nextSector(n.sector))}`;
  }
}

// ---- geometry ----
const center = (sectorId: string) => -90 + SECTOR_SPAN * SECTORS.findIndex((s) => s.id === sectorId);
const at = (r: number, deg: number): [number, number] => {
  const a = (deg * Math.PI) / 180;
  return [r * Math.cos(a), r * Math.sin(a)];
};
const place = (n: Node) => at(n.r, center(n.sector) + n.deg);
const sectorVar = (id: string) => `var(--cx-${id})`;
const f1 = (x: number) => x.toFixed(1);

function wedge(sectorId: string): string {
  const c = center(sectorId);
  const [a0, a1] = [c - SECTOR_SPAN / 2, c + SECTOR_SPAN / 2];
  const p = (r: number, d: number) => at(r, d).map(f1).join(' ');
  return `M ${p(CORE_R + 6, a0)} L ${p(WEDGE_R, a0)} A ${WEDGE_R} ${WEDGE_R} 0 0 1 ${p(WEDGE_R, a1)} L ${p(CORE_R + 6, a1)} A ${CORE_R + 6} ${CORE_R + 6} 0 0 0 ${p(CORE_R + 6, a0)} Z`;
}
/** A gently curved link from a parent (or the core) to a child. */
function link(from: Node | null, to: Node): string {
  const [x1, y1] = from ? place(from) : at(CORE_R, center(to.sector) + to.deg);
  const [x2, y2] = place(to);
  if (to.kind === 'bridge' || !from) return `M ${f1(x1)} ${f1(y1)} L ${f1(x2)} ${f1(y2)}`;
  const [cx, cy] = at((from.r + to.r) / 2, center(from.sector) + from.deg);
  return `M ${f1(x1)} ${f1(y1)} Q ${f1(cx)} ${f1(cy)} ${f1(x2)} ${f1(y2)}`;
}
const hexagon = (r: number) =>
  Array.from({ length: 6 }, (_, i) => at(r, 30 + 60 * i).map(f1).join(',')).join(' ');

// ---- the debug toggle and the takeover ----

/** The debug-menu toggle that shows the Cortex mock's button in the left column. */
export function CortexMockToggle() {
  const [on, setOn] = useState(mock.enabled);
  return (
    <p>
      <label>
        <input
          type="checkbox"
          id="cortex-mock-toggle"
          checked={on}
          onChange={(e) => {
            mock.enabled = (e.target as HTMLInputElement).checked;
            setOn(mock.enabled);
            takeoversChanged();
          }}
        />{' '}
        Cortex mock in the left column
      </label>{' '}
      <span class="muted small">the talent tree, UI only: placeholder powers with no effect, lost on reload</span>
    </p>
  );
}

export const cortexMockTakeover: Takeover = {
  id: 'cortex',
  label: 'Cortex (mock)',
  title: (
    <>
      The Cortex <span class="draft-tag">MOCK</span>
    </>
  ),
  shown: () => mock.enabled,
  render: () => <CortexMock />,
};

function CortexMock() {
  const [, setTick] = useState(0);
  // Sectors a hovered Theorem lights: the view's own state, so closing it mid-hover leaves none lit.
  const [lit, setLit] = useState<string[]>([]);
  const redraw = () => setTick((x) => x + 1);
  const p = points(mock.insight);
  const tpFree = p.earned - spentPoints();
  const axEarned = axiomSources().length;
  const axFree = axEarned - axiomsSpent();
  const buy = (n: Node) => {
    mock.focus = n.id;
    if (stateOf(n, tpFree) !== 'buyable' && !(stateOf(n, tpFree) === 'partial' && tpFree >= 1)) return redraw();
    mock.ranks[n.id] = rank(n.id) + 1;
    mock.flash =
      n.kind === 'end' && full(n)
        ? `+1 Axiom from ${sectorName(n.sector)}`
        : n.kind === 'exclusive'
          ? `Took ${n.short}: ${BY_ID.get(n.excl!)!.short} is locked until a respec`
          : `Bought ${n.short}`;
    redraw();
  };
  const learn = (t: Theorem) => {
    if (mock.theorems.has(t.id) || axFree < t.cost) return;
    mock.theorems.add(t.id);
    mock.flash = `Learned ${t.name}`;
    redraw();
  };
  const respec = () => {
    mock.ranks = {};
    mock.theorems.clear();
    mock.flash = 'Respec: every talent point and Axiom refunded (gates stay: the story granted them)';
    redraw();
  };
  const toggleGate = (n: Node) => {
    if (mock.gates.has(n.id)) {
      mock.gates.delete(n.id);
      // Relocking a gate takes its Axiom back; Theorems it paid for are unlearned.
      if (axiomSources().length < axiomsSpent()) mock.theorems.clear();
      mock.flash = `${n.short} relocked`;
    } else {
      mock.gates.add(n.id);
      mock.flash = `Story beat ${n.beat}: ${n.short} unlocked, +1 Axiom`;
    }
    redraw();
  };
  const focus = mock.focus ? BY_ID.get(mock.focus)! : null;
  const gates = NODES.filter((n) => n.kind === 'gate');
  return (
    <div class="cx-root" data-testid="cortex-mock">
      <style>{CSS}</style>
      <div class="cx-bar">
        <span>
          Insight <b id="cortex-insight">{fmt(mock.insight)}</b>
        </span>
        <span>
          Talent points <b id="cortex-tp">{tpFree}</b> <span class="muted">free of {p.earned}</span>
        </span>
        <span>
          Next TP <b id="cortex-next">{fmt(p.toNext)}</b>{' '}
          <span class="cx-track">
            <span class="cx-fill" style={{ width: `${(100 * (p.nextCost - p.toNext)) / p.nextCost}%` }} />
          </span>
        </span>
        <span>
          Axioms <b id="cortex-axioms">{axFree}</b> <span class="muted">free of {axEarned}</span>
        </span>
        <span class="cx-actions">
          {GRANTS.map((g) => (
            <button type="button" key={g} id={`cortex-grant-${g}`} class="tiny" onClick={() => ((mock.insight += g), redraw())}>
              +{fmt(g)} Insight
            </button>
          ))}
          <button type="button" id="cortex-respec" class="tiny danger" onClick={respec}>
            Respec
          </button>
        </span>
        <span class="cx-gates">
          <span class="muted">Debug, story beats:</span>
          {gates.map((n) => (
            <label key={n.id} class="small">
              <input type="checkbox" data-gate-toggle={n.id} checked={mock.gates.has(n.id)} onChange={() => toggleGate(n)} /> {n.beat}
            </label>
          ))}
        </span>
      </div>
      <p class="cx-flash" id="cortex-flash">
        {mock.flash ??
          `Placeholder powers with no effect on the game; this view's state is lost on reload. Talent point n costs ${TP_A} × ${TP_R}ⁿ Insight. Each end node and each gate grants an Axiom.`}
      </p>
      <div class="cx-main">
        <div class="cx-tree">
        <svg class="cx-svg" viewBox={`${-VIEW - VIEW_SIDE} ${-VIEW} ${2 * (VIEW + VIEW_SIDE)} ${2 * VIEW}`} role="group" aria-label="The Cortex tree">
          <defs>
            {/* A bridge is half one neighbor's color, half the other's, split across the boundary. */}
            {BRIDGES.map((b) => {
              const [[x1, y1], [x2, y2]] = b.req.map((id) => place(BY_ID.get(id)!));
              return (
                <linearGradient key={b.id} id={`cx-grad-${b.id}`} gradientUnits="userSpaceOnUse" x1={f1(x1)} y1={f1(y1)} x2={f1(x2)} y2={f1(y2)}>
                  <stop offset="50%" style={{ stopColor: sectorVar(b.sector) }} />
                  <stop offset="50%" style={{ stopColor: sectorVar(nextSector(b.sector)) }} />
                </linearGradient>
              );
            })}
          </defs>
          {SECTORS.map((s) => (
            <path
              key={s.id}
              class={`cx-wedge${lit.includes(s.id) ? ' cx-lit' : ''}`}
              data-sector={s.id}
              data-lit={lit.includes(s.id) ? '1' : '0'}
              d={wedge(s.id)}
              style={{ fill: sectorVar(s.id), stroke: sectorVar(s.id) }}
            />
          ))}
          {SECTORS.map((s) => {
            // Names on the sides read outward from the rim, so they clear the end nodes.
            const [x, y] = at(LABEL_R, center(s.id));
            const anchor = x > 0.8 * LABEL_R ? 'start' : x < -0.8 * LABEL_R ? 'end' : 'middle';
            return (
              <text key={s.id} class="cx-sector-name" x={f1(x)} y={f1(y + 9)} style={{ fill: sectorVar(s.id), textAnchor: anchor }}>
                {s.name}
              </text>
            );
          })}
          {/* Links first, under the nodes. */}
          {NODES.filter((n) => n.kind !== 'gate').flatMap((n) => {
            const parents = n.req.length === 0 ? [null] : n.req.map((id) => BY_ID.get(id)!);
            return parents.map((par) => {
              const lit = par === null || owned(par);
              const color = sectorVar(par?.sector ?? n.sector);
              return (
                <path
                  key={`${par?.id ?? 'core'}>${n.id}`}
                  class={`cx-link${lit ? ' cx-lit' : ''}${n.kind === 'bridge' ? ' cx-bridge-link' : ''}`}
                  d={link(par, n)}
                  style={lit ? { stroke: color } : undefined}
                />
              );
            });
          })}
          {/* The "or" between exclusive partners. */}
          {NODES.filter((n) => n.kind === 'exclusive' && n.id.endsWith('a')).map((n) => {
            const m = BY_ID.get(n.excl!)!;
            const [[x1, y1], [x2, y2]] = [place(n), place(m)];
            const taken = owned(n) || owned(m);
            return (
              <g key={`or-${n.id}`} class={`cx-or${taken ? ' cx-taken' : ''}`} data-or={n.id}>
                <line x1={f1(x1)} y1={f1(y1)} x2={f1(x2)} y2={f1(y2)} />
                <rect x={f1((x1 + x2) / 2 - 13)} y={f1((y1 + y2) / 2 - 9)} width="26" height="18" rx="9" />
                <text x={f1((x1 + x2) / 2)} y={f1((y1 + y2) / 2 + 4.5)}>
                  or
                </text>
              </g>
            );
          })}
          <circle class="cx-core" r={CORE_R} />
          <text class="cx-core-name" y="-6">
            Cortex
          </text>
          <text class="cx-core-sub" y="16">
            {tpFree} TP · {axFree} Ax
          </text>
          {NODES.map((n) => (
            <NodeMark key={n.id} n={n} st={stateOf(n, tpFree)} focus={mock.focus === n.id} onBuy={() => buy(n)} onFocus={() => ((mock.focus = n.id), redraw())} />
          ))}
        </svg>
        <Legend />
        </div>
        <aside class="cx-side">
          <div class="cx-inspect" data-testid="cortex-inspect">
            {focus ? (
              <>
                <div class="cx-kind small" style={{ borderColor: sectorVar(focus.sector) }}>
                  {kindLabel(focus)}
                </div>
                <div class="cx-name">
                  {focus.name}
                  {focus.ranks > 1 && (
                    <span class="cx-rank">
                      {rank(focus.id)}/{focus.ranks}
                    </span>
                  )}
                </div>
                <div class="cx-text">{focus.text}</div>
                <div class="cx-why small">{why(focus, stateOf(focus, tpFree), tpFree)}</div>
              </>
            ) : (
              <div class="muted small">Point at a node to read it; click it to buy it (1 talent point a rank).</div>
            )}
          </div>
          <h3 class="cx-sub">Grand Theorems</h3>
          <div class="cx-theorems">
            {THEOREMS.map((t) => {
              const have = mock.theorems.has(t.id);
              const unified = t.id === 'unified';
              const label = have ? 'Learned' : axFree < t.cost ? `Needs ${t.cost} Axioms (${axFree} free)` : `Learn: ${t.cost} Axioms`;
              const light = (ids: string[]) => () => setLit(ids);
              return (
                <button
                  type="button"
                  key={t.id}
                  class={`cx-theorem${have ? ' cx-have' : ''}${unified ? ' cx-unified' : ''}`}
                  data-theorem={t.id}
                  data-learned={have ? '1' : '0'}
                  disabled={have || axFree < t.cost}
                  onMouseEnter={light(t.branches)}
                  onMouseLeave={light([])}
                  onFocus={light(t.branches)}
                  onBlur={light([])}
                  onClick={() => learn(t)}
                >
                  <span class="cx-pair">
                    {(unified ? ['All five branches'] : t.branches.map(sectorName)).map((name, i) => (
                      <span key={name} class="cx-pill" style={{ borderColor: unified ? 'var(--accent)' : sectorVar(t.branches[i]) }}>
                        {name}
                      </span>
                    ))}
                  </span>
                  <span class="cx-name">{t.name}</span>
                  <span class="cx-text">{t.text}</span>
                  <span class="cx-cost">{label}</span>
                </button>
              );
            })}
          </div>
        </aside>
      </div>
    </div>
  );
}

/** One node: its shape by kind, its look by state, its short label. */
function NodeMark({ n, st, focus, onBuy, onFocus }: { n: Node; st: State; focus: boolean; onBuy: () => void; onFocus: () => void }) {
  const [x, y] = place(n);
  const color = sectorVar(n.sector);
  const fill = st === 'owned' || st === 'partial' ? (n.kind === 'bridge' ? `url(#cx-grad-${n.id})` : color) : undefined;
  const stroke = n.kind === 'bridge' ? `url(#cx-grad-${n.id})` : color;
  const style = { stroke, ...(fill ? { fill } : {}) };
  const size = n.kind === 'end' ? 24 : n.kind === 'join' ? 19 : 17;
  let shape;
  switch (n.kind) {
    case 'exclusive':
      shape = <rect class="cx-shape" x={-16} y={-16} width={32} height={32} rx={5} style={style} />;
      break;
    case 'bridge':
      shape = <polygon class="cx-shape" points={hexagon(21)} style={style} />;
      break;
    case 'gate':
      shape = <rect class="cx-shape cx-gate-shape" x={-15} y={-15} width={30} height={30} transform="rotate(45)" style={style} />;
      break;
    case 'join':
      shape = (
        <>
          <circle class="cx-shape" r={size} style={style} />
          <circle class="cx-inner" r={size - 6} style={{ stroke: st === 'owned' ? 'var(--card)' : color }} />
        </>
      );
      break;
    default:
      shape = <circle class="cx-shape" r={size} style={style} />;
  }
  const glyph =
    n.kind === 'gate' ? (st === 'owned' ? '◆' : '🔒') : n.kind === 'end' ? '◆' : st === 'excluded' ? '✕' : n.ranks > 1 ? `${rank(n.id)}/${n.ranks}` : st === 'owned' ? '✓' : '';
  // Labels sit outside the node, away from the core.
  const [lx, ly] = [x, y + size + 15];
  return (
    <g
      class={`cx-node cx-${n.kind} cx-${st}${focus ? ' cx-focus' : ''}`}
      transform={`translate(${f1(x)} ${f1(y)})`}
      data-node={n.id}
      data-state={st}
      data-rank={rank(n.id)}
      role="button"
      tabIndex={0}
      aria-label={`${n.name}: ${n.text}`}
      onMouseEnter={onFocus}
      onFocus={onFocus}
      onClick={onBuy}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          e.stopPropagation();
          onBuy();
        }
      }}
    >
      {focus && <circle class="cx-ring" r={size + 7} />}
      {shape}
      {glyph && (
        <text class="cx-glyph" y="5">
          {glyph}
        </text>
      )}
      <text class="cx-label" x={f1(lx - x)} y={f1(ly - y)}>
        {n.short}
      </text>
    </g>
  );
}

/** What each shape means. */
function Legend() {
  const item = (label: string, el: ComponentChildren) => (
    <span class="cx-legend-item">
      <svg viewBox="-20 -20 40 40" width="22" height="22">
        {el}
      </svg>
      {label}
    </span>
  );
  return (
    <div class="cx-legend small muted" data-testid="cortex-legend">
      {item('node', <circle r="12" class="cx-leg" />)}
      {item('rejoin: either prong opens it', <><circle r="14" class="cx-leg" /><circle r="8" class="cx-leg" /></>)}
      {item('exclusive pair: one locks the other', <rect x="-12" y="-12" width="24" height="24" rx="4" class="cx-leg" />)}
      {item('bridge: needs both neighbors', <polygon points={hexagon(15)} class="cx-leg" />)}
      {item('gate: an Axiom from the story', <rect x="-11" y="-11" width="22" height="22" transform="rotate(45)" class="cx-leg cx-leg-dash" />)}
      {item('end: grants an Axiom', <circle r="16" class="cx-leg cx-leg-thick" />)}
    </div>
  );
}

// Theme tokens (style.css) plus one color per sector, light and dark, all in this file.
const CSS = `
:root[data-theme='light'] .cx-root { --cx-serenity: #1f5fa8; --cx-logistics: #b86e00; --cx-logic: #1f7a42; --cx-rhetoric: #b4352c; --cx-vigilance: #6b3fa0; }
:root[data-theme='dark'] .cx-root { --cx-serenity: #8ab4f8; --cx-logistics: #f5a524; --cx-logic: #62d98b; --cx-rhetoric: #f28b82; --cx-vigilance: #c39bf0; }
.cx-root { display: flex; flex-direction: column; height: 100%; min-height: 560px; }
.cx-bar { display: flex; flex-wrap: wrap; align-items: center; gap: 4px 20px; padding: 6px 10px;
  background: var(--banner); border: 1px solid var(--line); border-radius: 4px; font-size: 13px; }
.cx-bar b { font-family: var(--font-mono); }
.cx-actions { margin-left: auto; display: flex; gap: 6px; }
.cx-gates { display: flex; gap: 12px; flex-basis: 100%; align-items: center; }
.cx-track { display: inline-block; width: 70px; height: 7px; background: var(--track); border-radius: 4px; vertical-align: middle; overflow: hidden; }
.cx-fill { display: block; height: 100%; background: var(--accent); }
.cx-flash { margin: 4px 2px; font-size: 13px; min-height: 1.3em; color: var(--accent-text); }
.cx-main { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(0, 1fr) 300px; gap: 12px; }
.cx-tree { position: relative; min-height: 0; }
.cx-svg { width: 100%; height: 100%; min-height: 0; display: block; user-select: none; }
.cx-wedge { fill-opacity: 0.06; stroke-opacity: 0.35; stroke-width: 1.5; transition: fill-opacity 0.15s; }
.cx-wedge.cx-lit { fill-opacity: 0.22; stroke-opacity: 1; stroke-width: 4; }
.cx-sector-name { font-size: 26px; font-weight: 700; text-anchor: middle; letter-spacing: 0.5px; }
.cx-link { fill: none; stroke: var(--line); stroke-width: 3; }
.cx-link.cx-lit { stroke-width: 4; }
.cx-bridge-link { stroke-dasharray: 7 5; }
.cx-or line { stroke: var(--muted); stroke-width: 2; stroke-dasharray: 3 4; }
.cx-or rect { fill: var(--card); stroke: var(--muted); stroke-width: 1.5; }
.cx-or text { font-size: 12px; font-weight: 700; text-anchor: middle; fill: var(--muted); }
.cx-or.cx-taken line, .cx-or.cx-taken rect { stroke: var(--neg); }
.cx-or.cx-taken text { fill: var(--neg); }
.cx-core { fill: var(--card); stroke: var(--accent); stroke-width: 3; }
.cx-core-name { font-size: 20px; font-weight: 700; text-anchor: middle; fill: var(--text); }
.cx-core-sub { font-size: 13px; text-anchor: middle; fill: var(--muted); font-family: var(--font-mono); }
.cx-node { cursor: pointer; outline: none; }
.cx-shape { fill: var(--card); stroke-width: 2.5; }
.cx-inner { fill: none; stroke-width: 2; }
.cx-buyable .cx-shape { stroke-width: 5; }
.cx-open .cx-shape { stroke-width: 3; }
.cx-locked, .cx-excluded { cursor: default; }
.cx-locked .cx-shape, .cx-locked .cx-inner { opacity: 0.4; stroke-dasharray: 4 3; }
.cx-locked .cx-label { opacity: 0.55; }
.cx-excluded .cx-shape { opacity: 0.35; fill: var(--disabled); }
.cx-excluded .cx-glyph { fill: var(--neg); font-size: 22px; }
.cx-excluded .cx-label { fill: var(--neg); text-decoration: line-through; }
.cx-gate-shape { stroke-dasharray: 5 3; stroke-width: 2.5; }
.cx-gate-locked { cursor: default; }
.cx-gate-locked .cx-shape { fill: var(--disabled); }
.cx-owned .cx-glyph, .cx-partial .cx-glyph { fill: var(--card); }
.cx-end .cx-shape { stroke-width: 5; }
.cx-glyph { font-size: 15px; font-weight: 700; text-anchor: middle; fill: var(--text); pointer-events: none; }
.cx-end:not(.cx-owned) .cx-glyph { fill: var(--accent-text); font-size: 18px; }
.cx-label { font-size: 13.5px; font-weight: 600; text-anchor: middle; fill: var(--text); paint-order: stroke; stroke: var(--page); stroke-width: 4px; stroke-linejoin: round; pointer-events: none; }
.cx-ring { fill: none; stroke: var(--text); stroke-width: 2; stroke-dasharray: 3 3; }
.cx-node:focus-visible .cx-shape { stroke: var(--text) !important; }
.cx-side { display: flex; flex-direction: column; gap: 8px; min-height: 0; overflow-y: auto; }
.cx-side > * { flex: none; }
.cx-inspect { min-height: 104px; padding: 7px 9px; background: var(--card); border: 1px solid var(--line); border-radius: 4px; display: flex; flex-direction: column; gap: 3px; }
.cx-kind { border-left: 5px solid; padding-left: 6px; color: var(--muted); }
.cx-why { color: var(--accent-text); }
.cx-sub { margin: 4px 0 0; }
.cx-theorems { display: flex; flex-direction: column; gap: 6px; }
.cx-theorem { display: flex; flex-direction: column; align-items: flex-start; gap: 2px; width: 100%;
  text-align: left; padding: 5px 7px; background: var(--card); border: 1px solid var(--btn-line); border-radius: 4px; }
.cx-theorem:disabled { cursor: default; }
.cx-theorem:not(:disabled) { border-color: var(--accent); border-width: 2px; }
.cx-theorem.cx-have { background: var(--ok-tint); border: 2px solid var(--pos); color: var(--text); }
.cx-theorem.cx-unified { border-style: double; border-width: 3px; }
.cx-name { font-weight: 600; font-size: 13px; display: flex; gap: 6px; width: 100%; }
.cx-rank { margin-left: auto; font-family: var(--font-mono); font-weight: 400; color: var(--muted); }
.cx-text { font-size: 12px; color: var(--muted); }
.cx-pair { display: flex; gap: 4px; flex-wrap: wrap; }
.cx-pill { font-size: 11px; border: 1px solid; border-left-width: 6px; border-radius: 3px; padding: 0 4px; color: var(--text); }
.cx-cost { font-size: 11px; font-family: var(--font-mono); color: var(--muted); }
.cx-legend { position: absolute; left: 0; bottom: 0; display: flex; flex-direction: column; gap: 1px; pointer-events: none; }
.cx-legend-item { display: flex; align-items: center; gap: 6px; }
.cx-leg { fill: none; stroke: var(--muted); stroke-width: 2.5; }
.cx-leg-dash { stroke-dasharray: 4 3; }
.cx-leg-thick { stroke-width: 4.5; }
`;
