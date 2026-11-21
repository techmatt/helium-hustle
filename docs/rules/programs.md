# Programs (module `programs`, variant `arc1`)

Program tabs (queues of instructions), the processors that run them, the instruction set
(commands, `build`, `buy land`, verbs, `if` blocks, `wait until`), command execution and overclocks. In [the day](README.md#the-day), expired overclocks drop before the building
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

**Instructions** (`state::Op`; flat, `<verb> <target> [amount]`: docs/rulings.md). All but
`if` and `wait until` cost a processor step:

| Line | JSON `op` | What it does |
|---|---|---|
| a command | `command` (`command`, `count`) | Runs the command (below) `count` times, a step each, before moving on. |
| `build <building>` | `build` (`building`, `up_to` null) | Buys one, as the Buy action would (economy.md); refused there, it fails. |
| `build <building> up to N` | `build` (`up_to` N) | The same while fewer than N are owned; with N owned it is **free** and passed over. |
| `buy land` | `buy_land` | Buys land, as the Buy Land action would; unaffordable, it fails. |
| a verb | `act` (`verb`, `target`, `tab`, `amount`) | *Verbs*, below. |
| `if <condition>` | `if` (`when`) | **Free.** Checked once, when the pointer reaches it. True: the pointer moves into its body, which then runs whole, whatever the condition does meanwhile. False: the pointer jumps past the body. |
| `wait until <condition>` | `wait_until` (`when`) | **Free.** True: passed over. False: the pointer stays, and every processor of the program idles that step. |

**Blocks.** A line may be `indented`: it is in the body of the `if` above it. One level only:
an `if` is never indented (no nesting), there is no `else`, and an indented line must follow
an `if` or another indented line. Any other line may sit in a body, `wait until` included.

**Verbs** (`act` lines; `state::Verb`). The target slot is typed by the verb; `tab` is a
program (0-based); the amount is `{"n": N}` or `"all"`; JSON `verb` is the snake_case name
(`launch_pads`, `load_program`, `assign_processors`, ...). A verb with nothing to do is
**done** and free, as a finished `build ... up to N`; one that can't act fails. The pad
verbs, `retire` and `load` always take their step.

| Verb | What it does |
|---|---|
| `launch pads <good \| all>` (`target` null: all) | Runs Launch Full Pads (its row's costs and boredom) on that good's pads, as the command line would. |
| `load pads <good \| all>` | Runs Load Launch Pads on that good's pads, likewise. |
| `set pads <good> up to N` | While fewer than N pads carry the good, an idle pad (empty, another good) switches to it, in pad order. Done at N; fails when none could switch. |
| `pause <building> [N \| all]` | N fewer running (1 with no amount; at most what runs), as Set Active. |
| `resume <building> [N \| all]` | N more running, at most what is owned. |
| `research <item>` | Buys it, as the Research action; done once researched. |
| `load <saved program> into <tab>` (`target` a name) | Loads the first loadout of that name, as `load_loadout`. Into its own tab, the new lines start at the top. |
| `assign processors <tab> N` (N may be 0) | As `set_processors` (`all`: every one not on another program). A program cut stops at its new count that day; one raised gains them the next. |
| `retire` | Fails unless retiring is allowed (career.md); else the run retires at the day's end, as the Retire action (logged `retired`, its summary in `last_run`). |

**Conditions.** `<reading> <cmp> <right side>`, or two such clauses joined by one `and` or
`or` (`state::Condition`: `first`, `rest`). `cmp` is `lt`, `le`, `gt`, `ge` or `eq`; the right
side is a number (`value`, an `Fx`) or, with `against`, another reading times that number
(`Solar Panel count < Excavator count × 2`): one multiplier, no other math. Readings
(`state::Reading`, JSON `kind`):

| Reading | Reads |
|---|---|
| `amount` (`resource`) | The amount held. Land's is the land free; boredom is a resource too. |
| `percent` (`resource`) | `amount / cap × 100`; a resource with no cap reads nothing. |
| `rate` (`resource`) | Production less demand over the last full day (`Run.day_net`): what the tick did to the stock before the clamp, less the demand it left unmet (`today.unmet`: on each input a building, command or project ran short of, what it wanted less what it got). So a brownout reads the shortfall; a capped resource its uncapped rate; nothing done by hand counts; 0 on day 1. |
| `days_to_cap` (`resource`) | `(cap − amount) / rate`; 0 at the cap; nothing with no cap or a rate of 0 or less. |
| `count` (`building`) | How many are owned. |
| `next_cost` (`building`) | The next one's price in credits (no credits cost: nothing). |
| `land_price` | The land price. |
| `day` | The run's day. |
| `every` (`days`) | 1 on a day that is a multiple of N, else 0 (the editor's `every N days`, `= 1`). |
| `pads_ready` (`good`, null: all) | Pads of the good holding a full load that can launch now (shipments.md). |
| `cargo` (`good`, null: all) | Cargo aboard the good's pads. |
| `days_since_launch` | Days since this run's last launch (none yet: the run's day). |
| `demand` (`good`) | The good's demand today; reads nothing until exact demand is revealed (Market Awareness, research.md). |

A clause whose reading or right side reads nothing is false. The view's `readings` offer only
what the player can see (visible resources and buildings; land and processors: amount only;
boredom: no rate; pad readings once there is a pad; demand once revealed); its `verbs` give
each verb's slot choices now.

The Resources panel's per-day number (`ResourceView.per_day`) is `rate` for the next day.

**Loadouts.** The library (`Career.library`, kept across retirements) holds up to
`library_slots` saved programs, each a display name (trimmed, 1 to `loadout_name_max`
characters, duplicates allowed) and its lines. Saving copies a program's lines into a new
slot or over one; loading replaces a program's lines with a slot's, as `set_program` does,
processors unchanged. A loaded line this run hasn't unlocked is not refused: it fails, or
reads nothing, when run; `set_program`'s run gates (the logic unlock, demand revealed) refuse
only a line new to the program. Programs are empty at each new run; open from the start.

**Unlock.** Every line but a command needs the career flag `program_logic`: set the first
day `logic_unlock_processors` (2) processors work, checked as programs start to run, logged
(`unlock`), and kept across retirements like any career flag (career.md).

**Running** (each day): programs in order; each running processor of a program takes one
step from the shared pointer. A step passes over free lines (an `if`, a `wait until` that
holds, a `build ... up to N` or a verb that is done) until it reaches a line that costs it:
that line runs once and the pointer advances whatever the outcome (ran, skipped or failed): a
command line's run count rises and, at its `count`, the pointer moves on; any other line
moves on at once. The pointer wraps from the last line to the first. A step that passes as
many free lines as the program has (one loop) idles, as does one stopped at a `wait until`.
An empty program does nothing. Each line's outcome today (the last time the pointer met it:
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
| `set_program {program, entries}` | more than `max_entries` lines; an unknown command, building or resource; a `count` or `up_to` of 0; a line other than a command before the unlock (unless already in the program); an indented line with no `if` or body line above it; an indented `if`; a `demand` reading of a good not traded, or before demand is revealed (unless already in the program); a verb with an unknown or missing target (`load`: a blank name; one no loadout has fails when run), a tab or amount it doesn't take or lacks, an amount of 0 (but `assign_processors`), `set_pads ... all`; `every 0`; or no such program |
| `set_processors {program, processors}` | no such program, or the other programs' processors plus this number exceed what the active Data Centers grant |
| `save_loadout {program, slot, name}` (`slot` null: a new slot) | no such program or slot; an empty program; a blank or overlong name; a new slot with the library full |
| `load_loadout {loadout, program}` | no such loadout or program; lines `set_program` refuses on their form (it skips the unlock and reveal checks) |
| `rename_loadout {loadout, name}` / `delete_loadout {loadout}` | no such loadout; a blank or overlong name (deleting moves later slots up) |

`set_program` replaces the whole queue and restarts its pointer at the top. A locked command
or building may be queued; it fails when run. A line is JSON `{"op": "command", "command":
"idle", "count": 1}`, with `"indented": true` on a body line. A scenario's starting programs
hold command lines only. A refused action changes nothing.

## Params

<!-- params:programs -->
| Param | Baseline | Range | Meaning |
|---|---|---|---|
| `program_count` | 5 | 1 to 100 | Program tabs. |
| `max_entries` | 128 | 1 to 10000 | Most lines in one program. |
| `overclock_base_cap` | 1.5 | 1 to 100 | The most an overclock multiplies output, before research. |
| `logic_unlock_processors` | 2 | 0 to 1000 | Working processors that first open every line but plain commands (a career first). |
| `library_slots` | 12 | 0 to 1000 | Saved programs the loadout library holds (career state). |
| `loadout_name_max` | 40 | 1 to 1000 | Most characters in a loadout's name. |
<!-- /params -->

The overclock duration bonus `overclock_duration_factor` is an ideology param (ideology.md).

## Content

`content/commands.json`: `requires`, `costs`, `produces`, `boredom`, `effect` (with an
overclock's `target`, `bonus`, `duration`) and `scale_modifier`. `content/research.json`:
`command_cost_override` and `overclock_cap` effects. `content/projects.json`:
`boredom_modifiers` rewards.

## Port notes

- Conflicts, the engine's pick: the overclock cap is 1.5, 2 with Overclock Boost (the text
  said "+200%"); Buy Ice needs nothing (handoffs wanted an Ice Extractor); research
  `unlocks_commands` is dropped for each command's `requires`; Microwave Receiver unlocks Buy
  Power ([economy.md](economy.md#port-notes)); Load's 5, Promote's 0.03 and Disrupt's 1-3
  live once in config.
- Candidates (Godot port): processors on Program 1 at a new run, unassigned when gained, cut
  from the last program; a repeat count, 64 lines (128 since 037, so a build order reaches the
  late buildings); effect commands need their full cost; the
  output-cap skip; one x1.05 overclock stack per execution for 5 days; AI Consciousness Act's
  boredom values replace the commands'.
- Candidates (not in the Godot build): everything past commands (`build ... up to N` buys one
  a pass; the unlock is 2 working processors; 035's `rate` and loadout picks, docs/rulings.md);
  037's verbs (docs/rulings.md, the flat template).
