// Typed wrapper over hh-wasm (crates/hh-wasm/src/lib.rs). No rules live here: prices,
// rates, caps and what is affordable all come from the engine's view.
//
// A state goes back to the engine as the JSON string it produced. Quantities are `Fx`
// decimal strings ("12.500000"); `Number()` on one is for display only.

import init, * as wasm from './wasm/hh_wasm.js';

export type Fx = string;

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
}

export interface BuildingView {
  id: string;
  name: string;
  owned: number;
  cost: Record<string, Fx>;
  affordable: boolean;
  buy: Action;
}

export interface View {
  tick: number;
  resources: ResourceView[];
  buildings: BuildingView[];
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
  apply: (state: string, action: Action): string => wasm.apply(state, JSON.stringify(action)),
  view: (state: string): View => JSON.parse(wasm.view(state)),
  fingerprint: (state: string): string => wasm.fingerprint(state),
  configHash: (state: string): string => wasm.config_hash(state),
  replay: (record: unknown): Replayed => JSON.parse(wasm.replay(JSON.stringify(record))),
};
