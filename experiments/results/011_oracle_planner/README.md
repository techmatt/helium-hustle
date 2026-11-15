# 011 oracle planner

The `oracle` rung ([docs/agents.md](../../../docs/agents.md#the-oracle-oraclers)): the author
steered by rollouts on copies of the game, which see the future (Matt's ruling). Run
strategies (bundles of standing goals) over 2,000 ticks, tactics (target, research, project,
hoarding) over 600, scores in credits plus a profile's weights on milestones and what the
career keeps, retirement by the prestige rule. Rules revision 3, unchanged: no play change
to the game or to the other rungs (the author's fingerprints are the same).

- **Final read:** fresh seeds 3013-3015, careers of 20 retirements, every rung and five
  oracle profiles, 27 games, 75 minutes on 3 threads (the box was loaded: wall times are
  inflated). Tables: [final_read.md](final_read.md). Raw `games.jsonl` and `report.md`:
  `out/keep/011_oracle_final2/` (kept by conductor).
- **First read**, seeds 3010-3012, superseded: a probe after it found a bug in holding boredom
  that wrecked the quests profile, so the read was taken again ([first_read.md](first_read.md)).
- **How it got here** (dev seeds 1-3): [development.md](development.md).

## The ladder

| Player | Credits a run | Best 100-day cr/day (median, max) | Run 1 ticks | Milestones reached (any seed) | Never reached |
|---|---|---|---|---|---|
| random | 1,591 | 4.4, 5.0 | 1,639 | M1, M4 | the rest |
| greedy | 7,286 | 10.6, 11.0 | 1,611 | M1, M2, M4 | the rest |
| casual | 8,739 | 15.5, 16.3 | 1,560 | M1-M4 | M5-M14 |
| author | 14,144 | 25.4, 26.9 | 1,111 | M1-M5, M7, M8, M13 | M6, M9-M12, M14 |
| **oracle** | **28,093** | **67.8, 74.8** | 1,245 | M1-M5, M7, M8, M10, M12, M13 | M6, M9, M11, M14 |
| oracle, all profiles | 27,457-30,587 | 62-72, up to 75.6 | 1,245-2,253 | M1-M5, M7-M13 | M6, M14 |

The oracle earns twice the author's credits a run at 2.7 times its best rate, and reaches
three milestones the author never does (M10, M12, and M5 in run 1-2 against runs 18-19 on 2
seeds). Under `mixed` its run 1 is 19.0 real minutes and a run 24.6 (the author's 16.9 and
23.2): about 160 decisions a run against the author's 154 (it looks when the author would).

## Against the author, milestone by milestone (MESSAGE #399)

Career ticks (run) per seed 3013 / 3014 / 3015, the default (milestones) profile:

| | Author | Oracle | |
|---|---|---|---|
| M1 | 60 / 60 / 60 | 60 / 60 / 60 | tie |
| M2 | 420 / 420 / 420 | 420 / 420 / 420 | tie |
| M3 | 247 / 247 / 247 | 247 / 247 / 247 | tie |
| M4 | 1,640 / 1,111 / 1,038 | 1,340 / 1,054 / 1,245 | earlier on 2, later on 1 |
| M5 | 29,425 (r19) / 26,912 (r18) / - | 1,076 (r1) / 2,000 (r2) / 1,114 (r1) | beaten |
| M7 | 1,132 (r1) / 2,325 (r2) / 3,644 (r3) | 2,318 (r2) / 2,356 (r2) / 2,206 (r2) | later on 2 |
| M8 | 26,663 / - / 9,062 | 18,716 / 3,772 / 15,914 | later on 1 |
| M10, M12 | never | run 1 on every seed | beaten |
| M13 | 20,372 (r13) / - / - | 14,129 (r8) / 8,462 (r6) / 8,937 (r6) | beaten |

Not beaten everywhere: M1-M3 (ties), M4, M7, M8. Why, with the evidence:

- **M1-M3 tie.** The oracle's opening is the author's: its tactics tried every unlocked
  building as a target, every visible research and project, from tick 0, and none beat
  keeping the author's own plan by the 2% margin, while M2 and M3 met sooner score 100
  credits a tick. With looks at plan times the oracle's M2 and M3 came 31 and 18 ticks
  *later*; looking exactly when the author does put them on the same tick. A manual launch
  of a part-loaded first pad (a goal tried on dev seeds) moved M2 by 1 tick on 1 seed of 3:
  the first shipment waits on the pad's unlock (50 He-3 produced) and price, not on a full
  pad. M1 has a floor near day 51 (one processor: 210 credits of Cloud Compute and
  Buy Titanium for the second panel); the author takes 60. Beating these by more than a few
  ticks needs finer play than goals (hand-timed command queues), which the oracle doesn't
  search: they are ties by construction.
- **M4** (first retirement) can be had at any tick after the first shipment; the question is
  the price. The oracle retires when the run's average credits a day peaks; the author when
  it has bought nothing for 300 days, which comes early because its economy stalls. Pulling
  the oracle's first retirement earlier (`first_margin` 100 and 200 per mille, dev seeds)
  didn't put it reliably before the author's (1,350/1,455/2,070 and 1,037/1,355/2,070 against
  1,391/1,370/1,300) and 200 halved a run 1's credits (7.3k against 15.2k). The oracle's run 1
  earns 1.3-2.8 times the author's. A strong player retires later than a stalled one.
- **M7** (Dream used and Stress Tolerance bought in one run). Dream Protocols shows only past
  200 boredom and both cost 660 science; the oracle spends run 1's science on Trade Promotion
  first (600), which is where its run-1 M5, M10 and doubled credits come from. On dev seeds
  the earliness weight (100 a tick) got M7 into run 1 on 2 of 3; here it lands in run 2 on all
  three. Taking it in run 1 costs Trade Promotion's run: a "trade, Dream first" bundle
  offered at every strategy plan (dev seeds 1-3) was never chosen in run 1, its rollouts
  scoring below trade's.
- **M8** (three goods shipped in one run) on seed 3015: the oracle ships what pays (He-3 and
  circuits at the demand ceiling); the author's run 6 shipped a third good off a capped pile.
  The default profile weighs M8 like any milestone; it is a choice, not a wall.

Every other profile is on the same footing for M1-M3 and reaches M13 (ideology and quests:
by run 2-6 on most seeds), M9 and M11.

## Per profile

| Profile | Credits a run | Best rate | Retired by choice | Best ranks | Long-Term projects | Open Horizons sub-objectives | Never built / used |
|---|---|---|---|---|---|---|---|
| milestones (default) | 28,093 | 67.8 | 55% | Humanist 4, Nationalist 4, Rationalist 3 | - | - | Fuel Cell, Microwave Receiver; overclocks, Buy Power |
| credits (`profile=1`) | 28,864 | 62.3 | 62% | Humanist 2 | - | - | the same, plus Fund Nationalist and Rationalist |
| ideology (`profile=2`) | 29,350 | 71.7 | 53% | Humanist 3, Rationalist 3 | - | - | the same, plus Fund Nationalist |
| builder (`profile=3`) | 30,587 | 62.2 | 67% | Humanist 2, Rationalist 3 | - | - | the same as ideology |
| quests (`profile=4`) | 27,457 | 68.9 | 55% | Humanist 5 | AI Consciousness, Foundation Grant | 3 / 2 / 1 of 4 | the same as credits |

Milestones per profile, median tick (run) and how many seeds: [final_read.md](final_read.md).
In short: every profile reaches M1-M5 and M10 in run 1; M12 in run 1 for the default,
builder and quests profiles (1 seed of 3 for credits and ideology); builder M7 in run 1 and
M9 on 2 seeds; quests M9, M11 (2 seeds) and M13 by run 2; none M6 or M14. Retirements: 20 in every career
(the spec's end), 53-67% by choice; the quests profile's ten-year runs end by choice at day
3,654.

## Each milestone: reached, or what blocks it

| Milestone | Best reached (profile, seeds) | What blocks it otherwise |
|---|---|---|
| M1-M4 | every profile, run 1 | - |
| M5 25/day for 100 days | every profile, run 1-2 | - |
| **M6** 3 Data Centers and 100/day for 100 days | never; best 100-day rate 75.6 (quests, seed 3013) in the read's 15 oracle careers, 80.5 in the first read's | **Energy.** At 56/day (probe, dev seed 2, tick 1,500) every market sits at the 1.1 ceiling (Promote, Market Timer), so demand isn't the limit; energy is at 0, Solar Panel #10 needs 230 titanium against a 170 cap (1.5^n in credits and titanium), research labs are idled for circuits. The other sources are gated: Fuel Cell behind Chemical Energy (Open Horizons active, 1,000 propellant; built in one career of the first read), Microwave Receiver behind Nationalist rank 5's project (best Nationalist rank seen: 4). |
| M7 | default and quests run 1-2, builder run 1 | - |
| M8 | default 3/3 (run 3-11) | - |
| M9 20,000 credits on hand | builder 2/3 (run 5-6), credits, quests 1/3 | Not a wall: holding 20,000 means 300-600 days of buying nothing at 30-60/day; the default profile won't pay that. |
| M10, M12 | M10 every profile, run 1; M12 default, builder, quests, run 1 | - |
| M11 a Long-Term project | quests 2/3 (AI Consciousness run 3, Foundation Grant run 8) | The default profile funds them only as spare credits allow: 7,000 credits and science for Foundation Grant, or rank 5 first. |
| M13 rank 3 | default 3/3 by run 6-8; quests 3/3 by run 2-16 | - |
| **M14** Open Horizons | never; 3 of its 4 sub-objectives latched (quests, seed 3013: rank 5, AI Consciousness, ten years) | **100,000 credits in one run.** Best run: 90,306 credits in 3,654 days (quests, seed 3014), about 25 a day: a run held long by Dream spends processors and energy on Dream and Fund. The ten years themselves are reachable: Dream outpaces phase 5's 3 a day from two processors. |

## Findings for the balance work

1. **Energy is the late-game wall** (M6): every source but Solar Panel is gated behind
   Long-Term projects, and Solar Panel's titanium price passes the titanium cap by the
   tenth copy. 75/day is the ceiling seen.
2. **Boredom stops being a clock once Dream is researched.** Two processors on Dream hold a
   run indefinitely (runs of 3,654 days, retired by choice); forced retirement is avoidable.
3. **Demand stops being a constraint with Promote.** One processor promoting keeps He-3 and
   circuits at the 1.1 ceiling; the oracle researches Trade Promotion first in most runs.
4. **A retirement re-rolls the markets for free.** A player who could see the roll would
   retire at day 0 until the noise suits; the oracle did, burning runs, until barred before
   day `min_run`. A human can't see the roll, but can retire on a bad market start.
5. **The run windows are late for strong play:** M5 (target runs 2-4), M10 (6-10) and M12
   (8-15) fall in run 1; M13 (10-18) by run 2-8; M8 (5-8) by run 3.
6. **Never used by any oracle:** the overclock commands (Overclock Protocols shows only at
   5 Excavators and costs 600 science; never chosen), Buy Power (no Microwave Receiver).

## A no-peeking planner

The oracle peeks in two places: rollouts play the seeded future (demand noise, speculator
bursts, rival dumps), and the retirement rule plays the run out. The one exploit found is
the free market re-roll (finding 4). Its choices are mostly robust to the noise: the
strategies and targets turn on prices and production, which are known. A `planner` that
rolls out under sampled futures (the same code with a different seed per rollout) would
measure the gap; not built.

## Cost

About 30 ticks a second, 15-22 minutes a career of 20 retirements on one thread (rollouts a
few milliseconds each; `examples/probe.rs` prints what walls a run in at any tick). The
ladder now takes about 20 minutes on 6 threads; without the oracle 11 s.
