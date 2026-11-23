# Programs (module `programs`, variant `arc1`)

Program tabs, the processors that run them and the instruction set; what a command does when
run, and overclocks, are in [commands.md](commands.md). In [the day](README.md#the-day), expired overclocks drop before the building
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
`if` cost a processor step, with something to do or not:

| Line | JSON `op` | What it does |
|---|---|---|
| a command | `command` (`command`, `count`) | Runs it (below) `count` times, a step each. |
| `build <building>` | `build` (`building`, `up_to` null) | Buys one, as the Buy action would (economy.md); refused there, it fails. |
| `build <building> up to N` | `build` (`up_to` N) | The same while fewer than N are owned; with N owned it is **done**: the step, nothing else. |
| `buy land` | `buy_land` | Buys land, as the Buy Land action would; unaffordable, it fails. |
| a verb | `act` (`verb`, `target`, `tab`, `amount`) | *Verbs*, below. |
| `if <condition>` | `if` (`when`) | **Free.** Checked once, when the pointer reaches it. True: the pointer moves into its body, which then runs whole, whatever the condition does meanwhile. False: the pointer jumps past the body. |

**Blocks.** A line may be `indented`: it is in the body of the `if` above it. One level only:
an `if` is never indented (no nesting), there is no `else`, and an indented line must follow
an `if` or another indented line. Any other line may sit in a body.

**Verbs** (`act` lines; `state::Verb`). The target slot is typed by the verb; `tab` is a
program (0-based); the amount is `{"n": N}` or `"all"`; JSON `verb` is the snake_case name.
A verb with nothing to do is **done** (its step, nothing else); one that can't act fails.

| Verb | What it does |
|---|---|
| `launch pads <good \| all>` (`target` null: all) | Runs Launch Full Pads (its row's costs and boredom) on that good's pads; done with none that would launch. |
| `load pads <good \| all>` | Runs Load Launch Pads on that good's pads; done when none would take cargo. |
| `set pads <good> up to N` | While fewer than N pads carry it, an idle pad (empty, another good) switches, in pad order. Done at N; fails when none could. |
| `pause <building> [N \| all]` | N fewer running (1 with no amount; at most what runs), as Set Active. |
| `resume <building> [N \| all]` | N more running, at most all owned. |
| `research <item>` | Buys it, as Research does; done once researched. |
| `load <saved program> into <tab>` (`target` a name) | Loads the first loadout of that name, as `load_loadout` (into its own tab: from the top). |
| `assign processors <tab> N` (N may be 0) | As `set_processors` (`all`: every one not on another program). A program cut stops at its new count that day; one raised gains them the next. |
| `retire` | Fails unless retiring is allowed (career.md); else the run retires at the day's end, as the Retire action. |
| `fund <project> [N \| all]` | Sets its daily rate on each cost to N, as the Projects panel does, each at most its most a day (`all` or none: the most); 0 stops it. Done when so; fails when the project isn't open. |

**Conditions.** `<reading> <cmp> <number>`, or two such clauses joined by one `and` or `or`
(`state::Condition`: `first`, `rest`). `cmp` is `lt` (the editor's *below*) or `gt`
(*above*), both strict; the right side is always a number (`value`, an `Fx`) in the
reading's unit. One reading per thing (`state::Reading`, JSON `kind`):

| Reading | Reads | Resources |
|---|---|---|
| `percent` (`resource`) | Fill: `amount / cap × 100`, at most 100 (programs run before the day's clamp). | A capped resource (Energy, Regolith, Ice, Helium-3, Titanium, Circuit Boards, Propellant) or boredom (of its cap, boredom.md). |
| `amount` (`resource`) | The amount held. | An uncapped resource but land, boredom and processors (Credits, Science). |
| `free_processors` | Processors active buildings grant that no program holds (`processors_free`): above 0 exactly when `assign processors <tab> all` would add some. | |
| `free_land` | Land not built on. | |
| `idle_pads` | Pads with no cargo aboard (a cooling pad counts). | |

A `percent` of an uncapped resource, an `amount` of a capped one (or of land, boredom or
processors) is refused. A clause whose reading reads nothing (a cap of 0) is false. The
view's `readings` offer, in that order, each visible capped resource's fill, boredom's, each
visible uncapped resource's amount, free processors, free land and, once there is a pad,
idle pads (`view/programs.rs`): 13 in Act 1. The Resources panel's per-day number is the
day's rate (`Run.day_net`): production less demand, a brownout's unmet demand counted.

**Loadouts.** The library (`Career.library`, kept across retirements) holds up to
`library_slots` saved programs, each a display name (trimmed, 1 to `loadout_name_max`
characters, duplicates allowed) and its lines. Saving copies a program's lines into a new
slot or over one; loading replaces a program's lines with a slot's, as `set_program` does,
processors unchanged. A loaded line this run hasn't unlocked fails (an `if`: reads false)
when run: the run gates refuse only a line new to the program.

**Unlock.** Every line but a command needs the career flag `program_logic`: set (and logged)
the first day `logic_unlock_processors` processors work, as programs start to run. `if` also
needs its own career flag (draft, 049): `logic_if`, the Conditional Logic global project's
reward ([projects.md](projects.md)); content must set it. A loaded `if` the career hasn't
opened runs as false: it skips its body. The view's `if_open` says whether it is open.

**Running** (each day): programs in order; each running processor of a program takes one
step from the shared pointer. A step passes over free lines (`if`s) until it reaches a line
that costs it: that line runs once and the pointer advances
whatever the outcome (ran, skipped, failed or done): a command line's run count rises and,
at its `count`, the pointer moves on; any other line moves on at once. The pointer wraps from the last line to the first. A step that passes as
many free lines as the program has (one loop) idles.
An empty program does nothing. Each line's outcome today (`state::LineOutcome`; a false
`if`'s body `jumped` unless it ran earlier that day) is display only. A failed `build` or
`buy land` notes what it was short of, as a command does.

**Commands.** What one execution of a command does (*Running a command*), overclocks and
when a command is visible: [commands.md](commands.md).

## Actions

| Action | Refused when |
|---|---|
| `set_program {program, entries}` | more than `max_entries` lines; an unknown command, building or resource; a `count` or `up_to` of 0; a line other than a command before the unlock, or an `if` before its flag (unless already in the program); an indented line with no `if` or body line above it; an indented `if`; a reading of the wrong kind for its resource (*Conditions*); a verb with an unknown or missing target (`load`: a blank name; one no loadout has fails when run), a tab or amount it doesn't take or lacks, an amount of 0 (but `assign_processors` and `fund`), `set_pads ... all`; a switched-off verb (unless already in the program); or no such program |
| `set_processors {program, processors}` | no such program, or the other programs' processors plus this number exceed what the active Data Centers grant |
| `save_loadout {program, slot, name}` (`slot` null: a new slot) | no such program or slot; an empty program; a blank or overlong name; a new slot with the library full |
| `load_loadout {loadout, program}` | no such loadout or program; lines `set_program` refuses on their form (it skips the unlock checks) |
| `rename_loadout {loadout, name}` / `delete_loadout {loadout}` | no such loadout; a blank or overlong name (deleting moves later slots up) |

`set_program` replaces the whole queue and restarts its pointer at the top. A locked command
or building may be queued; it fails when run. A body line carries `"indented": true`.

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
| `fund_verb` | 1 | 0 to 1 | 1: the `fund <project>` verb is on; 0: off (a new line refused, a held one fails). |
<!-- /params -->

The overclock duration bonus `overclock_duration_factor` is an ideology param (ideology.md).
Content fields for commands: [commands.md](commands.md#content).

## Port notes

- Candidates (Godot port): the processor rules above. Commands' port notes are in
  [commands.md](commands.md#port-notes).
- Everything past commands is new (picks: docs/rulings.md).
