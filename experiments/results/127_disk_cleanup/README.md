# 127: disk cleanup

Second pass after 034. Every size is a Python walk that never follows a link (`measure.py`,
034's walk taken deeper): `before.json`, `after.json`. `caches.py` sizes the shared tool caches
(every project's).

- `git gc --prune=now` + `git worktree prune` on the shared `.git`: 162.4 → 70.7 MB. The
  2314 loose objects left after a plain gc were unreachable pre-rebase copies of item commits
  (conductor rebases each branch onto `main`, so the originals dangle for gc's two weeks).
- `archive_art.py` moved, with `os.rename` after checking for links and existing
  destinations, the six finished pickers (`art/pickers/*`, 76 MB: every one has shipped,
  078/080/100/119) and the 20 rejected redraws (`*.try<n>.png`, 36 MB) to
  `C:\Code\gdrive-sync\helium-hustle\archive\art\` at the same relative paths.
- Kept: the main checkout's `target/` (193 MB). It is not stale: `target/wasm32-unknown-unknown`
  and the host build scripts in `target/release` are the wasm build cache conductor's Run game
  reuses (last written 12:47am on Nov 27), so a delete buys one cold build and it regrows.
