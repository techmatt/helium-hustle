# Simulator

`hh-sim`, the native command line. Run from the repo root (paths are relative to it).

```sh
S="cargo run -q --release -p hh-sim --"
$S run --seed 1 --ticks 1700 --policy script            # the scripted opening, to a retirement
$S run --scenario rich --ticks 100 --policy greedy      # from a named starting state
$S run --scenario-file my.json --ticks 50               # from any scenario file
$S run --seed 5 --ticks 1000 --json                     # machine-readable, with the full state
$S run --seed 5 --ticks 1000 --save out/r.json          # write the game's record
$S replay out/r.json                                    # rebuild and check records (--summary: the end state)
$S objectives reference/scenarios/run1_fresh.json      # a Godot optimizer scenario, scored
$S show                                                 # resolved config, hashes, revision
$S scenarios                                            # the embedded scenarios
$S golden                                               # the golden game's fingerprint
```

## Measuring play

The simulator measures balance and pacing: sim players ([agents.md](agents.md)) play whole
careers in game time, against the targets in [principles.md](principles.md).

```sh
$S ladder                                     # the ladder: experiments/ladder.json (~11 s)
$S ladder --agents author --humans mixed --seeds 8 --retirements 5   # any slice of it
$S experiment experiments/ceiling.json        # the oracle beside the author (~20 min on 6 threads)
$S experiment experiments/balance.json --out out/b   # any spec
$S summarize out/experiments/ladder/games.jsonl      # each game's runs, milestones, career (mid-read too)
$S compare --config-b configs/other.json      # two configs, paired on the same seeds
$S compare --set boredom.base_cap=500         # a param (or module.variant=name) against the base
$S balance                                    # the judge's missed targets, as balance: lines (~5 s)
$S trace --agents author --retirements 2      # one career: actions, the agent's notes (#), the career log
$S trace --lines                              # ... and each day's program lines run, in queue order
$S trace --agents oracle --first-seed 2 --at 1500   # the state at a tick: what walls a run in
$S agents                                     # the ladder's rungs
$S speed                                      # native engine speed
node scripts/wasm-speed.mjs                   # the same in wasm (on web/src/wasm)
$S params --write                             # docs/rules param tables from REGISTRY (a test fails when stale)
$S playlog C:/Code/gdrive-sync/helium-hustle/playlogs/<career>.jsonl   # a human's career (--json: one line)
$S vs-human C:/Code/gdrive-sync/helium-hustle/playlogs/<career>.jsonl  # the sim players beside it
$S accelerate --seed 3                        # debug: the autopilot
```

