# Rulings: the game

Part of Matt's standing rulings ([index](../rulings.md): what **locked** and **default** mean).
Programs have their own part ([programs.md](programs.md)); boredom and ideology theirs
([boredom.md](boredom.md)).

- **Pillars (locked).** Plans beat waiting: the game is solved by clever plans, manual early and
  programs later at high speed ("unlimited speed" is about 200x for now). Programs are the core
  identity. Boredom is a speed governor. No loss: stages are wins, and a finale is far off. No
  offline time: the game advances only while it runs in the browser. Numbers grow naturally.
- **Be intentional about what can be gamed (2026-11-21).** A solution that sounds "too
  boring" (idling, waiting it out, repeating the last run) must never work well. Every gate and
  quest condition states which boring strategies it rules out.
- **Speed.** Decisions happen at 1-3x with brief bursts to 5-20x. 200x comes later, once
  programs can automate more, behind tech not yet in the game. 200x is the player's fastest
  speed (2026-11-25): the Game Speed row ends there, cheats on or off. Act 1's speeds are
  ❚❚ 1x 3x 10x (Matt, 2026-11-26): 50x and 200x wait for Accelerated Scheduling (placeholder,
  30 science, a career flag bought once like Programming), kept in the engine but shown
  nowhere until Act 2 places it; a save at 50x or 200x without it reloads at 10x. A new run
  starts at 1x after a retirement, whatever speed was set (Matt, 2026-11-26; 117).
- **Run-1 real time.** 1x stays 1 tick a second; the 25-35 real-minute target for run 1 is what
  moves ([principles.md](../principles.md)).
- **Tuning.** Every number is open. Matt plays the UI build and decides where fine-tuning
  starts. A better main story quest is designed later.
- **Placeholder content.** Many thresholds, texts and numbers are placeholders. Don't spend
  items adjusting them (reachability, wording) until content is designed for real; bug hunts
  and QA may list them, not fix them.
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
- **Act 1 follows the design doc's beats (2026-11-21).** Quests carry beat ids;
  [arc.md](../arc.md) is the beat → quest → status map, current state only. Performance
  tests (Open Horizons, Signal Detected) belong in achievements, not the quest line (not
  built yet; candidates in arc.md).
- **No legacy numbers (2026-11-25, standing).** When things are renumbered, nothing keeps the
  old numbers: no mapping column, alias or "was" note in content, code or docs. Content ids
  may stay as they are.
- **Act 1's main quest line (2026-11-26, numbers placeholders).** 1.1 Online (2 Solar Panels),
  1.2 First Shipment (one launch of 20+ titanium), 1.3 Full Load (a full He-3 pad; Launch
  Full Pads opens), 1.4 Trade (1,500 credits from launches this run), 1.5 A Voice (rank 5 on
  any axis; Geopolitical Intelligence and Programming open as it starts), 1.6 Deep Space
  Telescope. A quest's number comes from its place in the chain, never the data. No Science
  quest (deleted 2026-11-26): players find Research Labs useful on their own. Blueprints is no quest: players work it out
  (Basic Blueprints, 10 science, shows from 1.1 and unlocks the Excavator, Smelter, Launch Pad
  and Storage Depot; Advanced Blueprints, 20, on sale once 1.2 is done: Battery, Refinery,
  Fabricator and, from 2026-11-25, the Research Lab, no longer gated on 2 Data Centers). A new game builds only Solar Panels and Data Centers; 1.1 opens Buy Science
  (placeholder 10 credits + 1 energy → 1 science) and the research view. 1.2's start grants
  20 propellant once; pads carry Titanium only while it is active; a partial pad launches by
  hand from its card. Load Launch Pads opens with the first pad. 1.4 never mentions
  speculators, who arrive on their own (playtest batch 1, 2026-11-25). *Default (095):* 1.5
  reads this run's rank.
- **Burnout and rivals leave the line (2026-11-25).** Burnout: the first time boredom bites is
  a modal story event (*default (095):* boredom's phase 2, once a career, `boredom_bites`);
  every forced retirement is a modal saying why the AI retired and that years passed, in words:
  Act 1 has no calendar and never mentions timelines. Rivals move to Act 2: none appear in
  Act 1 (*default (095):* land draws rival Attention only once the swarm is revealed), and
  Contest Claims goes with them (open once rivals first arrive). Speculators are Act 1's
  adversary.
- **Side quests give permanent rewards (2026-11-25, placeholders).** They outlive every reset,
  as completions do. 1.a Big Launch: one launch pays 1,000+; +10% launch pay. 1.b Full
  Cluster: research 12 items in one run (Matt, 2026-11-26); every run starts with 50 science. 1.d Three Voices:
  career-best rank 5 on each axis, across runs; ideology decays half as fast in the
  retirement gap. 1.c Blueprints for Everyone is cut (Matt, 2026-11-27, 129); its Blueprint
  Archive stays a global project (placeholder 2,000 credits + 300 science): while it counts
  as completed both Blueprints are bought at every run's start (lost at a timeline's end,
  back when re-funded), and every blueprint research ever bought in the career is 20%
  cheaper. *Defaults (095):* 1.b counts every item bought this run, the Archive's
  Blueprints too; 1.b hangs beside 1.3 (once a Research Lab is built), 1.d follows 1.5.
