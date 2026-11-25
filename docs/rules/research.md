# Research (module `research`, variant `arc1`)

Research items are one-time upgrades bought with science between days. Each is hidden
until a visibility condition holds, is lost at retirement (bought again each run), and
either unlocks commands or buildings (through their own `requires`) or changes a number
elsewhere. Buying is a player action; research has no step of its own in
[the day](README.md#the-day), and its effects are read where they apply.

## Rules

**Visibility.** An item with a `career_flag` effect is never shown once its flag is set.
Otherwise an item is shown, and so buyable, if it was bought in any run of the career
(`career.researched`), or its `visible_when` holds now:

| `visible_when` | Holds when |
|---|---|
| `always` | always |
| `event_completed` | that event fired this run ([events.md](events.md)) |
| `boredom_above` | boredom is strictly above `threshold` |
| `research_purchased` | that item was bought this run |
| `building_count` | at least `count` of the building owned this run (bonus buildings included) |
| `shipments_completed` | career shipments in finished runs plus this run's reach `count` |
| `quest_completed` | the quest is completed (in any run) |
| `flag` | the career flag `flag` is set |
| `all_of` | every condition in `all` holds |

Visibility is checked only at purchase: an item that was shown and stops being so (boredom
falls back) can't be bought until it holds again.

**Price** in science: `cost × discount_factor ^ rationalist rank` (the run's current
rank, [ideology.md](ideology.md); a negative rank raises the price). If the item was bought
in an earlier run and Rationalist is at its major rank, the price is 0 (the major effect,
[ideology.md](ideology.md#rules)); else, if a project with a `research_discount` reward is completed (Universal
Research Archive, [projects.md](projects.md)), the price is further multiplied by its
`mult` (0.75). Each product is floored once (the [rounding rule](README.md#numbers)).

**Buying** checks, in order: the `research` panel is enabled this run (quest 1.1 Online's completion opens
it, [events.md](events.md); the nav's Research tile shows with it); not already bought this
run; visible; its `requires` item
bought this run (if it names one); science at least the price. Then science falls by the
price, the item is added to the run's research and the career's `researched`, the run's
research count rises by one (folded into the career's lifetime total at retirement), and
the career log records it. A new run starts with no research, but for items a live
`starting_research` project reward names (the Blueprint Archive: both Blueprints), which
count as bought from day 0 of every run while the project counts as completed ([projects.md](projects.md)).

**Effects** (`effect` on the row), each live while the item is bought this run:

- `boredom_rate_multiplier`: multiplies boredom growth ([boredom.md](boredom.md)).
- `command_cost_override`: replaces one resource cost of one command, before the
  command's scale modifier ([commands.md](commands.md)).
- `overclock_cap`: the overclock cap becomes the larger of the base cap and `value`
  ([commands.md](commands.md)).
- `load_per_execution`: replaces the config's units per Load, before
  `cargo_capacity_mult` ([shipments.md](shipments.md)).
- `reveal_demand` (Market Awareness): the view shows each good's exact demand instead of
  a tier (display only, [demand.md](demand.md)).
- `modifier` (`key`, `value`): multiplies the modifier `key` by `value`, one more source in
  the shared modifier framework ([README.md](README.md), *Modifiers*). Regolith Processing
  (100 science, category *Extraction*, visible at 2 Data Centers): `excavator_output_mult`
  1.5, so Regolith Excavators make 50% more ([economy.md](economy.md)).
- `program_readings` (Process Telemetry, 50 science, visible once a Research Lab is built, its own category
  *Program Analysis*): programs' `energy per day` and `days since a purchase` readings
  open ([programs.md](programs.md)).

- `career_flag` (Programming, 200 science, visible once quest 1.4 Full Load is done, category
  *Programs*): buying it sets the career flag `flag` for good (`programming`); retirements
  and a timeline's end keep it, and the item never shows again. The `programming` flag shows
  the Programs nav tile (`nav.json`, a `flag` condition) and with it the Programs view (the
  text and block editors, the all-instructions panel); no rule reads it, and the right
  column's program, Add to Program and loadouts are open without it. A state
  saved before revision 29 that loads past 1.4 gets the flag (`Career.flag_research`).

A purchase of research (as of a building or land) sets the run's `last_purchase`.

Items with no effect unlock things: a command or building whose `requires` is
`{"type": "research", ...}` is available only while that item is bought this run, and a
project or quest can name it ([projects.md](projects.md), [events.md](events.md)).
Speculator Analysis has no effect and nothing names it: buying it changes nothing (its
description's burst forecast is not built).

## Actions

- **`research`** `{"type": "research", "research": <id>}`: buys an item. Refused for an
  unknown id, before the `research` panel is enabled, for an item already bought this run, one not visible, one whose `requires` isn't
  bought this run, or too little science.

## Params

<!-- params:research -->
The `arc1` variant has no params (`"params": {}`).
<!-- /params -->

Every number is on the content rows: costs, visibility thresholds and effect values are in
`content/research.json`.

`ideology.discount_factor` (0.97) sets the Rationalist discount ([ideology.md](ideology.md)).

## Content

`content/research.json`: 17 rows, each with `cost`, optional `requires`, optional `effect`,
`visible_when`, a `category` and a placeholder `description`. The *Blueprints* category comes
first: Basic Blueprints (10 science, always visible; quest 1.2 buys it) unlocks the Regolith
Excavator, Smelter, Launch Pad and Storage Depot, and Advanced Blueprints (20, visible once
quest 1.3 First Shipment is completed) the Battery, Refinery and Fabricator, through those
buildings' `requires` ([economy.md](economy.md)). Programming (category *Programs*) shows
from 1.4 Full Load until bought, Regolith Processing (category *Extraction*) from 2 Data
Centers (when a Research Lab can be built). Every other item but Novelty Simulation has the
`flag` `research_lab` in its `visible_when` (alone, or inside an `all_of` with its old
condition): the flag the silent `research_lab_built` event sets once a Research Lab is owned
(beat 1.8, no quest; [events.md](events.md)), so before a Lab the panel shows only those.
Speculator Analysis also needs the `speculators_seen` flag (speculators have arrived). A
category is a cluster:
buying every item of one of 2 or more items in a run (so not the one-item Political
Influence or Extraction), outside the categories its `exclude` names (1.b excludes Blueprints), meets side
quest 1.b's `research_cluster`
([events.md](events.md)). Dream Protocols is named Daydreaming; Novelty Simulation (draft,
[timeline.md](timeline.md)) stays in Self-Maintenance, so that cluster can't be finished
before quest 1.11 shows it;
Geopolitical Intelligence shows once a Research Lab is built; Market Awareness only reveals demand
(Disrupt Speculators opens on side quest 1.a, [demand.md](demand.md)). The `research` panel
id is fixed in code (`research::RESEARCH_PANEL`). Unlocks live on the unlocked rows'
`requires` (`content/commands.json`, `content/buildings.json`), not here.

## Port notes

- Conflict: item count. The handoffs list 12 research items, the data 11; the engine has
  the data's 11.
- Conflict: Overclock cap. The Godot command text said "+200%" (rewritten to the engine's caps); Overclock Boost's data says the
  cap goes from 150% to 200%. The engine has a base cap of 1.5 in config
  (`programs.overclock_base_cap`) and Overclock Boost's `overclock_cap` 2.
- Changed (049): Geopolitical Intelligence shows once a Research Lab is built (the handoffs had
  QAutomate, the data QMarket, a quest no longer in the line).
- Conflict: the data's research `unlocks_commands` lists duplicate each command's own
  `requires`. Dropped: the command's `requires` is the one home.
- Conflict: Shipping Efficiency's 7 replaces the per-Load amount, whose base 5 appears in
  both a command effect and `game_config`; the base lives once, in
  `shipments.load_per_execution`.
