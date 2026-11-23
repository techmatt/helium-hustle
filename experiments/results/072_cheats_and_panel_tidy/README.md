# 072: cheats and panel tidy

Three parked edits (Matt, 2026-11-23), no rules revision (cheats are debug actions; the golden
game uses none, so no pin moved; 058 added the cheat building on the same footing).

- **No CHEATED tag in the bottom bar.** The state's `cheated` mark is unchanged (save, record,
  play log); the debug info line in Options still says `CHEATED`.
- **Cheat Fountain: +1000 credits a day** per copy (`cheat::CHEAT_BUILDING_CREDITS_RATE`); its
  other incomes stay +10 a day and boredom −10 a day. The view carries `credits_rate`; the card
  reads `+1,000 / day credits`, `+10 / day to every other income`, `−10 / day boredom`.
- **All instructions lists only what can be used now.** `program_text::instructions` drops any
  entry `programs::locked` reports (and a group left empty); `Instruction.locked` is gone.
  On a fresh career that leaves Commands and a `#` (no `if`: program logic opens at 2 working
  processors). e2e section 10 (Blocks if/else, 4x dots) moved from a new game to scenario
  `everything` with one processor, since a fresh career's panel no longer offers `if`.

Shots (e2e): `instructions-fresh.png` (fresh career: Commands and `#` only),
`instructions.png` (`everything`), `programs-blocks.png`, `programs-dots.png`.
