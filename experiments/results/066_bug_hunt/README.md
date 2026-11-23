# 066: bug hunt (recent changes since 030)

The findings and their fixes are rows 78-87 of [docs/qa-findings.md](../../../docs/qa-findings.md);
the checker's new probes are in [web/TESTING.md](../../../web/TESTING.md), *QA checker*.

Screenshots (also in the shared screenshots folder):

- `066_grammar_probe.png`: the text editor holding a program in every form the grammar
  writes (qa's takeover probe); the one amber line is a reading Process Telemetry gates.
- `066_story_over_text_editor.png`: a story banner shown over the text editor (row 78), in
  run 2 after a forced retirement with an unapplied draft, which the editor kept (row 87).
- `066_late_left_column_vault.png`: a finished career (the autopilot's *Everything*) at
  1280×800: 16 tiles and the calendar, the Resources rates whole beside the scrollbar
  (row 85), the Vault's bar at its own width (row 86). Taken with an interim fix that
  reserved the scrollbar's gutter; the shipped fix widens the column by the same 17 px.
