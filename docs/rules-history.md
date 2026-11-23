# Rules history

One row per `RULES_REVISION`, newest first: what changed in play and why. A row stays under
about 400 bytes; rows older than the last ten revisions are trimmed to a line (Matt,
2026-11-22). The detail is in git and each item's write-up.

| Revision | Item | What changed | Why |
|---|---|---|---|
| 24 | 071_programs_view | Text form: a command's repeat count leads (`4x sell cloud compute`); a trailing number is always an argument, the old trailing count reads red. Display state `Program.repeats` (each repeated line's pass: runs done and failed) for the gutter; a Programs nav tile (content). Play unchanged. Bumped from the constant (23 → 24). | Matt (2026-11-23): Programs view, Text and Blocks first-class, repeat counts lead, repeat progress. |
| 23 | 070_instruction_panel_and_comparison_symbols | Conditions are `<reading> <op> <number>`, `op` one of `>`, `<`, `>=`, `<=` (new `cmp` values `ge`, `le`); text reads and writes only symbols (no `above`/`below`/`over`/`under`; `>=` no longer reads as `>`); game texts drop the two words. Saved programs play the same. Bumped from the constant (22 → 23). | Matt (2026-11-23): conditions use symbols. |
| 22 | 069_no_cargo_pads | A new pad (mid-run or at a run's start) has No cargo: idle, Load skips it, `idle_pads` counts it, `set pads` switches it; `set_pad_good` takes null for No cargo. Quest 1.2 teaches it. Sim players and the golden script pick He-3. Bumped from the constant (21 → 22). | Matt (2026-11-23): players must learn to pick a pad's cargo. |
| 21 | 068_bug_hunt_030_deferrals | Display only, no play change: `Run.events` records each event's day (Story by day); the 20-day average is a true mean over the run's days so far; texts: Q1 names Sell Cloud Compute, overclock `{days}`, Ideologies "the next day". Bumped from the constant (20 → 21). | Matt (030's deferrals): state moved the fingerprint. |
| 20 | 067_program_else | Programs get `else` (JSON `{"op":"else"}`): a free line after an `if` or its body, its indented lines run when that `if` was false; a locked `if` passes both bodies. Old programs play the same. Bumped from the constant (19 → 20). | Matt (2026-11-23): one decision per `if`, both branches, no complementary guard. |
| 19 | 064_general_cleanup | An `if` costs a step; a false one jumps its body in it. A locked line passes free (a locked `if` with its body). The fingerprint leaves out the config (`GOLDEN_CONFIG_HASH`). Bumped from the constant (18 → 19). | Matt (2026-11-22): one step per `if`, easy to follow at 1×; locked lines free; a no-play config key moves no pin. |
| 18 | 063_program_text_editor | Free `note` and `unparsed` lines. `set_program` keeps lines not unlocked this run (they fail or read false when run); new param `line_text_max` (200). | Matt (2026-11-22): a text editor whose red and amber lines are kept, so a loadout with lines not open yet still loads. |
| 17 | 062_readings_round2 | A capped resource (and boredom) reads its `amount` as well as its `percent`. Researched readings `energy_per_day` and `days_since_purchase`, opened by Process Telemetry (50 science, `program_readings`). | Matt (2026-11-22): exact amounts, an energy rate against brownout deadlocks, a stale-run guard for `retire`. |
| 16 | 059_program_simplify | `wait until` gone; a condition is `<reading> <above \| below> <number>`; one reading per thing (`percent`, `amount`, `free_processors`, `free_land`, `idle_pads`). 1.b's reward: every run starts with 50 Science (`grant`). | Matt (2026-11-22): `if` reads only what is generally useful and covers `wait until`. |
| 15 | 049_act1_alignment | Act 1 follows the design doc's beats (quest chain 1.1-1.9, side quests 1.a, 1.b); a draft `attention` module; retirement keeps ideology scores through a simulated gap; `if` opens with the Conditional Logic project; the golden game starts from `golden_start`. | Matt (2026-11-21): Act 1 draft-playable and aligned with the design doc. |
| 14 | 046_projects_fixes | Project `scope` (`run` / `timeline` / `forever`) replaces `tier`; `unlock` is an events condition; `rewards` a list, optional `repeat`; per-row `min_days`; Microwave Receiver and Fuel Cell Array require their project. | Matt (2026-11-21): the scopes coming designs need; one home for conditions. |
| 13 | 042_draft_whole_arc | Draft Acts 2 and 3 through the Dyson sphere. | Matt's item 042. |
| 12 | 040_golden_program_and_folded_readings | Readings `free processors` and `idle pads`; the golden game runs a condition program once program logic opens. | Matt (2026-11-20): the golden pins the interpreter; exact guards from folded readings. |
| 11 | 039_days_since_purchase_and_fund_project | A `days since last purchase` reading and a `fund <project> [N \| all]` verb, each with a switch param (`days_since_purchase_reading`, `fund_verb`). | Matt (2026-11-20): the hands-off player's last two hands become program lines. |
| 10 | 038_every_line_costs_a_step | Every non-`if` line costs a step; one with nothing to do is `done`. New guard readings (`researched`, `pads`, `running`, `processors`). | Matt (2026-11-20): a program a player can follow at speed. |
| 9 | 037_parameterized_instructions | One flat line template `<verb> <target> [amount]` with the `act` verbs. | Matt (2026-11-20). |
| 8 | 035_program_loadouts_and_rate | Program loadouts (a career library); `rate` is production less demand. | Matt (2026-11-20). |
| 7 | 031_program_conditions_and_build | `build`, `buy land`, `if` and `wait until` program lines; free conditions. | Matt (2026-11-19). |
| 6 | 030_bug_hunt | Unpaid upkeep withholds a building's effects. | Matt (030's questions). |
| 5 | 029_energy_priority_breakdown | Programs are paid before upkeep buildings. | Matt. |
| 4 | 027_art_in_game | `set_pad_good` on a loaded pad returns the cargo. | A parked edit (Matt). |
| 3 | 006_playable_ui | Boredom `base_cap` 500 → 1000; display state. | Matt's Godot cap. |
| 2 | 004_rules_revision1 | The Godot build's Arc 1 rules port. | Revision 1 of the game. |
| 1 | 002_engine_scaffold | First engine: the `placeholder` module. | A stand-in for the engine rules. |
