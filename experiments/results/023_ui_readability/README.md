# 023 UI readability: screenshot set

Made by `npm run e2e` with `HH_SHOTS` (section 5 of `web/scripts/e2e.mjs`, which also checks
each state). Light theme, 1600 x 900.

| Shot | Shows |
|---|---|
| [read-capped-and-cards.png](read-capped-and-cards.png) | Scenario `at_cap` with cheated resources: Energy 100 / 100 still showing its +98 surplus with a CAP tag (Resources) and "+98/day (capped)" (status bar); Battery and Storage Depot are affordable, with tinted name rows, while Data Center and Solar Panel are not |
| [read-pad-short-story-unread.png](read-pad-short-story-unread.png) | Scenario `pad_loading` at day 210: a full pad with "Fuel 19.4 / 20 Propellant" in red, "loading", a grey Launch button. The boredom beat (day 200) has fired: Story is folded with an unread count of 1, and Q2 stays in the main-quest slot |
| [read-pad-ready-story-open.png](read-pad-ready-story-open.png) | 0.6 Propellant granted: "Fuel 20 / 20" in green, a green **Ready**, green Launch button and border, a green dot on the Launch Pads tile. Story is unfolded with its count still showing during the first second |
| [read-story-read.png](read-story-read.png) | About a second later: Story's count is gone with no click, and the quest is still pinned |

The middle-column story banner now tells only main-quest texts. Before this change, `tell()` in
`web/src/store.ts` queued every event text (including boredom beats) into the same banner queue
as quest texts, so a beat could take the banner in front of a newly opened quest.
