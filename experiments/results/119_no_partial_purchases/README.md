# 119: no partial purchases (rules 43) and the ideology emblems

## Rules (revision 42 → 43, bumped from the constant)

Matt's parked edit P1 (2026-11-25) and his staging rulings (amendment #3522), in
`programs::execute` (`crates/hh-core/src/modules/programs.rs`):

1. **Full cost or nothing.** A command execution that doesn't hold every cost in full fails:
   nothing paid or made, no boredom, the step spent, the scarcest input noted (the line's red
   failed dot names it). The fraction run of a conversion (Idle, Cloud Compute, Run
   Simulations, Buy *, Buy Power) is gone; effect commands already needed their full cost.
2. **Cap-overflow skip.** Then, a run whose output would pass any capped product's cap
   (held + output > cap, as the line runs, the output after `scale_modifier`) is skipped. It
   once skipped only with every product already at its cap (`economy::would_overflow`
   replaces `all_capped` for commands; buildings keep `all_capped`). The cost check comes
   first, so a line it can't afford always says what's short.
3. Loading still tops pads up partially: Load pays its 2 energy in full, then each pad takes
   what cargo there is (new test `loading_tops_a_pad_up_partially_with_what_is_held`).

The golden game's fingerprint didn't move (its scripted policy buys only what it can afford
in full, below the cap); the sim pins did (`crates/hh-sim/tests/cli.rs`, by `set_pins.py`).
The hands-off loadout still ships and builds in run 3.

## Display

- A skipped line's gutter dot is gray with "skipped: its output would pass its cap" (it read
  green, "Ran on the last day"); the one-gutter ruling already says gray is skipped. The tab
  light's hover says "output would pass its cap" (was "outputs full").
- The Blocks row no longer shows a short on a line that ran (no command runs part way).

## Ideology emblems (amendment #3518)

`emblems.py` cuts Matt's picks from picker `ideology_emblems_r1` (Nationalist `marker`,
Humanist `garden`, Rationalist `archive`) from the square originals in the shared
`art/ideology_emblems_r1/raw/`: center-cropped to 4:5, 320x400 (2x the 160 px slot), WebP
q88, into `web/src/assets/ideology/<axis>.webp` (89 KB together). `panels.tsx` shows each in
its card's emblem slot (`object-fit: cover`; the card is about 160x197 at 1600 px wide),
the chip standing in for an axis with no file.

## Browser check

`check.mjs` against `vite preview` (HH_URL), all PASS:

| Check | Result |
|---|---|
| at_cap, Buy Titanium on 10 credits (costs 20) | nothing paid or made; failed dot "needed 20 Credits, had 10" ([shot](119_short_red_dot.png)) |
| titanium 19.5 of 20, 1,010 credits | not run; gray dot "its output would pass its cap" ([shot](119_overflow_skipped.png)) |
| pad_loading, pad on Titanium, 2 titanium held | Load ran, the pad took the 2 (2 / 100) ([shot](119_partial_load.png)) |
| everything, Ideologies | three emblems loaded in their slots ([shot](119_ideology_emblems.png)) |
