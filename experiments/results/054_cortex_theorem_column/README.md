# 054: Cortex mock, the Theorem column centered and its heading enlarged

Two layout fixes to the Grand Theorems column of the Cortex mock (`web/src/ui/CortexMock.tsx`),
both themes. Still a mock with all of 050-053's limits (UI-only, in memory, placeholder powers,
debug toggle).

- **Centered on the core.** The column (heading through the Unified plaque) is centered
  vertically in `.cx-main`, which is where the wheel's core sits (the SVG fills its box and
  centers the wheel). Done with `margin-top: auto` on the first child and `margin-bottom: auto`
  on the last: auto margins drop to zero when the column overflows, so a window too short to
  center it top-aligns instead of clipping both ends. Still the right-hand grid column.
- **Heading in the rim names' style.** Bold, uppercase, letter-spaced, at the rim names'
  on-screen size: the names are 21px with 7px tracking in wheel units, now the tokens
  `--cx-name-size` / `--cx-name-track` on `.cx-root`; a `ResizeObserver` on the tree writes
  `--cx-scale` (screen px per wheel unit) on `.cx-main`, and the heading's size and tracking are
  the tokens times that scale, never below 12px (its old size). It keeps its brass color.
  A left pad equal to the tracking balances the trailing letter-space so it centers.

## Check and screenshots

`shots.mjs` (HH_URL a `vite preview`) opens the view in dark, then light, and checks: the column's
middle within 2px of the core's (518.5 = 518.5 at 1600 × 1000), off the top and right-aligned;
heading size and tracking at least the rim names' on screen (16.4px / 5.5px both at 1600 × 1000;
12px vs 10.9px at 1280 × 720, the floor); at 1280 × 720 not clipped (it still centers there: the
plaques shrink). It saves `cortex-{dark,light}.png` (1600 × 1000) and
`cortex-{dark,light}-1280.png` (1280 × 720), and with `COPY` as `054_*.png` (copied to
`C:\Code\gdrive-sync\helium-hustle\screenshots`). 053's `shots.mjs` still passes on this build.
