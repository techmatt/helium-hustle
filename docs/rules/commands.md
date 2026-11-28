# Commands and overclocks (module `programs`, variant `arc1`)

What one execution of a command does, and overclocks. Commands run only as program lines
([programs.md](programs.md): a step each, in the programs step of [the
day](README.md#the-day)); expired overclocks drop
earlier that day, before the building pass.
The module's params, `overclock_base_cap` among them, are in [programs.md](programs.md#params).

## Rules

**Running a command** (one execution):

1. **Locked** (its `requires` doesn't hold): fails.
2. **Flows:** its `costs`, with any researched `command_cost_override` replacing that cost,
   and its `produces`; a `scale_modifier` multiplies both (Buy Power uses `buy_power_mult`,
   which includes the career's bonus; [career.md](career.md)). Dream at a Humanist major
   rank keeps only its energy cost ([dream.md](dream.md)).
3. **Full cost or nothing** (no partial purchases): unless every cost is held in full, it
   fails: nothing paid, nothing made, no boredom; the line notes the scarcest input
   (programs.md). A repeat count is per execution, so `3x buy titanium` on 45 credits buys
   two and fails the third.
4. **Output-cap skip:** if any capped product would pass its cap (economy.md) with this
   execution's output, held + output > cap as the line runs (upkeep due later that day
   doesn't count), it is skipped: nothing paid, nothing made, no boredom. A run that lands
   exactly on the cap goes ahead.
5. It pays its costs and adds its products. (Loading still tops pads up partially: Load
   pays its full energy, then each pad takes what cargo there is, shipments.md.)
6. It adds its flat `boredom`; a completed project with
   `boredom_modifiers` replaces a command's boredom with its own value. A negative one
   (Theorize, the draft Distract) takes that much off, never below zero, × `m` of the
   command's `ideology` axis when it has one (ideology.md). Before Burnout a positive one is
   0 (boredom.md, *Burnout starts it*).
7. The command is marked used in the career (it stays visible in later runs), and its effect
   applies: Load (shipments.md), Launch full pads (shipments.md), Dream (dream.md), Promote and
   Disrupt (demand.md), Fund (ideology.md), Neutralize (Contest Claims: the named
   adversary's Attention to 0, [attention.md](attention.md)), Overclock (below).
8. A command with both `ideology` and `nudge` adds `nudge` to that axis (Dream, Theorize:
   0.25; ideology.md, *Nudges*): after its effect, so every tool's relief reads the rank
   from before the run.

**Only as program lines.** No action runs a command directly: a Commands card's Add to
Program appends it to the active program, open from the start, as are the loadout buttons.
The Programming research opens only the Programs view, the full editor
([research.md](research.md)).

**Overclock.** Each execution adds its own stack on its `target` building category for
its `duration` days, live from the same
day's pass of the buildings that take upkeep: a stack run on day `d` lasts while the day is
below `d + days`. A category's overclock factor is the product of `1 + bonus` over its live
stacks, capped at the overclock cap: `overclock_base_cap`, or the highest `overclock_cap`
among researched items if higher. The factor multiplies production of every building of that
category (economy.md). The view fills a description's `{days}` with those days ("4 days",
"1 day").

**Visible:** a command is shown when unlocked or used in any run.

**Act 1 gates** (`requires`, kinds in [economy.md](economy.md)): Buy Science (10 credits and
1 energy for 1 science, the only science before a Research Lab) opens with quest 1.1
Online's completion, and Launch Full Pads with 1.3 Full Load's (`quest_completed`); before
then pads launch by hand ([shipments.md](shipments.md)). Run Simulations (3 energy and 1 ice
for 6 credits, no boredom) needs research Liquid Cooling. Theorize (Rationalist's boredom
tool: 20 science for −3 boredom × m, 130's parity tuning) needs research Thought Experiments ([research.md](research.md)). The Buy commands for goods cost 1 energy and credits: Ice 5,
Regolith and Propellant 10, Science 10, Titanium 20. Disrupt Speculators needs research
Speculator Analysis; Contest Claims, Act 2's, the career flag `contest_claims` (set by the
silent `rivals_arrive`, [attention.md](attention.md)).

## Content

`content/commands.json`: `requires`, `costs`, `produces`, `boredom`, `effect` (with an
overclock's `target`, `bonus`, `duration`), `scale_modifier`, and optional `ideology` (the
axis whose boredom tool it is) and `nudge`. `content/research.json`:
`command_cost_override` and `overclock_cap` effects. `content/projects.json`:
`boredom_modifiers` rewards.

## Port notes

- Conflicts, the engine's pick: the overclock cap is 1.5, 2 with Overclock Boost (the text
  said "+200%"); Buy Ice needs nothing (handoffs wanted an Ice Extractor); research
  `unlocks_commands` is dropped for each command's `requires`; Microwave Receiver unlocks Buy
  Power ([economy.md](economy.md#port-notes)).
- Candidates (Godot port): a repeat count; the output-cap skip, on any product its output
  would pass (Matt); every command needs its full cost, conversions too (Matt); one x1.05 overclock stack per execution for 5 days; AI Consciousness Act's
  boredom values replace the commands'.
