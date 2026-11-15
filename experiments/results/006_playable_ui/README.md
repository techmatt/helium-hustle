# 006 playable UI v0: decisions

What the item built and why, beyond what `web/README.md` and the rules pages say.

## Engine (rules revision 3)

- **Boredom cap 1000** (parked P2): `boredom.base_cap` 500 → 1000. The golden game's forced
  retirement moved from about tick 1100 to tick 1574; a run with no command boredom reaches
  the cap on day 1670. The balance judge now reads run 1 at 1334 ticks for author
  active/mixed (target 800-1100): see the report's warnings.
- **Display state rides the bump.** Recent Activity, line colors and the run summary need
  facts the view could not derive: what shipped, burst or dumped on which day; what each
  program line did today; how the last run ended. They are state (`run.activity`,
  `run.today.lines`, `career.last_run`), so they move the fingerprint, but no rule reads
  them. The feed is capped by a config param (`events.activity_kept`, 50) so long runs
  don't grow the state; the cap is display-only but lives in the play config because every
  number does. A record from revision 2 no longer loads (its config lacks the param); any
  new required param does that.
- **Cheats are actions** (`Action::Cheat`): grant (up to the cap), set boredom (up to the
  cap), jump ahead (a marker; the days pass through `advance`). Any cheat sets
  `GameState::cheated`, serialized only when true, so an honest game's JSON and
  fingerprint are unchanged by the field.
- **Nav tiles are content** (`content/nav.json`): `always`, `panel` (enabled by an event this
  run; seen events re-enable at run start) or `building_known` (owned now or in any run).
  Launch Pads and Research use `building_known`, not the Godot handoff's "this run only":
  the ruling says known things stay visible across retirements. Content checks every
  `enable_nav_panel` has a tile.
- **The view grew** (no rules in the client): descriptions, ideology tags, what is short,
  land per purchase and whether Buy Land applies, quest progress, quest and event texts,
  achievements, rivals, modifiers, overclocks, rank thresholds, the next run's bonuses if
  retired now (`career::retire` on a copy), the last run's summary, display names.
- **Resource visibility** now also shows a resource bought by a command (Buy Propellant)
  or held now. Before, Propellant (needed for every launch) stayed hidden until an
  Electrolysis Plant, so a player could not see their fuel (found by the diff review).
- `hh-wasm` gained `advance_run`: like `advance` but it stops after a tick that retired the
  run, so the summary shows before run 2 plays on.

## UI

- **Preact** (11, ~4 KB) for keyed rendering: a hand-rolled `innerHTML` render would
  rebuild buttons every tick and drop clicks at 200x. Handlers that edit a list (program
  lines, active counts, project rates, loading priority) read the engine's view at click
  time, not the render's: two quick clicks both land (an e2e check).
- **Exit tile dropped**: a browser tab has nothing to exit to. 11 tiles.
- **Projects tile** appears with Q5 (Market Awareness), not Q3 as the Godot handoff says:
  the content (and so the rules) enables it there.
- **Retirement summary** pauses the clock and survives a reload; Continue restores the
  speed before it. A click whose owed ticks end the run is refused with a notice rather
  than applied to the new run.
- **"New" markers** follow the Godot handoff: per run, from a baseline at run start or
  page load, cleared on hover (cards) or open (tiles). A tile also gets a dot when its
  panel holds something new.
- **Story banner**: a quest that opens or a one-off event that fires shows its text above
  the panel (Godot's EventModal, but not blocking). Placeholders like `{rate}` are shown
  as ported (the narrative ruling).
- Rules texts in the UI are kept generic (Ideologies says ranks multiply bonuses and
  discounts, not which), so they can't go stale against `docs/rules/`.

## Play logs (amendment #169)

- **Format:** JSON lines, one file per career, header first (`hh-playlog/1`: revision,
  config hash and the config, seed, scenario, commit), then timestamped lines whose
  `action`s are exactly the game's record. Replay needs nothing but the file; the
  fingerprint at the last checkpoint proves it. `hh-sim playlog --json` prints the same
  `GameResult` an agent's career writes to `games.jsonl`, so a later item can put a human
  row beside sim rows on the same seed with the ladder's own `summarize`.
- **Transport:** the dev server appends (`/__playlog`), not localStorage, so a discarded
  save's career survives as a closed file. Sends are serialized (one request in flight);
  the closing beacon may overtake one, so the reader orders lines by `t`.
- **Measuring:** the per-tick measuring moved out of `career::play_session` into
  `career::Meter`, which `playlog` reuses. Checked: the ladder (135 games) gives
  identical `games.jsonl` before and after, wall time aside.
- **Real time per tick:** the log's points (any line) give (wall time, tick); a tick is
  reached at its share of the time between the points around it, so a pause's time lands
  on the tick after it; closed time between sessions is left out.
- **Bot logs apart:** a browser under automation gets `bot_` careers, filed in
  `out/playlogs/`, so `C:\Code\gdrive-sync\helium-hustle\playlogs\` holds only people's
  play. The done-when check (3 minutes of a fresh game, scripted) wrote there before that
  split; its log was moved to the kept outputs (`006_playlog_sample/`) with its report.
- **Limits:** a crash loses at most the last 2 s of lines (the replay then stops at the
  last line; its fingerprint check needs a checkpoint at that tick, logged every 10 s and
  on hide, close and retirement). A static build has no log server and records nothing.

## Where the screenshot and the rules differ

- Screenshot shows Exit: dropped (above). Screenshot's ideology tags show on every card:
  here they show once Ideologies is unlocked (the Godot handoff's rule).
- Screenshot's Land bar reads "52 / 60 used": the view gives free land and the price, so
  the bar shows "N free" and the offer, with no total computed in the client.
