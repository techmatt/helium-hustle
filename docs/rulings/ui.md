# Rulings: art and UI

Part of Matt's standing rulings ([index](../rulings.md): what **locked** and **default** mean).

- **Art direction (2026-11-15): B "Earthrise".** Warm gray regolith surfaces, an amber accent,
  a lunar horizon with the habitat at the foot of the left column. The layout is kept.
  Earth lives in the paintings, with no globe over the middle column. Resource colors are
  never hidden. A **light theme** for debugging stays, 1:1 with dark: same layout, markup and art, only color tokens swap. "Flash"/3D (a live
  view of the base growing) comes later; plan a home for it, build later.
- **Project and research art (2026-11-25, project_art_r1, research_glyphs_r1).** Project cards
  carry a picture in the building cards' style and place (96 px, under the title beside the
  text; the funding rows run full width below). Research cards lead their title with a 22 px
  chip, its square in the item's category color (`ResearchChip` in `icons.tsx`). The picks
  are listed in `experiments/results/100_wire_project_art_and_research_glyphs/`.
- **Building art (2026-11-16).** Cards use painterly cut-outs at each building's natural 3/4
  view (019's `cut34`: no fixed camera, a small irregular regolith patch, transparent
  background), as a 96 px thumbnail cropped tight on the building, under the title beside the
  flows (and a line, when it has one). A later "base grows" view uses the crisp painterly isometric set
  (019's `crisp`: fixed 2:1 camera and patch guide, one sun). The icon styles of icon_style_r1
  were all rejected.
- **Building lines (2026-11-25).** No building description by default: the stat lines say
  it. A building gets a line only for a mechanic worth pointing out that its stat lines don't
  show; rising cost is never one. *Default (093):* only the Arbitrage Engine and the Microwave
  Receiver have one. An empty `description` shows no line.
- **Nothing upcoming is telegraphed (2026-11-25).** UI for something not yet present stays
  hidden until it appears. The speculator column (Launch Pads' Markets) and the Adversaries
  panel's Speculators section wait for speculators' first arrival in the run (093 default:
  per run), the Adversaries panel and Speculator Analysis for their first arrival in the
  career (the Arbitrage Engine waits for Market Awareness: Matt, 2026-11-26), and side quest 1.a, which names them, starts then too (once 1.4 is
  reached). The Adversaries panel's Rival AIs section and Attention row wait for the rivals'
  first arrival in the career (096 default: the view's `seen`, present now or their arrival
  event seen). Projects never shows a locked card. No player-facing text names timelines before
  the swarm's reveal (1.6): the global projects' section reads "Global" until then.
- **Hide completed (Matt, 2026-11-25).** Research and Projects each have a "Hide completed"
  checkbox at the top (bought research; finished one-off projects), remembered per browser
  (`localStorage`, like the Programs mode), not in the save. Something completed while the
  view is open stays shown until the player leaves the view. *Default (104):* Projects starts
  ticked (096's no finished one-offs), Research unticked.
- **Ideologies view (Matt, 2026-11-26; 117).** The three axes carry gravitas: each a bold card
  with its emblem filling a 160 px left-edge slot, its name centered in its color, a large
  rank numeral and a full-width rank bar. Emblems (Matt's picks, 2026-11-26; 119, picker
  `ideology_emblems_r1`): Nationalist the claim marker, Humanist the garden dome, Rationalist
  the archive (`web/src/assets/ideology/`); an axis with no file shows its chip at 112 px.
- **Project funding (Matt, 2026-11-26; 116).** Holding a funding − or + keeps stepping: after
  ~0.4 s at ~10 steps a second, held past ~1.5 s 5 steps a repeat, past 3 s 10, stopping at 0
  or the cap. A rate fully met shows green; one the stock can't supply reads actual / set
  ("9.2 / 15.9 per day", the actual in red), and "Done in N days" counts the actual rate.
- **Bottom bar and modals (Matt, 2026-11-25).** The status bar's slots have fixed widths, so
  no value changing width moves the row, and Energy never reads "capped" there. Behind the run
  summary the ended run's last day stays on screen until Continue; the summary's sections read
  "Run N" and "Run N+1 starts with", each start value with its change from the ended run's
  start (green better, red worse; lower boredom growth is better), rounded to the row's
  precision (credits whole, multipliers 2 places): a change that rounds to zero shows nothing. A panel a purchase opens
  (Geopolitical Intelligence's Ideologies) shows at once, paused too.
- **Nav: one layout (Matt, 2026-11-25; 109).** The top-left nav never switches layout: two
  columns of tiles, each a medium icon (24 px) beside its label, in fixed-height rows; Act 1's
  full set fits about the old 3×3 large grid's height. No reserved blank cells (Matt,
  2026-11-26, over 109's default): the open tiles pack from the top in `content/nav.json`'s
  order with no gaps, a panel that opens appears with a short fade and the unread dot; every nav
  dot (unread, pad ready, No cargo) sits fully inside its own tile, at its right edge centered
  on the label (111, 112), side by side there when a tile shows more than one; and the
  utility tiles (Stats, Story, Achievements, Options, the file's last rows) follow directly
  after the last open play tile, moving down as panels unlock. Painted icons, no chip glyphs.
- **Quest mechanics text (Matt, 2026-11-25; locked).** Mechanics never live in narrative
  text. A quest's `body`, `story` and `payoff` (and a one-off event's `summary`, `body` and
  `story`, amendment #3171) are narrative only: no instructions, no numbers. Goals, rewards
  and unlocks are their own content fields (a quest's `goal` and `reward`; a one-off's
  `effect`, where `{rate}` now lives) and always show apart, in the
  mechanics style (`Mechanics` in common.tsx; 108): the normal UI font one size smaller, each
  line led by a small muted label inline ("Goal:", "Reward:", "Effect:"), on a faint
  warm tan tint with no border. No monospace and no blue strip, anywhere. "Goal: …" reads the
  same done or not (no "Done" label: a ✓ on the card's title says it). A reward is a
  headline, not an inventory: only what the player actually gets (a granted command, panel,
  building, resource or lasting bonus; 1.3: "The Launch Full Pads command."), never research
  merely appearing on the market (players find it by the unread dots; Matt, 2026-11-26);
  secondary effects go unlisted; a quest that grants nothing shows no reward line at all (no
  "None."). No how-to hints (Matt, 2026-11-26): quests carry no `hints` and nothing shows
  a "How:" line, no handholding until playtests say what's needed. A one-off's Effect states
  only the actual change, tersely ("Boredom starts growing: +0.2/day."), never explaining it
  or telegraphing what comes later; the narrative carries the meaning (Matt, 2026-11-26). The
  popups (reward on the last payoff page, the next goal on the last page; a one-off's Effect)
  and Story cards use it. The right panel is the exception: the main-quest box (amendment
  #3203: its title, its goal as one plain line in the UI font, and its progress), the
  side-quest lines and the Events lines carry no mechanics text. A content test
  keeps digits, `{rate}` and UI instructions out of narrative; new quests are written so.
- **Story cards (Matt, 2026-11-25; 108).** Every card in Story, main or side, starts
  collapsed: its picture, title (a ✓ once finished) and its story's first line with "…";
  an unfinished one also its progress. A click expands it to the whole story and its Goal
  and Reward lines (the active beat's How lines too); a finished main beat's expanded card
  offers *Replay its story* (its modals). The spine is one unbroken line behind every dot,
  the active beat's included, with side quests branching off it.
- **Right panel (Matt, 2026-11-26; 112).** Top to bottom: the main-quest box (pinned); the program;
  **Events**, one line per one-off event fired this run (title and its one-line summary, no
  body or Effect), newest first, a click reopening its popup; **Side Quests** (Matt,
  2026-11-26), a fold like Events, "Side Quests (done/started)", folded showing the
  closest-to-done one's progress, the open ones only (title and progress bar; a click opens its
  popup: story, goal, reward), hidden when none is open; Recent Activity. Burnout and the boredom phases supersede each other
  in Events: only the latest fired this run shows (the engine view's `superseded`); Recent
  Activity still logs each. In the run Burnout fires in, Boredom Rising is skipped (no popup,
  no Events line; Matt, 2026-11-26, 113): it is the phase-2 notice from later runs. No
  "Completed" section and no side-quest banner: Story is the quest archive.
- **Popups (Matt, 2026-11-26; 112).** Every one-off event (Propellant discovery, Political
  Currents, Burnout, each boredom phase, each adversary's arrival: Matt, 2026-11-26, P7) and every main and side quest completion opens a
  popup when it fires (the story modal, the clock held until its last button). A side quest's
  completion popup shows its reward, then it leaves the right panel. **"Hide popups for
  events you've already seen"**, a checkbox in Options (Matt, 2026-11-26), on by default, remembered per browser
  (`localStorage`): when on, an event whose popup this career has shown before (any earlier
  run; the set is kept in the save) doesn't pop up again but still lands in Events and Recent
  Activity. Quest completions pop up once a career anyway.
- **Recent Activity filter (2026-11-25).** Checkboxes under the list show or hide each kind
  of line (shipments, quests, story, research, projects, achievements, adversaries, other),
  one only for a kind already in the feed. *Default (096):* remembered per browser
  (`localStorage`), not in the save.
- **Icons (025-027).** One icon module, `web/src/ui/icons.tsx`, over
  `web/src/assets/{chips,nav,horizon}/`, so a new style swaps files only. Painterly nav icons
  (24 px beside the label, one nav layout above); the earthlight horizon; chip icons (color
  square, knocked-out glyph, name as tooltip) for every resource, ideology axis and rival,
  everywhere they appear. Rival colors are fixed (`icons.tsx`).
- **Scope icons (2026-11-25).** A small corner icon on any card whose effect survives a
  retirement (research, projects, side-quest rewards, Programming): **Legacy** survives a
  retirement, **Eternal** survives everything. Run-only things have none. Before the swarm's
  reveal every durable thing shows Legacy; Eternal appears only after it (the difference needs
  timelines to explain). Hovering gives one line ("Kept when you retire."). The engine's view
  says which (`keep`); `ScopeIcon` in `icons.tsx`. *Default (099):* placeholder glyphs ◆ and ∞
  on the amber accent; the icon shows done or not; an owned Programming (its card gone) shows
  none; a side quest with no reward of its own (1.c) has none.
- **Chip colors (028, default).** Resource, axis and rival chip colors stay TS maps in
  `icons.tsx` (the chip needs the hex for its ink-contrast choice, and they are the same in
  both themes); only the axis text colors are CSS tokens.
- **Story art and title (2026-11-23, story_title_art_r2).** Each act opens with a painted
  side band (3:1, title at the left): Act 1 round 2's `side3` (launch pad and domes), Acts
  2-4 round 1's `side`. No sky, bleed, tall backdrop, painted spine or full-width takeover:
  Story stays in the middle column on the amber spine. Title: `split-glow` (HELIUM in ink,
  HUSTLE in glowing amber, a glowing rule), CSS on the theme tokens. Later acts' art shows
  only once the act is reached.
- **Quest pictures (2026-11-23).** Every quest may carry a picture (content, display only):
  medium above its story modal's paragraph, at the left column's foot while it is the active
  main quest (replacing the lunar-base scene, which stays the fallback for a quest without
  one), and as a thumbnail beside a beat's title in Story. Layouts must look right without
  one. Act 1's are Matt's picks from `quest_pictures_act1_r1` (080), 1.1's `earthrise`; a
  picture's stem is a word, never a quest number (099). A
  side quest shows its picture in its popups (112; the completion banner is gone).
- **Cortex mock (2026-11-22).** Contrast first: inside the wheel's rim a plain solid ground per
  theme; decoration (bezel, corona, neurons) only outside the rim. It follows the app's
  theme. The Grand Theorems sit left of the wheel in a framed brass case, clear of the nav,
  centered on the core. Pair Theorems join non-adjacent sectors (a pentagram); bridges join
  neighbors (057). A socket's or fan's part switches with one click, free.
  One grammar (056): lines are prerequisites, Y and V forks, split sockets, bridges, gates; a
  sector's personality is only how it arranges them (Vigilance: a ladder). Drawn lines and
  prerequisites are one set. Hovering a locked node outlines its prerequisites and the lines
  in, owned and missing apart.
  Not built: an Act 4 upgrade lets a fan hold several parts at once.
- **Cheats (2026-11-23).** No CHEATED tag in the bottom bar; a cheated career stays marked
  in its save and files (and the debug info). The Cheat Fountain makes +1000 credits a day;
  its other +10/day incomes and −10/day boredom are unchanged.
- **Screenshots (2026-11-22).** An item that makes screenshots keeps them in its results
  folder and also copies them to `C:\Code\gdrive-sync\helium-hustle\screenshots\` as
  `NNN_*.png` (NNN: the item's number), a standing exception to the outside-the-worktree
  rule (CLAUDE.md, *Working rules*).
