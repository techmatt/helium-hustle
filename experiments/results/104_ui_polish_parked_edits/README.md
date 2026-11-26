# 104: UI polish, parked edits

Each edit checked in the running game by `check.mjs` (a `vite preview` build, headless Edge);
all checks pass. Screenshots here and in the shared screenshots folder as `104_*.png`.

- **Bottom bar (P1).** Fixed-width slots (day, run, labels, rates; the two bars take the
  rest): the slots' left edges stay put with "Day 1" or "Day 1,234,567" and "0/day" or
  "+1,234.5/day". Energy at its cap reads its plain rate there ("+98/day"); Resources still
  marks it "cap". `104_p1_status_bar.png`.
- **Hide completed (P2).** Research (default unticked) and Projects (default ticked, 096's
  rule) each keep their own `localStorage` key (`hh.hideCompleted.research`,
  `hh.hideCompleted.projects`). Unticked, Projects now shows finished one-offs as done cards.
  A research bought while the view is open stays until the view is left.
  `104_p2_research_hidden.png`, `104_p2_projects_shown.png`.
- **At once (P3).** Not a UI refresh problem: the client already redraws after every action.
  The Ideologies tile came from an event that fires at the tick's end. The view
  (`events::pending_panels`) now shows a nav tile whose one-off event would fire at this
  tick's end. Research and Retirement are left out because their actions read the enabled set.
  No state changes, so the fingerprint is unmoved. `104_p3_ideologies_paused.png`: paused on
  day 300, the tile and the left column's Ideology section are up.
- **Retirement modal (P4, #3149).** The store keeps the ended run's last-day view (the
  retiring chunk replayed to its day before; for Retire, the state before the action) and the
  page renders it behind the summary until Continue. After a reload with the summary open, the
  new run shows behind it. The headings read "Run N" and "Run N+1 starts with". Each start row
  shows its change from the ended run's start, saved with the summary.
  `104_p4_summary_backdrop.png`.
