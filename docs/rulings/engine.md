# Rulings: engine, numbers and saves

Part of Matt's standing rulings ([index](../rulings.md): what **locked** and **default** mean).
The engine rules that bind every item are in [CLAUDE.md](../../CLAUDE.md), *Engine rules*.

## Engine and numbers

- **Architecture (locked).** `hh-core` holds every rule: no I/O, clock or global RNG, serde
  state, exact replay, native = wasm, a rules revision and a pinned golden game. `hh-sim`,
  `hh-wasm` and `web/` sit on top. `web/` is Vite + TypeScript; libraries and frameworks are
  chosen by fit, with no constraint (Preact since 006). Configs hold every number, with no
  defaults in code.
- **Numbers (locked).** State is fixed point, `Fx(i64)` at 10^-6, with `i128` intermediates and
  one rounding rule (floor; overflow panics). Floats only for factors, through `libm`; platform
  `f64`/`f32` transcendentals are banned in `hh-core`. `Fx` stays opaque so its width can
  change. Counts (buildings owned, ticks) are integers, not `Fx`. Whole powers use exact
  `Fx::pow_int`; `libm` only for fractional powers.
- **Fingerprints pin play, not the config (2026-11-22, 064).** `GameState::fingerprint`
  hashes the state less its config; the config is pinned beside it (`Config::hash`, the
  golden's `GOLDEN_CONFIG_HASH`, as `GOLDEN_CONTENT_HASH` pins content), so a config key that
  changes no play moves no pinned fingerprint.
- **Rules history rows (2026-11-22, docs rule).** A `docs/rules-history.md` row stays under
  about 400 bytes; rows older than the last ten revisions are trimmed to a line.
- **Content files one row per line (2026-11-27, 136).** Every `content/*.json` is an array
  with each row on one line; `python scripts/format_content.py` lays them out, and hh-core's
  `content_files_are_one_row_per_line` test holds them to it.

## Testing

- **Light testing per item (2026-11-25, 092).** An ordinary item is a fast edit with minimal
  testing: the fast lane and the checks covering what it touched, the full run before a commit
  that touches Rust, content or configs. Serious passes (a full `npm run qa`, e2e, balance
  reads, other sweeps) happen in bug hunts, when real problems arise, or when a prompt asks.

## Saves

- **Saves (locked).** A localStorage autosave tagged with the rules revision and config hash,
  discarded on a mismatch. Named scenario starts in the debug menu double as
  sim starting states.
- **Breaking changes.** No saves are live, so a rules change needs no caution: err on the side
  of breaking things, keep save handling minimal, and add no compatibility code for old saves
  until the game is more stable.
- **Save migration (2026-11-21).** An easy migration is fine when it helps development;
  otherwise avoid it (technical debt). It matters once the game is released.
