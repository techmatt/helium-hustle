# Rules history

One row per `RULES_REVISION`, newest first: what changed in play and why. A row stays under
about 400 bytes; rows older than the last ten revisions are trimmed to a line (Matt,
2026-11-22). The detail is in git and each item's write-up.

| Revision | Item | What changed | Why |
|---|---|---|---|
| 48 | 135_play_records_story_format | The first launch's 20 propellant moves from 1.2's `on_start` to 1.1's completion (new `on_complete`: once a career, paid as the quest completes in play; not re-applied by a run, nor by a scenario's starting quests). Quest text: one `story` and optional `payoff`, `--` between panels; no quest `body`. | Matt (2026-11-27): 1.1's reward carries the propellant; the quest text format. |
| 47 | 134_bug_hunt | A run's close raises the career's best ideology ranks and scores to the run's last ones first, so a rank a Recreation Dome's nudge reached between days, then Retire, counts (it was shown in the summary but never recorded). Bumped from the constant, 46 → 47. | A bug (134): a lost rank-5 unlock. |
| 46 | 131_sim_players_overhaul | Rank thresholds round to 2 significant figures (`rank_figures`): 55/140/260/450/730. AI Consciousness Act drops Load Pads and Disrupt Speculators boredom (Cloud Compute ×1.5 only). A tagged building's price rounds to whole units after ÷m. 45 → 46. | Matt's 131 rulings and parked edit P1 (2026-11-27). |
| 45 | 130_ideology_parity_sim | New research Electrochemistry (1,000 science, career flag, shown at 8 Solar Panels) unlocks Chemical Energy; Rationalist major effect removed; AI Consciousness: Cloud Compute boredom 0.15 (×1.5); new global project costs, Foundation Grant 2 Solar, 2 Excavators, 1 Smelter. Tuning: `rank_base_score` 100 → 55, Theorize −3, nudges Dome 2.5, Dream/Theorize 0.5. 44 → 45. | Matt's 130 rulings and amendment #3897; ideology parity. |
| 44 | 129_boredom_ideology_redesign | One modifier per axis, m = 1.1^rank, replaces every per-rank bonus and discount: tagged buildings cost ÷m; each axis's boredom tool ×m (Dome cap and new relief, Dream, new Theorize), and using it nudges the axis. Boredom ×1.25 a year past its last phase. Sell refunds half the average paid. 1.c, Predictive Maintenance cut. 43 → 44. | Matt's boredom/ideology redesign; parked P1, P3, P6. |
| 43 | 119_no_partial_purchases | No partial purchases: every command execution needs its full cost or fails (nothing paid or made, its step spent, the short noted); a command whose output would pass a cap (held + output > cap, as the line runs) is skipped. Loading still tops pads up partially. Bumped from the constant, 42 → 43. | Matt's parked edit (2026-11-25); staging rulings (amendment #3522). |
| 42 | 117_parked_batch_research_reveal_ideologies | Research groups reveal by this run's triggers (new row key `reveal_when`, binding even for items bought before): Self-Maintenance, Extraction and Recreation Dome Blueprints once a Research Lab is built this run, Market Analysis once speculators are present this run. Electrolysis Plant 1 propellant a day (was 2). Bumped from the constant, 41 → 42. | Matt's parked edits (2026-11-26): run 2+ unfold like run 1; propellant too easy. |
| 41 | 116_parked_batch_projects_panel_adversaries | Project funding cap: 30 credits and 10 of each other resource a day (projects params), replacing derived `min_days` (the telescope's 3,000 stays). Full Cluster (1.b) 12 research items. Arbitrage Engine requires Market Awareness. Speculators' and rivals' arrivals are popup events. Bumped from the constant, 40 → 41. | Matt's parked edits (2026-11-26). |
| 40 | 114_a_voice_projects_and_program_lights | A Voice's start opens Projects with Blueprint Archive (2,000 credits + 300 science), Conditional Logic, Lunar Cartography (1,500 + 200) and Foundation Grant (10,000 + 2,000); 1.c hangs beside A Voice. Liquid Cooling 200 science. Display: a per-day step tally per program (tab lights). Bumped from the constant, 39 → 40. | Matt: projects at A Voice, credits plentiful by then. |
| 39 | 113_handsoff_loadout_research | Boredom Rising (phase 2) doesn't fire in the run Burnout fires in (new condition `event_not_fired`); from later runs it is the phase-2 notice as before. Bumped from the constant, 38 → 39. | Matt's parked edit: both fired at phase 2. |
| 38 | 112_right_panel_events_and_popups | Buy Regolith and Propellant 10 credits; Launch Pad 20 titanium; Data Center upkeep 4, Solar Panel 6 energy. | Matt's parked edits (2026-11-26). |
| 37 | 111_act1_unlock_trim_and_run_simulations | 1.4 Science deleted; Trade counts this run's launch pay; the first lab's research; Programming from A Voice. | Matt (2026-11-26). |
| 36 | 110_boredom_after_burnout_and_parked | Boredom stays at 0 until Burnout; research Accelerated Scheduling (50x, 200x). | Matt (2026-11-26; amendment #3261). |
| 35 | 106_dome_blueprint_and_archive_discount | The Dome needs Recreation Dome Blueprints; the Blueprint Archive discounts earlier-run Blueprints ×0.8. | Matt (2026-11-25). |
| 34 | 103_lab_unlock_and_science_quest | The Research Lab requires Advanced Blueprints; 1.4 Science (50 science from Labs this run). | Matt (2026-11-25). |
| 33 | 095_act1_quest_line | Act 1's line is 1.1-1.7 (Online, First Shipment, Full Load, Processors, Trade from its start, A Voice, Telescope); Blueprints, Burnout (a once-a-career story event) and Land Rush left it; rivals only in Act 2. Side quests give permanent rewards (`modifier`). Staggered unlocks. 32 → 33. | Matt (2026-11-25). |
| 32 | 093_playtest_batch_1 | Playtest batch 1: Blueprints 10/20, Trade in the chain, Lab off it, Regolith Processing. | Matt (2026-11-25). |
| 31 | 091_programming_unlocks_editor_only | `run_command` and `by_hand` gone: commands run only as program lines. Programming opens only the Programs view; programs, Add to Program and loadouts are open from day 1. Sims and the script never buy it. Online's story names Add to Program; Processors' body as before 089. 30 → 31. | Matt (2026-11-25). |
| 30 | 090_bug_hunt | `run_command` refuses a free no-effect command (Idle). | By hand Idle was a free credit a click. |
| 29 | 089_programming_unlock | Programming research sets the career flag `programming`; the Programs tile follows it. | Matt: a new player starts with Commands only. |
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
