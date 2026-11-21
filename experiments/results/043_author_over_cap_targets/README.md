# 043: the author never waits on a target past its caps

Player strategy only (`crates/hh-sim/src/agents/author.rs`); no rules change, no golden move.

## The change

`Builder::resolve` already plans storage for a building priced past a cap (a Storage Depot, a
Battery for energy). The deadlock was the storage itself past its cap: the Storage Depot's
cost grows 1.5x a buy while the 75 regolith of cap it adds grows linearly, so the 9th depot
(769 regolith) can never fit the 650 cap that 8 depots give. Every target that resolved to it
(Data Center, Refinery, Research Lab, Launch Pad, Fabricator, and anything needing energy,
through a Solar Panel whose titanium price is past its cap too) then waited on it, with the
regolith eaters paused.

Now `reachable_target` takes the first of *the resolved step, the building itself* that fits
under every cap, and skips the building when neither does. The `toward_maker` step for an
input nothing makes gets the same check.

Tried first: skip every over-cap resolved step (no fallback to the building itself). Under
the autopilot's boost the author owns 8 depots by day 124 of run 1, so this skipped the
Arbitrage Engine (resolved to a Solar Panel, then the depot) and the Launch Pads: M8 and M12
went unmet on all three seeds. Planning storage was no different from that, since `resolve`
already does it.

## Done-when: *Everything* on the autopilot (native, default boost, budget 200,000)

| Seed | Before (main) | After |
|---|---|---|
| 2 | left M10 at 200,000 days | every step, 20,301 days (Q7 6,770, Q18 20,301) |
| 3088 | every step, but only at 180,165 days (M10 last) | every step, 22,777 days |
| 3096 | left M12 at 200,000 days | every step, 22,652 days |

Main's seed 2 run: run 1 deadlocked from day ~124 until its stale retirement (4,164); M8 came
in run 3, M14 and Q7 in run 4. After: run 1 retires at 3,120 with M14 met, M12 and M8 follow
in run 2 by day 3,404, Q7 at 6,770.

The slow test `a_fresh_career_on_the_autopilot_reaches_every_step` (seed 2) asserts every
milestone and quest step and the ending again; ~17 s.

## Balance judge (`hh-sim balance`, experiments/balance.json, 2 seeds)

`casual` is unchanged. `author`:

| Line | Before | After |
|---|---|---|
| M5 Positive Credit Flow | run 2, 1/2 seeds | met in band |
| M6, M9, M10, M11, M13, M14 | not reached in 20 runs | not reached in 20 runs |
| M7 Boredom Management | met in band | **not reached in 20 runs** |
| M8 Diversified Trade | not reached | run 16, 1/2 seeds |
| M12 Adversary Subverted | run 17, 1/2 seeds | run 2 (early: target 8-15) |
| Run 1 length | 1,443 ticks | 1,817 ticks (target 800-1,100) |
| Run 1 real time (mixed) | 18.9 min | 22.2 min (target 25-35) |

11 author lines before, 11 after. M7's loss is the one regression: the author no longer
sits paused at the depot wall, so its runs spend boredom differently; not chased here (the
judge's targets are placeholders, docs/rulings.md).

## Pins moved

`crates/hh-sim/tests/cli.rs`: the ladder's `author` and `handsoff` games (seeds 1-2) and the
oracle test's four fingerprints (the oracle's chosen target goes through `reachable_target`
too, so one past its caps is now passed over). `random`, `greedy` and `casual` play the same
games.

## Side effect to watch

The fallback goes from the end of `resolve`'s chain straight back to the building: an energy
user whose Solar Panel waits on a ninth depot is now bought with no spare energy (the panel's
titanium is past its cap too), which can deepen a brownout. It may be part of M7's loss.
