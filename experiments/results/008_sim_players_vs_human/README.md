# 008 sim players in game time; compare against Matt's play

- [ladder_rev3.md](ladder_rev3.md): the baseline, `hh-sim ladder --seeds 5 --first-seed 3005`
  (random, greedy, author, casual; 5 seeds a player, careers of 20 retirements, 20 games, about
  35 s on 6 threads), with real time under the three human models. Raw games and summaries:
  `out/keep/008_ladder_rev3/` (`games.jsonl`, `results.json`), kept by conductor.

Seeds: 3005-3009 taken from the new ledger (docs/simulator.md, *Seeds*) and read once, on the
final players. 3000-3004 were read once before the review fixes changed the author (below) and
are spent. Development and tuning used dev seeds 1-3.

## What changed in the simulator

- **Players play in game time.** `Agent::next_look` replaces 005's attention × speed grid: a
  player looks whenever a decision is useful (its target or next research comes within reach,
  boredom nears `dream_at`, a pad cools down, a stale run would retire; at most every
  `max_gap` ticks) and skips the rest. Results are ticks and decisions (looks that applied an
  action). No rules changed: `hh-core` is untouched, revision 3, the golden game unchanged.
- **Real time is a projection** (`configs/sim/humans.json`): `careful` (1x, bursts to 5x),
  `mixed` (2x, bursts to 10x), `brisk` (3x, bursts to 20x), each with seconds a decision. The
  ladder prints it as its own table; `--humans` picks the models.
- **`hh-sim vs-human LOG`**: the sim players on the log's seed, config and scenario, beside the
  human, and where the human first parted from the best player in what was built and when.
  Tried on the kept 006 sample log (seed 0, a 3-minute test session): the human bought a
  second Solar Panel where the author bought an Excavator (run 1, purchase 2), and took 130
  ticks longer over the first panel.
- **Tuning the author's `max_gap`** (dev seeds 1-3, 3 seeds each): 5, 10, 20, 30 and 60 tried.
  5 and 10 earned about the same (12,000-14,000 credits a run), 10 with half the decisions
  (about 140 a run vs 280); 20 and longer earned less (7,700-9,700) and longer gaps ship less
  (60: 3.6 shipments a run). The casual author is `min_gap` 120 (60 tried: run 1
  1,200 ticks, below the casual band).
- **Plan rewrites.** A rewrite of the main program counted as a decision almost every look (the
  filler's count followed the gap). Now the filler is a constant, a plan with nothing due isn't
  written over one already in its filler with enough left to reach the next look, and the due
  lines re-run when the pointer has passed them: about 140 decisions a run instead of 340.
- **Review fixes** (an independent review, 10 confirmed findings, all fixed): the projection's
  gaps bend at the burst (a milestone inside a gap read too early); a run's minutes are cut
  before the decisions at its boundary ticks; a look that retired counts in the new run, and a
  building bought before a Retire at the same tick in the old one; a human's time at the log's
  last tick counts; `vs-human` refuses a player named `human` or two of one name; the author's
  keep-the-plan rule is limited to hand plans with enough filler left; `ticks_to` rounds up.

## Baseline (rules revision 3, content `06d9cf540641e1af`)

Medians over 5 seeds for run 1, means over finished runs. `t`/`r`: career tick, retirements
before it.

| Player | Run 1 ticks | Run ticks | Run 1 dec | Dec/run | Run credits | Peak cr/day | Ships/run | Run 1 min (mixed) | M2 | M3 | M7 | M5 | M13 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| author | 1,343 | 1,492 | 136 | 146 | 12,945 | 24.7 | 12.4 | 20.3 | t420 | t247 | t2277 r1 | 2/5, r14 | 3/5, r6 |
| casual | 1,560 | 1,590 | 11 | 12 | 8,997 | 16.2 | 17.3 | 5.1 | t682 | t1200 | - | - | - |
| greedy | 1,609 | 1,636 | 124 | 128 | 7,072 | 10.9 | 17.2 | 21.7 | t515 | - | - | - | - |
| random | 1,646 | 1,664 | 330 | 333 | 1,584 | 3.9 | 0 | 35.7 | - | - | - | - | - |

M6, M9-M12 and M14 were reached by no player in 20 runs; M8 by the author on 2 of 5 seeds (run
7). Against 005's rev-2 baseline the author earns more a run (12,900 vs 9,900) and its runs are
longer, but the rules (boredom cap 500 → 1,000) and the players both changed between the two
reads, so neither difference belongs to either change alone.

## Main misses

1. **The author's run 1 is 1,343 ticks, not 800-1,100** (seeds range 1,067-1,704): with the cap
   at 1,000 a run with no command boredom lasts to day 1,670, and the author retires a run that
   has bought nothing for 300 days. Its later runs reach 2,100 with Dream.
2. **Progress doesn't compound.** The author's credits a run are about flat from the first three
   runs to the last three (12,800 → 13,100 on average over seeds); M5 is met by 2 of 5 seeds, in
   run 15; M6, M9-M12 and M14 never.
3. **Income plateaus near 25 credits a day** (peak 24.7; M5 asks 25, M6 100): regolith and energy
   sit at cap 28-45% of ticks, and over half the propellant made is lost to its cap.
4. **M7 lands in run 2, not runs 4-8:** Dream and Stress Tolerance are both within reach early
   at a 1,000 cap; M13 (ideology rank 3) comes early too, run 7 on 3 of 5 seeds (target 10-18).
5. **Few decisions earn most of the value.** The casual author makes 12 decisions a run (one per
   ~130 ticks) and earns 70% of the author's credits with 146; its run 1 is in its band (1,560,
   target 1,300-1,700), but it never meets M5 or M7.
6. **Real time:** the author's run 1 is 20 minutes under `mixed` (target 25-35), 34 under
   `careful`, 14 under `brisk`. The model's seconds a decision are guesses until a long human
   log calibrates them (`vs-human` prints the human's minutes a decision).
7. **Never used by the author:** Battery, Arbitrage Engine, Fuel Cell, Microwave Receiver;
   Overclock, Promote, Buy Power and two Fund commands.
8. **The Godot run-1 objectives:** the author's opening is late (first Excavator day 71 vs 5-20,
   first shipment 420 vs 260-380) but its second Data Center early (248 vs 380-530).

## Milestone and target definitions

No milestone's definition changed. Two run-1 targets were re-keyed, since attention profiles
no longer exist: the run-1 length bands are keyed by player (`author` 800-1,100, `casual`
1,300-1,700), and the 25-35 minute band is read for the author only, under `mixed`: that model
bursts through the casual player's long gaps (5 minutes a run), which measures the model, not
casual play. M6 ("Speed Becomes Useful") keeps its definition (3 Data Centers and 100 credits a
day): it reads automation, which still holds without a speed in play.
