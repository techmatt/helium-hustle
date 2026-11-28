# Agents

The simulator's players (`crates/hh-sim/src/agents/`). What they're measured against:
[principles.md](principles.md); how they're run: [simulator.md](simulator.md),
[experiments.md](experiments.md).

## Roster

Each player has one job (131's review): balance work reads the three tiers, the fixtures
guard engine and agent code.

| Player (spec) | Role | Read by |
|---|---|---|
| `random` | **Fixture**: uniform actions; replay, refusal and robustness floor | ladder, `PINNED` |
| `greedy` | **Fixture**: the builder alone (no programs, projects or retirement) | ladder, `PINNED` |
| `casual` (`author:min_gap=120`) | **Casual tier**: the author looking about 12 times a run | ladder, the judge (`balance.json`) |
| `author` | **Medium tier**: programs, research, projects, Dream, retirement | ladder, the judge, `axes` medium (`commit=k`) |
| `handsoff` | **Measure**: Act 1's exit, automation alone ([rulings/sim.md](rulings/sim.md)) | ladder *Hands-off*, `HANDSOFF_PIN` |
| `oracle` | **Skilled tier**: the author steered by rollouts, future included ([oracle.md](oracle.md)) | `ceiling.json`, `axes` skilled (`profile=4,commit=k`), `vs-human --skilled` (uncommitted: picks its axis) |
| human models (`configs/sim/humans.json`) | **Human model**: real minutes from looks; not a player | ladder `--humans`, `vs-human` |

Each is read by a check or a tier. Not built: `planner` (plans across retirements without
peeking; hook `Agent::new_run`).

**Missing for balance work** (not built): a cheaper skilled proxy (the oracle with
`long`/`horizon` cut, or the committed author with the oracle's run-1 project plan, checked
against the oracle; a read's cost: [experiments.md](experiments.md), *Costs*); a
human-calibrated tier from Matt's logged careers (`vs-human --skilled` sets one beside it).

## Rules every player follows

- **The view and actions only.** An agent gets a `Player`: `view()` (the engine's `View`, as
  the web client gets it) and `apply(action)`. It never sees the state: what an agent needs
  that the view lacks goes into the view. The one exception is the oracle's privilege (Matt's ruling: it may see the
  future), its privileged seats: `Player::state()` (the whole state), `Player::session()`
  (the session, for its milestone tracker) and `Player::fork()`, a copy of the game to play
  ahead on (`Session::fork`). It still acts only through `apply`.
- **Game time only.** A player acts at a look, then says how many ticks to skip to its next
  (`Agent::next_look`, below): no speed or attention setting touches a game. Real time is a
  human model's projection afterwards ([principles.md](principles.md#players-and-human-models)).
- **Deterministic.** Same seed, config and agent give the same game, on any thread
  count. An agent's randomness is its own stream (`hh_core::rng::Stream`, keyed by the game
  seed and `agent:<kind>`); agents use integers and `Fx`, never floats, so a decision can't
  differ between native and wasm. Every game's record is replayed and must reach its state.
- **Spec syntax:** `[name=]kind[:knob=value,...]`, integer knobs (but `handsoff`'s native
  `loadout=<path>`, docs/experiments.md); a name labels a player
  (`casual=author:min_gap=120`, the ladder's casual player); an unknown knob is an error.
- **Agent code is player code** ([rulings/sim.md](rulings/sim.md)).
- **Pinned games.** An agent edit changes how a player plays, not the game: no
  `RULES_REVISION` bump. Agent edits are guarded by the pinned games (`PINNED`, `HANDSOFF_PIN` and
  the oracle's, in `crates/hh-sim/tests/cli.rs`): a refactor keeps them.

| Rung | What it does |
|---|---|
| `random` | Each look, `actions` times: one action uniform over what the view offers (an affordable building or research, land, a random 1-4 line program on a random program, free processors to a random program, a random rate on an open project); points a No-cargo pad at the first good it may carry (`pick_cargo`), launches a full pad by hand; retires with chance 1 in `retire_odds` a look; looks every `every` ticks. |
| `greedy` | The **builder**: buys the best-scored building it can afford (land included), up to `buys` a look, never saving; the cheapest affordable research; one command on the first program with every processor, chosen afresh each look. No projects, no voluntary retirement. Looks again when its target comes within reach, at most `max_gap` ticks on. |
| `author` | The **program author**: the builder's score, but saves for its target; writes condition programs kept in the library; research, projects (the cheapest open `run` one beside the first other in `PROJECTS` order: re-funded after a timeline's end), ideology, Dream, market commands; retires a stale run. `all_research` is a no-op today (the author looks only at `RESEARCH`). |
| `handsoff` | **Automation alone**: the author's until the run `if` opens in (the loadout guards with `if`; again once a timeline's end closes it), then from the next run it loads the best known loadout (`configs/sim/handsoff_loadout.txt`) and seats processors at each reset; its one hand is a stale-run retirement, a backstop to the loadout's own `retire` line. |
| `oracle` | The **ceiling** ([oracle.md](oracle.md)). Profiles: `profile` 0 milestones, 1 credits, 2 ideology, 3 builder, 4 quests. |

