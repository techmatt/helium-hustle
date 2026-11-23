# 070: the All instructions panel and comparison symbols

Matt's rulings of 2026-11-23 (docs/rulings/programs.md): conditions are written with `>`,
`<`, `>=`, `<=` only, and the Text editor gets an "All instructions" panel.

- **Symbols.** `Cmp` gains `ge` / `le` (`>=` / `<=`, inclusive); the text form reads and
  writes only symbols (`≥` / `≤` still read). The block view's comparison control offers the
  four. The hands-off loadout and the example loadouts under 060 and 063 were rewritten in
  symbols (same comparisons: every old `above`/`below` was strict). Rules 23; the golden
  fingerprint and the sim pins did not move.
- **The panel** (`web/src/ui/Instructions.tsx`) shows the engine's list
  (`program_text::instructions`, wasm `program_instructions`): groups in the ruling's order,
  only revealed rows, locked ones amber with `programs::locked`'s reason, `else` only where
  `program_text::else_fits` says an inserted `else` reads. Hover cards are the panels' own
  components drawn from the live view.

Shots (e2e section 9): `instructions.png` (scenario everything, `if` open, a building card
hovered) and `instructions-fresh.png` (a new game: Build and the rest amber).
