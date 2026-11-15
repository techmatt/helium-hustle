// Wiring: the clock, the state panels, the buy buttons, autosave and the debug menu.
// Every number shown comes from the engine's view; this file computes no rule.

import baselineConfig from '../../configs/baseline.json?raw';
import { Clock, SPEEDS, type Speed } from './clock';
import { engine, loadEngine, type Scenario, type Stamped, type View } from './engine';
import { clearSave, loadSave, writeSave } from './save';

/** wasm's `advance` takes a u32 tick count; larger catch-ups go in chunks. */
const MAX_CHUNK = 1_000_000;
/** Autosave at most this often while ticking (and always on an action or leaving). */
const SAVE_EVERY_MS = 1000;

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

interface Game {
  state: string;
  seed: string;
  scenario: Scenario | null;
  actions: Stamped[];
}

let game: Game;
let configHash = '';
let lastSave = 0;
const clock = new Clock(runTicks);

function notice(text: string | null): void {
  const p = $('notice');
  p.hidden = text === null;
  p.textContent = text ?? '';
}

function fmt(fx: string): string {
  return Number(fx).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function newGame(seed: string, scenario: Scenario | null): void {
  game = { state: engine.newGame(baselineConfig, seed, scenario?.name ?? ''), seed, scenario, actions: [] };
  save();
  render();
}

function runTicks(ticks: number): void {
  while (ticks > 0) {
    const n = Math.min(ticks, MAX_CHUNK);
    game.state = engine.advance(game.state, n);
    ticks -= n;
  }
  if (performance.now() - lastSave > SAVE_EVERY_MS) save();
  render();
}

function save(): void {
  lastSave = performance.now();
  writeSave({
    revision: engine.rulesRevision(),
    configHash,
    state: game.state,
    seed: game.seed,
    scenario: game.scenario,
    actions: game.actions,
    speed: clock.speed,
  });
}

function record() {
  return {
    format: 'hh-record/1',
    rules_revision: engine.rulesRevision(),
    // The config the game plays under, as embedded in its state (not today's file,
    // whose name may differ under the same hash).
    config: JSON.parse(game.state).config,
    seed: game.seed,
    scenario: game.scenario,
    actions: game.actions,
    ticks: engine.view(game.state).tick,
    fingerprint: engine.fingerprint(game.state),
  };
}

function setSpeed(s: Speed): void {
  clock.setSpeed(s);
  save();
  render();
}

function renderSpeeds(): void {
  const box = $('speeds');
  box.replaceChildren(
    ...SPEEDS.map((s) => {
      const b = document.createElement('button');
      b.type = 'button';
      b.textContent = s === 0 ? 'Pause' : `${s}x`;
      b.dataset.speed = String(s);
      b.onclick = () => setSpeed(s);
      return b;
    }),
  );
}

function render(): void {
  const v: View = engine.view(game.state);
  $('day').textContent = `Day ${v.day.toLocaleString('en-US')}`;
  $('run').textContent = `Run ${v.run}`;
  $('quest').textContent = v.quest ? `${v.quest.title}: ${v.quest.summary}` : '';
  for (const b of $('speeds').querySelectorAll('button')) {
    b.setAttribute('aria-pressed', String(Number(b.dataset.speed) === clock.speed));
  }
  $('resources').replaceChildren(
    ...v.resources.filter((r) => r.visible).map((r) => {
      const tr = document.createElement('tr');
      const cap = r.cap === null ? '' : ` / ${fmt(r.cap)}`;
      tr.innerHTML = `<td></td><td></td><td class="rate"></td>`;
      const [name, amount, rate] = tr.children;
      name.textContent = r.name;
      amount.textContent = `${fmt(r.amount)}${cap}`;
      rate.textContent = `${Number(r.per_day) >= 0 ? '+' : ''}${fmt(r.per_day)} / day`;
      return tr;
    }),
  );
  $('buildings').replaceChildren(
    ...v.buildings.filter((b) => b.visible || b.owned > 0).map((b) => {
      const div = document.createElement('div');
      div.className = 'building';
      div.dataset.id = b.id;
      const btn = document.createElement('button');
      btn.type = 'button';
      const price = Object.entries(b.cost)
        .map(([res, amt]) => `${fmt(amt)} ${v.resources.find((r) => r.id === res)?.name ?? res}`)
        .join(', ');
      btn.textContent = `Buy ${b.name} (${price})`;
      btn.disabled = !b.affordable;
      btn.onclick = () => act(b.buy);
      const owned = document.createElement('span');
      owned.textContent = `${b.owned} owned`;
      div.append(btn, owned);
      return div;
    }),
  );
  // Program 1 runs one command, chosen here; the full program editor is a later item.
  const program = v.programs[0];
  const running = program?.entries[0]?.command;
  const name = (id: string | undefined) => v.commands.find((c) => c.id === id)?.name ?? id;
  $('program').textContent = running
    ? `Running ${name(running)} on ${program.processors} processor(s).`
    : `Idle: pick a command for ${program?.processors ?? 0} processor(s).`;
  $('commands').replaceChildren(
    ...v.commands.filter((c) => c.visible).map((c) => {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.textContent = c.name;
      btn.dataset.id = c.id;
      btn.disabled = !c.unlocked;
      btn.setAttribute('aria-pressed', String(c.id === running));
      btn.onclick = () => act({ type: 'set_program', program: 0, entries: [{ command: c.id, count: 1 }] });
      return btn;
    }),
  );
  $('debug-info').textContent =
    `engine ${engine.version()} · rules revision ${engine.rulesRevision()} · config ${configHash} · ` +
    `seed ${game.seed}${game.scenario ? ` · scenario ${game.scenario.name}` : ''} · ` +
    `${game.actions.length} actions · fingerprint ${engine.fingerprint(game.state)}`;
}

function act(action: Stamped['action']): void {
  clock.wake(); // settle the ticks owed so the action lands between the right ticks
  try {
    game.state = engine.apply(game.state, action);
    game.actions.push({ tick: engine.view(game.state).tick, action });
    notice(null);
  } catch (e) {
    notice(String(e));
  }
  save();
  render();
}

function setupDebug(): void {
  const list = engine.scenarios();
  const select = $<HTMLSelectElement>('scenario');
  select.replaceChildren(
    ...list.map((s) => {
      const o = document.createElement('option');
      o.value = s.name;
      o.textContent = s.name;
      o.title = s.description;
      return o;
    }),
  );
  const seed = () => $<HTMLInputElement>('seed').value.trim() || '0';
  $('start-scenario').onclick = () => {
    const s = list.find((x) => x.name === select.value) ?? null;
    try {
      newGame(seed(), s);
      notice(s ? `Started scenario ${s.name}: ${s.description}` : null);
    } catch (e) {
      notice(String(e));
    }
  };
  $('new-game').onclick = () => {
    try {
      newGame(seed(), null);
      notice(null);
    } catch (e) {
      notice(String(e));
    }
  };
  $('check-replay').onclick = () => {
    try {
      const r = engine.replay(record());
      const ok = r.fingerprint === engine.fingerprint(game.state) && r.warnings.length === 0;
      notice(ok ? `Replay ok: ${r.fingerprint}` : `Replay MISMATCH: ${r.fingerprint} ${r.warnings.join('; ')}`);
    } catch (e) {
      notice(`Replay failed: ${e}`);
    }
  };
}

async function main(): Promise<void> {
  await loadEngine();
  configHash = engine.resolveConfig(baselineConfig).hash;
  renderSpeeds();
  setupDebug();
  const loaded = loadSave(engine.rulesRevision(), configHash);
  if (loaded && 'save' in loaded) {
    try {
      const s = loaded.save;
      engine.view(s.state); // throws if the engine can't read it
      game = { state: s.state, seed: s.seed, scenario: s.scenario, actions: s.actions };
      clock.speed = SPEEDS.includes(s.speed) ? s.speed : 1;
    } catch (e) {
      clearSave();
      notice(`The saved game could not be loaded (${e}), so a new game started.`);
      newGame('0', null);
    }
  } else {
    if (loaded) notice(loaded.discarded);
    newGame('0', null);
  }
  render();
  clock.start();
  // Leaving the page pauses the game (no offline progress): save what was played.
  window.addEventListener('pagehide', () => {
    clock.wake();
    save();
  });
  // A page restored from the back/forward cache resumes where it left off.
  window.addEventListener('pageshow', (e) => {
    if (e.persisted) clock.forgetTimeAway();
  });
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'hidden') save();
  });
}

main().catch((e) => notice(`Engine failed to load: ${e}`));