**A career.** At each look the player reads the view and acts, then says how many ticks to
skip to its next look (`Agent::next_look`, [agents.md](agents.md#when-it-looks)); a forced
retirement wakes it early. The career ends at the spec's `retirements` (or `max_ticks`). Results are ticks and **decisions** (looks that applied an
action). Real time is not part of play: a **human model** (`configs/sim/humans.json`,
[principles.md](principles.md#players-and-human-models)) maps a career's looks to real minutes
afterwards, so the same game reads under any model.

**`playlog`** replays a web play log ([web/README.md](../web/README.md), *Play logs*) and
checks the fingerprint logged at its last tick (exit 1 on a mismatch). It measures the
career with the same `Meter` (`career.rs`) as a sim player's (a decision: a tick where the
human applied actions), so its tables are the ladder's for one human row; its real time is
the log's wall clock (each tick reached at its share of the time between logged points;
time paused before a tick counts toward it; time with the page closed doesn't count). Then
where the time went (sessions, hidden and closed, pauses, each speed, actions, panels). `--json` prints the
career's `GameResult` (the `games.jsonl` line a sim career writes) with the time use.

### Against a human

**`vs-human LOG`** replays the log, plays each sim player (the ladder's less any oracle, or `--agents`) on the
log's config, seed and scenario for as many ticks as the human played (`--ticks N` for more),
and prints them beside the human: ticks, retirements, each run's length, credits and
decisions (`+` marks the run under way), decisions and ticks a decision, buildings and
commands used; the milestones in career ticks and retirements; real time (the human's from
the log's clock, each player's under the ladder's human models or `--humans`), with minutes
a decision. Then the human against the **best player** (most milestones, then most credits
earned): buildings and commands only one side used, and where the human first parted from it,
matching the two build orders purchase by purchase, run by run (a purchase in a run's last
tick counts in the next run): first in **when** (the same building more than 50 ticks or a
quarter of the player's tick apart: `vs_human` in `configs/sim/milestones.json`) and first in **what** (a different building, or one side
bought nothing), with each side's next five purchases. `--json`: one line, every row's
`GameResult` and the divergence. Exit 1 if the log doesn't replay to its fingerprint.

### Accelerators

Debug only. The debug menu's cheats add 2k and 10k speeds and the **autopilot**
(`autopilot.rs`, with its default boost, [rules](rules/README.md#cheats)): a sim player
plays on honestly toward the **next milestone**, the **next main-quest step** (an `all_of`
sub-objective counts) or **everything**. Steps: `configs/sim/milestones.json` and the
`story` events not `never`. Its drivers (`DEFAULT_AGENT`) switch after a long stall. A boost or autopilot marks the career
`accelerated`, a debug speed its log: `playlog` says so, `vs-human` refuses it
(`--include-accelerated`), the autopilot's days count as jumped. `accelerate` plays a
fresh career so natively. The draft later acts: the author's `agents/arc.rs`
([042](../experiments/results/042_draft_whole_arc/README.md)).

### Grid reads

The grid options, specs, output, the career-log format, every measure, what a read costs and
recipes for reads against another revision: [experiments.md](experiments.md).

### Seeds

The **seed ledger**: a read that must not be tuned on (a baseline, a confirming read) takes
fresh seeds: `python scripts/seeds.py next N` prints the next N; `--take` also moves *Next
free* past them (every seed below it counts as spent). The item records its range here and in
its write-up. Before freezing a long read, every player and knob changed since its last full
dev career plays one. **Dev seeds** 1-3 are for development, on/off checks and the standing specs
(`ladder.json`, `ceiling.json`, `balance.json` start at 1).

Spent (superseded reads marked *sup.*): 1-3 and 100-103 (005, development), 1000-1004 (005,
rev-2 baseline), 2000-2003 (005, a compare), 3000-3004 (008, sup.), 3005-3009 (008, rev-3
baseline), 3010-3012 (011, sup.), 3013-3015 (011, the oracle's final read), 3016-3023 (035,
loadouts), 4-8 (036, development), 3024-3039 (036, sup.), 3040-3047 (036, hands-off against
main), 3048-3055 (037, sup.), 3056-3063 (037, verbs: hands-off share), 3064-3071 (038,
sup.), 3072-3079 (038, done steps), 3080-3087 (039, hands-off retire and fund lines),
0 (041, tuning), 3088-3093 (041, autopilot; 042, development), 3094-3096 (042), 3097-3104 (044), 3105-3112 (046, paired), 3113-3118 (060, sup.), 3119-3124 (060, loadouts).

Next free: 3125

## `run` and `replay`

Options and defaults: the usage (`hh-sim` with no command). `--save F` writes the game's
record (`hh-record/1`); `--json` prints one line: hashes, fingerprint, `replay_ok`, the state.

`run` prints the tick, run and day, each resource (amount, cap, next day's change), the
buildings owned, the career log (quests, achievements, projects, research, retirements),
the fingerprint and the replay check (the record replayed must equal the final state). Exit
codes: 0 ok, 1 a failed replay or record check, 2 bad arguments or a bad config.

`replay` prints each record's final tick and fingerprint, warns on another revision or a
differing fingerprint, and exits 1 on any warning or failure. `--summary` adds each end
state as `run` prints it (resources, buildings, the career log): the way to inspect a saved
mid-career state without a throwaway test.

## Policies

Policies live in `hh-core` (`driver.rs`) because the golden game and the wasm parity check
use one; the agents that measure play live here ([agents.md](agents.md)). Both read only the
view and act only through actions.

- **`greedy`** buys every affordable building in content order (from a new game, nothing:
  no program runs, so nothing is earned).
- **`script`** is a fixed Arc 1 opening, one decision a day (build order, Program 1's one
  command and when it switches: `driver.rs`); it never researches, funds projects or retires
  voluntarily.

## `objectives`

Plays one of the Godot optimizer's scenario files (`reference/scenarios/*.json`) with the
script, limited to the scenario's available buildings and commands (Godot ids mapped to
the port's), from a new game until the first run's forced retirement. It prints each
objective's tick against its target window (`in`, `early`, `late`, `not hit`), when the
scenario's end condition was met, and the career log. Objective types: `building` (with
`count`), `resource` (stockpile `threshold`), `event` `shipment_complete` and
`boredom_N`. About 0.2 s.
