# The oracle

The ladder's ceiling (`crates/hh-sim/src/agents/oracle/`): the program author steered by
rollouts on copies of the game. The other players and the rules every player follows:
[agents.md](agents.md); its spec's cost: [experiments.md](experiments.md#costs).

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
