# 138: the sim players' view and two small UI fixes

## The sim players' view

`Rules::agent_view` (`hh-core/src/rules.rs`) is `view::build_for(r, g, skip = true)`: the one
builder, with the fields no sim player reads wrapped in `view::Shown<T>` and left unbuilt. Those
are `next_run`, `verbs`, `story`, `event_texts`, `achievement_list`, `rivals`, `adversaries`,
`modifiers`, `nav`, `activity`, `breakdown` and `names` (137's list). Built, a `Shown` field
derefs, iterates and serializes as its value, so the full view's JSON and every client are
unchanged. Left out, a read panics naming the field, and serializing fails. A sim player that
starts reading one fails loudly instead of seeing it empty.

`hh-sim`'s `Player::view` builds the agent view. Other `hh-sim` readers keep the full view:
`axes`'s run ends, `trace --at` (`lab.rs`), `run`'s end summary and the hands-off loadout test.

**Drift guards:**

- `hh-core` `the_sim_players_view_is_the_full_view_without_what_they_never_read`: on the golden
  game, the agent view's `Debug` equals the full view's with those twelve fields set to
  `Skipped`. A read panics, and serializing fails.
- `hh-sim` `sim_players_play_the_same_games_on_both_views` (fast lane): the ladder's pinned
  careers (random, greedy, author, casual, handsoff on seeds 1-2) play once on each view. The
  results must match each other and `PINNED`.
- The oracle's slow-lane test plays its second pass on the full view against the same pinned
  fingerprints.
- The full view is a process-wide switch (`agents::set_full_view`). The pin tests in
  `tests/cli.rs` (ladder, hands-off, oracle, the drift test) hold one lock, so the switch never
  reaches their games. `cli.rs`'s other game-playing tests don't take the lock. While the
  drift test runs, their players may read the full view: the same games, but without the
  loud-fail check during that window.

The golden fingerprint, every pin and `RULES_REVISION` (49) are unchanged. Only the content hash
moved (`b2b730caea14ff94`, the 1.1 reward text).

## Speed: skilled `axes`, paired against main

`axes --skills skilled --seeds 3` (9 oracle careers, 6 threads). Base: main `1ef58e4`, built by
`python scripts/baseline.py 1ef58e4eae`. Runs were back to back in two rounds, base then new.
Nothing else ran on the box. `axes9.sh` is the script.

| Round | Base | New | Ratio |
|---|---|---|---|
| 1 | 83.3 s | 70.1 s | 0.84 |
| 2 | 75.9 s | 57.0 s | 0.75 |

Both new reads beat 137's 73.1 s for the same command. Treat the times as upper bounds: base
moved 83 → 76 s between rounds. The tables are identical on both sides and in both rounds, apart
from the per-case `Wall s` column. The 137 profile predicted about 15% (60 µs of a 241 µs view);
the measured gain is 16-25%.

## UI fixes

- **"Short of titanium" with titanium on hand:** not an engine bug. The view's `short` and
  `buy_blocker` agree. The resource panel showed stocks rounded to nearest (`num`), and building
  prices are whole units. So 13.96 titanium read "14" beside a Solar Panel price of 14, and the
  card was right to say short. Under 10,000, stocks now show rounded down to what they display
  (`held()` in `web/src/format.ts`). From 10,000 up, stocks and prices both abbreviate, so a
  stock rounds as its price does: an affordable stock never reads under its price there, but a
  stock just short can read equal to it at three digits. `held()` is used in the resource panel, the Research panel's Science line and the takeover
  strip. Checked in the game: 13.96 titanium reads "13.9" and the card says short. At 14 it reads
  "14", `short` is empty and the card buys (`138_titanium_under.png`, `138_titanium_at.png`).
- **Reloading with a run summary up:** this was already fixed by 124 (the save carries the
  ended run's last state). Checked again here: run 1 retired on day 3 read "Year 1, day 4"
  before and after the reload, and the summary was still up (`138_summary_reloaded.png`).
- **1.1's Reward line** now reads "Buy Science and 20 propellant." (`content/prose.md`, imported
  with `scripts/prose.py`). It shows in the game on the page after 1.1's payoff, with the
  Research tile open (`138_reward_line.png`).

`check.mjs` is the browser check (run it from `web/` against `vite preview`).
