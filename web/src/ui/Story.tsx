// Story and Achievements. Story is a vertical quest timeline: one node per main-story
// beat on one unbroken spine, oldest at the top, each act opening with its painted chapter
// band, side quests branching off the beat that started them, one dim "???" below the
// active beat. Every card starts collapsed: its quest picture's thumbnail, title (a ✓ once
// finished) and its story's first line, an unfinished one its progress too; a click expands
// it to the whole story and its Goal and Reward lines in the mechanics style (a finished
// main beat's also offers a replay of its story modals). Achievements are badges grouped
// by category. Every text, act and progress comes from the engine's view (the same quest
// rows the story modals read).

import type { ComponentChildren } from 'preact';
import { useEffect, useRef, useState } from 'preact/hooks';
import type { QuestView, SideQuestView, StoryView, View } from '../engine';
import { capitalize, num } from '../format';
import { justMoved, replayStory, type Store } from '../store';
import { QuestPicture, actBand } from './art';
import { ScopeIcon } from './icons';
import { Bar, Mechanics, QuestProgress, QuestTitle } from './common';

/** A finished quest's paragraphs: its opening (its body when it has none), then its payoff. */
function paragraphs(q: StoryView): string[] {
  const opening = q.story && q.story.length > 0 ? q.story : q.body ? [q.body] : [];
  return [...opening, ...(q.payoff ?? [])];
}

/** The active quest's paragraphs so far: its opening (its body when it has none). */
function opening(q: QuestView): string[] {
  return q.story.length > 0 ? q.story : q.body ? [q.body] : [];
}

/** A card's one line: the text's first sentence, with "…" when more follows (ui.md). */
export function firstLine(texts: string[]): string {
  const whole = (texts[0] ?? '').trim();
  // A draft's "DRAFT." marker stays, with the sentence after it.
  const mark = /^DRAFT\.\s+/.exec(whole)?.[0] ?? '';
  const t = whole.slice(mark.length);
  const m = /^.*?[.!?](?=\s|$)/s.exec(t);
  const first = m ? m[0] : t;
  const more = first.length < t.length || texts.length > 1;
  // A period gives way to the "…"; a ? or ! stays.
  return mark + (more ? `${first.replace(/\.+$/, '')}…` : first);
}

/** A Story card, collapsed by default (Matt, docs/rulings/ui.md, *Story cards*): its head
 * (picture, title, ✓) and the story's first line, then `collapsed` (an unfinished quest's
 * progress); a click on the head expands it to the whole story and `children` (its
 * mechanics lines). */
function Card({
  head,
  texts,
  collapsed,
  children,
  testid,
}: {
  head: ComponentChildren;
  texts: string[];
  collapsed?: ComponentChildren;
  children: ComponentChildren;
  testid: string;
}) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <button
        type="button"
        class="card-toggle"
        data-testid={testid}
        aria-expanded={open}
        title={open ? 'Collapse' : 'Show the whole story'}
        // The second click of a double-click on a modal's last button lands here once the
        // modal is gone: not a toggle (pointer clicks only).
        onClick={(e) => (e.detail > 0 && justMoved() ? undefined : setOpen(!open))}
      >
        {head}
        {!open && texts.length > 0 && (
          <span class="beat-text" data-testid="story-card-line">
            {firstLine(texts)}
          </span>
        )}
      </button>
      {open ? (
        <div class="card-body" data-testid="story-card-body">
          {texts.map((t, i) => (
            <p key={i} class="card-para">
              {t}
            </p>
          ))}
          {children}
        </div>
      ) : (
        collapsed
      )}
    </>
  );
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
  const bar = !x.done && x.progress && (
    <Bar frac={Number(x.progress.now) / Number(x.progress.target)} label={`${num(x.progress.now)} / ${num(x.progress.target)}`} />
  );
  return (
    <div class={`side-node ${x.done ? 'is-done' : 'is-open'}`} data-side={x.id} data-testid="story-side">
      <span class="side-dot" aria-hidden="true" />
      <div class="side-card">
        <Card
          testid="story-side-toggle"
          texts={x.body ? [x.body] : []}
          collapsed={bar}
          head={
            <span class="side-title">
              {x.done && <QuestPicture stem={x.picture} at="thumb" />}
              {x.title}
              {x.done && <span class="ok"> ✓</span>}
              <ScopeIcon keep={x.keep} />
            </span>
          }
        >
          <Mechanics goal={x.goal} reward={x.reward}>
            {bar}
          </Mechanics>
        </Card>
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
            <div
              class={`act-band${actBand(a.act) ? ' has-art' : ''}`}
              data-testid="act-band"
              data-act={a.act}
              style={actBand(a.act) ? { backgroundImage: `url(${actBand(a.act)})` } : undefined}
            >
              <span class="act-number">Act {a.act}</span>
              <span class="act-name">{a.name}</span>
            </div>
            {done
              .filter((q) => q.act === a.act)
              .map((q) => (
                <div key={q.id} class="beat-row">
                  <div class="beat-node is-done" data-beat={q.id} data-testid="story-beat-done">
                    <span class="beat-dot" aria-hidden="true" />
                    <div class="beat-card">
                      <Card
                        testid="story-beat-toggle"
                        texts={paragraphs(q)}
                        head={
                          <span class="beat-head">
                            <QuestPicture stem={q.picture} at="thumb" />
                            <span class="beat-title">
                              {q.title}
                              <span class="ok"> ✓</span>
                            </span>
                          </span>
                        }
                      >
                        <Mechanics goal={q.goal} reward={q.reward} />
                        <button type="button" class="link replay" data-testid="story-replay"
                          // Not the second click of a double-click on a modal's last button.
                          onClick={(e) => (e.detail > 0 && justMoved() ? undefined : replayStory(q))}
                        >
                          Replay its story
                        </button>
                      </Card>
                    </div>
                  </div>
                  <Sides v={v} parent={q.id} />
                </div>
              ))}
            {active && active.act === a.act && (
              // Keyed by quest: the next quest's card starts collapsed, not this one's state.
              <div key={active.id} class="beat-row">
                <div class="beat-node is-active" data-beat={active.id} data-testid="story-beat-active" ref={activeRef}>
                  <span class="beat-dot" aria-hidden="true" />
                  <div class="beat-card">
                    <Card
                      testid="story-beat-toggle"
                      texts={opening(active)}
                      collapsed={<QuestProgress q={active} />}
                      head={
                        <span class="beat-head">
                          <QuestPicture stem={active.picture} at="thumb" />
                          <span class="beat-title">
                            <QuestTitle q={active} />
                          </span>
                        </span>
                      }
                    >
                      <Mechanics goal={active.goal} reward={active.reward} hints={active.hints}>
                        <QuestProgress q={active} />
                      </Mechanics>
                    </Card>
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
