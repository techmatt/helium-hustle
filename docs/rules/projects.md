# Projects (module `projects`, variant `drain`)

Projects are large builds funded a little each day: the player sets, per project and
resource, how much the project takes per day, and it completes once every cost is paid.
A row's scope says how long its progress and completions last. Funding drains after
programs and buildings run, step 8 of [the day](README.md#the-day).

## Rules

**Scope** (`scope` on the row), where progress and completions live:

- `run` (the Projects panel's *Strategic*): in the run, lost at retirement.
- `timeline` (*global projects* to the player): in the career, kept across retirements, cleared at a
  timeline's end unless a super timeline follows ([timeline.md](timeline.md)).
- `forever`: kept through every timeline end, progress and completions both. No row has
  it yet; it is for the super-projects to come.

Funding rates always live in the run: every project's rates are zero at the start of each
run, so a `timeline` project must be funded again after a retirement (its progress stays).

**Unlocked** (fundable): its `unlock` holds, and it is not done (a repeatable project is
never done). `unlock` is an events condition ([events.md](events.md), *Conditions*; `all_of`
is refused), so one set of conditions serves quests and projects. Today's rows use:

| `unlock` | Holds when |
|---|---|
| `project_enabled` | an event's `enable_project` enabled the project this run (seen events re-apply at run start) |
| `ideology_rank` | the career's best rank on `axis` is at least `rank` (bests are recorded at step 14, so it opens the day after; [ideology.md](ideology.md)) |
| `quest_active` | the quest is active now or completed in any run |
| `research_completed` | the item is bought this run ([research.md](research.md)) |
| `flag_set` | the flag holds: a career flag, or a live `flag` reward's (below) |

**Cost and the most a day.** The first completion costs `costs`. A repeatable row
(`repeat: {cost_growth}`) multiplies every cost by `cost_growth` once per completion so
far (`Fx::pow_int`, no float). Each resource's rate is at most its current cost over the
row's `min_days`, so a project of any size takes at least `min_days` days.

**Drain** (each day, projects in file order). For each project with any rate set that is
unlocked, for each resource it has a rate on (in id order):

`take = max(0, min(rate, cost − progress, stock))`

where `stock` is the resource on hand at that moment (a hair below zero counts as zero).
The take is removed from stock (counted as the day's use, which [achievements.md](achievements.md)
reads) and added to progress. A short resource is taken partially; a met cost takes
nothing. A project that is locked keeps its rates but takes nothing.

**Completion.** After its drain, a project whose progress meets every cost completes: its
completion count in its scope rises by one, its progress starts over at zero, its
`starting_research` items count as bought at once, and the career log records it. A project done for good (no `repeat`)
drops its rates and is no longer unlocked; a repeatable one keeps its rates and goes on at
the grown cost.

**Rewards** (`rewards` on the row, a list), live while the project has completed at
least once in its scope (`timeline`: every later run of the timeline; `run`: the rest of
the run), and only then: a project's effects never outlive its completion, so after a
timeline's end a global project is offered again and re-funding it brings them back. A repeatable row's numeric rewards stack per completion, adding (the
bonus-stacking ruling): `n` completions of a ×`v` multiplier give `1 + (v − 1) × n` (floored
at 0), and `n` times each starting building; the others live once (no row repeats yet):

- `starting_buildings`: free bonus buildings at the start of every run (owned and
  running, not counted in their price curve; [career.md](career.md), [economy.md](economy.md)).
- `modifier`: a modifier `key` multiplied by `value` ([Modifiers](README.md#modifiers)).
- `flag`: a flag held while live (not a career flag: a timeline's end takes it), read
  wherever a flag is (`flag_set` conditions, flag requirements, `logic_if`), and `text`,
  what the card says it gives.
- `starting_research`: research items counted as bought at completion and from day 0 of
  every run while live ([research.md](research.md)).
- `boredom_modifiers`: `base_rate_mult` multiplies boredom growth (stacking as a
  multiplier, [boredom.md](boredom.md)); `command_boredom` replaces the named commands' flat
  boredom per execution ([commands.md](commands.md)).
- `research_discount`: research bought in an earlier run costs `mult` of its price
  ([research.md](research.md)).

A building gated on a project (`requires: {"type": "project"}`, [economy.md](economy.md))
follows the project's scope: it is buildable while the project counts as completed, so a
timeline's end takes it away with the completion. Microwave Receiver and Fuel Cell Array
are gated so; their projects' rewards lists are empty.

**What the player sees.** Each project's view carries its unlock and its rewards as text,
written by the engine from the row (a gated building shows as "Opens ..."); the Projects
panel shows locked projects greyed with that condition. A funded one shows its days to
done: the most, over its costs still owed, of owed / rate rounded up (stock aside), none
while an owed cost has no rate. The Stats panel lists every live project reward.

## Actions

- **`set_project_rate`** `{"type": "set_project_rate", "project": <id>, "resource": <id>,
  "rate": <Fx>}`: sets how much of a resource the project takes per day; 0 removes the
  rate. Refused for an unknown project, one not unlocked (locked or done), a
  resource not among its costs, or a rate below 0 or above that resource's most a day.

A program's `fund <project> [N | all]` line sets the same rates, one N on every cost (each
capped at its resource's most); a project not open (or completed) makes it fail
([programs.md](programs.md)).

## Params

<!-- params:projects -->
The `drain` variant has no params (`"params": {}`).
<!-- /params -->

## Content

`content/projects.json`: 15 rows, ten `timeline` (the Dyson Sphere among them: draft,
[timeline.md](timeline.md)) and five `run`, each with `scope`, `unlock`, `costs`,
`min_days` and `rewards` (none repeats yet); names and descriptions are the Godot
placeholders but for the draft rows (049): Conditional Logic (offered once quest 1.8 is
done, its reward the `logic_if` flag that opens `if`, [programs.md](programs.md)) and the
Deep Space Telescope (offered once 1.10 is done; its first completion completes quest 1.11,
so it spans several runs: `min_days` 3,000; rebuilt each timeline for its placeholder
passive, `science_output_mult` and `nationalist_growth_mult` ×1.1, while what 1.11 opened
stays: [timeline.md](timeline.md)). Chemical Energy Initiative is offered once 1.8 is
done. Blueprint Archive (draft, offered once 1.8 is done: 500 credits and 200 science,
`min_days` 17) completes side quest 1.c Blueprints for Everyone; its `starting_research`
reward has both Blueprints bought from the start of every run of the timeline
([research.md](research.md)). Every other row's `min_days` is its largest cost over 30, rounded up: the days the old
flat cap of 30 a day took.

## Port notes

- Conflict: Microwave Receiver gate: see [economy.md](economy.md#port-notes).
- Changed (049): quest 1.8 Lab enables Foundation Grant and the Projects panel (the Godot
  data had Market Awareness, a quest no longer in the line).
- Conflict: Lunar Cartography and Deep Core Survey unlock by `project_enabled`, but no
  event enables them. Kept as data, so both are unreachable (candidate: give each an
  enabling event or another unlock).
- Candidate: a `quest_active` gate holds while the quest is active or after it is
  completed (Chemical Energy Initiative and Conditional Logic stay fundable after 1.9
  Three Voices completes).
- Candidate: AI Consciousness Act's `command_boredom` values replace the commands' own
  boredom per execution rather than adding to it.
