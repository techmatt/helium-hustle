# Boredom (module `boredom`, variant `phased`)

Boredom is the clock on a run. It grows every day at a rate set by how long the run has
lasted, and when it reaches its cap the run ends in forced retirement ([career.md](career.md)).
Growth is the first step of [the day](README.md#the-day); the retirement check is the last.

## Rules

**Phase.** A run's phase is the number of `phase_days` entries at or below the run's day
(1-based: day 0 is phase 1). At the start of each tick the phase is stored on the run
(`boredom_phase`, which the `boredom_phase` event trigger reads, [events.md](events.md)) and
boredom grows by

```
rate = phase_rates[phase - 1] × multiplier
```

where the run's day is the day before this tick's day count rises (the first tick of a run
grows at day 0's rate). Growth is added straight to boredom; it is not production and does
not count in the day's flows.

**Rate multiplier.** The product of, in this order (each `Fx` product rounded by the one rule
in [README.md](README.md)):

1. every research bought this run with a `boredom_rate_multiplier` effect (Stress Tolerance, ×0.85);
2. `discount_factor ^ humanist rank` ([ideology.md](ideology.md); above 1 for a negative rank);
3. every completed project with a `boredom_modifiers` reward, its `base_rate_mult`
   (AI Consciousness Act, ×0.85; a `timeline` project, so it holds every later run of the timeline);
4. the `boredom_growth_mult` modifier (Boredom Dampers, a draft Earth tech: [timeline.md](timeline.md));
5. the career's boredom resilience, `boredom_resilience_base ^ (best_run_days / boredom_resilience_period)`
   ([career.md](career.md)).

**Other sources.** Each command execution adds its flat `boredom` (or the replacement a
completed AI Consciousness Act gives it), and Dream takes boredom away, never below zero; see
[programs.md](programs.md) and [dream.md](dream.md). Boredom is never a command product, so
it never causes the output-cap skip.

**Cap.** `cap = base_cap + Σ store(boredom)` over *working* buildings (active, upkeep paid): each working Recreation
Dome adds its content `store` of 100. The `storage_cap_mult` modifier does not apply to
boredom. The cap is computed on demand, so buying, selling or idling a dome moves it at once.

**Clamp.** Like every capped resource, boredom is cut to its cap in the end-of-tick clamp
([economy.md](economy.md)), and lifted to zero if below it. Between ticks boredom can sit
above the cap (after a dome is sold or idled); the next tick's clamp cuts it back.

**Forced retirement.** After the tick's last step (the run's day count has risen), if
`boredom >= cap` the run retires (`forced`) and the next run starts within the same tick
([career.md](career.md)). Because of the clamp, reaching the cap and passing it are the same.
Lowering the cap below current boredom (selling or idling domes) retires the run on the next
tick.

## Actions

None. Boredom moves only through the day and commands.

## Params

<!-- params:boredom -->
| Param | Baseline | Range | Meaning |
|---|---|---|---|
| `phase_days` | 0, 200, 500, 1000, 2000 | 0 to 1000000 | The run day each phase starts (first 0, rising). |
| `phase_rates` | 0.1, 0.2, 0.5, 1, 3 | 0 to 1000 | Boredom per day in each phase, before modifiers. |
| `base_cap` | 1000 | 1 to 1000000 | The boredom cap before Recreation Domes. |
<!-- /params -->

`phase_days` must start at 0 and rise, one per rate (a load error otherwise). At baseline
with no modifiers a run has 20 + 60 + 250 = 330 boredom by day 1000 and reaches 1000 on day
1670; command boredom brings it sooner.

## Content

`content/resources.json` row `boredom` (uncapped there: its cap is this module's);
`content/buildings.json` `store` effects on boredom (Recreation Dome);
`content/research.json` `boredom_rate_multiplier` effects; `content/projects.json`
`boredom_modifiers` rewards; `content/commands.json` per-command `boredom`;
`content/events.json` `boredom_phase` triggers (phase 2 to 5 messages).

## Port notes

- Conflict: cap. The handoffs say 500 + 100 per Recreation Dome; the data's `boredom_max` is
  1000. By ruling the base is 1000 in config (`base_cap`, as in the Godot build Matt played;
  rules revision 3), and the dome's 100 is its content `store` effect.
- Conflict: the phase 3 to 5 event texts say Dream effectiveness falls to 20/15/10%; the
  handoffs' Dream is flat. By ruling Dream is flat ([dream.md](dream.md)) and the event text
  is left as the Godot placeholder: the phases change only the growth rate.
- Candidate: the day. A tick grows boredom at the run's current day, the day count rises at
  the end (Godot's "advance day" last), forced retirement is checked after it, and the next
  run starts at once.
- Candidate: the AI Consciousness Act's command boredom values replace the commands' own
  (0.1 to 0.2 for Cloud Compute, for instance); the handoffs say only that it modifies them.
