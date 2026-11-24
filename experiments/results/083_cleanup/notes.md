# 083 cleanup: each audit note against the code (Nov 24, 2026)

One row per note (repeats merged). Verdict: what the code said at 2803626; action: what 083 did.

| Note (item) | Verdict | Action |
|---|---|---|
| Hands-off loadout baked in, a sweep needs a rebuild per variant (065) | true | `handsoff:loadout=<path>` knob, native only (`agents::build`, `read_loadout`); docs/experiments.md recipe, docs/agents.md spec line |
| Bridge guards `titanium < 225%` always true (065) | true | `build_order_program` leaves the clause out when the percent is over 100 (pins unchanged) |
| `load_pads` command vs `load pads all` verb in `hand_plan` / `standing_cycle` (065) | true | deferred: changes play (energy, steps, pre-logic loading) and moves pins |
| "Steps a day = processors" stated nowhere (065) | true | one sentence in docs/rules/programs.md *Processors* |
| `trace --at` prints no pads (065) | true, wrong file (it's `lab.rs` `state_at`, not experiment.rs) | pad lines added; `trace_at_prints_the_state_at_a_tick` checks one |
| Judge can't show sim-player items; boredom retirements (065) | design | deferred |
| Web `programEdit.ts` mirrors `programs::Block` by hand (067) | true | deferred (exposing it needs a wasm entry and makes the module impure) |
| `line_060` / `clause`, 060's reader kept for one test (067, 070) | true | removed; `the_loadout_files_parse` keeps the six files read |
| hh-sim reads `PadView.good` directly (069) | true | left: seven reads in five forms, an accessor saves nothing |
| No index of where each sim player acts (069) | true | index in `agents/mod.rs` above `RUNGS` (docs/agents.md is at its size margin) |
| e2e and qa duplicate the first-shipment loop (069) | partly: the loops differ on purpose | identical pieces shared in `cdp.mjs`; each loop names the other |
| Scenario pads key, old-save migration (069) | design | deferred |
| Comparisons checklist (070) | true | README *A new comparison* |
| `Instructions.tsx` builds a full View per call (070) | true | deferred (perf, bounded at one call a second) |
| qa-findings lists the fixed "if held" flake as open (070) | wrong: it is in the fixed table (row 94) | web/TESTING.md's flake line was the stale one: fixed |
| over/under aliases, instruction panel before logic (070) | design | deferred |
| `ProgramLines` / gutter / insertLine moves (071) | done in 071 | none |
| e2e one-section runner (071, 081) | true | `HH_ONLY` in e2e.mjs |
| e2e failures don't print `#notice` (071) | true | printed on a thrown failure and by `checkAct` after acts |
| e2e nav loop assumes panels (071) | already handled (comment, `programs` filtered) | none |
| qa.mjs still says "text editor" (071) | partly | labels and misplaced JSDoc fixed |
| Programs nav icons not generated (071) | true | deferred: low value, output not byte-identical |
| README: write edit scripts with Write (071) | true | README *This machine* bullet |
| web/README: which component is the block editor (071) | partly | web/README takeover paragraph names `BlockEditor`, `ProgramLines.tsx`, `ProgramLogic.tsx` |
| Programs mode in localStorage; Blocks edits apply at once (071) | design | deferred |
| Opening hand copied in driver.rs and agents (074) | true, and the copies differ | cross-reference comments both sides; merging moves the golden or the pins: deferred |
| Quest gates spread over four mechanisms (074) | true (six places) | `hh-sim gates` lists what each quest opens; pointers in docs/arc.md and docs/simulator.md |
| Scenarios asserted on by tests (074) | true | README paragraph; docs/architecture.md *Scenarios* names the dependents |
| `cargo test-all -p hh-sim` trap (074) | fixed by 081 (xtask) | none |
| store.ts duplicate clock hold (074) | partly (only the stop is duplicated) | `holdClock()` |
| `set_golden.py` silent under a foreground timeout (074) | true | README step says to background it |
| hh-sim usage looked up twice (074) | true; `--help` was an error | `hh-sim --help` (stdout, exit 0); docs/simulator.md |
| Which lane writes the sim pins (074) | partly | README names `cargo test-all -p hh-sim` and `out/hh-sim-tests/` |
| `content.rs` `check()` validity matches (074) | true | `Requirement`, `Visibility`, `Unlock` `ids_ok` methods, exhaustive (no `_ => true`) |
| `Requirement::Quest` vs `QuestCompleted` (074) | true; no content row uses `quest` | renamed `QuestReached`, JSON tag still `quest` |
| docs/rules/README content counts stale (074) | already right now | `the_rules_readme_counts_the_content` test keeps them right |
| Author rewrite resets the pointer (074) | true | deferred: strategy |
| 074 design calls (1.b, 1.4 pace, story pause log, story save, Research Lab gate) | design | deferred |
| Gutter else status decided twice (081) | true (outputs differ only on missing reasons) | `noteStatus` / `flaggedStatus` in Gutter.tsx |
| e2e `freshLogicGame` (081) | true | helper, used by sections 7-10 |
| CLAUDE.md: web-only commits (081) | true | CLAUDE.md *Test lanes* line |
| Progress ETAs; parked rulings (081) | conductor/planner side, not the repo | none |
