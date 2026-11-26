// The Earthrise art: the starry sky over the center column (Earth lives in the paintings
// now: the act bands and the quest pictures), the Story act bands and the quest pictures.
// The lunar horizon at the foot of the left column is icons.tsx's.
//
// The sky is the home for a later live view of the base (Flash/3D): replace what <Sky>
// draws and keep its box (style.css `.sky`), so nothing else on the page moves.
//
// Files: src/assets/story/act<N>.webp (the act bands) and src/assets/quests/<stem>.webp
// (a quest's `picture` in content/events.json), written by
// experiments/results/078_ship_story_headers_title_and_picture_slots/process.py (the bands)
// and experiments/results/080_ship_act1_quest_pictures/process.py (Act 1's first set; 099
// renamed them to bare stems, and 095's new line moved beats, so that script no longer
// matches content: rerun it only after updating its beat-to-stem map).

import { byName } from './assets';

const STARS: [number, number, number][] = [
  [4, 18, 1], [11, 62, 0.8], [17, 30, 1.2], [26, 80, 0.8], [33, 12, 0.9], [41, 50, 1.1],
  [48, 88, 0.7], [55, 24, 1], [61, 66, 0.8], [68, 40, 1.2], [74, 10, 0.8], [79, 92, 0.9],
  [86, 36, 0.8], [93, 70, 1], [97, 20, 0.7],
];

export function Sky() {
  return (
    <div class="sky" aria-hidden="true">
      <svg class="stars" width="100%" height="100%">
        {STARS.map(([x, y, r], i) => (
          <circle key={i} cx={`${x}%`} cy={`${y}%`} r={r} />
        ))}
      </svg>
    </div>
  );
}

const stems = (files: Record<string, string>) => byName(files, '.webp');

const ACT_BANDS = stems(import.meta.glob<string>('../assets/story/act*.webp', { eager: true, query: '?url', import: 'default' }));
const QUEST_PICTURES = stems(import.meta.glob<string>('../assets/quests/*.webp', { eager: true, query: '?url', import: 'default' }));

/** An act band's painting, if the act has one. */
export function actBand(act: number): string | undefined {
  return ACT_BANDS[`act${act}`];
}

/** A quest's picture (its `picture` stem), undefined when it has none. */
export function questPicture(stem: string | null | undefined): string | undefined {
  return stem ? QUEST_PICTURES[stem] : undefined;
}

/** A quest's picture where it shows: the story modal (`modal`), the left column's foot
 * (`left`), a finished beat's thumbnail (`thumb`) or a side quest's completion banner
 * (`banner`, its modal's stand-in); nothing when it has none. */
export function QuestPicture({ stem, at }: { stem: string | null | undefined; at: 'modal' | 'left' | 'thumb' | 'banner' }) {
  const src = questPicture(stem);
  if (!src) return null;
  return <img class={`quest-pic quest-pic-${at}`} data-testid={`quest-pic-${at}`} data-picture={stem} src={src} alt="" draggable={false} />;
}
