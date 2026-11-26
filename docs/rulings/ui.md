# Rulings: art and UI

Part of Matt's standing rulings ([index](../rulings.md): what **locked** and **default** mean).

- **Art direction (2026-11-15): B "Earthrise".** Warm grey regolith surfaces, an amber accent,
  a lunar horizon with the habitat at the foot of the left column. The layout is kept.
  Earth lives in the paintings, with no globe over the middle column. Resource colors are
  never hidden. A **light theme** for debugging stays, 1:1 with dark: same layout, markup and art, only color tokens swap. "Flash"/3D (a live
  view of the base growing) comes later; plan a home for it, build later.
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
  per run), the Adversaries panel, Speculator Analysis and the Arbitrage Engine for their first
  arrival in the career, and side quest 1.a, which names them, starts then too (once 1.5 is
  reached).
- **Icons (025-027).** One icon module, `web/src/ui/icons.tsx`, over
  `web/src/assets/{chips,nav,horizon}/`, so a new style swaps files only. Painterly nav icons at
  normal size (compact tiles use chips or glyphs); the earthlight horizon; chip icons (color
  square, knocked-out glyph, name as tooltip) for every resource, ideology axis and rival,
  everywhere they appear. Rival colors are fixed (`icons.tsx`).
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
  one. Act 1's are Matt's picks from `quest_pictures_act1_r1` (080), 1.1's `earthrise`. A
  side quest, which has no modal, shows its picture in its completion banner (Matt, 2026-11-23).
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
