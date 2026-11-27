# 116: parked batch (projects panel, right panel, adversary arrivals)

Rules revision 40 → 41. Browser check: `check.mjs` against a `vite preview` build
(`HH_URL=http://localhost:<port>/ node experiments/results/116_parked_batch_projects_panel_adversaries/check.mjs`),
about 100 s. Every row passed on the final build.

| Done-when | Check | Result |
|---|---|---|
| Holding + keeps stepping and speeds up | late_unlocks, Foundation Grant credits: rate sampled every 100 ms while held | 1 on press, still 1 at 0.3 s, then +1 per 0.1 s, +5 a repeat from ~1.5 s, stops at 30 (+ disabled); − held runs back to 0 |
| Underfunded shows actual vs set | credits at 30 a day with the stock emptied | "0 / 30 per day", the actual red; science met at "10/day" green; card "Done in 334 days" → "Stalled" |
| Side quests below Events, collapsible | right column order; fold header | quest-pin, events-list, side-quests, events-activity; "Side Quests (1/3)"; folded "… 1.b — Full Cluster: 2 / 12"; fold kept across panel switches |
| Hide-popups checkbox in Options | DOM | absent from the right panel; in Options, on by default |
| Speculators' first arrival opens a modal | new scenario `speculators_due`, pads launched by hand | popup "Speculators" with its Effect line |
| Arbitrage Engine waits for Market Awareness; Full Cluster at 12 | `tests/story.rs` `speculators_arrive_with_a_popup_and_the_engine_waits_for_market_awareness` | not met with speculators seen; met once Market Awareness is bought; 11 items no, 12 items complete 1.b |

Screenshots: `116_*.png` here.
