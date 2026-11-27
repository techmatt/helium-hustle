# Rulings: programs

Part of Matt's standing rulings ([index](../rulings.md): what **locked** and **default** mean).
The rules as implemented: [rules/programs.md](../rules/programs.md).

- **One flat program template (2026-11-20).** Every line is `<verb> <target> [amount]`: the
  target slot is typed by the verb, the amount `N`, `up to N` or `all`. No nesting, variables,
  labels or jumps; a new capability is a new verb or reading, never new grammar (`else`,
  below, is Matt's one addition); per-target
  commands (Buy X, Fund X) are one verb with a target. Conditions stay
  `<reading> <op> <number>` with one optional `and` / `or`; the right side is
  always a number. **Symbols (2026-11-23):** `op` is `>`, `<`, `>=` or `<=`; the words
  above / below are gone everywhere (text, block view, docs, loadouts); state stores the
  comparison, so saved programs keep working. Pads are addressed only by good. Every line is open wherever program logic
  is, but `if`, which has its own unlock (*Program logic unlocks*, below). No `wait until`
  (2026-11-22): programs run every day, so `if` covers it. *Defaults (037):* the pad verbs run
  the Launch / Load commands' rows; `retire` retires at the day's end like the button.
- **`else` (2026-11-23).** One level, as `if`: an `else` line directly follows an `if`'s
  body (or the `if` itself, with an empty body), and its own indented lines are its body. No
  `else if`, no nesting. The `if` keeps its one step and decides both branches: true runs the
  if-body and jumps past the else-body; false jumps to the else-body. The `else` line costs no
  step (like a note), so watching a program still shows one decision per `if`. Open whenever
  `if` is (no unlock of its own); a locked `if` passes over its else-body too.
- **The program language unlocks early (2026-11-23).** It opens mostly during Act 1
  (instructions, readings and features may still open there through quests, research or
  projects); after that no progression adds program functionality, so loadouts never depend
  on a later unlock. The `if` step cost exists so a player can watch a program run line by
  line.
- **Every line costs a step (2026-11-22, 064; replaces 2026-11-20's free `if`).** Each line
  that runs takes one processor step, so play is easy to follow at 1×. A line with nothing to
  do (`research X` once researched, `pause` with nothing running, `set pads` already at N,
  `launch pads` with no pad ready, `build X up to N` with N owned) still takes its step, and
  nothing else: no resources, no boredom. An `if` takes its step too, costs nothing else and
  can't fail; a false `if` skips its whole body at once, however long. Notes and locked lines
  are free. *Default (038):* `load pads` is done when no pad of the good would take cargo.
- **Locked lines (2026-11-22, 064; replaces 063's "fails and advances").** A line for
  something not unlocked this run (program logic, `if`, a reading's research, a command,
  building, research item or project) is kept (loaded, pasted or set) and skipped for free,
  like a note; a locked `if` skips its body free too. The amber status stays (both modes).
- **Program logic unlocks (2026-11-21).** `if` is not available from the start: a cheap
  global project offered from A Voice's start (2026-11-26; or a first retirement before it)
  unlocks it. Its effect lives on the project's
  completion (2026-11-24): a timeline's end locks `if` again until Conditional Logic is
  re-funded, and saved loadouts' `if` lines stay locked (passed free) meanwhile; no upgrade
  that keeps it is planned.
- **Commands run only as program lines (2026-11-25).** No command is runnable by clicking:
  there is no Run button and no by-hand action. From day 1 the right column's program, every
  "Add to Program" button and the loadout Save/Load buttons are there, as are processors
  (the Processors row). 1.1's hint (mechanics text, never its story: ui.md, *Quest
  mechanics text*) names Add to Program on Sell Cloud Compute as the way to earn credits. A player can play Act 1 up to the first Research Lab with the right
  column's program alone (add, drag to reorder, remove, −/+ counts; logic lines from its
  own add row once they open).
- **No partial purchases (Matt, 2026-11-25; 119).** Every command needs its full cost for
  each execution, or that execution doesn't happen (a red dot naming what's short; its step
  still spent); repeat counts are per execution; a buy whose output would overflow its cap
  doesn't run, judged as the line runs (held + output > cap: Buy Power at 95 of 100 energy
  is skipped even if upkeep drains it later), for every command with a capped product;
  loading pads still tops up partially (not a purchase); fractions from production rates
  stay.
- **Programming (2026-11-25, replacing 2026-11-24's).** **Programming** is a research-view
  item (placeholder 200 science), on sale from quest 1.5 A Voice's start on (Matt, 2026-11-26); not a quest
  task. Buying it is permanent: a career flag (`programming`) that survives retirement and
  a timeline's end, like `swarm_revealed`; once bought it never shows again. It hides only
  the Programs nav tile and its view (Text | Blocks, the all-instructions panel); the
  program rules are unchanged. Saves already past it get the flag on load (a state saved
  before revision 29). Sim players and the golden script never buy it (nothing they use is
  gated). The price takes the Rationalist and Archive discounts like any research.
- **Program loadouts (2026-11-20).** Each program has save / load buttons. A saved program
  goes into a library that persists across retirements (career state), so a player keeps
  programs for different stages and loads one into any program tab. Programs still empty on
  retirement; loading is how they come back. *Defaults (035):* 12 slots, display names up to
  40 characters, loading leaves processors as assigned; the buttons are open from run 1.
- **Programs persist (2026-11-22).** A player's programs carry over across runs (a saved
  library); building it is its own item when Matt asks.
