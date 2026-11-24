# 074: Act 1 opening and story modals

Rules revision 25. The ruling and the chain: [docs/rulings/game.md](../../../docs/rulings/game.md)
(*Act 1 opening*, *Story modals*), [docs/arc.md](../../../docs/arc.md). Story text: the quest
rows' `story` and `payoff` in `content/events.json` only.

## The author's run 1 (fresh seeds 3143-3146)

`hh-sim trace --agents author --first-seed S --max-ticks 2200`, debug build; the career
log's quest lines: [author_run1_quests.txt](author_run1_quests.txt). Every seed passes 1.1 to
1.5 in run 1:

| Quest | Day (all four seeds) |
|---|---|
| 1.1 Online | 60 |
| 1.2 Blueprints | 120 |
| 1.3 First Shipment | 285 |
| 1.4 Full Load | 1,010-1,031 |
| 1.5 Processors | same day as 1.4 (its 2 Data Centers were already built) |
| forced retirement | 1,422-1,434 |

1.4 is the slow beat: ~740 days from 1.3, most of it waiting on 50 science for Advanced
Blueprints (Buy Science from credits past the build target's price) and then filling a
100-cargo pad from He-3 stores capped at 20-50. A placeholder number for Matt to tune, not
fixed here.

## Screenshots

From the e2e's section 1 (a new game played by hand): `story_modal_intro.png` (1.1's modal
on a new game), `story_modal_blueprints.png` and `story_modal_first_shipment.png` (the 1.1→1.2
and 1.2→1.3 chains' pages with their task), `pad_titanium_only.png` (1.3's picker: No cargo
and Titanium only). Copied to the shared screenshots folder as `074_*.png`.
