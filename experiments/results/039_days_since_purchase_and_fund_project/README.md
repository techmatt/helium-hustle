# 039: `days since last purchase` and `fund <project>`

Rules revision 11. Programs get a `days_since_purchase` reading (days since this run last
bought a building, land or research item, by hand or by a program line) and a
`fund <project> [N | all]` verb, which sets the project's daily rate on every cost resource as
the Projects panel does. Each has a 0/1 switch in the programs config (`days_since_purchase_reading`,
`fund_verb`). Rules: [docs/rules/programs.md](../../../docs/rules/programs.md). Ruling:
[docs/rulings.md](../../../docs/rulings.md), *`days since last purchase` and `fund <project>`*.

## Hands-off share (the ladder's *Hands-off* table)

Fresh seeds 3080-3087: 8 careers of 20 retirements a side (`ladder --agents author,handsoff
--seeds 8 --first-seed 3080`). Main is 7cf739e (038, rules 10), run from its own worktree on
main's config. The author plays identically on both sides (18,862 credits a run, the same
milestones), so this compares the hands-off player alone, but it changes several things in
that player at once (below).

| Side | Hands-off credits/run (run 2+) | Author's | Share | Milestones first met from run 2 (hands-off / author) | Hands-off decisions, runs 2-20 (all seeds) |
|---|---|---|---|---|---|
| main | 19,080 | 18,817 | 101% | 6 / 17 (35%) | 624 |
| **039** | **19,421** | **18,817** | **103%** | **12 / 17 (71%)** | **152** (one per reset: the load) |

Full tables: [ladder-main-3080.txt](ladder-main-3080.txt), [ladder-branch-3080.txt](ladder-branch-3080.txt).
Per seed ([runs.py](runs.py) over the reads' `games.jsonl`):

| Seed | Mean run ticks (main / 039) | Forced retirements of 19 (main / 039) | Credits/run (main / 039) | Projects completed (039; main: none) |
|---|---|---|---|---|
| 3080 | 1828 / 1851 | 1 / 0 | 20801 / 20441 | - |
| 3081 | 1620 / 1663 | 13 / 13 | 17730 / 18103 | - |
| 3082 | 1690 / 1708 | 10 / 16 | 19831 / 19816 | - |
| 3083 | 1616 / 1572 | 11 / 17 | 18548 / 17263 | - |
| 3084 | 1782 / 1759 | 0 / 8 | 19512 / 20296 | Foundation Grant |
| 3085 | 1774 / 1753 | 1 / 3 | 19538 / 19729 | Foundation Grant |
| 3086 | 1762 / 1737 | 0 / 5 | 18570 / 19755 | Foundation Grant |
| 3087 | 1789 / 1876 | 3 / 13 | 18110 / 19965 | Foundation Grant |

- **No hand decisions after the reset.** Every run from run 2 has one decision, the load and
  seating at its reset. On main, the stale retirement and the reassignment of processors
  gained mid-run were extra decisions by hand.
- **Program retirements:** 77 of 152 runs end on the `retire` line, and the rest at the boredom
  cap (main: 113 by the hand rule, 39 at the cap). Research and land now count as purchases,
  and a run's late research (one every ~150-200 days in dev traces) keeps resetting the reading.
  So more runs reach the cap than under main's hand rule, which counted buildings only.
- **Funding:** Foundation Grant completes on 4 of 8 seeds, and every dev seed (1-3) funded it.
  M11, *First Major Project* (4 of 8 seeds, median run 11), is 4 of the 6 more milestones
  from run 2. The author meets M11 on no seed (why was not read).

## What changed in the hands-off player

1. **Its lead** (in the build order, inside the author's 28-line room) gains seat lines. A side
   program short of its processor takes one once enough work: the first at 3, the rest
   together at 2 plus their number. This replaces the hand reassignment.
2. **The Dream watch** gains these lines before its filler:
   - every 49 days, with 3,000 credits or more, `fund` seven projects at 3 a day;
   - every 9 days, once a pad stands, `assign processors 1 all`;
   - `if days since last purchase >= stale_days` then `retire`;
   - the research tail, which no longer fits the build order.
3. Three traps found on the way:
   - **The 128-line cap.** A 99-line build order plus a 30-line lead is over 128, so the whole
     lead was dropped (dev seed 2 collapsed to 2,215 credits a run).
   - **`every N` and the pointer.** A one-processor loop reads its guards every other day when
     Dream runs on alternate days. A true block's body also takes the step on its own day. An
     even period can therefore never land, and two periodic blocks shadow each other; hence the
     odd periods, with funding first.
   - **Drained inputs.** Chemical Energy and Grid Recalibration drain propellant and energy, so
     they are left out of the funding list.
