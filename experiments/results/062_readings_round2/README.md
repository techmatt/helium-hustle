# 062 Readings, round 2

Rules revision 17. 060's most useful missing readings, with the list growing by two only after
research ([docs/rules/programs.md](../../../docs/rules/programs.md), *Conditions*):

- **Amounts for capped resources.** A capped resource (and boredom) still has one entry in the
  editor's reading list; its number box has a **% / amt toggle** that switches the clause
  between `percent` and `amount` ("Propellant below 20%" or "below 20"). An amount reads at
  most the cap, as the panel shows it. Credits and Science stay amount-only.
- **Energy per day** (`energy_per_day`): energy's last full day's rate (`Run.day_net`, the
  Resources panel's per-day number), negative in a loss.
- **Days since a purchase** (`days_since_purchase`): days since this run last bought a
  building, land or research, by hand or by a program (`Run.last_purchase`, restored from 039).
- **The gate:** both open while **Process Telemetry** is done this run (new research row: 50
  science, always visible once research opens at 1.6, its own category *Program Analysis*, so
  1.b's two-item cluster rule is untouched; new effect `program_readings`). Before it they are
  not offered, a new clause using one is refused, and a held or loaded one reads false.

## Sim players

Only the hands-off loadout changes (`configs/sim/handsoff_loadout.txt`). Its Dream tab
(Program 3) gains, ahead of its lines:

```text
if science above 49
  research process_telemetry
if days since a purchase above 299
  retire
```

so the run's own program retires a stalled run; the player's hand stale-run retirement
(`stale_days` 300) stays as a backstop for runs where Program 3 has no processor. Since a
researched reading can't be written before its research, the player writes such a tab
without those blocks the first time and rewrites it whole once the research is done (once a
career: the library holds it after), as a player would.

| Read | Build | Hands-off credits a run from run 2 | Author's | Share | Milestones | Voluntary runs with no hand move after the load |
|---|---|---|---|---|---|---|
| ladder, seeds 1-3, 20 retirements | 061 loadout | 22,708 | 8,180 | 278% | 3 | 3 of 52 |
| ladder, same | lines in Program 1 (tried) | 21,125 | 8,180 | 258% | 3 | 43 of 51 |
| ladder, same | **lines in Program 3 (kept)** | 23,399 | 8,180 | 286% | 3 | 31 of 52 |
| fresh, seeds 3131-3136, 10 retirements | 061 loadout | 23,043 | 8,273 | 279% | 7 | 8 of 46 |
| fresh, same | **lines in Program 3** | 23,122 | 8,273 | 280% | 7 | 26 of 48 |

The last column counts runs boredom didn't end with a single hand decision, the load
([runs.py](runs.py)): with the `retire` line, mostly runs the program retired; the 061 rows'
few are runs that ended without the hand retirement (likely the career's end). In Program 1 the `research` line costs a done step every
loop once science stands above 49 all run, which cost 7%; in the Dream tab it runs only
once Program 3 is seated. Credits are flat on fresh seeds; the gain is the hand move.

Energy per day isn't adopted: the loadout's brownout guards (`energy below 21%` / `idle`, the
energy guards on building and fuel) would need a 060-style loadout read to replace, which
this item didn't run.

**Other players, by the new research alone.** The author's research list doesn't include
Process Telemetry, so its games are unchanged (its ladder figures above match 061's). Players
that buy research beyond a list now also buy it (50 science, from 1.6): the oracle (its
seed-2 pin moved), the greedy builder (cheapest research first) and random. Their short
pinned games (2 retirements, `PINNED`) didn't move; their longer careers weren't read. The
autopilot's second driver (`all_research=1`) buys it too. The slow-lane test
`a_fresh_career_on_the_autopilot_reaches_every_step` still meets every milestone and
main-quest step through the Telescope (1.9) and the ending, on this build.

**Library room.** The rewrite is once a career only if the library has room for the
`hands-off 3` save (12 slots); with no room, each run would write the tab without its
blocks and rewrite it mid-run (one more hand decision). On the reads above it saved: 26 of
48 fresh runs had the load as their only decision.

Reads kept outside the repo: conductor `kept/helium-hustle/062_readings_round2`.
