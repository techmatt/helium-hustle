# Rules

The rules as implemented, rules revision **33** (`RULES_REVISION` in `crates/hh-core/src/lib.rs`).
History: [../rules-history.md](../rules-history.md).

The game is the Godot build's **Arc 1** ported into `hh-core` (revision 2), with the
boredom cap the Godot build was played at (revision 3): numbers from
`reference/data/*.json` (as content rows in `content/` and params in
`configs/baseline.json`), prose rules from `reference/docs/handoff_*.md`. Where the two
disagreed the data won, unless a ruling said otherwise; each module page's *Port notes*
lists its conflicts and the candidates the port had to invent (the item's write-up:
[port_notes.md](../../experiments/results/004_rules_revision1/port_notes.md)).

You are an AI running helium-3 mining on the Moon. Buildings extract and process
resources, all competing for energy; programs of commands run on processors; launch pads
ship goods to Earth for credits at a demand that moves; boredom rises until it forces
retirement, and the career keeps what you learned for the next run.

## Modules

Every module plays; each is one page. A config must name all of them. Each page's *Params*
table is generated from `REGISTRY` and the baseline (`hh-sim params --write`; a test fails when stale).

| Module | Variant | Page | What |
|---|---|---|---|
| `boredom` | `phased` | [boredom.md](boredom.md) | Growth by phase of the run's day, the cap, forced retirement. |
| `economy` | `arc1` | [economy.md](economy.md) | Resources, caps, buildings and their daily resolution, land. |
| `demand` | `arc1` | [demand.md](demand.md) | Demand per tradeable good: noise, speculators, rivals, saturation, Promote. |
| `attention` | `minimal` | [attention.md](attention.md) | **Draft** Each adversary's Attention: what draws the speculators and rivals in. |
| `programs` | `arc1` | [programs.md](programs.md), [commands.md](commands.md) | Programs, processors, instructions (`if` blocks, `build`); running commands, overclocks (commands.md). |
| `dream` | `flat` | [dream.md](dream.md) | What Dream does to boredom. |
| `projects` | `drain` | [projects.md](projects.md) | Projects scoped to a run, a timeline or forever, funded over time. |
| `shipments` | `arc1` | [shipments.md](shipments.md) | Launch pads, loading, launching, payouts. |
| `research` | `arc1` | [research.md](research.md) | Science-bought upgrades, gated by visibility. |
| `ideology` | `geometric` | [ideology.md](ideology.md) | Three axes, Fund commands, ranks and per-rank bonuses. |
| `events` | `arc1` | [events.md](events.md) | The quest chain, side quests and one-off events, with their unlocks. |
| `achievements` | `arc1` | [achievements.md](achievements.md) | Six achievements with permanent rewards. |
| `career` | `arc1` | [career.md](career.md) | Runs, retirement and its gap, what persists, career bonuses. |
| `timeline` | `draft` | [timeline.md](timeline.md) | **Draft** Acts 2 and 3: calendar, swarm, Earth techs, hard resets, vault, super timeline, ending ([../arc.md](../arc.md)). |

## The day

One tick is one in-game day. Each tick, in this order (`Rules::tick`, `rules.rs`):

1. Yesterday's flows fold into the breakdown's rolling average and the day's record
   clears (economy; display only).
2. **Boredom** grows by the phase rate (boredom); then the debug **cheat building**, if
   bought, makes and relieves (*Cheats*).
3. **Overclocks** past their last day expire (programs).
4. **Attention** decays (attention), then **demand** decays its forces and is set for the
   day (demand).
