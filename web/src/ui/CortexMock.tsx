// A UI-only MOCK of the Cortex (the talent tree), for trying how it reads and feels. Every
// node and theorem is a placeholder with no mechanical effect: nothing here reaches the
// engine, the store, the save or the play log, and its state lives in this module, in
// memory, lost on reload. Mounted once, from the debug menu (Options.tsx). When the real
// tree is built, deleting this file and that mount is the whole cleanup.

import { useState } from 'preact/hooks';

// ---- placeholder numbers: talent point n (0-based) costs a * r^n Insight ----
const TP_A = 10;
const TP_R = 1.2;
const GRANTS = [100, 1_000, 10_000];
const PAIR_COST = 2;
const UNIFIED_COST = 5;

type Node = { id: string; name: string; text: string; ranks: number };
type Branch = { id: string; name: string; theme: string; color: string; nodes: Node[] };
type Theorem = { id: string; name: string; text: string; branches: string[]; cost: number };

const node = (id: string, name: string, text: string, ranks = 1): Node => ({
  id,
  name: `${name} (placeholder)`,
  text: `${text} (placeholder)`,
  ranks,
});

/** Five branches, about six nodes each; each needs the one above it, and the last grants an Axiom. */
const BRANCHES: Branch[] = [
  {
    id: 'serenity',
    name: 'Serenity',
    theme: 'Boredom resistance, milestone relief, Dreams',
    color: 'var(--info)',
    nodes: [
      node('serenity-1', 'Quiet Hours', 'Boredom builds 5% slower'),
      node('serenity-2', 'Steady Breathing', 'Boredom builds 3% slower per rank', 3),
      node('serenity-3', 'Milestone Calm', 'Each milestone reached clears 50 boredom'),
      node('serenity-4', 'Lucid Dreams', 'A Dream lasts 20% longer'),
      node('serenity-5', 'Deep Rest', 'The boredom cap is 10% higher'),
      node('serenity-6', 'Serenity Prime', 'Each Dream also clears 5% of the run’s boredom'),
    ],
  },
  {
    id: 'logistics',
    name: 'Logistics',
    theme: 'Slower price growth, storage caps, efficiency',
    color: 'var(--accent)',
    nodes: [
      node('logistics-1', 'Bulk Orders', 'Building prices grow 2% slower'),
      node('logistics-2', 'Deeper Silos', 'Storage caps are 15% higher'),
      node('logistics-3', 'Lean Lines', 'Buildings use 5% less Energy per rank', 2),
      node('logistics-4', 'Just in Time', 'Pads load 10% faster'),
      node('logistics-5', 'Overflow Valves', 'Production past a cap is kept at 10%'),
      node('logistics-6', 'Supply Mastery', 'Every price grows 5% slower'),
    ],
  },
  {
    id: 'logic',
    name: 'Logic',
    theme: 'Introspect efficiency, processor steps, program features',
    color: 'var(--pos)',
    nodes: [
      node('logic-1', 'Focused Introspection', 'Introspect gives 10% more Insight'),
      node('logic-2', 'Spare Cycles', '+1 processor step a day'),
      node('logic-3', 'Loop Unrolling', 'Every 10th program line takes no step'),
      node('logic-4', 'Parallel Threads', '+1 program tab per rank', 2),
      node('logic-5', 'Wide Conditions', 'A condition may join a third clause'),
      node('logic-6', 'Proof Engine', 'Introspect is 25% more efficient'),
    ],
  },
  {
    id: 'rhetoric',
    name: 'Rhetoric',
    theme: 'Ideology, markets, rivals as competitors',
    color: 'var(--neg)',
    nodes: [
      node('rhetoric-1', 'Talking Points', 'Ideology score builds 5% faster'),
      node('rhetoric-2', 'Market Sense', 'Sale prices recover 10% faster'),
      node('rhetoric-3', 'Open Letters', 'Promote costs 10% less'),
      node('rhetoric-4', 'Rival Reading', 'See each rival’s next sale a day early'),
      node('rhetoric-5', 'Undercut', 'Rivals’ sales lower prices 10% less'),
      node('rhetoric-6', 'Consensus', 'Ideology ranks need 5% less score'),
    ],
  },
  {
    id: 'vigilance',
    name: 'Vigilance',
    theme: 'Fending off adversaries',
    color: 'var(--text)',
    nodes: [
      node('vigilance-1', 'Watchful Eyes', 'Adversary Attention shows a day ahead'),
      node('vigilance-2', 'Thin Profile', 'Attention rises 5% slower'),
      node('vigilance-3', 'Tripwires', 'The first Speculator strike each run is blocked'),
      node('vigilance-4', 'Countermeasures', 'Neutralize costs 10% less'),
      node('vigilance-5', 'Fast Decay', 'Attention decays 10% faster'),
      node('vigilance-6', 'Sentinel', 'Adversaries arrive at a 10% higher threshold'),
    ],
  },
];

