# 004 rules revision 1: port notes

Working paper for the port of the Godot build's Arc 1 (`reference/`) into `hh-core`
(rules revision 2). The current rules are in [docs/rules/](../../../docs/rules/README.md);
this file records where the sources disagreed and what the port had to invent.

## Source conflicts (data wins unless a ruling says otherwise)

| # | Topic | Handoffs say | Data says | Engine does |
|---|---|---|---|---|
| 1 | Boredom cap | 500 + 100 per Recreation Dome | 1000 (`resources.json`, `boredom_max`) | **Ruling:** 500 in config (`boredom.base_cap`) + the dome's `store` 100 |
| 2 | Land purchase | 15 credits, x1.5, 10 land | 200 credits, x1.5, 15 land | data |
| 3 | Noise frequencies | 0.025-0.07 | 0.018-0.05 | data |
| 4 | Ideology ranks | rank n costs 100 x 1.5^(n-1) | `rank_thresholds` 70/175/333/570/925 | geometric: the prompt's design doc and `handoff_active` ("formula has changed") make the thresholds stale |
| 5 | Starting credits bonus | best run credits / 100 | `career_credits_bonus_fraction` 0.02 (/50) | data (0.02) |
| 6 | Arbitrage Engine | flat 0.04 per engine per pool | building effect `spec_decay` 2.0; `game_config` 0.04 per building | flat 0.04 (game_config and handoff agree); the content effect is a marker |
| 7 | Speculator bleedover | removed (per-resource pools) | `speculator_bleedover_*` params | not ported |
| 8 | Disrupt amount | 1-3 | command effect 0.05; game_config 1-3 | 1-3 |
| 9 | Microwave Receiver flag | - | building requires `microwave_power_active`, project sets `microwave_power_completed` | one flag, `microwave_power_completed` (else unreachable) |
| 10 | Microwave Receiver output | produces energy | no production (enables Buy Power) | data |
| 11 | QShipment unlocks | Foundation Grant, Retirement, Projects | Retirement only; QMarket unlocks Foundation Grant and Projects | data |
| 12 | Geopolitical Intelligence visible | QAutomate completed | QMarket completed | data |
| 13 | Buy Ice | needs an Ice Extractor | no requirement | data (Fabricators cost ice before Ice Extractors exist) |
| 14 | Research items | 12 | 11 | data (11) |
| 15 | Overclock cap | command text "+200%" | Overclock Boost: 150% to 200% | base cap 1.5 (config), research 2 |
| 16 | Load per execution | 5 | command effect 5; game_config 5 | config (one home) |
| 17 | Promote effect | - | command effect 0.03; game_config 0.03 | config (one home) |
| 18 | Dream | flat: -2 boredom x 1.05^humanist rank | effect `dream_echo`; phase events say Dream effectiveness falls to 20/15/10% | **Ruling:** flat Dream (module `dream`, variant `flat`); phase-event text left as placeholder |
| 19 | Rival targets | "pick targets dynamically" | each rival has `target_resource` | data |
| 20 | Lunar Cartography, Deep Core Survey | - | unlock `event_unlocked`, but no event enables them | data: unreachable (candidate) |
| 21 | Research `unlocks_commands` | - | duplicates each command's `requires` | dropped: the command's `requires` is the one home |
| 22 | Milestones | removed | `game_config.milestones` | not ported |
| 23 | Starting land | - | 30 land, a Solar Panel and a Data Center | the 30 is free land: starting buildings take none (and selling one refunds none) |
| 24 | Arbitrage Engine requirement | - | research `market_analysis` (not a research id; a category name) | `market_awareness` |
| 25 | Buy Power scaling | re-derived from the career at run start | - | career peak power folds in at retirement, so the bonus is fixed within a run |

## Port adaptations (candidates: needed to play, not specified)

- **Demand base** 0.5 (`demand.base_demand`): handoffs call 0.5 "average demand"; data has none.
- **Noise:** 4 octaves at x1, 2.137, 4.581, 9.719 with weights 1, 1/2, 1/4, 1/8 (config);
  gradients from a hash of a per-good key drawn each run; all fixed point, no floats.
- **Coupling:** a good gains `coupling_fraction` x the other goods' Promote effects.
- **Promote:** +0.03 x `promote_effectiveness_mult`, fading 0.001 a day; a Promote multiplies
  its good's speculator pool by 0.9 (`promote_speculator_dampening`, named in data, no formula).
- **Rival pressure, launch saturation, promote** fade linearly per day.
- **Burst target:** each good's shipment revenue since the last burst; no revenue, no burst.
- **Launch cooldown** 10 days (handoff number; not in data).
- **Manual launch** needs some cargo (not a full pad) and fuel, and works on a paused pad.
- **A pad's good** can change only while it is empty (or carries that good).
- **Processors:** a new run puts every processor on Program 1; a new Data Center's
  processor starts unassigned; losing processors unassigns from the last program first.
- **Program lines** have a repeat count; at most 64 lines (config).
- **Effect commands** (Load, Launch, Dream, Overclock, Promote, Disrupt, Fund) need their
  full cost; pure conversions (Buy *, Cloud Compute, Idle) run partially.
- **Output-cap skip** for any command whose every product is capped (boredom isn't a product).
- **Overclock:** each execution is its own x(1 + 0.05) stack for 5 days (x 1.03^rationalist
  rank, whole days), starting at the next building pass; the product is capped.
- **Peak power** is gross energy buildings made in a day.
- **Quests** ignore a `game_start` run filter (the chain picks up where it left off); a
  project's `quest_active` gate holds while the quest is active or after it completes.
- **One-off events** fire once per run; `shipment_completed` counts this run's shipments.
- **AI Consciousness Act** command boredom values replace the commands' own (Cloud Compute
  0.1 to 0.2; Load Launch Pads 0 to 0.3; Disrupt Speculators 0 to 0.5).
- **Command boredom** is flat per execution, even when a conversion runs partially.
- **Starting buildings** count toward the cost curve (they are owned, not bonus): the
  second Solar Panel costs 30 credits, as in Godot's opening.
- **Loading priority** orders Load Launch Pads: pads load in their goods' priority order.
- **Content flags and effects** instead of ids in code: Disrupt Speculators' `sets_flag`
  (`used_disrupt_speculators`), Market Awareness's `reveal_demand` effect.
- **Demand tiers** (display before Market Awareness): bounds 0.25/0.5/0.75 (config).
- **The day:** a tick runs with the run's current day, which rises at the end (Godot's
  "advance day" last); forced retirement is checked after it, and the next run starts at once.

## Rounding decision

The 002 cost curve was `libm::pow` then floor, so 1.15^2 came out 1.322499. The port uses
`Fx::pow_int`: whole powers by repeated multiplication at 10^-18 in `i128`, floored once to
10^-6 (1.15^2 = 1.3225). Every whole power in play uses it: building cost curves (1.5, 1.4),
land, ideology ranks, per-rank bonuses (1.05^r, 0.97^r, 1.03^r) and burst growth. Only
fractional powers go through libm: boredom resilience (0.995^(days/400)) and the ideology head
start (a continuous rank). It matters: 0.97^r and 1.05^r are not exact in `f64`, so the float
floor would land a step low on discounts and bonuses, and a cost curve should not depend on
float rounding at all.

## Run read (scripted policy, `hh-sim objectives reference/scenarios/run1_fresh.json`)

Seed 1: QPower tick 52, QExtract 157, first shipment and QShipment 303, QAutomate 789,
forced retirement 1098 (boredom 500). Seeds 2 and 3: the same first three ticks (the opening
has no randomness), QAutomate 527 and 549, retirement 1109 and 1105.
