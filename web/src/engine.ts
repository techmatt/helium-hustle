// Typed wrapper over hh-wasm (crates/hh-wasm/src/lib.rs). No rules live here: prices,
// rates, caps and what is affordable all come from the engine's view.
//
// A state goes back to the engine as the JSON string it produced. Quantities are `Fx`
// decimal strings ("12.500000"); `Number()` on one is for display only.

import init, * as wasm from './wasm/hh_wasm.js';

export type Fx = string;
export type Amounts = Record<string, Fx>;

export interface Action {
  type: string;
  [field: string]: unknown;
}

export interface Stamped {
  tick: number;
  action: Action;
}

export interface Scenario {
  name: string;
  description: string;
  start: { tick: number; resources: Record<string, Fx>; buildings: Record<string, number> };
}

export interface ResourceView {
  id: string;
  name: string;
  amount: Fx;
  cap: Fx | null;
  /** The next day's rate: the same quantity as a program's `rate` reading (before the
   * cap's cut, unmet demand counted). */
  per_day: Fx;
  /** The stock's actual change that day. */
  change_per_day: Fx;
  /** What the cap throws away that day (`per_day` reads this far past the change). */
  overflow: Fx;
  /** Demand left unmet that day (`per_day` reads this far below the change, the cap aside). */
  unmet: Fx;
  visible: boolean;
  /** The debug grant cheat may grant it. */
  grantable: boolean;
}

export interface BuildingView {
  id: string;
  name: string;
  description: string;
  category: string;
  ideology: string | null;
  owned: number;
  active: number;
  /** Active copies whose upkeep the last pass paid: the ones giving effects. */
  working: number;
  bonus: number;
  max_count: number | null;
  cost: Amounts;
  land: number;
  short: string[];
  produces: Amounts;
  upkeep: Amounts;
  stores: Amounts;
  grants: Amounts;
  affordable: boolean;
  /** It owns as many as its limit allows. */
  at_limit: boolean;
  unlocked: boolean;
  visible: boolean;
  stall: 'input_starved' | 'output_capped' | null;
  /** Ran below full capacity today: the input it was short of. */
  starved: ShortView | null;
  buy: Action;
}

/** Who ran short the last day, and of what. */
export interface ShortView {
  source: string;
  label: string;
  resource: string;
  need: Fx;
  had: Fx;
  /** The fraction it ran at (0: not at all). */
  ran: Fx;
}

/** One source's line in a resource's breakdown: the last day and the rolling average. */
export interface FlowRow {
  source: string;
  label: string;
  produced: Fx;
  consumed: Fx;
  avg_produced: Fx;
  avg_consumed: Fx;
}

export interface BreakdownView {
  resource: string;
  rows: FlowRow[];
  shorts: ShortView[];
}

export interface CommandView {
  id: string;
  name: string;
  description: string;
  unlocked: boolean;
  visible: boolean;
  costs: Amounts;
  produces: Amounts;
  boredom: Fx;
  /** A per-target command folded into one verb (Buy, Promote, Fund, Overclock) and its target's label. */
  verb: string | null;
  target: string | null;
}

export type Cmp = 'lt' | 'le' | 'gt' | 'ge' | 'eq';

/** What a condition reads (hh-core `state::Reading`). */
export type Reading =
  | { kind: 'amount' | 'percent' | 'rate'; resource: string }
  | { kind: 'count'; building: string }
  | { kind: 'day' | 'land_price' | 'days_since_launch' | 'days_since_purchase' | 'free_processors' }
  | { kind: 'pads_ready' | 'cargo' | 'pads' | 'idle_pads'; good?: string | null }
  | { kind: 'demand'; good: string }
  | { kind: 'next_cost' | 'running'; building: string }
  | { kind: 'researched'; item: string }
  | { kind: 'completed'; project: string }
  | { kind: 'processors'; tab: number }
  | { kind: 'days_to_cap'; resource: string }
  | { kind: 'every'; days: number };

/** `<reading> <cmp> <value>`, or with `against`, `<reading> <cmp> <against> × <value>`. */
export interface Clause {
  reading: Reading;
  cmp: Cmp;
  value: Fx;
  against?: Reading | null;
}

/** The verbs of an `act` line (hh-core `state::Verb`). */
export type Verb =
  | 'launch_pads'
  | 'load_pads'
  | 'set_pads'
  | 'pause'
  | 'resume'
  | 'research'
  | 'load_program'
  | 'assign_processors'
  | 'retire'
  | 'fund';

