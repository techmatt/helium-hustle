# 128: art's Drive home, the unused Rust toolchain

- `move_art.py --do` moved the main checkout's `art/` (502 files, 495.9 MB) with one
  `os.rename` (same volume, so no copy) to `C:\Code\gdrive-sync\helium-hustle\art\`, after
  checking: `art/` a real folder, no link under it, no destination. It then made the junction
  `C:\Code\helium-hustle\art` -> that folder and re-counted through it (same 502 files, same
  bytes). Worktree `art/` links point at the main checkout's path, so they now resolve through
  two junctions.
- Proof a reader still finds its files: 027's `images.py` (reads `art/nav_horizon/` and
  `art/small_icons/`, writes `web/src/assets/`) run from this worktree rewrote its 14 WebP
  files byte-identical to the committed ones (`git status` clean).
- `rustup toolchain uninstall 1.96.0-x86_64-pc-windows-msvc` (968 MB) after checking pins:
  `rustup override list` empty, no `RUSTUP_TOOLCHAIN`, and the only `rust-toolchain.toml`
  files under `C:\Code` (fractal-wallpapers' engine and a copy in fractal-website) set
  components with no `channel`. `stable` (1.100.0) stays, the default.
