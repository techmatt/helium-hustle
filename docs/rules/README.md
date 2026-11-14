# Rules

The rules as implemented, rules revision **1** (`RULES_REVISION` in `crates/hh-core/src/lib.rs`).
History: [../rules-history.md](../rules-history.md).

**Everything here is a placeholder.** The engine scaffold ships one stand-in module,
`placeholder`, just big enough to exercise the engine. The rules port replaces it whole;
nothing in it is game design.

## `placeholder` (variant `basic`)

Resources (`content/resources.json`): Credits, Energy. Building
(`content/buildings.json`): Solar Panel, costs 10 credits, makes 1 energy a day.

Each day (tick), in this order:

1. A sunlight factor is drawn uniformly from `1 - sunlight_spread` to
   `1 + sunlight_spread` in steps of 0.000001 (RNG purpose `sunlight`).
2. Credits rise by `credits_per_day`.
3. Each building makes its content `produces` per copy, times the sunlight factor.
4. Energy is capped at `energy_cap`.

**Buy** (the one action, between days): a building's price is its content `cost` times
`cost_growth ^ owned` (the power through `libm`, then floored to 0.000001: the floor of the
float, so 1.15² comes out 1.322499). Refused, with nothing changed, if any resource is short.
A price past the number range (about 9.2 × 10^12) is never affordable; credits and
production stop at that range instead of overflowing.

A new game starts at day 0 with `start_credits` credits and nothing else.

| Param | Baseline | Meaning |
|---|---|---|
| `start_credits` | 20 | Credits at day 0. |
| `credits_per_day` | 1 | Flat credit income. |
| `energy_cap` | 100 | Most energy held. |
| `sunlight_spread` | 0.2 | Daily production varies by up to this fraction either way. |
| `cost_growth` | 1.15 | Each building owned multiplies the next one's price by this. |