/** An `act` line's amount: N, or all. */
export type Amount = { n: number } | 'all';

/** A verb on offer (hh-core `view::VerbView`): its slots and their choices now. */
export interface VerbView {
  verb: Verb;
  label: string;
  /** Choices for the target slot (null: the verb takes none). */
  targets: { value: string; label: string }[] | null;
  /** The target may be left at all. */
  target_all: boolean;
  /** It takes a program tab. */
  tab: boolean;
  amount: 'none' | 'optional' | 'n_or_all' | 'up_to';
}

export interface Condition {
  first: Clause;
  rest: { join: 'and' | 'or'; clause: Clause } | null;
}

/** A program instruction (hh-core `state::Op`). */
export type Op =
  | { op: 'command'; command: string; count: number }
  | { op: 'build'; building: string; up_to: number | null }
  | { op: 'buy_land' }
  | { op: 'if'; when: Condition }
  | { op: 'wait_until'; when: Condition }
  | { op: 'act'; verb: Verb; target?: string | null; tab?: number | null; amount?: Amount | null };

/** A program line: an instruction, `indented` in the body of the `if` above it. */
export type Entry = Op & { indented?: boolean };

export type LineOutcome = 'ran' | 'skipped' | 'failed' | 'true' | 'false' | 'jumped' | 'waiting' | 'done';

export interface ConditionView {
  holds: boolean;
  clauses: { value: Fx | null; holds: boolean }[];
}

/** A reading a condition may use now, with its display label and value. */
export interface ReadingView {
  reading: Reading;
  label: string;
  value: Fx | null;
}

/** A saved program in the loadout library: a display name and its lines. */
export interface Loadout {
  name: string;
  entries: Entry[];
}

export interface ProgramView {
  entries: Entry[];
  /** Assigned processors. */
  processors: number;
  /** Of those, the ones running: fewer while Data Centers go unpaid. */
  running: number;
  entry: number;
  step: number;
  outcomes: (LineOutcome | null)[];
  /** Per line, the input it was short of the last time it ran today. */
  shorts: (ShortView | null)[];
  /** Per line, an `if` or `wait until` line's condition as it reads now. */
  conditions: (ConditionView | null)[];
}

/** The program lines run over a span of days: per program, per line in queue order,
 * its outcome the last time it ran in the span (null: not run). */
export interface LinesRun {
  days: number;
  lines: (LineOutcome | null)[][];
}

export interface PadView {
  good: string;
  paused: boolean;
  active: boolean;
  /** Switched on, but its Launch Pad's upkeep went unpaid: idle until paid. */
  unpaid: boolean;
  cargo: Fx;
  capacity: Fx;
  cooldown: number;
  fuel: Fx;
  load: Fx;
  payout: Fx;
  can_launch: boolean;
  /** The shared Propellant pool holds a launch's `fuel`. */
  fueled: boolean;
  /** Full and able to launch now. */
  ready: boolean;
  /** Cargo a switch to another good would return (the rest is lost over its cap). */
  switch_returns: Fx;
}

export interface MarketView {
  good: string;
  name: string;
  demand: Fx | null;
  tier: string;
  speculators: Fx | null;
}

export interface ResearchView {
  id: string;
  name: string;
  description: string;
  category: string;
  cost: Fx;
  requires: string | null;
  done: boolean;
  visible: boolean;
  affordable: boolean;
}

export interface AxisView {
  axis: string;
  score: Fx;
  rank: number;
  rank_score: Fx;
  next_rank_score: Fx;
}

export interface ProjectView {
  id: string;
  name: string;
  description: string;
  /** Where its progress and completions live: this run, this timeline, or forever. */
  scope: 'run' | 'timeline' | 'forever';
  /** Fundable now (its unlock holds and it is not done; a repeatable one is never done). */
  unlocked: boolean;
  /** Completed at least once in its scope. */
  completed: boolean;
  completions: number;
  repeatable: boolean;
  /** The next completion's costs. */
  costs: Amounts;
  progress: Amounts;
  rates: Amounts;
  /** The most each rate may be, per resource, per day. */
  max_rates: Amounts;
  /** Days to the next completion at today's rates; null while a cost still owed has no rate. */
  days_to_done: number | null;
  /** What opens it (the engine's text). */
  unlock_text: string;
  /** What it gives, one line per reward (the engine's text). */
  reward_text: string[];
}

