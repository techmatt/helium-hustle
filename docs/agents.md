# Agents

The simulator's players (`crates/hh-sim/src/agents/`), weakest first. What they're measured
against: [principles.md](principles.md); how they're run: [simulator.md](simulator.md).

- **The view and actions only.** An agent gets a `Player`: `view()` (the engine's `View`, as
  the web client gets it) and `apply(action)`. It never sees the state. What an agent needed
  that the view lacked went into the view: each building's content rates (`produces`,
  `upkeep`, `stores`), a pad's `fuel` and `load` (cargo per Load), and the run's totals
  (`run_stats`). The one exception is the oracle's privilege (Matt's ruling: it may see the
  future): `Player::state()` and `Player::fork()`, a copy of the game to play ahead on
  (`Session::fork`, an empty record). It still acts only through `apply`.
- **Game time only.** A player acts at a look, then says how many ticks to skip to its next
  (`Agent::next_look`, below): no speed or attention setting touches a game. Real time is a
  human model's projection afterwards ([principles.md](principles.md#players-and-human-models)).
- **Deterministic.** Same seed, config and agent give the same game, on any thread
  count. An agent's randomness is its own stream (`hh_core::rng::Stream`, keyed by the game
  seed and `agent:<kind>`); agents use integers and `Fx`, never floats, so a decision can't
  differ between native and wasm. Every game's record is replayed and must reach its state.
