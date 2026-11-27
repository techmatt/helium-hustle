# The arc

A map from the design doc's beats to the quests that carry them (`story` and `side` rows in
`content/events.json`; a side quest carries its `beat` id, a chain quest's number comes from its
place in the chain) and their status. It records the current state
only. Status: **aligned** (matches the design), **draft** (the beat's trigger and opens are
in, every mechanic in its simplest form, placeholder numbers and text), **unaligned** (the 042
scaffolding, not yet aligned with the design doc). The rules are in [rules/](rules/README.md):
[events.md](rules/events.md) for quests, [attention.md](rules/attention.md) for the
adversaries, [career.md](rules/career.md) for retirement, [timeline.md](rules/timeline.md) for
Acts 2-4. A chain quest's number (`act.k`, the `k`th quest of its act) is derived at load
(`Content::number_chain`), so a reorder or deletion touches only the chain; numbers are still
written out in docs, code comments, tests and web/scripts: a renumbering greps each moved
number (`git grep -n '\b1\.5\b'`).

The acts' placeholder names and first quests (`content/acts.json`, display only: Story's
chapter bands): Act 1 The Lunar Operation (1.1), Act 2 The Swarm (2.1 Experiments as a
Service), Act 3 The Vault (3.1 The Vault), Act 4 The Dyson Sphere (4.1 Dyson Sphere).

## Act 1: the lunar operation (draft-playable)

About 25% of play (a readout, not a target). Skill: from clicking commands to programs that run
whole runs. The chain is 1.1 to 1.6 in order, each opening one mechanic at a time, and marks
only significant things; 1.a to 1.d are side quests that complete beside it, never hold it up,
and give permanent rewards. A new game can build only Solar Panels and Data Centers. Each chain
quest's transition is a story modal in the UI (its `story` and `payoff` paragraphs in
`content/events.json`, placeholder text; no doc copies them; [rules/events.md](rules/events.md),
*Story modals*). The Opens columns are the design's, by hand; `hh-sim gates` lists what each
quest opens as built (its unlocks, the `requires`, research and projects gated on it).

