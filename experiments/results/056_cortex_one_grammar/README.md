# 056: Cortex mock, one grammar: Vigilance a ladder, every line a prerequisite, locked hover

The Cortex mock (`web/src/ui/CortexMock.tsx`), light and dark, by Matt's rulings of this
session. Still a mock with all of 050-055's limits (UI-only, in memory, placeholder powers,
debug toggle).

- **One grammar.** Inside the bezel the tree draws only its five pieces: lines (prerequisites),
  Y and V forks, split sockets, bridges, gates. Vigilance's layered walls are gone (the dashed
  arcs, the gap rule and every wall or gap word in the tooltips), and so are the dashed sector
  seams: they ran from the core to the rim straight through each bridge, reading as an edge
  into it. The sectors now differ only by their floors' tint at the boundaries.
- **One line per prerequisite, Rhetoric too.** Rhetoric's three-part Ideology Line socket used
  to take three lines from Open Letters, one into each part, two of them left dim into an owned
  socket as if still missing: a rule of its own. It now takes one line like every socket.
  Rhetoric's fan is the three-way fork at Market Sense (two vanes and the stem).
- **Vigilance is a ladder.** Watchful Eyes (the foot) opens both rails, three nodes each:
  Thin Profile, Countermeasures, Low Profile on the left; Tripwires (2 ranks), Fast Decay and a
  new Cold Trail on the right. Two rungs join the rails' upper two steps; a rung is a
  prerequisite both ways (like Serenity's loop links), so either side opens the other. Both
  rails' tops feed the Posture split socket (a V), then Bastion, the end node with its Axiom.
  Nine nodes besides the Hard Lessons gate. Bridges: Public Image now takes Countermeasures
  (on the lower rung, so either rail reaches it), Calm Watch takes Cold Trail.
- **Edge lighting is one rule.** A line lights when its prerequisite is at full rank and the
  node it opens is owned, either way round for a link that is a prerequisite both ways;
  Serenity's loop no longer has its own case. A link lit the other way round draws reversed,
  so its pulse always flows into the node it opened.
- **Locked hover.** Hovering a locked node outlines each prerequisite with a ring and draws the
  line from it over the trace, both cased in the hover ink (white in dark, near-black in light)
  so they can't be taken for a trace (a bridge feed is dashed in its sector's color, and
  Rhetoric's red and Logic's green sit close to the two looks): solid green (`--cx-ok`) for one
  at full rank, dotted red (`--cx-bad`) for one still missing (a prerequisite with some ranks but not all counts as
  missing, as it does for unlocking; the tooltip shows its ranks). The "Locked: needs ..." line
  stays. Only a bridge can be locked with a prerequisite already met (any-of nodes are open as
  soon as one is), so the bridges show both looks.
- **Tooltip placement.** The tooltip used to open right of the pointer and covered exactly
  the prerequisites it now outlines. It now takes the nearest place round the pointer, outward
  first, that fits and covers neither the pointer nor an outlined prerequisite.

## Check and screenshots

`shots.mjs` (HH_URL a `vite preview`) buys a few nodes, climbs Vigilance's left rail and
crosses the lower rung (Fast Decay opens with Tripwires unowned, the rung lights, removing
Countermeasures is refused), then in dark and light checks from the DOM:

- drawn edges (`data-from`, `data-to`) and prerequisites (`data-req`) are the same set of
  pairs, 57 of them, none missing and none extra, with exactly one line per pair; every edge is
  a prerequisite one way; gates draw none ([edges.md](edges.md): the per-node table). Both
  sides come from the same `req` lists, so this pins how the lines are built (no extra or
  doubled line, none dropped); the geometry below is what proves each one is drawn where it
  says;
- every edge's path starts on its prerequisite (or in the core) and ends on its node, and none
  of its points (57 paths, 2,554 points sampled every 2 units) comes within 12 units of any
  other node's rim (past its bezel and outline ring), so no line seems to reach a node it does
  not open;
- nothing else stroked comes inside the rim (the five rim arcs, from r 505.9, are the nearest);
  no walls or seams in the DOM;
- Vigilance: nine nodes, two rails from the foot, two mutual rungs at one height each, both
  tops into the socket, then the end node; no tooltip of any node or part mentions walls or
  gaps, and every locked one says what it needs;
- hovering the locked Serenity/Logistics bridge outlines Dreamweaver solid (have) and Overflow
  Valves dotted (missing) with the two lines in the same looks, the tooltip unchanged and
  covering neither; Public Image likewise (Countermeasures have, Rival Reading missing);
  Posture outlines both rail tops as missing; an open node outlines nothing; leaving clears it.

It saves `cortex-{dark,light}.png` (the view), `cortex-{dark,light}-locked-bridge.png` (the
locked bridge hovered) and `cortex-{dark,light}-locked-ladder.png` (Posture hovered), here and
with `COPY` as `056_*.png` (copied to `C:\Code\gdrive-sync\helium-hustle\screenshots`).
055's `shots.mjs` no longer runs on this build: it buys the removed `vigilance-i2`.
