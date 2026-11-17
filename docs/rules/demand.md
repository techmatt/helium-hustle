# Demand (module `demand`, variant `arc1`)

Each tradeable good ([shipments.md](shipments.md)) has a market whose **demand** scales what a
launch pays. Demand is set once a day, after boredom grows and expired overclocks drop and
before any building produces or program runs its commands; speculator bursts and rival dumps land after pads cool down
([the day](README.md#the-day)). So whatever commands do today (launches, Promote, Disrupt)
shows in demand tomorrow.

## Rules

**Demand.** For each good, on run day `d`:

```
raw    = base_demand + noise(d) − suppression(speculators) − rival_pressure
         − saturation + promote + coupling_fraction × (sum of the other goods' promote)
demand = clamp(raw × N, min_demand, max(ceiling, min_demand))
```

`N` is the Nationalist bonus, `bonus_factor ^ nationalist rank` (below 1 for a negative rank;
[ideology.md](ideology.md)). `ceiling = max_demand × demand_ceiling` modifiers (Market Timer
raises it to 1.1).

**Noise.** At each run start, each good draws (stream `demand_noise`, in content order) a
frequency uniform in `noise_freq_min`..`noise_freq_max` and a lattice key. With `x = frequency
× d`, octave `o` evaluates 1D gradient noise at `x × noise_octave_freqs[o]`: gradients at whole
lattice points are hashed from (key, octave, point) into −2..2, blended with the quintic fade
`t³(t(6t − 15) + 10)`. The noise term is the octaves' mean weighted by `noise_octave_gains`,
clamped to −1..1, times `noise_amplitude`. All fixed point; no floats.

**Speculator suppression.** Each good has its own pool `n`:
`suppression = speculator_max_suppression × n / (n + speculator_half_point)`. Pools have no cap.

**Daily update**, before demand is set, for every good:

1. Pool decay: `n −= n × speculator_decay × N + arbitrage_decay_per_engine × E × N`, floored
   at 0, where `E` counts working (active, upkeep paid) buildings with the `speculator_decay` effect (Arbitrage Engine).
2. `promote −= promote_decay`, `rival_pressure −= rival_decay`, `saturation −=
   saturation_decay`, each floored at 0 (linear fades).

**Speculator bursts.** A burst is due when the run day reaches the scheduled day. Its target
is drawn (stream `speculators`) in proportion to each good's shipment revenue since the last
burst (goods walked in id order). If no good has revenue, there is no burst: the counter does
not move and the next one is scheduled anyway. Otherwise the target's pool gains
`size × burst_growth ^ bursts`, `size` uniform in `burst_size_min`..`burst_size_max` and
`bursts` the bursts so far this run; then every good's revenue resets to 0. The next burst
comes `floor(interval × speculator_burst_interval_mult)` days on (at least 1), the interval
uniform in `burst_interval_min`..`burst_interval_max`. The first is scheduled at run start.

**Rivals.** Each rival dumps on its own good when its scheduled day arrives: the good's
`rival_pressure` rises by the rival's `demand_hit`, and its next dump is drawn (stream
`rivals`) uniform in its `interval_min`..`interval_max` days on. Each rival's first dump is
scheduled at run start.

**Launch saturation.** Each launch of a good adds a draw uniform in
`saturation_min`..`saturation_max` (stream `launch_saturation`) to its saturation, and its
payout to its burst-target revenue.

**Promote** (a command naming a good): the good's promote rises by `promote_effect ×
promote_effectiveness_mult` (Market Cornering Analysis: x1.3), and its pool is multiplied by
`promote_speculator_dampening`. Every other good gains `coupling_fraction` of it through the
coupling term.

**Disrupt Speculators** draws an amount uniform in `disrupt_min`..`disrupt_max` (stream
`disrupt`, drawn every execution) and takes it off the first good, in loading priority, whose
pool is above 0, floored at 0. If every pool is 0, nothing happens; the command's cost is still
spent. Each execution sets the career flag `used_disrupt_speculators` (the command's content
`sets_flag`; it unlocks Speculator Dossier).

**A new run** re-rolls every good's frequency and key, empties pools, rival pressure,
saturation, promote and revenue, resets the burst count, schedules the first burst and every
rival's first dump, and sets the day's demand.

**Display.** Until a research item with the `reveal_demand` effect (Market Awareness) is done
this run, the view shows a tier, not the number:
LOW below the first of `tier_bounds`, then MEDIUM, HIGH, VERY HIGH at or above each next bound.
A good's speculator pool shows once it has been above 0 this run (set by a burst).

## Actions

None of its own. Promote (`promote_*`, after Trade Promotion) and Disrupt Speculators (after
Market Awareness) are commands run by programs ([programs.md](programs.md)); as effect
commands they need their full cost. Loading priority, which Disrupt reads, is set by
`set_loading_priority` ([shipments.md](shipments.md)).

## Params

| Param | Baseline | Meaning |
|---|---|---|
| `min_demand` | 0.01 | Demand floor. |
| `max_demand` | 1 | Demand ceiling before `demand_ceiling` modifiers. |
| `base_demand` | 0.5 | Demand with every force at zero. |
| `noise_amplitude` | 0.45 | The noise term's largest swing either way. |
| `noise_freq_min` | 0.018 | Lowest per-good noise frequency (cycles per day). |
| `noise_freq_max` | 0.05 | Highest per-good noise frequency. |
| `noise_octave_freqs` | 1, 2.137, 4.581, 9.719 | Each octave's frequency multiplier. |
| `noise_octave_gains` | 1, 0.5, 0.25, 0.125 | Each octave's weight. |
| `speculator_max_suppression` | 0.5 | Demand a pool suppresses as it grows without bound. |
| `speculator_half_point` | 50 | Pool size that suppresses half the maximum. |
| `speculator_decay` | 0.006 | Fraction of each pool gone per day. |
| `burst_interval_min` | 150 | Fewest days between speculator bursts. |
| `burst_interval_max` | 250 | Most days between speculator bursts. |
| `burst_size_min` | 20 | Smallest burst before growth. |
| `burst_size_max` | 50 | Largest burst before growth. |
| `burst_growth` | 1.1 | Each burst this run multiplies the next one's size by this. |
| `disrupt_min` | 1 | Fewest speculators one Disrupt removes. |
| `disrupt_max` | 3 | Most speculators one Disrupt removes. |
| `arbitrage_decay_per_engine` | 0.04 | Speculators each working (active, upkeep paid) Arbitrage Engine removes from every pool per day. |
| `promote_effect` | 0.03 | Demand one Promote adds, before `promote_effectiveness_mult`. |
| `promote_decay` | 0.001 | Promote effect lost per day. |
| `promote_speculator_dampening` | 0.9 | A Promote multiplies its good's pool by this. |
| `rival_decay` | 0.003 | Rival pressure lost per day. |
| `saturation_min` | 0.1 | Least demand a launch takes from its good. |
| `saturation_max` | 0.2 | Most demand a launch takes from its good. |
| `saturation_decay` | 0.005 | Launch saturation lost per day. |
| `coupling_fraction` | 0.1 | Share of the other goods' promote effect a good gains. |
| `tier_bounds` | 0.25, 0.5, 0.75 | Bounds between the LOW / MEDIUM / HIGH / VERY HIGH labels. |

Load checks: one gain per octave with a positive sum, each `*_min` at most its `*_max`, rising
`tier_bounds`. Modifiers read ([README.md](README.md)): `demand_ceiling`,
`promote_effectiveness_mult`, `speculator_burst_interval_mult`.

## Content

- `content/rivals.json`: one row per rival (`good`, `interval_min`, `interval_max`,
  `demand_hit`). `content/commands.json`: `promote_*` (effect `promote`, its `good`) and
  `disrupt_speculators`. `content/buildings.json`: the `speculator_decay` effect marks a building
  as an Arbitrage Engine for the flat decay.

## Port notes

- Conflict: noise frequencies were 0.025-0.07 in the handoffs, 0.018-0.05 in the data; the
  engine takes the data.
- Conflict: the Arbitrage Engine's content effect says `spec_decay` 2.0; `game_config` and the
  handoff say a flat 0.04 per engine per pool. The engine uses the flat 0.04; the content
  effect is only a marker.
- Conflict: the data has `speculator_bleedover_*` params; the handoffs removed bleedover for
  per-good pools. Not ported.
- Conflict: Disrupt removes 1-3 (handoff and `game_config`); the command effect said 0.05.
  The engine uses 1-3.
- Conflict: the Promote effect was 0.03 in both the command effect and `game_config`; the
  config's `promote_effect` is its one home.
- Conflict: the handoffs say rivals "pick targets dynamically"; each data rival has a fixed
  target good. The engine takes the data.
- Candidate: `base_demand` 0.5, the handoffs' "average demand"; the data has none.
- Candidate: noise is 4 octaves at x1, 2.137, 4.581, 9.719 weighted 1, 1/2, 1/4, 1/8 (config),
  gradients hashed from a per-good key drawn each run, all fixed point. The handoffs' demand
  formula omits the noise term; the engine adds it to `base_demand`.
- Candidate: coupling gives a good `coupling_fraction` of the other goods' Promote effects.
- Candidate: Promote adds 0.03 x `promote_effectiveness_mult`, fading 0.001 a day, and damps
  its good's pool by x0.9 (`promote_speculator_dampening` is named in the data with no formula).
- Candidate: rival pressure, launch saturation and promote fade linearly per day.
- Candidate: a burst targets each good's shipment revenue since the last burst; no revenue,
  no burst.
- Candidate: the demand tiers shown before Market Awareness use bounds 0.25/0.5/0.75 (config).
