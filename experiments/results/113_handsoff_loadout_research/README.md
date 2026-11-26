# 113: the hands-off loadout under rules 38's energy

The old loadout (060's L5 line, three tabs) walled in seed 1's run 3, its first hands-off run:
0 shipments, 6 buildings. State at day 1313 (`hh-sim trace --at 4800`): energy 0, titanium 0,
4,478 credits. A run starts with one Solar Panel (+6) and one Data Center (-4), no titanium
and one processor. The Solar Panel line needs titanium, and so does every source of titanium
but Buy Titanium, which the loadout never ran. The Smelter sat behind the `energy > 70%` build
line, which cloud compute held near 20%. With one processor, all of it on Program 1, Programs
2 and 3 (shipping, science) never ran.

## Candidates (seed 1, run 3 unless noted; `out/113_handsoff_loadout_research/`)

| Loadout | Change | Run 3 shipments | Run 3 buildings | Run 3 days |
|---|---|---|---|---|
| old | (rules 38) | 0 | 6 | 1430 |
| v1 | one tab: Buy Titanium when short, Solar at energy < 50%, Smelter/Excavators early, Fabricator and Data Centers later | 0 | 10 | 650 |
| v2 | Solar whenever credits allow; Launch Pad before Refinery; no early storage | 1 | 7 | 600 |
| v3 | v2 + Fabricator, Data Centers on 12 circuits, storage at titanium 90% | 0 | 11 | 770 |
| **v4** | v3 without Fabricator, Data Center or Research Lab; loading needs energy > 10%; compute only above 50% | **5** | **21** | **1616** |
| v5 | v4 with twice the pads, refineries, smelters and 6 Excavators from the start | 0 | 11 | 660 |
| v6 | v4 + a second pad and refinery once credits pass 500 | 3 | 16 | 1070 |

v3 shows the deadlock: a Fabricator's 5 energy took energy to 0, so it had no credits for
Cloud Compute, Solar or loading. What decided it: a run here is energy-bound, not step-bound.
A second Data Center buys a processor (steps) for 4 energy, and a step without energy earns
nothing.

v4 shipped, as the new `configs/sim/handsoff_loadout.txt`, over seeds 1 to 4 and 5 retirements:

| Seed | Hands-off runs | Shipments a run | Buildings a run |
|---|---|---|---|
| 1 | 3, 4, 5 | 5, 5, 5 | 21, 21, 21 |
| 4 | 4, 5 | 4, 4 | 21, 21 |
| 2, 3 | none | | |

On seeds 2 and 3 the author's runs never open `if`, so the loadout never loads (every run
there is the author's, about 720 days and 0 shipments). That's the author under rules 38,
not this loadout's, and is left for a later item.

Rerun: copy `try.py` and `peek.py` (this folder) into `out/113_handsoff_loadout_research/`
(they find the repo two levels up), build `hh-sim` release, then `python try.py <loadout>
<seeds> <retirements>`; `peek.py <loadout> <run start tick> <days...>` reads seed 1's state at
given days. The candidates are in `loadouts/` (`orig.txt` is rules 38's).
