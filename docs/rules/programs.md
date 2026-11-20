# Programs (module `programs`, variant `arc1`)

Program tabs (queues of instructions), the processors that run them, the instruction set
(commands, `build`, `buy land`, `if` blocks, `wait until`), command execution and overclocks. In [the day](README.md#the-day), expired overclocks drop before the building
pass, and programs run after the demand update and the upkeep-free buildings' production,
before every building that takes upkeep (programs are paid first), projects and shipments.

## Rules

**Programs.** There are `program_count` programs (0-based in actions). Each holds a queue of
lines (one instruction each, below), a shared pointer (line, and runs done of a command line)
and a number of assigned processors. A new run gives every program
an empty queue and puts all processors on the first program.

**Processors.** All programs together may hold at most the processors the *active* Data
Centers grant (1 each, paid or not). When that drops (a Data Center sold or set inactive),
assignments are cut from the last program first. A processor gained mid-run (a new Data
Center) starts unassigned. Each day at most the processors resource (economy.md: 1 per
*working*, upkeep-paid Data Center) runs: when fewer work than are assigned, the shortfall
comes off the last programs first for that day, and their assignments stay.

**Instructions.** A line is one of these (`state::Op`; the set is built to grow). Only the
first three cost a processor step:

| Line | JSON `op` | What it does |
|---|---|---|
| a command | `command` (`command`, `count`) | Runs the command (below) `count` times, a step each, before moving on. |
| `build <building>` | `build` (`building`, `up_to` null) | Buys one, as the Buy action would (economy.md); refused there, it fails. |
| `build <building> up to N` | `build` (`up_to` N) | The same while fewer than N are owned; with N owned it is **free** and passed over. |
| `buy land` | `buy_land` | Buys land, as the Buy Land action would; unaffordable, it fails. |
| `if <condition>` | `if` (`when`) | **Free.** Checked once, when the pointer reaches it. True: the pointer moves into its body, which then runs whole, whatever the condition does meanwhile. False: the pointer jumps past the body. |
| `wait until <condition>` | `wait_until` (`when`) | **Free.** True: passed over. False: the pointer stays, and every processor of the program idles that step. |

**Blocks.** A line may be `indented`: it is in the body of the `if` above it. One level only:
an `if` is never indented (no nesting), there is no `else`, and an indented line must follow
an `if` or another indented line. Any other line may sit in a body, `wait until` included.

**Conditions.** `<reading> <cmp> <number>`, or two such clauses joined by one `and` or `or`
(`state::Condition`: `first`, `rest`). `cmp` is `lt`, `le`, `gt`, `ge` or `eq`; the number is
an `Fx`. Readings (`state::Reading`, JSON `kind`):

| Reading | Reads |
|---|---|
| `amount` (`resource`) | The amount held. Land's is the land free; boredom is a resource too. |
| `percent` (`resource`) | `amount / cap × 100`; a resource with no cap reads nothing. |
| `rate` (`resource`) | What the last full day's tick made less what it used (buildings, programs' commands, projects; not purchases, pad loading, the cap's cut or anything done by hand between days), kept in `Run.day_net` at each day's end. Actual flows: a building short of upkeep uses only what it got, so in a steady brownout (Energy at 0) Energy's rate reads 0, and is negative only while the stock drains. 0 on a run's first day. |
| `count` (`building`) | How many are owned. |
| `day` | The run's day. |
| `pads_ready` | Pads holding a full load that can launch now (shipments.md). |
| `demand` (`good`) | The good's demand today; reads nothing until exact demand is revealed (Market Awareness, research.md). |

A clause whose reading reads nothing is false. The view offers (`readings`, with display
labels and current values) only what the player can see: amount, percent (capped ones) and
rate per visible resource (land and processors: amount only; boredom: no rate), a count per
visible building, the day, pads ready once there is a pad, demand once revealed. No adversary
reading (speculators, rivals) is offered yet.

**Unlock.** Every line but a command needs the career flag `program_logic`: set the first
day `logic_unlock_processors` (2) processors work, checked as programs start to run, logged
(`unlock`), and kept across retirements like any career flag (career.md).

**Running** (each day): programs in order; each running processor of a program takes one
step from the shared pointer. A step passes over free lines (an `if`, a `wait until` that
holds, a `build ... up to N` that is done) until it reaches a line that costs it: that line
runs once and the pointer advances whatever the outcome (ran, skipped or failed): a command
line's run count rises and, at its `count`, the pointer moves on; a `build` or `buy land`
moves on at once. The pointer wraps from the last line to the first. A step that passes as
many free lines as the program has (one loop) idles, as does one stopped at a `wait until`.
A program with no lines does nothing. So a program with 3 processors runs three consecutive
costed steps of its queue a day. Each line's outcome today (the last time the pointer met it:
`ran`, `skipped`, `failed`, `true`, `false`, `jumped` for a false `if`'s body unless it ran
earlier that day, `waiting`, `done`) is kept for display (the UI colors the line); no rule reads it. A failed `build` or
`buy land` notes what it was short of, as a command does.

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
`floor(duration × overclock_duration_factor ^ rationalist rank)` days, live from the same
day's pass of the buildings that take upkeep (every overclock target, extraction and
processing, does): a stack run on day `d` lasts while the day is below `d + days`. A category's overclock factor is the product of `1 + bonus` over its live
stacks, capped at the overclock cap: `overclock_base_cap`, or the highest `overclock_cap`
among researched items if higher. Expired stacks are dropped at the start of each building
pass. The factor multiplies production of every building of that category (economy.md).

