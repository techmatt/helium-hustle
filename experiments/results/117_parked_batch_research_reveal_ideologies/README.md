# 117: parked batch (research reveal, Electrolysis, 1x after retirement, Ideologies)

Each edit checked in the running game by `check.mjs` (a `vite preview` build, headless Edge,
the `late_unlocks` scenario with a save edit: Regolith Processing, Liquid Cooling, Market
Awareness, Daydreaming and Propellant Synthesis bought in an earlier run). All checks pass.
Screenshots here and in the shared screenshots folder as `117_*.png`.

- **1x after a retirement (P1).** Retired at 10x; after the summary's Continue the new run
  plays at 1x (`web/src/store.ts`: the summary resumes 1x, and space resumes 1x too).
- **Research groups by this run's triggers (P2, rules 42).** Each research row has a
  `reveal_when`, its group's trigger this run, which binds even for an item bought in an
  earlier run. Run 2's day 0: groups Blueprints and Political Influence (Programs is gone for
  good in this scenario: its career flag is set), though the career had bought items in
  Extraction, Self-Maintenance and Market Analysis (`117_p2_run2_day0_research.png`). A Research
  Lab bought: Self-Maintenance and Extraction appear at the day's end, with the Research tile's
  dot (`117_p2_nav_dot.png`, `117_p2_after_lab.png`). In 116's `speculators_due` scenario with
  Market Awareness bought before, Research shows only Blueprints until the speculators arrive,
  then Market Analysis (`117_p2_market_analysis.png`). Programs on day 0 (the Programming
  flag not yet held) is checked in hh-core's `research_groups_reveal_by_this_runs_triggers`.
- **Electrolysis Plant (P3, rules 42).** Its card reads +1 Propellant/day, −1 Energy/day,
  −2 Ice/day (`117_p3_electrolysis.png`).
- **Ideologies view (P4).** Each axis a bold card: a 160 px left-edge emblem slot the card's
  full height (193 px at 1600 px wide), tinted in the axis color, holding the axis chip at
  112 px until the emblem art lands; the name centered in its color (30 px, spaced capitals);
  "RANK" and a 56 px numeral; a full-width rank bar 30 px tall carrying the score and the next
  rank in its label; career best under it. `117_p4_ideologies.png` (light),
  `117_p4_ideologies_other_theme.png` (dark).

Run it from the repo root against a static build:
`HH_URL=http://localhost:<port>/ node experiments/results/117_parked_batch_research_reveal_ideologies/check.mjs out/117_parked_batch_research_reveal_ideologies/shots`
