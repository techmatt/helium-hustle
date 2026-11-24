# The arc

A map from the design doc's beats to the quests that carry them (`story` and `side` rows in
`content/events.json`, each with its `beat` id) and their status. It records the current state
only. Status: **aligned** (matches the design), **draft** (the beat's trigger and opens are
in, every mechanic in its simplest form, placeholder numbers and text), **unaligned** (the 042
scaffolding, not yet aligned with the design doc). The rules are in [rules/](rules/README.md):
[events.md](rules/events.md) for quests, [attention.md](rules/attention.md) for the
adversaries, [career.md](rules/career.md) for retirement, [timeline.md](rules/timeline.md) for
Acts 2-4.

The acts' placeholder names and first quests (`content/acts.json`, display only: Story's
chapter bands): Act 1 The Lunar Operation (1.1), Act 2 The Swarm (Q8 Experiments as a
Service), Act 3 The Vault (Q14 The Vault), Act 4 The Dyson Sphere (Q17 Dyson Sphere).

## Act 1: the lunar operation (draft-playable)

About 25% of play (a readout, not a target). Skill: from clicking commands to programs that run
whole runs. The chain is 1.1 to 1.11 in order, each opening one mechanic at a time (074); 1.a,
1.b and 1.c are side quests that complete beside it and never hold it up. A new game can build
only Solar Panels and Data Centers. Each chain quest's transition is a story modal in the UI
(its `story` and `payoff` paragraphs in `content/events.json`, placeholder text; no doc copies
them; [rules/events.md](rules/events.md), *Story modals*). The Opens columns are the design's,
by hand; `hh-sim gates` lists what each quest opens as built (its unlocks, the `requires`,
research and projects gated on it).

