// Story and Achievements. Story is a vertical quest timeline: one node per main-story
// beat on a spine, oldest at the top, each act opening with its chapter band; finished
// beats lit (a click replays their story modals), the active beat glowing with its task
// and progress, one dim "???" below it, and side quests branching off the beat that
// started them. Achievements are badges grouped by category. Every text, act and
// progress comes from the engine's view (the same quest rows the story modals read).

import { useEffect, useRef } from 'preact/hooks';
import type { SideQuestView, StoryView, View } from '../engine';
import { capitalize, num } from '../format';
import { replayStory, type Store } from '../store';
import { Bar, Quest } from './common';

/** A finished quest's paragraphs: its opening (its body when it has none), then its payoff. */
function paragraphs(q: StoryView): string[] {
  const opening = q.story && q.story.length > 0 ? q.story : q.body ? [q.body] : [];
  return [...opening, ...(q.payoff ?? [])];
}

/** The side quests branching off one beat. */
function Sides({ v, parent }: { v: View; parent: string }) {
  const list = v.side_quests.filter((x) => x.parent === parent);
  if (list.length === 0) return null;
  return (
    <div class="side-branches">
      {list.map((x) => (
        <Side key={x.id} x={x} />
      ))}
    </div>
  );
}

function Side({ x }: { x: SideQuestView }) {
  return (
    <div class={`side-node ${x.done ? 'is-done' : 'is-open'}`} data-side={x.id} data-testid="story-side">
      <span class="side-dot" aria-hidden="true" />
      <div class="side-card">
        <div class="side-title">
          {x.title}
          {x.done && <span class="ok"> ✓</span>}
        </div>
        {x.done ? (
          <p class="small">{x.body}</p>
        ) : (
          <>
            <div class="small">{x.summary}</div>
            {x.progress && <Bar frac={Number(x.progress.now) / Number(x.progress.target)} label={`${num(x.progress.now)} / ${num(x.progress.target)}`} />}
          </>
        )}
      </div>
    </div>
  );
}

export function Story({ s }: { s: Store }) {
  const v = s.view;
  const done = v.story.filter((q) => q.chain);
  const active = v.quest;
  const activeRef = useRef<HTMLDivElement>(null);
  // On opening, the middle column (only it: scrollIntoView would move the page too)
  // scrolls the active beat to its center.
  useEffect(() => {
    const el = activeRef.current;
    const box = el?.closest('.col-mid');
    if (!el || !box) return;
    const r = el.getBoundingClientRect();
    const b = box.getBoundingClientRect();
    box.scrollTop += r.top - b.top - (b.height - r.height) / 2;
  }, []);
  return (
    <div class="panel story-panel" data-panel="story">
      <h2>Story</h2>
      <div class="story-timeline" data-testid="story-timeline">
        {v.acts.map((a) => (
          <section key={a.act} class="story-act" data-act={a.act}>
            <div class="act-band" data-testid="act-band" data-act={a.act}>
              <span class="act-number">Act {a.act}</span>
              <span class="act-name">{a.name}</span>
            </div>
            {done
              .filter((q) => q.act === a.act)
              .map((q) => (
                <div key={q.id} class="beat-row">
                  <button
                    type="button"
                    class="beat-node is-done"
                    data-beat={q.id}
                    data-testid="story-beat-done"
                    title="Replay its story"
                    onClick={() => replayStory(q)}
                  >
                    <span class="beat-dot" aria-hidden="true" />
                    <span class="beat-card">
                      <span class="beat-title">{q.title}</span>
                      {paragraphs(q).map((t, i) => (
                        <span key={i} class="beat-text">
                          {t}
                        </span>
                      ))}
                    </span>
                  </button>
                  <Sides v={v} parent={q.id} />
                </div>
              ))}
            {active && active.act === a.act && (
              <div class="beat-row">
                <div class="beat-node is-active" data-beat={active.id} data-testid="story-beat-active" ref={activeRef}>
                  <span class="beat-dot" aria-hidden="true" />
                  <div class="beat-card">
                    <Quest v={v} />
                  </div>
                </div>
                <Sides v={v} parent={active.id} />
              </div>
            )}
          </section>
        ))}
        {active?.more && (
          <div class="beat-row">
            <div class="beat-node is-unknown" data-testid="story-unknown">
              <span class="beat-dot" aria-hidden="true" />
              <span class="beat-card">???</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// ---- Achievements --------------------------------------------------------------------

export function Achievements({ s }: { s: Store }) {
  const v = s.view;
  const cats = [...new Set(v.achievement_list.map((a) => a.category))];
  const earned = v.achievement_list.filter((a) => a.earned).length;
  return (
    <div class="panel" data-panel="achievements">
      <h2>
        Achievements{' '}
        <span class="muted">
          {earned}/{v.achievement_list.length}
        </span>
      </h2>
      {cats.map((cat) => {
        const list = v.achievement_list.filter((a) => a.category === cat);
        return (
          <section key={cat} class="badge-group" data-category={cat}>
            <h3>
              {capitalize(cat)}{' '}
              <span class="muted">
                {list.filter((a) => a.earned).length}/{list.length}
              </span>
            </h3>
            <div class="badges">
              {list.map((a) => (
                <div
                  key={a.id}
                  class={`badge ${a.earned ? 'is-earned' : 'is-unearned'}`}
                  data-achievement={a.id}
                  data-testid="badge"
                  title={a.earned ? 'Earned' : 'Not earned yet'}
                >
                  <span class="badge-medal" aria-hidden="true">
                    {a.earned ? '★' : '☆'}
                  </span>
                  <div class="badge-name">{a.name}</div>
                  <div class="badge-desc">{a.description}</div>
                  <div class="badge-reward">Reward: {a.reward}</div>
                </div>
              ))}
            </div>
          </section>
        );
      })}
    </div>
  );
}