- **"Add to Program" (2026-11-22)**, not "Add to Program 1": the active program is implied.
- **Programs view (2026-11-23, replaces the Text editor's button and title).** A
  **Programs** nav tile opens a full-width takeover titled "Programs" on the active
  program; the Commands tab and the right column's at-a-glance program stay. **Text |
  Blocks**, both first-class, edit the same program: the game is beatable without typing
  (everything the text says, `if`/`else`, repeats, every instruction and its numbers,
  builds in Blocks). The view remembers the last mode; a new player starts in Blocks.
  Blocks is the block editor, controls minimal (add, drag, indent, delete, number fields).
  *Defaults (071):* the mode is a browser preference (localStorage), not game state; Blocks
  edits apply at once (as the right column), Text on Apply or leaving; a Blocks row is
  selected by a click; a note's text is a field; a panel `if` goes after the selected
  row's whole block, so it never takes lines below into its body.
  *No hint text (073):* the view carries no lines explaining unlocks or nudging the player
  (no unlock hint, no how-to line in either mode, no empty-program hint); the rules they
  described stand. A Blocks number field selects its text on a click or focus, and a drag
  inside it selects text without dragging the row.
- **Text mode (2026-11-22).** The block view's words, `#` notes kept, aliases fine; red
  and amber lines kept; saved on Apply or leaving; copy/paste is import/export.
  *Defaults (063):* blanks dropped.
- **One gutter (2026-11-23).** Both modes show run state the same way: green ran, gray
  skipped, red unparsed, amber locked, fired k/10 on `if`s, the reasons. A repeated line
  shows one small dot per run up to 8 (green done, red failed, hollow pending, filling left
  to right); above 8, a short bar with "done/total, N failed" on hover; a single-run line
  keeps its one dot. *Default (071):* a repeated line shows its current pass, else its last
  (engine display state `Program.repeats`, cleared when the lines change).
- **Repeat counts lead (2026-11-23).** `4x sell cloud compute`; a trailing number is always
  an argument (`pause data center 1`, `build solar panel up to 2`, `fund <project> 50`). The
  old trailing count no longer reads (red, its reason naming the new form); saved state is
  unaffected. *Default (071):* `4X`, `4×`, `4*` and `4 x` read as `4x`.
- **All instructions panel (2026-11-23).** Right of the program in both modes (no
  search; a click adds a line in Text, a block in Blocks, below the caret's or selected
  row): Commands; Build; Pads; Buildings on/off; Research and projects; Programs;
  Control flow (one `if <reading> > 0` per reading on offer, `else` only where legal at the
  cursor, `#`). Only what the player can use right now: nothing locked
  (no amber entries; 2026-11-23, replacing 070's amber default). Hover: the live card of the Commands / Buildings panel. Click: a new row below
  the cursor's line with its indent, cursor moved there; an `if` adds an empty indented line
  with the cursor on it; number slots 1, `up to N` owned + 1. *Defaults (070):* `if` and
  `else` go in unindented (they can't be indented) and `else` gets a body line too; an
  empty cursor line is filled in place; research and project entries hover their panels'
  cards.
- **`fund <project>` (2026-11-20).** A `fund <project> [amount]` verb funding a project the
  way the Projects panel does; it may be switched off later, one config change
  (`fund_verb`). *Defaults (039):* `fund P N` sets N a day on every cost of P (rules in
  programs.md); JSON verb `fund`, so a later `fund <axis>` can share it.
- **Readings (2026-11-22).** One reading per thing, only what the player has
  unlocked ([rules/programs.md](../rules/programs.md)). A capped
  reading's number box has a % / amount unit toggle, no new entries; Energy per day and Days
  since a purchase are research-unlocked (Process Telemetry, 062's pick); no building counts
  yet. The golden game runs a short condition program, pinning the interpreter, from the
  `golden_start` scenario with `if` open (049_1). *Defaults (040):* `free processors` is
  granted by active buildings less assigned (what `assign processors ... all` would add);
  `idle pads` is pads with no cargo (a cooling pad counts, and a No-cargo pad, 069).
- **Program tabs at a glance (Matt, 2026-11-26).** Each of the right column's five tabs
  shows its processor count beside its number (`1 ⚙2`); a tab with none shows just its
  number, dimmed. A tab with processors has a status light left of the number, judged on
  the last day played: green, every step ran in full; yellow, some steps were short, failed
  or skipped; red, nothing useful ran (empty, no step taken, every step failed, or a line
  that doesn't read). Its hover says why ("2 processors · 6 of 8 steps ran in full · Load
  Launch Pads: short on energy"). The engine judges it (the view's `status`, from a per-day
  step tally); the open tab's "N assigned (M free)" line stays. *Defaults (114):* skipped
  is an output that would pass its cap or `load`/`launch pads` with no pad ready (another verb with its goal
  met is done, in full); a
  failed step is short (no command runs part way); a locked line or a processor not running (Data Centers unpaid)
  makes it yellow too; no step taken (also before the first day with processors) is red; an
  edit keeps the day's tally; the hover names the first line in trouble, in line order. A
  command line (Load Launch Pads) that runs with nothing to load counts as run.
