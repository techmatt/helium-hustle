# 124: parked UI batch (research bar, compact building cards, nav order, project popups) and three fixes

No rules change: the golden fingerprint is unchanged (`e2e48fc08ede4792`); only the content
hash moved (nav.json's order).

- `check.mjs`: the browser check of every done-when item against `vite preview` (all ok;
  `facts.json` has what it read). Run from the repo root with `HH_URL` set.
- `cards.mjs`: the Buildings view's card heights, every card shown (`?all-cards`, scenario
  `everything`). Before: base 8ef8d0d's web built in `out/` (same wasm); after: this branch.

| | median card | shortest | tallest | Buildings view |
|---|---|---|---|---|
| before (8ef8d0d) | 339 px | 235 px | 391 px | 3,941 px |
| after (124) | 172 px | 172 px | 208 px | 2,312 px |

Why the cards were tall: a second global `.card-body` rule (Story's cards, `flex-direction:
column`) overrode the building and project cards' row, so their flows sat under the picture.
Story's is now `.story-body`; project cards get their picture beside the text again too.

Screenshots (also in the Drive screenshots folder): `124_nav`, `124_research` /
`124_research-unaffordable`, `124_buildings-before` / `124_buildings-after`,
`124_project-popup`, `124_project-events`, `124_jump-refused`, `124_repeat-skipped`,
`124_summary-before-reload` / `124_summary-after-reload`.
