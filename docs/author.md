# The program author (`author.rs`)

The `author` rung's play, the medium tier ([agents.md](agents.md): the roster, the rules
every player follows, the builder it scores with, when it looks). Split from agents.md
(133) at the docs size margin.


Its choices (target, research, projects, when to look) are in `author.rs`, what the oracle's
and the committed players' goals change in `author/directives.rs`; every program text
it writes, saved or not, is built in `author/templates.rs`. The rung that matters most: program authoring is the core skill. Its target is the best-scored
building whose missing inputs can be made or bought; one short of an input nothing makes and no
command buys (circuits) works toward that input's maker, or the building that unlocks the maker
(`requires`). A resolved target still past a cap (a ninth Storage Depot) gives way to the
building itself if under its caps, else is skipped (a maker step past a cap is skipped). It
buys others only from credits past the target's price.
How it writes the main program depends on the gap to its next look ([agents.md](agents.md#when-it-looks)):

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
in a fixed order (`author.rs`), the Blueprints first, saving science for the first visible
item not bought (`next_research`): an unaffordable head starves all later research; while short of the next one on sale,
its main program opens with Buy Science × as many as the credits past the target's price
cover. **Projects:** the cheapest open `run` project, beside the
first open other one in a fixed order; each only when every cost can move (on hand or
made); science at the project's most a day when no research waits, else `science_share`
percent of it; credits a tenth
of those past twice the target's price plus 200 a day, anything else `project_rate`, all under
the funding caps (`configs/baseline.json` `projects`: 30 credits, 10 others). It retires
once the run has bought nothing for `stale_days` days.
