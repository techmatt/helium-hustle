# Rulings: projects, bonuses, retirement

Part of Matt's standing rulings ([index](../rulings.md): what **locked** and **default** mean).
Split from [game.md](game.md) (133) when that part reached the docs size margin.

- **Project scopes (2026-11-21).** A project's `scope` is `run`, `timeline` or `forever`
  (replacing `tier`; the player calls `timeline` ones **global projects**, `run` ones
  Strategic). A `forever` project's progress survives a timeline's end as well as its
  completions. A building gated on a project's reward follows that project's scope: it
  requires the project's completion, not a career flag that outlives it. Every repeatable
  project carries `max_completions` (typically 5), and its rewards stack per completion
  (2026-11-21, by *Bonus stacking*; replaces 046's apply-once default). A global project's
  effects live on its completion, never on a career flag or quest reward (2026-11-24): a
  timeline's end takes them, the project is offered again and re-funding brings them back.
  The Deep Space Telescope is rebuilt each timeline for a passive while completed
  (placeholder +10% science, +10% nationalist ideology growth); what it revealed (the swarm,
  the calendar, what 1.6 opened) stays for the career, and completing it again replays no
  story. *Default (084):* the science bonus multiplies Research Lab output, not Buy Science. *Defaults (046):* a
  repeatable row keeps its rates across completions. **Funding cap (Matt, 2026-11-26):** a
  project takes at most 30 credits a day and 10 of each other resource (global params,
  raisable by later unlocks), so a run's stockpile can't be dumped in on the last day;
  deliberate per-row minimums stay (the Deep Space Telescope's 3,000 days).
- **Bonus stacking (2026-11-21).** Bonuses from one source add (+5% ×10 is +50%); separate
  sources multiply. Exponential stacking only for a few named exceptions (Paradigms, late
  techs, the catastrophic-success tech); repeatable projects are rare exceptions.
- **Starting buildings are free extras (Matt, 130).** A project's `starting_buildings`
  reward is owned and running (every building count includes it) but outside the price
  curve and the average price paid, and sells for no refund
  ([rules/projects.md](../rules/projects.md), *Rewards*).
- **Retirement gap (2026-11-21).** Retirement opens a gap (placeholder 5 years) simulated
  day by day, unseen in Act 1, in which only slow systems run: ideology decays
  proportionally toward zero (one half-life for all three axes, placeholder 1 year),
  Attention linearly. Retirement wipes buildings, resources, research, talent points (not
  built) and active programs; it keeps loadouts (which exist from the start and are always kept), bests
  and firsts, achievements, storyline progress, global projects, ideology and adversaries.
  Every retirement, in any act, simulates this same gap (2026-11-24): one parameter
  (`career.gap_days`), which in Acts 2 and up also moves the swarm's countdown; no year
  counter or calendar in Act 1.
- **The swarm's clock (2026-11-24).** The countdown starts when 1.6's Deep Space Telescope
  completes; before that nothing counts toward the swarm, so a player who hasn't finished it
  never reaches a timeline's end. The discovery is career-long (2026-11-24, 086): from then on
  every timeline's countdown, super timelines too, runs from that timeline's first day and
  arrives on schedule whether or not the Telescope has been rebuilt (same cost, no cheaper
  rebuild); the rebuild gives only its passive. Arrival: placeholder 100 years (36,500 days) after discovery,
  the same every timeline; carried Earth science and research only push it later. No
  per-timeline shortening or arrival floor (escalation is an Act 4 idea); the false victory's
  entanglement still shortens later timelines (Matt, question 085_1: the draft's intended
  escalation), and the other draft swarm mechanics stand. *Default (085):* a super
  timeline's `super_arrival_days` is also 100 years (was 6,000 days).
