# 060 Loadout exploration

Five loadouts under RULES_REVISION 16 (13 readings, `above`/`below` a number, one `and`/`or`,
no `wait until`), measured in the sim against the author and hands-off players on the same seeds.
No rules, content or sim-player changes.

**Headline.** A two-tab loadout of 28 lines (L4) earns **17.7k credits a run, 2.5x the author**
(7.0k) and 27x today's hands-off player (0.65k), with no hand action in 47 of 48 runs (one
ended by the 300-day hand retirement, below). Adding a
3-line Dream tab (L5) earns 24.2k by holding boredom off for longer runs. The short ones show
the floor: one Cloud Compute line earns 1.5k a run; a 16-line chain 3.6k.

## How it was measured

`harness/` is a scratch sim player built on `hh-sim`'s library (its own workspace, not part of
the repo's build). Per career: runs 1 and 2 are the author's, exactly as `handsoff` plays them
(`if` opens in run 2 on every seed here: the Conditional Logic project), so every player below
shares those two runs. From run 3, at each reset it **writes the loadout's programs** (all
processors start on Program 1 at a reset) and makes **one other hand move: it retires a run
that has bought nothing (building, research or land) for 300 days**, as `handsoff` does. `agent=author` and
`agent=handsoff` run the stock players through the same loop (fingerprints equal to `hh-sim
ladder`'s).

Final read: fresh seeds **3119-3124** (6 careers each, 10 retirements, so 48 runs from run 3).
Development used dev seeds 1-3 and 3113-3118 (a first read of an earlier L4/L5, superseded).
Each run is measured from run 3 on; `Forced` is the share of runs that boredom ended (the
loadouts' and hands-off's other runs ended by the 300-day hand retirement; the author's, by its
own retirements).

**Phase.** Every loadout uses `if`, so all suit a reset after Conditional Logic (offered after
beat 1.6; run 2 on these seeds), run 3 on. None is for run 1 before that: a loaded `if` reads
false there, which leaves L1 as plain Cloud Compute and the others mostly idle.

| Player | Lines | Credits a run (SE over seeds) | Credits / 1000 days | Ships a run | Run days | Forced | Buildings a run |
|---|---|---|---|---|---|---|---|
| author | - | 6,999 (352) | 4,491 | 5.1 | 1,559 | 75% | 31 |
| handsoff | - | 654 (62) | 683 | 0.0 | 957 | 0% | 13 |
| L1 compute floor | 3 | 1,550 (0) | 4,860 | 0.0 | 319 | 0% | 0 |
| L2 short chain | 16 | 3,593 (21) | 4,734 | 4.0 | 759 | 0% | 13 |
| L3 one tab | 31 | 6,958 (470) | 4,413 | 2.3 | 1,577 | 100% | 25 |
| **L4 two tabs** | **28** | **17,700 (389)** | **12,388** | **10.4** | 1,429 | 98% | 24 |
| L5 L4 + Dream tab | 34 | 24,202 (550) | 11,484 | 13.2 | 2,107 | 2% | 29 |

Days into the run (median over the runs that got there; all 48 unless noted):

| Player | 1st pad | 1st ship | 2nd DC | 3rd DC | 5th ship | 1st research |
|---|---|---|---|---|---|---|
| author | d175 | d428 | d302 | d871 (13/48) | d906 (22/48) | d647 (11/48) |
| handsoff | - | - | - | - | - | - |
| L2 | d196 | d408 | - | - | - | - |
| L3 | d314 | d1334 | d970 | d1351 | - | - |
| L4 | d228 | d482 | d420 | d528 | d802 | - |
| L5 | d271 | d594 | d531 | d642 | d926 | d863 |

No player reached a milestone past M5 and M11 (Conditional Logic) in 10 retirements; no loadout
completes a global project. Per-seed, per-run credits and ships: [per-seed.txt](per-seed.txt).

## The loadouts

The blocks are line for line what the program editor shows (display names; indented lines sit
under the `if` above). **The editor has no text paste**, and it offers only what the run shows
now (Idle pads once a pad stands; a building or research once visible), so a fresh reset can't
take most of these lines. Enter them late in a run where all of it shows, save each tab to the
library, and load it at each reset (a load skips the unlock checks). `loadouts/*.txt` are the same
programs in the harness's id form; `harness ... --print` turns one into `set_program` JSON.
`render.py` prints these blocks from the files.

### L1 Compute floor (3 lines): the baseline (works as plain compute before `if` too)

```text
If Energy below 50% and Credits above 20
    Build Solar Panel
Sell Cloud Compute (x10)
```

The Solar Panel line never fires from a reset (no titanium), so this is one working line: Cloud
Compute on the starting Solar Panel's energy, 5 credits a step. It needs a hand retirement (it
buys nothing), and its short runs make its credits a day match the author's. It is the floor a
real loadout must beat, and shows how much the author's play is worth.

