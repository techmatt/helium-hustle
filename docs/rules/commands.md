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
3. **Output-cap skip:** if it produces anything and every product is at or over its cap
   (economy.md), it is skipped: nothing paid, nothing made, no boredom.
4. **Fraction:** `frac` = the least of `max(held, 0) / cost` over its costs, at most 1 (1 with
   no costs). If `frac` is 0, it fails. A command with an effect (Load, Launch, Dream,
   Overclock, Promote, Disrupt, Fund, Neutralize) fails unless `frac` is 1; a pure conversion (Idle,
   Cloud Compute, Buy *) runs at `frac`.
5. It pays `cost × frac` and adds `produces × frac`. Products may pass their cap until the
   day's clamp.
6. It adds its flat `boredom` (whole, not scaled by `frac`); a completed project with
   `boredom_modifiers` replaces a command's boredom with its own value. A negative one (the
   draft Distract) takes that much off, never below zero.
7. The command is marked used in the career (it stays visible in later runs), and its effect
   applies: Load (shipments.md), Launch full pads (shipments.md), Dream (dream.md), Promote and
   Disrupt (demand.md), Fund (ideology.md), Neutralize (Contest Claims: the named
   adversary's Attention to 0, [attention.md](attention.md)), Overclock (below).

**Only as program lines.** No action runs a command directly: a Commands card's Add to
Program appends it to the active program, open from the start, as are the loadout buttons.
The Programming research opens only the Programs view, the full editor
([research.md](research.md)).

**Overclock.** Each execution adds its own stack on its `target` building category for
`floor(duration × overclock_duration_factor ^ rationalist rank)` days, live from the same
day's pass of the buildings that take upkeep: a stack run on day `d` lasts while the day is
below `d + days`. A category's overclock factor is the product of `1 + bonus` over its live
stacks, capped at the overclock cap: `overclock_base_cap`, or the highest `overclock_cap`
among researched items if higher. The factor multiplies production of every building of that
category (economy.md). `overclock_duration_factor` is an ideology param (ideology.md). The
view fills a description's `{days}` with those days at the current rank ("4 days", "1 day").

**Visible:** a command is shown when unlocked or used in any run.

**Act 1 gates** (`requires`, kinds in [economy.md](economy.md)): Buy Science (10 credits and
1 energy for 1 science, the only science before a Research Lab) opens with quest 1.1
Online's completion, and Launch Full Pads with 1.4 Science's (`quest_completed`); before
then pads launch by hand ([shipments.md](shipments.md)). Disrupt Speculators needs research
Speculator Analysis; Contest Claims, Act 2's, the career flag `contest_claims` (set by the
silent `rivals_arrive`, [attention.md](attention.md)).

## Content

`content/commands.json`: `requires`, `costs`, `produces`, `boredom`, `effect` (with an
overclock's `target`, `bonus`, `duration`) and `scale_modifier`. `content/research.json`:
`command_cost_override` and `overclock_cap` effects. `content/projects.json`:
`boredom_modifiers` rewards.

## Port notes

- Conflicts, the engine's pick: the overclock cap is 1.5, 2 with Overclock Boost (the text
  said "+200%"); Buy Ice needs nothing (handoffs wanted an Ice Extractor); research
  `unlocks_commands` is dropped for each command's `requires`; Microwave Receiver unlocks Buy
  Power ([economy.md](economy.md#port-notes)).
- Candidates (Godot port): a repeat count; effect commands need their full cost; the
  output-cap skip; one x1.05 overclock stack per execution for 5 days; AI Consciousness Act's
  boredom values replace the commands'.
