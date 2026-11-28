# 133 cleanup: working papers

**`hh-sim axes` on the shared driver (`career::drive`) plays the same.** Both sides are
release `hh-sim` builds from this branch, one before and one after `axes` moved onto
`drive`. The new `x@day` column was already in both.

- `axes_before.txt` / `axes_after.txt`: `axes --skills medium --seeds 2 --runs 3`, six
  cases. Identical apart from the wall-time column.
- `skilled_before.txt` / `skilled_after.txt`: `axes --skills skilled --axes
  humanist,rationalist --runs 1`, seed 1. Identical apart from wall time. They show the new
  per-run project days (`ba@1255 lc@1345`).

`play_session` (the ladder, `trace`, every pinned game) went onto `drive` in the same change.
The pinned games in `crates/hh-sim/tests/cli.rs` (`PINNED`, `HANDSOFF_PIN`, the oracle's) and
the ladder pin all pass unchanged under `cargo test-all`, so that side plays the same too.

These are seed-1/2 equivalence checks, not balance reads, so no seeds were drawn from the
ledger.
