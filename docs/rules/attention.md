# Attention (module `attention`, variant `minimal`): DRAFT

**Draft (049).** The minimal Attention model that brings Act 1's adversaries in: the
speculators (beat 1.6) and the rival AIs (1.10). Thresholds, drivers and decay are
placeholders ([../arc.md](../arc.md)).

## Rules

**Attention.** Each adversary, `speculators` and `rivals` (fixed ids in code), has one
Attention number, kept in the career: a retirement keeps it, and it decays through the
retirement gap ([career.md](career.md)). A timeline's end clears it (draft,
[timeline.md](timeline.md)). Attention never goes below 0.

| Adversary | Driver | Rises by |
|---|---|---|
| speculators | units sold | each launch's cargo × `speculator_per_unit` |
| rivals | land claimed | each Buy Land's land (`land_per_purchase`) × `rival_per_land`, by hand or program |

**Decay.** Each day, before demand is set ([the day](README.md#the-day), step 4, inside
demand's update), each adversary loses its `*_decay` (linear), floored at 0. Each day of the
retirement gap does the same.

**Present (one tier).** An adversary is present while its Attention is at or above its
`*_threshold`, and gone once it falls below. While present:

- *speculators*: speculator bursts land ([demand.md](demand.md)); while absent a burst that
  comes due lands nothing (the next is still scheduled).
- *rivals*: each rival dumps on its good when its day comes ([demand.md](demand.md)), and
  the land price is multiplied by `rival_land_price_mult` ([economy.md](economy.md)).

**Techniques.** The *neutralize* stub sets an adversary's Attention to 0: the Contest Claims
command (effect `neutralize`, adversary `rivals`; open once beat 1.10 sets its flag) runs it.
Disrupt Speculators (open once side quest 1.a sets its flag) also takes `disrupt_attention`
off the speculators' Attention per execution ([demand.md](demand.md)).

**Quests.** The `adversary_present` condition holds while the adversary is present
([events.md](events.md)): beats 1.6 and 1.10.

**Display.** The view's `adversaries` gives each adversary's Attention, threshold and
presence (the Adversaries panel, whose tile beat 1.6 opens: the `adversaries` panel).

## Actions

None of its own: Contest Claims and Disrupt Speculators are commands run by programs.

## Params

<!-- params:attention -->
| Param | Baseline | Range | Meaning |
|---|---|---|---|
| `speculator_per_unit` | 1 | 0 to 1000 | Speculator Attention per unit of cargo launched. |
| `speculator_threshold` | 500 | 0 to 100000000 | Speculator Attention at which speculators are present. |
| `speculator_decay` | 0.5 | 0 to 1000000 | Speculator Attention lost per day (linear). |
| `rival_per_land` | 1 | 0 to 1000 | Rival Attention per unit of land claimed. |
| `rival_threshold` | 100 | 0 to 100000000 | Rival Attention at which the rival AIs are present. |
| `rival_decay` | 0.02 | 0 to 1000000 | Rival Attention lost per day (linear). |
| `rival_land_price_mult` | 3 | 1 to 1000 | Land price multiplier while the rivals are present. |
| `disrupt_attention` | 20 | 0 to 1000000 | Speculator Attention one Disrupt Speculators takes off. |
<!-- /params -->

## Content

`content/commands.json`: Contest Claims (`neutralize`), Disrupt Speculators.
`content/events.json`: the `adversary_present` conditions of 1.6 and 1.10.
