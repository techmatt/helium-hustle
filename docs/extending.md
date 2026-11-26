# Extending the engine

What a new piece of the engine touches, so nothing is missed: each list names the files
and the functions in them. Paths are under `crates/hh-core/src/` unless named otherwise;
`hh-sim`'s are under `crates/hh-sim/src/`, the web's under `web/src/`. A rules change
also follows README, *Rules changes* (the golden game, `RULES_REVISION`).

**A new module** touches its `modules/<id>.rs`, `modules/mod.rs` (`mod`, `REGISTRY`, `Params`),
`configs/baseline.json` and a `docs/rules/<id>.md` page whose `<!-- params:<id> -->` block
`hh-sim params --write` fills; a new event condition (quests and project unlocks share
them), `content.rs` (`Condition`, `condition_ok`), `events.rs` (`holds`, `progress`, `kind`),
`view/projects.rs` (`condition_text`) and `hh-sim`'s `agents/progress.rs` (`condition`).

**A new program reading** touches `state::Reading` (`describe`, and `researched` for a
research-gated one), `modules/programs.rs` (`check_line`, `reading`; `locked` for a gated
one), `program_text.rs` (its text both ways), `view/programs.rs` (`readings`, the editor's
list), the web
`Reading` type (`engine.ts`) and `readingLabel` (`ui/ProgramLogic.tsx`), and
`docs/rules/programs.md`; `npm run check` is the only check on the web two.

**A new program line kind** (as `else`, 067) touches `state::Op` (`describe`),
`modules/programs.rs` (`check_line`, `Block` if it opens or joins a block, `locked`, `step`),
`program_text.rs` (`op_text`, `parse_op`), the web `Op` (`engine.ts`), `ProgramLogic.tsx`
(`lineText`, `LogicBody`, `AddLogic`), `ProgramLines.tsx`'s row (both the right column and
the Programs view's Blocks mode), `programEdit.ts` (the block shape, with its
checks in `web/scripts/unit.mjs`), the run gutter (`Gutter.tsx` `entryStatus`), and
`docs/rules/programs.md`.

**A new comparison** touches `state::Cmp` (`symbol`), `program_text.rs` (`clause`'s
`cmp_of`), `modules/programs.rs` (`clause_holds`), the comparison loop in `crates/hh-core/src/tests/logic.rs`,
the web `Cmp` (`engine.ts`) and `CMPS` (`ui/ProgramLogic.tsx`), the form line in
`agents/loadout.rs`'s doc, and `docs/rules/programs.md` *Conditions*.
