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
far (`Fx::pow_int`, no float). **The funding cap:** a project takes at most
`credits_per_day` credits a day and at most `other_per_day` of each other resource (params;
raisable later by unlocks). A row with a deliberate `min_days` (only the Deep Space
Telescope's 3,000) also caps each rate at its current cost over those days, rounded up a step.
So a run's stockpile can't be dumped into a project on its last day.

**Drain** (each day, projects in file order). For each project with any rate set that is
unlocked, for each resource it has a rate on (in id order):

`take = max(0, min(rate, cost − progress, stock))`

where `stock` is the resource on hand at that moment (a hair below zero counts as zero).
The take is removed from stock (counted as the day's use, which [achievements.md](achievements.md)
reads) and added to progress. A short resource is taken partially; a met cost takes
nothing. A project that is locked keeps its rates but takes nothing. A take short of its
want is kept until the next drain (`project_short`): the **actual rate** the Projects panel
shows beside the set one, and the rate `days_to_done` ("Done in N days") counts.

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

- `starting_buildings`: free bonus buildings at the start of every run, free extras (Matt,
  130): owned and running, so every building count (quest conditions, research shown at a
  count) includes them, but not counted in their price curve, outside the average price
  paid, and sold for no refund ([career.md](career.md), [economy.md](economy.md)). A
  scenario's completed projects grant them too.
- `modifier`: a modifier `key` multiplied by `value` ([Modifiers](README.md#modifiers)).
- `flag`: a flag held while live (not a career flag: a timeline's end takes it), read
  wherever a flag is (`flag_set` conditions, flag requirements, `logic_if`), and `text`,
  what the card says it gives.
- `starting_research`: research items counted as bought at completion and from day 0 of
  every run while live ([research.md](research.md)).
- `boredom_modifiers`: `base_rate_mult` multiplies boredom growth (stacking as a
  multiplier, [boredom.md](boredom.md)); `command_boredom` replaces the named commands' flat
  boredom per execution ([commands.md](commands.md)).
- `research_discount`: research bought in an earlier run (career-wide) costs `mult` of its
  price, only research of `category` (a research row's) when it names one, any when null
  ([research.md](research.md)).

A building gated on a project (`requires: {"type": "project"}`, [economy.md](economy.md))
follows the project's scope: it is buildable while the project counts as completed, so a
timeline's end takes it away with the completion. Microwave Receiver and Fuel Cell Array
are gated so; their projects' rewards lists are empty.

**What the player sees.** Each project's view carries its unlock and its rewards as text,
written by the engine from the row (a gated building shows as "Opens ..."); the Projects
panel shows only projects that can be funded now (no locked or finished one-off cards,
docs/rulings/ui.md), and names no timeline before the swarm's reveal (its global section
reads "Global" until then). A funded one shows its days to
done: the most, over its costs still owed, of owed / rate rounded up (stock aside), none
while an owed cost has no rate. The Stats panel lists every live project reward. The view's
`project_done` lists this run's completions from the career log (day, which completion in
its scope, its rewards, and what that completion cost: the cost after the completions
before it); the UI pops each up and lists it in Events (rulings/story-ui.md, *Popups*).

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
| Param | Baseline | Range | Meaning |
|---|---|---|---|
| `credits_per_day` | 30 | 0.000001 to 1000000 | The most credits a project takes a day. |
| `other_per_day` | 10 | 0.000001 to 1000000 | The most of each other resource a project takes a day. |
<!-- /params -->

## Content

`content/projects.json`: 14 rows, ten `timeline` (the Dyson Sphere among them: draft,
[timeline.md](timeline.md)) and four `run`, each with `scope`, `unlock`, `costs`,
`rewards` (none repeats yet; `min_days` on the telescope only); names and descriptions are the Godot
placeholders but for the draft rows (049): Conditional Logic (1,000 credits and 100 science;
its reward the `logic_if` flag that opens `if`, [programs.md](programs.md)) and the
Deep Space Telescope (offered once quest 1.6 is active; its first completion completes 1.6,
so it spans several runs: `min_days` 3,000; rebuilt each timeline, at the same cost, for
its placeholder passive, `science_output_mult` and `nationalist_growth_mult` ×1.1, and only
that: the swarm's countdown runs without it, and what 1.6 opened stays:
[timeline.md](timeline.md)). Chemical Energy Initiative (2,000 credits, 2,000 propellant,
1,000 ice) is offered once the research Electrochemistry is bought (its career flag
`electrochemistry`, [research.md](research.md)). Blueprint Archive (draft: 2,000 credits and 300 science):
its `starting_research`
reward has Basic and Advanced Blueprints bought from the start of every run of the timeline,
and its `research_discount` (0.8, category Blueprints) makes every Blueprints item bought in
an earlier run cost 20% less while it counts as completed ([research.md](research.md)).
Placeholder costs (Matt, 130; each at most three resources): Lunar Cartography 1,000
credits, 200 science and 500 regolith; Foundation Grant 4,000 credits, 500 science and 500
titanium, for 2 Solar Panels, 2 Regolith Excavators and 1 Smelter at each run's start;
Microwave Power Initiative 2,000 credits, 300 science and 1,000 titanium; AI Consciousness
Act 2,000 credits, 1,000 circuits and 1,000 He-3; Universal Research Archive 1,000 credits,
1,500 science and 300 circuits. At the caps Blueprint Archive takes 67 days (credits),
Foundation Grant 134, Lunar Cartography 50 (regolith). **Where they open.** The Projects panel, where a player sees them, opens
at A Voice's start (quest 1.5; the silent `a_voice_projects` event, a `quest_active`
condition) with Blueprint Archive, Conditional Logic, Lunar Cartography and Foundation
Grant (each `project_enabled`); a first retirement before it (the silent `first_retirement`
event, [events.md](events.md)) opens the panel with all of those but Lunar Cartography.

## Port notes

- Conflict: Microwave Receiver gate: see [economy.md](economy.md#port-notes).
- Changed: A Voice's start (and the first retirement) enables Foundation Grant and the
  Projects panel (the Godot data had Market Awareness, a quest no longer in the line).
- Conflict: Deep Core Survey unlocks by `project_enabled`, but no event enables it. Kept
  as data, so it is unreachable (candidate: give it an enabling event or another unlock).
- Candidate: a `quest_active` gate holds while the quest is active or after it is
  completed (the Deep Space Telescope stays fundable after 1.6 completes).
- Candidate: AI Consciousness Act's `command_boredom` values replace the commands' own
  boredom per execution rather than adding to it: Cloud Compute's 0.15 is ×1.5 its own 0.1
  (Matt, 130). It is the only entry (Matt, 131: Load Pads' 0.3 and Disrupt Speculators' 0.5
  dropped).
