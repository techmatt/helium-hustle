# 024: parked UI edits

Five small UI edits, in 023's tokens and styling. The e2e (section 5) checks each one, and a
1280 `npm run qa` pass (seeds 11 to 13, both themes) finds nothing.

| Shot | Shows |
|---|---|
| `p1-p2-p3-land-groups-capped-energy-light.png` | P1: the Land row affordable (green outline, `ok-head` tint, like an affordable card's head). P2: the two Resources groups. P3: capped Energy's `+98 CAP` in green, in Resources and the status bar. |
| `p1-p2-dark-buildings.png` | Dark theme: the Land row when land is *not* affordable (100 credits < 200: plain), the groups, capped Energy green. |
| `p2-p4-1280-light-late.png`, `p2-1280-dark-late.png` | The groups at 1280 late in run 2, with eleven resources shown: the column still fits. |
| `p4-resources-green-light-2x.png` | P4: the light Resources panel at 2x; `+1/day` in the new `--pos-text`. |
| `p5-story-latest-boredom-phase.png` | P5: past day 500 Story lists only *Boredom Accelerating* (phase 3); Recent Activity keeps both day 200 and day 500. |

- **P1:** `.land-bar.can-buy` when `view.can_buy_land`; the border was already 1 px, so the
  row's box doesn't change.
- **P2:** the order is a display list in `web/src/ui/Left.tsx` (`FIRST_GROUP`), two
  `<tbody>`s in one table so the columns stay aligned, a rule between them.
- **P3:** `CappedRate`'s `spare` flag, set for Energy only (Resources and the status bar).
- **P4:** a new token `--pos-text`: light `#166235` (7.10:1 on the panel, was `--pos`
  `#1f7a42` at 5.13:1), dark the same as `--pos` (9.55:1). `contrast.mjs` takes a per-pair
  minimum and holds `pos-text` on `panel` to 7:1. Only the Resources table's green and
  capped Energy use it; `--pos` (borders, card lines) is unchanged.
- **P5:** in the view (`hh-core` `view.rs`), not the client: `event_texts` keeps only the
  highest fired boredom phase's text. Display only: no state, fingerprint or replay
  change, so no `RULES_REVISION` bump. Stats' *Events this run* reads the same list.
- **Also:** a Resources rate too small to show (`0/day`) is no longer colored (late
  Credits read a green `0/day`).