5. **Upkeep-free buildings** (Solar Panels) produce (economy).
6. **Programs** run, paid first: each processor one step (programs, with shipments,
   demand, ideology and dream for their commands' effects, economy for `build`).
7. **Buildings that take upkeep** resolve on what is left: full passes, then partial
   production (economy).
8. **Projects** drain their funding (projects).
9. **Pads** cool down (shipments).
10. **Speculator bursts and rival dumps** when due, landing only while their adversary is
    present (demand).
11. The day's **rate** per resource is kept for the Resources panel's per-day number (programs),
    then the **clamp**: every resource cut to its cap (overflow counted) and lifted to zero
    (economy).
12. **Quests, side quests and events** check their conditions (events).
13. **Achievements** check the day's flows and stockpiles (achievements).
14. **Ideology** bests are recorded in the career (ideology).
15. The run's **day** count rises.
16. Boredom at its cap **forces retirement**; otherwise a program's `retire` line that ran today
    retires the run voluntarily. The next run starts at once (career).
17. **Timeline** (draft, after the reveal): Earth science goes to Earth, the countdown
    rises and the swarm arrives on its day (timeline).
18. The **activity feed** is cut to its newest `activity_kept` lines (events; display only).
    The same trim ends every applied action.

Player actions apply between ticks; an illegal one changes nothing. Actions are listed on
the page of the module that takes them (`crates/hh-core/src/action.rs` has them all).

## Content

`content/*.json`, embedded in `hh-core`: resources (16), buildings (20), commands (23),
research (17), events (34: nineteen the quest chain, four side quests), projects (15),
achievements (6), rivals (4), Earth techs (6, `earth_tech.json`), and two display-only
files: the UI's nav tiles (`nav.json`, 18: when each panel's tile shows) and the acts
(`acts.json`, 4: each act's name and the chain quest it opens with, [events.md](events.md)). Act 1's beats are draft from
1.5 on (1.1 to 1.4 open one mechanic at a time from a base of Solar Panels and Data Centers), Acts 2 to 4 unaligned scaffolding ([../arc.md](../arc.md)). Ids are unique across all files but `nav.json`, whose ids are panel names.
`messages.json` holds UI prose the view carries (display only; its ids are fixed in code,
`content::MESSAGES`): the run summary's line, `forced_retirement` or `retired`.
Buildings resolve in file order; quests chain in file order. `content/prose.md` is not
content: it is every player-facing string, generated by `scripts/prose.py` for editing and
imported back by id.

A few content ids are fixed in code, each listed in its page's *Content*: resources
([economy.md](economy.md), [shipments.md](shipments.md)), `launch_pad`, the `retirement`
panel ([career.md](career.md)), the `research` panel ([research.md](research.md)), the
`logic_if` flag ([programs.md](programs.md)), the `nationalist_growth_mult` key's axis
([ideology.md](ideology.md)), the adversaries
([attention.md](attention.md)), the ideology axes ([ideology.md](ideology.md)) and the
modifier keys (`content::MODIFIER_KEYS`, [Modifiers](#modifiers)).

The port renamed the Godot data's short ids: `eng` energy, `reg` regolith, `ti` titanium,
`cir` circuits, `prop` propellant, `cred` credits, `sci` science, `proc` processors, `panel`
solar_panel; commands `promote_ti`/`_cir`/`_prop` to `promote_titanium`/`_circuits`/
`_propellant`, `disrupt_spec` to `disrupt_speculators`. Requirements name quests by their
event ids (`qpower_boot_sequence`, not `QPower`). Narrative text (names, descriptions,
event bodies) is the Godot data's placeholders, but where a text was made to say what the
game does (bug hunts, `docs/qa-findings.md`).

## Numbers

Every quantity is `Fx` (10^-6, floor rounding: [architecture](../architecture.md#numbers-fx));
counts (buildings, days, ticks, ranks, pads) are integers.

**Whole powers are exact.** `Fx::pow_int` builds `x^n` by repeated multiplication at 10^-18
in `i128` and floors once to 10^-6, so 1.15^2 is 1.3225, not the float floor 1.322499. It
serves every whole power in play: building cost curves (x1.5, x1.4), land prices, ideology
rank scores, per-rank bonuses and discounts (1.05^r, 0.97^r, 1.03^r) and speculator burst
growth. A negative power is `1 / x^|n|`, floored once. Only fractional powers use floats,
through `libm` (`crate::math`): boredom resilience (0.995^(days/400)) and the retirement
gap's daily ideology decay (0.5^(1/365)). This matters: 0.97^r and 1.05^r aren't exact as `f64`, so the
float floor would land discounts and bonuses a step low, and a cost curve shouldn't hang on
float rounding.

## Modifiers

A modifier key's value is the product of every live source naming it, 1 with none
(`modifiers.rs`). It is derived on demand from the career and the run, never stored.

| Key | Source | Applies to |
|---|---|---|
| `extractor_output_mult` | Deep Core Survey (Strategic project; unreachable, [projects.md](projects.md#port-notes)) | Regolith Excavator and Ice Extractor output |
| `excavator_output_mult` | Strip Mining (achievement) | Regolith Excavator output |
| `solar_output_mult` | Grid Recalibration (Strategic) | Solar Panel output |
| `building_upkeep_mult` | Predictive Maintenance (Strategic) | every building's upkeep |
| `promote_effectiveness_mult` | Market Cornering (Strategic) | Promote's demand effect |
| `speculator_burst_interval_mult` | Speculator Dossier (Strategic) | days between speculator bursts |
| `land_cost_mult` | Lunar Cartography (global; unreachable) | land price |
| `storage_cap_mult` | Silicon Valley (achievement) | every cap but energy's (and boredom's) |
| `shipment_credit_mult` | First Profit (achievement), 1.a Big Launch (side quest) | shipment payouts |
| `cargo_capacity_mult` | Bulk Shipper (achievement) | cargo loaded per Load |
| `demand_ceiling` | Market Timer (achievement) | the demand ceiling |
| `buy_power_mult` | career peak power ([career.md](career.md)) | Buy Power's cost and output |
| `boredom_growth_mult` | Boredom Dampers (Earth tech, draft) | boredom growth |
| `ideology_cross_mult` | Consensus Engine (Earth tech, draft) | Fund's cross-penalty on the other axes |
| `science_output_mult` | Deep Space Telescope (global) | Research Lab output |
| `nationalist_growth_mult` | Deep Space Telescope (global) | a Fund Nationalist's push to its own axis |
| `ideology_half_life_mult` | 1.d Three Voices (side quest) | the retirement gap's ideology half-life ([ideology.md](ideology.md)) |

A building lists the keys that multiply its output (`output_modifiers`); a command its
`scale_modifier`. Achievement and `forever` project rewards apply from the moment they are
earned and in every later run, across timelines; global (`timeline`) ones for the rest of
the timeline; Strategic (`run`) ones for the rest of their run.

## Not built

Dream Echo (Dream is the flat variant), block/skip toggles, the
Stats panel's lifetime source totals (the last day's and a rolling average's per-source
breakdown is built: economy), and playtest telemetry. The view reports what the UI needs for progressive
disclosure (`visible` flags, nav tiles); the "new" markers are the UI's own, not state
([web/README.md](../../web/README.md)).

## Cheats

The web debug menu's cheats are actions (`Action::Cheat`, `cheat.rs`), so a cheated game
replays exactly: **grant** adds to a resource up to its cap (not processors or boredom),
**set boredom** sets it up to its cap (at the cap the run retires on the next tick), and
**jump ahead** marks a jump whose days then pass as usual, **complete research** finishes
every research item not done this run (free; known to the career as if bought, so later
runs see it), and the **cheat building** (1 credit, `Run`'s `cheat_buildings`, absent while
0, gone at retirement) adds per copy, unboosted, 1000 credits a day and 10 a day to every
other resource the Resources panel shows (but land, processors and boredom) and takes 10
a day off boredom, down to 0 (`cheat::tick`, after boredom grows). Its numbers are debug constants in
`cheat.rs`, not config; the UI offers it only while Cheats are on. Any cheat sets the
state's `cheated` mark for good (absent from an honest game's JSON; the UI shows it only
in the debug info, not the bottom bar); no rule reads it.

Two cheats are **accelerators**, debug-only, for zooming a career through the arc
([simulator.md](../simulator.md#accelerators)). **Boost** sets the career's factors (1 is
off; state `boost`, absent until set): `production` multiplies every resource made but
science (buildings, commands, launch payouts: one place, `GameState::produce`), `research`
science made, `boredom` boredom's growth (the boredom multiplier). **Autopilot** marks
where a sim player took over toward a target; its actions follow as any others. Either
also sets `accelerated` (absent while false), which `hh-sim vs-human` refuses; so do the
cheat building and complete research. No honest
game, the golden, a sim agent or a report uses a cheat, so none moves `RULES_REVISION`.
