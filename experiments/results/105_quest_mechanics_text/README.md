# 105: quest mechanics text

Matt's ruling (2026-11-25, docs/rulings/ui.md, *Quest mechanics text*): mechanics never live in
narrative. Quests carry `goal`, `reward` and `hints`, and one-off events carry `effect`
(amendment #3171). All of them render in the mechanics strip (`Mechanics`, web/src/ui/common.tsx).

- `check.mjs`: a browser check against `vite preview` using the `story_acts` scenario. It
  checks that every Story card (finished beats, the active beat, side quests, 1.b Full
  Cluster included) shows a goal and a reward in monospace with no number in its narrative
  line, that 1.2's replayed payoff modal shows its reward, that each Events Story entry has
  its strip, and that the main-quest box has its strip. 79 checks, all ok, at 10:45pm.
- `105_story_act1_side_quests.png`: Story with 1.4 to 1.6, side quests 1.a, 1.b and 1.d, and
  the Events list with effects.
- `105_story_modal_reward.png`: 1.2's payoff page, with its reward in the strip.
