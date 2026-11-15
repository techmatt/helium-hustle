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
  per_day: Fx;
  visible: boolean;
}

export interface BuildingView {
  id: string;
  name: string;
  description: string;
  category: string;
  ideology: string | null;
  owned: number;
  active: number;
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
  buy: Action;
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
}

export interface Entry {
  command: string;
  count: number;
}

export type LineOutcome = 'ran' | 'skipped' | 'failed';

export interface ProgramView {
  entries: Entry[];
  processors: number;
  entry: number;
  step: number;
  outcomes: (LineOutcome | null)[];
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
  cargo: Fx;
  capacity: Fx;
  cooldown: number;
  fuel: Fx;
  load: Fx;
  payout: Fx;
  can_launch: boolean;
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
  tier: string;
  unlocked: boolean;
  completed: boolean;
  costs: Amounts;
  progress: Amounts;
  rates: Amounts;
  max_rate: Fx;
}

export interface ProgressView {
  now: Fx;
  target: Fx;
}

export interface QuestView {
  id: string;
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

export interface LogEntry {
  tick: number;
  run: number;
  day: number;
  kind: string;
  id: string;
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

/** hh-core's `View` (crates/hh-core/src/view.rs). */
export interface View {
  tick: number;
  day: number;
  run: number;
  cheated: boolean;
  resources: ResourceView[];
  buildings: BuildingView[];
  land_price: Fx;
  land_per_purchase: Fx;
  can_buy_land: boolean;
  commands: CommandView[];
  programs: ProgramView[];
  processors: number;
  processors_free: number;
  pads: PadView[];
  loading_priority: string[];
  markets: MarketView[];
  research: ResearchView[];
  ideology: AxisView[];
  projects: ProjectView[];
  quest: QuestView | null;
  completed_quests: string[];
  story: StoryView[];
  events: string[];
  event_texts: StoryView[];
  achievements: string[];
  achievement_list: AchievementView[];
  rivals: { id: string; name: string; good: string; dumps_seen: number }[];
  overclocks: { target: string; bonus: Fx; until_day: number }[];
  modifiers: Record<string, Fx>;
  career: CareerView;
  next_run: NextRunView;
  last_run: LastRun | null;
  run_stats: RunStats;
  boredom: { amount: Fx; cap: Fx; rate: Fx; phase: number; phases: number };
  panels: string[];
  nav: { id: string; name: string; visible: boolean }[];
  can_retire: boolean;
  log: LogEntry[];
  activity: Activity[];
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
  advance: (state: string, ticks: number): string => wasm.advance(state, ticks),
  /** Like `advance`, but stops after a tick that retired the run; also returns the
   * program lines run over those ticks. */
  advanceRun: (state: string, ticks: number): { state: string; lines: LinesRun } =>
    JSON.parse(wasm.advance_run(state, ticks)),
  apply: (state: string, action: Action): string => wasm.apply(state, JSON.stringify(action)),
  view: (state: string): View => JSON.parse(wasm.view(state)),
  fingerprint: (state: string): string => wasm.fingerprint(state),
  configHash: (state: string): string => wasm.config_hash(state),
  replay: (record: unknown): Replayed => JSON.parse(wasm.replay(JSON.stringify(record))),
};
