# 037: parameterized instructions

Rules revision 9. Programs get one flat line template, `<verb> <target> [amount]` (`Op::Act`),
with nine verbs. They also get six new readings and `every N days`, and a clause's right side
can be another reading times a number. Rules: [docs/rules/programs.md](../../../docs/rules/programs.md).
Rulings: [docs/rulings.md](../../../docs/rulings.md), *One flat program template*.

## Hands-off share (the ladder's *Hands-off* table)

Careers of 5 retirements. "Share" is the hands-off player's mean credits a finished run, from
run 2 on, divided by the author's on the same seeds.

| Read | Seeds | Hands-off credits/run | Author's | Share | Milestones from run 2 (hands-off / author) |
|---|---|---|---|---|---|
| 036 (main then, kept read) | 3040-3047 | 2876 | 22805 | 13% | 0 / 11 |
| research lines, pad verbs, set pads (dev) | 1-8 | 2919 | 20790 | 14% | 0 / 14 |
| + processor lead, pause/resume at cap, storage (dev) | 1-8 | 8192 | 19335 | 42% | 0 / 13 |
| + `max_entries` 64 -> 128 (dev) | 1-8 | 18437 | 18785 | 98% | 6 / 8 |
| fresh, before the review fixes | 3048-3055 | 18581 | 19381 | 96% | 5 / 8 |
| after the review fixes (dev) | 1-8 | 19332 | 21687 | 89% | 8 / 10 |
| **final, fresh** | **3056-3063** | **17745** | **21448** | **83%** | **7 / 13** |

The final read changes two things against 036: the hands-off player's programs and the
author's. The author now carries verb lines too, and its build order reaches further under
128 lines. So the 13% -> 83% move is a change in the share, not a hands-off gain against a
fixed author. Per run, the hands-off player's credits rose from 2.9k to 17.7k, and the
author's sat at 19-22k across reads.

## What moved it

1. **Processor lead** (`assign processors`). While no Launch Pad stands, the ship program has
   nothing to ship, so the build order takes every processor. A buy of a short input and the
   build it serves then land on the same day, before the clamp cuts the input back to its cap.
2. **Pause/resume at cap.** Smelters pause while titanium sits at its cap, and Fabricators while
   circuits do. A Storage Depot is bought while titanium is at its cap and credits reach 3× its
   price. Run 2 used to stall on a 6th Solar Panel whose titanium price (45.56) was above the
   cap (45). The `buy_titanium` bridge was skipped there by the output-cap rule.
3. **`max_entries` 128.** At 64 lines, a build order that also holds research lines and the
   lead stopped before the Launch Pad.
4. The research lines and pad verbs alone left the share at 14% (row 2; main on dev seeds
   was not read). Before the stalls were fixed, the run never reached them.
5. The review fixes cost about 10 points (dev: 98% -> 89%). The author gained from its build
   order carrying only the essential research, and every program's pass now stops when its
   own `assign processors` line cuts it.

## Still a hand

- **The stale retirement.** No reading sees "days since the last purchase". `retire` exists,
  but the hands-off player's rule can't be written as a condition.
- **Projects.** There is no `fund <project>` verb, so the hands-off player funds none.
- **A processor gained mid-run, or one stopped by unpaid upkeep.** It is assigned by the same
  hand rule as before. Programs could do it (`assign processors`), but no reading sees free
  processors per tab.

Kept reads: `ladder-branch-3048.txt`, `ladder-branch-3056.txt` (conductor's kept folder for 037).
