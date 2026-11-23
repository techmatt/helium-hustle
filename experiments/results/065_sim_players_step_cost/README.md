# 065 sim players for the step cost

Rules 19 (064) made every `if` cost a processor step. The program-driven players (the
author once `if` opens, and the hands-off loadout) were written for free `if`s; this item
rewrites them for the step cost. No rules or content change: random, greedy and casual
play the same games (casual never opens `if` on these seeds).

## What changed

The rule that drives every edit: under rev 19 a bare line that can't act (a `build` short of
credits, a `research` short of science, a `launch pads` with nothing full) costs one step and
nothing else, exactly what a false guard costs; a true guard costs its step and the line's.
So a guard only pays where the bare line would spend something (credits, cargo, a filler's
compute) or would take many steps.

- **Author's `ship` cycle** (`author/templates.rs`, `ship_cycle`). With no pad it is the
  filler alone (it was six `if`s and one compute: with the author's usual one processor,
  five days in six went to guards). Once a pad stands: fuel under one guard as before,
  `launch pads all` bare, `load pads all` `ship_loads` (2) times under one `He-3 above
  that many loads` guard (the verb: done, no energy, while the pad is full and cooling), and the pad-pointing lines only with two pads or more (He-3's bare). The
  filler runs × `fill` (8) a loop, not × 1. Both are new author knobs (player strategy).
- **Author's Dream watch**: unchanged (its filler × `dream_fill`, 1; 8 read the same on
  seeds 1-6: the watch rarely holds a processor).
- **Author's build order**: `build X up to N` and the plan's `research` lines bare, not under
  `if Credits above <price>` / `if Science above <price>`.
- **Hands-off loadout** (`configs/sim/handsoff_loadout.txt`): Program 1's `cloud_compute x10`
  is `x100` a pass, so its eight guards cost a few steps in a hundred, and the
  `if energy below 21%` / `idle x5` block is gone. Still a loadout plus its one hand move
  (the stale-run retirement); `HANDSOFF_PIN` re-pinned.

## The ladder, before and after

`hh-sim ladder` (5 players × seeds 1-3, careers of 20 retirements; the same seeds as 064's
read). *rev 18* is 064's kept read of the main before rules 19; *main* is bef4d9e, this
item's base; run credits are every finished run's mean, *from run 2* the runs after the first
(`table.py`).

| Player | Side | Run credits | From run 2 | Ships/run | Run ticks | Forced | Run 1 ticks |
|---|---|---|---|---|---|---|---|
| author | rev 18 (064's main) | 8,895 | 8,180 | 6.8 | 1,471 | 68% | 1292 |
| author | main (rev 19) | 2,787 | 1,751 | 1.1 | 1,069 | 7% | 1292 |
| author | 065 | 7,836 | 7,065 | 3.3 | 1,554 | 80% | 1292 |
| handsoff | rev 18 (064's main) | 23,353 | 23,399 | 13.2 | 1,987 | 10% | 1292 |
| handsoff | main (rev 19) | 9,469 | 8,785 | 8.1 | 1,779 | 52% | 1292 |
| handsoff | 065 | 22,109 | 22,090 | 10.4 | 2,027 | 97% | 1292 |

Recovered of 064's drop (run credits): author 83% (bar 50%: 5,841), hands-off 91% (bar
50%: 16,411); from run 2, 83% and 91%.
Hands-off table (credits a run from run 2 against the author): main 8,785 vs 1,751 (502%);
065 22,090 vs 7,065 (313%; rev 18's was 286%). Milestones first met from run 2: 3 of 3 both
sides. Run 1 and every milestone median (M1-M5, M11) are identical to main's: run 1 is
hand-played and the rewrite moves nothing before `if` opens.

**Fresh seeds 3137-3142** (author and hands-off, main's binary from `scripts/baseline.py`):

| Player | Side | Run credits | From run 2 | Ships/run | Run ticks | Forced | Run 1 ticks |
|---|---|---|---|---|---|---|---|
| author | main | 2,796 | 1,741 | 1.0 | 1,063 | 9% | 1292 |
| author | 065 | 7,858 | 7,070 | 2.9 | 1,584 | 92% | 1292 |
| handsoff | main | 8,045 | 7,267 | 6.1 | 1,635 | 54% | 1292 |
| handsoff | 065 | 22,214 | 22,182 | 10.3 | 2,030 | 99% | 1292 |

Milestone medians identical there too; hands-off share 314% (main 417%, its author was weak).

**`hh-sim balance`** prints the same lines on both sides (also with `--agents
author,handsoff --seeds 3`): the judge reads milestones and run 1's length and minutes only
(docs/principles.md, *The judge*), and none of those moved. It has no credits measure and no
hands-off cell, so this item can't show there.

## Findings

- **Runs now end by boredom.** Forced retirements: author 7% → 80%, hands-off 52% → 97%.
  More credits buy more and the stale-run retirement (nothing bought for 300 days) rarely
  fires; runs last longer and reach the boredom cap. Rev 18's author was at 68%.
- **Ships stay below rev 18's** (author 3.3 vs 6.8, hands-off 10.4 vs 13.2): the recovered
  credits are mostly Cloud Compute's. With one processor (most of the author's runs) a
  longer filler earns more than more loads (sweep below).
- **Author sweep** (`ship_loads` × `fill`, run credits; loads 2-8 × fill 1-4 on seeds 1-3,
  loads 1-3 × fill 4-16 on seeds 1-6): fill matters most (fill 1-2: 3.5-6.8k; 4:
  5.5-8.1k; 8: 7.9-8.6k; 16: 8.2-8.7k), fewer loads a batch better, ships falling
  as fill rises (2/8: 3.5 ships; 1/16: 1.5). 2/8 picked: near the top on credits with
  more ships than the longer fills.
- **Hands-off sweep** (Program 1's compute count with the idle block gone, seeds 1-6, run
  credits; the author's new templates in run 1): the shipped x10 with the idle block 9.2k;
  without it x20 6.3k, x30 9.2k, x40 10.1k, x60 18.6k, x80 20.9k, x100 22.1k, x120 20.1k,
  x150 16.1k, x200 8.6k. Too long a pass and the build guards come round too rarely. A
  larger ship batch (`if he3 above 20%` / `load_pads x10`) changed nothing past noise (x60
  18.8k, x100 22.0k).

- **The `load_pads` command against the verb** (diff review): with the command
  (`load_pads x2`) the author read 8,441 on seeds 1-3 (8,563 on 1-6); with `load pads all`
  twice, 7,836 (8,227). The command pays its energy with no pad to fill (a full pad
  cooling); the verb is done for a step. The verb is kept: it wastes nothing, and the
  4-7% is the size of a seed's swing in these chaotic careers, not a mechanism found.
- The sweeps ran before the verb change; their shape, not their level, is what they show.

## Files

- `table.py`: the tables above from reads' `games.jsonl`.
- The reads (ladders, fresh, sweeps, balance) are kept outside the repo (065's `out/keep/065_reads/`).
