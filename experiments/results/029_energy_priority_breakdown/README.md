# 029: programs paid first, and shortages made visible

Rules revision 5. The day now runs upkeep-free buildings (Solar Panels), then programs
(paid first), then buildings with upkeep on what is left (Matt's answer to this item's
question: moving programs before the whole building pass would pay them only from
yesterday's stock, which any partial day leaves at 0).

## Shots

`shots.mjs` (headless Edge, 1280 x 800, light and dark, against `vite preview`) starts the
`late_unlocks` scenario, jumps two days and checks what each shot shows; all checks pass.

| Shot | What |
|---|---|
| `shots/*-buildings-1280.png`, `*-card-starved.png` | Research Lab "running at 67% · ⚡ short"; every card's status row 20 px |
| `shots/*-program-lines.png` | Load Launch Pads ran (green); Buy Titanium failed: "needs 💰20, had 0" |
| `shots/*-energy-expanded*.png` | Energy row open: Solar +30, each building, Program 1 −2, who went short |
| `shots/*-stats*.png` | Stats: every resource's breakdown, last day and 20-day average |
| `shots/*-pads.png` | Pad head tints: neutral loading, yellow Paused (P3) |
| `shots/*-land-row.png` | Land row bought by a click on its text (P4) |

QA: one pass, seed 11 at 1280 (light, late panels dark), 0 findings: `qa/contact-11-1280-light.png`,
`qa/report.json`. e2e: ok.

## Balance

`hh-sim balance`, base 704c29f against this branch (`balance-main.txt`, `balance-029.txt`):
casual unchanged; author's M5 Positive Credit Flow now met (was run 9 on 1 of 2 seeds),
run 1 lasts 1520 ticks (was 1380; target 800-1100), 22.8 real minutes (was 21.1), M12/M13
shift between seeds. Every other line as on the base.
