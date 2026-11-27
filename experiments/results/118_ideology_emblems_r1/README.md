# 118: ideology emblems, round 1

> Archived (127): `art/pickers/ideology_emblems_r1/` now lives at `C:\Code\gdrive-sync\helium-hustle\archive\art\pickers\`.

Picker `ideology_emblems_r1` (`art/pickers/ideology_emblems_r1/manifest.json`, in the shared,
uncommitted `art/` folder): one row per axis (Nationalist, Humanist, Rationalist), five
captioned candidates each, nothing selected. Nothing goes into the game here; a later item
ships the picks with the redesigned Ideologies view.

## Recipe

From the repo root (Node 24, Python with Pillow, Codex CLI on Matt's ChatGPT login):

1. `node …/gen.mjs [<id> …] [--pool=N]`: 077/079's Codex draw with this folder's
   `prompts.json` (`axes`: the faction line, from content and `docs/rules/ideology.md`, with its
   chip hex from `web/src/ui/icons.tsx` and its chip glyph; `draws`: id → [axis, subject]).
   Square originals into `art/ideology_emblems_r1/raw/`, one log line each.
2. `captions.json` lists the kept draws, each with its one-line caption.
3. `python …/compose.py`, then `python …/picker.py`.

## Preview size

The redesign isn't built, so the card is a guess: 760x300 on the dark theme's tokens, the
emblem a **240x300 panel** (the square draw center-cropped to 4:5) on its left edge, a 3 px
rule in the chip color, then the chip, the name, a sample rank line and bar and what a rank
does. Beside it the full square at 300x300 with the crop's edges marked, so the shipping item
can pick another panel shape. Drawn square so a portrait or square panel both work.

## Candidates

15 draws (5 per axis: a heraldic shield or medallion, a banner, two of scene, monument or
seal, and a carved crater-wall relief added in a second run), 33-44 s each, 3 at once, no
failure, no redraw, no usage limit
(`art/ideology_emblems_r1/raw/log.jsonl`). None cut.

## Findings

- Each axis's subject came from its content: Nationalist from demand, land and launch pads and
  the Microwave Power Initiative (a cargo rocket, a claim marker); Humanist from boredom, the
  Dream and the AI Consciousness Act (a garden dome, a human and a robot hand clasping);
  Rationalist from science and the Universal Research Archive (the archive vault, a telescope).
  The emblems reuse the chip glyphs: shield and star, two companions, drafting dividers.
- **Red star on white** (Nationalist) reads as Soviet in several draws, the crater relief and
  the obelisk most. The chip glyph already is a star on a shield; worth a thought if the
  faction should read as generic flag-waving nation-states.
- Earth came out in the archive draw unasked; the telescope draw added a blue spiral galaxy.
- The hands seal is the only candidate with a human body part; the preamble bans faces, not
  hands.
- The Codex route is quick and steady here: 15 for 15, about 4 minutes of wall time over two runs.
