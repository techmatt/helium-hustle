# 053: Cortex mock, contrast first, and a light draft

The third round of the Cortex mock (`web/src/ui/CortexMock.tsx`). Still a mock with all of
050-052's limits: placeholder powers with no effect, UI-only state in module memory, nothing
reaching the engine, sim, save or content, a debug-only toggle.

## What changed

- **A plain ground.** The wafer, die grid, notch, circuit traces, vias and neural threads are
  gone; the view sits on one solid color per theme (dark `#15181d`, light `#efede8`). Kept as
  they were: sector floor tints and rims, glowing bought links with their running pulse, rank
  pips, the core, the rim names, the plaques lighting their sectors. The walls' crenels now
  cut in the ground color.
- **Follows the app's theme.** Every color of the mock is a `--cx-*` token: the light block on
  `.cx-root` is the default (as the app's is), `:root[data-theme='dark'] .cx-root` overrides
  it. Sector colors are tokens too (`--cx-serenity` ...), re-tuned for light: deeper blue,
  amber, green, red and purple (`#2f6bc9 #b86e00 #23864a #c8433a #7b4bc0`) so they reach 3:1
  on the light ground and keep white glyphs readable on a bought node; dark keeps 052's
  colors. Light's bought nodes fill from a lighter tint of the sector color (`--cx-hi-mix`)
  instead of dark's white highlight, with white glyphs; plaques are pale brass with dark ink.
- **One-click switching.** Left-clicking a dim part of an owned split socket, or a dim line of
  Rhetoric's three-part fan, lights it at once, free; the socket stays owned whichever part is
  lit (its rank doesn't change), so nothing beyond it is stranded. Right-click still removes
  the point (refused while something further out depends on it). The tooltip on a dim part
  says "Click to switch ...", in the actionable green; the cost line says switching is free.
- **Bought bridges filled.** A bought bridge's disc is split along the sector boundary, each
  half filled like a bought node of that neighbor (its gradient), with the owned glyph on top.

"Rhetoric's fan" is read as the three-part socket with its three fan lines (the item's
"socket and fan"); the hub's two side vanes and the stem stay independent nodes, all
ownable.

## Contrast

[contrast.md](contrast.md) (from `contrast.py`, reading the token blocks): sector colors
3.4-5.0:1 on light and 7.4-10:1 on dark; open-node bezels 3.1 / 3.6; unbought links 2.3 / 2.4
(dim by design, but visible); locked bezels 1.9 / 1.6 (meant dim); every text pair over 4.5:1
in both themes; owned glyphs 4.0-5.8:1 (light, white on color) and 8-10.8:1 (dark). The
tooltip's sector line mixes the sector color 70% toward the ink in light (`--cx-ink-mix`), so
Logistics' amber reads at 6.3:1 there; dark uses the pure color.

## Check and screenshots

`shots.mjs` (HH_URL a `vite preview`; 126 checks) opens the view in dark, checks a solid ground
with no backdrop art and the no-scroll fit (now measured on the rim names and rim glow, since
the wafer it used is gone), keeps 052's keyboard and refusal regressions, then in dark and
again in light: takes Serenity's crown socket, switches it both ways with one click each
(points unchanged; the socket's removal still refused because Serenity Prime past it depends
on it, the check that it counts as owned whichever part is lit);
takes the fan's Reason line and switches to the other two lines (one fan line lit each time,
Spin past it still owned); buys the Serenity/Logistics bridge and checks its two halves'
fills. It saves `cortex-{dark,light}.png` (the whole view) and `cortex-{dark,light}-tip.png`
(a tooltip on a dim fan part) here, and with `COPY` as `053_*.png` (copied to
`C:\Code\gdrive-sync\helium-hustle\screenshots`).

052's `shots.mjs` no longer runs against this build (it measures `.cx-wafer` and expects the
old "You took" tooltip); this one supersedes it.