**Visible:** a command is shown when unlocked or used in any run.

## Actions

| Action | Refused when |
|---|---|
| `set_program {program, entries}` | more than `max_entries` lines; an unknown command, building or resource; a `count` or `up_to` of 0; a line other than a command before the unlock; an indented line with no `if` or body line above it; an indented `if`; a `demand` reading of a good not traded or before demand is revealed; or no such program |
| `set_processors {program, processors}` | no such program, or the other programs' processors plus this number exceed what the active Data Centers grant |

`set_program` replaces the whole queue and restarts its pointer at the top. A locked command
or building may be queued; it fails when run. A line is JSON `{"op": "command", "command":
"idle", "count": 1}`, with `"indented": true` on a body line. A scenario's starting programs
hold command lines only. A refused action changes nothing.

## Params

| Param | Baseline | Meaning |
|---|---|---|
| `program_count` | 5 | Program tabs. |
| `max_entries` | 64 | Most lines in one program. |
| `overclock_base_cap` | 1.5 | The most an overclock multiplies output, before research. |
| `logic_unlock_processors` | 2 | Working processors that first open `if`, `wait until`, `build` and `buy land`. |

The overclock duration bonus `overclock_duration_factor` is an ideology param (ideology.md).

## Content

`content/commands.json`: `requires`, `costs`, `produces`, `boredom`, `effect` (with an
overclock's `target`, `bonus`, `duration`) and `scale_modifier`. `content/research.json`:
`command_cost_override` and `overclock_cap` effects. `content/projects.json`:
`boredom_modifiers` rewards.

## Port notes

- Conflict: overclock cap. The Godot command text said "+200%"; Overclock Boost research raises
  150% to 200%. The engine: base cap 1.5 in config, research raises it to 2 (the text now
  says so: up to +50%, +100% with Overclock Boost).
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
  (x 1.03^rationalist rank, whole days), from the same day's building pass; the product is
  capped.
- Candidate (not in the Godot build): the instruction set beyond commands. `build ... up to
  N` buys one a pass, not N in a row; `rate` is the last day's flows, not the view's forecast;
  the unlock trigger is 2 working processors.
- Candidate: AI Consciousness Act's command boredom values replace the commands' own (0.1 to
  0.2).
