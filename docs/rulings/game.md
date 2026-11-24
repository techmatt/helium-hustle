# Rulings: the game

Part of Matt's standing rulings ([index](../rulings.md): what **locked** and **default** mean).
Programs have their own part ([programs.md](programs.md)).

- **Pillars (locked).** Plans beat waiting: the game is solved by clever plans, manual early and
  programs later at high speed ("unlimited speed" is about 200x for now). Programs are the core
  identity. Boredom is a speed governor. No loss: stages are wins, and a finale is far off. No
  offline time: the game advances only while it runs in the browser. Numbers grow naturally.
- **Be intentional about what can be gamed (2026-11-21).** A solution that sounds "too
  boring" (idling, waiting it out, repeating the last run) must never work well. Every gate and
  quest condition states which boring strategies it rules out.
- **Speed.** Decisions happen at 1-3x with brief bursts to 5-20x. 200x comes later, once
  programs can automate more, behind tech not yet in the game.
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
  the calendar, what 1.11 opened) stays for the career, and completing it again replays no
  story. *Default (084):* the science bonus multiplies Research Lab output, not Buy Science. *Defaults (046):* a
  repeatable row keeps its rates across completions; every row's `min_days` is its largest
  cost over 30 rounded up (the old flat cap's fastest finish).
- **Bonus stacking (2026-11-21).** Bonuses from one source add (+5% ×10 is +50%); separate
  sources multiply. Exponential stacking only for a few named exceptions (Paradigms, late
  techs, the catastrophic-success tech); repeatable projects are rare exceptions.
- **Act 1 follows the design doc's beats (2026-11-21).** Quests carry the design doc's beat
  ids; [arc.md](../arc.md) is the beat → quest → status map, current state only. Q6 Open
  Horizons and Q7 Signal Detected left the quest line: performance tests belong in
  achievements (not built yet; candidates in arc.md). Placeholder techniques stand for 1.10
  (Contest Claims) and 1.a (Disrupt Speculators).
- **Act 1 opening: one mechanic at a time (2026-11-23).** The chain is 1.1 Online, 1.2
  Blueprints, 1.3 First Shipment, 1.4 Full Load, 1.5 Processors, then Speculators, Burnout, Lab,
  Three Voices, Land Rush and the Telescope (1.6-1.11, conditions and unlocks as before). A
  new game builds only Solar Panels and Data Centers; 1.1 opens Buy Science (placeholder 10
  credits + 1 energy → 1 science) and the research view, which shows only the Blueprints until
  1.8. Basic Blueprints (20 science) unlocks the Excavator, Smelter, Launch Pad and Storage
  Depot; Advanced Blueprints (50, on sale once 1.3 is done) the Battery, Refinery and
  Fabricator. 1.3: one launch of 20+ titanium; its start grants 20 propellant once; pads
  carry Titanium only while it is active; a partial pad launches by hand from its card. 1.4: a
  full pad of He-3 (it teaches He-3). Launch Full Pads stays hidden until 1.5 completes; Load
  Launch Pads opens with the first pad. The Launch Plans event is gone. Boredom runs from day
  1 with its meter shown; 1.7's modals explain it, and a retirement forced earlier completes 1.7
  as it is reached. Side quest 1.c Blueprints for Everyone (from 1.8): the Blueprint Archive
  global project (placeholder 500 credits + 200 science, offered first) makes both Blueprints
  bought at every run's start while it counts as completed (2026-11-24: lost at a timeline's
  end, back when re-funded; 1.c stays completed). Refinery upkeep is 5 regolith + 2 energy. *Default (074):* 1.b's
  cluster leaves out the Blueprints category (else 1.c would complete 1.b for free).
- **Story modals (2026-11-23).** Each chain quest's transition is a modal that pauses the
  game: the finished quest's payoff, then the next quest's story, then its task in one line;
  one paragraph a modal, chained or merged as the story reads best, one button, play resumes
  after the last; a chain's unread pages and the speed it resumes are in the save, so a
  reload mid-chain shows them again (082's deferral). No modal at game start but the first quest's; side quests unchanged; sims
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
  discarded, by *Saves* in [engine.md](engine.md)). Every sim player and script picks a cargo. *Defaults (069, 074):* quest 1.3
  teaches it; the sim players' pick is one fixed hand, the first good a pad may carry now
  (`agents::pick_cargo`: Titanium during 1.3, else the loading priority's first); the Launch Pads tile shows a red dot while a working pad has No cargo.
- **Adversaries on minimal Attention (2026-11-21).** Speculators and rivals each have one
  Attention, raised by a driver, present at a threshold (one tier), decaying linearly, with
  a neutralize stub. All placeholders ([rules/attention.md](../rules/attention.md)).
- **Retirement gap (2026-11-21).** Retirement opens a gap (placeholder 5 years) simulated
  day by day, unseen in Act 1, in which only slow systems run: ideology decays
  proportionally toward zero (one half-life for all three axes, placeholder 1 year),
  Attention linearly. Retirement wipes buildings, resources, research, talent points (not
  built) and active programs; it keeps loadouts (which exist from the start and are always kept), bests
  and firsts, achievements, storyline progress, global projects, ideology and adversaries.
- **Major ranks (2026-11-21).** Ideology rank 5 is a major rank (probably 10, 15 and 20 too)
  bringing a discrete major change; the ranks between scale.
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
