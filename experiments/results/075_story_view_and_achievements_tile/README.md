# 075: Story as a quest timeline, Achievements as its own tile

Screenshots in `shots/` (also in the shared screenshots folder as `075_*.png`), made by
`shots.mjs` against a `vite preview` of a static build: a new game played by the debug
autopilot's *Next quest step* through 1.6, then Story at the top (the Act 1 band) and at
the active beat, and the Achievements panel, in both themes.

- `story-top-*.png`: the Act 1 band, finished beats lit with their story and payoff text.
- `story-active-*.png`: 1.6 lit, side quest 1.a (started) branching off it, 1.7 Burnout
  glowing with its task and bar, one dim "???" below.
- `achievements-*.png`: badges by category, earned lit and unearned dim, each with its reward.

The autopilot never stops with 1.6 active: its step from 1.5 also completes 1.6 (the
speculators are already present when 1.5 finishes) and stops at 1.7 Burnout, the next
quest needing a retirement. e2e section 11 checks the timeline wherever that step stops
(1.6 or 1.7).