| Beat | Quest (id) | Trigger | Condition | Opens | Status |
|---|---|---|---|---|---|
| 1.1 Online | `qpower_boot_sequence` | game start | 2 Solar Panels | the Buy Science command and the research view (Basic Blueprints, 10 science, shows: the Excavator, Smelter, Launch Pad and Storage Depot; nothing asks for it) | aligned |
| 1.2 First shipment | `qfirst_shipment` | 1.1 | one launch carrying 20+ titanium | its start grants 20 Propellant once a career; while it is active pads carry Titanium only, and a partial pad launches by hand from its card; done, Advanced Blueprints (20 science: Battery, Refinery, Fabricator, Research Lab) goes on sale | aligned |
| 1.3 Full load | `qfull_load` | 1.2 | one launch carrying a full pad of He-3 | (teaches He-3, the claim's real export); the Launch Full Pads command | aligned |
| 1.4 Trade | `qtrade` | 1.3 | 1,500 credits from launches this run (from before the quest's start too) | nothing. Speculators arrive on their own (Attention from units sold, [attention.md](rules/attention.md)) | draft |
| 1.5 A voice | `qa_voice` | 1.4 | rank 5 on any ideology axis this run | on start, Geopolitical Intelligence (Fund, the Ideologies panel) and Programming (research, 200 science, bought once for the career: the Programs view, the full editor; programs, Add to Program and loadouts are open from day 1); done, Overclock Protocols | draft |
| 1.6 Deep Space Telescope | `qdeep_space_telescope` | 1.5 | the Deep Space Telescope global project (30,000 credits and 5,000 science at no faster than 3,000 days, so several runs) | Act 2: the `swarm_revealed` flag, the swarm's 100-year countdown, the Timeline view, the Experiment Lab, the rivals | draft |
| 1.a Big launch | `qbig_launch` (side) | the `speculators_seen` flag (beside 1.4) | one launch pays 1,000+ credits | for good: +10% launch pay (`shipment_credit_mult` ×1.1) | draft |
| 1.b Full cluster | `qfull_cluster` (side) | the `research_lab` flag (beside 1.3) | 12 research items bought in one run | for good: every run starts with 50 Science | draft |
| 1.c Blueprints for everyone | `qblueprints_for_everyone` (side) | the `a_voice` flag (beside 1.5, so it starts as the Archive is offered) | the Blueprint Archive global project (placeholder: 2,000 credits, 300 science) | (the Archive's effects while it counts as completed: Basic and Advanced Blueprints bought at the start of every run; Blueprints research bought in an earlier run 20% cheaper) | draft |
| 1.d Three voices | `qthree_voices` (side) | 1.5 | career-best rank 5 on each axis, in any runs | for good: ideology decays half as fast in the retirement gap (`ideology_half_life_mult` ×2) | draft |

**Silent unlocks** (one-off events, re-applied from day 0 of every later run once seen; the
research they reveal waits for each run's own firing):

| When | Event | Opens |
|---|---|---|
| the first Research Lab | `research_lab_built` (flag `research_lab`) | each run: the Self-Maintenance and Extraction research (Regolith Processing, Liquid Cooling: opens Run Simulations) |
| A Voice's start (1.5) | `a_voice_projects` (flag `a_voice`) | the Projects panel and the first global projects: Blueprint Archive, Conditional Logic (opens `if`), Lunar Cartography, Foundation Grant |
| the first retirement | `first_retirement` (flag `retired`) | the Projects panel with Blueprint Archive, Conditional Logic and Foundation Grant if A Voice hasn't opened them, Chemical Energy, Daydreaming and the boredom research and Recreation Dome Blueprints (300 science; the Dome needs them) once each run's Lab is built, the Retirement panel (voluntary retirement) |
| speculators' arrival | `speculators_arrive` (flag `speculators_seen`) | the Adversaries panel; each run: the Market Analysis research (Speculator Analysis opens Disrupt Speculators) |
| rivals' arrival (Act 2) | `rivals_arrive` (flag `contest_claims`) | Contest Claims |

**Burnout** is no quest: the first time boredom bites (its phase 2, once a career) the story
event `boredom_bites` is a modal, and every forced retirement's summary modal says why the AI
retired and that years passed. Boredom runs from day 1 with its meter shown. Act 1 has no
calendar.

Placeholder choices (049), all for Matt to keep or replace:

- **Major ranks (rank 5).** Nationalist: speculator bursts stop landing. Humanist: Dream costs
  only its energy (no He-3). Rationalist: research bought in an earlier run is free. The ranks
  between scale as before.
- **Techniques.** Speculator disruption: *Disrupt Speculators*, open once Speculator Analysis
  is researched, takes 20 speculator Attention off per execution. Anti-rival (Act 2): *Contest
  Claims*, 500 credits and 10 energy, sets the rivals' Attention to zero (the neutralize stub).
- **Retirement.** A 5-year gap (1,825 days), simulated day by day, the same in every act and
  unseen in Act 1 (no calendar): ideology scores halve every 365 days (so a run starts with
  about 3% of the last one's scores; every 730 with 1.d), Attention decays linearly. Retirement
  keeps loadouts, bests and firsts, achievements, quests, global projects, ideology and
  adversaries.
- **Mapped onto what existed.** Self-Maintenance (the lesser boredom measure) is Stress
  Tolerance, in the research category of that name; Daydreaming is the renamed Dream
  Protocols. Market Awareness is a research item, not a quest.

**Achievement candidates** (performance tests, out of the quest line; achievements for them
aren't built yet):

- *Open Horizons*: rank 5 on any axis, a global project, ten years in one run and 100,000
  credits in one run.
- *Signal Detected*: a run of 300 days with no order from the player, only programs. The
  `quiet_days` condition it used stays in the engine.

## Acts 2-4: unaligned with the design doc

The 042 scaffolding: rough mechanics, placeholder numbers, art and text, kept running so the
autopilot reaches the ending. **Unaligned with the design doc.** Act 2 starts when the
Telescope (1.6) is finished.

**The swarm's clock.** Finishing the Telescope discovers the swarm and starts its countdown:
placeholder 100 years (36,500 days), the same every timeline, pushed later only by carried
Earth science and research; each retirement's 5-year gap moves it 5 years. Nothing counts
before the Telescope, so a player who never finishes it never reaches a timeline's end. The
discovery is career-long: every later timeline's countdown runs from its first day, whether
or not the Telescope (which a timeline's end takes) is rebuilt; the rebuild gives only its
passive. The false victory's entangled swarm comes sooner in later timelines (×1/1.5); a
super timeline's is its own `super_arrival_days` (also 100 years).

| Story step | Quest | Opens | Status |
|---|---|---|---|
| Experiments as a Service | 2.1 `qexperiments_service`: an Experiment Lab | the Earth Lab view | unaligned |
| Earth Science | 2.2 `qearth_science`: 1,000 Earth science this timeline | | unaligned |
| The Swarm Arrives | 2.3 `qswarm_arrives`: a timeline ends (the mind is sent back to run 1) | the Timeline history | unaligned |
| Catastrophic Success | 2.4 `qcatastrophic_success`: the Consensus Engine tech | zero-cost Fund | unaligned |
| Rules for Building | 2.5 `qrules_for_building`: the Meta-Controller tech | the Meta-Controller view (reset plan) | unaligned |
| False Victory | 2.6 `qfalse_victory`: repel the swarm once | a stronger, sooner swarm | unaligned |
| The Vault | 3.1 `qthe_vault`: the Temporal Vault tech | the Vault view, the Tachyon Condenser | unaligned |
| Filling the Vault | 3.2 `qfilling_vault`: 3,000 tachyon in the vault | | unaligned |
| Super Timeline | 3.3 `qsuper_timeline`: spend the vault | the Dyson view, Neodymium Mine, Dyson Foundry, the Dyson Sphere | unaligned |
| Dyson Sphere | 4.1 `qdyson_sphere`: complete the Dyson Sphere | | unaligned |
| Overload | 4.2 `qoverload`: the swarm meets the finished sphere | the ending | unaligned |
| Epilogue | `qepilogue`, never completed | the game runs on | unaligned |
