# Agents

The simulator's players (`crates/hh-sim/src/agents/`). What they're measured against:
[principles.md](principles.md); how they're run: [simulator.md](simulator.md),
[experiments.md](experiments.md).

## Roster

Each player has one job (131's review). Balance work reads the three tiers; the fixtures guard
the engine and agent code; the human model maps game time to real minutes.

| Player (spec) | Role | Read by |
|---|---|---|
| `random` | **Fixture**: uniform actions; replay, refusal and robustness floor | ladder, `PINNED` |
| `greedy` | **Fixture**: the builder alone (no programs, projects or retirement) | ladder, `PINNED` |
| `casual` (`author:min_gap=120`) | **Casual tier**: the author looking about 12 times a run | ladder, the judge (`balance.json`) |
| `author` | **Medium tier**: programs, research, projects, Dream, retirement | ladder, the judge, `axes` medium (`commit=k`) |
| `handsoff` | **Measure**: Act 1's exit, automation alone ([rulings/sim.md](rulings/sim.md)) | ladder *Hands-off*, `HANDSOFF_PIN` |
| `oracle` | **Skilled tier**: the author steered by rollouts, future included ([oracle.md](oracle.md)) | `ceiling.json`, `axes` skilled (`profile=4,commit=k`), `vs-human --skilled` (uncommitted: picks its axis) |
| human models (`configs/sim/humans.json`) | **Human model**: real minutes from looks; not a player | ladder `--humans`, `vs-human` |

None merged or retired: each is read by a check or a tier. Not built: `planner` (plans
across retirements without peeking; hook `Agent::new_run`).

**Missing for balance work** (recommended, not built): a cheaper skilled proxy (a skilled
`axes` read is ~3.5 min on 6 threads: the oracle with `long`/`horizon` cut, or the committed
author with the oracle's run-1 project plan, checked against the oracle on a few seeds); an
early stop in `axes` once a case's bar is decided (`--runs 1` already cuts a run-1 question
short); a human-calibrated tier from Matt's logged
careers (`vs-human --skilled` sets one beside the skilled player).

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

Its choices (target, research, projects, when to look) are in `author.rs`; every program text
it writes, saved or not, is built in `author/templates.rs`. The rung that matters most: program authoring is the core skill. Its target is the best-scored
building whose missing inputs can be made or bought; one short of an input nothing makes and no
command buys (circuits) works toward that input's maker, or the building that unlocks the maker
(`requires`). A resolved target still past a cap (a ninth Storage Depot) gives way to the
building itself if under its caps, else is skipped (a maker step past a cap is skipped). It
buys others only from credits past the target's price.
How it writes the main program depends on the gap to its next look (*When it looks*):

| Program | Template | Processors |
|---|---|---|
| 1, gap ≤ `hand_gap` | a plan to the next look: `launch_pads` if a pad is full and fueled (with `wait_demand=1`, only into HIGH demand or better, MEDIUM once the good piles at its cap); `buy_propellant` × the fuel short when a pad is within 25 of full; `load_pads` × the loads that would move cargo, at most `loads`; `buy_*` × the target's missing titanium, regolith or ice as far as spare credits go; then `cloud_compute` × 2 × `max_gap` × processors | the rest |
| 1, longer gaps | a standing cycle that earns before it spends: `load_pads × loads`, `launch_pads`, `cloud_compute × loads`, `buy_propellant × fuel`, `buy_*` × the target's missing inputs no building makes, at most `loads` | the rest |
| 2 | `dream` | 1 while boredom is past `dream_at`% of its cap and the run still builds |
| 3 | `fund_<axis>`: the first of humanist, rationalist, nationalist whose persistent project isn't done | 1 when 3 or more are left |
| 4 | `promote_<good>` per pad good, `disrupt_speculators` | 1 when 3 or more are left |
| 5 (`construct`) | construction: `if Free land < its land and Credits > the land price` / `buy land` (while short of land); `if Credits > the target's price and <its scarcest other input> > its price` (a capped input as a percent of its cap now) / `build <target> up to owned + 1`; then the filler × 1 | 1 when 2 or more are left, once the game unlocks `if` and `build` (not under the oracle's `main_last`: the main program takes the slot) |

**Condition programs** (`cond` 1 once `if` is open; the oracle runs the table's, `cond` 0)
read the game, not this look's counts, so each is saved once by name; side programs load at
each reset and keep their lines while unassigned (`author/templates.rs`). Every `if` costs a
step (rev 19) and a bare line that can't act costs the same one step, so they guard only what
a bare line would waste: `ship` (1, its numbers percents of the caps when written; with
no pad, the filler alone; once one stands, fuel while Propellant is under a launch's worth
and a pad holds cargo, `launch pads all` bare, `load pads all` `ship_loads` times once
Helium-3 holds that many loads; with two pads or more, `set pads he3 up to 1` bare and the same
`up to 1` for a good `> 99%` while two are idle; then the filler × `fill`; a hand plan
replaces it while the target waits on a bought input), `dream watch` (2: dream past
`dream_at`, else the filler × `dream_fill`), `fund`, `market`, and `build order` (5: the last run's
purchases as bare `build X up to N`, one line a building in the order first bought (no count
reading, so a step can't wait on the one before; short of credits it fails in a
step), a short input bought first under `if` it is short `and` credits cover it, bare
`research` lines; rewritten when a run bought past it). Done lines (a researched item, a
finished `build`) cost a step each loop.

A program is rewritten only when its text changes (a rewrite restarts its pointer), through
the library (`Player::put_program`). Extra pads carry a good other
than He-3 at its cap. Looking often (gap ≤ `hand_gap`), it pauses buildings that eat a
material its target is short of, and runs them again once it isn't.

**Science.** Research resets each run; a `timeline` project's progress doesn't. Research comes
in a fixed order (`author.rs`), the Blueprints first; while short of the next one on sale,
its main program opens with Buy Science × as many as the credits past the target's price
cover. **Projects:** the cheapest open `run` project, beside the
first open other one in a fixed order; each only when every cost can move (on hand or
made); science at the project's most a day when no research waits, else `science_share`
percent of it; credits a tenth
of those past twice the target's price plus 200 a day, anything else `project_rate`. It retires
once the run has bought nothing for `stale_days` days.

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
