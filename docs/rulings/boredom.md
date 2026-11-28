# Rulings: boredom and ideology

Part of Matt's standing rulings ([index](../rulings.md): what **locked** and **default** mean).
The game's other rulings are in [game.md](game.md); the rules themselves in
[rules/boredom.md](../rules/boredom.md) and [rules/ideology.md](../rules/ideology.md).

- **Boredom is a speed governor, not a punishment (Matt, 2026-11-27; 129).** Through Acts 1
  and 2 it is an escalating, ongoing crisis to plan around: good boredom control makes runs
  long, never infinite. Each ideology has its own boredom tool, and embracing that path is the
  easier (not the required) route to rank 5. Simplicity prior: the same rule for every
  ideology.
- **Numbers easy to think through (Matt, 2026-11-27; 129; a soft goal, not a rule).** Keep
  player-facing numbers easy to think through for as long as is reasonable (0.5 is fine,
  0.495 is not).
- **Escalation (2026-11-27; 129).** After the last boredom phase the growth rate is
  multiplied by 1.25 each year (365 days; a placeholder param), so no fixed relief outlasts
  it. Boredom phases are restated in years where that reads better (dates: [ui.md](ui.md)).
- **One modifier per axis (2026-11-27; 129).** `m = 1.1^rank`, negative ranks symmetric
  (below 1, never zero); each axis's ×m shows on its Ideologies card. It does exactly two
  things: its tagged buildings (2-3 per axis, its boredom building among them) cost ÷m, land
  excluded, and built buildings are unaffected; and its boredom tool's effect is ×m. Nothing
  else scales by rank: not energy, general output or upkeep. It replaced every earlier
  per-rank bonus and discount (demand, speculator decay, science output, land, research
  price, boredom growth, overclock duration). *Default (129):* the tags are Nationalist:
  Recreation Dome, Launch Pad, Arbitrage Engine; Humanist: Data Center, Battery;
  Rationalist: Research Lab, Fabricator.
- **Boredom tools (2026-11-27; 129, numbers placeholders).** Nationalist: the Recreation
  Dome, its cap increase ×m dominant in Act 1 (+100 base), plus a small daily relief ×m
  (0.5 a day while working). Humanist: Dream, flat relief ×m (replacing ×1.05^rank), costs
  as before. Rationalist: Theorize, a new command spending science for flat relief ×m (20
  science for −3 boredom an execution: −2 until 130's parity tuning, a free variable by
  Matt's 130 ruling), its unlock beside Daydreaming's. *Default (129):*
  Theorize's unlock is the research Thought Experiments (300 science, Self-Maintenance,
  shown and revealed with Daydreaming and listed right after it).
- **Axis nudges (2026-11-27; 129).** Using a path's tool nudges its axis score a little: Dome
  purchases Nationalist, Dream runs Humanist, Theorize runs Rationalist, with no cross
  penalty. *Default (129; tuned in 130 for parity):* 2.5 per Dome purchase (was 5), 0.5 per Dream or
  Theorize execution (was 0.25).
- **Refunds (2026-11-27; 129).** Each building type tracks its average price paid; selling a
  bought copy refunds half of it (Matt: share 0.5, every resource), not of today's price, so
  rank swings can't be farmed. Land is refunded as before; free copies refund nothing. Not
  shown in the UI.
- **Major ranks (2026-11-21).** Ideology rank 5 is a major rank (probably 10, 15 and 20 too)
  bringing a discrete major change; the ranks between raise the modifier.
