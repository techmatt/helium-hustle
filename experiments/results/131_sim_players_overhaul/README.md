# 131: sim players overhaul

Rules revision 46. `hh-sim axes` against 130's bars (unchanged), with rank 5 now at 730
score (two significant figures). **Final read: fresh seeds 3159-3161**
(`final_3159-3161.txt`), the shipped players. `pre-review_3156-3158.txt` is the same player
before the diff review's bank fixes (fresh seeds too, so not paired with the final; the
bank variants read identically on 3159-3161, a paired check). Dev reads on seeds 1-3, each
after a player change: `dev2` (the bank during Trade, side-by-side funding), `dev7` (plus
run-1 science and credits on hand to them; the shipped shape), `dev9` (per-resource rates
with credits past the target's price; not shipped).

## Against the bars

| Bar | Target | Final (3159-3161) | Pre-review (3156-3158) | Met |
|---|---|---|---|---|
| Skilled run 1 finishes BA and LC | most seeds | **2/9** (R 1/3: 3159, 3161) | 4/9 (N 3/3, R 1/3) | **no** |
| Skilled run 1 short of rank 5 | best < ~80% | 0-23%, 9/9 | 0-23% | yes |
| Skilled run 2 reaches rank 5 | every axis | 9/9 (130: R 1/3) | 9/9 | yes |
| Parity: run-2 day within ±15% of the 3-axis mean | each axis | mean 1,292: N 1,185 (−8.3%), H 1,327 (+2.7%), R 1,363 (+5.5%) | N −15.5%, H +14.8%, R +0.8% | yes |
| Medium rank 5 within one run | 9/9 | 9/9, days 951-1,095 | 9/9 | yes |

Dev seeds 1-3 with the `dev7` player: run 1 BA+LC 7/9.

## What changed in the committed players (agent code, `author.rs`)

- **Bank** while 1.4 Trade is under way: two Research Labs first, then nothing bought until
  credits and regolith after the purchase cover Blueprint Archive's and Lunar Cartography's
  unpaid costs (each capped at the resource's cap).
- **Side by side**: once open, both funded together (credits on hand at the cap; regolith
  and science at the cap; in run 1 science goes to them before Fund's research).
- **Fund seated before the tool**: the tool's processor count (growing with boredom in a long
  run) no longer takes Fund's two. This fixed Rationalist run 2 (130: 1/3; now 9/9 overall).
- The run-2 science starvation 130 hit twice did not recur in any 131 read.

## Why run 1 still misses

Traces (3157 Humanist, seed 1 Nationalist) show both projects' rates switching off and on
look by look: the author funds a project only while every cost "can move" (on hand, or made
by an *active* building), and its pause rule toggles smelters, refineries and fabricators.
Trade completes at days ~1,000-1,100, so the bank engages for a short stretch only, and the
oracle's run-1 economy is thin (about 9 credits a day, ~14K a run): funding both at the cap
starves its buildings (dev reads with rates per resource: 1-2/9). Nationalist (no command
tool spending energy, He-3 or science) did best before the review fix. The driver is the
skilled player's run-1 economy, not project timing.
