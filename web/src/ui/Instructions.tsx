// The text editor's "All instructions" panel: every line the player can use right now, by
// group, as the engine lists them (`program_instructions`: revealed and not locked this run,
// `else` only where it fits at the caret). Clicking one inserts it below
// the caret's line (TextEditor.tsx); hovering one shows the card its Commands, Buildings,
// Research or Projects panel shows, drawn live from the view.

import { useMemo, useState } from 'preact/hooks';
import type { InstructionGroup, View } from '../engine';
import { programInstructions, type Store } from '../store';
import { Card } from './Buildings';
import { CommandBody, Project, ResearchCard } from './panels';

/** Re-ask the engine at most this often as the game plays (what is revealed changes). */
const REFRESH_MS = 1000;

const NO_FRESH = new Set<string>();

export function InstructionPanel({ s, text, cursor, insert }: { s: Store; text: string; cursor: number; insert: (line: string) => void }) {
  // The game redraws every day: ask again on a new draft or caret line, or a second later.
  const tick = Math.floor(performance.now() / REFRESH_MS);
  const groups = useMemo<InstructionGroup[]>(() => {
    try {
      return programInstructions(text, cursor);
    } catch {
      return [];
    }
  }, [text, cursor, tick]);
  const [hover, setHover] = useState<{ card: string; top: number; right: number } | null>(null);
  return (
    <div class="te-instructions" data-testid="instructions" onMouseLeave={() => setHover(null)}>
      <h3>All instructions</h3>
      {groups.map((g) => (
        <div key={g.name} class="ins-group" data-group={g.name}>
          <h4>{g.name}</h4>
          <ul>
            {g.items.map((it) => (
              <li key={it.text}>
                <button
                  type="button"
                  class="ins-item"
                  data-instruction={it.text}
                  title="Insert below the cursor"
                  // Keep the caret in the text area.
                  onMouseDown={(ev) => ev.preventDefault()}
                  onClick={() => insert(it.text)}
                  onMouseEnter={(ev) => {
                    if (!it.card) return setHover(null);
                    const r = ev.currentTarget.getBoundingClientRect();
                    setHover({ card: it.card, top: r.top, right: window.innerWidth - r.left + 8 });
                  }}
                >
                  <code>{it.text}</code>
                </button>
              </li>
            ))}
          </ul>
        </div>
      ))}
      {hover && <HoverCard v={s.view} card={hover.card} top={hover.top} right={hover.right} />}
    </div>
  );
}

/** The card an entry names, from the view now (live), beside the entry. */
function HoverCard({ v, card, top, right }: { v: View; card: string; top: number; right: number }) {
  const [kind, id] = card.split(/:(.*)/s);
  let body = null;
  if (kind === 'command') {
    const c = v.commands.find((x) => x.id === id);
    if (c)
      body = (
        <div class="card command" data-id={c.id}>
          <div class="card-head">
            <h3>{c.name}</h3>
          </div>
          <CommandBody v={v} c={c} />
        </div>
      );
  } else if (kind === 'building') {
    const b = v.buildings.find((x) => x.id === id);
    if (b) body = <Card v={v} b={b} tags={false} fresh={NO_FRESH} />;
  } else if (kind === 'research') {
    const r = v.research.find((x) => x.id === id);
    if (r) body = <ResearchCard v={v} r={r} fresh={NO_FRESH} />;
  } else if (kind === 'project') {
    const p = v.projects.find((x) => x.id === id);
    if (p) body = <Project v={v} p={p} fresh={NO_FRESH} />;
  }
  if (!body) return null;
  // Kept on screen: a card near the bottom opens upward.
  const at = Math.max(8, Math.min(top, window.innerHeight - 340));
  return (
    <div class="ins-card" data-testid="instruction-card" data-card={card} style={{ top: `${at}px`, right: `${right}px` }}>
      {body}
    </div>
  );
}
