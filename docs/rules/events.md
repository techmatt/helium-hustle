# Events (module `events`, variant `arc1`)

Events are the quest chain ("Breadcrumbs", the `story` rows, in file order), side quests
(the `side` rows) and one-off events (the `ongoing` rows). Each has a trigger, a condition
and unlock effects that open buildings, panels, projects or flags. A quest may carry the
design doc's `beat` id (`1.1` to `1.11`, `1.a` to `1.c`; [../arc.md](../arc.md) maps beats to
quests); no rule reads it. Events are checked at step 12 of
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
| `ideology_rank` | the career's best rank on `axis` is at least `rank` |
| `project_completed_any` | any project of `scope` (`run`, `timeline`, `forever`) is completed |
| `project_enabled` | an event's `enable_project` enabled the project this run |
| `quest_active` | the quest is active now or completed in any run |
| `flag_set` | the career flag is set |
| `days_survived` | the run's days, today included, reach `days` |
| `credits_earned` | credits produced this run reach `amount` |
| `retirements` | the career's retirements, forced or chosen, reach `count` |
| `adversary_present` | the adversary is present now ([attention.md](attention.md)) |
| `launch_credits` | one launch this run paid at least `amount` credits ([shipments.md](shipments.md)) |
| `launch_cargo` | one launch this run carried at least `amount` of `resource` |
| `launch_full` | one launch this run carried a full pad (`pad_cargo_capacity`) of `resource` |
| `research_cluster` | every research item of some category, not one `exclude` names, with at least `min_items` items is bought this run |
| `project_completed` | the project is completed ([projects.md](projects.md)) |
| `quiet_days`, `earth_science`, `earth_tech`, `timelines_ended`, `swarm_repelled`, `vault_holds`, `super_timeline`, `victory` | the later acts' (draft): [timeline.md](timeline.md). `quiet_days` is unused (it was Signal Detected's, now an achievement candidate) |

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

**Quest start.** The first time a chain quest becomes active in the career (as the check
reaches it), its `on_start` grants are added to the run's stock, once: a retirement doesn't
repeat them (career `started_quests`). 1.3 First Shipment's 20 Propellant is the one.
While a chain quest naming `pad_goods` is active, a pad may be set only to those goods (1.3:
Titanium; [shipments.md](shipments.md)).

**Side quests.** After the chain, each `side` row in file order that is not completed
completes if its trigger has happened and its condition holds: recorded in the career like
a chain quest, fired and logged. Each completes once per career, beside the chain, which
never waits on one. A side quest's condition can't be `all_of` (sub-objectives latch only
for the chain's active quest).

**One-off events.** After the quests, each `ongoing` row in file order that has not fired
this run fires if its trigger has happened and its condition holds. Each fires at most once
per run, and again in the next run (its fired mark is per run). One triggered by a quest
completed earlier in the same check can fire that same day.

**Firing** records the event as fired this run (research `event_completed` reads it,
[research.md](research.md)) and as seen by the career, then applies its unlocks:

| Unlock | Effect |
|---|---|
| `enable_building` | the building may be bought this run. A building named by any event's `enable_building` is locked until one enables it ([economy.md](economy.md)). |
| `enable_nav_panel` | the panel is shown this run. Two gate a rule: `retirement` (the Retire action, [career.md](career.md)) and `research` (buying research, [research.md](research.md)); others (`adversaries` from 1.6, `projects`, `ideologies`, ...) are display. |
| `enable_project` | a project whose unlock is `project_enabled` may be funded this run ([projects.md](projects.md)) |
| `set_flag` | sets a career flag |
| `grant` (`resource`, `amount`) | adds the amount to this run's stock; re-applied at each run start (below), a starting bonus from then on |
| `grant_research` (`research`) | the item counts as bought this run (and known to the career); re-applied at each run start, so it is bought from day 0 |

**Run start.** Every event the career has seen (quests and one-offs alike) re-applies its
unlocks at the start of each run, so an unlocked building, panel or project stays unlocked
after a retirement from day 0. One-off events still fire again during the run once their
trigger and condition hold.

`category` decides chain quest, side quest or one-off. `silent` keeps a one-off event out of the activity
feed and the view's event list (display only). `title`, `summary` and `body` are text no
rule reads (the view hands a boredom phase's body its `rate`, the phase's boredom per day,
and the UI fills `{rate}` with it as Stats shows boredom growth). The run records the day
each one-off fired (`Run.events`, display only); the view lists them by that day, content
order within a day, and Story shows them newest first.

**Story modals** (UI only: no rule reads them, sims never see them). A chain quest's `story`
(shown as it becomes active) and `payoff` (shown as it completes) are paragraphs, one modal
each, in `content/events.json`; the quest's `summary` is its one-line task. When the active
quest moves on, the web UI pauses and shows the finished quests' payoffs (and the story of any
quest passed through the same day), then the new quest's story with its task, resuming after
the last; a new game shows only 1.1's. A quest with no `story` shows its `body`. The view
carries them (`quest.story`; each completed quest's `chain`, `story` and `payoff`).

**Activity feed** (display: no rule reads it). The run keeps a feed of what happened, by
day: each completed quest, earned achievement, completed project and bought research (the
career log's lines), each fired non-silent one-off event, each shipment (good, cargo,
credits), speculator burst (good, speculators added) and rival dump. At the end of each day
and after each action the oldest lines beyond `activity_kept` are dropped; a new run starts
it empty. The view also gives each quest's countable progress (owned, produced, shipments,
days, credits, best rank, retirements, Attention against its threshold, best launch) beside
its condition.

