# 035: program loadouts, and `rate` that reads demand

Rules revision 8 ([rules-history](../../../docs/rules-history.md)). Rules:
[programs.md](../../../docs/rules/programs.md) (*Conditions*, *Loadouts*, *Actions*).

## Sim: against main

Paired seeds 3016-3023 (fresh, from the ledger), 5 retirements, author, casual
(`author:min_gap=120`) and greedy (the builder), `hh-sim ladder` on main (1080135) and on
this branch. Every table cell matches: ticks, decisions, credits, shipments, milestones and
run-1 objectives. Loading a saved program puts the same lines in the same look as writing
them would, and no sim player reads `rate`, so neither change moves sim play. (The
engine-speed line differs only by load on the box.)

Program actions per run, seed 3016 (`hh-sim trace`, counted by
[count_loads.py](count_loads.py); run 6 is the career's last look):

| Player | Run | Writes | Loads | Saves |
|---|---|---|---|---|
| author | 1 | 192 | 6 | 68 |
| author | 2-5 (mean) | 168 | 1.3 | 55 |
| casual | 1 | 7 | 0 | 7 |
| casual | 2-5 (mean) | 7 | 0 | 7 |
| greedy | 1 | 5 | 101 | 5 |
| greedy | 2-5 (mean) | 0.25 | 126 | 0.25 |

What it shows: loadouts help a player whose programs are general. The builder's
one-command programs (`buy_titanium` × 1 and the like) are all loads from run 2 on. The
author's are written for the moment: the standing cycle holds counts (`loads`, the fuel
short, the target's missing inputs) and the construction program names this look's
target, so a saved copy almost never matches what it wants next; only Dream and Fund
programs come back as loads. A player who writes conditions (`if Energy rate < 0`,
`build X up to N`) instead of counts would reuse far more: the sim author doesn't write
that way yet.

## `rate`, before and after

| Case | 031 (rev 7) | 035 (rev 8) |
|---|---|---|
| Steady brownout (six Data Centers on one Solar Panel) | 0 | -7 every day (5 made, 12 wanted) |
| Draining stock | the drain | the drain |
| Capped resource | uncapped (the cut was never in it) | uncapped |
| Smelter starved of energy, regolith on hand | regolith: actual use | regolith: actual use (no shortage of it) |
| Cloud Compute failed for 0 energy | nothing | 2 wanted, unmet |
| Project funding past the stock | what it took | what it was set to take |
| Program `build`/`buy land`, pad loading, boredom growth, Dream | left out | counted |

The Resources panel's per-day number is the same quantity, for the next day: in a
brownout Energy reads "−7 SHORT" (red; the stock's actual change in the tooltip) where it
read 0, and at a cap it reads the uncapped surplus with its CAP tag, as before.

Tests: `rate_reads_demand_so_a_line_under_rate_below_zero_runs_every_day_of_a_brownout`,
`unmet_demand_counts_only_on_the_input_that_ran_short`,
`a_failed_command_and_a_short_project_count_as_demand`,
`a_loadout_saved_in_one_run_loads_and_runs_in_the_next`,
`a_loaded_line_this_run_has_not_unlocked_fails_or_reads_nothing` (`crates/hh-core/src/tests/logic.rs`);
the e2e saves a program in run 1, retires, loads it in run 2 and watches it run.
