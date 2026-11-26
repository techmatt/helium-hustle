// The page: left column, the active panel, the right column, the status bar, and the
// retirement summary over everything when a run ends.

import { useCallback, useEffect, useRef, useState } from 'preact/hooks';
import { SPEEDS, type Speed } from '../clock';
import { capitalize, delta, mult, num, perDay } from '../format';
import {
  continueAfterRetirement,
  dismissStory,
  justMoved,
  nextStoryPage,
  noteOpened,
  setNotice,
  setSpeed,
  togglePause,
  useStore,
  type Store,
} from '../store';
import { QuestPicture, Sky } from './art';
import { Buildings } from './Buildings';
import { Bar, BoredomBar, Mechanics, RateCell, Section, isCapped, navShown } from './common';
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
} from './panels';
import { Right } from './Right';
import { Achievements, Story } from './Story';
import { Dyson, EarthLab, Ending, MetaController, Timeline, Vault } from './Saga';
import { TakeoverView, useTakeoversChanged, type Takeover } from './Takeover';
import { cortexMockTakeover } from './CortexMock';
import { programsTakeover } from './Programs';

/** The takeover views, in their left-column order. */
const TAKEOVERS: Takeover[] = [cortexMockTakeover];

/** Keys 1-5 pick the speeds after pause, in order. */
const KEY_SPEEDS: Record<string, Speed> = Object.fromEntries(SPEEDS.slice(1).map((sp, i) => [String(i + 1), sp]));

export function App() {
  const live = useStore();
  // Behind the run summary the screen keeps the ended run (its last day) until Continue.
  const s: Store = live.summary && live.backdrop ? { ...live, view: live.backdrop } : live;
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
  // The Programs view works on the program tab the right column shows; its nav tile opens it.
  const programs = programsTakeover(program, pickProgram);
  // A takeover whose button hides closes with it.
  const takeover = [...TAKEOVERS, programs].find((t) => t.id === takeoverId && t.shown(s)) ?? null;
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
            {s.stories[0].summary && <span class="muted">{s.stories[0].summary}</span>}
          </div>
          <div class="story-banner-body">
            <QuestPicture stem={s.stories[0].picture} at="banner" />
            <blockquote>{s.stories[0].body}</blockquote>
          </div>
          <Mechanics goal={s.stories[0].goal} reward={s.stories[0].reward} done />
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
        panel={takeover ? (takeover.id === programs.id ? programs.id : null) : shown}
        open={(id) => {
          noteOpened('panel', id);
          // The Programs tile opens its takeover (again: closes it); the middle panel stays.
          if (id === programs.id) {
            setTakeoverId((cur) => (cur === id ? null : id));
            return;
          }
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
      <Right s={s} program={program} setProgram={pickProgram} />
      {takeover && <TakeoverView s={s} t={takeover} close={closeTakeover} banners={banners} />}
      <StatusBar s={s} />
      {live.summary && <Summary s={live} />}
      {/* Never two modals at once: a story chain that came with a retirement waits for
          the summary's Continue (the clock stays stopped throughout). */}
      {!s.summary && s.storyPages.length > 0 && <StoryModal s={s} />}
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
    case 'achievements':
      return <Achievements s={s} />;
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
      <span class="status-label">Boredom</span>
      <BoredomBar v={v} />
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
        {/* Short or plain: Energy at its cap is spare power, not news, so no "capped" here
            (Resources still marks it). */}
        {v.retires_next ? '—' : energy !== undefined && !isCapped(energy) ? <RateCell r={energy} /> : perDay(energy?.per_day ?? '0')}
      </span>
    </footer>
  );
}

/** A main-story transition: one paragraph a page, under the quest's picture when it has
 * one, the next quest's task on the last;
 * the clock is stopped until its last button. */
function StoryModal({ s }: { s: Store }) {
  const p = s.storyPages[0];
  const last = s.storyPages.length === 1;
  // Each page takes the keyboard (Enter reads on): `autoFocus` only works on a page
  // load, so the first modal of a visit had it and later ones didn't.
  const button = useRef<HTMLButtonElement>(null);
  useEffect(() => button.current?.focus(), [p]);
  // Enter reads on wherever the focus went since (a click behind the backdrop, a remount
  // taking it): the modal holds the game, so the key is its.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Enter' || e.ctrlKey || e.altKey || e.metaKey || document.activeElement === button.current) return;
      e.preventDefault();
      if (!e.repeat && !justMoved()) nextStoryPage();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);
  return (
    <div class="modal-back">
      <div
        class="modal story-modal"
        role="dialog"
        aria-label={p.title}
        data-testid="story-modal"
        data-quest={p.quest}
        data-kind={p.kind}
      >
        <h2>
          {p.title}
          {p.kind === 'payoff' && <span class="muted story-done"> complete</span>}
        </h2>
        <QuestPicture stem={p.picture} at="modal" />
        <p class="story-text">{p.text}</p>
        {(p.reward || p.effect) && <Mechanics reward={p.reward} effect={p.effect} />}
        {p.task && (
          <div class="story-task">
            <Mechanics goal={p.task} hints={p.hints} />
          </div>
        )}
        <button
          type="button"
          class="big"
          data-testid="story-next"
          ref={button}
          // A page is read before it goes: the second click of a double-click, a click
          // just as the modal came up, a held Enter's repeats or an Enter in the moment
          // after a page turned would each turn one unread (pointer clicks only; a click
          // with no pointer behind it, detail 0, was aimed).
          onClick={(e) => (e.detail > 0 && justMoved() ? undefined : nextStoryPage())}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && (e.repeat || justMoved())) e.preventDefault();
          }}
        >
          {last ? 'Begin' : 'Continue'}
        </button>
      </div>
    </div>
  );
}

