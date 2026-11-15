# Agents

The simulator's players (`crates/hh-sim/src/agents/`), weakest first. What they're measured
against: [principles.md](principles.md); how they're run: [simulator.md](simulator.md).

- **The view and actions only.** An agent gets a `Player`: `view()` (the engine's `View`, as
  the web client gets it) and `apply(action)`. It never sees the state. What an agent needed
  that the view lacked went into the view: each building's content rates (`produces`,
  `upkeep`, `stores`), a pad's `fuel` and `load` (cargo per Load), and the run's totals
  (`run_stats`).
- **Deterministic.** Same seed, config, agent and profile give the same game, on any thread
  count. An agent's randomness is its own stream (`hh_core::rng::Stream`, keyed by the game
  seed and `agent:<kind>`); agents use integers and `Fx`, never floats, so a decision can't
  differ between native and wasm. Every game's record is replayed and must reach its state.
- **Spec syntax:** `kind[:knob=value,...]`, integer knobs (`author:hand_gap=0,compute=0`); an
  unknown knob is an error.

| Rung | What it does | Knobs (defaults) |
|---|---|---|
| `random` | Each look, `actions` times: one action uniform over what the view offers (an affordable building or research, land, a random 1-4 line program on a random program, free processors to a random program, a random rate on an open project); retires with chance 1 in `retire_odds` a look. | `actions` 2, `retire_odds` 500 |
| `greedy` | The **builder**: buys the best-scored building it can afford (land included), up to `buys` a look, never saving; the cheapest affordable research; one command on the first program with every processor, chosen afresh each look. No projects, no voluntary retirement. | `buys` 3, `decay`, `starved` 100, `reserve` 3 |
| `author` | The **program author**: the builder's score, but saves for its target; writes programs from templates; research, projects, ideology, Dream, market commands; retires a stale run. | the builder's, `compute` 1, `hand_gap` 60, `loads` 20, `pause` 1, `wait_demand` 0, `dream_at` 60, `stale_days` 300, `project_rate` 3, `fund` 1, `market` 1 |
| `planner` | **Not built.** A strategic player that plans across retirements: the hook is `Agent::new_run`, called once a run: at the first look after a forced retirement, before `act`; after a retirement the agent chose, at the end of that look (whatever else `act` did in the new run came first, so a planner should retire last in its look). | |
| `oracle` | **Not built.** A full-information search, the ceiling. | |

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
How it writes the main program depends on the ticks since its last look:

| Program | Template | Processors |
|---|---|---|
| 1, gap ≤ `hand_gap` | a plan to the next look: `launch_pads` if a pad is full and fuelled (with `wait_demand=1`, only into HIGH demand or better, MEDIUM once the good piles at its cap: it earned less on 4 seeds); `buy_propellant` × the fuel short when a pad is within 25 of full; `load_pads` × the loads that would move cargo; `buy_*` × the target's missing titanium, regolith or ice as far as spare credits go; then `cloud_compute` × gap × processors | the rest |
| 1, longer gaps | a standing cycle that earns before it spends: `load_pads × loads`, `launch_pads`, `cloud_compute × loads`, `buy_propellant × fuel`, `buy_*` for target inputs no building makes | the rest |
| 2 | `dream` | 1 while boredom is past `dream_at`% of its cap and the run still builds |
| 3 | `fund_<axis>`: the first of humanist, rationalist, nationalist whose persistent project isn't done | 1 when 3 or more are left |
| 4 | `promote_<good>` per pad good, `disrupt_speculators` | 1 when 3 or more are left |

A program is rewritten only when its text changes (a rewrite restarts its pointer). Extra pads
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

## Adding a rung

A rung is a module under `agents/` implementing `Agent` (`act`, and `new_run` for plans across
retirements), named in `agents::build` and `RUNGS`, reading only the `Player`. A test plays it
in a small grid (`crates/hh-sim/tests/cli.rs`).
