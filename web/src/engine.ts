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
  /** The next day's rate (before the cap's cut, unmet demand counted). */
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
  /** Daily boredom relief per working copy now (× its axis's modifier); 0 for none. */
  boredom_relief: Fx;
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
  /** What one run does to boredom now, signed (negative: it relieves), already × its axis. */
  boredom: Fx;
  /** The axis whose boredom tool it is, if any. */
  ideology: string | null;
  /** A per-target command folded into one verb (Buy, Promote, Fund, Overclock) and its target's label. */
  verb: string | null;
  target: string | null;
}

/** The four comparisons, written `<`, `>`, `<=`, `>=`. */
export type Cmp = 'lt' | 'gt' | 'le' | 'ge';

/** What a condition reads (hh-core `state::Reading`): one reading per thing. `amount`
 * for the uncapped resources (credits, science), `percent` for a capped resource's fill
 * or boredom's, or its `amount` (the editor's unit toggle). `energy_per_day` and
 * `days_since_purchase` open with research (`program_readings`). */
export type Reading =
  | { kind: 'amount' | 'percent'; resource: string }
  | { kind: 'free_processors' | 'free_land' | 'idle_pads' | 'energy_per_day' | 'days_since_purchase' };

/** `<reading> <cmp> <value>`: the right side is always a number. */
export interface Clause {
  reading: Reading;
  cmp: Cmp;
  value: Fx;
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
  /** Follows an `if` or its body: its indented lines run when that `if` was false. Free. */
  | { op: 'else' }
  | { op: 'act'; verb: Verb; target?: string | null; tab?: number | null; amount?: Amount | null }
  /** A `#` comment line from the text editor (text without the `#`): free, never runs. */
  | { op: 'note'; text: string }
  /** A text-editor line that doesn't parse, kept as typed until fixed: free, never runs. */
  | { op: 'unparsed'; text: string };

/** A program line: an instruction, `indented` in the body of the `if` or `else` above it. */
export type Entry = Op & { indented?: boolean };

export type LineOutcome = 'ran' | 'skipped' | 'failed' | 'true' | 'false' | 'jumped' | 'done';

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
  /** Per line, an `if` line's condition as it reads now. */
  conditions: (ConditionView | null)[];
  /** Per line, why it is not unlocked this run (it is kept, and skipped). */
  locked: (string | null)[];
  /** Per line, why an `unparsed` line doesn't parse. */
  errors: (string | null)[];
  /** Per repeated command line, its current or last pass (the run gutter's dots). */
  repeats: (Repeat | null)[];
  /** The tab's status light, judged on the last day played; null with no processors. */
  status: ProgramStatus | null;
}

/** A program tab's light: green every step ran in full, yellow some were short or
 * skipped, red nothing useful ran. `tip` is the hover, written by the engine. */
export interface ProgramStatus {
  light: 'green' | 'yellow' | 'red';
  steps: number;
  full: number;
  tip: string;
}

/** A repeated line's pass: runs done, failed, and bit k set when run k (of the first 8) failed. */
export interface Repeat {
  done: number;
  failed: number;
  first_fails: number;
  /** Runs skipped at their cap, and which of the first eight (bit k: run k); absent: none. */
  skipped?: number;
  first_skips?: number;
}

/** The program lines run over a span of days: per program, per line in queue order,
 * its outcome the last time it ran in the span (null: not run). */
export interface LinesRun {
  days: number;
  lines: (LineOutcome | null)[][];
  /** Per program, per line: the days an `if` line was true, as bits (bit 0: the span's
   * last day, bit k: k days before it; the span's last 32 days only). */
  if_true: number[][];
}

/** A text line's status from the engine's parse of the text editor's draft. */
export type TextLineStatus = 'ok' | 'amber' | 'red' | 'note' | 'blank';

/** `parse_program_text`: the draft's entries (a red line as `unparsed`, so they are always
 * a valid `set_program`) and one item per text line; `entry` indexes `entries`. */
export interface ParsedText {
  entries: Entry[];
  lines: { entry: number | null; status: TextLineStatus; reason: string | null }[];
}

/** One entry of the text editor's instruction panel (`program_instructions`, only what can
 * be used now): a line as text and the card a hover shows (`command:<id>`, `building:<id>`,
 * `research:<id>`, `project:<id>`). */
export interface Instruction {
  text: string;
  card: string | null;
}

export interface InstructionGroup {
  name: string;
  items: Instruction[];
}

export interface PadView {
  /** null: No cargo (Load skips the pad until a good is picked). */
  good: string | null;
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
  keep: Keep;
}

/** A card's scope icon (`ScopeIcon`): kept when the AI retires, kept through
 * everything, or null for run-only (the engine folds Eternal into Legacy before the reveal). */