/** A next-run value's change from what the ended run started with: " (+10)", green when
 * better, red when worse. The change is rounded to the row's `places` first (credits 0,
 * multipliers 2), and one that rounds to zero shows nothing, as does an unknown start. */
function Delta({ now, was, places, show = String, lowerIsBetter = false }: { now: string; was: string | undefined; places: number; show?: (d: number) => string; lowerIsBetter?: boolean }) {
  const d = delta(now, was, places);
  if (d === null) return null;
  const text = show(Math.abs(d));
  const better = lowerIsBetter ? d < 0 : d > 0;
  return (
    <span class={`summary-delta ${better ? 'pos' : 'neg'}`} data-testid="summary-delta">
      {' '}
      ({d > 0 ? '+' : '−'}
      {text})
    </span>
  );
}

function Summary({ s }: { s: Store }) {
  const r = s.summary!;
  const v = s.view;
  const base = s.summaryBase;
  return (
    <div class="modal-back">
      <div class="modal" role="dialog" aria-label="Run summary" data-testid="summary">
        <h2>
          Run {r.run} ended on day {num(r.days)}
        </h2>
        <p data-testid="summary-why">{v.last_run_why}</p>
        <Section title={`Run ${r.run}`}>
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
                <td class="num">
                  {num(v.career.starting_credits)}
                  <Delta now={v.career.starting_credits} was={base?.starting_credits} places={0} show={(d) => num(d)} />
                </td>
              </tr>
              <tr>
                <td>Boredom growth (resilience)</td>
                <td class="num">
                  {mult(v.career.boredom_resilience)}
                  <Delta now={v.career.boredom_resilience} was={base?.boredom_resilience} places={2} lowerIsBetter />
                </td>
              </tr>
              {Number(v.career.buy_power_mult) !== 1 && (
                <tr>
                  <td>Buy Power scaling</td>
                  <td class="num">
                    {mult(v.career.buy_power_mult)}
                    <Delta now={v.career.buy_power_mult} was={base?.buy_power_mult} places={2} />
                  </td>
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
          each program tab), {navShown(v, 'programs') ? 'Programming, ' : ''}ideology (faded by the years
          away), adversaries and everything you have seen.
        </p>
        <button type="button" class="big" data-testid="continue" onClick={continueAfterRetirement}>
          Continue
        </button>
      </div>
    </div>
  );
}
