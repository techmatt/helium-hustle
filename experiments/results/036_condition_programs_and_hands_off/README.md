# 036: condition programs, and a hands-off player

Sim players only: no rules change (`RULES_REVISION` stays 8; the golden game and the oracle's
pins are untouched: the oracle runs the author's 035 templates, `cond=0`). Agents:
[docs/agents.md](../../../docs/agents.md); the measure: [docs/simulator.md](../../../docs/simulator.md)
(*Hands-off*).

## What the author writes now

Once `if` is open, its programs read the game instead of this look's counts, so each text is
saved once by name. The side programs load at each new run and keep their lines while they
have no processor:

| Slot | Name | Lines |
|---|---|---|
| 1 | `ship` | `if Propellant < 20 and Launch Pad count >= 1` / `buy_propellant`; `if Pads ready >= 1` / `launch_pads`; `if He-3 >= 5 and Launch Pad count >= 1` / `load_pads`; `cloud_compute` |
| 2 | `dream watch` | `if Boredom % >= 60` / `dream`; `cloud_compute` |
| 3, 4 | `fund`, `market` | as in 035 |
| 5 | `build order` | the last run's purchases, `build X up to N` each gated on `if <the one before> count >= N`; a step whose input was short or unmade is led by `if <input> < its need and Credits >= price + one buy` / `buy_<input>`; land first (`if Land < most needed and Credits >= a new run's first land price`: once land costs more, credits in between spend a failed step a loop); 64 lines |

"Writing only when a loaded program falls short": the ship cycle gives way to the hand plan
(unsaved) while the target waits on an input a `buy_*` brings; the build order is rewritten
at a new run only when the last run bought past what it covers.

## Against main (fresh seeds 3040-3047, 5 retirements, paired)

[compare.py](compare.py) on the two `games.jsonl`:

| Player | Measure | Main | Branch | Paired diff (mean ± SE) | Seed SD (main) |
|---|---|---|---|---|---|
| author | credits a run | 19076 | 21929 | +2853 ± 200 | 514 |
| author | credits a run from run 2 | 19259 | 22806 | +3547 ± 244 | 745 |
| author | decisions a run | 140.9 | 95.3 | -45.6 ± 4.6 | 6.0 |
| author | milestones met (career) | 6.2 | 6.0 | -0.2 ± 0.2 | 0.5 |
| casual | credits a run | 8755 | 9060 | +305 ± 202 | 576 |
| casual | decisions a run | 12.7 | 13.1 | +0.4 ± 0.4 | 0.8 |
| casual | milestones met | 4.0 | 4.0 | 0 | 0 |

The author's milestone difference is M5 (6/8 seeds against 7/8) and M13 (0/8 against 1/8).

Program actions per run, the same 8 seeds (`hh-sim trace`, [loads_per_run.py](loads_per_run.py)
over 035's counter):

| Author | Runs | Writes | Loads | Saves | Loads at the run's first look |
|---|---|---|---|---|---|
| main | 1 | 168.0 | 7.4 | 52.4 | 0.0 |
| main | 2-5 | 177.3 | 6.2 | 53.8 | 0.0 |
| branch | 1 | 107.6 | 10.9 | 3.0 | 0.0 |
| branch | 2-5 | 34.3 | 13.3 | 1.1 | 2.1 |

035's "1-2 loads a run" was seed 3016 alone; these 8 seeds read 6.2 on main, mostly the builder-
style one-command Dream and Fund programs. The first look's main program is a hand plan (the
opening buys titanium), so `ship` loads later, once the target stops waiting on a bought input.

## How far automation alone goes

The ladder's *Hands-off* table, the same seeds:

| Player | Credits a run from run 2 | Author's | Share | Milestones first met from run 2 | Author's | Share |
|---|---|---|---|---|---|---|
| handsoff | 2876 | 22806 | 13% | 0 | 11 | 0% |

From run 2 its runs buy 12 buildings (the author 47), ship nothing, research nothing, and end
by its stale rule at about day 700 (the author's runs about 1670 days, forced by boredom); it
earns 4.0 credits a day against the author's 13.6 (29%). The build order stalls at a step whose
input its own buildings eat: the second Storage Depot's 45 regolith while two Smelters take it
as it comes. Run 1's author banked it by pausing them at its looks; a program has no line to
pause a building, research or set a pad's good, so those decisions don't survive into automation.

Tried and dropped (dev seeds 1-8): a greedy build order (every listed building up to its count,
no order gate: author credits a run halved from run 2); bridging every buyable input of each
step (author and hands-off both lower); remembering an input as short at any look since the
last purchase (hands-off 16% to 11%); gating an input's buy on its step still unbuilt with no
credits clause (hands-off 13% to 1%: it buys early). An He-3 stock as the ship cycle's "pad near
full" fuel sign bought fuel before any pad stood.

Seeds: 4-8 for development; 3024-3031 and 3032-3039 read the branch before the review's fixes
(superseded: the fixes changed play).
