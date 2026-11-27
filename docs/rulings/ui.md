# Rulings: art and UI

Part of Matt's standing rulings ([index](../rulings.md): what **locked** and **default** mean).
The story side (quest text, Story cards, the right panel, popups, quest pictures) is
[story-ui.md](story-ui.md).

- **Art direction (2026-11-15): B "Earthrise".** Warm gray regolith surfaces, an amber accent,
  a lunar horizon with the habitat at the foot of the left column. The layout is kept.
  Earth lives in the paintings, with no globe over the middle column. Resource colors are
  never hidden. A **light theme** for debugging stays, 1:1 with dark: same layout, markup and art, only color tokens swap. "Flash"/3D (a live
  view of the base growing) comes later; plan a home for it, build later.
- **Project and research art (2026-11-25, project_art_r1, research_glyphs_r1).** Project cards
  carry a picture in the building cards' style and place (96 px, under the title beside the
  text; the funding rows run full width below). Research cards lead their title with a 22 px
  chip, its square in the item's category color (`ResearchChip` in `icons.tsx`). The picks
  are listed in `experiments/results/100_wire_project_art_and_research_glyphs/`. An
  affordable research shows it as a building card does, its name row tinted green, with no
  outline; an unaffordable one stays plain (Matt, 2026-11-27; 124).
- **Building art (2026-11-16).** Cards use painterly cut-outs at each building's natural 3/4
  view (019's `cut34`: no fixed camera, a small irregular regolith patch, transparent
  background), as a 96 px thumbnail cropped tight on the building, under the title beside the
  flows (and a line, when it has one). *Compact cards (Matt, 2026-11-27; 124):* the flows
  and the status row ("Output at cap") sit right of the picture; the cost (chips and amounts,
  land last) and the sell buttons share one row below, every number still shown; a card with
  a description keeps its text. A later "base grows" view uses the crisp painterly isometric set
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
  summary the ended run's last day stays on screen until Continue, a reload included (its state
  is saved with the open summary; 124); the summary's sections read
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
  on the label (111, 112), side by side there when a tile shows more than one. The order,
  read row by row (Matt, 2026-11-27; 124): Buildings, Research / Commands, Programs / Launch
  Pads, Projects / every other tile (Adversaries, Ideologies, Retirement, later panels) /
  Achievements, Story / Stats, Options; the utility tail (the file's last rows) follows
  directly after the last open play tile, moving down as panels unlock. Painted icons, no
  chip glyphs.
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
