# Shipments (module `shipments`, variant `arc1`)

Launch pads turn stockpiled goods into credits: commands load cargo onto pads, a launch burns
propellant and pays by the good's demand ([demand.md](demand.md)). Loading and launching happen
when programs run their commands ([programs.md](programs.md)); pads cool down later in the same
day ([the day](README.md#the-day)).

## Rules

**Goods.** The tradeable goods are the keys of `base_values`, listed in content resource order
(Helium-3, Titanium, Circuit Boards, Propellant at baseline). A key that is not a content
resource, or an empty map, is a config load error.

**Pads.** There is one pad per Launch Pad owned (`launch_pad`), in purchase order. Each pad
holds a good or **No cargo**, a paused flag, its cargo and a cooldown in days. Whenever
buildings change, pads are matched to the count: a new pad has No cargo, empty and cooled down, so
the player picks its good (`set_pad_good`); selling a Launch Pad drops the last pad, cargo and
all. A No-cargo pad is idle: Load skips it, it never holds cargo, and the `idle_pads` reading
counts it ([programs.md](programs.md)). A state saved before No cargo reads back with its pads' goods (the web's autosave from another rules revision is still discarded: [rulings/engine.md](../rulings/engine.md)). A pad is **active** while its
index is below the number of working Launch Pads (active, upkeep paid:
[economy.md](economy.md)): switching Launch Pads off, or leaving their upkeep unpaid, idles
the last pads, which keep their cargo.

**Loading** (Load Launch Pads). Every active, unpaused pad with a good, in loading-priority order of its
good (then pad order), takes from its good's stockpile:

`take = min(load amount, pad_cargo_capacity − cargo, stockpile)` (never below 0)

where `load amount = load_per_execution × cargo_capacity_mult`. Bought research with a
`load_per_execution` effect replaces the config value (Shipping Efficiency: 7). Pads load
in the loading-priority order of their good; pad order only breaks ties within a good, so a
short stockpile fills that good's first pads and leaves its later ones short. A partial
load is fine: the pad takes what there is.

**Can launch.** A pad can launch when it is active, its cooldown is 0, it holds some cargo,
and propellant on hand is at least `fuel_per_pad`.

**Launch Full Pads** launches, in pad order, every unpaused pad that can launch and holds a
full `pad_cargo_capacity`. Each launch burns its fuel before the next pad is checked, so
propellant can run out partway through.

**A launch:**

1. Burns `fuel_per_pad` propellant.
2. Pays `base_values[good] × demand × cargo × shipment_credit_mult` credits, at the good's
   demand for today (set earlier in the day).
3. Counts a shipment for the run, raises the run's best launch (the most credits one launch
   paid, which the `launch_credits` condition reads, [events.md](events.md)), adds the pay
   to the run's `trade_credits` (every launch's pay this run, which the `trade_credits`
   condition reads) and raises the good's
   best cargo (the most of it one launch carried this run, `best_cargo`, which the
   `launch_cargo` and `launch_full` conditions read), adds the cargo
   × `speculator_per_unit` to the speculators' Attention ([attention.md](attention.md)),
   empties the pad and sets its cooldown to `launch_cooldown`.
4. Raises the good's launch saturation and adds the payout to its burst-target revenue
   ([demand.md](demand.md)). Demand moves at the next day's update, so every launch of a good
   in one day pays the same demand.
5. Checks the shipment achievements: payout, shipments this run, and demand at launch
   ([achievements.md](achievements.md)).

**Cooldown.** At the end of each day's commands (after projects drain), every pad's cooldown
drops by 1, stopping at 0. A pad launched by a program can launch again `launch_cooldown` days
later. A manual launch between days loses no day: the next tick's commands still see the full
cooldown, so a program can launch that pad again only on the `launch_cooldown + 1`th tick.

**Loading priority** is an ordering of every tradeable good, reset to content order each run.
Load Launch Pads serves pads in this order of their goods (pad order within a good), so a
short stockpile goes to the higher-priority good's pads first; each pad still loads only its
own good. It also decides the pool Disrupt Speculators hits ([demand.md](demand.md)); it
no longer picks a new pad's good (a new pad has No cargo).

