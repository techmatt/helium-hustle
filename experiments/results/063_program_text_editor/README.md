# 063 Program text editor

A "Text editor" takeover on the active program: the program as text in the block editor's
words, `#` notes kept, live gutter status, saved on Apply or on leaving. The form and its
aliases are in [docs/rules/programs.md](../../../docs/rules/programs.md), *Text form*; the
UI in [web/README.md](../../../web/README.md).

- `L4_program1.txt`, `L4_program2.txt`: 060's L4 loadout written in the text form (paste
  one into a program's text editor). `L5_program3.txt`: L5's Dream tab (the hands-off
  player's). The test `the_written_up_loadouts_read_without_a_red_line` (hh-core) keeps
  them reading clean and canonical.
- `063_text_editor_{dark,light}.png`: L4 Program 1 plus a typo (red) and a building not
  open yet (amber), after 10 days at 10x in scenario `everything`: green and grey dots,
  "fired k/10" on the `if` lines.
- `063_block_editor_{dark,light}.png`: the same program back in the block editor: a note,
  an unparsed line with its reason, a locked line dimmed with "locked this run".
- `shots.mjs`: takes them (HH_URL a `vite preview`; OUT, COPY optional).

Same moves as revision 17 for every program without the new lines: the ladder grid (5
players, seeds 1-2, 2 retirements) and the hands-off slow-lane game (3 runs) play the same
runs on main and here, figure for figure; only state fingerprints moved, with the config's
new `line_text_max`.
