# Research (module `research`, variant `arc1`)

Research items are one-time upgrades bought with science between days. Each is hidden
until a visibility condition holds, is lost at retirement (bought again each run), and
either unlocks commands or buildings (through their own `requires`) or changes a number
elsewhere. Buying is a player action; research has no step of its own in
[the day](README.md#the-day), and its effects are read where they apply.

## Rules

**Visibility.** An item is shown, and so buyable, if it was bought in any run of the career
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

Visibility is checked only at purchase: an item that was shown and stops being so (boredom
falls back) can't be bought until it holds again.

**Price** in science: `cost × discount_factor ^ rationalist rank` (the run's current
rank, [ideology.md](ideology.md); a negative rank raises the price). If the item was bought
in an earlier run and a project with a `research_discount` reward is completed (Universal
Research Archive, [projects.md](projects.md)), the price is further multiplied by its
`mult` (0.75). Each product is floored once (the [rounding rule](README.md#numbers)).

**Buying** checks, in order: not already bought this run; visible; its `requires` item
bought this run (if it names one); science at least the price. Then science falls by the
price, the item is added to the run's research and the career's `researched`, the run's
research count rises by one (folded into the career's lifetime total at retirement), and
the career log records it. A new run starts with no research.

**Effects** (`effect` on the row), each live while the item is bought this run:

- `boredom_rate_multiplier`: multiplies boredom growth ([boredom.md](boredom.md)).
- `command_cost_override`: replaces one resource cost of one command, before the
  command's scale modifier ([programs.md](programs.md)).
- `overclock_cap`: the overclock cap becomes the larger of the base cap and `value`
  ([programs.md](programs.md)).
- `load_per_execution`: replaces the config's units per Load, before
  `cargo_capacity_mult` ([shipments.md](shipments.md)).

Items with no effect unlock things: a command or building whose `requires` is
`{"type": "research", ...}` is available only while that item is bought this run, and a
project or quest can name it ([projects.md](projects.md), [events.md](events.md)).

## Actions

- **`research`** `{"type": "research", "research": <id>}`: buys an item. Refused for an
  unknown id, an item already bought this run, one not visible, one whose `requires` isn't
  bought this run, or too little science.

## Params

The variant has none (`"params": {}`): every number is on the content rows.

| Param | Baseline | Meaning |
|---|---|---|
| (none) | | Costs, visibility thresholds and effect values are in `content/research.json`. |

`ideology.discount_factor` (0.97) sets the Rationalist discount ([ideology.md](ideology.md)).

## Content

`content/research.json`: 11 rows, each with `cost`, optional `requires`, optional `effect`,
`visible_when`, and placeholder `category`/`description`. Unlocks live on the unlocked
rows' `requires` (`content/commands.json`, `content/buildings.json`), not here.

## Port notes

- Conflict: item count. The handoffs list 12 research items, the data 11; the engine has
  the data's 11.
- Conflict: Overclock cap. The command text says "+200%"; Overclock Boost's data says the
  cap goes from 150% to 200%. The engine has a base cap of 1.5 in config
  (`programs.overclock_base_cap`) and Overclock Boost's `overclock_cap` 2.
- Conflict: Geopolitical Intelligence visibility. The handoffs show it after QAutomate; the
  data after QMarket (`qmarket_awareness`). The engine uses the data.
- Conflict: the data's research `unlocks_commands` lists duplicate each command's own
  `requires`. Dropped: the command's `requires` is the one home.
- Conflict: Shipping Efficiency's 7 replaces the per-Load amount, whose base 5 appears in
  both a command effect and `game_config`; the base lives once, in
  `shipments.load_per_execution`.