### L2 Short chain (16 lines): the first days after a reset, one processor

```text
If Energy below 60% and Credits above 20
    Build Solar Panel
Sell Cloud Compute (x10)
If Titanium above 99% or Regolith above 99%
    Build Storage Depot
If Credits above 150 and Energy above 70%
    Build Regolith Excavator up to 2
    Build Smelter up to 1
    Build Refinery up to 1
    Build Launch Pad up to 1
    Set pads Helium-3 up to 1
    Build Refinery up to 2
Load Launch Pads (x5)
Launch pads all
If Propellant below 67% and Credits above 300
    Buy Propellant (x5)
```

The mining chain to one He-3 pad: first ship by day ~400, as soon as the author's. With no Data
Center it stays on one processor, builds out by day ~450 and then buys nothing, so it needs the
hand retirement (or the player takes over). Good as an opener a player replaces.

### L3 One tab (31 lines): a whole run on Program 1

```text
If Free processors above 0
    Assign processors Program 1 all
If Energy below 60% and Credits above 20
    Build Solar Panel
If Energy above 20%
    Sell Cloud Compute (x10)
If Energy below 21%
    Idle (x5)
If Titanium above 99% or Regolith above 99%
    Build Storage Depot
If Free land below 5 and Credits above 200
    Buy Land
If Circuit Boards above 79% and Credits above 150
    Build Data Center up to 6
If Ice below 30% and Credits above 200
    Buy Ice (x10)
If Credits above 150 and Energy above 70%
    Build Regolith Excavator up to 2
    Build Smelter up to 1
    Build Refinery up to 1
    Build Launch Pad up to 1
    Build Fabricator up to 1
    Build Regolith Excavator up to 4
    Build Smelter up to 2
    Build Refinery up to 2
If Idle pads above 0
    Set pads Helium-3 up to 1
Load pads Helium-3
Launch pads all
If Propellant below 67% and Credits above 300
    Buy Propellant (x5)
```

Matches the author (6.96k against 7.0k) with no hands, every run ended by boredom. It ships
little (2.3 a run): one `Load pads` verb a loop moves 5 He-3 while the loop is ~20 steps long.

### L4 Two tabs (28 lines): from a reset after `if` opens; the pick

Program 1:

```text
If Free processors above 0
    Assign processors Program 2 1
    Assign processors Program 1 all
If Energy below 60% and Credits above 20
    Build Solar Panel
If Energy above 20%
    Sell Cloud Compute (x10)
If Energy below 21%
    Idle (x5)
If Titanium above 99% or Regolith above 99%
    Build Storage Depot
If Free land below 5 and Credits above 200
    Buy Land
If Circuit Boards above 79% and Credits above 150
    Build Data Center up to 6
If Ice below 30% and Credits above 200
    Buy Ice (x10)
If Credits above 150 and Energy above 70%
    Build Regolith Excavator up to 2
    Build Smelter up to 1
    Build Refinery up to 1
    Build Launch Pad up to 1
    Build Fabricator up to 1
```

Program 2:

```text
If Helium-3 above 10%
    Load Launch Pads (x5)
Launch pads all
If Propellant below 67% and Energy above 30%
    Buy Propellant (x5)
```

Program 1 builds and earns on the one processor a reset gives; its first block hands the second
Data Center's processor to Program 2, the 5-line ship cycle, and every later one to itself.
Manual: load both tabs at each reset (two loads); 98% of runs end by boredom, the rest by the
hand retirement. Program 2 alone is
the mid-run add-on for a player who builds by hand: give it one processor once a pad stands.
Against L3 it changes two things at once (the ship tab, and Program 1 dropping its own ship
lines and the second round of excavators, smelters and refineries). On dev seeds 1-3 each
step helped: L3 7.0k, L3 plus the ship tab 12.0k, then the trims to 14.4k and more.

