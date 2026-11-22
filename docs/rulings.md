# Standing rulings

Matt's standing decisions about the game and the project, as they hold now. A ruling marked
**locked** changes only by Matt's decision; one marked **default** is an executor's pick that
stands until Matt overrides it. Engine and docs rules that bind every item are in
[CLAUDE.md](../CLAUDE.md); this page is the why behind several of them and the calls that are
not code rules. A new ruling, or a change to one, updates its line here in place.

## Game

- **Pillars (locked).** Plans beat waiting: the game is solved by clever plans, manual early and
  programs later at high speed ("unlimited speed" is about 200x for now). Programs are the core
  identity. Boredom is a speed governor. No loss: stages are wins, and a finale is far off. No
  offline time: the game advances only while it runs in the browser. Numbers grow naturally.
- **Be intentional about what can be gamed (2026-11-21).** A solution that sounds "too
  boring" (idling, waiting it out, repeating the last run) must never work well. Every gate and
  quest condition states which boring strategies it rules out.
- **Speed.** Decisions happen at 1-3x with brief bursts to 5-20x. 200x comes later, once
  programs can automate more, behind tech not yet in the game.
- **Run-1 real time.** 1x stays 1 tick a second; the 25-35 real-minute target for run 1 is what
  moves ([principles.md](principles.md)).
- **Tuning.** Every number is open. Matt plays the UI build and decides where fine-tuning
  starts. A better main story quest is designed later.
- **Placeholder content.** Many thresholds, texts and numbers are placeholders. Don't spend
  items adjusting them (reachability, wording) until content is designed for real; bug hunts
  and QA may list them, not fix them.
- **Program loadouts (2026-11-20).** Each program has save / load buttons. A saved program
  goes into a library that persists across retirements (career state), so a player keeps
  programs for different stages and loads one into any program tab. Programs still empty on
  retirement; loading is how they come back. A loaded line not unlocked yet in this run fails
  and advances. *Defaults (035):* 12 slots, display names up to 40 characters, open from
  run 1, loading leaves processors as assigned.
- **One flat program template (2026-11-20).** Every line is `<verb> <target> [amount]`: the
  target slot is typed by the verb (building, resource, good, ideology axis, research item,
  saved program, program tab), the amount `N`, `up to N` or `all`. No nesting, variables,
  labels or jumps; a new capability is a new verb or reading, never new grammar; per-target
  commands (Buy X, Promote X, Fund X, Overclock X) are one verb with a target. Conditions stay
  `<reading> <op> <right side>` with one optional `and` / `or`; the right side is a number or
  another reading times a number (one multiplier). Pads are addressed only by good. Every verb
  costs a processor step. Unlocking is out of scope: everything is open wherever program logic
  is. *Defaults (037):* the pad verbs run the Launch / Load commands' rows; `retire` retires at
  the day's end like the button; `every N days` reads 1.