## Actions

None: events are checked each day.

## Params

Triggers, conditions and unlocks are content (`content/events.json`).

<!-- params:events -->
| Param | Baseline | Range | Meaning |
|---|---|---|---|
| `activity_kept` | 50 | 1 to 100000 | Lines of the run's activity feed kept (newest; display only). |
<!-- /params -->

## Content

`content/events.json`: 33 rows. Twenty-three `story`: Act 1's beats 1.1 Online to 1.11 Deep
Space Telescope (the swarm's reveal; [../arc.md](../arc.md)), then the draft Q8 to Q18 and the
Epilogue. Three `side`: 1.a Big Launch (opens Disrupt Speculators), 1.b Full Cluster (a placeholder
reward: a `grant` of 50 Science, so every later run starts with it; a cluster of 2 or more
items, `min_items`, the Blueprints excluded) and 1.c Blueprints for Everyone (`grant_research` of
both Blueprints once the Blueprint Archive is done). Seven `ongoing`: the Recreation Dome unlock
(once 1.8 is done), Political Currents, propellant discovery, boredom phases 2 to 5.

## Port notes

- Changed (049): the Godot chain's Q1 to Q7 became the design doc's beats. Q5 Market
  Awareness, Q6 Open Horizons and Q7 Signal Detected left the line (Q6 and Q7 are achievement
  candidates, [../arc.md](../arc.md)). Changed (074): the opening teaches one mechanic at a
  time (1.2 Blueprints, 1.3 First Shipment, 1.4 Full Load are new; Q2's Launch Plans event is
  gone); Retirement opens at 1.7, Foundation Grant and the Projects panel at 1.8.
- Conflict: Dream's falling effectiveness. The boredom phase 3 to 5 events' text says Dream
  drops to 20/15/10%; the ruling is a flat Dream ([dream.md](dream.md)), so these events
  unlock nothing and their text stays as placeholder.
- Conflict: Lunar Cartography and Deep Core Survey are unreachable ([projects.md](projects.md#port-notes)).
- Conflict: Godot's milestones (`game_config.milestones`) were removed per the handoffs;
  not ported.
- Candidate: quests ignore a `game_start` run filter, so the chain picks up where the
  career left it (1.1's `run_number` 1 would otherwise strand the chain after run 1).
- Candidate: a `quest_active` gate (projects, building `requires`) holds while the quest is
  active or after it completes.
- Candidate: one-off events fire once per run; `shipment_completed` counts this run's
  shipments.
