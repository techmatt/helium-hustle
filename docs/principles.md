# Principles: targets and measures

What the game should feel like, as numbers the simulator can check. The targets come from the
design doc (quoted below); the rest is this repo's: the milestones and human models are data in
`configs/sim/`, the players are code (`crates/hh-sim/src/agents/`) with their specs in
`experiments/*.json`, and the judge's cells are `experiments/balance.json`'s. The simulator that
reads them: [simulator.md](simulator.md); its players: [agents.md](agents.md).

## Targets (design doc)

> The first run should take about 30 minutes of real time at mixed speeds: about 800–1,100 ticks
> played well, about 1,500 played casually (1 tick = 1 in-game day; 1x speed = 1 tick a second).
>
> M5–M7 land together, M8–M10 overlap, M11/M12 converge, then M13, then M14. Each run ideally
> reaches one major milestone, with 2–3 real strategy branches open at any point.

Objective design rules (carried from the Godot optimizer): building existence for capability
milestones, events for pipeline outputs, cumulative counters for volume, production-rate
thresholds for scaling, stockpile thresholds only for uncapped resources, and never a stockpile
threshold for a capped, flowing resource. `Milestones::from_json` refuses a stockpile threshold
on a capped resource.

## Milestones

`configs/sim/milestones.json`. Run *n* is the run after *n − 1* retirements; a milestone is met
the first tick its condition holds (checked after every tick and every look), and reported as
its career tick, its run and the run's day (real minutes under a human model). A milestone first seen
in the step that ended a run (a retirement, or what the run's last tick did) counts in the run
it ended, on its last day. What a run's last tick reached and the new run reset (a pile of
credits, a building count) is not seen.

