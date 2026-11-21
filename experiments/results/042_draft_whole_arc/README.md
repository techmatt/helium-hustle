# 042: a playable draft of the whole arc

Rules 13. The story spine is [docs/arc.md](../../../docs/arc.md); the draft rules are
[docs/rules/timeline.md](../../../docs/rules/timeline.md) (module `timeline`, variant `draft`).
Everything past Q6 is draft: placeholder mechanics, numbers, text and art.

## Done-when reads

**Native, fresh seeds 3094-3096 (ledger).** `hh-sim accelerate --target end --budget 200000`
(the default boost and drivers), alone on the box.

| Seed | Q7 (reveal) | Q10 (first timeline ends) | Q13 (false victory) | Q16 (super timeline) | Q18 (ending) | Every step | Wall |
|---|---|---|---|---|---|---|---|
| 3094 | 89,495 | 92,465 | 95,345 | 96,608 | 102,578 | yes, 102,578 days | 19.5 s |
| 3095 | 25,219 | 28,189 | 33,272 | 34,371 | 40,341 | yes, 52,041 days | 13.1 s |
| 3096 | 14,151 | 17,365 | 19,521 | 20,812 | 26,782 | no: M12 (an Act 1 milestone) not met in 200k days | 41.6 s |

Career days. Act 1 to Q6 is 041's autopilot, unchanged. Each later act takes a few thousand
days: Q7 to Q18 in 12,600-13,100 days on all three seeds.

**Browser, fresh seed 3094.** The same script: Q18 after 18 presses in 9.4 s real time,
all five views opened, replay ok.

**Browser, seed 3092 (development).** `arc.mjs` against `vite preview` (headless Chrome).
It presses *Next quest step* until the ending overlay shows, and opens each later-act view
as its tile appears. Q18 came after 18 presses, in 17.5 s real time (22.8 s with the views
and screenshots). All five views opened (Timeline, Earth Lab, Meta-Controller, Vault,
Dyson). Replay check ok. Screenshots of each view as it first opened, and of the ending, are in
this folder (`*.png`, light theme).

**Act 1 is unchanged.** The golden game plays its 1,042 actions and logs the same lines
before and after this item (`hh-sim run --policy script`). Only its fingerprint moves: the
config gained the `timeline` module and the content gained four resources at 0. The author's
3-retirement `trace` is identical. The ladder's first eight careers end on the same tick
counts. Those were all the cells that finished inside a 55 s timeout.

## Known gap: Act 1 milestones under *Everything*

Every quest through Q18 is reached on all seeds tried: 2, 3088, 3090, 3092 and 3094-3096.
Some Act 1 *milestones* now go unmet inside the budget:

- seed 2: M10, even in 600k days;
- seed 3088: M10, in 120k days;
- seed 3096: M12, in 200k days.

Before 042, long Act 1 careers met them eventually. Now the career hard-resets every few
thousand days after Q7. On seed 2, after victory, the author deadlocks on an Act 1 target. Inspected at day 300k
(run 64, day 2,378): its build target is the 9th Storage Depot, which costs 769 regolith
against a 650 cap. While it waits, it keeps every Refinery, Smelter and Fabricator paused,
because those eat the regolith its target is short of. Science stays near 0, nothing is
researched, so Promote never unlocks. Each run repeats this until the 4,000-day stale
retirement. The Boredom Dampers make post-victory runs long enough to reach that 9th depot.
The fix is player strategy, a target above its cap being unreachable in `reachable_target`
(`agents/author.rs`). Left for a later item: it changes the author's Act 1 play, so it moves
the ladder pins and the balance reads. 041's slow test now asserts the
quests and the ending, and prints the milestones it misses (question
`helium-hustle__042_draft_whole_arc__1`).

## Tuning (development seeds 3088, 3090, 3092)

| Change | Why |
|---|---|
| `min_arrival_days` 1500 and `max_strength` 100 (new params) | A run of lost timelines grew the swarm's strength geometrically until `Fx` overflowed (a panic). |
| Act 3 building costs cut to below the base caps (titanium 20, circuits 10) | The Dyson Foundry cost 20 circuits against a cap of 10, so it could never be bought. |
| The author's arc layer moved to the end of its look | The author stops funding every project but its own pick, which zeroed the Dyson Sphere's rates each look. |
| After victory, the arc layer idles the draft buildings | The Experiment Labs ate the science that Act 1's research needs, so M7 and M10 were never met afterwards. |
| The repel bar is a fixed `repel_science`, not `× strength` | In the browser (a fresh driver each press) one career lost over 300 timelines in a row and never repelled the swarm. |
