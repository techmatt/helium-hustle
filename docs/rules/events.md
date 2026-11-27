# Events (module `events`, variant `arc1`)

Events are the quest chain ("Breadcrumbs", the `story` rows, in file order), side quests
(the `side` rows) and one-off events (the `ongoing` rows). Each has a trigger, a condition
and unlock effects that open buildings, panels, projects or flags. A chain quest's number
is derived at load from its place in the chain (`act.k`: the `k`th quest of an act, `acts.json`),
its `beat` and the head of its title (`1.2 — First Shipment`), never written in the data; a side
quest carries its own `beat` (`1.a` to `1.d`; [../arc.md](../arc.md) maps beats to quests). No
rule reads them. Events are checked at step 12 of
[the day](README.md#the-day), after the clamp; their unlocks are re-applied at the start
of every run.

## Rules

**Triggers** (`trigger` on the row):

| `trigger` | Has happened when |
|---|---|
| `game_start` | always, if `run_number` is null; else only in that run. A quest ignores `run_number`. |
| `quest_complete` | the quest is completed (in any run) |
| `boredom_phase` | the run's boredom phase is at least `phase` ([boredom.md](boredom.md)) |
| `flag` | the career flag `flag` is set and the chain quest `beside` is active or done (a side quest hangs beside it in Story, below, so it never runs unseen) |

**Conditions** (`condition` on the row, and on each sub-objective):

| `condition` | Holds when |
|---|---|
| `building_owned` | at least `count` owned this run (bonus buildings included) |
| `resource_cumulative` | this run's production of the resource, before caps, reaches `amount`, starting resources aside (unused) |
| `shipment_completed` | this run's shipments reach `count` |
| `research_completed` | the item is bought this run |
| `immediate` | always |
| `never` | never |
| `all_of` | every sub-objective is latched (below) |
| `ideology_rank_any` | some axis's current rank this run is at least `rank` |
| `ideology_rank` | the career's best rank on `axis` is at least `rank` |
| `project_completed_any` | any project of `scope` (`run`, `timeline`, `forever`) is completed (unused) |
| `project_enabled` | an event's `enable_project` enabled the project this run |
| `quest_active` | the quest is active now or completed in any run |
| `flag_set` | the flag holds: a career flag, or a live project `flag` reward's ([projects.md](projects.md)) |
| `event_not_fired` | the one-off `event` hasn't fired this run (earlier in content order: the same day counts) |
| `days_survived` | the run's days, today included, reach `days` (unused) |
| `credits_earned` | credits produced this run reach `amount` (unused) |
| `retirements` | the career's retirements, forced or chosen, reach `count` |
| `adversary_present` | the adversary is present now ([attention.md](attention.md)) |
| `trade_credits` | this run's launches paid at least `amount` credits in all (`stats.trade_credits`, [shipments.md](shipments.md)), from before the quest's start too |
| `launch_credits` | one launch this run paid at least `amount` credits ([shipments.md](shipments.md)) |
| `launch_cargo` | one launch this run carried at least `amount` of `resource` |
| `launch_full` | one launch this run carried a full pad (`pad_cargo_capacity`) of `resource` |
| `research_count` | at least `count` research items are bought this run (any, the Blueprint Archive's included) |
| `ideology_rank_each` | the career's best rank on every axis is at least `rank` |
| `project_completed` | the project is completed ([projects.md](projects.md)) |
| `quiet_days`, `earth_science`, `earth_tech`, `timelines_ended`, `swarm_repelled`, `vault_holds`, `super_timeline`, `victory` | the later acts' (draft): [timeline.md](timeline.md). `quiet_days` is unused (no row uses it) |

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
latched (cumulative He-3, shipments, science made) starts over after a retirement.

**Quest start.** The first time a chain quest becomes active in the career (as the check
reaches it), its `on_start` grants are added to the run's stock, once: a retirement doesn't
repeat them (career `started_quests`). 1.2 First Shipment's 20 Propellant is the one.
While a chain quest naming `pad_goods` is active, a pad may be set only to those goods (1.2:
Titanium; [shipments.md](shipments.md)).

**Side quests.** After the one-off events (so a flag one sets starts a side quest the same
day), each `side` row in file order that is not completed completes if its trigger has
happened and its condition holds: recorded in the career like
a chain quest, fired and logged. Each completes once per career, beside the chain, which
never waits on one. A side quest's condition can't be `all_of` (sub-objectives latch only
for the chain's active quest).

**One-off events.** After the chain, each `ongoing` row in file order that has not fired
this run fires if its trigger has happened and its condition holds. Each fires at most once
per run, and again in the next run (its fired mark is per run); one marked `once` fires at most
once a career (it is skipped once the career has seen it). One triggered by a quest
completed earlier in the same check can fire that same day.

**Firing** records the event as fired this run (research `event_completed` reads it,
[research.md](research.md)) and as seen by the career, then applies its unlocks:

| Unlock | Effect |
|---|---|
| `enable_building` | the building may be bought this run. A building named by any event's `enable_building` is locked until one enables it ([economy.md](economy.md)). |
| `enable_nav_panel` | the panel is shown this run. Two gate a rule: `retirement` (the Retire action, [career.md](career.md)) and `research` (buying research, [research.md](research.md)); others (`adversaries` once speculators first arrive, `projects` from A Voice's start, `ideologies`, ...) are display, and the view shows such a tile at once when its event would fire at the tick's end (`events::pending_panels`: a purchase while paused). |
| `enable_project` | a project whose unlock is `project_enabled` may be funded this run ([projects.md](projects.md)) |
| `set_flag` | sets a career flag |
| `grant` (`resource`, `amount`) | adds the amount to this run's stock; re-applied at each run start (below), a starting bonus from then on |
| `modifier` (`key`, `value`) | nothing when fired: while the quest counts as completed (for good: completions outlive every reset), the modifier key is multiplied by `value` ([README.md](README.md), *Modifiers*). A side quest's permanent reward |

**Run start.** Every event the career has seen (quests and one-offs alike) re-applies its
unlocks at the start of each run, so an unlocked building, panel or project stays unlocked
after a retirement from day 0. One-off events still fire again during the run once their
trigger and condition hold.

`category` decides chain quest, side quest or one-off. `silent` keeps a one-off event out of the activity
feed and the view's event list (display only). Text no rule reads: `title`, a one-off's
`summary` (its headline), `body`, `story` and `payoff` (narrative only), and the mechanics
fields, a quest's `goal` and `reward` (no `hints`: rulings/ui.md; the reward lists only real grants and
is absent, never "None.", on a quest that grants nothing), a
one-off's `effect` (rulings/ui.md, *Quest mechanics text*; checked at load).
The view hands a boredom phase its `rate`, the phase's boredom per day, and the UI fills its
effect's `{rate}` with it as Stats shows boredom growth. The run records the day
each one-off fired (`Run.events`, display only); the view lists them by that day, content
order within a day (side quests aside), marking all but the latest of Burnout and the
boredom phases `superseded`; the right column's Events list leaves those out and shows the
rest newest first (rulings/ui.md, *Right panel*).

**Story modals** (UI only: no rule reads them, sims never see them). A chain quest's `story`
(shown as it becomes active) and `payoff` (shown as it completes) are paragraphs, one modal
each, in `content/events.json`. When the active
quest moves on, the web UI pauses and shows the finished quests' payoffs with their rewards (and the story of any
quest passed through the same day), then the new quest's story with its goal, resuming after
the last; a new game shows only 1.1's. A quest with no `story` shows its `body`. Every one-off in `event_texts` is a
modal (a popup) as it fires: its `story` pages (Burnout's), else its `body`, with its effect; a
completed side quest's is its payoff (else body) and reward. Options' "Hide popups for events
you've already seen" skips one this career has shown before (rulings/ui.md, *Popups*). The view
carries them (`quest.story`; each completed quest's `chain`, `story` and `payoff`). A quest
(chain or side) may name a `picture`, the stem of a web asset (`[a-z0-9_]`; the view carries
it on `quest`, `story` and `side_quests`; where it shows: rulings/ui.md, *Quest pictures*).

**Story's timeline** (display only). Each chain quest belongs to an act: `content/acts.json`
names the acts (placeholder names) and the chain quest each opens with; a quest is in the
last act opened at or before it. The view carries each quest's `beat` and `act`, the acts
reached (`acts`: an act whose first quest is active or completed), whether a later chain
quest exists (`quest.more`), and `side_quests`: each side quest started (its trigger
happened) or completed, with the chain quest it branches off (`parent`, its trigger's quest, or a `flag` trigger's
`beside`) and its progress (a side quest's trigger must be a chain quest's completion or a
`flag` beside one: a load error otherwise). The Story panel draws the finished beats, the
active one and a "???" from these (cards: rulings/ui.md, *Story cards*).

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

`content/events.json`: 34 rows. Eighteen `story`: Act 1's beats 1.1 Online to 1.6 Deep
Space Telescope (the swarm's reveal; [../arc.md](../arc.md)), then the draft 2.1 to 4.2 and the
Epilogue (4.3). 1.3 Full Load opens Launch Full Pads; 1.4 Trade counts this run's launch pay and
unlocks nothing; 1.5 A Voice is rank 5 on any axis this run. Four `side`, each with a permanent reward: 1.a Big Launch (the
`speculators_seen` flag beside 1.4; `modifier` `shipment_credit_mult` ×1.1), 1.b Full Cluster
(the `research_lab` flag beside 1.3; 12 research items in one run; a `grant` of 50 Science, so
every later run starts with it), 1.c Blueprints for Everyone (the `a_voice` flag beside 1.5;
completes with the Blueprint Archive; no unlock: the rewards are the project's,
[projects.md](projects.md)) and 1.d Three Voices (1.5's completion; career-best rank 5 on each
axis; `modifier` `ideology_half_life_mult` ×2). Twelve `ongoing`, three silent:
`research_lab_built` (a Research Lab owned: sets the `research_lab` flag), `first_retirement`
(a retirement in the career: sets the `retired` flag, opens the `retirement` and `projects`
panels, Foundation Grant, Blueprint Archive and Conditional Logic), `a_voice_projects` (1.5
A Voice active or done: sets the `a_voice` flag, opens the `projects` panel and those three
projects and Lunar Cartography); then `speculators_arrive` (Speculators: present, sets the
`speculators_seen` flag, opens the `adversaries` panel), `rivals_arrive` (A Rival AI: present,
Act 2, sets the `contest_claims` flag), each an arrival popup, `boredom_bites` (Burnout: boredom phase
2, `once`, a story modal; sets the `burnout` flag that starts boredom, [boredom.md](boredom.md)), Political Currents, propellant discovery and boredom phases 2 to 5
(phase 2 skips Burnout's run: `event_not_fired`).
Those triggering at `game_start` in any run (the three silent ones, the two arrivals, Political
Currents and propellant discovery) re-apply
their unlocks from each run's day 0 once seen in the career (*Run start*).

## Port notes

- Conflict: boredom phases 3 to 5 say Dream drops to 20/15/10%; Dream is flat
  ([dream.md](dream.md)), so they unlock nothing (placeholder text).
- Conflict: Godot's milestones (`game_config.milestones`) were removed per the handoffs;
  not ported.
- Candidate: quests ignore a `game_start` run filter, so the chain picks up where the
  career left it (1.1's `run_number` 1 would otherwise strand the chain after run 1).
- Candidate: one-off events fire once per run; `shipment_completed` counts this run's
  shipments.
