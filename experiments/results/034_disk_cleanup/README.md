# 034: disk cleanup

Where `helium-hustle` spends disk, measured 2026-11-20 by `measure.py` (a Python walk that never
follows a junction or symlink; a link's target counted once, at its real home). Raw numbers:
`before.json`, `after.json`. `rmtree_safe.py` is the delete used: it removes every link under
the target itself before deleting, and stops if it meets one mid-delete.

| Location | Before | After |
|---|---|---|
| main checkout, total | 521.6 MB | 245.5 MB |
| `art/` (shared, local only) | 239.7 MB | 134.4 MB |
| `target/` | 168.5 MB | 0 |
| `web/node_modules` | 76.3 MB | 76.3 MB |
| `.git` | 19.8 MB | 17.5 MB |
| `experiments/` (tracked) | 13.9 MB | 13.9 MB |
| Drive home `gdrive-sync\helium-hustle` | 1.3 MB | 106.6 MB (105.3 MB `archive\`) |
| worktrees folder | 15.6 MB | 31.3 MB (035 started beside, 15.6 MB) |

Storage policy as practiced: generated art lives in the main checkout's `art/` (linked into every
worktree); finished pickers and superseded draw rounds go to the Drive archive at the same
relative path. `gdrive-sync` is a local mirror, so a move there backs up but frees no local disk.

A worktree's own `target/debug` reaches ~680 MB after one `cargo build --workspace` (30 s cold
on 3 jobs): the biggest transient cost, removed with the worktree.