Knobs and their defaults: each rung's `KNOBS` list (`random.rs`, `builder.rs`, `author.rs`,
`oracle/mod.rs`) and the `k.get(name, default)` beside it; `hh-sim agents` lists the rungs.

Research: random's in `Random::act`, greedy's cheapest affordable, the author's
`next_research` (handsoff and the oracle inherit it); all skip `agents::NEVER_BUY`.

## The builder (`builder.rs`)

A scored build order, roughly the Godot optimizer's greedy over purchase orderings. Each unlocked
building scores its base weight (`WEIGHTS`: Data Center 90 first; Storage Depot and Battery 0,
bought only when a price needs them, below) times its keep percent per copy owned (Launch Pad
40%, Data Center 85%; `decay` sets one for all), then:

- 0 if every product sits at its cap (no more of a capped pile);
- +`starved`% for the maker of an input some building is starved of (Solar Panel for energy);
- +`starved`% for the first Launch Pad while a tradeable good sits at its cap.

The best is then **resolved** to what it needs first, up to four steps: an energy user that
would leave less than `reserve` spare energy a day (content rates of active buildings) gives
way to a Solar Panel; a price past a cap gives way to a Storage Depot (a Battery for energy).
Land is bought first when short and credits cover it and the price.

Every player (random, greedy, the author and those built on it) has one fixed hand on pads:
a pad with No cargo, or a first pad on another good (empty, or its cargo fits back in the
stockpile), carries the first good a pad may carry now (`pad_goods`: Titanium during 1.2,
then He-3; `agents::pick_cargo`, each look; the golden script re-points every empty pad
likewise). Until Launch Full Pads opens (1.3), every player hand-launches a full pad or one
holding a `launch_cargo` quest's amount (`agents::hand_launch`). The hands-off loadout
sets `set pads he3 up to 1`.

A hand-played command (`hand_command`, greedy's): launch a full fueled pad; fuel a full pad;
load when a load of its good waits; Buy Science while short of the next Blueprints on
sale; buy the target's missing titanium, regolith or ice from
credits past its price; else Cloud Compute (Idle without the energy).

## The program author (`author.rs`)

The program author, the medium tier and the base of the casual, hands-off and oracle
players: its target, programs, research and projects are in [author.md](author.md).

## When it looks

Every player looks when a decision is useful; how often is a knob. After acting, the author
reads, from the view alone, the ticks until: its target is affordable (credits at this run's
income a day between its last two looks; other costs at the view's `per_day`); the next
research it wants is affordable; boredom reaches `dream_at`; the pad's cooldown ends; a stale
run would retire. The soonest not already true is its gap, clamped to
`min_gap`..`max_gap`; `max_gap` covers what it can't foresee (events, demand, stalls). Greedy
waits for its target or `max_gap`; random looks every `every` ticks; a forced retirement wakes
any player. The casual player (`min_gap` 120) writes the standing cycle, about 12 looks a run.

A hand plan is also rewritten to run its due lines again once the pointer passed into its
filler, never over one that reaches the next look: about 90 decisions a run.

**The hands-off player** (`handsoff.rs`): its programs are data,
`configs/sim/handsoff_loadout.txt` (the form: `agents/loadout.rs`; what its tabs do: the
file's header), replaced when the rules or the best loadout change (then re-pin
`HANDSOFF_PIN`; a refused tab panics). Each reset it puts the file's tabs in through the
library (saved as `hands-off N`) and seats processors by its `seat` line; a tab reading
research not yet done goes in without those `if` blocks and is rewritten once it is.

## The oracle (`oracle/`)

The ceiling: the author steered by rollouts on copies of the game, the future included:
[oracle.md](oracle.md).

## Adding a rung

A rung is a module under `agents/` implementing `Agent` (`act`, `next_look`, and `new_run` for
plans across retirements), named in `agents::build` and `RUNGS`, reading only the `Player`. A test plays it
in a small grid (`crates/hh-sim/tests/cli.rs`).
