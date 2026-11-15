# Rules history

One row per `RULES_REVISION`, newest first: what changed in play and why.

| Revision | Item | What changed | Why |
|---|---|---|---|
| 3 | 006_playable_ui | Boredom `base_cap` 500 → 1000 (+100 per Recreation Dome unchanged): a run with no command boredom now reaches the cap on day 1670, not 1170. Display-only state, which plays no differently: a run activity feed (`activity_kept` newest lines), each program line's outcome today, the last finished run's summary, and the cheat mark; cheat actions (grant, set boredom, jump ahead) for the debug menu. Bumped from `RULES_REVISION` 2. | The cap Matt played in the Godot build (a parked edit). The display state is what the playable UI shows (Recent Activity, line colors, the run summary); it moves the fingerprint, so it rides this bump. |
| 2 | 004_rules_revision1 | The Godot build's Arc 1 replaces `placeholder`: 12 modules (economy with multi-pass building resolution, boredom, flat Dream, programs and commands, shipments, demand with speculators and rivals, research, ideology, projects, events and the quest chain, achievements, career and retirement), content converted from `reference/data`, boredom cap 500 + 100 per Recreation Dome, exact whole powers (`Fx::pow_int`) for cost curves and per-rank factors. The golden game is now the scripted policy. Bumped from `RULES_REVISION` 1. | Revision 1 of the game: the playtested Arc 1 rules, as data and handoffs last specified them ([rules/](rules/README.md), conflicts in each page's *Port notes*). |
| 1 | 002_engine_scaffold | First engine: the `placeholder` module (credits, energy with a cap, Solar Panel bought with credits, daily sunlight draw). | A stand-in to exercise config, content, `Fx`, RNG, replay and native = wasm; the rules port replaces it. |