export interface ProgressView {
  now: Fx;
  target: Fx;
}

export interface QuestView {
  id: string;
  /** The condition's type (`building_owned`, `quiet_days`, ...). */
  kind: string;
  title: string;
  summary: string;
  body: string;
  progress: ProgressView | null;
  sub_objectives: { id: string; label: string; done: boolean; progress: ProgressView | null }[];
}

export interface StoryView {
  id: string;
  title: string;
  summary: string;
  body: string;
}

export interface AchievementView {
  id: string;
  name: string;
  category: string;
  description: string;
  reward: string;
  earned: boolean;
}

export interface RunStats {
  credits_earned: Fx;
  shipments: number;
  buildings_built: number;
  research_completed: number;
  peak_energy: Fx;
  produced: Amounts;
}

export interface LastRun {
  run: number;
  days: number;
  forced: boolean;
  stats: RunStats;
  ideology_ranks: Record<string, number>;
}

export interface Activity {
  day: number;
  kind: string;
  id: string;
  amount?: Fx;
  quantity?: Fx;
}

export interface CareerView {
  retirements: number;
  best_run_days: number;
  best_run_credits: Fx;
  best_run_shipments: number;
  peak_power: Fx;
  starting_credits: Fx;
  boredom_resilience: Fx;
  buy_power_mult: Fx;
  lifetime_shipments: number;
  lifetime_credits_earned: Fx;
  lifetime_days: number;
  lifetime_buildings_built: number;
  lifetime_research_completed: number;
  max_ideology_ranks: Record<string, number>;
}

export interface NextRunView {
  starting_credits: Fx;
  boredom_resilience: Fx;
  buy_power_mult: Fx;
  ideology: AxisView[];
}

/** A timeline (hh-core `state::Timeline`): the calendar and the swarm (draft, 042). */
export interface TimelineView {
  number: number;
  /** Calendar days since it began (retirements cost days too). */
  day: number;
  /** The calendar day the swarm arrives. */
  arrival: number;
  strength: Fx;
  science_made: Fx;
  super_timeline: boolean;
}

/** A past timeline (hh-core `state::TimelineEnd`). */
export interface TimelineEnd {
  number: number;
  days: number;
  science_made: Fx;
  strength: Fx;
  /** `fell`, `super` or `victory`. */
  outcome: string;
}

/** An Earth tech (hh-core `view::TechView`). */
export interface TechView {
  id: string;
  name: string;
  description: string;
  cost: Fx;
  /** `timeline`, `next_timeline` or `forever`. */
  scope: string;
  requires: string | null;
  held: boolean;
  /** Timeline ends it still survives (null: not held, or kept for good). */
  ends_left: number | null;
  /** Why it can't be bought now (null: buyable). */
  blocker: string | null;
}

/** The later acts (hh-core `view::SagaView`, draft): calendar, swarm, Earth techs, vault, ending. */
export interface SagaView {
  revealed: boolean;
  timeline: TimelineView | null;
  days_left: number | null;
  /** Earth science to spend. */
  science: Fx;
  ended: number;
  repelled: number;
  entangled: boolean;
  victory: boolean;
  techs: TechView[];
  history: TimelineEnd[];
  reset_plan_open: boolean;
  /** Program tab -> loadout slot. */
  reset_plan: Record<string, number>;
  vault_open: boolean;
  vault: Amounts;
  /** Tachyon on hand this run (what Deposit would move). */
  tachyon_here: Fx;
  deposit_blocker: string | null;
  super_tachyon: Fx;
  /** Why Begin Super Timeline can't apply now (null: it can). */
  super_blocker: string | null;
}

/** The later acts' actions (draft, crates/hh-core/src/action.rs). */
export type SagaAction =
  | { type: 'buy_earth_tech'; tech: string }
  | { type: 'set_reset_plan'; program: number; loadout: number | null }
  | { type: 'deposit'; resource: string }
  | { type: 'begin_super_timeline' };

