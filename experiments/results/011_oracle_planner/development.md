# 011 oracle planner: how it got here (dev seeds 1-3)

Each step on dev seeds 1-3, careers of 3-4 retirements unless noted; the final read is in
[README.md](README.md). Numbers are means a finished run unless noted.

| Step | What changed | What it showed |
|---|---|---|
| v0 | Rollouts of the author under one changed goal (target, research, project, axis, Dream, all research), 300 ticks, every 50 | About the author (12.3k credits a run). Score swung on one shipment in the last 50 ticks of a rollout (closing income read over 50 ticks). |
| v1 | Closing income over the rollout's second half; market, diversified pads, hoarding added | Seed 1, 3 runs: 15k, 30k, 58k a run, peak 52/day; M5, M10-M13 reached (the author: none). Two runs ended forced: Dream starved of He-3 because Load ran first. |
| v2-v3 | Dream's program first under sustain; progress toward each open milestone scored; Long-Term projects by fraction paid, not per unit | Hoarding chosen at run start (M9's progress pulled from day 0): progress squared from then on. Science pushed into Foundation Grant instead of research. |
| probe | `examples/probe.rs`, the author at tick 1,200 | Energy at 0 (buildings starved; Fund, Dream, Load and Promote need their whole energy cost or fail), circuits, He-3 and titanium piled at caps unsold, He-3 demand 0.14, one pad blocked by land. Every run capped at 8-9 Solar Panels (titanium price 1.5^n). |
| v4 | Two levels: run strategies (bundles) over 2,000 ticks through retirements; tactics within | Peaks 46-65/day; M10 in run 1. Some runs collapsed: a mid-run switch to sustain, then hoarding, because building cut Dream's energy. |
| v5 | Sustain reserves energy for two Dreams; hoarding only from 5,000 credits; rollouts look as often as the real game (10, not 20) | Run 1 credits 21-32k (from 12-17k), M5 and M10 in run 1. |
| retire | "Retire now" as a strategy rollout | Retired at day 0-500 again and again: a retirement re-rolls the markets for free (the oracle sees which roll is better), and the retiring branch ends its 2,000 ticks in a mature run. Replaced by the prestige rule (retire when the run's average credits a day peaks) from day `min_run`. |
| power | Energy users leave room for every processor's command energy | Worse (11.5k a run against 15.4k): Solar Panels crowd out producers. Dropped for `main_last` (Dream, Fund and market programs before the main one). |
| full careers | 20 retirements, the milestones profile | 27.4k a run against the author's 12.2k, peak 68 against 24/day. Seed 2 stuck at 10k a run for 17 runs: 300-tick tactics steered off the circuits chain (no Fabricator, one Data Center, no science). |
| horizon | Tactics over 600 and 900 ticks, margin 2% | 600: 27.2k a run against 20.6k, M13 on 2 seeds of 3. 900: M11 on 2, M13 on 3, fewer credits, M2 later. 600 kept. |
| fidelity | The real game looked at plan times too; now only when the author would | M2 and M3 on the author's own ticks (420, 247) instead of 31 and 18 ticks later, every seed. |
| early | Milestone earliness 100 a tick against 20 | M7 in run 1 on 2 seeds of 3 against 1; the same M2, M3. Kept. |
| retire checks | Every 100 ticks instead of at strategy plans (500) | Run lengths no longer in 500-day steps. A 10% margin on the first retirement ended run 1 at day 625-730 on 5k credits: dropped; `min_run` 1,000 (400 let a young run retire at 730). |
| first read | Fresh seeds 3010-3012, every profile ([first_read.md](first_read.md)) | The oracle on top (29.8k a run against 12.1k); the quests profile forced in 87% of runs. |
| first margin | `first_margin` 100 and 200 (dev 1-3, 2 runs) | M4 1,350/1,455/2,070 and 1,037/1,355/2,070 against the author's 1,391/1,370/1,300; 200 halved seed 1's run 1 (7.3k against 15.2k). Kept at 0. |
| sustain fix | Probe of the quests campaign at tick 4,500, seed 3011: run 3 day 217 with one Data Center, 3 Solar Panels, no Excavator, 93 credits | The sustain goal's room for two Dreams applied before Dream was researched: every energy user gave way to a Solar Panel it couldn't pay for. Applied once Dream is unlocked: the campaign's runs 12-48k a run (dev), M9, M11, M13 and two Open Horizons sub-objectives on one seed. The final read was taken again on fresh seeds. |
| early launch | A goal: the run's first shipment by manual Launch with any cargo (dev 1-3, 3 runs) | M2 one tick earlier on 1 seed of 3, the rest the same: the first shipment waits on the pad's unlock and price. Reverted, so the shipped oracle is the one the final read measured (same fingerprint checked on seed 3014). |
| ideology weight | `ideology` 10 against 3 (dev 1-3, 3 runs) | The same games but one M11 more on 1 seed. Kept at 3; the knob stays. |
| Dream first | A "trade, Dream first" strategy (Dream Protocols researched before Trade Promotion), for M7 in run 1 | Never chosen in run 1 (dev 1-3, 2 runs): run 1 identical on every seed. Reverted. |

## Strategy probes (pinned with `strategy=k`, dev seeds 1-3, 4 runs)

| Strategy | Credits a run | Peak cr/day | Notes |
|---|---|---|---|
| trade, main program last | 14.9k | 50 | |
| trade, main program first | 15.4k | 54 | |
| trade + energy reserve for commands | 11.5k | 28 | dropped |
| trade + Humanist funded | 18.4k | 29 | M13 on 1 seed, M9 on 1 |
| trade + sustain | 11-30k | 15-19 | runs of 4,300 days, 0 shipments in them: Dream comes first, the economy starves |
| search (the oracle) | 17.9k-27k | 44-55 | |

Strategy search beats every pinned strategy on credits: the right bundle changes within a
career (author or trade early, funding once Geopolitical Intelligence is visible).