export type Keep = 'legacy' | 'eternal' | null;

export interface AxisView {
  axis: string;
  score: Fx;
  rank: number;
  rank_score: Fx;
  next_rank_score: Fx;
  /** The axis's modifier m now (1.1^rank). */
  modifier: Fx;
}

export interface ProjectView {
  id: string;
  name: string;
  description: string;
  /** Where its progress and completions live: this run, this timeline, or forever. */
  scope: 'run' | 'timeline' | 'forever';
  keep: Keep;
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
  /** What each set rate actually gets: the last day's take where the stock fell short, else the rate (keys as `rates`). */
  actual_rates: Amounts;
  /** Days to the next completion at the actual rates; null while a cost still owed gets nothing. */
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
  /** Mechanics text (shown in the mechanics style, never in the narrative): the one-line
   * task and what completing it does. */
  goal: string;
  reward: string;
  /** Its story's panels (the engine splits `story` at `--` lines), one modal page each,
   * shown when this quest begins; a panel's paragraphs are split by a blank line. */
  story: string[];
  progress: ProgressView | null;
  sub_objectives: { id: string; label: string; done: boolean; progress: ProgressView | null }[];
  /** The design doc's beat id (`1.6`), if it carries one. */
  beat: string | null;
  /** The act it belongs to. */
  act: number | null;
  /** A later chain quest exists (false on the last). */
  more: boolean;
  /** Its picture's file stem in `src/assets/quests/` (`questPicture`), null when it has none. */
  picture: string | null;
}

export interface StoryView {
  id: string;
  title: string;
  /** A one-off event's one line (empty on a quest). */
  summary: string;
  /** A quest's mechanics text: its task and what completing it did (empty on an event). */
  goal: string;
  reward: string;
  /** A one-off event's mechanics text: what it does (empty on a quest). May hold `{rate}`:
   * fill it with `fillRate` (format.ts). */
  effect: string;
  /** A one-off event's text (empty on a quest: it has `story`). */
  body: string;
  /** A boredom-phase event's boredom per day, for `{rate}`; null for the rest. */
  rate?: Fx | null;
  /** A quest's (or a once one-off's) story panels, shown when it began. */
  story?: string[];
  /** A chain quest's payoff panels, shown when it completed (empty: no popup of its own;
   * its reward closes the next quest's story). */
  payoff?: string[];
  /** A main-story quest (false for a side quest; absent on event texts). */
  chain?: boolean;
  /** The design doc's beat id (`1.6`), on quests that carry one. */
  beat?: string | null;
  /** A chain quest's act (`content/acts.json`); null for the rest. */
  act?: number | null;
  /** A quest's picture's file stem (`questPicture`), null when it has none. */
  picture?: string | null;
  /** A one-off event a later one of its kind replaced in the Events list (Burnout and the
   * boredom phases show only the latest this run). */
  superseded?: boolean;
  /** The day a one-off event fired this run; null on a quest. */
  day?: number | null;
}

/** One project completion this run: its popup and its Events line. */
export interface ProjectDoneView {
  id: string;
  name: string;
  day: number;
  /** Which completion in its scope it was (1 for the first). */
  completion: number;
  /** What that completion cost in total. */
  costs: Amounts;
  /** What it gives, one line per reward (the engine's text). */
  reward_text: string[];
}

/** A side quest started (its trigger happened) or completed. */
export interface SideQuestView {
  id: string;
  beat: string | null;
  title: string;
  /** Mechanics text: the one-line task and what completing it does. */
  goal: string;
  reward: string;
  /** Its story's panels, one popup page each. */
  story: string[];
  /** The chain quest it branches off (its completion started it). */
  parent: string | null;
  done: boolean;
  progress: ProgressView | null;
  /** Its picture's file stem (`questPicture`), null when it has none. */
  picture: string | null;
  /** Its reward's scope icon; null when it has no reward of its own. */
  keep: Keep;
}

