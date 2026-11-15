# Events (module `events`, variant `arc1`)

Events are the quest chain ("Breadcrumbs", the `story` rows, in file order) and one-off
events (the `ongoing` rows). Each has a trigger, a condition and unlock effects that open
buildings, panels, projects or flags. Events are checked at step 10 of
[the day](README.md#the-day), after the clamp; their unlocks are re-applied at the start
of every run.

## Rules

**Triggers** (`trigger` on the row):

| `trigger` | Has happened when |
|---|---|
| `game_start` | always, if `run_number` is null; else only in that run. A quest ignores `run_number`. |
| `quest_complete` | the quest is completed (in any run) |
| `boredom_phase` | the run's boredom phase is at least `phase` ([boredom.md](boredom.md)) |

**Conditions** (`condition` on the row, and on each sub-objective):

| `condition` | Holds when |
|---|---|
| `building_owned` | at least `count` owned this run (bonus buildings included) |
| `resource_cumulative` | this run's production of the resource, before caps, reaches `amount` (starting resources don't count) |
| `shipment_completed` | this run's shipments reach `count` |
| `research_completed` | the item is bought this run |
| `immediate` | always |
| `never` | never |
| `all_of` | every sub-objective is latched (below) |
| `ideology_rank_any` | some axis's current rank this run is at least `rank` |
| `persistent_project_completed_any` | any Long-Term project is completed |
| `days_survived` | the run's days, today included, reach `days` |
| `credits_earned` | credits produced this run reach `amount` |

**The quest chain.** The active quest is the first `story` row in file order that is not
completed and whose trigger has happened; the last quest's condition is `never`, so once
the chain starts there is always one. Each check:

1. For the active quest, if its condition is `all_of`, each sub-objective whose condition
   holds now is latched in the career (`quest:sub_objective`) and stays met for good,
   across retirements. Sub-objectives latch only while their quest is active.
2. If the active quest's condition holds, it completes: recorded in the career, fired
   (below) and logged. The next quest becomes active and is checked at once, so several
   can complete the same day.
3. The chain stops at the first active quest whose condition doesn't hold.

Quests complete once per career. Their conditions read this run, so progress that isn't
latched (cumulative He-3, shipments, Data Centers) starts over after a retirement.

**One-off events.** After the quests, each `ongoing` row in file order that has not fired
this run fires if its trigger has happened and its condition holds. Each fires at most once
per run, and again in the next run (its fired mark is per run). One triggered by a quest
completed earlier in the same check can fire that same day.

**Firing** records the event as fired this run (research `event_completed` reads it,
[research.md](research.md)) and as seen by the career, then applies its unlocks:

| Unlock | Effect |
|---|---|
| `enable_building` | the building may be bought this run. A building named by any event's `enable_building` is locked until one enables it ([economy.md](economy.md)). |
| `enable_nav_panel` | the panel is shown this run. Only `retirement` gates a rule (the Retire action, [career.md](career.md)); others are display. |
| `enable_project` | an `event_unlocked` project may be funded this run ([projects.md](projects.md)) |
| `set_flag` | sets a career flag |

**Run start.** Every event the career has seen (quests and one-offs alike) re-applies its
unlocks at the start of each run, so an unlocked building, panel or project stays unlocked
after a retirement from day 0. One-off events still fire again during the run once their
trigger and condition hold.

`category` decides quest or one-off. `silent`, `title`, `summary` and `body` are text the
engine doesn't read (body placeholders like `{rate}` are left as is).

## Actions

None: events are checked each day.

## Params

The variant has none (`"params": {}`): triggers, conditions and unlocks are content.

| Param | Baseline | Meaning |
|---|---|---|
| (none) | | Everything is in `content/events.json`. |

## Content

`content/events.json`: 14 rows, seven `story` (Q1 Boot Sequence to Q7 Signal Detected)
and seven `ongoing` (the Recreation Dome unlock, Political Currents, propellant discovery,
boredom phases 2 to 5).

## Port notes

- Conflict: QShipment unlocks. The handoffs have it unlock Foundation Grant, Retirement and
  Projects; the data has Retirement only, with QMarket unlocking Foundation Grant and the
  Projects panel. The engine uses the data.
- Conflict: Dream's falling effectiveness. The boredom phase 3 to 5 events' text says Dream
  drops to 20/15/10%; the ruling is a flat Dream ([dream.md](dream.md)), so these events
  unlock nothing and their text stays as placeholder.
- Conflict: Lunar Cartography and Deep Core Survey are `event_unlocked` projects, but no
  event enables them: unreachable (candidate).
- Conflict: Godot's milestones (`game_config.milestones`) were removed per the handoffs;
  not ported.
- Candidate: quests ignore a `game_start` run filter, so the chain picks up where the
  career left it (Q1's `run_number` 1 would otherwise strand the chain after run 1).
- Candidate: a `quest_active` gate (projects, building `requires`) holds while the quest is
  active or after it completes.
- Candidate: one-off events fire once per run; `shipment_completed` counts this run's
  shipments.
