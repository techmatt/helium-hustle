// The page: left column, the active panel, the right column, the status bar, and the
// retirement summary over everything when a run ends.

import { useCallback, useEffect, useState } from 'preact/hooks';
import { SPEEDS, type Speed } from '../clock';
import { capitalize, mult, num, perDay, storyBody } from '../format';
import {
  continueAfterRetirement,
  dismissStory,
  noteOpened,
  setNotice,
  setSpeed,
  togglePause,
  useStore,
  type Store,
} from '../store';
import { Sky } from './art';
import { Buildings } from './Buildings';
import { Bar, RateCell, Section, navShown } from './common';
import { Chip } from './icons';
import { Left } from './Left';
import { Options } from './Options';
import {
  Adversaries,
  Commands,
  Ideologies,
  LaunchPads,
  Projects,
  Research,
  Retirement,
  Stats,
  Story,
} from './panels';
import { Right } from './Right';
import { Dyson, EarthLab, Ending, MetaController, Timeline, Vault } from './Saga';
import { TakeoverView, useTakeoversChanged, type Takeover } from './Takeover';
import { cortexMockTakeover } from './CortexMock';
import { textEditorTakeover } from './TextEditor';

/** The takeover views, in their left-column order. */
const TAKEOVERS: Takeover[] = [cortexMockTakeover];

/** Keys 1-5 pick the speeds after pause, in order. */
const KEY_SPEEDS: Record<string, Speed> = Object.fromEntries(SPEEDS.slice(1).map((sp, i) => [String(i + 1), sp]));

