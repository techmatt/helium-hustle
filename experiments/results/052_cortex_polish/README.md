# 052: Cortex mock, polished

The second round of the Cortex mock (`web/src/ui/CortexMock.tsx`) in its takeover view. Still a
mock with all of 050/051's limits: placeholder powers with no effect, UI-only state in module
memory, nothing reaching the engine, sim, save or content, a debug-only toggle, and removable by
deleting the file, its `TAKEOVERS` entry (App.tsx) and the toggle in Options.tsx.

**Takeover change (permanent):** a `Takeover` entry may set `bleed`, which drops the body's
padding and scrolling (`.takeover-bleed` in style.css) for a view that fits itself to the space.
The Cortex uses it; the strip and head are unchanged.

## How it plays

- **Left-click adds a point, right-click takes one back** (keyboard: Enter / Delete). A removal
  is refused, with the reason in the tooltip in red, when it would strand an owned node: the
  check finds every owned node that still traces back to the core through full-rank parents (a
  least fixed point, so Serenity's loop can't hold itself up), and refuses if any is left out.
  An end node whose Axiom pays for a learned Theorem is refused too (respec frees it).
- **Fill before advancing:** a node opens only when a parent is at **full rank** (or both, for a
  bridge). Multi-rank nodes: Deeper Silos 2, Focused Introspection 3, Open Letters 2,
  Tripwires 2, shown as rank pips around the top of the socket.
- **Split sockets** replace 051's exclusive pairs: one node drawn as two (or three) slices,
  each with its own glyph; a click on a slice takes it (1 point), lights it, and leaves the
  rest dim. A dim slice doesn't buy; its tooltip names what was taken. Right-click frees it.
- **Hover tooltip** at the cursor (moved by a ref, no re-render): sector, shape and place,
  name, rank pips, effect, cost, and the status line (why it's locked, what a click does, or a
  refusal). The inspector is gone.

## The five shapes (8-9 nodes each, plus the end node's Axiom)

| Sector | Shape | Nodes |
|---|---|---|
| Serenity | a loop: entry → the loop's foot → a six-node ring, each ring node opening from either neighbor; its crown is a two-part socket, the end node above it | 8 + gate |
| Logistics | a supply line: a six-node spine (a 2-rank node, a two-part socket near the end) with two short stubs | 8 |
| Logic | a binary tree: a 3-rank root, two forks, four leaves; the inner two rejoin in a V (OR gate). Junction glyphs are logic gates (buffer, AND, NAND, OR) | 9 + gate |
| Rhetoric | a three-way fan: a hub fanning to two side vanes and a 2-rank stem; the stem feeds a three-part socket (Homeland / Commons / Reason line: Nationalist, Humanist, Rationalist), one fan line per part | 8 |
| Vigilance | layered walls: two concentric wall arcs with gaps, three nodes in the inner wall, two in the outer, each outer node reached from either inner node below it | 8 + gate |

Bridges (5) sit on each boundary, fed by dashed lines in their two neighbors' colors and needing
both feeders at full rank; their bezel is half each color. Gates (3) stay: dashed, a lock until
their story beat's debug checkbox, then owned, free, and worth an Axiom.

## Art (dark only, both themes)

A silicon wafer (die grid, notch) with seeded circuit traces and vias, and neural threads near
the core; a glowing core showing free points (and free Axioms); round sockets with chip glyphs
(026's `glyphs.mjs`, copied in as placeholders, plus a few drawn here: logic gates, lock,
tower, bridge, speech); available nodes pulse, locked ones are hollow and dim; bought links glow
in their sector's color with a small light running outward along them; unbought links are dim
traces; sector names curve along the rim (upright on the lower half). Grand Theorems are brass
plaques in the side column; hovering one lights its sectors' rims and floors. Motion is CSS
only (opacity and dash-offset animations), gone when the view closes;
`prefers-reduced-motion` stops it.

## Check and screenshots

`shots.mjs` (HH_URL a `vite preview`) walks the done-when flow with checks (no scrolling at
1600 × 1000 and 1280 × 720; five named sectors of 8-9 nodes; the hover tooltip; filling Focused
Introspection before its branches open; right-click refused and allowed; the loop held up from
its far side; both kinds of split socket; a bridge; Axioms; a Theorem lit and learned; a gate;
Esc; respec; no game actions; no page errors) and saves dark PNGs here: `cortex-dark.png` (the
whole view), `cortex-dark-tip.png` (a tooltip on Rhetoric's socket), `cortex-dark-refused.png`
(a refused removal), `cortex-dark-small.png` (1280 × 720). With `COPY` it also copies them
there as `052_*.png`.
