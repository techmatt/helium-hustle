# 079: Act 1 quest pictures, round 1

Picker `quest_pictures_act1_r1` (`art/pickers/quest_pictures_act1_r1/manifest.json`, in the
shared, uncommitted `art/` folder): 14 `many` rows, one per Act 1 quest (1.1-1.11, then the
side quests 1.a-1.c), 49 captioned candidates. Nothing goes into the game here; a later item
ships the picks.

## Recipe

From the repo root (Node 24, Python with Pillow, Codex CLI on Matt's ChatGPT login):

1. `node …/gen.mjs [<id> …] [--pool=N]`: 077's Codex draw with this folder's `prompts.json`
   (`draws`: id → [beat, earth, scene]; `earth` picks "no Earth in the sky" or "Earth as
   described"). Originals into `art/quest_act1_r1/raw/`, one log line each.
2. `python …/contact.py [<beat> …]`: review rows in `out/<item>/contact/` (each draw at
   360x240 and at the 72x48 thumbnail).
3. `captions.json` lists the kept draws, each with its one-line caption; a cut draw has none.
4. `python …/compose.py`, then `python …/picker.py`.

## How the sheets are made

`compose.py` draws, at 1x on the dark theme's tokens: the story modal (style.css `.modal`,
560 px with 22 px padding, so a 516x344 picture between the quest's title and its first story
paragraph; a side quest's `body`, as it has no story yet), the left column at 287 px (the prompt's
width; the horizon box inside it measures 286, the column's 1 px border off) with the picture
at its foot where the lunar-base scene sits, and a Story timeline row with a 72x48
thumbnail beside the beat's title and summary. The thumbnail size is this item's guess (the
timeline has no thumbnail today); the shipping item picks the real one. Not a game screenshot:
the pictures are the same in both themes, and laying 49 into the running page would add a
browser run for nothing a picker needs.

## Candidates and cuts

53 prompts (4 per main beat, 3 per side quest), 54 draws with one redraw, 28-46 s each, 4 at once, no failure, no usage
limit (`art/quest_act1_r1/raw/log.jsonl`). Cut before the picker:

| Draw | Why |
|---|---|
| 1.5 `cores` | a near-duplicate of 1.5 `tworacks` (a wall of amber rack lights) |
| 1.6 `horizon` | the watchers on the horizon didn't come out: it reads as just the base |
| 1.10 `crossing` | the rival excavator came back in our white and orange: no rival in it |
| 1.11 `done` | a near-duplicate of 1.11 `silhouette`, the swarm barely visible |
| 1.11 `void`, first draw | the swarm drew as a whale, a creature the text doesn't name; redrawn shapeless |

## Findings

- **Earth** is in 9 of the 49, only where the text puts it: 1.1 (Earth overhead), 1.3 and 1.4
  (cargo bound for Earth), 1.6 (what Earth pays), 1.9 (Earth watching). The rest ask for no
  planet in the sky, and none drew one.
- **Readability at 72x48**: the single-subject draws (a canister, a dish, a rack, the vault
  door) hold up; the busy base scenes (1.5 `routine`, 1.10 `plots`) turn to texture.
- **Invented imagery to watch**: the factions in 1.9 are shown only as their chip colors (red,
  green, blue), the rival in 1.10 as cold cyan lights (the land rival is not one of the four
  rival AIs whose colors `icons.tsx` fixes); in 1.10 `beacons` the far base is still white with
  orange trim, only its lights cyan, so it reads as a rival only by caption), the speculators in 1.6 and 1.a as small red-lit drones. These are picture shorthand,
  not lore; the text names none of them.
- The telescope draws as a radio dish every time; "telescope" alone reads that way to the tool.
