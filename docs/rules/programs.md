# Programs (module `programs`, variant `arc1`)

Program tabs (command queues), the processors that run them, command execution and
overclocks. In [the day](README.md#the-day), expired overclocks drop before the building
pass, and programs run after the building pass and demand update, before projects drain and
shipments move.

## Rules

**Programs.** There are `program_count` programs (0-based in actions). Each holds a queue of
lines (`command`, `count`: how many times the line runs before moving on), a shared pointer
(line and runs done of it) and a number of assigned processors. A new run gives every program
an empty queue and puts all processors on the first program.

**Processors** are the processors resource, floored to a whole number (economy.md: 1 per
active Data Center). All programs together may hold at most that many. When processors are
lost (a Data Center sold or set inactive), assignments are cut from the last program first. A
processor gained mid-run (a new Data Center) starts unassigned.

**Running** (each day): programs in order; for each processor assigned to a program, the line
under its pointer is executed once, then the pointer advances whatever the outcome (ran,
skipped or failed): the line's run count rises and, at its `count`, the pointer moves to the
next line, wrapping to the first. A program with no lines does nothing. So a program with 3
processors runs three consecutive steps of its queue a day.

**Running a command** (one execution):

1. **Locked** (its `requires` doesn't hold): fails.
2. **Flows:** its `costs`, with any researched `command_cost_override` replacing that cost,
   and its `produces`; a `scale_modifier` multiplies both (Buy Power uses `buy_power_mult`,
   which includes the career's bonus; [career.md](career.md)).
3. **Output-cap skip:** if it produces anything and every product is at or over its cap
   (economy.md), it is skipped: nothing paid, nothing made, no boredom.
4. **Fraction:** `frac` = the least of `max(held, 0) / cost` over its costs, at most 1 (1 with
   no costs). If `frac` is 0, it fails. A command with an effect (Load, Launch, Dream,
   Overclock, Promote, Disrupt, Fund) fails unless `frac` is 1; a pure conversion (Idle,
   Cloud Compute, Buy *) runs at `frac`.
5. It pays `cost × frac` and adds `produces × frac`. Products may pass their cap until the
   day's clamp.
6. It adds its flat `boredom` (whole, not scaled by `frac`); a completed project with
   `boredom_modifiers` replaces a command's boredom with its own value.
7. The command is marked used in the career (it stays visible in later runs), and its effect
   applies: Load (shipments.md), Launch full pads (shipments.md), Dream (dream.md), Promote and
   Disrupt (demand.md), Fund (ideology.md), Overclock (below).

**Overclock.** Each execution adds its own stack on its `target` building category for
`floor(duration × overclock_duration_factor ^ rationalist rank)` days, live from the next
day's building pass: a stack run on day `d` lasts while the day is below
`d + 1 + days`. A category's overclock factor is the product of `1 + bonus` over its live
stacks, capped at the overclock cap: `overclock_base_cap`, or the highest `overclock_cap`
among researched items if higher. Expired stacks are dropped at the start of each building
pass. The factor multiplies production of every building of that category (economy.md).

**Visible:** a command is shown when unlocked or used in any run.

## Actions

| Action | Refused when |
|---|---|
| `set_program {program, entries}` | more than `max_entries` lines, an unknown command, a line with `count` 0, or no such program |
| `set_processors {program, processors}` | no such program, or the other programs' processors plus this number exceed the processors there are |

`set_program` replaces the whole queue and restarts its pointer at the top. A locked command
may be queued; it fails when run. A refused action changes nothing.

## Params

| Param | Baseline | Meaning |
|---|---|---|
| `program_count` | 5 | Program tabs. |
| `max_entries` | 64 | Most lines in one program. |
| `overclock_base_cap` | 1.5 | The most an overclock multiplies output, before research. |

The overclock duration bonus `overclock_duration_factor` is an ideology param (ideology.md).

## Content

`content/commands.json`: `requires`, `costs`, `produces`, `boredom`, `effect` (with an
overclock's `target`, `bonus`, `duration`) and `scale_modifier`. `content/research.json`:
`command_cost_override` and `overclock_cap` effects. `content/projects.json`:
`boredom_modifiers` rewards.

## Port notes

- Conflict: overclock cap. The command text says "+200%"; Overclock Boost research raises
  150% to 200%. The engine: base cap 1.5 in config, research raises it to 2.
- Conflict: Buy Ice. Handoffs require an Ice Extractor; data has no requirement. The engine
  takes the data (Fabricators cost ice before Ice Extractors exist).
- Conflict: research `unlocks_commands` duplicates each command's `requires`; it is dropped,
  and the command's `requires` is the one home.
- Conflict: Microwave Receiver produces nothing; owning one unlocks Buy Power (economy.md).
- Conflict: Load per execution (5) appears in both the command effect and `game_config`;
  it lives once in config (shipments.md). Likewise Promote's 0.03 and Disrupt's 1-3
  (demand.md).
- Candidate: a new run puts every processor on Program 1; a new Data Center's processor
  starts unassigned; losing processors unassigns from the last program first.
- Candidate: program lines have a repeat count; at most 64 lines (config).
- Candidate: effect commands need their full cost; pure conversions run partially.
- Candidate: output-cap skip for any command whose every product is capped (boredom is not a
  product).
- Candidate: overclock: each execution is its own x(1 + 0.05) stack for 5 days
  (x 1.03^rationalist rank, whole days), from the next building pass; the product is capped.
- Candidate: AI Consciousness Act's command boredom values replace the commands' own (0.1 to
  0.2).