/** An act the story has reached. */
export interface ActView {
  act: number;
  name: string;
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
  /** Countdown days this timeline: from the discovery (1.6) in timeline 1, from the first day after (each retirement's gap counts too). */
  day: number;
  /** The countdown day the swarm arrives. */
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
  /** Days left to the swarm (null before the discovery, 1.6). */
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
  cheat_building: { owned: number; cost: Fx; rate: Fx; credits_rate: Fx; affordable: boolean };
  /** The later acts (draft). */
  saga: SagaView;
  resources: ResourceView[];
  buildings: BuildingView[];
  land_price: Fx;
  land_per_purchase: Fx;
  can_buy_land: boolean;
  commands: CommandView[];
  programs: ProgramView[];
  /** `build`, `buy land` and verb lines may be queued (a career first). */
  program_logic: boolean;
  /** `if` may be queued (program logic and the Conditional Logic project). */
  if_open: boolean;
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
  /** The goods a pad's cargo picker may offer now, in loading-priority order ("No cargo"
   * is always allowed besides). */
  pad_goods: string[];
  loading_priority: string[];
  markets: MarketView[];
  research: ResearchView[];
  ideology: AxisView[];
  projects: ProjectView[];
  quest: QuestView | null;
  completed_quests: string[];
  story: StoryView[];
  /** Side quests started or completed, in content order. */
  side_quests: SideQuestView[];
  /** The acts reached, in order. */
  acts: ActView[];
  event_texts: StoryView[];
  /** This run's project completions, oldest first (each pops up and lists in Events). */
  project_done: ProjectDoneView[];
  achievement_list: AchievementView[];
  rivals: { id: string; name: string; good: string; dumps_seen: number }[];
  /** Each adversary's Attention, threshold and presence (docs/rules/attention.md); `seen`: it has arrived at some point in the career. */
  adversaries: { id: string; attention: Fx; threshold: Fx; present: boolean; seen: boolean }[];
  overclocks: { target: string; bonus: Fx; until_day: number }[];
  modifiers: Record<string, Fx>;
  /** Every live project reward, as `Project: what it gives`. */
  live_rewards: string[];
  career: CareerView;
  next_run: NextRunView;
  last_run: LastRun | null;
  /** Why it ended, the run summary's line (content/messages.json). */
  last_run_why: string | null;
  run_stats: RunStats;
  boredom: {
    amount: Fx;
    cap: Fx;
    rate: Fx;
    phase: number;
    phases: number;
    /** Rate multiplier from the yearly escalation after the last phase (1 before it). */
    escalation: Fx;
    /** This run's boredom by source (`growth`, `command:<id>`, `building:<id>`, `cap`,
     * `other`), signed, summing to `amount`; largest first. */
    /** This run's boredom by source: [source key, label, amount], summing to the amount. */
    sources: [string, string, Fx][];
  };
  nav: { id: string; name: string; visible: boolean }[];
  /** The 50x and 200x speeds are open (Accelerated Scheduling, bought once a career). */
  fast_speeds: boolean;
  can_retire: boolean;
  /** The next day ends this run (boredom at its cap): the per-day rates read 0. */
  retires_next: boolean;
  activity: Activity[];
  breakdown: BreakdownView[];
  /** Days in the breakdown's rolling average. */
  flow_average_days: number;
  /** Days in a calendar year (dates read "Year 4, day 112"). */
  days_per_year: number;
  names: Record<string, string>;
}

/** `advance_run`'s answer; `record`: a recorder's new lines (none without one). */
export interface Advanced {
  state: string;
  lines: LinesRun;
  stopped: boolean;
  ran: number;
  record?: RecordLine[];
}

/** A play record line (`hh_sim::playrecord::Line`): `kind` and its fields. */
export type RecordLine = { kind: string } & Record<string, unknown>;

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
  /** `stopped`: it stopped early, after a tick that retired the run or changed the active
   * quest; `ran`: the ticks it ran. */
  advanceRun: (state: string, ticks: number): Advanced => JSON.parse(wasm.advance_run(state, ticks)),
  /** The play record a career's log keeps as it plays (`hh_sim::playrecord`): feed it
   * every state reached (its `advance_run` records each tick), log the lines it returns.
   * `resumed`: the career began before it. Free it when done. */
  recorder: (state: string, resumed: boolean): wasm.Recorder => new wasm.Recorder(state, resumed),
  /** A recorder saved with its `to_json`, resumed. */
  restoreRecorder: (json: string): wasm.Recorder => wasm.Recorder.restore(json),
  apply: (state: string, action: Action): string => wasm.apply(state, JSON.stringify(action)),
  view: (state: string): View => JSON.parse(wasm.view(state)),
  fingerprint: (state: string): string => wasm.fingerprint(state),
  replay: (record: unknown): Replayed => JSON.parse(wasm.replay(JSON.stringify(record))),
  /** A program's lines as the text editor's text (display names, 2-space indent, `# ` notes). */
  programText: (entries: Entry[]): string => wasm.program_text(JSON.stringify(entries)),
  /** Parses the text editor's text against the state (for what is unlocked this run).
   * Throws a string when the text can't be taken at all (too many lines). */
  parseProgramText: (state: string, text: string): ParsedText => JSON.parse(wasm.parse_program_text(state, text)),
  /** The text editor's instruction panel for the state's run; `cursor`: the caret's text
   * line (`else` is listed only where it fits below it). */
  programInstructions: (state: string, text: string, cursor: number): InstructionGroup[] =>
    JSON.parse(wasm.program_instructions(state, text, cursor)),
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
