# Experiments

Grid reads with `hh-sim` (`ladder`, `experiment`, `compare`, `balance`, `trace`): the spec,
the output, what each measure means, what a read costs, and recipes for reads against another
revision. The commands themselves and the seed ledger: [simulator.md](simulator.md); the
players: [agents.md](agents.md); the targets: [principles.md](principles.md).

**Kept outputs.** An item's `out/keep/<name>/` leaves the worktree when it ends: conductor
moves it to `C:\Code\helium-hustle-control\.conductor\kept\helium-hustle\<name>\`, and no
worktree or the main checkout has it afterwards. A write-up citing `out/keep/<name>/` means
that folder; a script a later item must rerun belongs under `experiments/results/`, committed.

## Grids and specs

Grid options (`ladder`, `experiment`, `compare`, `balance`, `trace`) override the spec:
`--agents a,b` (players; knobs stay with their agent: `author:a=1,b=2,greedy` is two;
`casual=author:min_gap=120` names one), `--humans a,b` (human models for the real-time
table), `--seeds N`, `--first-seed S`, `--retirements N`, `--max-ticks N`, `--config F`,
`--set module.key=value` (repeatable; added to every variant), `--threads T` (default 6),
`--human-models F`, `--milestones F`; `--out DIR` for `ladder`, `experiment` (default
`out/experiments/<name>/`) and `compare`. `compare`, `balance` and `trace` default to
`experiments/balance.json`'s grid; `compare` drops the spec's variants (side A is the plain
config, side B `--config-b` and the `--set`s). `trace` plays the first cell's first seed.

**A spec** (`experiments/*.json`): `name`, `note`, `configs`, `variants` (optional, `[{"name",
"set": ["module.key=value", "module.variant=name"]}]`), `agents` (players, `[name=]agent[:knobs]`,
names unique), `humans` (optional, human models), `seeds`, `first_seed`, `retirements`,
`max_ticks`, `scenario` (optional, an embedded scenario laid over each new game), `objectives`
(optional, a Godot optimizer scenario file such as `reference/scenarios/run1_fresh.json`: its
objectives scored on run 1, as `objectives` scores them, in a *Run-1 objectives* table). The
grid is configs × variants × players, each cell `seeds` games; results are in game order
whatever the thread count.

**Paths** (a spec, `configs/`, `configs/sim/`, `reference/scenarios/`) resolve from the
working directory, not the binary: run any `hh-sim`, a copied one included, from the root of
the tree whose files it should read.

**Output.** The tables on stdout and in `report.md`; `results.json` (`hh-experiment/1`: the
spec, revision, content hash and each cell's summary); `games.jsonl` (a line a game: every
measure, each run's ticks, decisions and projects completed (`runs[].projects`, every scope),
every building bought, the career's best ranks, the career's `timeline` and `forever` projects
(`projects`; Strategic ones are only in `runs[].projects`), quests,
sub-objectives and achievements at the end, its cell, the fingerprint and `replay_ok`),
appended as each game finishes and rewritten in game order at the end: `summarize` reads it,
a read still running included. The milestones
(`--milestones`) are the oracle's too. Every command exits 1 if a game's record doesn't
replay to its state.
`compare` prints the paired table (B minus A on the same seeds, mean difference and its
standard error, per player; run 1's minutes under the spec's first human model) and both
sides' tables; `--out` writes `compare.md`.

**`trace`** prints the agent's actions and notes (`#`), then the **career log**, one line an
entry: `<tick>  run <n> day <d>  <kind> <id>`, `kind` one of `quest`, `achievement`,
`project`, `research`, `unlock`, `retired`, `forced_retirement` (`state::LogEntry`). `--at T`
prints the state at tick T instead: resources, buildings with their next price, each
non-empty program (`program <n> procs <p>:`, then its lines one to a row, numbered as the
editor shows them, `>` marking a body line), markets, the agent's last note and the career's
goals.

## Measures

| Column | Meaning |
|---|---|
| Ret | Retirements reached (mean). |
| Run 1 ticks, Run ticks | Run 1's length (median), every finished run's (mean). |
| Run 1 dec, Dec/run, Ticks/dec | Decisions (looks that acted) in run 1 (median) and a finished run (mean); career ticks a decision. |
| Forced | Share of runs ended by boredom (the rest retired by choice). |
| Cmd boredom | Boredom's share from commands net of Dream, of all boredom gained (the rest is daily growth). |
| Idle | Share of ticks with nothing to buy: no building and no research affordable. |
| Piles at cap | Resources at their cap, as a share of ticks (top three, 5% or more). |
| Overflow lost | Production lost to caps, as a share of production (top three, 5% or more). |
| Ticks/s | Simulated ticks per wall second, agent and measurement included. |
| Milestones | Per milestone, medians: career ticks (`t`), retirements before it (`r`; it fell in run r+1) and that run's day (`d`); `k/n` when only k of n seeds reached it. |
| Run credits, Peak cr/day, Ships/run | Credits earned and shipments a finished run (means); each career's best credit rate over the milestones' longest `credit_rate` window, 100 days (median over seeds). |
| Real time | With `humans`: per player and model, run 1's real minutes (median), a finished run's (mean), minutes a decision, and each milestone's minutes from the career's start (median). |
| Hands-off | With a `handsoff` player and an `author` one: how far the best known loadout (`configs/sim/handsoff_loadout.txt`, loaded at each reset once `if` opens) carries a run. Credits a finished run from run 2 on (mean) and milestones first met from run 2 on (summed over seeds), each against the author's on the same seeds and as a share of it (runs until `if` opens, and that one, are the author's in both: run 2 on most seeds, so "from run 2" can hold author runs). Its runs end by the loadout's own `retire` (`days since a purchase`, researched each run, 062), its backstop hand retirement (`stale_days` with nothing bought) or the boredom cap. |
| Run-1 objectives | With a spec's `objectives`: each objective's median run-1 day against its Godot window (`in`, `early`, `late`). |
| Never built, never used | Buildings never owned and commands never run in any seed's career. |

After `ladder`'s and `experiment`'s tables, the engine's speed natively and in wasm.

## Costs

This machine, idle: the ladder (4 players × 3 seeds, careers of 20 retirements)
about 20 s on 6 threads; `ceiling` about 20 minutes, set by the oracle's careers
([oracle.md](oracle.md)); `balance`
(4 careers) about 6.5 s (092, on mimalloc: half the Windows allocator's time); `vs-human` on a 1,600-tick log a second. Other careers play
4,000-20,000 ticks a second, slower the more often their
player looks (random and the author every 5-10 ticks, the casual author every ~130): a view
(~130-270 µs) costs ten state copies, so a rollout that plays an agent is view-bound.

**The page's own cost** (094): headless Edge's main thread while the game runs, busy
milliseconds a wall second, early (a new game) and late (`everything`): under 10 at 1x,
about 35-70 at 200x. A wake (every 100 ms) costs ~1.2-3 ms of engine calls (wasm), and
the rest is Preact's redraw. `experiments/results/094_profiling_pass/page_profile.mjs`
measures it against a `vite preview` (an unminified build, `npx vite build --minify false`,
names the functions); `wake_wasm.mjs` there splits a wake's wasm calls.

**Tuning a sim player:** one ladder with named knob variants, `--agents
"a=author:fill=4,b=author:fill=8" --seeds 6` (9 author variants × 6 seeds about 1-2 minutes),
needs no rebuild. Hands-off loadout variants the same way, natively: `--agents
"a=handsoff:loadout=out/x/a.txt,b=handsoff:loadout=out/x/b.txt"` plays each file in place of
the built-in `configs/sim/handsoff_loadout.txt` (a path without commas).

## Reads against another revision

Every read of another tree runs from that tree's root (*Paths*, above): a base binary run in
the branch's worktree reads the branch's `configs/` and may fail to load them.

- **Main, or any commit:** `python scripts/baseline.py <rev>` extracts it (`git archive`, no
  worktree registered) to `out/baseline/<sha>/`, builds its release `hh-sim` in its own
  target dir and prints the copy's root and binary: run the binary from that root. It sees
  committed files only.
- **New rules, old agents:** `python scripts/baseline.py HEAD --agents-from main` lays main's
  `crates/hh-sim/src/agents/` over a copy of `HEAD`. Agents and engine must still compile
  together.
- **A content or config what-if** (a readonly item included): a copy as above, edit its
  `content/` or `configs/`, rebuild there with an absolute `CARGO_TARGET_DIR`
  ([README](../README.md#build-and-test)), run from it. A `--set` is enough for a param.
- **Golden plays the same** (a refactor, or a state-only change): diff a pre-change binary's
  `run --policy script --seed 18446744073709551615 --ticks 2000` printout against the new
  one's; when play is unchanged only the fingerprint lines differ. A config-only change
  (a new key) that plays the same moves no fingerprint at all (the fingerprint leaves the
  config out), so the ladder's pins prove it directly.