export function App() {
  const s = useStore();
  const [panel, setPanel] = useState('buildings');
  const [program, setProgram] = useState(0);
  const [takeoverId, setTakeoverId] = useState<string | null>(null);
  useTakeoversChanged();
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const t = e.target as HTMLElement;
      if (t && (t.tagName === 'INPUT' || t.tagName === 'SELECT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return;
      // Browser shortcuts (Ctrl+1 picks a tab) are not ours.
      if (e.ctrlKey || e.altKey || e.metaKey) return;
      if (e.key === ' ') {
        e.preventDefault();
        // A held space repeats: one press, one toggle.
        if (!e.repeat) togglePause();
      } else if (KEY_SPEEDS[e.key]) setSpeed(KEY_SPEEDS[e.key]);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);
  // A panel whose tile hides (a new run re-gates it) falls back to Buildings.
  const shown = navShown(s.view, panel) ? panel : 'buildings';
  const pickProgram = useCallback((i: number) => {
    noteOpened('program_tab', i);
    setProgram(i);
  }, []);
  // The text editor works on the program tab the right column shows; the program panel opens it.
  const textEditor = textEditorTakeover(program, pickProgram);
  // A takeover whose button hides closes with it.
  const takeover = [...TAKEOVERS, textEditor].find((t) => t.id === takeoverId && t.shown(s)) ?? null;
  const closeTakeover = useCallback(() => setTakeoverId(null), []);
  // The notice and story banner: over the middle column, or over a takeover while one
  // hides that column (a story or a failed save mustn't wait for it to close).
  const banners = (
    <>
      {s.notice && (
        <p id="notice" class="notice">
          {s.notice}{' '}
          <button type="button" class="tiny" onClick={() => setNotice(null)}>
            ×
          </button>
        </p>
      )}
      {s.stories.length > 0 && (
        <div class="story-banner" data-testid="story-banner" data-story={s.stories[0].id}>
          <div class="story-head">
            <strong>{s.stories[0].title}</strong>
            <span class="muted">{s.stories[0].summary}</span>
          </div>
          <blockquote>{storyBody(s.stories[0])}</blockquote>
          <button type="button" onClick={dismissStory}>
            {s.stories.length > 1 ? `Next (${s.stories.length - 1} more)` : 'OK'}
          </button>
        </div>
      )}
    </>
  );
  return (
    <div class={`app${takeover ? ' taking-over' : ''}`}>
      <Left
        s={s}
        panel={takeover ? null : shown}
        open={(id) => {
          noteOpened('panel', id);
          setPanel(id);
          // A panel picked from the nav replaces an open takeover.
          setTakeoverId(null);
        }}
        takeovers={TAKEOVERS}
        takeover={takeover?.id ?? null}
        toggleTakeover={(id) => setTakeoverId((cur) => (cur === id ? null : id))}
      />
      <main class="col-mid">
        <Sky />
        {!takeover && banners}
        <Panel id={shown} s={s} program={program} />
      </main>
      <Right s={s} program={program} setProgram={pickProgram} openTextEditor={() => setTakeoverId(textEditor.id)} />
      {takeover && <TakeoverView s={s} t={takeover} close={closeTakeover} banners={banners} />}
      <StatusBar s={s} />
      {s.summary && <Summary s={s} />}
      <Ending v={s.view} />
    </div>
  );
}

function Panel({ id, s, program }: { id: string; s: Store; program: number }) {
  switch (id) {
    case 'commands':
      return <Commands s={s} program={program} />;
    case 'launch_pads':
      return <LaunchPads s={s} />;
    case 'research':
      return <Research s={s} />;
    case 'projects':
      return <Projects s={s} />;
    case 'ideologies':
      return <Ideologies s={s} />;
    case 'adversaries':
      return <Adversaries s={s} />;
    case 'stats':
      return <Stats s={s} />;
    case 'story':
      return <Story s={s} />;
    case 'timeline':
      return <Timeline s={s} />;
    case 'earth_lab':
      return <EarthLab s={s} />;
    case 'meta':
      return <MetaController s={s} />;
    case 'vault':
      return <Vault s={s} />;
    case 'dyson':
      return <Dyson s={s} />;
    case 'retirement':
      return <Retirement s={s} />;
    case 'options':
      return <Options s={s} />;
    default:
      return <Buildings s={s} />;
  }
}

function StatusBar({ s }: { s: Store }) {
  const v = s.view;
  const energy = v.resources.find((r) => r.id === 'energy');
  const bored = v.resources.find((r) => r.id === 'boredom');
  return (
    <footer class="status">
      <span id="day" class="status-day">
        Day {num(v.day)}
      </span>
      <span id="run" class="status-run">
        Run {v.run}
      </span>
      {v.cheated && (
        <span class="cheated" title="A cheat was used in this game">
          CHEATED
        </span>
      )}
      <span class="status-label">Boredom</span>
      <Bar frac={Number(v.boredom.amount) / Number(v.boredom.cap)} tone="red" label={`${num(v.boredom.amount)} / ${num(v.boredom.cap)}`} />
      <span class="status-rate">
        {/* The day that retires the run has no rate to show: say what comes. */}
        {v.retires_next ? <span class="neg">cap tomorrow</span> : perDay(bored?.per_day ?? '0', 2)}
      </span>
      <span class="status-label">Energy</span>
      <Bar
        frac={Number(energy?.amount ?? 0) / Number(energy?.cap ?? 1)}
        tone="blue"
        label={`${num(energy?.amount ?? '0')} / ${num(energy?.cap ?? '0')}`}
      />
      <span class="status-rate" data-testid="energy-rate">
        {/* Capped, short or plain, as in Resources. */}
        {v.retires_next ? '—' : energy !== undefined ? <RateCell r={energy} /> : perDay('0')}
      </span>
    </footer>
  );
}

function Summary({ s }: { s: Store }) {
  const r = s.summary!;
  const v = s.view;
  return (
    <div class="modal-back">
      <div class="modal" role="dialog" aria-label="Run summary" data-testid="summary">
        <h2>
          Run {r.run} ended on day {num(r.days)}
        </h2>
        <p>{r.forced ? 'Boredom reached its cap: forced retirement.' : 'You retired.'}</p>
        <Section title="That run">
          <table class="grid-table kv">
            <tbody>
              <tr>
                <td>Credits earned</td>
                <td class="num">{num(r.stats.credits_earned)}</td>
              </tr>
              <tr>
                <td>Shipments</td>
                <td class="num">{r.stats.shipments}</td>
              </tr>
              <tr>
                <td>Buildings built</td>
                <td class="num">{r.stats.buildings_built}</td>
              </tr>
              <tr>
                <td>Research completed</td>
                <td class="num">{r.stats.research_completed}</td>
              </tr>
              <tr>
                <td>Peak energy production</td>
                <td class="num">{perDay(r.stats.peak_energy)}</td>
              </tr>
            </tbody>
          </table>
        </Section>
        <Section title={`Run ${v.run} starts with`}>
          <table class="grid-table kv">
            <tbody>
              <tr>
                <td>Starting credits</td>
                <td class="num">{num(v.career.starting_credits)}</td>
              </tr>
              <tr>
                <td>Boredom growth (resilience)</td>
                <td class="num">{mult(v.career.boredom_resilience)}</td>
              </tr>
              {Number(v.career.buy_power_mult) !== 1 && (
                <tr>
                  <td>Buy Power scaling</td>
                  <td class="num">{mult(v.career.buy_power_mult)}</td>
                </tr>
              )}
              {v.ideology
                .filter((a) => Number(a.score) > 0)
                .map((a) => (
                  <tr key={a.axis}>
                    <td>
                      <Chip id={a.axis} title={capitalize(a.axis)} /> {capitalize(a.axis)} carried
                    </td>
                    <td class="num">
                      score {num(a.score)} (rank {a.rank})
                    </td>
                  </tr>
                ))}
            </tbody>
          </table>
        </Section>
        <p class="muted small">
          What persists: quests, achievements, global projects, career bests, saved programs (Load, in
          each program tab), ideology (faded by the years away), adversaries and everything you have seen.
        </p>
        <button type="button" class="big" data-testid="continue" onClick={continueAfterRetirement}>
          Continue
        </button>
      </div>
    </div>
  );
}
