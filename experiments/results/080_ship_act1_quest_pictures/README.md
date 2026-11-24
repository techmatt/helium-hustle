# 080: Act 1's quest pictures in the game

Ships Matt's picks from picker `quest_pictures_act1_r1` (one per quest, 1.1-1.11 and
1.a-1.c) into 078's slots: the story modal, the left column's foot while the quest is the
active main quest, and Story's thumbnail. 1.1's `earthrise` replaces 078's `lunar_base`
stand-in (its file is gone; the lunar horizon stays the left column's fallback for a quest
without a picture, which is every quest past Act 1).

## Recipe

- `python process.py` (repo root, Pillow): for each pick, the painted original 079 drew,
  `art/quest_act1_r1/raw/<row>-<key>.png` (1536x1024; the picker's composed sheet carries
  the same file name), scaled to 768x512 and saved as `web/src/assets/quests/<row>_<key>.webp`
  (43-83 KB, 0.9 MB in all), and `"picture": "<row>_<key>"` on the quest's row in
  `content/events.json`.
- `node shots.mjs <preview url> sizes|final|banner`: the shots in `shots/` (PNG there,
  committed as JPEG; the PNGs are in the shared screenshots folder as `080_*.png`), 1280x800
  light and 1920x1080 dark.

## Decisions

- **Thumbnail 60x40** (a done side quest's 45x30), between 078's 48x32 and 079's 72x48,
  judged in the real Story timeline (`thumbs-<size>__*.jpg`: story_acts, Act 1 scrolled into
  view, the size forced by an injected style). At 48x32 the pictures read as color patches
  (1.3's canister, 1.5's racks vanish); at 72x48 every beat's title row grows about 16 px
  and the title floats mid-picture; 60x40 keeps each subject legible with the title row
  barely taller than the text.
- **Side quests' banner** (Matt, answering question 1): a side quest has no story modal and
  is never the active main quest, so 078's slots reached 1.a-1.c only as Story's thumbnail.
  Its completion banner (the side quest's modal stand-in) now shows its picture at 120x80
  left of its text (`QuestPicture at="banner"`). The left column stays main-quest only.
- Pictures are exported at 768x512: 2x the modal's 360x240 and past the left column's
  287 css px at 2x; a 3:2 painting fills all three slots with no crop.

## Shots

`intro` (1.1's modal and left column), `left2` (1.2 active: its picture at the column's
foot), `act1` and `sides` (Story in story_acts: every Act 1 beat and side quest with its
thumbnail), `replay` (1.7's replayed modal), `banner` (1.b's completion banner).
