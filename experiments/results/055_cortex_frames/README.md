# 055: Cortex mock, frames: a dial bezel, a neural corona and backdrop, a Theorem case

An art round for the Cortex mock (`web/src/ui/CortexMock.tsx`), light and dark, by Matt's
rulings of this session. Still a mock with all of 050-054's limits (UI-only, in memory,
placeholder powers, debug toggle). Inside the rim nothing changed: 053's plain ground.

- **Dial bezel.** A brass ring from r 512 to 556 (wheel units; the rim is 506) with a beveled
  radial gradient, keylines on both edges, fine ticks every 2 degrees on the outer edge (longer
  every 10), a fine row on the inner edge and a brass stud on each sector boundary. The sector
  names stay where they were (r 524-539) and now read as engraved on the band. The view grew
  from 572 to 612 units round the core to make room for the corona, so the wheel draws about
  7% smaller at a given window.
- **Corona.** 140 short filaments from r 559, each in its sector's color at low opacity, with
  side twigs and a few end dots; one slow opacity breathe on the whole group. A hovered
  Theorem lights its sectors' corona with their rim. Built so every step and twig heads within
  80 degrees of its root's outward line: nothing can reach inside the bezel.
- **Neural backdrop.** About 90 sparse neurons (a soma, three to five branching dendrites)
  outside r 640, with curved synapse traces between near neighbors (kept only if every sampled
  point clears r 640) and a bouton by each target. A few traces carry a slow traveling pulse.
  It is its own SVG under the wheel with the same viewBox, so it fills the box's margins round
  the wheel's square, masked to fade toward the box's edges. A vignette on the whole view,
  clear out to just past the corona (via the wheel's on-screen scale, `--cx-scale`), darkens
  the edges. Dark is dimmer: the field at 0.26 opacity against light's 0.5.
- **Theorem case, left.** The plaques sit in one brass display case: a beveled frame with
  diamond corner studs, a recessed well, and a chamfered brass title plate on the top edge
  ("GRAND THEOREMS", replacing the loose heading). Its column is 312px with a 30px left pad,
  so the frame stands well clear of the game's nav. Still centered on the core by auto
  margins (top-aligns instead of clipping when short), and hovering a plaque still lights its
  sectors.
- All decoration is generated once at load from a seeded generator (same picture every load)
  as a handful of SVG paths; motion is CSS only and off under `prefers-reduced-motion`.

## Check and screenshots

`shots.mjs` (HH_URL a `vite preview`) buys a few nodes, then in dark and light checks: the
title plate replaces the heading and sits on the case's top edge; the case is left of the
wheel, 30px clear of the nav, its middle on the core's (518.5 = 518.5 at 1600 x 1000) and
not clipped at 1280 x 720 (still centered there); every point sampled along the corona and
field paths lies outside the bezel (corona from r 558.9 against the bezel's outer edge at 556;
field from r 677.9 against its limit of 640); hovering Theorem of Persuasion lights Logic and Rhetoric, rim and corona.
It saves `cortex-{dark,light}.png` (1600 x 1000), `cortex-{dark,light}-hover.png` and
`cortex-{dark,light}-1280.png`, and with `COPY` as `055_*.png` (copied to
`C:\Code\gdrive-sync\helium-hustle\screenshots`). 054's `shots.mjs` no longer passes on this
build by design: it checks a right-hand column and the old heading.
