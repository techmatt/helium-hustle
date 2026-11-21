# Projects (module `projects`, variant `drain`)

Projects are large builds funded a little each day: the player sets, per project and
resource, how much the project takes per day, and it completes once every cost is paid.
Long-Term projects keep their progress across retirements and reward every later run;
Strategic ones start over each run and reward the rest of it. Funding drains after
programs and buildings run, step 8 of [the day](README.md#the-day).

## Rules

**Tiers** (`tier` on the row):

- `persistent` (Long-Term): progress and completion live in the career.
- `personal` (Strategic): progress and completion live in the run and are lost at
  retirement.

Funding rates always live in the run: every project's rates are zero at the start of each
run, so a Long-Term project must be funded again after a retirement (its progress stays).

**Unlocked** (fundable): not yet completed, and its `unlock` holds:

| `unlock` | Holds when |
|---|---|
| `event_unlocked` | an event's `enable_project` enabled it this run (seen events re-apply at run start, [events.md](events.md)) |
| `ideology_rank` | the career's best rank on `axis` is at least `rank` (bests are recorded at step 14, so it opens the day after the rank is reached; [ideology.md](ideology.md)) |
| `quest_active` | the quest is active now or completed in any run |
| `research_completed` | the item is bought this run ([research.md](research.md)) |
| `flag_set` | the career flag is set |

**Drain** (each day, projects in file order). For each project with any rate set that is
unlocked, for each resource it has a rate on (in id order):

`take = max(0, min(rate, cost − progress, stock))`

where `stock` is the resource on hand at that moment (a hair below zero counts as zero).
The take is removed from stock (counted as the day's use, which [achievements.md](achievements.md)
reads) and added to progress. A short resource is taken partially; a met cost takes
nothing. A project that is locked keeps its rates but takes nothing.

**Completion.** After its drain, a project whose progress meets every cost completes:
it is marked completed (career or run, by tier), its rates are cleared, a `set_flag`
reward sets its flag, and the career log records it. A completed project is no longer
unlocked, so it can't be funded again.

**Rewards** (`reward` on the row), live while the project counts as completed (Long-Term:
every later run; Strategic: the rest of the run):

- `starting_buildings`: free bonus buildings at the start of every run (owned and
  running, not counted in their price curve; [career.md](career.md), [economy.md](economy.md)).
  Long-Term only in practice: it is read from the career's completions.
- `modifier`: a modifier `key` multiplied by `value` ([Modifiers](README.md#modifiers)).
- `set_flag`: a career flag, set once at completion (buildings and projects can require
  it).
- `boredom_modifiers`: `base_rate_mult` multiplies boredom growth
  ([boredom.md](boredom.md)); `command_boredom` replaces the named commands' flat boredom
  per execution ([programs.md](programs.md)).
- `research_discount`: research bought in an earlier run costs `mult` of its price
  ([research.md](research.md)).

## Actions

- **`set_project_rate`** `{"type": "set_project_rate", "project": <id>, "resource": <id>,
  "rate": <Fx>}`: sets how much of a resource the project takes per day; 0 removes the
  rate. Refused for an unknown project, one not unlocked (locked or already completed), a
  resource not among its costs, or a rate below 0 or above `max_drain_rate`.

A program's `fund <project> [N | all]` line sets the same rates, one N on every cost
([programs.md](programs.md), *Verbs*).

## Params

<!-- params:projects -->
| Param | Baseline | Range | Meaning |
|---|---|---|---|
| `max_drain_rate` | 30 | 0 to 1000000 | Most of one resource a project may take per day. |
<!-- /params -->

## Content

`content/projects.json`: 12 rows, seven Long-Term (the Dyson Sphere among them: draft,
[timeline.md](timeline.md)) and five Strategic, each with `tier`,
`unlock`, `costs` (resource amounts) and one `reward`; names and descriptions are the
Godot placeholders.

## Port notes

- Conflict: Microwave Receiver flag: see [economy.md](economy.md#port-notes).
- Conflict: Foundation Grant's unlock. The handoffs have QShipment enable it; the data has
  QMarket (`qmarket_awareness`) enable it and the Projects panel. The engine uses the data.
- Conflict: Lunar Cartography and Deep Core Survey unlock by `event_unlocked`, but no
  event enables them. Kept as data, so both are unreachable (candidate: give each an
  enabling event or another unlock).
- Candidate: a `quest_active` gate holds while the quest is active or after it is
  completed (Chemical Energy Initiative stays fundable after Open Horizons completes).
- Candidate: AI Consciousness Act's `command_boredom` values replace the commands' own
  boredom per execution rather than adding to it.