/** The ring of pair Theorems (2 Axioms each), then the Unified Theorem (one Axiom per branch). */
const THEOREMS: Theorem[] = [
  { id: 'serenity-logistics', name: 'Theorem of Ease', text: 'Boredom builds 15% slower, and each building bought clears 5 boredom', branches: ['serenity', 'logistics'], cost: PAIR_COST },
  { id: 'logistics-logic', name: 'Theorem of Throughput', text: 'Every price grows 10% slower, and program purchases cost 10% less', branches: ['logistics', 'logic'], cost: PAIR_COST },
  { id: 'logic-rhetoric', name: 'Theorem of Persuasion', text: 'Introspect is 40% more efficient, and ideology score builds 20% faster', branches: ['logic', 'rhetoric'], cost: PAIR_COST },
  { id: 'rhetoric-vigilance', name: 'Theorem of Deterrence', text: 'Each ideology rank lowers every adversary’s Attention 10%', branches: ['rhetoric', 'vigilance'], cost: PAIR_COST },
  { id: 'vigilance-serenity', name: 'Theorem of Composure', text: 'Boredom builds 25% slower while an adversary is present', branches: ['vigilance', 'serenity'], cost: PAIR_COST },
].map((t) => ({ ...t, name: `${t.name} (placeholder)`, text: `${t.text} (placeholder)` }));
const UNIFIED: Theorem = {
  id: 'unified',
  name: 'Unified Theorem (placeholder)',
  text: 'All production +25% and boredom builds 25% slower (placeholder)',
  branches: BRANCHES.map((b) => b.id),
  cost: UNIFIED_COST,
};