### L5 Dream tab (L4 + 6 lines): once runs end by boredom

L4's two programs, with `Build Research Lab up to 2` under the Data Center line and
`Assign processors Program 3 1` after Program 2's seat line, plus:

Program 3:

```text
If Science above 300
    Research Daydreaming
If Boredom above 50%
    Dream (x5)
```

Daydreaming is Dream Protocols' research (300 science, ~day 860 from two labs). Dreaming holds
boredom, so runs last ~2,100 days and end by the hand retirement (98%), not boredom. Most
credits a run; slightly fewer a day than L4. A few runs stall near 8k (1 of 48 here, 3 of 72
in development): Dream burns
0.5 He-3 a time, starving the pad, while fuel keeps buying. Guarding Dream with `and Helium-3
above 20%` cured the stall but cost 40% (runs fell back to boredom endings), so it stays off.

## What the 13 readings forced

- **Percent of a growing cap.** Capped resources read only as a fill percent, and Storage
  Depots grow the caps, so a fixed amount drifts: `Propellant below 67%` is the 20 units a
  launch needs at the base cap of 30, but 100 units later (L5's stalled runs held 150).
  `Helium-3 above 25%` stopped loading at all once the cap grew, with 46 He-3 in store; 10%
  fixed it for now. **An amount reading for capped resources** would make both exact.
- **No energy rate.** Energy fill lags what new buildings draw; the trap is a brownout with
  credits spent below a Solar Panel's price, where Cloud Compute fails and nothing recovers.
  Two blocks exist only for it (`Idle` below 21% energy, energy guards on building and on
  fuel), and an early two-tab version deadlocked on 2 of 9 seeds until the fuel line read
  energy instead of credits. **An energy per day reading** (or production less demand of any
  resource) would replace 4 lines and the guesswork.
- **No building counts.** `Buy Ice (x10)` can't stop once the Fabricator stands (it buys
  again whenever Ice dips); a chain can't wait for its prerequisite, so its `up to N` lines run
  as done steps each loop; a pad's good can only be set via `Idle pads`. **Owned <building>**
  would turn L4's build block into exact guards and drop the re-buys.
- **Two clauses at most.** Fuel wants three facts (fuel short, a pad loaded, money spare);
  each version drops one, and which one decides whether a seed deadlocks.
- **`Idle pads` reads 0 with no pad**, so `Idle pads below 1` ("a pad holds cargo") is
  also true before any pad exists: it bought fuel with nothing to launch.
- **No "days since a purchase".** A program can't tell a run has stalled, so L1, L2 and L5
  need the hand retirement. A `Retire` line guarded by it would make them fully hands-off.
- **Worked well:** `Free processors` (a 3-line seat block) and `if` being free (a 13-block
  program costs only its true lines).

## Other findings

- **One processor at a reset decides the shape.** Until the second Data Center (day ~420)
  only Program 1 runs, so a split loadout whose builder is on Program 2 earns nothing (dev: 0
  credits). Every line costs a step, so short loops earn more: on dev seeds, dropping L4's
  three second-round build lines added 2.1k a run, and its three ship lines in Program 1 1.3k.
- **The hands-off player is broken under the 13 readings:** from run 3 it never builds a pad
  or a second Data Center on any seed (0 ships, 654 credits a run, 9% of the author from run
  3). The ladder's own share (33% on these seeds) counts run 2, which the author plays.
- **Credits a day vs a run.** L1 and L2 match the author's credits a day only because their
  runs are short; each retirement costs a restart, so credits a run is the fairer measure.

## Files

- `loadouts/` the five programs (harness form: `seat:` the reset's processors, `[n]` Program n).
- `harness/` the player; build with `CARGO_TARGET_DIR=<abs>/out/... cargo build --release
  --manifest-path experiments/results/060_loadout_exploration/harness/Cargo.toml`, run from
  the repo root: `loadout-harness loadouts/L4_two_tabs.txt --seeds 6 --first-seed 3119
  --retirements 10 --out out/x.jsonl` (or `agent=author`). A loadout of Matt's in the same
  form plays the same way.
- `stats.py` (the first table but its Lines column), `milestones.py` (the second),
  `perseed.py`, `render.py`. The final read's games.jsonl are kept outside the repo (conductor
  `kept/helium-hustle/060_loadout_exploration/060_reads`).
