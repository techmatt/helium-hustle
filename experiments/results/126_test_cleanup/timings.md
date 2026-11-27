# 126: test timings

This machine, otherwise idle, one browser at a time, Nov 27 2026 (1am). Wall clock.

| Lane | Command | Before | After |
|---|---|---|---|
| Rust fast | `cargo test --workspace` (built) | 6.4 s, 5.9 s | 6.9 s, 6.6 s (no Rust change: noise) |
| Rust full | `cargo test-all` (built) | 46.4 s | 45.2 s (no Rust change; hh-sim's lib, the autopilot career, is the whole of it) |
| web fast | before `npm run check && npm run unit`; after `npm test` | 2.6 s | 1.3 s, 1.2 s |
| e2e | `HH_URL=... npm run e2e` | 4:21, passed (script unchanged, measured once) | same |
| qa | `HH_URL=... npm run qa` (defaults: seeds 11,12,13, widths 1280,1920) | 36:04 (6 passes, 0 findings) | 19:44 (3 passes, 0 findings) |

qa runs were timed by `out/126_test_cleanup/qa-run.sh` (every line stamped; wall from
start to exit), against `vite preview` of the same build.

## qa, per pass

Before: six passes of 6:05, 6:00, 6:00, 6:02, 5:58, 5:59 (249-255 moments each).

After, seed 11's pass (played at 1280 light, also captured at 1920), phase starts:

| Phase | Starts at |
|---|---|
| run 1 (fresh game to Programming, 3x then 10x) | 0:00 |
| run 1's end probes | 2:53 |
| retirement, summary | 2:56 |
| run 2 (150 days at 10x), reload, replay | 3:10 |
| the late panels with cheats, other theme, forced retirement | 3:28 |
| the debug scenarios | 3:46 |
| the autopilot's Everything, the later acts, second tab, site data | 5:18 |
| every building's card, both themes | 6:18 |
| done | 6:36 |

Captures: 29 s at the played width, 45 s at the other (a resize, two frames, the audit and
a screenshot, then back).

After, the three passes: 11 at 1280 light (+1920) 6:36, 12 at 1920 light (+1280) 6:33,
13 at 1920 dark (+1280) 6:34; 246-252 moments each, at both widths; captures 29-31 s at
the played width, 41-45 s at the other. Each pass's only finding, as before, is the
known won't-fix slow frame at the autopilot's ending (1.3-1.5 s).

A first after run, which also flipped the theme at the other width by setting
`data-theme`, gave 68 contrast findings in pass 1, all false: the page sets some colors
from its theme listeners, so the audit read dark text on light backgrounds. It was stopped
after pass 1; the other width now keeps the screen's theme.
