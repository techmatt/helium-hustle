// Options: the career, keyboard help, and the debug menu (scenario starts, replay check,
// and the cheats, off by default). Cheats are engine actions, so replay stays exact.

import { useEffect, useState } from 'preact/hooks';
import { engine, type AutopilotTarget } from '../engine';
import { onTheme, setTheme, theme, type Theme } from '../theme';
import * as playlog from '../playlog';
import {
  act,
  checkReplay,
  fingerprint,
  jumpAhead,
  newCareer,
  newGame,
  pilotStatus,
  runAutopilot,
  setCheatsEnabled,
  setNotice,
  stopAutopilot,
  type Store,
} from '../store';
import { Section } from './common';
import { CortexMockToggle } from './CortexMock';

export function Options({ s }: { s: Store }) {
  const [confirm, setConfirm] = useState(false);
  return (
    <div class="panel" data-panel="options">
      <h2>Options</h2>
      <Section title="Game">
        <p>
          The game saves itself in this browser every second and when you leave. Closing the page pauses it: there is
          no offline progress.
        </p>
        <p class="muted small">Keys: space pauses or resumes, 1 to 5 pick 1x to 200x.</p>
        <ThemePick id="theme" />
        <p class="small" data-testid="playlog-status" data-career={playlog.currentCareer() ?? ''}>
          {playlog.status === 'off'
            ? 'Play log: not recording (no log server; play through npm run dev).'
            : playlog.status === 'failed'
            ? `Play log: NOT recording: the log server refused it (${playlog.lastError}).`
            : `Play log: recording this career to ${playlog.logDir || 'the dev server'}${playlog.currentCareer() ? ` as ${playlog.currentCareer()}.jsonl` : ''}.`}
        </p>
        {!confirm ? (
          <button type="button" onClick={() => setConfirm(true)}>
            Start a new career…
          </button>
        ) : (
          <div class="confirm">
            <span>Erase this career and start over?</span>
            <button
              type="button"
              class="danger"
              onClick={() => {
                setConfirm(false);
                newCareer();
              }}
            >
              Yes, start over
            </button>
            <button type="button" onClick={() => setConfirm(false)}>
              Cancel
            </button>
          </div>
        )}
      </Section>
      <Debug s={s} />
    </div>
  );
}

/** Light or dark, remembered in this browser. In Options and again in the debug menu. */
function ThemePick({ id }: { id: string }) {
  const [t, setT] = useState<Theme>(theme());
  useEffect(() => onTheme(() => setT(theme())), []);
  return (
    <p class="theme-pick" role="group" aria-label="Theme" id={id}>
      <span>Theme</span>
      {(['light', 'dark'] as const).map((x) => (
        <button type="button" key={x} data-theme-pick={x} aria-pressed={t === x} onClick={() => setTheme(x)}>
          {x === 'light' ? 'Light' : 'Dark'}
        </button>
      ))}
    </p>
  );
}