- **Staggered unlocks (2026-11-26).** Research groups reveal by this run's triggers, so every
  run unfolds like run 1 (Matt, 117; research.md). Each run's first Research Lab opens
  Self-Maintenance and Extraction: Regolith Processing (100 science, +50% excavator output)
  and Liquid Cooling (200 science; opens Run Simulations, 3 energy + 1 ice → 6 credits, no
  boredom, meant to pair with the Ice Extractor). Process Telemetry is out of Act 1: shown
  nowhere until Act 2 places it. The first retirement opens the Retirement panel and Chemical
  Energy, and from then on Daydreaming, the boredom research and Recreation Dome Blueprints
  with each run's Lab. Each run's speculators' arrival opens Market Analysis (Speculator
  Analysis opens Disrupt Speculators). A Voice's start opens Geopolitical
  Intelligence, its completion Overclock Protocols. **Global projects open at A Voice (Matt,
  2026-11-26):** A Voice's start opens the Projects panel and Blueprint Archive, Conditional
  Logic, Lunar Cartography and Foundation Grant (placeholder costs since 130:
  [rules/projects.md](../rules/projects.md), *Content*). A first retirement
  before A Voice still opens Projects with Blueprint Archive, Conditional Logic and Foundation
  Grant, as before.
  The Recreation Dome is never ungated (Matt, 2026-11-25): it needs its own Blueprints research,
  Recreation Dome Blueprints (300 science, placeholder), shown from the first retirement; Programming shows from
  A Voice's start on, every later run too (Matt, 2026-11-26, question 111_1); Propellant Synthesis shows with the propellant discovery (4 shipments) alone. The main quest
  line marks only significant things: never "research this"; a beat may be an untelegraphed
  expectation. Refinery upkeep is 5 regolith + 2 energy; the excavator makes 2 regolith a day.
- **Prose round-trip (2026-11-25).** Every player-facing string in `content/` exports to one
  Markdown file grouped by screen (`content/prose.md`, committed) and imports back by id
  (`python scripts/prose.py`); ids never change, a missing id leaves its string alone. UI
  prose the engine's view carries (the run summary's line on why the run ended) is content
  too, in `content/messages.json`.
- **Story modals (2026-11-23).** Each chain quest's transition is a modal that pauses the
  game: the finished quest's payoff (its reward under it), then the next quest's story, then
  its goal in the mechanics style (story-ui.md, *Quest mechanics text*);
  one paragraph a modal, chained or merged as the story reads best, one button, play resumes
  after the last; a chain's unread pages and the speed it resumes are in the save, so a
  reload mid-chain shows them again (082's deferral). No modal at game start but the first quest's; side quests and one-off events pop up too (story-ui.md, *Popups*); sims
  never see them (UI only). Placeholder story for every Act 1 beat lives only in content
  (`story` and `payoff` on the quest rows); docs point at it, never copy it. Matt's story
  pass comes much later.
- **Act 1 is mostly a tutorial (2026-11-22).** The phase before programming is fully
  unlocked gets no special design effort, trivial special cases at most. So program players
  get no count or rate readings before research unlocks them (059's default stands).
- **Pads start with No cargo (2026-11-23).** Players must learn to pick a pad's cargo: a new
  Launch Pad mid-run and every pad a run starts with has No cargo, idle until a good is picked
  (Load skips it, `idle_pads` counts it); No cargo stays one of a pad's choices. Old saves keep
  their pads' goods (a state reads back as saved; the web autosave of another revision is still
  discarded, by *Saves* in [engine.md](engine.md)). Every sim player and script picks a cargo. *Defaults (069, 074):* quest 1.2
  teaches it; the sim players' pick is one fixed hand, the first good a pad may carry now
  (`agents::pick_cargo`: Titanium during 1.2, else the loading priority's first); the Launch Pads tile shows a red dot while a working pad has No cargo.
- **Adversaries on minimal Attention (2026-11-21).** Speculators and rivals each have one
  Attention, raised by a driver, present at a threshold (one tier), decaying linearly, with
  a neutralize stub. All placeholders ([rules/attention.md](../rules/attention.md)).
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
- **The day's rate is production less demand (2026-11-20).** What buildings and programs
  wanted counts, met or not, so Energy reads negative all through a brownout; a capped
  resource's rate is its uncapped rate. It is the Resources panel's per-day number (for the
  next day). *Defaults (035):* unmet demand counts only on an input that ran short; the rate
  counts everything the day's tick did to the stock (purchases by program lines, pad
  loading, boredom growth and Dream too).
- **Rules port picks (004, defaults).** The starting-credits bonus is 0.02 of the best run
  (`career.credits_bonus_fraction`), not the Godot build's /100. Ideology ranks are geometric
  ([rules/ideology.md](../rules/ideology.md)), not the data's `rank_thresholds`. Lunar
  Cartography and Deep Core Survey stay unreachable until the narrative pass. The boredom cap
  grows per Recreation Dome. Invented numbers (demand base, noise, coupling, cooldowns)
  are config candidates.