| Beat | Quest (id) | Trigger | Condition | Opens | Status |
|---|---|---|---|---|---|
| 1.1 Online | `qpower_boot_sequence` | game start | 2 Solar Panels | the Buy Science command and the research view (only the Blueprints show in it until 1.8) | aligned |
| 1.2 Blueprints | `qblueprints` | 1.1 | the Basic Blueprints research (20 science) | (the research itself unlocks the Regolith Excavator, Smelter, Launch Pad and Storage Depot) | aligned |
| 1.3 First shipment | `qfirst_shipment` | 1.2 | one launch carrying 20+ titanium | its start grants 20 Propellant once a career; while it is active pads carry Titanium only, and a partial pad launches by hand from its card; done, Advanced Blueprints (50 science: Battery, Refinery, Fabricator) goes on sale | aligned |
| 1.4 Full load | `qfull_load` | 1.3 | one launch carrying a full pad of He-3 | (teaches He-3, the claim's real export) | aligned |
| 1.5 Processors | `qautomate_automation` | 1.4 | 2 Data Centers | the Launch Full Pads command; program logic (the `program_logic` flag at 2 working processors); loadouts are open from run 1 and always kept | draft |
| 1.6 Speculators notice you | `qspeculators_notice` | 1.5 | speculators present (Attention from units sold) | the adversary tab (the `adversaries` panel; speculator bursts land only while they are present) | draft |
| 1.7 Burnout | `qburnout` | 1.6 | 1 retirement (boredom forces the first; one forced before 1.7 counts, so 1.7 completes as it is reached) | the Retirement panel (voluntary retirement); bests and firsts carry. Boredom runs from day 1 with its meter shown; 1.7's modals explain it | draft |
| 1.8 Lab | `qlab` | 1.7 | a Research Lab | the full research catalog, Projects and Foundation Grant, the Recreation Dome; Geopolitical Intelligence shows; the first global projects (Blueprint Archive, then Conditional Logic, opens `if`) are offered | draft |
| 1.9 Three voices | `qthree_voices` | 1.8 | career-best rank 5 on each axis (three sub-objectives) | each axis's major rank-5 effect (ideology.md); its rank-5 global project | draft |
| 1.10 Land rush | `qland_rush` | 1.9 | rivals present (Attention from land claimed) | the Contest Claims command (neutralizes the rivals) | draft |
| 1.11 Deep Space Telescope | `qdeep_space_telescope` | 1.10 | the Deep Space Telescope global project (30,000 credits and 5,000 science at no faster than 3,000 days, so several runs) | Act 2: the `swarm_revealed` flag, the swarm's 100-year countdown, the Timeline view, the Experiment Lab | draft |
| 1.a Big launch | `qbig_launch` (side) | 1.6 | one launch pays 1,000+ credits | Disrupt Speculators (also takes speculator Attention off) | draft |
| 1.b Full cluster | `qfull_cluster` (side) | 1.8 | every item of one research category of 2+ items (not the Blueprints) bought in one run | placeholder: every run starts with 50 Science | draft |
| 1.c Blueprints for everyone | `qblueprints_for_everyone` (side) | 1.8 | the Blueprint Archive global project (placeholder: 500 credits, 200 science) | Basic and Advanced Blueprints count as bought at the start of every run | draft |

Placeholder choices (049), all for Matt to keep or replace:

- **Major ranks (rank 5).** Nationalist: speculator bursts stop landing. Humanist: Dream costs
  only its energy (no He-3). Rationalist: research bought in an earlier run is free. The ranks
  between scale as before.
- **Techniques.** Anti-rival (1.10): *Contest Claims*, 500 credits and 10 energy, sets the
  rivals' Attention to zero (the neutralize stub). Speculator disruption (1.a): the existing
  *Disrupt Speculators*, now gated on 1.a instead of Market Awareness, also takes 20
  speculator Attention off per execution.
- **Retirement.** A 5-year gap (1,825 days), simulated day by day, the same in every act and
  unseen in Act 1 (no calendar): ideology scores halve every 365 days (so a run starts with
  about 3% of the last one's scores), Attention decays linearly. Retirement keeps loadouts,
  bests and firsts, achievements, quests, global projects, ideology and adversaries.
- **Mapped onto what existed.** Self-Maintenance (the lesser boredom measure) is Stress
  Tolerance, in the research category of that name; Daydreaming is the renamed Dream
  Protocols. Research categories are 1.b's clusters (Overclock Algorithms and Market Analysis
  can be finished in Act 1; Self-Maintenance holds the draft Novelty Simulation, shown after
  1.11; the Blueprints category is no cluster). Q2 First Extraction became an event, since
  removed for 1.2 Blueprints (074); Q5 Market Awareness left the line (Market Awareness stays a
  research item).

**Achievement candidates** (performance tests, out of the quest line; achievements for them
aren't built yet):

- *Open Horizons* (was Q6): rank 5 on any axis, a global project, ten years in one run and
  100,000 credits in one run.
- *Signal Detected* (was Q7): a run of 300 days with no order from the player, only programs.
  The `quiet_days` condition it used stays in the engine.

## Acts 2-4: unaligned with the design doc

The 042 scaffolding: rough mechanics, placeholder numbers, art and text, kept running so the
autopilot reaches the ending. **Unaligned with the design doc.** Act 2 starts when the
Telescope (1.11) is finished.

**The swarm's clock.** Finishing the Telescope discovers the swarm and starts its countdown:
placeholder 100 years (36,500 days), the same every timeline, pushed later only by carried
Earth science and research; each retirement's 5-year gap moves it 5 years. Nothing counts
before the Telescope, so a player who never finishes it never reaches a timeline's end. A
timeline's end takes the Telescope, and each new timeline's countdown starts when it is
rebuilt (same cost). The false victory's entangled swarm comes sooner in later timelines
(×1/1.5); a super timeline's is its own `super_arrival_days` (also 100 years), counted only
while the Telescope stands (kept if it stood when the vault was spent, else rebuilt).

| Story step | Quest | Opens | Status |
|---|---|---|---|
| Experiments as a Service | Q8 `qexperiments_service`: an Experiment Lab | the Earth Lab view | unaligned |
| Earth Science | Q9 `qearth_science`: 1,000 Earth science this timeline | | unaligned |
| The Swarm Arrives | Q10 `qswarm_arrives`: a timeline ends (the mind is sent back to run 1) | the Timeline history | unaligned |
| Catastrophic Success | Q11 `qcatastrophic_success`: the Consensus Engine tech | zero-cost Fund | unaligned |
| Rules for Building | Q12 `qrules_for_building`: the Meta-Controller tech | the Meta-Controller view (reset plan) | unaligned |
| False Victory | Q13 `qfalse_victory`: repel the swarm once | a stronger, sooner swarm | unaligned |
| The Vault | Q14 `qthe_vault`: the Temporal Vault tech | the Vault view, the Tachyon Condenser | unaligned |
| Filling the Vault | Q15 `qfilling_vault`: 3,000 tachyon in the vault | | unaligned |
| Super Timeline | Q16 `qsuper_timeline`: spend the vault | the Dyson view, Neodymium Mine, Dyson Foundry, the Dyson Sphere | unaligned |
| Dyson Sphere | Q17 `qdyson_sphere`: complete the Dyson Sphere | | unaligned |
| Overload | Q18 `qoverload`: the swarm meets the finished sphere | the ending | unaligned |
| Epilogue | `qepilogue`, never completed | the game runs on | unaligned |
