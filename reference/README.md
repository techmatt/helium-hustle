# reference/

A **read-only snapshot of the old Godot build** of Helium Hustle (GDScript, Godot 4.x), kept as
source material for porting its rules into `hh-core`. Copied **2026-11-14** from
`C:\Code\storage\helium-hustle-godot` at commit `b0de7f4` (2026-04-18); every copied file was
clean at that commit.

**It is not ground truth for the new game.** It records what the Godot build did and what its
design sessions intended, including numbers that were never balanced and systems that were
half-built. Where this folder and `docs/rules/` disagree, `docs/rules/` wins. Don't edit these
files; port from them, and record the new game's rules in `docs/rules/`.

| Path | Source | What it is |
|---|---|---|
| `docs/handoff_*.md` | `docs/` | The Godot build's design handoffs. Start at `handoff_index.md` (overview and file map), then `handoff_core.md` (architecture, conventions, tick and tempo). The rest by system: buildings, programs, economy, demand, progression, narrative, stats, and `handoff_active.md` (status and priorities at the time). |
| `docs/optimizer_design.md` | `docs/` | Design of the old balance optimizer (scenario-based single-lifetime runs). |
| `data/*.json` | `godot/data/` | The game data: resources, buildings, commands, research, projects, events, achievements, and `game_config.json` (global tunables). |
| `scenarios/run1_fresh.json` | `sim/scenarios/` | The optimizer's fresh-start scenario: actions available, objectives, command policy, end condition. |
Not copied: the Godot scripts and scenes, `docs/generate_constants.py`, the top-level `data/`
(the design spreadsheet and its converters) and the logs. The source folder still holds them.

**Screenshots** live outside git, in `C:\Code\gdrive-sync\helium-hustle\screenshots\`:
`godot-buildings-day1427.png` (the latest Godot build mid-career, the layout `web/` follows),
`godot-options-day27.png` (an early Godot shot, Options panel open),
`pyqt-2024-buildings.png` (the 2024 PyQt prototype) and `magical-research-inspiration.png`
(a layout reference from another idle game, not Helium Hustle).