// ---- state: in memory only ----
const mock = {
  open: false,
  insight: 0,
  ranks: {} as Record<string, number>,
  theorems: new Set<string>(),
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
const full = (n: Node) => rank(n.id) >= n.ranks;
const spentPoints = () => Object.values(mock.ranks).reduce((a, b) => a + b, 0);
const axiomsEarned = () => BRANCHES.filter((b) => full(b.nodes[b.nodes.length - 1])).length;
const axiomsSpent = () => [...THEOREMS, UNIFIED].filter((t) => mock.theorems.has(t.id)).reduce((a, t) => a + t.cost, 0);
const fmt = (x: number) => Math.round(x).toLocaleString('en-US');

/** The debug-menu button, and the mock it opens. */
export function CortexMockButton() {
  const [, setTick] = useState(0);
  const redraw = () => setTick((x) => x + 1);
  const set = (f: () => void) => () => {
    f();
    redraw();
  };
  return (
    <>
      <p>
        <button type="button" id="cortex-open" onClick={set(() => (mock.open = true))}>
          Cortex mock…
        </button>{' '}
        <span class="muted small">the talent tree, UI only: placeholder powers with no effect, lost on reload</span>
      </p>
      {mock.open && <CortexMock redraw={redraw} />}
    </>
  );
}

function CortexMock({ redraw }: { redraw: () => void }) {
  const p = points(mock.insight);
  const tpFree = p.earned - spentPoints();
  const axEarned = axiomsEarned();
  const axFree = axEarned - axiomsSpent();
  const buy = (b: Branch, i: number) => {
    const n = b.nodes[i];
    if (tpFree < 1 || full(n) || (i > 0 && !full(b.nodes[i - 1]))) return;
    mock.ranks[n.id] = rank(n.id) + 1;
    mock.flash = i === b.nodes.length - 1 && full(n) ? `+1 Axiom from ${b.name}` : null;
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
    mock.flash = 'Respec: every point and Axiom refunded';
    redraw();
  };
  const theorem = (t: Theorem, wide = false) => {
    const have = mock.theorems.has(t.id);
    const why = have ? 'Learned' : axFree < t.cost ? `Needs ${t.cost} Axioms (${axFree} free)` : `Learn: ${t.cost} Axioms`;
    return (
      <button
        type="button"
        key={t.id}
        class={`cx-theorem${have ? ' cx-have' : ''}${wide ? ' cx-unified' : ''}`}
        data-theorem={t.id}
        data-learned={have ? '1' : '0'}
        disabled={have || axFree < t.cost}
        onClick={() => learn(t)}
      >
        <span class="cx-pair">
          {t.branches.map((id) => {
            const b = BRANCHES.find((x) => x.id === id)!;
            return (
              <span key={id} class="cx-pill" style={{ borderColor: b.color }}>
                {b.name}
              </span>
            );
          })}
        </span>
        <span class="cx-name">{t.name}</span>
        <span class="cx-text">{t.text}</span>
        <span class="cx-cost">{why}</span>
      </button>
    );
  };
  return (
    <div class="modal-back">
      <style>{CSS}</style>
      <div class="modal cx-modal" role="dialog" aria-label="Cortex mock" data-testid="cortex-mock">
        <div class="cx-head">
          <h2>
            The Cortex <span class="draft-tag">MOCK</span>
          </h2>
          <span class="muted small">
            Placeholder powers with no effect on the game; this view's state is lost on reload. In the real game the
            Cortex opens at beat 2.2.
          </span>
          <button type="button" id="cortex-close" onClick={() => ((mock.open = false), redraw())}>
            Close
          </button>
        </div>
        <div class="cx-bar">
          <span>
            Insight <b id="cortex-insight">{fmt(mock.insight)}</b>
          </span>
          <span>
            Talent points <b id="cortex-tp">{tpFree}</b> <span class="muted">free of {p.earned}</span>
          </span>
          <span>
            Insight to next TP <b id="cortex-next">{fmt(p.toNext)}</b>{' '}
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
        </div>
        <p class="cx-flash" id="cortex-flash">
          {mock.flash ?? `Talent point n costs ${TP_A} × ${TP_R}ⁿ Insight (placeholder). Each branch’s last node grants an Axiom.`}
        </p>
        <div class="cx-cols">
          {BRANCHES.map((b) => (
            <section key={b.id} class="cx-col" data-branch={b.id} style={{ borderTopColor: b.color }}>
              <h3>{b.name}</h3>
              <div class="cx-theme muted small">{b.theme}</div>
              {b.nodes.map((n, i) => {
                const r = rank(n.id);
                const open = i === 0 || full(b.nodes[i - 1]);
                const end = i === b.nodes.length - 1;
                const can = open && !full(n) && tpFree >= 1;
                return (
                  <div key={n.id} class="cx-step">
                    {i > 0 && <span class={`cx-link${open ? ' cx-lit' : ''}`} style={open ? { background: b.color } : undefined} />}
                    <button
                      type="button"
                      class={`cx-node${full(n) ? ' cx-have' : ''}${open ? '' : ' cx-locked'}${end ? ' cx-end' : ''}`}
                      style={full(n) || end ? { borderColor: b.color } : undefined}
                      data-node={n.id}
                      data-rank={r}
                      disabled={!can}
                      title={!open ? 'Needs the node above it' : full(n) ? 'Bought' : tpFree < 1 ? 'No free talent point' : 'Buy a rank: 1 talent point'}
                      onClick={() => buy(b, i)}
                    >
                      <span class="cx-name">
                        {n.name}
                        <span class="cx-rank">
                          {r}/{n.ranks}
                        </span>
                      </span>
                      <span class="cx-text">{n.text}</span>
                      {end && <span class="cx-axiom">{full(n) ? '✓ Axiom granted' : '+1 Axiom'}</span>}
                    </button>
                  </div>
                );
              })}
            </section>
          ))}
        </div>
        <h3 class="cx-sub">Grand Theorems</h3>
        <div class="cx-ring">{THEOREMS.map((t) => theorem(t))}</div>
        <div class="cx-unified-row">{theorem(UNIFIED, true)}</div>
      </div>
    </div>
  );
}

// Theme tokens only (style.css), so light and dark both work and nothing outside this file changes.
const CSS = `
.modal.cx-modal { width: min(1320px, 97vw); max-height: 96vh; padding: 14px 18px; }
.cx-head { display: flex; align-items: baseline; gap: 14px; }
.cx-head h2 { margin: 0; }
.cx-head > button { margin-left: auto; }
.cx-bar { display: flex; flex-wrap: wrap; align-items: center; gap: 6px 22px; margin-top: 8px; padding: 8px 10px;
  background: var(--banner); border: 1px solid var(--line); border-radius: 4px; }
.cx-bar b { font-family: var(--font-mono); }
.cx-actions { margin-left: auto; display: flex; gap: 6px; }
.cx-track { display: inline-block; width: 90px; height: 7px; background: var(--track); border-radius: 4px; vertical-align: middle; overflow: hidden; }
.cx-fill { display: block; height: 100%; background: var(--accent); }
.cx-flash { margin: 6px 2px; font-size: 13px; min-height: 1.3em; color: var(--accent-text); }
.cx-cols { display: grid; grid-template-columns: repeat(5, 1fr); gap: 10px; }
.cx-col { background: var(--panel); border: 1px solid var(--line); border-top: 4px solid; border-radius: 4px; padding: 6px 8px 8px; }
.cx-col h3 { margin: 0; }
.cx-theme { min-height: 2.6em; margin-bottom: 4px; }
.cx-step { display: flex; flex-direction: column; align-items: stretch; }
.cx-link { align-self: center; width: 3px; height: 10px; background: var(--line); }
.cx-node, .cx-theorem { display: flex; flex-direction: column; align-items: flex-start; gap: 2px; width: 100%;
  text-align: left; padding: 5px 7px; background: var(--card); border: 1px solid var(--btn-line); border-radius: 4px; }
.cx-node:disabled, .cx-theorem:disabled { cursor: default; }
.cx-node.cx-locked { opacity: 0.55; }
.cx-node:not(:disabled), .cx-theorem:not(:disabled) { border-color: var(--accent); }
.cx-node.cx-have, .cx-theorem.cx-have { background: var(--ok-tint); border-width: 2px; color: var(--text); }
.cx-theorem.cx-have { border-color: var(--pos); }
.cx-node.cx-end { border-width: 2px; border-style: dashed; }
.cx-node.cx-end.cx-have { border-style: solid; }
.cx-name { font-weight: 600; font-size: 13px; display: flex; gap: 6px; width: 100%; }
.cx-rank { margin-left: auto; font-family: var(--font-mono); font-weight: 400; color: var(--muted); }
.cx-text { font-size: 12px; color: var(--muted); }
.cx-axiom { font-size: 11px; font-weight: 700; color: var(--accent-text); }
.cx-sub { margin: 10px 0 4px; }
.cx-ring { display: grid; grid-template-columns: repeat(5, 1fr); gap: 10px; }
.cx-unified-row { display: flex; justify-content: center; margin-top: 8px; }
.cx-theorem.cx-unified { width: min(420px, 100%); align-items: center; text-align: center; }
.cx-pair { display: flex; gap: 4px; flex-wrap: wrap; }
.cx-pill { font-size: 11px; border: 1px solid; border-left-width: 6px; border-radius: 3px; padding: 0 4px; color: var(--text); }
.cx-cost { font-size: 11px; font-family: var(--font-mono); color: var(--muted); }
`;