function Debug({ s }: { s: Store }) {
  const v = s.view;
  const scenarios = engine.scenarios();
  const [scenario, setScenario] = useState(scenarios[0]?.name ?? '');
  const [seed, setSeed] = useState('0');
  const [res, setRes] = useState('credits');
  const [amount, setAmount] = useState('1000');
  const [boredom, setBoredom] = useState('0');
  const [days, setDays] = useState('100');
  const start = (name: string | null) => {
    try {
      const sc = scenarios.find((x) => x.name === name) ?? null;
      newGame(seed.trim() || '0', sc, sc ? `debug: scenario ${sc.name}` : 'debug: new game');
      setNotice(sc ? `Started scenario ${sc.name}: ${sc.description}` : null);
    } catch (e) {
      setNotice(String(e));
    }
  };
  return (
    <Section title="Debug" open={false} testid="debug">
      <ThemePick id="debug-theme" />
      <p>
        <label>
          Scenario{' '}
          <select id="scenario" value={scenario} onChange={(e) => setScenario((e.target as HTMLSelectElement).value)}>
            {scenarios.map((x) => (
              <option key={x.name} value={x.name} title={x.description}>
                {x.name}
              </option>
            ))}
          </select>
        </label>{' '}
        <button type="button" id="start-scenario" onClick={() => start(scenario)}>
          Start scenario
        </button>
      </p>
      <p>
        <label>
          Seed <input id="seed" type="text" inputMode="numeric" size={22} value={seed} onInput={(e) => setSeed((e.target as HTMLInputElement).value)} />
        </label>{' '}
        <button type="button" id="new-game" onClick={() => start(null)}>
          New game
        </button>{' '}
        <button type="button" id="check-replay" onClick={checkReplay}>
          Check replay
        </button>
      </p>
      <CortexMockToggle />
      <p class="muted small" id="debug-info">
        engine {engine.version()} · rules revision {engine.rulesRevision()} · config {s.configHash} · seed {s.seed}
        {s.scenario ? ` · scenario ${s.scenario.name}` : ''} · {s.actionCount} actions · fingerprint {fingerprint()}
        {v.cheated ? ' · CHEATED' : ''}
      </p>
      <label>
        <input
          type="checkbox"
          id="cheats-toggle"
          checked={s.cheatsEnabled}
          onChange={(e) => setCheatsEnabled((e.target as HTMLInputElement).checked)}
        />{' '}
        Show cheats (a cheat marks the game as cheated for good)
      </label>
      {s.cheatsEnabled && (
        <div class="cheats" data-testid="cheats">
          <p>
            Grant{' '}
            <input id="cheat-amount" size={8} value={amount} onInput={(e) => setAmount((e.target as HTMLInputElement).value)} />{' '}
            <select id="cheat-resource" value={res} onChange={(e) => setRes((e.target as HTMLSelectElement).value)}>
              {v.resources
                .filter((r) => r.grantable)
                .map((r) => (
                  <option key={r.id} value={r.id}>
                    {r.name}
                  </option>
                ))}
            </select>{' '}
            <button
              type="button"
              id="cheat-grant"
              onClick={() => act({ type: 'cheat', cheat: { kind: 'grant', resource: res, amount: amount.trim() } })}
            >
              Grant
            </button>
          </p>
          <p>
            Set boredom to{' '}
            <input id="cheat-boredom" size={8} value={boredom} onInput={(e) => setBoredom((e.target as HTMLInputElement).value)} />{' '}
            <button type="button" id="cheat-set-boredom" onClick={() => act({ type: 'cheat', cheat: { kind: 'set_boredom', amount: boredom.trim() } })}>
              Set
            </button>
          </p>
          <Accelerators s={s} />
          <p>
            Jump ahead{' '}
            <input id="cheat-days" size={6} value={days} onInput={(e) => setDays((e.target as HTMLInputElement).value)} /> days{' '}
            <button
              type="button"
              id="cheat-jump"
              onClick={() => {
                const n = Math.floor(Number(days));
                if (n > 0 && n <= 1_000_000) jumpAhead(n);
                else setNotice('Jump: give a whole number of days from 1 to 1,000,000.');
              }}
            >
              Jump
            </button>
          </p>
        </div>
      )}
    </Section>
  );
}

/** The accelerators (docs/simulator.md, *Accelerators*): the boost, and the autopilot,
 * a sim player playing the game on honestly toward the next milestone, main-quest step
 * or the end. Either marks the career accelerated, so its play log is no human pace. */
function Accelerators({ s }: { s: Store }) {
  const v = s.view;
  const info = engine.autopilotInfo();
  const [boost, setBoost] = useState(info.boost);
  const [agent, setAgent] = useState(info.agent);
  const [budget, setBudget] = useState('500000');
  const go = (target: AutopilotTarget) => {
    const n = Math.floor(Number(budget));
    if (!(n > 0 && n <= 10_000_000)) return setNotice('Autopilot: give a budget of 1 to 10,000,000 days.');
    runAutopilot(target, agent, boost, n);
  };
  const setBoostOnly = () => {
    const [production, research, bored] = boost.split(',').map((x) => x.trim());
    act({ type: 'cheat', cheat: { kind: 'boost', production, research, boredom: bored ?? '' } });
  };
  const b = v.boost;
  return (
    <div class="accelerators" data-testid="accelerators">
      <p>
        Boost (production, research, boredom){' '}
        <input id="accel-boost" size={12} value={boost} onInput={(e) => setBoost((e.target as HTMLInputElement).value)} />{' '}
        <button type="button" id="accel-set-boost" onClick={setBoostOnly}>
          Set boost
        </button>{' '}
        <span class="muted small" id="accel-boost-now">
          {b ? `now ×${b.production}, ×${b.research}, boredom ×${b.boredom}` : 'off'}
          {v.accelerated ? ' · ACCELERATED' : ''}
        </span>
      </p>
      <p>
        Autopilot: player <input id="accel-agent" size={34} value={agent} onInput={(e) => setAgent((e.target as HTMLInputElement).value)} />{' '}
        budget <input id="accel-budget" size={8} value={budget} onInput={(e) => setBudget((e.target as HTMLInputElement).value)} /> days
      </p>
      <p>
        <button type="button" id="accel-milestone" disabled={pilotStatus.running} onClick={() => go('milestone')}>
          Next milestone
        </button>{' '}
        <button type="button" id="accel-quest" disabled={pilotStatus.running} onClick={() => go('quest')}>
          Next quest step
        </button>{' '}
        <button type="button" id="accel-end" disabled={pilotStatus.running} onClick={() => go('end')}>
          Everything
        </button>{' '}
        <button type="button" id="accel-stop" disabled={!pilotStatus.running} onClick={stopAutopilot}>
          Stop
        </button>
      </p>
      <p class="muted small" id="accel-status" data-running={pilotStatus.running ? '1' : '0'}>
        {pilotStatus.text ?? 'The autopilot plays with the boost above (empty: none); its days take no play time in the log.'}
      </p>
    </div>
  );
}
