# 064 general cleanup: how play moved

Rules revision 18 → 19: an `if` line costs a processor step (a false one jumps its whole
body in that step); a locked line is passed over free (a locked `if` with its body). The
author's build order no longer reserves 28 lines (`LEAD_LINES`). The fingerprint leaves the
config out (no play change; every pin moved once with it).

Three ladders on the same seeds (`hh-sim ladder`: 5 players × seeds 1-3, careers of 20
retirements), each from its own tree's root:

- **main**: 27c4c5c, revision 18.
- **A**: this branch's rules with `LEAD_LINES` restored.
- **B**: this branch.

**`LEAD_LINES` (A → B): no ladder game moves.** All 15 fingerprints are equal: the author's
build-order program never came within 28 lines of `max_entries` (128) on these seeds, so the
reserve never cut a line. The author's pins moved only with the rules.

**The rules (main → A).** Players whose programs are commands only (random, greedy, casual)
play the same games. The program-driven players lose most of their run-2+ income: their
condition programs (the author's `ship` cycle is ~14 `if`s around 7 acts) now spend a step
on every `if`, so with one or two processors a loop of the cycle takes several days.

| Player | Run credits (main) | (B) | Ships/run (main) | (B) | Forced retirements (main) | (B) | Run ticks (main) | (B) |
|---|---|---|---|---|---|---|---|---|
| author | 8,895 | 2,787 | 6.8 | 1.1 | 68% | 7% | 1,471 | 1,069 |
| handsoff | 23,353 | 9,469 | 13.2 | 8.1 | 10% | 52% | 1,987 | 1,779 |
| greedy | 8,857 | 8,857 | 19.9 | 19.9 | 100% | 100% | 1,644 | 1,644 |
| casual | 11,277 | 11,277 | 16.9 | 16.9 | 100% | 100% | 1,607 | 1,607 |

Hands-off table (credits a run from run 2, against the author): main 23,399 vs 8,180
(286%); B 8,785 vs 1,751 (502%). Milestones first met from run 2: 3 of 3 in both. Milestone
medians (M1-M5, M11) are unchanged for every player: run 1 is played by hand.

Run 1 is untouched (the author's runs until `if` opens are hand-played). The fall is the sim
players' strategy meeting the new step cost, not a bug: their condition programs were written
when `if`s were free (059-062). Rewriting them around the step cost (fewer, broader guards,
more processors on the ship cycle) is a sim-player item, deferred.