- **Spec syntax:** `[name=]kind[:knob=value,...]`, integer knobs (`author:hand_gap=0,compute=0`);
  a name labels a player (`casual=author:min_gap=120`, the ladder's casual player); an
  unknown knob is an error.

| Rung | What it does | Knobs (defaults) |
|---|---|---|
| `random` | Each look, `actions` times: one action uniform over what the view offers (an affordable building or research, land, a random 1-4 line program on a random program, free processors to a random program, a random rate on an open project); retires with chance 1 in `retire_odds` a look; looks every `every` ticks. | `actions` 2, `retire_odds` 500, `every` 5 |
| `greedy` | The **builder**: buys the best-scored building it can afford (land included), up to `buys` a look, never saving; the cheapest affordable research; one command on the first program with every processor, chosen afresh each look. No projects, no voluntary retirement. Looks again when its target comes within reach, at most `max_gap` ticks on. | `buys` 3, `decay`, `starved` 100, `reserve` 3, `min_gap` 1, `max_gap` 10 |
| `author` | The **program author**: the builder's score, but saves for its target; writes programs from templates; research, projects, ideology, Dream, market commands; retires a stale run. | the builder's, `min_gap` 1, `max_gap` 10, `compute` 1, `hand_gap` 60, `loads` 20, `pause` 1, `wait_demand` 0, `dream_at` 60, `stale_days` 300, `project_rate` 3, `fund` 1, `market` 1 |
| `planner` | **Not built.** A strategic player that plans across retirements without peeking: the hook is `Agent::new_run`, called once a run: at the first look after a forced retirement, before `act`; after a retirement the agent chose, at the end of that look (whatever else `act` did in the new run came first, so a planner should retire last in its look). | |
| `oracle` | The **ceiling**: the author steered by rollouts on copies of the game, the future included (below). Profiles: `profile` 0 milestones, 1 credits, 2 ideology, 3 builder, 4 quests. | `profile` 0, `every` 50, `horizon` 600, `width` 5, `beam` 1, `margin` 20, `long` 2000, `replan` 500, `strategy` -1 (search), `retire` 1, `min_run` 1000, `retire_margin` 0, `first_margin` 0, `early` and `ideology` (the profile's weights), `tail` 400, `hoard_at` 5000, `stale_days` 300, `max_gap` 10, `roll_gap` = `max_gap` |

## The builder (`builder.rs`)

A scored build order, roughly the Godot optimizer's greedy over purchase orderings. Each unlocked
building scores its base weight (`WEIGHTS`: Data Center 90 first; Storage Depot and Battery 0,
bought only when a price needs them, below) times its keep percent per copy owned (Launch Pad
40%, Data Center 85%; `decay` sets one for all), then:

- 0 if every product sits at its cap (no more of a capped pile);
- +`starved`% for the maker of an input some building is starved of (Solar Panel for energy,
  Excavator for regolith, Ice Extractor, Fabricator, Electrolysis);
- +`starved`% for the first Launch Pad while a tradeable good sits at its cap.

The best is then **resolved** to what it needs first, up to four steps: an energy user that
would leave less than `reserve` spare energy a day (content rates of active buildings) gives
way to a Solar Panel; a price past a cap gives way to a Storage Depot (a Battery for energy).
Land is bought first when short and credits cover it and the price.

A hand-played command (`hand_command`, greedy's): launch a full fuelled pad; fuel a full pad;
load when a load's worth of the pad's good waits; bridge the target's missing titanium, regolith or ice with
its Buy command when credits past the target's price cover one; else Cloud Compute (Idle
without the energy).

## The program author (`author.rs`)

The rung that matters most: program authoring is the core skill. Its target is the best-scored
building whose missing inputs can be made or bought; one short of an input nothing makes and no
command buys (circuits) works toward that input's maker, or the building that unlocks the maker
(the view's `requires`). It buys another building only from credits past the target's price.
How it writes the main program depends on the gap to its next look (*When it looks*):

| Program | Template | Processors |
|---|---|---|
| 1, gap ≤ `hand_gap` | a plan to the next look: `launch_pads` if a pad is full and fuelled (with `wait_demand=1`, only into HIGH demand or better, MEDIUM once the good piles at its cap: it earned less on 4 seeds); `buy_propellant` × the fuel short when a pad is within 25 of full; `load_pads` × the loads that would move cargo; `buy_*` × the target's missing titanium, regolith or ice as far as spare credits go; then `cloud_compute` × 2 × `max_gap` × processors | the rest |
| 1, longer gaps | a standing cycle that earns before it spends: `load_pads × loads`, `launch_pads`, `cloud_compute × loads`, `buy_propellant × fuel`, `buy_*` for target inputs no building makes | the rest |
| 2 | `dream` | 1 while boredom is past `dream_at`% of its cap and the run still builds |
| 3 | `fund_<axis>`: the first of humanist, rationalist, nationalist whose persistent project isn't done | 1 when 3 or more are left |
| 4 | `promote_<good>` per pad good, `disrupt_speculators` | 1 when 3 or more are left |

A program is rewritten only when its text changes (a rewrite restarts its pointer; the main
plan's re-arm: *When it looks*). Extra pads
carry a good other than He-3 that sits at its cap. While looking often (gap ≤ `hand_gap`) it
pauses buildings that eat a material its target is short of, so the stock builds up, and runs
them again once it isn't.

**Science.** Research resets each run; a Long-Term project's progress doesn't. Research comes
in a fixed order (Propellant Synthesis, Market Awareness, Dream Protocols, Stress Tolerance,
Shipping Efficiency, Geopolitical Intelligence, ...), but while a Long-Term project is open only
the essential ones (`ESSENTIAL`: Propellant Synthesis, Market Awareness until its quest is done,
Dream Protocols, Stress Tolerance, Geopolitical Intelligence) are bought. **Projects:** the
first open one in a fixed order, Long-Term first (Lunar Cartography, Foundation Grant, the three
ideology projects, Chemical Energy), then Strategic; only when every cost can move (on hand or
made); science at the project's most a day when no research worth it is waiting, credits a tenth
of those past twice the target's price plus 200 a day, anything else `project_rate`. It retires
voluntarily once the run has bought nothing for `stale_days` days.

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

The author's plan is rewritten only when its text changes, or to run its due lines (all but
the filler) again once the pointer has passed them into the filler; a plan with nothing due
isn't written over one already in its filler with enough left to reach the next look. So a
decision (a look that applied an action)
is a real change, and `max_gap` 10 gives about 140 decisions a run (tuned on dev seeds 1-3: 5,
10, 20, 30 and 60 tried; 5 and 10 earned about the same, 12,000-14,000 credits a run, 10 with
half the decisions; 20 and longer earned less and met M3 later).

## The oracle (`oracle.rs`)

The reasonable upper bound on a dedicated human, not an optimum: no per-tick search, no speed
model. The author plays every look (looking at most every `max_gap`); the oracle sets its
**goals** (`author::Directives`, none set for the plain author): a building to save for,
research first, a project pushed (all spare credits; science the research doesn't want), an
axis funded on N processors, Dream held against boredom (`sustain`: enough Dream processors
to outpace growth, Dream's program first, energy users leave room for two Dreams, no stale
retirement), all research, the market program on one processor once two are free, a different
good on each extra pad, hoarding, and the main program run last (`main_last`, so Dream, Fund
and Promote get energy before Cloud Compute's filler). It chooses them by **rollouts**: the
author, cloned, plays a copy of the game under the goals, and the score picks. The real game
looks exactly when the author would (plans fall due at its next look) and rollouts look as
often, so a rollout of the current goals is what will happen: with extra looks at plan times
the oracle's first shipment and second Data Center came 31 and 18 ticks after the author's;
without, on the same ticks.

- **Strategy** (each run's start, every `replan` ticks): standing bundles (`strategies`: the
  author's own; *trade*: Trade Promotion first, market, diversified pads, main last; trade
  with the main program first; trade + sustain, + all research, + Foundation Grant; once the
  market quest is done, trade + each axis funded on 2 processors, with and without sustain),
  each played `long` ticks through retirements. The quests profile, while Open Horizons is
  open, has one: its campaign (trade, Dream held, Humanist funded, then AI Consciousness).
- **Tactics** (every `every` ticks, and once its target is bought): one change each to the
  target (every unlocked building), the research (once the strategy's is done), the project
  (when the strategy names none) or hoarding (from `hoard_at` credits on hand); all screened
  over a third of `horizon`, the best `width` over all of it, the two best together. A change
  must beat keeping the goals by `margin` per mille. 300 ticks steered off slow chains
  (circuits for a second Data Center) toward quick payback; 600 earned a third more.
- **Score**, in credits: credits earned; the closing income of the run under way at the end
  times the days it has left (room to the boredom cap at its closing growth, at most `tail`);
  and the profile's weights on what the career keeps (ideology best scores, Long-Term project
  progress, building types, commands, quests and sub-objectives, achievements, research), on
  each open milestone met (`configs/sim/milestones.json`, embedded: 4,000 plus `early` 100 a
  tick sooner) and on each open milestone's progress squared (`progress.rs`). Credits weighs
  credits alone.
- **Retirement**, checked every 100 ticks from day `min_run`: it plays the run out on a copy
  and retires once the run's average credits a day is the best it will see (the prestige
  rule). Judging it by rollouts of retiring against staying instead favored retiring at once:
  the retiring branch ends its window in a mature run. Before `min_run` it never retires: a
  retirement re-rolls the markets for free, which a peeking player farms, and the run played
  out under fixed goals underestimates a young run (400 let runs end at day 625-730).
  `first_margin` loosens the rule for the first retirement (a milestone) in a profile that
  scores milestones: 100 and 200 per mille didn't put it reliably before the author's and 200
  halved one seed's run 1 credits.

It plays about 30 ticks a second (a rollout a few milliseconds): a career of 20 retirements
is about 18 minutes on one thread. Results and what blocks each milestone:
[experiments/results/011_oracle_planner/](../experiments/results/011_oracle_planner/README.md).

## Adding a rung

A rung is a module under `agents/` implementing `Agent` (`act`, `next_look`, and `new_run` for
plans across retirements), named in `agents::build` and `RUNGS`, reading only the `Player`. A test plays it
in a small grid (`crates/hh-sim/tests/cli.rs`).
