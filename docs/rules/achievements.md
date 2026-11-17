# Achievements (module `achievements`, variant `arc1`)

Achievements are optional accomplishments, earned once per career, each with a permanent
reward: a modifier or free buildings at the start of every run. Day conditions are checked
at step 13 of [the day](README.md#the-day), after the clamp and the events; shipment
conditions are checked at each launch.

## Rules

**Earning.** An achievement not yet earned is earned when its condition holds at a check;
it is recorded in the career (kept across retirements) and logged once.

**Day conditions** (end of each day):

| `condition` | Holds when |
|---|---|
| `produced_in_a_day` | today's production of `resource` (before caps) is at least `at_least` |
| `stockpile` | the resource on hand, after the day's clamp to its cap, is at least `at_least` |
| `consumed_in_a_day` | today's use of `resource` is strictly more than `more_than` |

Today's production and use are counted from the start of the day's step 1 to the check:
building output and upkeep, command costs and products, project drain and launch fuel all
count. Changes made by player actions between days fall outside any day's count.

**Shipment conditions** (at every launch, by a Launch command or the Launch action;
[shipments.md](shipments.md)):

| `condition` | Holds when |
|---|---|
| `shipment_revenue` | this launch paid at least `at_least` credits (after `shipment_credit_mult`) |
| `shipments_this_run` | this run's shipments, this one included, reach `at_least` |
| `shipment_demand` | the good's demand just before this launch's saturation is strictly above `more_than` |

**Rewards** (`reward` on the row), from the moment earned and in every later run:

- `modifier`: the modifier `key` is multiplied by `value` ([Modifiers](README.md#modifiers)).
- `bonus_buildings`: `count` free bonus buildings of `building` at the start of each run,
  owned and running, not counted in their price curve ([career.md](career.md)). The first
  is granted at the next run's start, not when earned.

## Actions

None: achievements are checked by the engine.

## Params

The variant has none (`"params": {}`): conditions and rewards are content.

| Param | Baseline | Meaning |
|---|---|---|
| (none) | | Everything is in `content/achievements.json`. |

## Content

`content/achievements.json`: six rows, three `miner` (Strip Mining, Silicon Valley,
Powerhouse) and three `trader` (First Profit, Bulk Shipper, Market Timer), each with one
`condition` and one `reward`; `description` and `reward_description` are display text.

## Port notes

No source conflicts or port candidates concern this module: the six achievements,
conditions and rewards are the data's.
