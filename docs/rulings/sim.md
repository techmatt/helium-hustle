# Rulings: simulation, measurement and tooling

Part of Matt's standing rulings ([index](../rulings.md): what **locked** and **default** mean).

- **Sim players.** The default sim player is independent of game speed: it acts whenever a
  decision is useful and skips ahead through stretches that need none. Attention and speed are a
  separate, occasional human model that maps ticks to real minutes ([agents.md](../agents.md)).
- **Agent code is player code (022, default).** Sim agents' numbers (knob defaults, weights,
  caps) are strategy, not rules, so CLAUDE.md's "no defaults in code" does not cover them; a
  number that mirrors a config or content value says so in a comment.
- **Hands-off measure (036, default; redefined 2026-11-22).** "Runs are smooth" (Act 1's exit)
  is read by the `handsoff` player: it loads the best known loadout at every reset; its only
  hand move is a stale-run retirement (`stale_days` with nothing bought), a backstop since 062's `retire` line.
  Its programs are data, a loadout file in the repo, updated when the rules or the best
  loadout change; runs until `if` opens (and that one) are the author's. Its
  measure is its credits a run and milestones from run 2 on as a share of the author's on the
  same seeds (the ladder's *Hands-off* table, [experiments.md](../experiments.md#measures)).
- **View (009, default).** `per_day` and `next_run` keep playing a day or a retirement on a
  copy (exact; they show the next day, not the last). Sim players keep the full view.
- **Oracle (013, defaults).** The oracle lives in `experiments/ceiling.json`; `hh-sim ladder` stays fast and `vs-human` excludes the
  oracle; its strategies are a fixed table; it takes `--milestones`. Its `Looks` and
  `milestones::Tracker` judge history milestones differently: the next oracle item moves it to
  `Tracker` semantics and re-pins.
- **Warning checks.** Surfaced, never gating; which are on: CLAUDE.md, *Warning checks*.
