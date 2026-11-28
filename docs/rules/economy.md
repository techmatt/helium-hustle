# Economy (module `economy`, variant `arc1`)

Resources, storage caps, buildings (buy, sell, set active), land, the daily building
production and the end-of-day clamp. In [the day](README.md#the-day), demand is set first;
then upkeep-free buildings produce, programs run, and the buildings that take upkeep resolve
on what is left; the clamp runs late in the day, after shipments and bursts. Prices and amounts are `Fx`; counts
(owned, active, land bought) are integers.

## Rules

**Resources.** Every content resource is held from the start of a run (0 unless
`starting_resources` says otherwise). Credits, science, land and processors have no cap.
A resource is **shown** (the view's `visible`; display only) if its content says always,
a building making it is owned or was in any run, a command making it was ever used (Buy
Propellant shows Propellant), or some is held now.

**Caps.** A capped resource's cap is its content `cap` plus the `store` effects of every
*working* building on it (per copy; see *Working copies*), times the modifier `storage_cap_mult`; energy's cap skips
`storage_cap_mult`. Boredom's cap is the boredom module's (`boredom.base_cap` plus working
Recreation Domes' `store`; see boredom.md). Caps bind only in the clamp: production during the
day may run past a cap.

**Working copies.** A building's effects (`grant`, `store`, a Launch Pad's pad,
`speculator_decay`) come from its *working* copies: the active ones whose upkeep the last
building pass paid. A type run at `frac` in the partial pass leaves `active −
floor(active × frac)` copies unpaid until a later day's pass pays them, so a Data Center short
of energy costs its processor from the next day (production is scaled by `frac` the same day).

**Processors** are derived: after any building change (buy, sell, set active, a new run, a
pass that changes how many are unpaid) the processors resource is set to the sum of working
buildings' `grant` effects (1 per working Data Center); assignments are bounded by the active
ones' grants, paid or not (programs.md). Launch Pads owned and pads are matched
(shipments.md).

**Price.** The next building of a type costs, per resource in its `cost`:
`cost × cost_scaling ^ (owned − bonus) ÷ m`, the last factor only for a building tagged
with an `ideology` axis (`m`, that axis's modifier, ideology.md: Recreation Dome, Launch Pad
and Arbitrage Engine Nationalist; Data Center and Battery Humanist; Research Lab and
Fabricator Rationalist), then rounded to whole units, halves up, tagged or not (Matt, 131 and
2026-11-27: what's shown is what's charged). Land is not part of the price, so `m` never touches it. `bonus` counts free
buildings granted at run start, so they don't raise the price. Whole powers use `Fx::pow_int`
(see [the rounding rule](README.md#numbers)). A price past `Fx`'s range is shown as the maximum and is
never affordable.

**Buy** checks, in order: unlocked (its `requires` holds and, if an event enables it, that
event enabled it this run), under `max_count`, enough land (its `land`), and every price
resource on hand. It then pays the price and the land, and the new building is owned and
active. The price paid is added to the run's `paid` book for the type (per resource), and a
building with a `nudge` adds it to its tagged axis (the Recreation Dome: 2.5 Nationalist,
ideology.md). Starting buildings and run-start bonus buildings take no land.

A `requires` (on a building or command, [commands.md](commands.md)) is one of: `none`;
`building` (at least `count` owned); `quest` (the quest active now or completed in any run);
`quest_completed` (the quest completed in any run); `research` (bought this run); `flag`
(a career flag or a live project `flag` reward's); `project` (completed, [projects.md](projects.md)).

**Visible:** unlocked, or (for a building-count requirement only, and not event-gated) owned
in any run of the career.

**Sell.** A copy that took land (a bought one) goes first and refunds its `land` plus, per
resource in the type's `paid` book, `sell_refund_share` (0.5) × the average price paid
(`total ÷ bought copies owned`), lowering the total by that average: half what the copies
cost, never today's price, so a rank swing can't be farmed (not shown in the UI). Once only free copies remain (starting and run-start bonus buildings,
counted as `free`), one goes with no land back. Active is lowered only if it would exceed
owned (an idle copy goes first); bonus is lowered the same way, so selling a bought copy
lowers the next price. Starting buildings count toward the price (they are not `bonus`).

**Land.** The next land purchase costs
`land_base_cost × land_cost_scaling ^ land_bought × land_cost_mult × R`
credits and gives `land_per_purchase` land, where `R` is `rival_land_price_mult` while the
rivals are present, else 1 ([attention.md](attention.md)). Each purchase raises the rivals'
Attention by the land gained × `rival_per_land`, once Act 2 begins (`swarm_revealed`). `land_bought` counts this run's purchases.

**Daily resolution**, over building types in content order, each type's active copies
together, in two stages around the programs ([the day](README.md#the-day)): first the
types that take no upkeep (`produce_free`: Solar Panels), then programs run and pay, then
the types that take upkeep (`resolve`) on what is left. So programs are paid before any
building's upkeep, from the stock and the upkeep-free production; energy an upkeep
building makes (Fuel Cell) reaches programs the next day, through the stock. Each stage:

1. Each type's production is `produces × active × output factor`, where the output factor is
   the overclock factor for its `category` (commands.md) times each of its
   `output_modifiers`. Its upkeep is `upkeep × active × building_upkeep_mult`.
2. A type whose every product is at or over its cap is marked `output_capped` for the day; it
   still pays and produces.
3. **Full passes:** each pending type whose every upkeep resource has at least
   `need − upkeep_epsilon` pays its upkeep and adds its production. Passes repeat while one
   succeeds and any remain, so a type later in content order can feed an earlier one.
4. **Partial pass:** each remaining type, in order, runs at `frac` = the least of
   `max(held, 0) / need` over its upkeep (at most 1), paying and producing that fraction, and
   is marked `input_starved` (also when `frac` is 0 and it does nothing) unless an earlier
   partial type made what it lacked and it ran in full.

Energy produced by buildings in the day (gross) updates the run's peak energy (the career's
`peak_power` takes it at retirement, [career.md](career.md)).

**Clamp** (end of the day): every capped resource over its cap is cut to it, the excess
recorded as the day's overflow; every resource below zero (left by `upkeep_epsilon`) is
raised to zero.

**Breakdown** (display only, no rule reads it): every change to a resource in play (not
cheats, run setup, derived processors or the clamp's lift of a rounding dip) is recorded
under its source for the day (`today.flows`: each building type, each program, pad
loading and switch returns, launches, building and land purchases, sales, research,
projects, boredom growth, `command:<id>` boredom, `building:<id>` relief, and `cap`, the clamp's cut). Player actions after a
tick count in that day. Who ran short is recorded too (`today.shorts`): a building type
in the partial pass, or a program line short of a cost (it failed, fraction 0): the scarcest
input, the need, what there was and the fraction run. At each tick's
start the day folds into a rolling average, `avg += (day − avg) / k`, `k` the days folded
this run up to `flow_average_days` (`Run.flow_days`, counted from the first fold with a
flow: actions before the first tick are one): a source new to it enters at zero and one
absent from the day counts zero, so it is a true mean over the run's first `k` days (a
one-off 200 on day 30 reads 10); a step too small to move it lands on the day's value, so a quiet source
reaches zero and drops out. The view's `breakdown` lists both per resource, sources first.

**New run:** resources from `starting_resources`, buildings from `starting_buildings` (owned
and active, no land taken), then run-start bonus buildings from projects and achievements
([career.md](career.md)).

## Actions

| Action | Refused when |
|---|---|
| `buy {building}` | unknown id, locked, at `max_count`, short of land, or can't afford the price |
| `sell {building}` | unknown id, or none owned |
| `set_active {building, active}` | unknown id, or `active` over the number owned |
| `buy_land` | credits short of the price, or the price is past `Fx`'s range |

A refused action changes nothing.

## Params

<!-- params:economy -->
| Param | Baseline | Range | Meaning |
|---|---|---|---|
| `starting_resources` | energy 100, land 30 | 0 to 1000000000 | Resources at the start of every run, by id. |
| `starting_buildings` | data_center 1, solar_panel 1 | 0 to 1000 | Buildings owned (and active) at the start of every run. |
| `land_base_cost` | 200 | 0 to 1000000000 | Credits for the first land purchase. |
| `land_cost_scaling` | 1.5 | 1 to 10 | Each land purchase multiplies the next one's price by this. |
| `land_per_purchase` | 15 | 0 to 1000000 | Land gained per purchase. |
| `upkeep_epsilon` | 0.001 | 0 to 1 | A building pays full upkeep if short by no more than this. |
| `flow_average_days` | 20 | 1 to 1000 | Days in the resource breakdown's rolling average (display only). |
| `sell_refund_share` | 0.5 | 0 to 1 | Share of a type's average price paid that selling a bought copy refunds. |
| `days_per_year` | 365 | 1 to 100000 | Days in a player-facing year (dates; boredom's yearly escalation). |
<!-- /params -->

Dates show as years and days: day count `d` reads "Year ⌊d / days_per_year⌋ + 1, day
d mod days_per_year + 1" (day 0 is Year 1, day 1). State stores days.

## Content

`content/resources.json`: each resource's `cap` (null for none). `content/buildings.json`:
`category`, `ideology`, `requires`, `land`, `cost`, `cost_scaling`, `max_count`, `produces`,
`upkeep`, `effects` (`grant`, `store`, `speculator_decay` for demand.md, `boredom_relief`
for boredom.md), `nudge` (ideology.md) and
`output_modifiers`, and a `description` no rule reads: empty (the card shows no line) unless
the building has a mechanic its stat lines don't show (now the Arbitrage Engine and the
Microwave Receiver). A run opens with only the Solar Panel and Data Center buildable: the
Regolith Excavator, Smelter, Launch Pad and Storage Depot require research Basic Blueprints,
and the Battery, Refinery, Fabricator and Research Lab Advanced Blueprints ([research.md](research.md);
quest 1.2 First Shipment needs the first, and its completion shows the second). The Recreation
Dome requires research Recreation Dome Blueprints, shown from the first retirement
([research.md](research.md)). The Regolith Excavator makes 2 regolith a day
(`excavator_output_mult`, 1.5 with research Regolith Processing). The Refinery's upkeep is
5 regolith and 2 energy a day. A Solar Panel makes 6 energy a day; a Data Center's upkeep is
4 energy a day. A Launch Pad costs 150 credits and 20 titanium.

Six resource ids are fixed in code (`economy.rs` constants), named by content rows:
`energy` (its cap skips `storage_cap_mult`; its output counts toward peak power),
`credits` (prices, payouts, credits earned), `land`, `processors` (assigned, not stocked),
`boredom` (its cap is the boredom module's) and `science` (research).

## Port notes

- Conflict: land purchase. Handoffs: 15 credits, x1.5, 10 land; data: 200 credits, x1.5,
  15 land. The engine takes the data.
- Conflict: starting land. Data starts a run with 30 land, a Solar Panel and a Data Center;
  the 30 is free land, and starting buildings take none.
- Conflict: Microwave Receiver. The building requires flag `microwave_power_active` but the
  project sets `microwave_power_completed`; the engine gates it (and Fuel Cell Array) on its
  project's completion, `requires: {"type": "project"}`, so it follows the project's scope
  ([projects.md](projects.md)). Handoffs say it produces energy; the data
  has no production (it enables Buy Power, commands.md). The engine takes the data.
- Conflict: Arbitrage Engine's `spec_decay`: see [demand.md](demand.md#port-notes).
- Changed: the data's Arbitrage Engine requires research `market_analysis` (not a research id);
  the engine requires research Market Awareness (Matt, 2026-11-26), so it isn't buildable the
  moment speculators arrive.
- Candidate: starting buildings take no land and selling one refunds none; they still count
  toward the price (the second Solar Panel costs 30 credits, as in Godot's opening).
- Candidate: peak power is gross energy buildings made in a day.
- Candidate: a new Data Center's processor starts unassigned; losing processors unassigns
  from the last program first.
- Candidate: cost curves use exact whole powers (`Fx::pow_int`), not `libm::pow` then floor
  (1.15² = 1.3225, not 1.322499); see [the rounding rule](README.md#numbers).
