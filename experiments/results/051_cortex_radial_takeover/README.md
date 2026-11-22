# 051: Cortex radial mock in a takeover view

Two pieces, one permanent and one a mock.

**The takeover view** (`web/src/ui/Takeover.tsx`, permanent): a large view covering the
middle and right columns while the left column stays. Its button sits under the nav grid; the
same button, Esc or any nav tile closes it. A strip on top shows the day, Boredom, Energy,
Credits, He-3 and Science (when visible) and a pause / resume button. The covered columns stay
mounted (CSS-hidden), so program editors keep their state and the game ticks on. A new view is
one `Takeover` entry (`id`, `label`, `title`, `shown`, `render`) in `App.tsx`'s `TAKEOVERS`;
`takeoversChanged()` redraws the buttons when a `shown` flips.

**The Cortex mock** (`web/src/ui/CortexMock.tsx`, still a mock): every power a placeholder with
no effect, state in module memory (the debug toggle too), nothing reaching the engine, store,
save or play log. Options → Debug → *Cortex mock in the left column* shows its button. To
remove it: delete the file, its entry in `TAKEOVERS` (App.tsx) and the `CortexMockToggle`
import and mount in `Options.tsx`; the takeover view stays.

- **Ring:** Serenity at the top, then clockwise Logistics, Logic, Rhetoric, Vigilance; each a
  colored, named 72° wedge.
- **A sector (9 nodes):** entry → a fork of two prongs, two nodes deep → the rejoin (either
  prong opens it; double ring) → an exclusive pair (squares joined by an "or"; early vs late,
  or one focus vs another, Rhetoric's an ideology shift) → the end node (thick ring, ◆, grants
  an Axiom; either pick opens it). A node opens at one rank of a parent.
- **Bridges (5):** hexagons, half each neighbor's color, at mid-radius on each boundary, needing
  the near prong's outer node in both sectors (Serenity–Logistics: Small Wins and Overflow
  Valves), so a hybrid picks its prongs toward the neighbor.
- **Gates (3):** dashed diamonds with a lock: Serenity (The Swarm Arrives), Logic (Rules for
  Building), Vigilance (False Victory); placeholder beats. A debug checkbox per beat unlocks
  one: it is owned at once, free, and grants an Axiom; respec keeps it.
- **Ranks:** single, but Focused Introspection (3) and Deeper Silos (2), shown as n/m.
- **Grand Theorems:** a vertical list beside the ring: five pair Theorems (2 Axioms) and the
  Unified Theorem (5). Hovering or focusing a pair Theorem lights its two wedges.
- **Inspector:** pointing at a node shows its kind, full name, text and why it can or can't be
  bought (e.g. "Locked: you took Fresh Start, its exclusive partner").
- **Talent points:** TP n costs round(10 × 1.15ⁿ) Insight (placeholder); grants +100, +1K,
  +10K; respec refunds points and Theorems.

`shots.mjs` (HH_URL a `vite preview`) walks the done-when flow with checks (the toggle, open
from the left column, the covered columns, the strip, days passing behind it, a fork's two
prongs, a bridge, an exclusive pair locking, an end node's Axiom, a pair Theorem lit by hover
and learned, a gate, pause from the strip, Esc, the button closing it, respec, no game
actions, no page errors) and saves `cortex-light.png` and `cortex-dark.png` at 1600 × 1000;
with `COPY` it also copies them there as `051_*.png` (run with
`C:\Code\gdrive-sync\helium-hustle\screenshots`).
