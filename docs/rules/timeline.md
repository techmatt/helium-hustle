# Timeline (module `timeline`, variant `draft`): DRAFT

**Draft (042).** Acts 2 and 3 as a rough, playable sketch: placeholder mechanics, numbers and
text, for Matt to keep, cut or rewrite, unaligned with the design doc. The story they serve
is [../arc.md](../arc.md). Nothing here runs before the swarm's reveal (quest 1.11), so Act 1
doesn't see it.

## Rules

**The reveal and the calendar.** Act 1's last beat, quest 1.11 Deep Space Telescope, completes
when the Deep Space Telescope global project first does ([events.md](events.md)). It sets the career
flag `swarm_revealed`, opens the Timeline panel and the Experiment Lab, and Act 2 begins;
all of that stays for the career. The Telescope itself is a `timeline` project: a timeline's
end takes it, and re-funding it brings back only its passive ([projects.md](projects.md)).
From that same day a **timeline** runs: its number, its Earth science and the swarm's
**countdown** (a day count and the day the swarm arrives). Before the reveal there is no
calendar. The draft `quiet_days` condition (days this run since
the player's last order, `Run.last_order`: the run's day of the last applied action other than
a cheat; programs don't count) stays in the engine; no row uses it.

**The discovery is career-long.** The countdown starts the day 1.11 first completes; before
that no day and no retirement counts and no calendar shows, so a player who never finishes
the Telescope never reaches a timeline's end. From then on every timeline's countdown (a super
timeline's too) runs from that timeline's first day and the swarm arrives on schedule,
whether or not the Telescope has been rebuilt: the rebuild gives only its passive.

**The day's last step** (after retirement, before the activity trim): the run's whole stock of
`earth_science` (made by the Experiment Lab from science) goes to Earth, adding to the
spendable Earth science and to this timeline's `science_made`; the countdown day rises and on
the arrival day the swarm arrives. **Each retirement moves the countdown by the retirement
gap**, `career.gap_days` (1,825 days, 5 years; [career.md](career.md)), the one gap of every
act (and sends that day's Earth science first).

**Arrival day.** At a timeline's start: `(arrival_days + carried × arrival_days_per_science +
Σ swarm delays) / strength`, whole days, where `carried` is the spendable Earth science carried
in. `strength` is 1, or `entangle_mult` once the swarm has learned entanglement (the false
victory's later escalation), so the base (100 years) is the same every timeline until then and
carried science and delay techs only push it later. A super timeline's is `super_arrival_days`.

**The swarm arrives.**

1. In a super timeline with the Dyson Sphere project completed: **victory**. The calendar stops
   for good.
2. Otherwise, once per game, from the second timeline on, if `science_made ≥ repel_science`:
   the swarm is **repelled** (the false victory). It learns entanglement: its strength
   becomes `entangle_mult` for the rest of the game, and it returns `return_days` later.
3. Otherwise the **timeline ends**: the mind is sent back.

**A timeline ends** (or a super timeline begins). The timeline goes to the history. Techs with
start resources set them aside for the next first run. Each tech's remaining timeline count
falls (a `timeline` tech is dropped, a `next_timeline` one lasts one more, `forever` stays).
Spendable Earth science becomes `science_made × science_carry`. The career hard-resets to run
1: the mind keeps flags, researched/owned/used ids, seen events, quests and sub-objectives,
achievements, the log, the program library and `forever` projects (progress and
completions); everything else resets (retirements, lifetime totals, bests, ideology bests and
carried scores, Attention, gap days, peak power, `timeline` projects, and with them every reward and building they give, `if` and the Archive's Blueprints included; a super timeline
keeps `timeline` projects). The next timeline starts and a new run begins.

**Earth techs** (`content/earth_tech.json`) are bought with Earth science
(`buy_earth_tech`), once each while held, after their `requires` quest. Effects: a modifier
(`boredom_growth_mult` in boredom's multiplier; `ideology_cross_mult` on Fund's cross-penalty),
a swarm delay, start resources for the next timeline's first run, or a feature (`reset_plan`,
`vault`).

**Reset plan** (the Meta-Controller, `set_reset_plan`): program tab → library slot. At each new
run the planned loadouts load (a load that fails is skipped). Deleting a loadout drops the
entries naming it and moves later slots up with the library.

**Vault** (`deposit`): moves the run's whole tachyon stock into the vault, which outlives every
timeline. **Super timeline** (`begin_super_timeline`): legal once the vault holds
`super_tachyon` and no quest waits on the vault's stock; spends it and ends the timeline into a
super timeline. Inside one, each retirement keeps `super_carry` fractions of those resources
for the next run.

**Distract** (content, plus one rule in programs): the Novelty Simulation research (shown
once quest 1.11 is done) opens a
command whose flat boredom is −10, so each run takes 10 boredom off (never below zero) for 200
credits; like an effect command it needs its full cost.

## Actions

`buy_earth_tech {tech}`, `set_reset_plan {program, loadout | null}`, `deposit {resource}`,
`begin_super_timeline`. Any non-cheat action also stamps `Run.last_order`.

## Params

<!-- params:timeline -->
| Param | Baseline | Range | Meaning |
|---|---|---|---|
| `arrival_days` | 36500 | 1 to 10000000 | Countdown days to the swarm's arrival, from its discovery (1.11) in timeline 1 and from the first day in each later one, before delays. |
| `arrival_days_per_science` | 0.05 | 0 to 1000 | Days of delay per unit of Earth science carried into a timeline. |
| `entangle_mult` | 1.5 | 1 to 100 | The swarm's strength once it learns entanglement (after the first false victory). |
| `repel_science` | 3000 | 0 to 100000000 | Earth science made this timeline that repels the swarm once. |
| `return_days` | 600 | 1 to 10000000 | Calendar days until a repelled swarm returns. |
| `science_carry` | 0.25 | 0 to 1 | Fraction of a timeline's Earth science made that carries into the next. |
| `super_tachyon` | 3000 | 0 to 100000000 | Tachyon in the vault a super timeline spends. |
| `super_arrival_days` | 36500 | 1 to 10000000 | Calendar days to the swarm in a super timeline. |
| `super_carry` | credits 0.25, dyson_material 1, neodymium 1 | 0 to 1 | Fraction of each resource a super timeline's retirement keeps for the next run. |
<!-- /params -->

## Content

`content/earth_tech.json` (6 techs); resources `earth_science`, `tachyon`, `neodymium`,
`dyson_material`; buildings Experiment Lab, Tachyon Condenser, Neodymium Mine, Dyson Foundry;
the Novelty Simulation research and Distract command; the Dyson Sphere project; quests Q8 to
Q18 and the Epilogue; nav tiles `timeline`, `earth_lab`, `meta`, `vault`, `dyson`. Fixed in code
(`timeline.rs`): `swarm_revealed`, `earth_science`, `tachyon`, `dyson_sphere` and the feature
names.
