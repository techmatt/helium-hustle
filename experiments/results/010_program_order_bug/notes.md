# 010: programs reorder as they run

**Cause (display only):** the clock wakes every 100 ms and hands over `speed / 10` days at
once, and the program panel drew the pointer (▶) and "ran today" colors of only the last
of those days. Above 10x, each redraw moved the ▶ by (days per redraw) mod (queue length),
so it hopped around the queue and often stepped backward. The engine runs lines in queue
order, and the queue never reorders (engine, view, UI keys, CSS and drag paths all checked).

Each redraw's ▶ line index and lit lines, read by a MutationObserver on the program list.
Queue: cloud_compute, buy_regolith, buy_titanium, each x1, 1 processor, static build.

| Speed | Before (▶ per redraw) | After |
|---|---|---|
| 10x | 1,2,0,1,2,0,... (in order, one day a redraw) | same (x2/x1/x3 queue: 0,1,2,2,2,0,0,1,...) |
| 50x | 2,0,0,1,1,2,2,0,0,1,**0**,2,2,**0**,2,1,0: steps backward | no ▶; lines lit by what ran over the 5 days |
| 200x | 0,1,1,2,0,0,1,1,2,2,0,0,1,2,2,0,0,1: hops | no ▶; all lines lit (a 20-day span) |

**Fix:** `hh_core::view::LinesRun` collects each line's last outcome over a span of days
(display only, not state; the view's per-day `outcomes` use it too). `advance_run` (wasm)
returns it with the state; above 10x the panel shows those outcomes and no ▶, with a note
"N days per redraw". `hh-sim trace --lines` prints each day's lines run, in queue order.

**Checks:** hh-core `programs_run_and_show_their_lines_in_queue_order_day_after_day`
(counts 2/1/3/1, 1 to 3 processors, 20 days: queue order, lines run each day, pointer);
hh-sim `trace_lines_show_the_queue_in_order`; e2e `programOrder` (10x pointer walk, 50x
no pointer and span colors, order kept at both).

Also fixed: the line's progress bar (60px minimum) spilled over the − button from its 48px
grid column.