**A new run** starts with fresh pads (one per Launch Pad the run starts with, No cargo, empty,
unpaused, cooled down) and the default loading priority.

## Actions

- **`launch { pad }`**: launches that pad now. Works on a paused pad and on one that is not
  full. Illegal if there is no such pad, or it can't launch (inactive, cooling down, empty, or
  propellant below `fuel_per_pad`).
- **`set_pad_good { pad, good }`**: sets the good a pad carries; `good` null returns it to
  No cargo. Illegal if the good is not tradeable, the pad doesn't exist, or the active
  quest's `pad_goods` list doesn't name the good (1.2 First Shipment allows only Titanium;
  a pad already on another good keeps it; the view's `pad_goods` lists the goods a pad may
  be set to now, [events.md](events.md)); setting the same
  choice again does nothing. A loaded pad
  switching goods (or to No cargo) returns its cargo to that good's stockpile, up to the stockpile's cap; the
  rest is lost, and a Recent Activity line (`cargo_lost`) says how much.
- **`set_pad_paused { pad, paused }`**: pauses or resumes a pad. A paused pad keeps its good
  and cargo; Load and Launch Full Pads skip it. Illegal only for a missing pad.
- **`set_loading_priority { goods }`**: replaces the loading priority. Illegal unless the list
  holds every tradeable good exactly once.

Pad indices are 0-based. The commands Load Launch Pads and Launch Full Pads run through
programs; they need their full cost to run (Load costs energy; Launch Full Pads has no cost of
its own, the propellant is the launch's). Launch Full Pads is locked until quest 1.4
Processors is completed (`requires: quest_completed`, [commands.md](commands.md)); before
then a pad launches by hand (`launch`, from its card), which 1.2 First Shipment (one launch
carrying 20 titanium) and 1.3 Full Load (a full pad of He-3) teach.

## Params

<!-- params:shipments -->
| Param | Baseline | Range | Meaning |
|---|---|---|---|
| `base_values` | circuits 15, he3 10, propellant 4, titanium 6 | 0 to 1000000 | The tradeable goods and each one's credits per unit at demand 1. |
| `pad_cargo_capacity` | 100 | 1 to 1000000 | Cargo a pad holds when full. |
| `fuel_per_pad` | 20 | 0 to 1000000 | Propellant burned per launch. |
| `load_per_execution` | 5 | 0 to 1000000 | Units Load Launch Pads puts on each pad, before research. |
| `launch_cooldown` | 10 | 0 to 100000 | Days after a launch before the pad can launch again. |
<!-- /params -->

Modifiers read ([README.md](README.md)): `cargo_capacity_mult` (Bulk Shipper achievement),
`shipment_credit_mult` (First Profit achievement; side quest 1.a Big Launch's ×1.1, live
while it is completed).

## Content

- `content/buildings.json`: `launch_pad` (one pad each). `content/commands.json`: `load_pads`
  (effect `load_pads`), `launch_pads` (effect `launch_full_pads`).
- `content/research.json`: rows with a `load_per_execution` effect (`shipping_efficiency`).
- Fixed ids in code (`shipments.rs` constants): building `launch_pad` (one pad per copy)
  and resource `propellant` (the fuel a launch burns).

## Port notes

- Conflict: load per execution was 5 in both the command effect and `game_config`; the engine
  keeps one home, the config's `load_per_execution`.
- Candidate: the 10-day launch cooldown is the handoff's number; the data has none.
- Candidate: a manual launch needs only some cargo (not a full pad) and fuel, and works on a
  paused pad.
- Ruling (revision 4): a loaded pad may switch goods; its cargo returns to the stockpile up
  to the cap and the rest is lost.
- Ruling (Matt, 2026-11-23; revision 22): players must learn to pick a pad's cargo, so a new
  pad, and every pad a run starts with, has No cargo; quest 1.2 teaches it.
- Candidate: `shipment_completed` events and the shipments-this-run achievement count this
  run's shipments.
