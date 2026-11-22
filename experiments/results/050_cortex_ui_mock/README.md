# 050: Cortex UI mock

Superseded by 051 (`../051_cortex_radial_takeover/`): the mock is now a radial tree in a
takeover view, and this folder's `shots.mjs` no longer runs against it.

A UI-only mock of the talent tree (the Cortex), in `web/src/ui/CortexMock.tsx`, opened from
Options → Debug → *Cortex mock…*. Every node and theorem is a placeholder with no effect; the
state lives in that module, in memory, and is lost on reload. To remove it: delete the file,
and the import and `<CortexMockButton />` in `web/src/ui/Options.tsx`, and the README line.

- **Talent points:** TP n (0-based) is earned once lifetime Insight reaches the sum of
  round(10 × 1.2ⁿ) (placeholder). All 34 ranks take about 24K Insight. Grants: +100, +1K, +10K.
- **Branches:** Serenity, Logistics, Logic, Rhetoric, Vigilance, 6 nodes each, a node needs the
  one above it. Multi-rank: Steady Breathing 3, Lean Lines 2, Parallel Threads 2. Each last node
  grants one Axiom.
- **Theorems:** five pair Theorems in the ring, 2 Axioms each; the Unified Theorem, 5. Axioms
  are fungible.
- **Respec** refunds every point, Axiom and Theorem; Insight stays.

`shots.mjs` (HH_URL a `vite preview`) walks the done-when flow with checks (grant, Serenity to
its end, the Axiom, Logistics, the Serenity+Logistics Theorem, close and reopen, respec, no
game actions, no page errors) and saves `cortex-light.png` and `cortex-dark.png`; with `COPY`
it also copies them there (run with `C:\Code\gdrive-sync\helium-hustle\screenshots`).
