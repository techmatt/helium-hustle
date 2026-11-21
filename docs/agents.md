# Agents

The simulator's players (`crates/hh-sim/src/agents/`), weakest first. What they're measured
against: [principles.md](principles.md); how they're run: [simulator.md](simulator.md).

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
- **Spec syntax:** `[name=]kind[:knob=value,...]`, integer knobs; a name labels a player
  (`casual=author:min_gap=120`, the ladder's casual player); an unknown knob is an error.
- **Agent code is player code** ([rulings.md](rulings.md)).
- **Pinned games.** An agent edit changes how a player plays, not the game: no
  `RULES_REVISION` bump. Agent edits are guarded by the pinned games (`PINNED` and the
  oracle's, in `crates/hh-sim/tests/cli.rs`): a refactor keeps them.

| Rung | What it does | Knobs (defaults) |
|---|---|---|
| `random` | Each look, `actions` times: one action uniform over what the view offers (an affordable building or research, land, a random 1-4 line program on a random program, free processors to a random program, a random rate on an open project); retires with chance 1 in `retire_odds` a look; looks every `every` ticks. | `actions` 2, `retire_odds` 500, `every` 5 |
| `greedy` | The **builder**: buys the best-scored building it can afford (land included), up to `buys` a look, never saving; the cheapest affordable research; one command on the first program with every processor, chosen afresh each look. No projects, no voluntary retirement. Looks again when its target comes within reach, at most `max_gap` ticks on. | `buys` 3, `decay`, `starved` 100, `reserve` 3, `min_gap` 1, `max_gap` 10 |
| `author` | The **program author**: the builder's score, but saves for its target; writes condition programs kept in the library; research, projects, ideology, Dream, market commands; retires a stale run. | the builder's, `min_gap` 1, `max_gap` 10, `compute` 1, `hand_gap` 60, `loads` 20, `pause` 1, `wait_demand` 0, `dream_at` 60, `stale_days` 300, `project_rate` 3, `fund` 1, `market` 1, `construct` 1, `cond` 1, `diversify` 0, `all_research` 0 (the autopilot's) |
| `handsoff` | **Automation alone**: run 1 is the author's; from run 2 it loads its programs and seats processors at each reset, nothing else. | the author's, `every` 10 |
| `planner` | **Not built.** A strategic player that plans across retirements without peeking: the hook is `Agent::new_run`, called once a run: at the first look after a forced retirement, before `act`; after a retirement the agent chose, at the end of that look (whatever else `act` did in the new run came first, so a planner should retire last in its look). | |
| `oracle` | The **ceiling**: the author steered by rollouts on copies of the game, the future included (below). Profiles: `profile` 0 milestones, 1 credits, 2 ideology, 3 builder, 4 quests. | `profile` 0, `every` 50, `horizon` 600, `width` 5, `beam` 1, `margin` 20, `long` 2000, `replan` 500, `strategy` -1 (search), `retire` 1, `min_run` 1000, `retire_margin` 0, `first_margin` 0, `early` and `ideology` (the profile's weights), `tail` 400, `hoard_at` 5000, `stale_days` 300, `max_gap` 10, `roll_gap` = `max_gap` |

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

A hand-played command (`hand_command`, greedy's): launch a full fuelled pad; fuel a full pad;
load when a load of its good waits; buy the target's missing titanium, regolith or ice from
credits past its price; else Cloud Compute (Idle without the energy).

## The program author (`author.rs`)

The rung that matters most: program authoring is the core skill. Its target is the best-scored
building whose missing inputs can be made or bought; one short of an input nothing makes and no
command buys (circuits) works toward that input's maker, or the building that unlocks the maker
(the view's `requires`). It buys another building only from credits past the target's price.
How it writes the main program depends on the gap to its next look (*When it looks*):

| Program | Template | Processors |
|---|---|---|
| 1, gap ≤ `hand_gap` | a plan to the next look: `launch_pads` if a pad is full and fuelled (with `wait_demand=1`, only into HIGH demand or better, MEDIUM once the good piles at its cap); `buy_propellant` × the fuel short when a pad is within 25 of full; `load_pads` × the loads that would move cargo, at most `loads`; `buy_*` × the target's missing titanium, regolith or ice as far as spare credits go; then `cloud_compute` × 2 × `max_gap` × processors | the rest |
| 1, longer gaps | a standing cycle that earns before it spends: `load_pads × loads`, `launch_pads`, `cloud_compute × loads`, `buy_propellant × fuel`, `buy_*` × the target's missing inputs no building makes, at most `loads` | the rest |
| 2 | `dream` | 1 while boredom is past `dream_at`% of its cap and the run still builds |
| 3 | `fund_<axis>`: the first of humanist, rationalist, nationalist whose persistent project isn't done | 1 when 3 or more are left |
| 4 | `promote_<good>` per pad good, `disrupt_speculators` | 1 when 3 or more are left |
| 5 (`construct`) | construction: `if Land free < its land and Credits >= the land price` / `buy land` (while short of land); `if Credits >= the target's price and <its scarcest other input> >= its price` / `build <target> up to owned + 1`; then the filler × 1 | 1 when 2 or more are left, once the game unlocks `if` and `build` (not under the oracle's `main_last`: the main program takes the slot) |

**Condition programs** (`cond` 1 once `if` is open; the oracle runs the table's, `cond` 0)
read the game, not this look's counts, so each is saved once by name; side programs load at
each reset and keep their lines while unassigned (`author.rs`): `ship` (1: once a pad stands, fuel, `launch pads all`, `load pads all` while
`cargo of pads < Launch Pad count × 100`, `set pads he3 up to 1` while no pad carries He-3,
the same `up to 1` for a good at its cap while two carry He-3; the filler; a hand plan replaces it while the target waits on a bought
input), `dream watch` (2), `fund`, `market`, and `build order` (5: the last run's purchases
as `build X up to N`, each gated on the one before and on fewer than N owned, a short input
bought first, `research` once science covers it and it isn't researched; rewritten when a
run bought past it). Guards skip done lines (a step each).

A program is rewritten only when its text changes (a rewrite restarts its pointer), through
the library (`Player::put_program`). Extra pads carry a good other
than He-3 at its cap. Looking often (gap ≤ `hand_gap`), it pauses buildings that eat a
material its target is short of, and runs them again once it isn't.

**Science.** Research resets each run; a Long-Term project's progress doesn't. Research comes
in a fixed order (`author.rs`), but while a Long-Term project is open only the essential ones
(`ESSENTIAL`; Market Awareness until its quest is done) are bought. **Projects:** the first
open one in a fixed order, Long-Term first, then Strategic; only when every cost can move (on hand or
made); science at the project's most a day when no research worth it is waiting, credits a tenth
of those past twice the target's price plus 200 a day, anything else `project_rate`. It retires
once the run has bought nothing for `stale_days` days.

## When it looks

Every player looks whenever a decision is useful and skips stretches that need none; how
often is a knob, never a profile. After acting, the author reads, from the view alone (integers
and `Fx`), the ticks until: its target is affordable (credits at its own reading of its income,
credits earned this run between its last two looks a day; other costs at the view's
`per_day`); the next research it wants is affordable; boredom reaches `dream_at`; the pad's
cooldown ends; a stale run would retire. The soonest (one already true doesn't count) is its
gap, clamped to `min_gap`..`max_gap`; `max_gap` covers what it can't foresee (events, demand,
a stall). Greedy waits for its target or `max_gap`; random looks every `every` ticks; a forced
retirement wakes any player. The casual player is the author with `min_gap` 120: it writes the
standing cycle and looks about 12 times a run.

A hand plan is also rewritten to run its due lines again once the pointer passed into its
filler, never over one that reaches the next look: about 90 decisions a run.

**The hands-off player** (`handsoff.rs`) is the author in run 1; then at each new run it loads
the five programs by name (run 1's build order saved first) and seats processors (build
order first, one each while two are left, the rest to Program 1), and makes no other
decision. Its build order leads with what were hands: processors (all to it until a pad
stands, then one; a side program seated once enough work), Smelters and Fabricators paused
while their product is at cap (on again under 90%), a Storage Depot while titanium is and
credits reach 3× its price. The Dream watch gains, before its filler: every 49
days at 3000 credits, `fund` most open projects at 3 a day; every 9 days a free
processor to Program 1; `retire` once `days since last purchase` reaches `stale_days`; the
author's hand research (odd periods: a guard read every other day lands).

## The oracle (`oracle/`)

`score.rs` (profiles, rollouts, the score), `strategy.rs` (the strategy table, tactics),
`retire.rs` (the retirement rule), `mod.rs` (knobs, each look). Its spec is
`experiments/ceiling.json` (beside the author), not the ladder: its careers take minutes.

The reasonable upper bound on a dedicated human, not an optimum: no per-tick search, no speed
model. The author plays every look (looking at most every `max_gap`); the oracle sets its
**goals** (`author::Directives`, each field's doc says what it changes): a building to save
for, research first, a project pushed, an axis funded, Dream held against boredom
(`sustain`), all research, the market program, diversified pads, hoarding, and the main
program run last (`main_last`). It chooses them by **rollouts**: the
author, cloned, plays a copy of the game under the goals, and the score picks. The real game
looks exactly when the author would (plans fall due at its next look) and rollouts look as
often, so a rollout of the current goals is what will happen.

- **Strategy** (each run's start, every `replan` ticks): standing bundles, a fixed table
  (`strategies`, its rows documented there; `strategy=k` pins row k): the author's own,
  *trade* variants, trade + sustain, research or Foundation Grant, and trade + each axis
  funded. Each is played `long` ticks through retirements. The quests profile, while Open Horizons is
  open, has one: its campaign (trade, Dream held, Humanist funded, then AI Consciousness).
- **Tactics** (every `every` ticks, and once its target is bought): one change each to the
  target (every unlocked building), the research (once the strategy's is done), the project
  (when the strategy names none) or hoarding (from `hoard_at` credits on hand); all screened
  over a third of `horizon`, the best `width` over all of it, the two best together. A change
  must beat keeping the goals by `margin` per mille.
- **Score**, in credits (`score.rs`): credits earned, the run's closing income over the days
  it has left (at most `tail`), and the profile's weights on what the career keeps, on each
  open milestone met (4,000 plus `early` 100 a tick sooner) and on its progress squared
  (`progress.rs`). Credits weighs credits alone.
- **Retirement** (`retire.rs`), tried every 100 ticks from day `min_run`: it plays the run out
  on a copy and retires once the run's average credits a day is the best it will see. Never
  before `min_run` (a retirement re-rolls the markets, which a peeking player farms);
  `first_margin` loosens the first one in a profile that scores milestones.

It plays about 30 ticks a second (a rollout a few milliseconds): a career of 20 retirements
is about 18 minutes on one thread. Results and what blocks each milestone:
[experiments/results/011_oracle_planner/](../experiments/results/011_oracle_planner/README.md).

## Adding a rung

A rung is a module under `agents/` implementing `Agent` (`act`, `next_look`, and `new_run` for
plans across retirements), named in `agents::build` and `RUNGS`, reading only the `Player`. A test plays it
in a small grid (`crates/hh-sim/tests/cli.rs`).
