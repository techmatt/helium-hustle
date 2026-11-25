# 090 bug hunt

Recent changes since the 082 hunt (2803626), 089's Programming unlock first. Findings and
what became of each: [docs/qa-findings.md](../../../docs/qa-findings.md) rows 112-116 and the
deferred table.

- **qa sweep**, seeds 41-43 at 1280 and 1920 (both themes) on 089's build, then seed 45 on
  this item's: every pass reached the ending; the only findings were frame gaps while cargo
  ran beside it (and the Cortex mock's known slow open, now tagged known).
- **`probe1.mjs`** (run from the repo root, against a `vite preview` on port 59564): a new
  game paused, Sell Cloud Compute's Run clicked 60 times (50 ran, on the 100 energy of day
  0; `run-by-hand-mashed.png`), each other Run once (Idle's ran for free: finding 112), then
  the autopilot's quest steps to 1.9 watching the Programs tile, right column and Add to
  Program appear with Programming.
- **`probe2.mjs`**: a real mouse click on a Run, then Space (the pause key: no second run)
  and five Enters (five runs: a focused button, as any).
- **HH_PACE=120** (qa's real-speed look, now turning story pages): an untouched first
  visit reaches Day 99 at 0 credits with nothing naming Commands
  (`pace-day99-untouched.png`): deferred to Matt.