- **Every non-`if` line costs a step (2026-11-20, replaces 037's "done and free").** A line
  with nothing to do (`research X` once researched, `pause` with nothing running, `set pads`
  already at N, `launch pads` with no pad ready, `build X up to N` with N owned) still takes a
  processor step, and nothing else: no resources, no boredom. A false `if` still skips its
  whole block for free, conditions stay free, `wait until` is unchanged. Guards are how a
  player saves steps: wrap a finished line in an `if`. *Defaults (038):* guard readings
  `researched <item>`, `pads <good | all>` (pads carrying it), `running <building>` and
  `processors <tab>`; `load pads` is done when no pad of the good would take cargo.
- **`days since last purchase` and `fund <project>` (2026-11-20).** A reading for days since
  the last purchase, and a `fund <project> [amount]` verb funding a project the way the
  Projects panel does; both may be switched off later, each one config change
  (`days_since_purchase_reading`, `fund_verb`). *Defaults (039):* a purchase is a building,
  land or research item, by hand or program (not commands, funding or free buildings); `fund
  P N` sets N a day on every cost of P (each at most its resource's most a day, 046; `all` or
  no amount the most; 0 stops it); the switches are 0/1 programs params; JSON verb `fund`, so
  a later `fund <axis>` can share it.
- **Project scopes (2026-11-21).** A project's `scope` is `run`, `timeline` or `forever`
  (replacing `tier`: Long-Term is `timeline`, Strategic `run`). A `forever` project's
  progress survives a timeline's end as well as its completions. A building gated on a
  project's reward follows that project's scope: it requires the project's completion, not a
  career flag that outlives it. Every repeatable project carries `max_completions` (typically
  5). *Defaults (046):* a repeatable row's rewards live once however
  many completions, and it keeps its rates across completions; every row's `min_days` is its
  largest cost over 30 rounded up (the old flat cap's fastest finish); the `completed
  <project>` reading is the completion count.
- **Bonus stacking (2026-11-21).** Bonuses from one source add (+5% ×10 is +50%); separate
  sources multiply. Exponential stacking only for a few named exceptions (Paradigms, late
  techs, the catastrophic-success tech); repeatable projects are rare exceptions.
- **Folded guard readings and a golden program (2026-11-20).** Exact guards come from folded
  readings, not more `and` clauses; the golden game's scripted policy runs a short condition
  program, so the golden pins the interpreter. *Defaults (040):* `free processors` is granted
  by active buildings less assigned (what `assign processors ... all` would add); `idle pads
  <good | all>` is pads with no cargo (a cooling pad counts), so `set pads X` has one to switch
  exactly when `idle pads (all) > idle pads X × 1`.
- **`rate` reads production less demand (2026-11-20).** What buildings and programs wanted
  counts, met or not, so `Energy rate < 0` stays true all through a brownout; a capped
  resource's rate is its uncapped rate. The condition's rate and the Resources panel's
  per-day number are one quantity (the panel's for the next day). *Defaults (035):* unmet
  demand counts only on an input that ran short; the rate counts everything the day's tick
  did to the stock (purchases by program lines, pad loading, boredom growth and Dream too).
- **Rules port picks (004, defaults).** The starting-credits bonus is 0.02 of the best run
  (`career.credits_bonus_fraction`), not the Godot build's /100. Ideology ranks are geometric:
  rank n costs 100 × 1.5^(n−1) more score, so rank 5 is at 1,318.75 (`rank_base_score`, `rank_growth`), not the data's `rank_thresholds`. Lunar
  Cartography and Deep Core Survey stay unreachable until the narrative pass. The boredom cap is
  1000 plus 100 per Recreation Dome. Invented numbers (demand base, noise, coupling, cooldowns)
  are config candidates.

## Engine and numbers

- **Architecture (locked).** `hh-core` holds every rule: no I/O, clock or global RNG, serde
  state, exact replay, native = wasm, a rules revision and a pinned golden game. `hh-sim`,
  `hh-wasm` and `web/` sit on top. `web/` is Vite + TypeScript; libraries and frameworks are
  chosen by fit, with no constraint (Preact since 006). Configs hold every number, with no
  defaults in code.
- **Numbers (locked).** State is fixed point, `Fx(i64)` at 10^-6, with `i128` intermediates and
  one rounding rule (floor; overflow panics). Floats only for factors, through `libm`; platform
  `f64`/`f32` transcendentals are banned in `hh-core`. `Fx` stays opaque so its width can
  change. Counts (buildings owned, ticks) are integers, not `Fx`. Whole powers use exact
  `Fx::pow_int`; `libm` only for fractional powers.
- **Agent code is player code (022, default).** Sim agents' numbers (knob defaults, weights,
  caps) are strategy, not rules, so CLAUDE.md's "no defaults in code" does not cover them; a
  number that mirrors a config or content value says so in a comment.

## Saves

- **Saves (locked).** A localStorage autosave tagged with the rules revision and config hash,
  discarded on a mismatch. Named scenario starts in the debug menu double as
  sim starting states.
- **Breaking changes.** No saves are live, so a rules change needs no caution: err on the side
  of breaking things, keep save handling minimal, and add no compatibility code for old saves
  until the game is more stable.
- **Save migration (2026-11-21).** An easy migration is fine when it helps development;
  otherwise avoid it (technical debt). It matters once the game is released.

## Simulation and measurement

- **Sim players.** The default sim player is independent of game speed: it acts whenever a
  decision is useful and skips ahead through stretches that need none. Attention and speed are a
  separate, occasional human model that maps ticks to real minutes ([agents.md](agents.md)).
- **Hands-off measure (036, default).** "Runs are smooth" (Act 1's exit) is read by the
  `handsoff` player: run 1 the author's, then only loads at each reset and processors. Its
  measure is its credits a run and milestones from run 2 on as a share of the author's on the
  same seeds (the ladder's *Hands-off* table, [experiments.md](experiments.md#measures)).
- **View (009, default).** `per_day` and `next_run` keep playing a day or a retirement on a
  copy (exact; they show the next day, not the last). Sim players keep the full view.
- **Oracle (013, defaults).** The oracle lives in `experiments/ceiling.json` (~20 min, the
  author beside it as a reference row); `hh-sim ladder` stays fast and `vs-human` excludes the
  oracle; its strategies are a fixed table; it takes `--milestones`. Its `Looks` and
  `milestones::Tracker` judge history milestones differently: the next oracle item moves it to
  `Tracker` semantics and re-pins.
- **Warning checks.** On: native = wasm parity, tunables in code, platform floats in `hh-core`,
  docs over 15 KB, balance (`hh-sim balance`, judged on the author and casual players), and by
  hand a rules change without its history row or golden move (CLAUDE.md, *Warning checks*).

## Art and UI

- **Art direction (2026-11-15): B "Earthrise".** Warm grey regolith surfaces, an amber accent,
  Earth in the sky over the work area, a lunar horizon with the habitat at the foot of the left
  column. The layout is kept. Resource colors are never hidden. A **light theme** for debugging
  stays, 1:1 with dark: same layout, markup and art, only color tokens swap. "Flash"/3D (a live
  view of the base growing) comes later; plan a home for it, build later.
- **Building art (2026-11-16).** Cards use painterly cut-outs at each building's natural 3/4
  view (019's `cut34`: no fixed camera, a small irregular regolith patch, transparent
  background), as a 96 px thumbnail cropped tight on the building, under the title beside the
  description and flows. A later "base grows" view uses the crisp painterly isometric set
  (019's `crisp`: fixed 2:1 camera and patch guide, one sun). The icon styles of icon_style_r1
  were all rejected.
- **Icons (025-027).** One icon module, `web/src/ui/icons.tsx`, over
  `web/src/assets/{chips,nav,horizon}/`, so a new style swaps files only. Painterly nav icons at
  normal size (compact tiles use chips or glyphs); the earthlight horizon; chip icons (color
  square, knocked-out glyph, name as tooltip) for every resource, ideology axis and rival,
  everywhere they appear. Rival colors: ARIA-7 `#ec407a`, CRUCIBLE `#ff6f00`, NODAL `#006064`,
  FRINGE-9 `#9e9d24`.
- **Chip colors (028, default).** Resource, axis and rival chip colors stay TS maps in
  `icons.tsx` (the chip needs the hex for its ink-contrast choice, and they are the same in
  both themes); only the axis text colors are CSS tokens.
