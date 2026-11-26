# Rules history

One row per `RULES_REVISION`, newest first: what changed in play and why. A row stays under
about 400 bytes; rows older than the last ten revisions are trimmed to a line (Matt,
2026-11-22). The detail is in git and each item's write-up.

| Revision | Item | What changed | Why |
|---|---|---|---|
| 37 | 111_act1_unlock_trim_and_run_simulations | 1.4 Science deleted; 1.3 opens Launch Full Pads; Trade (1.4) counts this run's launch pay. The first lab shows Regolith Processing and Liquid Cooling (Run Simulations: 3 energy + 1 ice → 6 credits). Programming from A Voice; Process Telemetry, Accelerated Scheduling `never`. Buy Ice 5. 36 → 37. | Matt (2026-11-26). |
| 36 | 110_boredom_after_burnout_and_parked | Boredom stays at 0 until Burnout (`boredom_bites`, phase 2, once a career) sets the career flag `burnout`: before it the day's growth and positive command boredom are zero. New research Accelerated Scheduling (30 science, after 1.2; career flag `fast_speeds`, opening the UI's 50x and 200x). Nav order: utility tiles last. Bumped from the constant, 35 → 36. | Matt (2026-11-26; amendment #3261). |
| 35 | 106_dome_blueprint_and_archive_discount | The Recreation Dome requires research Recreation Dome Blueprints (Blueprints, 300 science, shown from the first retirement); `first_retirement` no longer enables it. The Blueprint Archive adds `research_discount` ×0.8 for Blueprints bought in an earlier run (the reward's new `category`). Bumped from the constant, 34 → 35. | Matt (2026-11-25). |
| 34 | 103_lab_unlock_and_science_quest | The Research Lab requires Advanced Blueprints, not 2 Data Centers. 1.4 is Science: Research Labs make 50 science this run from its start (new condition `building_produced`; bought science doesn't count), not a second Data Center. The golden script builds its Lab before its second Data Center. Bumped from the constant, 33 → 34. | Matt (2026-11-25): players built the second Data Center only to switch it off. |
| 33 | 095_act1_quest_line | Act 1's line is 1.1-1.7 (Online, First Shipment, Full Load, Processors, Trade from its start, A Voice, Telescope); Blueprints, Burnout (a once-a-career story event) and Land Rush left it; rivals only in Act 2. Side quests give permanent rewards (`modifier`). Staggered unlocks. 32 → 33. | Matt (2026-11-25). |
| 32 | 093_playtest_batch_1 | Blueprints 10/20 science. Trade (`qtrade`: 1,500 trade credits this run) joins the chain; speculators' arrival opens Adversaries (silent event). Lab left the chain: the first Research Lab opens its unlocks silently (flag `research_lab`). Regolith Processing (100 science, excavator ×1.5); excavator 3 → 2 regolith. 31 → 32. | Matt's playtest batch 1 (2026-11-25). |
| 31 | 091_programming_unlocks_editor_only | `run_command` and `by_hand` gone: commands run only as program lines. Programming opens only the Programs view; programs, Add to Program and loadouts are open from day 1. Sims and the script never buy it. Online's story names Add to Program; Processors' body as before 089. 30 → 31. | Matt (2026-11-25). |
| 30 | 090_bug_hunt | `run_command` refuses a command that costs nothing, adds no boredom and has no effect (Idle): it runs only as a program line; the view's `by_hand` hides its Run. A refusal names the input it lacks. Processors' body points to Programming. Bumped from the constant (29 → 30). | By hand Idle was a free credit a click, paused too (090 default, question 090_1). |
| 29 | 089_programming_unlock | Programming research (200 science, after Full Load) sets the career flag `programming` for good (`career_flag` effect; never shown again) and the Programs nav tile follows it; `run_command` runs a command by hand; a pre-29 state past Full Load gets the flag on load; sims and the script buy it. 28 → 29. | Matt (2026-11-24): a new player starts with Commands only. |
| 28 | 086_swarm_clock_after_discovery | The countdown runs once the Telescope quest has first completed (career-long), every later timeline's from its first day, Telescope rebuilt or not (`timeline::located` and the view's `located` gone); retirements move it as before. 27 → 28. | Matt (2026-11-24): rebuilding the Telescope is optional, only for its passive. |
| 27 | 085_swarm_clock_and_retirement_gap | One retirement gap (`career.gap_days`) moves the swarm's countdown; arrivals 36,500 days. 26 → 27. | Matt (2026-11-24). |
| 26 | 084_timeline_scoped_project_effects | Project effects live on completion: the `flag` reward opens `if` only while Conditional Logic is done; Blueprint Archive's `starting_research` replaces `grant_research`; Telescope passive. 25 → 26. | Matt (2026-11-24): a timeline's end takes them; re-funding restores them. |
| 25 | 074_act1_opening_and_story_modals | Act 1 opening: Blueprints, First Shipment (Titanium-only pads, 20 propellant once), Full Load; start builds only Solar Panel and Data Center; Buy Science; Blueprints research gates buildings; Refinery 5 regolith + 2 energy. 24 → 25. | Matt (2026-11-23): one mechanic at a time. |
| 24 | 071_programs_view | Repeat count leads in the text form; `Program.repeats`. | Matt (2026-11-23). |
| 23 | 070_instruction_panel_and_comparison_symbols | Conditions use `>`, `<`, `>=`, `<=` only. | Matt (2026-11-23). |
| 22 | 069_no_cargo_pads | New pads start with No cargo. | Matt (2026-11-23). |
| 21 | 068_bug_hunt_030_deferrals | `Run.events` days; a true 20-day mean; texts. | Matt (030's deferrals). |
| 20 | 067_program_else | Programs get `else`. | Matt (2026-11-23). |
| 19 | 064_general_cleanup | An `if` costs a step; locked lines pass free; the fingerprint leaves out the config. | Matt (2026-11-22). |
| 18 | 063_program_text_editor | Free `note` and `unparsed` lines; `set_program` keeps locked lines; `line_text_max`. | Matt (2026-11-22). |
| 17 | 062_readings_round2 | Capped resources read `amount`; readings `energy_per_day`, `days_since_purchase`. | Matt (2026-11-22). |
| 16 | 059_program_simplify | `wait until` gone; one reading per thing; Full Cluster grants 50 Science. | Matt (2026-11-22). |
| 15 | 049_act1_alignment | Act 1 on the design doc's beats; draft `attention`; `if` from a project. | Matt (2026-11-21). |
| 14 | 046_projects_fixes | Project `scope` replaces `tier`; `unlock` an events condition; `rewards` a list. | Matt (2026-11-21). |
| 13 | 042_draft_whole_arc | Draft Acts 2 and 3 through the Dyson sphere. | Matt's item 042. |
| 12 | 040_golden_program_and_folded_readings | Readings `free processors`, `idle pads`; the golden runs a condition program. | Matt (2026-11-20). |
| 11 | 039_days_since_purchase_and_fund_project | `days since last purchase` reading; `fund` verb. | Matt (2026-11-20). |
| 10 | 038_every_line_costs_a_step | Every non-`if` line costs a step; guard readings. | Matt (2026-11-20). |
| 9 | 037_parameterized_instructions | One flat line template `<verb> <target> [amount]` with the `act` verbs. | Matt (2026-11-20). |
| 8 | 035_program_loadouts_and_rate | Program loadouts (a career library); `rate` is production less demand. | Matt (2026-11-20). |
| 7 | 031_program_conditions_and_build | `build`, `buy land`, `if` and `wait until` program lines; free conditions. | Matt (2026-11-19). |
| 6 | 030_bug_hunt | Unpaid upkeep withholds a building's effects. | Matt (030's questions). |
| 5 | 029_energy_priority_breakdown | Programs are paid before upkeep buildings. | Matt. |
| 4 | 027_art_in_game | `set_pad_good` on a loaded pad returns the cargo. | A parked edit (Matt). |
| 3 | 006_playable_ui | Boredom `base_cap` 500 → 1000; display state. | Matt's Godot cap. |
| 2 | 004_rules_revision1 | The Godot build's Arc 1 rules port. | Revision 1 of the game. |
| 1 | 002_engine_scaffold | First engine: the `placeholder` module. | A stand-in for the engine rules. |
