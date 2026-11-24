# 078: Story act bands, the split-glow title and quest-picture slots

Ships story_title_art_r2's final calls (art/pickers/story_title_art_r2/manifest.json): a painted
side band per act (Act 1 `side3`, Acts 2-4 round 1's `side`), the `split-glow` title in CSS on
the theme tokens, no Earth globe over the middle column; and an optional `picture` on every
quest (content), shown above its story modal's paragraph, at the left column's foot while it
is the active main quest, and as a finished beat's thumbnail. Only 1.1 has one: `lunar_base`,
the left column's own lunar-base scene, which stays the fallback.

## Recipe

- `python process.py` (repo root, Pillow): crops the four picked paintings (1536x1024) to the
  band's 3:1 at round 1's vertical focus (076's `crops.json`: Act 2 at 38%, the rest 50%) into
  `web/src/assets/story/act<N>.webp`, and copies `horizon/bare.webp` to
  `web/src/assets/quests/lunar_base.webp`. A new quest picture is a file
  `web/src/assets/quests/<stem>.webp` and `"picture": "<stem>"` on its row in
  `content/events.json` (a stem the client has no file for shows nothing).
- `node shots.mjs <preview url>`: the shots in `shots/` (PNG there, committed as JPEG; the PNGs
  are in the shared screenshots folder as `078_*.png`), 1280x800 light and 1920x1080 dark: a
  new game's 1.1 modal (`intro`), Story on 1.1 (`story11`: 1.1's thumbnail on the active
  beat) and after it (`act1`), each act band from the new
  `story_acts` scenario (`act<N>_band`), the title at 2x (`title`).

## Notes

- Acts 2-4 are reachable in a browser now through scenario `story_acts` (everything's base, every
  chain quest before Act 4's first done), so the later bands are shot in the real panel.
- The middle column's top padding drops from 100 px (room for the globe) to 36 px; the starry
  sky gradient stays as the home for the later live base view.
- Slot sizes: modal at most 360x240 px (`object-fit: cover`), thumbnail 48x32 (36x24 on a done
  side quest), left column full width at the picture's own aspect, in the horizon's shrinking
  box. 079's candidates are about 3:2, which fits all three.