/** hh-core's `View` (crates/hh-core/src/view.rs). */
export interface View {
  tick: number;
  day: number;
  run: number;
  cheated: boolean;
  /** An accelerator (boost or autopilot) was used. */
  accelerated: boolean;
  /** The debug boost's factors, if set. */
  boost: { production: Fx; research: Fx; boredom: Fx } | null;
  /** The debug cheat building, shown only while Cheats are on. */
  cheat_building: { owned: number; cost: Fx; rate: Fx; affordable: boolean };
  /** The later acts (draft). */
  saga: SagaView;
  resources: ResourceView[];
  buildings: BuildingView[];
  land_price: Fx;
  land_per_purchase: Fx;
  can_buy_land: boolean;
  commands: CommandView[];
  programs: ProgramView[];
  /** `if`, `wait until`, `build` and `buy land` may be queued (a career first). */
  program_logic: boolean;
  /** `if` may be queued (program logic and the Conditional Logic project). */
  if_open: boolean;
  /** `wait until` may be queued (program logic and the Full Cluster side quest). */
  wait_open: boolean;
  logic_unlock_processors: number;
  /** Most lines in one program (`max_entries`). */
  max_program_lines: number;
  /** The loadout library (career): saved programs, in slot order. */
  library: Loadout[];
  library_slots: number;
  loadout_name_max: number;
  readings: ReadingView[];
  /** The verbs an `act` line may use. */
  verbs: VerbView[];
  processors: number;
  processors_free: number;
  /** Assigned processors running today (fewer while Data Centers go unpaid). */
  processors_running: number;
  pads: PadView[];
  loading_priority: string[];
  markets: MarketView[];
  research: ResearchView[];
  ideology: AxisView[];
  projects: ProjectView[];
  quest: QuestView | null;
  completed_quests: string[];
  story: StoryView[];
  event_texts: StoryView[];
  achievement_list: AchievementView[];
  rivals: { id: string; name: string; good: string; dumps_seen: number }[];
  /** Each adversary's Attention, threshold and presence (docs/rules/attention.md). */
  adversaries: { id: string; attention: Fx; threshold: Fx; present: boolean }[];
  overclocks: { target: string; bonus: Fx; until_day: number }[];
  modifiers: Record<string, Fx>;
  /** Every live project reward, as `Project: what it gives`. */
  live_rewards: string[];
  career: CareerView;
  next_run: NextRunView;
  last_run: LastRun | null;
  run_stats: RunStats;
  boredom: { amount: Fx; cap: Fx; rate: Fx; phase: number; phases: number };
  nav: { id: string; name: string; visible: boolean }[];
  can_retire: boolean;
  /** The next day ends this run (boredom at its cap): the per-day rates read 0. */
  retires_next: boolean;
  activity: Activity[];
  breakdown: BreakdownView[];
  /** Days in the breakdown's rolling average. */
  flow_average_days: number;
  names: Record<string, string>;
}

export interface Replayed {
  fingerprint: string;
  warnings: string[];
}

export async function loadEngine(): Promise<void> {
  await init();
}

export const engine = {
  version: (): string => wasm.version(),
  rulesRevision: (): number => wasm.rules_revision(),
  resolveConfig: (config: string): { hash: string; config: unknown } => JSON.parse(wasm.resolve_config(config)),
  scenarios: (): Scenario[] => JSON.parse(wasm.scenarios()),
  newGame: (config: string, seed: string, scenario = ''): string => wasm.new_game(config, seed, scenario),
  /** Plays up to `ticks` days, stopping after a tick that retired the run; also returns
   * the program lines run over those ticks. */
  advanceRun: (state: string, ticks: number): { state: string; lines: LinesRun } =>
    JSON.parse(wasm.advance_run(state, ticks)),
  apply: (state: string, action: Action): string => wasm.apply(state, JSON.stringify(action)),
  view: (state: string): View => JSON.parse(wasm.view(state)),
  fingerprint: (state: string): string => wasm.fingerprint(state),
  replay: (record: unknown): Replayed => JSON.parse(wasm.replay(JSON.stringify(record))),
  /** The debug autopilot (an accelerator): a sim player plays the game on toward
   * `target`; `met`: step ids met earlier in the career. Free it when done. */
  autopilot: (state: string, target: AutopilotTarget, agent: string, boost: string, budget: number, met: string[]): wasm.Autopilot =>
    new wasm.Autopilot(state, target, agent, boost, budget, JSON.stringify(met)),
  autopilotInfo: (): { agent: string; boost: string; steps: { id: string; name: string }[] } =>
    JSON.parse(wasm.autopilot_info()),
};

export type AutopilotTarget = 'milestone' | 'quest' | 'end';

/** `hh_sim::autopilot::Status`. */
export interface AutopilotStatus {
  aim: string[];
  reached: boolean;
  gave_up: boolean;
  ticks_played: number;
  tick: number;
  run: number;
  hits: { id: string; name: string; tick: number; run: number }[];
  left: string[];
}