| Id | Name (design) | Measurable definition | Rule | Target |
|---|---|---|---|---|
| M1 | First Light: self-sustaining energy | 2 Solar Panels owned (the Boot Sequence quest) | capability | run 1, day 0–350 |
| M2 | First Shipment: full pipeline working | first shipment launched | event | run 1, day 250–700 |
| M3 | Program Awakening: first meaningful automation | 2 Data Centers owned: a second processor (the Automation quest) | capability | runs 1–2 |
| M4 | First Retirement | first retirement, forced or chosen | event | end of run 1 (its day: run 1's length bands, below) |
| M5 | Positive Credit Flow | credits earned in the run over the last 100 days ≥ 25 a day | rate | runs 2–4 |
| M6 | Speed Becomes Useful: automation + speed synergy | 3 Data Centers owned and credits earned ≥ 100 a day over 100 days | capability + rate | runs 3–5 |
| M7 | Boredom Management: Dream and research extend runs | Dream run once in the career and Stress Tolerance bought in the run | event + capability | runs 4–8 |
| M8 | Diversified Trade | 3 different goods launched in one run | event | runs 5–8 |
| M9 | Credit Surplus | 20,000 credits held | stockpile (credits are uncapped) | runs 5–10 |
| M10 | Market Manipulation: Promote + Disrupt active | a Promote and Disrupt Speculators each run once in the career | event | runs 6–10 |
| M11 | First Major Project (persistent) | a Long-Term project completed | event | runs 8–15 |
| M12 | Adversary Subverted: speculators managed | an Arbitrage Engine owned | capability | runs 8–15 |
| M13 | Ideology Influence: rank 3+ in an axis | the career's best rank on any axis ≥ 3 | counter | runs 10–18 |
| M14 | Timeline Unlocked: Arc 1 → Arc 2 | the Open Horizons quest completed (the End Signal quest opens) | event | runs 15–20 |

**Judgement calls** (candidates; the data may move them):

- **M1** reads "self-sustaining" as the Boot Sequence quest's two panels, not an energy-rate
  threshold: at two panels a run's energy runs positive with the opening buildings.
- **M3** reads "meaningful automation" as the second processor (Automation quest), not as a
  program of a given shape: a program is the agent's choice, a processor is the game's.
- **M5** and **M6**'s thresholds (25 and 100 credits a day over 100 days) are guesses at
  "positive" and "useful": one Cloud Compute processor earns 5 a day, so 25 needs shipping or
  several processors. Credits earned can't be split between shipments and commands without a
  state change, so the rate counts both.
- **M7** asks for Dream used and Stress Tolerance bought ("Dream and research"), not for a run
  that actually lasted longer.
- **M8**'s "diversified" is three goods of the four; a launch is seen as its good's market
  revenue rising over a tick, or a ready pad cooling down after it.
- **M9**'s 20,000 is about two and a half times what one idle Cloud Compute processor earns
  in a forced-length run (5 a day × ~1,600 days), so idling can't meet it; 8,000 could.
- **M10** counts use in the career, not both in the same run.
- **M12** reads "speculators managed" as owning the building that decays them; Speculator
  Dossier (the Strategic project) is the other candidate.

## Players and human models

**Sim players play in game time** (Matt's ruling): a player acts whenever a decision is useful
and skips stretches that need none ([agents.md](agents.md#when-it-looks)), so results are ticks
and **decisions** (looks that applied an action), never speed-dependent. The ladder's players:
`random`, `greedy`, `author` (playing well: about 140 decisions a run), `casual` (the author
looking at most every 120 ticks: about 12 a run). Above them, in its own spec
(`experiments/ceiling.json`), `oracle` (the ceiling: the author steered by rollouts that see
the future; not read by the judge).

**Human models** (`configs/sim/humans.json`) are a separate, occasional projection of a career
to real minutes: each decision takes `decide_s` seconds, and the gap to the next look plays its
first `burst_after` ticks at `play` speed and the rest at `burst`. Human play is decisions at
1-3x with brief bursts to 5-20x (200x comes later, once programs automate more): `careful` 1x
and 5x, `mixed` 2x (a mix of 1x and 3x) and 10x, `brisk` 3x and 20x. A model changes no game;
a human's play log has its own clock (`hh-sim playlog`, `vs-human`), against which a model can
be checked.

## The judge

The balance warnings (`python scripts/warning_checks.py`, `balance:` lines) read the `author`
and `casual` players ([agents.md](agents.md)), 2 seeds, careers of 20 retirements, real time
under the `mixed` model (`experiments/balance.json`, the judge's one home). A line is written
for:

- a milestone whose median run falls outside its window, or that half or fewer of the seeds
  reached, or that no seed reached in a career long enough to reach it;
- a run-1 milestone (M1, M2) whose median day falls outside its day window;
- run 1's median length outside its player's band (`author` 800–1,100 ticks, `casual`
  1,300–1,700);
- the author's run 1 outside 25–35 real minutes under `mixed`.

The run-1 bands are keyed by player: "played well" is the author, "played casually" the casual
author. The 30-minute band is read for the author only: the `mixed` model bursts through every
quiet stretch, so the casual player's 120-tick gaps take about 5 real minutes a run there, which
says how the model treats a player who leaves the game alone, not how long casual play takes.

## What limits income

Found by the oracle (rev 3, [011's findings](../experiments/results/011_oracle_planner/README.md#findings-for-the-balance-work));
a balance item starts here:

- **Energy is the late wall.** Every source but Solar Panel sits behind a Long-Term project
  or a rank-5 project, and Solar Panel's titanium price passes the titanium cap by about the
  tenth copy (#10 needs 230 against a 170 cap). With energy at 0, income topped out near 75
  credits a day (best 100-day rate 75.6, short of M6's 100). Programs, Dream and Promote all
  draw on it.
- **Demand stops binding under Promote.** One processor promoting what ships holds He-3 and
  circuits at the 1.1 demand ceiling; the oracle researches Trade Promotion first in most
  runs.
- **Boredom stops being a clock once Dream is researched:** two processors on Dream hold a run
  indefinitely, so a strong run ends by choice (the prestige rule), not by force.

## Measures

Every game reports (`hh-sim ladder`, [simulator.md](simulator.md#measures)): time to each
milestone in ticks and runs (in real minutes under a human model); run length, decisions and
boredom's share; the share of ticks with nothing to buy; resources sitting at cap and the
production lost to caps; buildings never bought and commands never used; and simulation speed,
native and wasm.
