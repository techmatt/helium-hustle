# 071: the Programs view

Matt's rulings of 2026-11-23 (docs/rulings/programs.md): a Programs nav tile opening a
full-width takeover with Text | Blocks on the same program, one instruction panel and one
run gutter for both, repeat counts first (`4x sell cloud compute`), and per-run dots on a
repeated line.

Shots from e2e section 10 (`programsBlocks`), on a new game:

- `programs-blocks.png`: Blocks, built by panel clicks only: an `if` (amber, `if` not open
  yet), its body, `else` and its body, then a line moved out with ⇤ and given 4 runs in its
  count field. The dashed frame is the selected row (the panel adds below it), the solid
  one the next line to run.
- `programs-text.png`: the same program in Text, `4x idle` written count first.
- `programs-dots.png`: at 1x with one processor, four days on: the `4x idle` line's four
  run dots filled green (the locked `if` passes both bodies free, so the line runs daily).

e2e reads them as `0/4 → 1/4 → 2/4 → 3/4 → 4/4`, one a day.
